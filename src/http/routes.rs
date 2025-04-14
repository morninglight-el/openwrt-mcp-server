/*
 * HTTP Routes Module
 * This module defines the HTTP routes for the server, including context and command endpoints.
 * Each endpoint requires a valid API token in the "x-api-token" header.
 */

use warp::Filter;

/// Returns the HTTP API routes, requiring the provided token for authentication.
pub fn routes(token: &'static str) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    println!("Initializing HTTP routes...");

    // Filter to check the x-api-token header
    let token_filter = warp::header::exact("x-api-token", token);

    let context_route = warp::path!("api" / "context")
        .and(warp::get())
        .and(token_filter.clone())
        .map(|| warp::reply::json(&serde_json::json!({
            "jsonrpc": "2.0",
            "result": "Context data placeholder",
            "id": "ctx-001"
        })));

    let command_route = warp::path!("api" / "cmd")
        .and(warp::post())
        .and(token_filter.clone())
        .and(warp::body::json())
        .map(|cmd: serde_json::Value| {
            println!("Received command: {:?}", cmd);
            warp::reply::json(&serde_json::json!({
                "jsonrpc": "2.0",
                "result": "Command executed successfully",
                "id": cmd["id"]
            }))
        });

    context_route.or(command_route)
}
