use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, warn};

use crate::auth::AuthManager;
use crate::bridge::CommandBridge;
use crate::protocol::{ClientMessage, ServerMessage};

pub struct MessageRouter {
    bridge: Arc<CommandBridge>,
}

impl MessageRouter {
    pub fn new(bridge: Arc<CommandBridge>) -> Self {
        Self { bridge }
    }

    pub async fn route(
        &self,
        msg: ClientMessage,
        tx: mpsc::UnboundedSender<ServerMessage>,
    ) -> Result<()> {
        debug!("Routing message: {:?}", msg);

        match msg {
            ClientMessage::AuthLogin { username, password } => {
                self.handle_auth_login(&username, &password, tx).await?;
            }
            ClientMessage::AuthLogout { session_id } => {
                self.handle_auth_logout(&session_id, tx).await?;
            }
            ClientMessage::Execute { channel_id, command, args, env } => {
                self.handle_execute(channel_id, &command, &args, env, tx).await?;
            }
            ClientMessage::ReadFile { channel_id, path } => {
                self.handle_read_file(channel_id, &path, tx).await?;
            }
            ClientMessage::WriteFile { channel_id, path, content } => {
                self.handle_write_file(channel_id, &path, &content, tx).await?;
            }
            ClientMessage::PluginList { channel_id } => {
                self.handle_plugin_list(channel_id, tx).await?;
            }
            ClientMessage::SystemdListUnits { channel_id } => {
                self.handle_systemd_list_units(channel_id, tx).await?;
            }
            ClientMessage::SystemdControl { channel_id, unit, control_action } => {
                self.handle_systemd_control(channel_id, &unit, &control_action, tx).await?;
            }
            ClientMessage::SystemdLogs { channel_id, unit, lines } => {
                self.handle_systemd_logs(channel_id, &unit, lines, tx).await?;
            }
            _ => {
                warn!("Unhandled message type");
                let _ = tx.send(ServerMessage::error(
                    msg.channel_id().unwrap_or("unknown"),
                    "Unsupported action".to_string(),
                ));
            }
        }

        Ok(())
    }

    async fn handle_auth_login(
        &self,
        username: &str,
        password: &str,
        tx: mpsc::UnboundedSender<ServerMessage>,
    ) -> Result<()> {
        let auth = AuthManager::new("/var/lib/web-omarchy/sessions")?;
        let session = auth.authenticate(username, password).await?;

        if let Some(session) = session {
            let _ = tx.send(ServerMessage::AuthResponse {
                success: true,
                session_id: Some(session.id),
                user: Some(session.user),
                error: None,
            });
        } else {
            let _ = tx.send(ServerMessage::AuthResponse {
                success: false,
                session_id: None,
                user: None,
                error: Some("Invalid credentials".to_string()),
            });
        }
        Ok(())
    }

    async fn handle_auth_logout(
        &self,
        session_id: &str,
        _tx: mpsc::UnboundedSender<ServerMessage>,
    ) -> Result<()> {
        let auth = AuthManager::new("/var/lib/web-omarchy/sessions")?;
        auth.delete_session(session_id).await?;
        Ok(())
    }

    async fn handle_execute(
        &self,
        channel_id: String,
        command: &str,
        args: &[String],
        env: Option<Vec<(String, String)>>,
        tx: mpsc::UnboundedSender<ServerMessage>,
    ) -> Result<()> {
        let bridge = self.bridge.clone();
        let tx_clone = tx.clone();
        let channel_id_clone = channel_id.clone();
        let args_owned = args.to_vec();
        let env_owned = env;

        let command_path = bridge.find_omarchy_command(command).unwrap_or_else(|| command.to_string());

        tokio::spawn(async move {
            let tx_stdout = tx_clone.clone();
            let tx_stderr = tx_clone.clone();
            let channel_id_stdout = channel_id_clone.clone();
            let channel_id_stderr = channel_id_clone.clone();

            let result = bridge.execute_streaming(
                &command_path,
                &args_owned,
                env_owned,
                Box::new(move |line: &str| {
                    let _ = tx_stdout.send(ServerMessage::stdout(&channel_id_stdout, line.to_string()));
                }),
                Box::new(move |line: &str| {
                    let _ = tx_stderr.send(ServerMessage::stderr(&channel_id_stderr, line.to_string()));
                }),
            ).await;

            match result {
                Ok(exit_code) => {
                    let _ = tx_clone.send(ServerMessage::completed(&channel_id_clone, exit_code));
                }
                Err(e) => {
                    let _ = tx_clone.send(ServerMessage::error(&channel_id_clone, format!("Execution failed: {}", e)));
                }
            }
        });

        Ok(())
    }

