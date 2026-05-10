// Configuration Module
// This module is responsible for loading and managing server configuration.

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct MqttConfig {
    pub broker: String,
    pub client_id: String,
    pub username: String,
    pub password: String,
    pub topic_prefix: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct HttpConfig {
    pub enable: bool,
    pub listen_addr: String,
    pub port: u16,
    pub token: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub mqtt: MqttConfig,
    pub http: HttpConfig,
}

impl Config {
    /// Load configuration from config.toml in the current directory.
    pub fn load() -> Self {
        use std::fs;

        println!("Loading configuration...");

        let config_content = fs::read_to_string("config.toml").expect("Failed to read config.toml");

        toml::from_str(&config_content).expect("Failed to parse config.toml")
    }
}

// Example config.toml:
//
// [mqtt]
// broker = "mqtts://iot.example.com:8883"
// client_id = "openwrt-one"
// username = "mcp-user"
// password = "mcp-pass"
// topic_prefix = "mcp/device/openwrt-one"
//
// [http]
// enable = true
// listen_addr = "0.0.0.0"
// port = 8080
// token = "your-api-token"
