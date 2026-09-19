use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use signal_hook::consts::{SIGINT, SIGTERM};
use signal_hook_tokio::Signals;
use tokio::net::TcpListener;
use tokio::sync::{broadcast, mpsc, Mutex};
use tower_http::services::ServeDir;
use tracing::{error, info, warn};

mod auth;
mod bridge;
mod protocol;
mod router;

use bridge::CommandBridge;
use protocol::{ClientMessage, ServerMessage};
use router::MessageRouter;

#[derive(Parser, Debug)]
#[command(name = "web-omarchy-daemon", version, about = "Omarchy Web Management Interface Daemon")]
struct Args {
    #[arg(short, long, default_value = "0.0.0.0:8080")]
    bind: SocketAddr,

    #[arg(long, default_value = "/var/lib/web-omarchy/sessions")]
    session_dir: String,

    #[arg(long, default_value = "omarchy-web")]
    daemon_user: String,

    #[arg(long, default_value = "/usr/share/web-omarchy/www")]
    www_dir: String,
}

#[derive(Clone)]
struct AppState {
    router: Arc<MessageRouter>,
    tx: broadcast::Sender<ServerMessage>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    let bridge = Arc::new(CommandBridge::new(args.daemon_user.clone()));
    let router = Arc::new(MessageRouter::new(bridge.clone()));
    let (tx, _rx) = broadcast::channel(1024);

    let state = AppState { router, tx };

    let www_dir = PathBuf::from(&args.www_dir);

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .fallback_service(ServeDir::new(www_dir).append_index_html_on_directories(true))
        .with_state(state.clone());

    let listener = TcpListener::bind(args.bind).await
        .context("Failed to bind TCP listener")?;

    info!("Web Omarchy Daemon listening on {}", args.bind);

    let mut signals = Signals::new([SIGINT, SIGTERM])?;
    let state_clone = state.clone();

    tokio::spawn(async move {
        while let Some(signal) = signals.next().await {
            match signal {
                SIGINT | SIGTERM => {
                    info!("Received shutdown signal, stopping gracefully...");
                    state_clone.shutdown().await;
                    std::process::exit(0);
                }
                _ => {}
            }
        }
    });

    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .context("Server error")?;

    Ok(())
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws(socket, addr, state))
}

async fn handle_ws(socket: WebSocket, addr: SocketAddr, state: AppState) {
    let (ws_sender, mut ws_receiver) = socket.split();
    let ws_sender = Arc::new(Mutex::new(ws_sender));

    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<ServerMessage>();

    let mut rx = state.tx.subscribe();
    let ws_sender_clone = ws_sender.clone();

    let send_task = tokio::spawn(async move {
        while let Some(msg) = cmd_rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            let mut sender = ws_sender_clone.lock().await;
            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    let ws_sender_clone = ws_sender.clone();
    let broadcast_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let json = serde_json::to_string(&msg).unwrap();
            let mut sender = ws_sender_clone.lock().await;
            if sender.send(Message::Text(json)).await.is_err() {
                break;
            }
        }
    });

    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                    let state = state.clone();
                    let cmd_tx = cmd_tx.clone();
                    tokio::spawn(async move {
                        if let Err(e) = state.router.route(client_msg, cmd_tx).await {
                            error!("Routing error: {}", e);
                        }
                    });
                } else {
                    warn!("Invalid message format from {}: {}", addr, text);
                }
            }
            Ok(Message::Close(_)) => break,
            Err(e) => {
                error!("WebSocket error from {}: {}", addr, e);
                break;
            }
            _ => {}
        }
    }

    send_task.abort();
    broadcast_task.abort();
}

impl AppState {
    async fn shutdown(&self) {
        info!("Shutting down...");
    }
}