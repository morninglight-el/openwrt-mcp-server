// Configuration Module
// This module is responsible for loading and managing server configuration.

use serde::Deserialize;
use std::net::IpAddr;

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
        Self::load_from_path("config.toml")
            .unwrap_or_else(|error| panic!("Failed to load config.toml: {error}"))
    }

    pub fn load_from_path(path: &str) -> Result<Self, String> {
        crate::logging::info("config.load", &[("path", serde_json::json!(path))]);

        let config_content = std::fs::read_to_string(path)
            .map_err(|error| format!("unable to read {path}: {error}"))?;

        Self::from_toml_str(&config_content)
    }

    pub fn from_toml_str(config_content: &str) -> Result<Self, String> {
        let config: Self =
            toml::from_str(config_content).map_err(|error| format!("invalid TOML: {error}"))?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), String> {
        validate_mqtt(&self.mqtt)?;
        validate_http(&self.http)?;
        Ok(())
    }
}

fn validate_mqtt(config: &MqttConfig) -> Result<(), String> {
    validate_non_empty("mqtt.broker", &config.broker)?;
    validate_non_empty("mqtt.client_id", &config.client_id)?;
    validate_non_empty("mqtt.topic_prefix", &config.topic_prefix)?;

    if config.topic_prefix.contains('#') || config.topic_prefix.contains('+') {
        return Err("mqtt.topic_prefix must not contain MQTT wildcard characters".to_string());
    }

    Ok(())
}

fn validate_http(config: &HttpConfig) -> Result<(), String> {
    config
        .listen_addr
        .parse::<IpAddr>()
        .map_err(|error| format!("http.listen_addr is invalid: {error}"))?;

    if config.port == 0 {
        return Err("http.port must be between 1 and 65535".to_string());
    }

    if config.enable {
        validate_non_empty("http.token", &config.token)?;
    }

    Ok(())
}

fn validate_non_empty(field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{field} must not be empty"))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Config;

    const VALID_CONFIG: &str = r#"
[mqtt]
broker = "mqtt://localhost:1883"
client_id = "openwrt-one"
username = ""
password = ""
topic_prefix = "mcp/device/openwrt-one"

[http]
enable = true
listen_addr = "127.0.0.1"
port = 8080
token = "test-token"
"#;

    #[test]
    fn parses_and_validates_config() {
        let config = Config::from_toml_str(VALID_CONFIG).unwrap();

        assert_eq!(config.mqtt.client_id, "openwrt-one");
        assert_eq!(config.http.port, 8080);
    }

    #[test]
    fn rejects_invalid_http_address() {
        let config = VALID_CONFIG.replace("127.0.0.1", "not-an-ip");

        assert!(Config::from_toml_str(&config)
            .unwrap_err()
            .contains("http.listen_addr"));
    }

    #[test]
    fn rejects_mqtt_topic_wildcards() {
        let config = VALID_CONFIG.replace("mcp/device/openwrt-one", "mcp/device/+");

        assert!(Config::from_toml_str(&config)
            .unwrap_err()
            .contains("wildcard"));
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
