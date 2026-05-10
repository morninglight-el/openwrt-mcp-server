// Context Collector Module
// This module is responsible for collecting system context data from OpenWrt devices.

use crate::model::types::ContextReport;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::Path;
use tokio::process::Command;

const SCHEMA_VERSION: &str = "1.0.0";

pub async fn collect_context() -> ContextReport {
    let interfaces = collect_interfaces().await;

    ContextReport {
        device_id: read_trimmed("/proc/sys/kernel/hostname")
            .unwrap_or_else(|| "unknown".to_string()),
        uptime: read_uptime().unwrap_or(0),
        cpu_load: read_cpu_load(),
        interfaces,
        wifi_clients: collect_wifi_clients().await,
        schema_version: SCHEMA_VERSION.to_string(),
    }
}

async fn collect_interfaces() -> Value {
    if let Some(openwrt) = collect_openwrt_interfaces().await {
        return openwrt;
    }

    let ip_addr = run_command_json("ip", &["-j", "addr", "show"]).await;
    let mut interfaces = BTreeMap::new();

    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            let base = entry.path();
            let status =
                read_trimmed_path(&base.join("operstate")).unwrap_or_else(|| "unknown".to_string());
            let mac = read_trimmed_path(&base.join("address")).unwrap_or_default();
            let addresses = addresses_for_interface(&ip_addr, &name);

            interfaces.insert(
                name,
                json!({
                    "status": status,
                    "mac": mac,
                    "addresses": addresses,
                }),
            );
        }
    }

    json!(interfaces)
}

async fn collect_openwrt_interfaces() -> Option<Value> {
    let dump = run_command_json("ubus", &["call", "network.interface", "dump"]).await?;
    let interfaces = dump.get("interface")?.as_array()?;
    let mut result = BTreeMap::new();

    for interface in interfaces {
        if let Some(name) = interface.get("interface").and_then(Value::as_str) {
            result.insert(name.to_string(), json!({
                "status": if interface.get("up").and_then(Value::as_bool).unwrap_or(false) { "up" } else { "down" },
                "proto": interface.get("proto").cloned().unwrap_or(Value::Null),
                "device": interface.get("device").cloned().unwrap_or(Value::Null),
                "ipv4_address": interface.get("ipv4-address").cloned().unwrap_or_else(|| json!([])),
                "ipv6_address": interface.get("ipv6-address").cloned().unwrap_or_else(|| json!([])),
            }));
        }
    }

    Some(json!(result))
}

async fn collect_wifi_clients() -> u32 {
    let Some(dump) = run_command_json("ubus", &["call", "hostapd.*", "get_clients"]).await else {
        return 0;
    };

    dump.as_object()
        .map(|radios| {
            radios
                .values()
                .filter_map(|radio| radio.get("clients").and_then(Value::as_object))
                .map(|clients| clients.len() as u32)
                .sum()
        })
        .unwrap_or(0)
}

async fn run_command_json(program: &str, args: &[&str]) -> Option<Value> {
    let output = Command::new(program).args(args).output().await.ok()?;
    if !output.status.success() {
        return None;
    }

    serde_json::from_slice(&output.stdout).ok()
}

fn addresses_for_interface(ip_addr: &Option<Value>, name: &str) -> Vec<Value> {
    let Some(entries) = ip_addr.as_ref().and_then(Value::as_array) else {
        return Vec::new();
    };

    entries
        .iter()
        .filter(|entry| entry.get("ifname").and_then(Value::as_str) == Some(name))
        .flat_map(|entry| {
            entry
                .get("addr_info")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default()
        })
        .collect()
}

fn read_uptime() -> Option<u64> {
    let uptime = read_trimmed("/proc/uptime")?;
    let first = uptime.split_whitespace().next()?;
    first.split('.').next()?.parse().ok()
}

fn read_cpu_load() -> Vec<f32> {
    read_trimmed("/proc/loadavg")
        .map(|loadavg| {
            loadavg
                .split_whitespace()
                .take(3)
                .filter_map(|value| value.parse::<f32>().ok())
                .collect()
        })
        .unwrap_or_default()
}

fn read_trimmed(path: &str) -> Option<String> {
    read_trimmed_path(Path::new(path))
}

fn read_trimmed_path(path: &Path) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::addresses_for_interface;
    use serde_json::json;

    #[test]
    fn extracts_addresses_for_named_interface() {
        let ip_addr = Some(json!([
            {
                "ifname": "lo",
                "addr_info": [{ "local": "127.0.0.1" }]
            },
            {
                "ifname": "eth0",
                "addr_info": [
                    { "local": "192.0.2.10", "family": "inet" },
                    { "local": "2001:db8::10", "family": "inet6" }
                ]
            }
        ]));

        let addresses = addresses_for_interface(&ip_addr, "eth0");

        assert_eq!(addresses.len(), 2);
        assert_eq!(addresses[0]["local"], "192.0.2.10");
    }
}
