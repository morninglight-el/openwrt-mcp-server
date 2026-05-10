# Project Status and Backlog

This file tracks completed implementation work and the next engineering backlog for `openwrt-mcp-server`.

## Completed Baseline

- [x] Load and validate TOML configuration.
- [x] Run authenticated HTTP endpoints for context, command execution, and capability description.
- [x] Run MQTT command subscription and response publishing.
- [x] Collect real context from OpenWrt `ubus` with Linux fallbacks.
- [x] Execute only registered allowlisted commands.
- [x] Expose command and context registries as extension points.
- [x] Emit structured JSON logs.
- [x] Add JSON Schema files for command and context payload documentation.
- [x] Add focused unit tests for config validation, MQTT broker parsing, HTTP command parsing, context extraction, command validation, capability description, and logging helpers.
- [x] Document HTTP and MQTT usage examples.
- [x] Document EdgePulse integration, standalone fleet mode, RBAC, and multi-tenant direction.

## Near-Term Backlog

- Add runtime JSON Schema validation for HTTP and MQTT payloads.
- Add persistent audit logging for requests, command results, and policy decisions.
- Add an EdgePulse client adapter for `edgepulse-ctl`.
- Route state-changing commands through EdgePulse policy-gated named actions when available.
- Add syslog integration for OpenWrt packaging.
- Add a CLI helper for local request testing and debugging.

## Fleet and Security Backlog

- Add a device registry abstraction while preserving the current single-device config.
- Add request actor identity for users and service accounts.
- Add rule-based access control for tenants, devices, methods, permissions, and command risk levels.
- Add tenant-aware config, token scopes, MQTT topic routing, context cache, and audit records.
- Add storage isolation rules for future multi-tenant operation.

## Longer-Term Backlog

- Add WebSocket support for dashboards and live command progress.
- Add streaming telemetry or `/metrics` output.
- Add context delta compression for low-bandwidth MQTT links.
- Add scheduler support for recurring commands.
- Add secure boot and system integrity context.
- Add optional gRPC transport for external orchestrators.
