# openwrt-mcp-server Functional Requirements and Architecture

## Project Overview

`openwrt-mcp-server` is a lightweight Rust service that exposes an MCP-style JSON-RPC interface for OpenWrt and Linux edge devices. The current implementation is a single-device HTTP/MQTT bridge. The planned architecture evolves it into a standalone fleet MCP server that can manage multiple OpenWrt devices through EdgePulse and other adapters.

The project should support two compatible roles:

- Device-local bridge: runs on or near an OpenWrt/Linux device and exposes context plus allowlisted actions.
- Fleet MCP server: runs independently from OpenWrt and coordinates many managed devices with RBAC, tenant isolation, and audit records.

## Core Goals

| Goal | Current requirement |
| --- | --- |
| Multi-protocol | Support HTTP and MQTT JSON-RPC entrypoints. |
| Lightweight | Keep the local-device mode small enough for constrained OpenWrt targets. |
| Extensible | Add commands and context collectors through registries before introducing dynamic plugins. |
| AI friendly | Use stable JSON-RPC 2.0 envelopes and machine-readable capability descriptions. |
| Secure by default | Require HTTP tokens, command allowlisting, config validation, and clear risk metadata. |
| EdgePulse compatible | Treat EdgePulse as the local authority for richer telemetry, policy, actions, audit, and conversation state. |
| Fleet ready | Preserve a path toward device registry, RBAC, and multi-tenant operation. |

## Current Architecture

```text
src/
├── main.rs
├── config/
│   └── mod.rs
├── context/
│   ├── collector.rs
│   └── registry.rs
├── executor/
│   ├── command.rs
│   └── registry.rs
├── http/
│   └── routes.rs
├── model/
│   ├── mod.rs
│   └── types.rs
├── mqtt/
│   └── handler.rs
└── logging.rs

schema/
├── command.schema.json
└── context.schema.json

docs/
└── edgepulse-integration-roadmap.md
```

## Runtime Behavior

### HTTP

The HTTP API is implemented with `warp`.

Required behavior:

- Require `x-api-token` for all routes.
- Expose `GET /api/context`.
- Expose `GET /api/describe`.
- Expose `POST /api/cmd`.
- Return JSON-RPC compatible result or error envelopes.
- Dispatch commands only after method parsing and command allowlist checks.

### MQTT

The MQTT client is implemented with `rumqttc`.

Required behavior:

- Connect to the configured broker.
- Subscribe to `{topic_prefix}/cmd`.
- Publish responses to `{topic_prefix}/resp`.
- Support plain MQTT and `mqtts://` broker URLs.
- Dispatch `device.executeCommand`, `device.getContext`, `device.reportContext`, and `device.describe`.
- Return JSON-RPC errors for invalid or unknown methods where possible.

### Context Collection

Required fields:

- `device_id`
- `uptime`
- `cpu_load`
- `interfaces`
- `wifi_clients`
- `schema_version`

Collection strategy:

- Prefer OpenWrt `ubus` for network interface and Wi-Fi client data.
- Fall back to Linux `/proc`, `/sys/class/net`, and `ip -j addr show` where OpenWrt-specific commands are unavailable.
- Keep collector metadata in `context::registry` so new collectors can be added without changing transport code.

### Command Execution

Commands must be registered before dispatch.

Current built-in commands:

| Command | Purpose | Risk |
| --- | --- | --- |
| `get_context` | Return current context report. | Read-only |
| `report_context` | Alias for `get_context`. | Read-only |
| `device_describe` | Return server capability metadata. | Read-only |
| `restart_interface` | Restart a named OpenWrt interface. | Network restart |
| `reload_network` | Reload the OpenWrt network service. | Network restart |

Command requirements:

- Reject unknown commands.
- Validate command arguments before invoking system tools.
- Avoid arbitrary shell execution.
- Keep command metadata in `executor::registry`.
- Move state-changing operations toward EdgePulse policy-gated named actions as the integration matures.

## Configuration

Example:

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

Validation requirements:

