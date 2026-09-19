use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum ClientMessage {
    #[serde(rename = "auth.login")]
    AuthLogin {
        username: String,
        password: String,
    },
    #[serde(rename = "auth.logout")]
    AuthLogout {
        session_id: String,
    },
    #[serde(rename = "execute")]
    Execute {
        channel_id: String,
        command: String,
        args: Vec<String>,
        env: Option<Vec<(String, String)>>,
    },
    #[serde(rename = "read_file")]
    ReadFile {
        channel_id: String,
        path: String,
    },
    #[serde(rename = "write_file")]
    WriteFile {
        channel_id: String,
        path: String,
        content: String,
    },
    #[serde(rename = "dbus_call")]
    DbusCall {
        channel_id: String,
        destination: String,
        path: String,
        interface: String,
        method: String,
        args: Vec<serde_json::Value>,
    },
    #[serde(rename = "plugin.list")]
    PluginList {
        channel_id: String,
    },
    #[serde(rename = "plugin.enable")]
    PluginEnable {
        channel_id: String,
        plugin_id: String,
    },
    #[serde(rename = "plugin.disable")]
    PluginDisable {
        channel_id: String,
        plugin_id: String,
    },
    #[serde(rename = "pty.spawn")]
    PtySpawn {
        channel_id: String,
        cols: u16,
        rows: u16,
        env: Option<Vec<(String, String)>>,
    },
    #[serde(rename = "pty.resize")]
    PtyResize {
        channel_id: String,
        cols: u16,
        rows: u16,
    },
    #[serde(rename = "pty.stdin")]
    PtyStdin {
        channel_id: String,
        data: String,
    },
    #[serde(rename = "systemd.list_units")]
    SystemdListUnits {
        channel_id: String,
    },
    #[serde(rename = "systemd.control")]
    SystemdControl {
        channel_id: String,
        unit: String,
        control_action: String,
    },
    #[serde(rename = "systemd.logs")]
    SystemdLogs {
        channel_id: String,
        unit: String,
        lines: Option<u32>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "auth.response")]
    AuthResponse {
        success: bool,
        session_id: Option<String>,
        user: Option<String>,
        error: Option<String>,
    },
    #[serde(rename = "stream")]
    Stream(StreamResponse),
    #[serde(rename = "plugin.list")]
    PluginListResponse {
        channel_id: String,
        plugins: Vec<PluginInfo>,
    },
    #[serde(rename = "systemd.units")]
    SystemdUnitsResponse {
        channel_id: String,
        units: Vec<UnitInfo>,
    },
    #[serde(rename = "systemd.logs")]
    SystemdLogsResponse {
        channel_id: String,
        logs: String,
    },
    #[serde(rename = "error")]
    ErrorResponse {
        channel_id: String,
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamResponse {
    pub channel_id: String,
    #[serde(rename = "type")]
    pub stream_type: StreamType,
    pub data: StreamData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StreamType {
    Data,
    Error,
    Control,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StreamData {
    #[serde(rename = "stdout")]
    Stdout { stdout: String },
    #[serde(rename = "stderr")]
    Stderr { stderr: String },
    #[serde(rename = "control")]
    Control { status: String, exit_code: Option<i32> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitInfo {
    pub name: String,
    pub description: String,
    pub active_state: String,
    pub sub_state: String,
    pub load_state: String,
}

impl ClientMessage {
    pub fn channel_id(&self) -> Option<&str> {
        match self {
            ClientMessage::Execute { channel_id, .. } => Some(channel_id),
            ClientMessage::ReadFile { channel_id, .. } => Some(channel_id),
            ClientMessage::WriteFile { channel_id, .. } => Some(channel_id),
            ClientMessage::DbusCall { channel_id, .. } => Some(channel_id),
            ClientMessage::PluginList { channel_id } => Some(channel_id),
            ClientMessage::PluginEnable { channel_id, .. } => Some(channel_id),
            ClientMessage::PluginDisable { channel_id, .. } => Some(channel_id),
            ClientMessage::PtySpawn { channel_id, .. } => Some(channel_id),
            ClientMessage::PtyResize { channel_id, .. } => Some(channel_id),
            ClientMessage::PtyStdin { channel_id, .. } => Some(channel_id),
            ClientMessage::SystemdListUnits { channel_id } => Some(channel_id),
            ClientMessage::SystemdControl { channel_id, .. } => Some(channel_id),
            ClientMessage::SystemdLogs { channel_id, .. } => Some(channel_id),
            _ => None,
        }
    }
}

impl ServerMessage {
    pub fn error(channel_id: &str, message: String) -> Self {
        ServerMessage::ErrorResponse {
            channel_id: channel_id.to_string(),
            message,
        }
    }

    pub fn stdout(channel_id: &str, stdout: String) -> Self {
        ServerMessage::Stream(StreamResponse {
            channel_id: channel_id.to_string(),
            stream_type: StreamType::Data,
            data: StreamData::Stdout { stdout },
        })
    }

    pub fn stderr(channel_id: &str, stderr: String) -> Self {
        ServerMessage::Stream(StreamResponse {
            channel_id: channel_id.to_string(),
            stream_type: StreamType::Error,
            data: StreamData::Stderr { stderr },
        })
    }

    pub fn completed(channel_id: &str, exit_code: i32) -> Self {
        ServerMessage::Stream(StreamResponse {
            channel_id: channel_id.to_string(),
            stream_type: StreamType::Control,
            data: StreamData::Control {
                status: "completed".to_string(),
                exit_code: Some(exit_code),
            },
        })
    }
}