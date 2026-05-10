// Command Executor Module
// This module is responsible for executing commands received via MQTT or HTTP.

use crate::context::collector;
use crate::context::registry::builtin_collectors;
use crate::executor::registry::{builtin_commands, find_command};
use serde_json::{json, Value};
use tokio::process::Command;

pub async fn execute_command(command: &str, args: Value) -> Value {
    if find_command(command).is_none() {
        return json!({
            "status": "error",
            "output": format!("Unsupported command: {command}"),
        });
    }

    match command {
        "get_context" | "report_context" => {
            let context = collector::collect_context().await;
            json!({
                "status": "ok",
                "output": context,
            })
        }
        "device_describe" => device_describe(),
        "restart_interface" => restart_interface(args).await,
        "reload_network" => {
            run_allowlisted_command(
                "/etc/init.d/network",
                &["reload"],
                "Network service reload requested",
            )
            .await
        }
        _ => unreachable!("command registry and dispatcher are out of sync"),
    }
}

pub fn device_describe() -> Value {
    json!({
        "status": "ok",
        "output": {
            "server": "openwrt-mcp-server",
            "schema_version": "1.0.0",
            "transports": ["http", "mqtt"],
            "commands": builtin_commands(),
            "context_collectors": builtin_collectors(),
            "extension_points": {
                "commands": "Add command definitions in executor::registry and dispatch handlers in executor::command.",
                "context": "Add collector definitions in context::registry and merge outputs in context::collector."
            }
        }
    })
}

async fn restart_interface(args: Value) -> Value {
    let Some(interface) = args.get("interface").and_then(Value::as_str) else {
        return json!({
            "status": "error",
            "output": "Missing required argument: interface",
        });
    };

    if !is_safe_interface_name(interface) {
        return json!({
            "status": "error",
            "output": "Invalid interface name",
        });
    }

    let down = run_program("ifdown", &[interface]).await;
    let up = run_program("ifup", &[interface]).await;

    if down.status_success && up.status_success {
        json!({
            "status": "ok",
            "output": format!("Interface {interface} restarted"),
            "details": {
                "ifdown": down.output,
                "ifup": up.output,
            }
        })
    } else {
        json!({
            "status": "error",
            "output": format!("Failed to restart interface {interface}"),
            "details": {
                "ifdown": down.output,
                "ifup": up.output,
            }
        })
    }
}

async fn run_allowlisted_command(program: &str, args: &[&str], success_message: &str) -> Value {
    let result = run_program(program, args).await;
    if result.status_success {
        json!({
            "status": "ok",
            "output": success_message,
            "details": result.output,
        })
    } else {
        json!({
            "status": "error",
            "output": result.output,
        })
    }
}

async fn run_program(program: &str, args: &[&str]) -> ProgramResult {
    match Command::new(program).args(args).output().await {
        Ok(output) => ProgramResult {
            status_success: output.status.success(),
            output: command_output(&output.stdout, &output.stderr),
        },
        Err(error) => ProgramResult {
            status_success: false,
            output: error.to_string(),
        },
    }
}

fn command_output(stdout: &[u8], stderr: &[u8]) -> String {
    let stdout = String::from_utf8_lossy(stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();

    match (stdout.is_empty(), stderr.is_empty()) {
        (false, false) => format!("{stdout}\n{stderr}"),
        (false, true) => stdout,
        (true, false) => stderr,
        (true, true) => String::new(),
    }
}

fn is_safe_interface_name(interface: &str) -> bool {
    !interface.is_empty()
        && interface.len() <= 32
        && interface
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b':'))
}

struct ProgramResult {
    status_success: bool,
    output: String,
}

#[cfg(test)]
mod tests {
    use super::{command_output, device_describe, execute_command, is_safe_interface_name};
    use serde_json::json;

    #[test]
    fn validates_interface_names() {
        assert!(is_safe_interface_name("wan"));
        assert!(is_safe_interface_name("br-lan.10"));
        assert!(!is_safe_interface_name(""));
        assert!(!is_safe_interface_name("wan;reboot"));
        assert!(!is_safe_interface_name("wan up"));
    }

    #[test]
    fn combines_stdout_and_stderr() {
        assert_eq!(command_output(b"ok\n", b"warn\n"), "ok\nwarn");
        assert_eq!(command_output(b"ok\n", b""), "ok");
        assert_eq!(command_output(b"", b"warn\n"), "warn");
    }

    #[tokio::test]
    async fn rejects_unsupported_commands() {
        let result = execute_command("format_flash", json!({})).await;

        assert_eq!(result["status"], "error");
    }

    #[test]
    fn describes_builtin_capabilities() {
        let result = device_describe();

        assert_eq!(result["status"], "ok");
        assert!(result["output"]["commands"].as_array().unwrap().len() >= 3);
    }
}