- `mqtt.broker`, `mqtt.client_id`, and `mqtt.topic_prefix` must be non-empty.
- `mqtt.topic_prefix` must not contain MQTT wildcards.
- `http.listen_addr` must parse as an IP address.
- `http.port` must be non-zero.
- `http.token` must be non-empty when HTTP is enabled.

## JSON-RPC Methods

### `device.getContext`

Returns the current context report.

### `device.reportContext`

Currently returns the same context report as `device.getContext`. The name is kept for compatibility with device-to-controller context report semantics.

### `device.describe`

Returns capability metadata, including:

- Server name and schema version.
- Supported transports.
- Registered commands and risk metadata.
- Registered context collectors.
- Extension point notes.

### `device.executeCommand`

Executes an allowlisted command:

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
  "id": "cmd-1"
}
```

## Schemas

Schema files:

- `schema/command.schema.json`
- `schema/context.schema.json`

Current schema role:

- Document expected request and report shapes.
- Provide a stable contract for external clients and future tests.

Future schema role:

- Runtime validation for HTTP and MQTT payloads.
- CI checks for example payloads.
- Versioned schema migration as fleet features are added.

## Logging

The service emits compact JSON log records through `src/logging.rs`.

Required fields:

- `level`
- `event`
- `timestamp_unix_ms`

Event-specific fields may include route, method, command, topic, error, port, device ID, or other operational metadata.

Future logging work:

- Add persistent audit records.
- Add syslog integration for OpenWrt.
- Include actor, tenant, target device, policy decision, and request ID once RBAC and fleet mode exist.

## OpenWrt Deployment Requirements

| Component | Recommendation |
| --- | --- |
| Init system | Use `procd` or an init.d wrapper. |
| Config path | Current development uses local `config.toml`; package builds should install under `/etc/openwrt-mcp/` or an equivalent OpenWrt config path. |
| Logging | Emit stdout/stderr JSON logs initially; package integration can route them to syslog. |
| TLS | Use `mqtts://` for MQTT where broker support exists; HTTP TLS termination can initially sit in front of the service. |
| Binary | Cross-compile with the OpenWrt SDK or musl target. |

## EdgePulse Integration Requirements

`openwrt-mcp-server` should integrate with `../edgepulse` as a remote bridge.

Boundary:

- EdgePulse owns local OpenWrt policy, named actions, telemetry, audit logs, and conversation persistence.
- The Rust server owns remote HTTP/MQTT/MCP-facing transports, authentication, request translation, and future fleet orchestration.

Integration sequence:

1. Add an EdgePulse client adapter for `edgepulse-ctl`.
2. Add `ubus` or Unix socket support when EdgePulse exposes a long-running local API.
3. Route state-changing operations through EdgePulse named actions.
4. Confirm Rust-originated requests and local EdgePulse requests produce consistent audit records.
5. Keep direct Rust OpenWrt commands only as compatibility or emergency fallback paths.

## Fleet, RBAC, and Multi-Tenant Requirements

The long-term Rust server should be able to run independently from OpenWrt and manage many devices.

Fleet requirements:

- Device registry with IDs, labels, endpoints, tenant ownership, and trust state.
- Explicit target selection for every device operation.
- Transport adapters for MQTT, HTTP, EdgePulse local APIs, and optional SSH-backed workflows.
- Context cache and telemetry summaries per device.

Rule-based access control requirements:

- Identify users and service accounts.
- Authorize access by tenant, device, method, permission, and command risk.
- Evaluate policy before dispatch.
- Log allow and deny decisions.
- Support confirmation requirements for state-changing actions.

Multi-tenant requirements:

- Tenant IDs on users, service accounts, devices, tokens, MQTT topics, cached context, policy defaults, and audit records.
- Tenant-scoped device listing and command execution.
- Storage isolation so one tenant cannot infer another tenant's device names, telemetry, policies, or operations.
- Single-tenant compatibility mode only when tenant support is not enabled.

## Future Extensions

- Runtime JSON Schema validation.
- Persistent audit log and syslog integration.
- WebSocket transport for dashboards.
- Streaming telemetry metrics endpoint.
- Context delta compression for low-bandwidth MQTT.
- Scheduler support for recurring commands.
- Secure boot and system integrity reporting.
- Optional gRPC transport for external orchestrators.
