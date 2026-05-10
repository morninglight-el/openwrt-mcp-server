mod config;
mod context;
mod executor;
mod http;
mod model;
mod mqtt;

#[tokio::main]
async fn main() {
    println!("Starting OpenWrt MCP Server...");

    // Load configuration
    let config = config::Config::load();

    // Initialize MQTT subsystem
    mqtt::handler::initialize_mqtt(&config).await;

    // Initialize HTTP server only if enabled in config
    let mut http_server = if config.http.enable {
        let addr: std::net::IpAddr = config
            .http
            .listen_addr
            .parse()
            .expect("Invalid listen_addr");
        let token: &'static str = Box::leak(config.http.token.clone().into_boxed_str());
        tokio::spawn(async move {
            warp::serve(http::routes::routes(token))
                .run((addr, config.http.port))
                .await;
        })
    } else {
        // Dummy future if HTTP is disabled
        tokio::spawn(async {})
    };

    // Start context collector
    let context_data = context::collector::collect_context().await;
    println!("Collected Context Data: {:?}", context_data);

    println!("OpenWrt MCP Server is running.");

    // Keep the server running and handle shutdown signals
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
