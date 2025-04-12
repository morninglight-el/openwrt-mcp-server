# openwrt-mcp-server — Functional Requirements & Architecture

## 🧠 Project Overview

`openwrt-mcp-server` is a lightweight, Rust-based implementation of the Model Context Protocol (MCP), running on OpenWrt. It enables communication between edge devices and AI agents using MQTT and/or HTTP, using the JSON-RPC 2.0 format.

---

## 📆 Core Goals

| Goal                 | Description                                                                     |
| -------------------- | ------------------------------------------------------------------------------- |
| 🌐 Multi-Protocol     | Support MQTT and HTTP interfaces for full-duplex device ↔ AI ↔ network control  |
| 🚚 Lightweight Design | Efficient enough for low-resource OpenWrt devices (32MB+ flash / 64MB+ RAM)     |
| 🔄 Extensible         | Easily add commands, extend context fields, plugin-style architecture           |
| 🧠 AI Friendly        | JSON-RPC 2.0 structure for easy integration with LLMs and orchestration systems |
| 🔒 Secure             | Support for TLS, token auth, command allowlisting                               |

---

## 📊 Architecture Modules

```
openwrt-mcp-server
├── main.rs
├── config/
│   └── mod.rs
├── mqtt/
│   ├── client.rs
│   └── handler.rs
├── http/
│   └── routes.rs
├── context/
│   └── collector.rs
├── executor/
│   └── command.rs
├── model/
│   └── types.rs
├── schema/
│   └── context.schema.json
│   └── command.schema.json
└── utils/
```

---

## 🔧 OpenWrt Deployment Considerations

| Component   | Recommendation                                           |
| ----------- | -------------------------------------------------------- |
| Init system | Use `procd` or `init.d/openwrt-mcp`                      |
| Config path | `/etc/config/mcp.toml` or `/etc/openwrt-mcp/config.toml` |
| Logging     | `/tmp/log/mcp.log` or syslog                             |
| TLS certs   | Store under `/etc/mcp-server/certs/`                     |
| Binary      | Cross-compiled static binary via musl or OpenWrt SDK     |

---

## 🔹 Example config.toml

```toml
[mqtt]
broker = "mqtts://iot.example.com:8883"
client_id = "openwrt-one"
username = "mcp-user"
password = "mcp-pass"
topic_prefix = "mcp/device/openwrt-one"

[http]
enable = true
listen_addr = "0.0.0.0"
port = 8080
token = "your-api-token"
```

---

## 📃 JSON-RPC 2.0 Message Format

### 🔄 Context Report (Device → AI)

```json
{
  "jsonrpc": "2.0",
  "method": "device.reportContext",
  "params": {
    "device_id": "openwrt-one",
    "uptime": 846120,
    "cpu_load": [0.05, 0.12, 0.09],
    "interfaces": {
      "lan": { "status": "up", "ip": "192.168.1.1" },
      "wan": { "status": "connected", "ip": "203.0.113.45" }
    },
    "wifi_clients": 5,
    "schema_version": "1.0.0"
  },
  "id": "ctx-20250412-001"
}
```

### 🧰 Command (AI → Device)

```json
{
  "jsonrpc": "2.0",
  "method": "device.executeCommand",
  "params": {
    "command": "restart_interface",
    "args": {
      "interface": "wan"
    },
    "schema_version": "1.0.0"
  },
  "id": "cmd-20250412-001"
}
```

### 📢 Command Result (Device → AI)

```json
{
  "jsonrpc": "2.0",
  "result": {
    "command_id": "cmd-20250412-001",
    "status": "ok",
    "output": "WAN interface restarted",
    "timestamp": "2025-04-12T09:12:00Z"
  },
  "id": "cmd-20250412-001"
}
```

---

## 🚀 Recommended Rust Crates

| Function | Crate                           |
| -------- | ------------------------------- |
| MQTT     | `rumqttc`                       |
| HTTP     | `axum`                          |
| Config   | `config`, `toml`                |
| Logging  | `tracing`, `tracing_subscriber` |
| JSON     | `serde`, `serde_json`           |

---

## 🚡 Future Extensions

- WebSocket API for dashboard integration
- AI-driven decision loop (device.describe, device.predict)
- Streaming telemetry support (e.g., `/metrics`)
- Plugin system for third-party commands or collectors