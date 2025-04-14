/*
 * MQTT Handler Module
 * This module is responsible for handling MQTT communication, including subscribing to topics and processing messages.
 */

use rumqttc::{AsyncClient, MqttOptions, QoS};
use crate::config::Config;
use tokio::time::Duration;
use crate::executor::command; // Import the command executor module

pub async fn initialize_mqtt(config: &Config) -> AsyncClient {
    println!("Initializing MQTT...");

    // Initialize MQTT options using the configuration struct fields
    let mut mqttoptions = MqttOptions::new(
        &config.mqtt.client_id,
        &config.mqtt.broker,
        1883,
    );
    mqttoptions.set_keep_alive(Duration::from_secs(30));
    mqttoptions.set_credentials(&config.mqtt.username, &config.mqtt.password);

    let (client, eventloop) = AsyncClient::new(mqttoptions, 10);

    // Subscribe to the command topic using the topic_prefix from config
    let cmd_topic = format!("{}/cmd", &config.mqtt.topic_prefix);
    client
        .subscribe(cmd_topic, QoS::AtMostOnce)
        .await
        .expect("Failed to subscribe to command topic");

    // Spawn a task to handle the MQTT event loop and process incoming messages
    tokio::spawn(async move {
        let mut eventloop = eventloop;
        while let Ok(event) = eventloop.poll().await {
            use rumqttc::Event::Incoming;
            use rumqttc::Packet::Publish;
            if let Incoming(Publish(publish)) = event {
                let payload = &publish.payload;
                if let Ok(text) = std::str::from_utf8(payload) {
                    println!("Received MQTT message: {}", text);
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
                        if let Some(method) = json.get("method").and_then(|m| m.as_str()) {
                            match method {
                                "device.executeCommand" => {
                                    // Call the command execution logic and publish the response
                                    println!("Received executeCommand: {:?}", json);
                                    if let Some(params) = json.get("params") {
                                        let cmd_str = params.get("command").and_then(|c| c.as_str()).unwrap_or("");
                                        let args = params.get("args").cloned().unwrap_or(serde_json::json!({}));
                                        // Execute the command asynchronously
                                        let result = command::execute_command(cmd_str, args).await;
                                        // Prepare the JSON-RPC response
                                        let response = serde_json::json!({
                                            "jsonrpc": "2.0",
                                            "result": result,
                                            "id": json.get("id").cloned().unwrap_or(serde_json::json!(null))
                                        });
                                        // Publish the response to the reply topic (e.g., topic_prefix + "/resp")
                                        let device_id = params.get("device_id").and_then(|d| d.as_str()).unwrap_or("unknown");
                                        let topic = format!("mcp/device/{}/resp", device_id);
                                        // NOTE: To actually publish, the client instance must be accessible here (e.g., via Arc/Mutex)
                                        // client.publish(topic, QoS::AtMostOnce, false, response.to_string()).await.ok();
                                        println!("(TODO) Would publish response to topic {}: {}", topic, response);
                                    }
                                }
                                "device.reportContext" => {
                                    // TODO: 處理 context 上報
                                    println!("Received reportContext: {:?}", json);
                                }
                                _ => {
                                    println!("Unknown method: {}", method);
                                }
                            }
                        }
                    } else {
                        println!("Failed to parse JSON-RPC payload");
                    }
                } else {
                    println!("Failed to decode MQTT payload as UTF-8");
                }
            } else {
                println!("MQTT Event: {:?}", event);
            }
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
