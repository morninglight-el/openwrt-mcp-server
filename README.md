# openwrt-mcp-server

`openwrt-mcp-server` is a Rust-based MCP-style bridge for OpenWrt and Linux edge devices. Today it exposes a single-device JSON-RPC control surface over HTTP and MQTT. The long-term direction is to let the Rust server also run outside OpenWrt as a standalone fleet MCP service that can manage many OpenWrt devices through EdgePulse and other device adapters.

Current deployment modes:

- Local OpenWrt or Linux companion process for one device.
- Remote bridge endpoint for AI agents and orchestration systems.
- Future standalone fleet server with device registry, rule-based access control, and multi-tenant isolation.

## Current Capabilities

- HTTP API using `warp`.
- MQTT transport using `rumqttc`.
- JSON-RPC 2.0 request and response envelopes.
- Token-protected HTTP endpoints via the `x-api-token` header.
- TOML configuration with startup validation.
- Real context collection from OpenWrt `ubus`, with Linux fallback data from `/proc`, `/sys`, and `ip -j`.
- Allowlisted command execution for supported device actions.
- `device.describe` capability introspection for transports, commands, command risk metadata, and context collectors.
- Lightweight command and context registries as extension points for future modules.
- Structured JSON logs for service, config, HTTP, and MQTT events.
- JSON Schema documentation for command and context payloads under `schema/`.

## Project Layout

- `src/context/collector.rs`: Collects device context.
- `src/context/registry.rs`: Declares built-in context collectors and extension metadata.
- `src/executor/command.rs`: Dispatches allowlisted commands.
- `src/executor/registry.rs`: Declares command names, required args, and risk levels.
- `src/http/routes.rs`: Exposes authenticated HTTP JSON-RPC endpoints.
- `src/mqtt/handler.rs`: Handles MQTT subscription, dispatch, and response publishing.
- `src/config/mod.rs`: Loads and validates `config.toml`.
- `src/logging.rs`: Emits compact JSON log records.
- `src/model/types.rs`: Defines shared request/result/context types.
- `schema/`: Contains JSON Schema files for documented payload shapes.
- `docs/edgepulse-integration-roadmap.md`: Describes EdgePulse, fleet, RBAC, and multi-tenant plans.

## Build and Test

```sh
cargo build --release
cargo test
```

For OpenWrt deployment, cross-compile with the OpenWrt SDK or a musl target appropriate for the device.

## Configuration

Example `config.toml`:

```toml
[mqtt]
broker = "mqtt://localhost:1883"
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

Validation currently rejects:

- Empty `mqtt.broker`, `mqtt.client_id`, or `mqtt.topic_prefix`.
- MQTT topic prefixes containing `+` or `#`.
- Invalid `http.listen_addr`.
- `http.port = 0`.
- Empty `http.token` when HTTP is enabled.

## HTTP API

All HTTP endpoints require:

```text
x-api-token: <configured token>
```

Get current context:

```sh
curl -H 'x-api-token: change-me' \
  http://127.0.0.1:8080/api/context
```

Describe server capabilities:

```sh
curl -H 'x-api-token: change-me' \
  http://127.0.0.1:8080/api/describe
```

Execute an allowlisted command:

```sh
curl -H 'x-api-token: change-me' \
  -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","method":"device.executeCommand","params":{"command":"get_context"},"id":"cmd-1"}' \
  http://127.0.0.1:8080/api/cmd
```

Supported built-in commands are declared in `src/executor/registry.rs`.

## MQTT API

Command topic:

```text
{topic_prefix}/cmd
```

Response topic:

```text
{topic_prefix}/resp
```

Example capability request:

```json
{
  "jsonrpc": "2.0",
  "method": "device.describe",
  "params": {},
  "id": "describe-1"
}
```

Example command request:

```json
{
  "jsonrpc": "2.0",
  "method": "device.executeCommand",
  "params": {
    "command": "restart_interface",
    "args": {
      "interface": "wan"
    }
  },
  "id": "cmd-1"
}
```

## Schemas

Schema files document the expected payloads:

- `schema/command.schema.json`
- `schema/context.schema.json`

Runtime validation is currently implemented through typed parsing, command allowlisting, argument checks, and config validation. Full JSON Schema validation is planned as a future hardening step.

## EdgePulse Integration

`openwrt-mcp-server` is intended to integrate with `../edgepulse` as a remote MCP bridge. EdgePulse should remain the OpenWrt-local authority for telemetry, policy, named actions, audit logs, and AI agent conversation state. This Rust server should translate remote HTTP/MQTT/MCP-facing requests into EdgePulse local APIs where available.

See [docs/edgepulse-integration-roadmap.md](docs/edgepulse-integration-roadmap.md) for the detailed plan.

## Roadmap

Current baseline:

- Single-device HTTP/MQTT JSON-RPC bridge.
- OpenWrt/Linux context collection.
- Command and context registries.
- `device.describe` introspection.
- Structured JSON logs.
- Config validation.
- Schema documentation.

Near-term hardening:

- Runtime JSON Schema validation.
- Persistent audit log for HTTP/MQTT requests and command results.
- EdgePulse client adapter for `edgepulse-ctl`, then `ubus` or Unix socket.
- Replace direct mutation commands with EdgePulse policy-gated named actions where available.
- Command sandboxing beyond the initial allowlist.
- CLI utility for testing and debugging requests.

Fleet direction:

- Standalone MCP server mode outside OpenWrt.
- Device registry and explicit target selection.
- Rule-based access control for users, service accounts, devices, methods, and permissions.
- Tenant-aware config, tokens, MQTT routing, context caches, policies, and audit records.
- Multi-tenant fleet service operation.

Longer-term extensions:

- WebSocket transport for dashboards and live operations.
- Streaming telemetry metrics endpoint.
- Context delta compression for low-bandwidth MQTT.
- Secure boot and system integrity reporting.
- Scheduler support for recurring commands.
