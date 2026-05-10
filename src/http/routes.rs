/*
 * HTTP Routes Module
 * This module defines the HTTP routes for the server, including context and command endpoints.
 * Each endpoint requires a valid API token in the "x-api-token" header.
 */

use warp::Filter;

/// Returns the HTTP API routes, requiring the provided token for authentication.
pub fn routes(
    token: &'static str,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    println!("Initializing HTTP routes...");

    // Filter to check the x-api-token header
    let token_filter = warp::header::exact("x-api-token", token);

    let context_route = warp::path!("api" / "context")
        .and(warp::get())
        .and(token_filter.clone())
        .and_then(|| async move {
            let context = crate::context::collector::collect_context().await;
            Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                "jsonrpc": "2.0",
                "result": context,
                "id": "ctx-001"
            })))
        });

    let command_route = warp::path!("api" / "cmd")
        .and(warp::post())
        .and(token_filter.clone())
        .and(warp::body::json())
        .and_then(|cmd: serde_json::Value| async move {
            println!("Received command: {:?}", cmd);
            let id = cmd.get("id").cloned().unwrap_or(serde_json::Value::Null);
            let response = match parse_command_params(&cmd) {
                Ok((command, args)) => {
                    let result = crate::executor::command::execute_command(&command, args).await;
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "result": result,
                        "id": id,
                    })
                }
                Err(message) => serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32602,
                        "message": message,
                    },
                    "id": id,
                }),
            };

            Ok::<_, warp::Rejection>(warp::reply::json(&response))
        });

    context_route.or(command_route)
}

fn parse_command_params(
    cmd: &serde_json::Value,
) -> Result<(String, serde_json::Value), &'static str> {
    if cmd.get("jsonrpc").and_then(serde_json::Value::as_str) != Some("2.0") {
        return Err("Invalid or missing jsonrpc version");
    }

    if cmd.get("method").and_then(serde_json::Value::as_str) != Some("device.executeCommand") {
        return Err("Unsupported method");
    }

    let params = cmd.get("params").ok_or("Missing params")?;
    let request: crate::model::types::CommandRequest =
        serde_json::from_value(params.clone()).map_err(|_| "Invalid command params")?;

    Ok((request.command, request.args))
}

#[cfg(test)]
mod tests {
    use super::parse_command_params;
    use serde_json::json;

    #[test]
    fn parses_valid_command_params() {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "device.executeCommand",
            "params": {
                "command": "restart_interface",
                "args": { "interface": "wan" }
            },
            "id": "cmd-1"
        });

        let (command, args) = parse_command_params(&request).unwrap();

        assert_eq!(command, "restart_interface");
        assert_eq!(args["interface"], "wan");
    }

    #[test]
    fn rejects_wrong_method() {
        let request = json!({
            "jsonrpc": "2.0",
            "method": "device.reboot",
            "params": {}
        });

        assert!(parse_command_params(&request).is_err());
    }
}
