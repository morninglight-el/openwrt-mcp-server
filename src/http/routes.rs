// HTTP Routes Module
// This module defines the HTTP routes for the server, including context and command endpoints.

use warp::Filter;

pub fn routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    println!("Initializing HTTP routes...");
    let context_route = warp::path!("api" / "context")
        .and(warp::get())
        .map(|| warp::reply::json(&serde_json::json!({
            "jsonrpc": "2.0",
            "result": "Context data placeholder",
            "id": "ctx-001"
        })));

    let command_route = warp::path!("api" / "cmd")
        .and(warp::post())
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
