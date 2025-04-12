// Configuration Module
// This module is responsible for loading and managing server configuration.

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub mqtt_broker: String,
    pub http_port: u16,
}

impl Config {
    pub fn load() -> Self {
        use std::fs;

        println!("Loading configuration...");

        let config_content = fs::read_to_string("config.toml")
            .expect("Failed to read config.toml");

        toml::from_str(&config_content)
            .expect("Failed to parse config.toml")
    }
}
