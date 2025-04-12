// Model Types Module
// This module defines the data structures used throughout the server.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ContextReport {
    pub device_id: String,
    pub uptime: u64,
    pub cpu_load: Vec<f32>,
    pub interfaces: serde_json::Value,
    pub wifi_clients: u32,
    pub schema_version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommandRequest {
    pub command: String,
    pub args: serde_json::Value,
    pub schema_version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommandResult {
    pub command_id: String,
    pub status: String,
    pub output: String,
    pub timestamp: String,
}
