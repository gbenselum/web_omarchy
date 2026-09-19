use anyhow::{Context, Result};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;

pub struct CommandBridge {
    daemon_user: String,
}

impl CommandBridge {
    pub fn new(daemon_user: String) -> Self {
        Self { daemon_user }
    }

    pub async fn execute(
        &self,
        command: &str,
        args: &[String],
        env: Option<Vec<(String, String)>>,
    ) -> Result<CommandOutput> {
        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(env_vars) = env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        cmd.env("PATH", "/usr/local/bin:/usr/bin:/bin");

        let mut child = cmd.spawn().context("Failed to spawn command")?;

        let stdout = child.stdout.take().context("Failed to capture stdout")?;
        let stderr = child.stderr.take().context("Failed to capture stderr")?;

        let mut stdout_reader = BufReader::new(stdout);
        let mut stderr_reader = BufReader::new(stderr);

        let mut stdout_lines = Vec::new();
        let mut stderr_lines = Vec::new();

        let mut stdout_buf = String::new();
        let mut stderr_buf = String::new();

        loop {
            tokio::select! {
                result = stdout_reader.read_line(&mut stdout_buf) => {
                    match result {
                        Ok(0) => break,
                        Ok(_) => {
                            stdout_lines.push(stdout_buf.clone());
                            stdout_buf.clear();
                        }
                        Err(e) => {
                            eprintln!("Stdout read error: {}", e);
                            break;
                        }
                    }
                }
                result = stderr_reader.read_line(&mut stderr_buf) => {
                    match result {
                        Ok(0) => break,
                        Ok(_) => {
                            stderr_lines.push(stderr_buf.clone());
                            stderr_buf.clear();
                        }
                        Err(e) => {
                            eprintln!("Stderr read error: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        let status = child.wait().await.context("Failed to wait for child")?;
        let exit_code = status.code().unwrap_or(-1);

        Ok(CommandOutput {
            stdout: stdout_lines.join(""),
            stderr: stderr_lines.join(""),
            exit_code,
        })
    }

    pub async fn execute_streaming(
        &self,
        command: &str,
        args: &[String],
        env: Option<Vec<(String, String)>>,
        on_stdout: Box<dyn FnMut(&str) + Send>,
        on_stderr: Box<dyn FnMut(&str) + Send>,
    ) -> Result<i32> {
        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(env_vars) = env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        cmd.env("PATH", "/usr/local/bin:/usr/bin:/bin");

        let mut child = cmd.spawn().context("Failed to spawn command")?;

        let stdout = child.stdout.take().context("Failed to capture stdout")?;
        let stderr = child.stderr.take().context("Failed to capture stderr")?;

        let mut stdout_reader = BufReader::new(stdout);
        let mut stderr_reader = BufReader::new(stderr);

        let mut stdout_buf = String::new();
        let mut stderr_buf = String::new();

        let mut on_stdout = on_stdout;
        let mut on_stderr = on_stderr;

        loop {
            tokio::select! {
                result = stdout_reader.read_line(&mut stdout_buf) => {
                    match result {
                        Ok(0) => break,
                        Ok(_) => {
                            on_stdout(&stdout_buf);
                            stdout_buf.clear();
                        }
                        Err(_) => break,
                    }
                }
                result = stderr_reader.read_line(&mut stderr_buf) => {
                    match result {
                        Ok(0) => break,
                        Ok(_) => {
                            on_stderr(&stderr_buf);
                            stderr_buf.clear();
                        }
                        Err(_) => break,
                    }
                }
            }
        }

        let status = child.wait().await.context("Failed to wait for child")?;
        Ok(status.code().unwrap_or(-1))
    }

    pub fn find_omarchy_command(&self, name: &str) -> Option<String> {
        let paths = [
            "/usr/bin",
            "/usr/local/bin",
            &format!("/home/{}/.local/bin", self.daemon_user),
        ];

        for path in paths {
            let full_path = format!("{}/{}", path, name);
            if std::path::Path::new(&full_path).exists() {
                return Some(full_path);
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl CommandBridge {
    pub async fn read_file(&self, path: &str) -> Result<String> {
        tokio::fs::read_to_string(path).await
            .context("Failed to read file")
    }

    pub async fn write_file(&self, path: &str, content: &str) -> Result<()> {
        tokio::fs::write(path, content).await
            .context("Failed to write file")
    }
}