mod mqtt;
mod http;
mod context;
mod config;

#[tokio::main]
async fn main() {
    println!("Starting OpenWrt MCP Server...");

    // Initialize MQTT
    // Load configuration
    let config = config::Config::load();

    // Initialize MQTT
    mqtt::handler::initialize_mqtt(&config).await;

    // Initialize HTTP
    // Initialize HTTP server
    let mut http_server = tokio::spawn(async move {
        warp::serve(http::routes::routes())
            .run(([127, 0, 0, 1], config.http_port))
            .await;
    });

    // Start Context Collector
    // Start Context Collector
    let context_data = context::collector::collect_context().await;
    println!("Collected Context Data: {}", context_data);

    println!("OpenWrt MCP Server is running.");

    // Keep the server running and handle shutdown
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            println!("Received Ctrl+C, shutting down...");
        }
        _ = &mut http_server => {
            println!("HTTP server task completed.");
        }
    }
    println!("Shutting down OpenWrt MCP Server...");
}
