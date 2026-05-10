// Model Types Module
// This module defines the data structures used throughout the server.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ContextReport {
    pub device_id: String,
    pub uptime: u64,
    pub cpu_load: Vec<f32>,
    pub interfaces: serde_json::Value,
    pub wifi_clients: u32,
    pub schema_version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CommandRequest {
    pub command: String,
    #[serde(default = "default_args")]
    pub args: serde_json::Value,
    #[serde(default = "default_schema_version")]
    pub schema_version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CommandResult {
    pub command_id: String,
    pub status: String,
    pub output: String,
    pub timestamp: String,
}

fn default_schema_version() -> String {
    "1.0.0".to_string()
}

fn default_args() -> serde_json::Value {
    serde_json::json!({})
}

#[cfg(test)]
mod tests {
    use super::{CommandRequest, CommandResult};
    use serde_json::json;

    #[test]
    fn command_request_defaults_optional_fields() {
        let request: CommandRequest = serde_json::from_value(json!({
            "command": "get_context"
        }))
        .unwrap();

        assert_eq!(request.command, "get_context");
        assert_eq!(request.args, json!({}));
        assert_eq!(request.schema_version, "1.0.0");
    }

    #[test]
    fn command_result_serializes_status_payload() {
        let result = CommandResult {
            command_id: "cmd-1".to_string(),
            status: "ok".to_string(),
            output: "done".to_string(),
            timestamp: "1970-01-01T00:00:00Z".to_string(),
        };

        let value = serde_json::to_value(result).unwrap();

        assert_eq!(value["command_id"], "cmd-1");
        assert_eq!(value["status"], "ok");
    }
}