    async fn handle_read_file(
        &self,
        channel_id: String,
        path: &str,
        tx: mpsc::UnboundedSender<ServerMessage>,
    ) -> Result<()> {
        let content = self.bridge.read_file(path).await?;
        let _ = tx.send(ServerMessage::stdout(&channel_id, content));
        let _ = tx.send(ServerMessage::completed(&channel_id, 0));
        Ok(())
    }

    async fn handle_write_file(
        &self,
        channel_id: String,
        path: &str,
        content: &str,
        tx: mpsc::UnboundedSender<ServerMessage>,
    ) -> Result<()> {
        self.bridge.write_file(path, content).await?;
        let _ = tx.send(ServerMessage::stdout(&channel_id, "File written successfully".to_string()));
        let _ = tx.send(ServerMessage::completed(&channel_id, 0));
        Ok(())
    }

    async fn handle_plugin_list(
        &self,
        channel_id: String,
        tx: mpsc::UnboundedSender<ServerMessage>,
    ) -> Result<()> {
        let plugins = vec![
            crate::protocol::PluginInfo {
                id: "networking".to_string(),
                name: "Networking".to_string(),
                version: "0.1.0".to_string(),
                description: "Network interface management".to_string(),
                enabled: true,
                author: Some("Omarchy Team".to_string()),
            },
            crate::protocol::PluginInfo {
                id: "storage".to_string(),
                name: "Storage".to_string(),
                version: "0.1.0".to_string(),
                description: "Disk and partition management".to_string(),
                enabled: true,
                author: Some("Omarchy Team".to_string()),
            },
            crate::protocol::PluginInfo {
                id: "btrfs".to_string(),
                name: "Btrfs Snapshots".to_string(),
                version: "0.1.0".to_string(),
                description: "Btrfs snapshot management".to_string(),
                enabled: true,
                author: Some("Omarchy Team".to_string()),
            },
            crate::protocol::PluginInfo {
                id: "updates".to_string(),
                name: "System Updates".to_string(),
                version: "0.1.0".to_string(),
                description: "Package update management".to_string(),
                enabled: true,
                author: Some("Omarchy Team".to_string()),
            },
            crate::protocol::PluginInfo {
                id: "terminal".to_string(),
                name: "Terminal".to_string(),
                version: "0.1.0".to_string(),
                description: "Web-based terminal emulator".to_string(),
                enabled: true,
                author: Some("Omarchy Team".to_string()),
            },
        ];

        let _ = tx.send(ServerMessage::PluginListResponse {
            channel_id,
            plugins,
        });

        Ok(())
    }

    async fn handle_systemd_list_units(
        &self,
        channel_id: String,
        tx: mpsc::UnboundedSender<ServerMessage>,
    ) -> Result<()> {
        let output = self.bridge.execute("systemctl", &["list-units".to_string(), "--no-legend".to_string(), "--type=service".to_string()], None).await?;

        let units: Vec<crate::protocol::UnitInfo> = output.stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    Some(crate::protocol::UnitInfo {
                        name: parts[0].to_string(),
                        description: parts[3..].join(" "),
                        active_state: parts[1].to_string(),
                        sub_state: parts[2].to_string(),
                        load_state: "loaded".to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();

        let _ = tx.send(ServerMessage::SystemdUnitsResponse {
            channel_id,
            units,
        });

        Ok(())
    }

    async fn handle_systemd_control(
        &self,
        channel_id: String,
        unit: &str,
        action: &str,
        tx: mpsc::UnboundedSender<ServerMessage>,
    ) -> Result<()> {
        let valid_actions = ["start", "stop", "restart", "enable", "disable"];
        if !valid_actions.contains(&action) {
            let _ = tx.send(ServerMessage::error(&channel_id, format!("Invalid action: {}", action)));
            return Ok(());
        }

        let output = self.bridge.execute("systemctl", &[action.to_string(), unit.to_string()], None).await?;

        if output.exit_code == 0 {
            let _ = tx.send(ServerMessage::stdout(&channel_id, format!("{} {} completed", unit, action)));
        } else {
            let _ = tx.send(ServerMessage::stderr(&channel_id, output.stderr));
        }
        let _ = tx.send(ServerMessage::completed(&channel_id, output.exit_code));

        Ok(())
    }

    async fn handle_systemd_logs(
        &self,
        channel_id: String,
        unit: &str,
        lines: Option<u32>,
        tx: mpsc::UnboundedSender<ServerMessage>,
    ) -> Result<()> {
        let mut args = vec!["journalctl".to_string(), "-u".to_string(), unit.to_string(), "--no-pager".to_string()];
        if let Some(n) = lines {
            args.push("-n".to_string());
            args.push(n.to_string());
        }

        let output = self.bridge.execute("journalctl", &args, None).await?;

        let _ = tx.send(ServerMessage::SystemdLogsResponse {
            channel_id,
            logs: output.stdout,
        });

        Ok(())
    }
}