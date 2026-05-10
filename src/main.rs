mod config;
mod context;
mod executor;
mod http;
mod logging;
mod model;
mod mqtt;

#[tokio::main]
async fn main() {
    logging::info("server.starting", &[]);

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
            crate::logging::info(
                "http.server.starting",
                &[
                    ("listen_addr", serde_json::json!(addr.to_string())),
                    ("port", serde_json::json!(config.http.port)),
                ],
            );
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
    logging::info(
        "context.initial_collected",
        &[
            ("device_id", serde_json::json!(context_data.device_id)),
            ("uptime", serde_json::json!(context_data.uptime)),
        ],
    );

    logging::info("server.running", &[]);

    // Keep the server running and handle shutdown signals
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            logging::info("server.shutdown_signal", &[("signal", serde_json::json!("ctrl_c"))]);
        }
        _ = &mut http_server => {
            logging::warn("http.server.completed", &[]);
        }
    }
    logging::info("server.shutdown", &[]);
}
