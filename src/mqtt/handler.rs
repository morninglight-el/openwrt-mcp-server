/*
 * MQTT Handler Module
 * This module is responsible for handling MQTT communication, including subscribing to topics and processing messages.
 */

use crate::config::Config;
use crate::executor::command;
use rumqttc::{AsyncClient, MqttOptions, QoS, Transport};
use tokio::time::Duration;

pub async fn initialize_mqtt(config: &Config) -> AsyncClient {
    crate::logging::info("mqtt.initializing", &[]);

    let (host, port, tls) = broker_endpoint(&config.mqtt.broker);

    let mut mqttoptions = MqttOptions::new(&config.mqtt.client_id, host, port);
    mqttoptions.set_keep_alive(Duration::from_secs(30));
    mqttoptions.set_credentials(&config.mqtt.username, &config.mqtt.password);
    if tls {
        mqttoptions.set_transport(Transport::tls_with_default_config());
    }

    let (client, eventloop) = AsyncClient::new(mqttoptions, 10);

    // Subscribe to the command topic using the topic_prefix from config
    let cmd_topic = format!("{}/cmd", &config.mqtt.topic_prefix);
    client
        .subscribe(cmd_topic, QoS::AtMostOnce)
        .await
        .expect("Failed to subscribe to command topic");
    crate::logging::info(
        "mqtt.subscribed",
        &[(
            "topic",
            serde_json::json!(format!("{}/cmd", &config.mqtt.topic_prefix)),
        )],
    );

    // Spawn a task to handle the MQTT event loop and process incoming messages
    let response_client = client.clone();
    let response_topic = format!("{}/resp", &config.mqtt.topic_prefix);
    tokio::spawn(async move {
        let mut eventloop = eventloop;
        while let Ok(event) = eventloop.poll().await {
            use rumqttc::Event::Incoming;
            use rumqttc::Packet::Publish;
            if let Incoming(Publish(publish)) = event {
                let payload = &publish.payload;
                if let Ok(text) = std::str::from_utf8(payload) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
                        if let Some(method) = json.get("method").and_then(|m| m.as_str()) {
                            crate::logging::info(
                                "mqtt.message.received",
                                &[
                                    ("method", serde_json::json!(method)),
                                    ("topic", serde_json::json!(publish.topic.clone())),
                                ],
                            );
                            match method {
                                "device.executeCommand" => {
                                    let response = execute_command_response(&json).await;
                                    publish_response(&response_client, &response_topic, response)
                                        .await;
                                }
                                "device.getContext" | "device.reportContext" => {
                                    let context =
                                        crate::context::collector::collect_context().await;
                                    let response = serde_json::json!({
                                        "jsonrpc": "2.0",
                                        "result": context,
                                        "id": json.get("id").cloned().unwrap_or(serde_json::Value::Null),
                                    });
                                    publish_response(&response_client, &response_topic, response)
                                        .await;
                                }
                                "device.describe" => {
                                    let response = serde_json::json!({
                                        "jsonrpc": "2.0",
                                        "result": crate::executor::command::device_describe(),
                                        "id": json.get("id").cloned().unwrap_or(serde_json::Value::Null),
                                    });
                                    publish_response(&response_client, &response_topic, response)
                                        .await;
                                }
                                _ => {
                                    let response = serde_json::json!({
                                        "jsonrpc": "2.0",
                                        "error": {
                                            "code": -32601,
                                            "message": format!("Unknown method: {method}"),
                                        },
                                        "id": json.get("id").cloned().unwrap_or(serde_json::Value::Null),
                                    });
                                    publish_response(&response_client, &response_topic, response)
                                        .await;
                                }
                            }
                        }
                    } else {
                        crate::logging::warn(
                            "mqtt.message.invalid_json",
                            &[("topic", serde_json::json!(publish.topic))],
                        );
                    }
                } else {
                    crate::logging::warn(
                        "mqtt.message.invalid_utf8",
                        &[("topic", serde_json::json!(publish.topic))],
                    );
                }
            } else {
                crate::logging::info(
                    "mqtt.event",
                    &[("event", serde_json::json!(format!("{event:?}")))],
                );
            }
        }
    });

    crate::logging::info("mqtt.initialized", &[]);

    client
}

async fn execute_command_response(json: &serde_json::Value) -> serde_json::Value {
    let id = json.get("id").cloned().unwrap_or(serde_json::Value::Null);
    let Some(params) = json.get("params") else {
        return json_rpc_error(-32602, "Missing params", id);
    };

    let Some(cmd_str) = params.get("command").and_then(|c| c.as_str()) else {
        return json_rpc_error(-32602, "Missing command", id);
    };

    let args = params
        .get("args")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    let result = command::execute_command(cmd_str, args).await;

    serde_json::json!({
        "jsonrpc": "2.0",
        "result": result,
        "id": id,
    })
}

async fn publish_response(client: &AsyncClient, topic: &str, response: serde_json::Value) {
    let payload = response.to_string();
    if let Err(error) = client.publish(topic, QoS::AtMostOnce, false, payload).await {
        crate::logging::error(
            "mqtt.response.publish_failed",
            &[
                ("topic", serde_json::json!(topic)),
                ("error", serde_json::json!(error.to_string())),
            ],
        );
    }
}

fn json_rpc_error(code: i64, message: &str, id: serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "error": {
            "code": code,
            "message": message,
        },
        "id": id,
    })
}

fn broker_endpoint(broker: &str) -> (String, u16, bool) {
    let (tls, broker) = if let Some(rest) = broker.strip_prefix("mqtts://") {
        (true, rest)
    } else if let Some(rest) = broker.strip_prefix("ssl://") {
        (true, rest)
    } else if let Some(rest) = broker.strip_prefix("mqtt://") {
        (false, rest)
    } else if let Some(rest) = broker.strip_prefix("tcp://") {
        (false, rest)
    } else {
        (false, broker)
    };

    let default_port = if tls { 8883 } else { 1883 };
    let broker = broker.split('/').next().unwrap_or(broker);
    let (host, port) = broker
        .rsplit_once(':')
        .and_then(|(host, port)| port.parse::<u16>().ok().map(|port| (host, port)))
        .unwrap_or((broker, default_port));

    (host.to_string(), port, tls)
}

#[cfg(test)]
mod tests {
    use super::broker_endpoint;

    #[test]
    fn parses_plain_mqtt_broker_urls() {
        assert_eq!(
            broker_endpoint("mqtt://broker.example:1884"),
            ("broker.example".to_string(), 1884, false)
        );
        assert_eq!(
            broker_endpoint("localhost"),
            ("localhost".to_string(), 1883, false)
        );
    }

    #[test]
    fn parses_tls_mqtt_broker_urls() {
        assert_eq!(
            broker_endpoint("mqtts://iot.example.com"),
            ("iot.example.com".to_string(), 8883, true)
        );
    }
}
