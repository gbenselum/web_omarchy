use anyhow::Context;

/// Session configuration constants
pub const SESSION_TIMEOUT_SECS: u64 = 3600; // 1 hour
pub const SESSION_MAX_AGE_SECS: u64 = 86400; // 24 hours
pub const CLEANUP_INTERVAL_SECS: u64 = 3600; // 1 hour

/// Simple rate limiter for authentication attempts
#[derive(Debug)]
struct RateLimiter {
    attempts: std::collections::HashMap<String, Vec<std::time::Instant>>,
    max_attempts: usize,
    window: std::time::Duration,
}

impl RateLimiter {
    fn new(max_attempts: usize, window_secs: u64) -> Self {
        Self {
            attempts: std::collections::HashMap::new(),
            max_attempts,
            window: std::time::Duration::from_secs(window_secs),
        }
    }

    fn check(&mut self, key: &str) -> bool {
        let now = std::time::Instant::now();
        let attempts = self.attempts.entry(key.to_string()).or_default();
        
        // Remove old attempts outside the window
        attempts.retain(|&t| now.duration_since(t) < self.window);
        
        if attempts.len() >= self.max_attempts {
            false
        } else {
            attempts.push(now);
            true
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Session {
    pub id: String,
    pub user: String,
    pub created_at: u64,
    pub last_activity: u64,
}

impl Session {
    /// Check if session is still valid (not expired)
    pub fn is_valid(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Check idle timeout
        if now.saturating_sub(self.last_activity) > SESSION_TIMEOUT_SECS {
            return false;
        }
        
        // Check max age
        if now.saturating_sub(self.created_at) > SESSION_MAX_AGE_SECS {
            return false;
        }
        
        true
    }
}

pub struct AuthManager {
    session_dir: String,
    sessions: std::sync::Arc<tokio::sync::RwLock<std::collections::HashMap<String, Session>>>,
    rate_limiter: std::sync::Arc<tokio::sync::Mutex<RateLimiter>>,
}

impl AuthManager {
    pub fn new(session_dir: &str) -> anyhow::Result<Self> {
        std::fs::create_dir_all(session_dir)
            .context("Failed to create session directory")?;

        Ok(Self {
            session_dir: session_dir.to_string(),
            sessions: std::sync::Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            rate_limiter: std::sync::Arc::new(tokio::sync::Mutex::new(RateLimiter::new(5, 300))), // 5 attempts per 5 minutes
        })
    }

    pub async fn authenticate(&self, username: &str, password: &str) -> anyhow::Result<Option<Session>> {
        use tracing::{debug, warn};
        
        debug!("Attempting authentication for user: {}", username);
        
        // Rate limiting
        let mut limiter = self.rate_limiter.lock().await;
        let client_key = format!("auth:{}", username);
        if !limiter.check(&client_key) {
            warn!("Rate limit exceeded for user: {}", username);
            return Ok(None);
        }
        drop(limiter);
        
        let output = tokio::process::Command::new("pkexec")
            .args(["/usr/lib/web-omarchy/auth-helper", username, password])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .await
            .context("Failed to execute pkexec")?;

        debug!("pkexec exit code: {:?}", output.status.code());
        debug!("pkexec stdout: {}", String::from_utf8_lossy(&output.stdout));
        debug!("pkexec stderr: {}", String::from_utf8_lossy(&output.stderr));

        match output.status.code() {
            Some(0) => {
                debug!("Authentication successful for user: {}", username);
                let session_id = uuid::Uuid::new_v4().to_string();
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
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

    pub async fn validate_session(&self, session_id: &str) -> anyhow::Result<bool> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id).map(|s| s.is_valid()).unwrap_or(false))
    }

    pub async fn get_session(&self, session_id: &str) -> anyhow::Result<Option<Session>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id).filter(|s| s.is_valid()).cloned())
    }

    pub async fn update_activity(&self, session_id: &str) -> anyhow::Result<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            if !session.is_valid() {
                // Session expired, remove it
                sessions.remove(session_id);
                let session_file = std::path::Path::new(&self.session_dir).join(format!("{}.session", session_id));
                if session_file.exists() {
                    std::fs::remove_file(session_file)?;
                }
                return Ok(());
            }
            session.last_activity = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            self.persist_session(session)?;
        }
        Ok(())
    }

    pub async fn delete_session(&self, session_id: &str) -> anyhow::Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.remove(session_id);
        let session_file = std::path::Path::new(&self.session_dir).join(format!("{}.session", session_id));
        if session_file.exists() {
            std::fs::remove_file(session_file)?;
        }
        Ok(())
    }

    pub async fn load_sessions(&self) -> anyhow::Result<()> {
        let dir = std::path::Path::new(&self.session_dir);
        if !dir.exists() {
            return Ok(());
        }

        let mut sessions = self.sessions.write().await;

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("session") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(session) = serde_json::from_str::<Session>(&content) {
                        sessions.insert(session.id.clone(), session);
                    }
                }
            }
        }

        Ok(())
    }

    fn persist_session(&self, session: &Session) -> anyhow::Result<()> {
        let session_file = std::path::Path::new(&self.session_dir).join(format!("{}.session", session.id));
        let json = serde_json::to_string_pretty(session)?;
        std::fs::write(session_file, json)?;
        Ok(())
    }

    pub async fn cleanup_expired(&self, _max_age_secs: Option<u64>) -> anyhow::Result<()> {
        let mut sessions = self.sessions.write().await;
        let expired: Vec<String> = sessions
            .iter()
            .filter(|(_, s)| !s.is_valid())
            .map(|(k, _)| k.clone())
            .collect();

        for session_id in expired {
            sessions.remove(&session_id);
            let session_file = std::path::Path::new(&self.session_dir).join(format!("{}.session", session_id));
            if session_file.exists() {
                std::fs::remove_file(session_file)?;
            }
        }

        Ok(())
    }
}