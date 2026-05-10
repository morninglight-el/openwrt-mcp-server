# EdgePulse Integration and Fleet Roadmap

Review date: 2026-05-10

This document describes how `openwrt-mcp-server` should integrate with `../edgepulse` and how the Rust service should evolve from a single-device bridge into a standalone fleet MCP server.

## Summary

The near-term design keeps a clear ownership boundary:

- EdgePulse is the OpenWrt-local authority for telemetry, AI agent state, named actions, policy checks, audit logs, UCI, and device-local operations.
- `openwrt-mcp-server` is the remote bridge for HTTP, MQTT, JSON-RPC, and future MCP-compatible clients.
- State-changing operations should eventually flow through EdgePulse policy-gated named actions rather than being duplicated as direct Rust system commands.

The long-term design lets the Rust server run outside OpenWrt as a fleet control plane that manages many OpenWrt devices with RBAC and tenant isolation.

## Current State

The Rust server currently provides:

- Single-device HTTP and MQTT JSON-RPC transport.
- OpenWrt/Linux context collection.
- Allowlisted command execution.
- Command and context registries.
- `device.describe` capability introspection.
- Structured JSON logs.
- Config validation.
- Schema files for documented payloads.

It does not yet provide:

- An EdgePulse client adapter.
- Persistent audit storage.
- Device registry.
- User/service-account identity.
- RBAC policy evaluation.
- Multi-tenant storage or routing.

## Local Bridge Architecture

Near-term request flow:

```text
External AI client / orchestrator
        |
        | HTTP or MQTT JSON-RPC
        v
openwrt-mcp-server
        |
        | direct local collectors and allowlisted commands
        v
OpenWrt or Linux device
```

Preferred EdgePulse-integrated flow:

```text
External AI client / orchestrator
        |
        | HTTP or MQTT JSON-RPC
        v
openwrt-mcp-server
        |
        | edgepulse-ctl, ubus, or Unix socket
        v
EdgePulse runtime
        |
        | policy, audit, telemetry, UCI, SQLite
        v
OpenWrt device
```

The Rust server can continue collecting basic local context directly. Richer telemetry, AI agent conversations, confirmed actions, and audit-sensitive mutations should move to EdgePulse APIs as they become available.

## EdgePulse Method Mapping

Future MCP methods should map to stable EdgePulse capabilities.

| MCP method | EdgePulse target | Notes |
| --- | --- | --- |
| `edgepulse.status` | `edgepulse-ctl status --json` | Read-only device and telemetry summary. |
| `edgepulse.agent.status` | `edgepulse-ctl agent status` | Read-only agent, model, and policy status. |
| `edgepulse.agent.chat.list` | Shared conversation store | Read conversation metadata and messages. |
| `edgepulse.agent.chat.ask` | Shared agent runtime | Add a user message and assistant response. |
| `edgepulse.agent.action.run` | Policy-gated named action | Requires EdgePulse policy enforcement and explicit confirmation for mutations. |
| `edgepulse.agent.audit.list` | Audit log store | Read-only audit history. |

State-changing EdgePulse methods must preserve EdgePulse policy decisions and audit records. The Rust server should not become a bypass around local policy.

## Packaging Relationship

Short term:

- Keep `openwrt-mcp-server` independently buildable and runnable.
- Document it as an optional companion bridge for EdgePulse.
- Keep direct built-in commands available for development and non-EdgePulse deployments.

Medium term:

- Add an EdgePulse adapter module in Rust.
- Start with `edgepulse-ctl` because it is easiest to integrate and test.
- Prefer `ubus` or a Unix domain socket once EdgePulse exposes a long-running local API.
- Move OpenWrt mutation commands toward EdgePulse named actions.
- Validate that Rust-originated and local EdgePulse-originated operations produce consistent audit records.

Long term:

- Package `openwrt-mcp-server` as an optional OpenWrt feed package for devices with enough storage and memory.
- Also support a non-OpenWrt deployment where the Rust server manages many OpenWrt devices remotely.

## Standalone Fleet MCP Server

The intended long-term direction is to make the Rust server independent of OpenWrt itself.

Fleet-mode architecture:

```text
MCP clients / AI agents / operators
        |
        v
Standalone Rust MCP server
        |
        | device registry, RBAC, tenant isolation, audit
        v
Many OpenWrt devices running EdgePulse and/or local agents
```

Fleet-mode capabilities:

- Device registry with IDs, labels, endpoint metadata, tenant ownership, and trust state.
- Transport abstraction for MQTT, HTTP, EdgePulse local APIs, and optional SSH-backed workflows.
- Per-device context cache and telemetry summary.
- Explicit device targeting for every operation.
- Fleet-wide audit records containing actor, tenant, device, method, arguments, policy decision, and result.

## Rule-Based Access Control

Access control should become rule based rather than only token based.

The RBAC layer should authorize:

- Which users or service accounts can access the server.
- Which tenants each actor belongs to.
- Which devices each actor can see or operate.
- Which methods each actor can call.
- Which command risk levels are allowed.
- Whether the actor can perform read-only, confirmed, or administrative operations.

Example future rule shape:

```toml
[[access.rules]]
subject = "user:alice"
tenant = "home-lab"
devices = ["openwrt-one", "openwrt-lab-*"]
methods = ["device.getContext", "device.describe", "edgepulse.status"]
permissions = ["read_context", "read_status"]

[[access.rules]]
subject = "service:fleet-operator"
tenant = "isp-managed"
devices = ["tenant-device-*"]
methods = ["edgepulse.agent.action.run"]
permissions = ["run_confirmed_action"]
requires_confirmation = true
```

RBAC decisions should happen before command dispatch. Both allow and deny decisions should be auditable.

## Multi-Tenant Evolution

Multi-tenant support should be introduced after device registry and RBAC foundations exist.

Required concepts:

- Tenant IDs on devices, users, service accounts, API tokens, MQTT topic prefixes, cached context, policy defaults, and audit records.
- Tenant-scoped device listing and command execution.
- Tenant-scoped rate limits and quotas.
- Tenant-level policy defaults with per-device overrides.
- Storage isolation so one tenant cannot infer another tenant's device names, telemetry, actions, policy, or audit history.

A request without tenant identity should only be accepted in explicit single-tenant compatibility mode.

## Implementation Milestones

1. Keep the local bridge stable: HTTP, MQTT, schemas, command registry, context registry, structured logs, and config validation.
2. Add persistent request and command audit records.
3. Add EdgePulse client adapter support, starting with `edgepulse-ctl`.
4. Route EdgePulse-aware state-changing commands through EdgePulse policy-gated named actions.
5. Add a device registry abstraction while preserving single-device config compatibility.
6. Add actor identity and RBAC policy evaluation before dispatch.
7. Add tenant-aware config, storage, audit records, and MQTT topic routing.
8. Promote the Rust server to standalone fleet MCP mode while keeping OpenWrt-local deployment supported.
