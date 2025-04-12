# openwrt-mcp-server

`openwrt-mcp-server` is a lightweight and extensible MCP (Model Context Protocol) server designed to run on OpenWrt-based embedded routers and devices. It enables two-way communication between the device and external AI systems using MQTT and HTTP, with JSON-RPC 2.0 as the message format.

This server is intended to provide a secure and structured interface for AI agents to:

- Query live device context (network, Wi-Fi, system metrics)
- Execute system-level commands remotely
- Support real-time command-response and context streaming

## ✨ Features

- Built in Rust for performance and safety
- Supports MQTT (via `rumqttc`) and HTTP (via `axum`)
- Compatible with JSON-RPC 2.0 for AI model integration
- Modular architecture for future extensibility
- Low memory footprint, suitable for embedded OpenWrt targets

## 🌎 Use Cases

- AI-powered home gateway monitoring and orchestration
- Edge-managed device fleet context reporting
- Auto-recovery and self-healing network policies via AI
- Integration with LLMs and orchestration pipelines (e.g., n8n, LangChain)

## 🛠️ Components

- `context/collector.rs`: Gathers runtime status from OpenWrt (ubus, uci, ifstatus)
- `mqtt/handler.rs`: Subscribes and publishes context/command channels
- `http/routes.rs`: RESTful API for status and command entry
- `executor/command.rs`: Executes validated system-level instructions
- `config/mod.rs`: Loads `.toml` configuration

## 🛡️ Protocol

Follows JSON-RPC 2.0. See REQUIREMENTS.md for full message schemas.

## 🔧 Building

```bash
cargo build --release
```

Cross-compilation for OpenWrt (musl) recommended for deployment.

## 🌐 Configuration

See example `config.toml` in `/etc/openwrt-mcp/` or project root.

## 🚀 Roadmap

- [ ] Initial MQTT + HTTP dual-protocol support
- [ ] JSON-RPC 2.0 command and context schema
- [ ] Basic system command execution (e.g., reboot, restart interface)
- [ ] Context collector with UCI/UBUS/ifstatus integration
- [ ] Modular configuration loader (`.toml`)
- [ ] Device capability introspection (`device.describe`)
- [ ] WebSocket transport layer for real-time control
- [ ] Command allowlisting and sandboxing
- [ ] Plugin-style extensibility for new command modules
- [ ] Streaming telemetry metrics channel (e.g., `/metrics`)
- [ ] CLI interface for testing/debugging commands
- [ ] Optional gRPC support for external orchestrators
- [ ] JSON Schema-based validation for input/output
- [ ] OTA update interface (optional integration)
- [ ] Context delta compression for low-bandwidth MQTT
- [ ] Persistent log and audit tracking via syslog
- [ ] Secure boot detection and system integrity reporting
- [ ] Multilingual context formatting for LLM compatibility
- [ ] Scheduler support for recurring commands
