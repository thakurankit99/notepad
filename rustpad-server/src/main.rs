use rustpad_server::{server, database::Database, ServerConfig};
use std::time::Duration;
use tokio::time;
use std::sync::Arc;

// Function to ping the service at regular intervals
async fn keep_alive_ping(base_url: String, interval_secs: u64) {
    let client = reqwest::Client::new();
    
    loop {
        time::sleep(Duration::from_secs(interval_secs)).await;
        
        log::info!("Performing keep-alive ping to {}", base_url);
        
        match client.get(&base_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    log::info!("Keep-alive ping successful (status {})", response.status());
                } else {
                    log::warn!("Keep-alive ping received non-success status: {}", response.status());
                }
            },
            Err(e) => {
                log::error!("Keep-alive ping failed: {}", e);
            }
        }
    }
}

// Function to determine the application's base URL
fn determine_base_url(port: u16) -> String {
    // First try to get the URL from Render-specific environment variables
    if let Ok(render_url) = std::env::var("RENDER_EXTERNAL_URL") {
        log::info!("Using Render-provided external URL: {}", render_url);
        return render_url;
    }
    
    // Second, try to get the hostname from Render if URL isn't available
    if let Ok(render_hostname) = std::env::var("RENDER_EXTERNAL_HOSTNAME") {
        let url = format!("https://{}", render_hostname);
        log::info!("Constructed URL from Render hostname: {}", url);
        return url;
    }
    
    // Third, check if a custom URL is provided via KEEP_ALIVE_URL
    if let Ok(custom_url) = std::env::var("KEEP_ALIVE_URL") {
        log::info!("Using custom keep-alive URL: {}", custom_url);
        return custom_url;
    }
    
    // Finally, fall back to localhost with the configured port
    let fallback = format!("http://localhost:{}", port);
    log::info!("No external URL detected, using fallback: {}", fallback);
    fallback
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();
    
    log::info!("Starting Code Beautifier server...");

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| String::from("3030"))
        .parse()
        .expect("Unable to parse PORT");
    
    log::info!("Server will listen on port {}", port);

    let config = ServerConfig {
        expiry_days: std::env::var("EXPIRY_DAYS")
            .unwrap_or_else(|_| String::from("1"))
            .parse()
            .expect("Unable to parse EXPIRY_DAYS"),
        database: match std::env::var("POSTGRES_URI") {
            Ok(uri) => {
                log::info!("PostgreSQL URI found. Attempting to connect to database...");
                match Database::new(&uri).await {
                    Ok(db) => {
                        log::info!("✅ Successfully connected to PostgreSQL database!");
                        // Log the document count to verify read access
                        match db.count().await {
                            Ok(count) => log::info!("Database contains {} documents", count),
                            Err(e) => log::warn!("Could not count documents: {}", e),
                        }
                        Some(db)
                    },
                    Err(e) => {
                        log::error!("❌ Failed to connect to PostgreSQL database: {}", e);
                        log::error!("The application will continue without database persistence");
                        None
                    }
                }
            },
            Err(_) => {
                log::warn!("No POSTGRES_URI environment variable set. Running without database persistence.");
                None
            },
        },
    };

    log::info!("Document expiry set to {} days", config.expiry_days);
    log::info!("Database persistence is {}", if config.database.is_some() { "enabled" } else { "disabled" });
    
    // Setup the keep-alive ping using the interval from environment and auto-detected base URL
    let interval_secs = std::env::var("KEEP_ALIVE_INTERVAL_SECS")
        .unwrap_or_else(|_| String::from("30"))
        .parse::<u64>()
        .unwrap_or(30);
    
    // Auto-detect the base URL
    let base_url = determine_base_url(port);
    
    if interval_secs > 0 {
        log::info!("Starting keep-alive service with interval of {} seconds", interval_secs);
        log::info!("Keep-alive service will ping URL: {}", base_url);
        tokio::spawn(keep_alive_ping(base_url, interval_secs));
    } else {
        log::info!("Keep-alive service is disabled (KEEP_ALIVE_INTERVAL_SECS is 0)");
    }
    
    log::info!("Starting server on http://0.0.0.0:{}", port);
    
    // Create a server instance to spawn
    let routes = server(config);
    
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}
