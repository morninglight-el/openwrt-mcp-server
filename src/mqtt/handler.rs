// MQTT Handler Module
// This module is responsible for handling MQTT communication, including subscribing to topics and processing messages.

use rumqttc::{AsyncClient, MqttOptions, QoS};
use crate::config::Config;
use tokio::time::Duration;

pub async fn initialize_mqtt(config: &Config) -> AsyncClient {
    println!("Initializing MQTT...");

    let mut mqttoptions = MqttOptions::new("openwrt-mcp-server", &config.mqtt_broker, 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(30));

    let (client, eventloop) = AsyncClient::new(mqttoptions, 10);

    // Spawn a task to handle the MQTT event loop
    tokio::spawn(async move {
        let mut eventloop = eventloop;
        while let Ok(event) = eventloop.poll().await {
            println!("MQTT Event: {:?}", event);
        }
    });

    // Example: Subscribe to a topic
    client
        .subscribe("mcp/device/{device_id}/cmd", QoS::AtMostOnce)
        .await
        .expect("Failed to subscribe to command topic");

    println!("MQTT client initialized and subscribed to command topic.");

    client
}
