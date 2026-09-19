use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::process::Command;
use tokio::sync::RwLock;
use tracing::{debug, error, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user: String,
    pub created_at: u64,
    pub last_activity: u64,
}

pub struct AuthManager {
    session_dir: String,
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl AuthManager {
    pub fn new(session_dir: &str) -> Result<Self> {
        fs::create_dir_all(session_dir)
            .context("Failed to create session directory")?;

        Ok(Self {
            session_dir: session_dir.to_string(),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn authenticate(&self, username: &str, password: &str) -> Result<Option<Session>> {
        debug!("Attempting authentication for user: {}", username);
        
        // TEST BYPASS: Allow any credentials if TEST_AUTH_BYPASS env var is set
        if std::env::var("TEST_AUTH_BYPASS").is_ok() {
            warn!("TEST_AUTH_BYPASS enabled - allowing any credentials for user: {}", username);
            let session_id = Uuid::new_v4().to_string();
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let session = Session {
                id: session_id.clone(),
                user: username.to_string(),
                created_at: now,
                last_activity: now,
            };
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id.clone(), session.clone());
            self.persist_session(&session)?;
            return Ok(Some(session));
        }
        
        let output = Command::new("pkexec")
            .args(["/usr/lib/web-omarchy/auth-helper", username, password])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .context("Failed to execute pkexec")?;

        debug!("pkexec exit code: {:?}", output.status.code());
        debug!("pkexec stdout: {}", String::from_utf8_lossy(&output.stdout));
        debug!("pkexec stderr: {}", String::from_utf8_lossy(&output.stderr));

        match output.status.code() {
            Some(0) => {
                debug!("Authentication successful for user: {}", username);
                let session_id = Uuid::new_v4().to_string();
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                let session = Session {
                    id: session_id.clone(),
                    user: username.to_string(),
                    created_at: now,
                    last_activity: now,
                };

                let mut sessions = self.sessions.write().await;
                sessions.insert(session_id.clone(), session.clone());

                self.persist_session(&session)?;

                Ok(Some(session))
            }
            _ => {
                warn!("Authentication failed for user: {} (exit code: {:?})", username, output.status.code());
                warn!("stderr: {}", String::from_utf8_lossy(&output.stderr));
                Ok(None)
            }
        }
    }

    pub async fn validate_session(&self, session_id: &str) -> Result<bool> {
        let sessions = self.sessions.read().await;
        Ok(sessions.contains_key(session_id))
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Option<Session>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id).cloned())
    }

    pub async fn update_activity(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.last_activity = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            self.persist_session(session)?;
        }
        Ok(())
    }

    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
        let session_file = Path::new(&self.session_dir).join(format!("{}.session", session_id));
        if session_file.exists() {
            fs::remove_file(session_file)?;
        }
        Ok(())
    }

    pub async fn load_sessions(&self) -> Result<()> {
        let dir = Path::new(&self.session_dir);
        if !dir.exists() {
            return Ok(());
        }

        let mut sessions = self.sessions.write().await;

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("session") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<Session>(&content) {
                        sessions.insert(session.id.clone(), session);
                    }
                }
            }
        }

        Ok(())
    }

    fn persist_session(&self, session: &Session) -> Result<()> {
        let session_file = Path::new(&self.session_dir).join(format!("{}.session", session.id));
        let json = serde_json::to_string_pretty(session)?;
        fs::write(session_file, json)?;
        Ok(())
    }

    pub async fn cleanup_expired(&self, max_age_secs: u64) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut sessions = self.sessions.write().await;
        let expired: Vec<String> = sessions
            .iter()
            .filter(|(_, s)| now.saturating_sub(s.last_activity) > max_age_secs)
            .map(|(k, _)| k.clone())
            .collect();

        for session_id in expired {
            sessions.remove(&session_id);
            let session_file = Path::new(&self.session_dir).join(format!("{}.session", session_id));
            if session_file.exists() {
                fs::remove_file(session_file)?;
            }
        }

        Ok(())
    }
}