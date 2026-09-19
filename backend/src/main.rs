use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, State,
    },
    http::{header, HeaderMap, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
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
use auth::{AuthManager, CLEANUP_INTERVAL_SECS};

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

    // Create AuthManager for session cleanup
    let auth_manager = Arc::new(AuthManager::new(&args.session_dir)?);
    
    // Spawn session cleanup task (runs every hour)
    let auth_cleanup = auth_manager.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(CLEANUP_INTERVAL_SECS));
        loop {
            interval.tick().await;
            if let Err(e) = auth_cleanup.cleanup_expired(None).await {
                error!("Session cleanup failed: {}", e);
            }
        }
    });

    let bridge = Arc::new(CommandBridge::new(args.daemon_user.clone()));
    let router = Arc::new(MessageRouter::new(bridge.clone()));
    let (tx, _rx) = broadcast::channel(1024);

    let state = AppState { router, tx };

    let www_dir = PathBuf::from(&args.www_dir);

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .fallback_service(ServeDir::new(www_dir).append_index_html_on_directories(true))
        .layer(middleware::from_fn(security_headers))
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
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    // Validate Origin header
    let allowed_origins = [
        "http://localhost:8080",
        "http://127.0.0.1:8080",
        "https://localhost:8080",
    ];
    
    if let Some(origin) = headers.get(header::ORIGIN) {
        let origin_str = origin.to_str().unwrap_or("");
        if !allowed_origins.iter().any(|&o| origin_str.starts_with(o)) {
            warn!("Rejected WebSocket connection from unauthorized origin: {}", origin_str);
            return axum::response::Response::builder()
                .status(axum::http::StatusCode::FORBIDDEN)
                .body(axum::body::Body::empty())
                .unwrap()
                .into_response();
        }
    } else {
        warn!("Rejected WebSocket connection from {}: missing Origin header", addr);
        return axum::response::Response::builder()
            .status(axum::http::StatusCode::FORBIDDEN)
            .body(axum::body::Body::empty())
            .unwrap()
            .into_response();
    }
    
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

/// Security headers middleware
async fn security_headers(req: axum::http::Request<axum::body::Body>, next: Next) -> Response {
    let mut response = next.run(req).await;
    
    let headers = response.headers_mut();
    
    // Content Security Policy
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'self'; \
             script-src 'self' 'unsafe-inline'; \
             style-src 'self' 'unsafe-inline'; \
             img-src 'self' data:; \
             font-src 'self'; \
             connect-src 'self' ws: wss:; \
             frame-ancestors 'none'; \
             base-uri 'self'; \
             form-action 'self'"
        ),
    );
    
    // Prevent MIME type sniffing
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    
    // Prevent clickjacking
    headers.insert(
        header::X_FRAME_OPTIONS,
        HeaderValue::from_static("DENY"),
    );
    
    // XSS protection
    headers.insert(
        header::X_XSS_PROTECTION,
        HeaderValue::from_static("1; mode=block"),
    );
    
    // Referrer policy
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );
    
    // Permissions policy
    headers.insert(
        "Permissions-Policy",
        HeaderValue::from_static("geolocation=(), microphone=(), camera=()"),
    );
    
    // HSTS (only if using HTTPS)
    // headers.insert(
    //     header::STRICT_TRANSPORT_SECURITY,
    //     HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    // );
    
    response
}