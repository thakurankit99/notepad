use rustpad_server::{server, database::Database, ServerConfig};
use log::{info, warn, error, debug};
use std::{io::Write, time::Duration};
use tokio::time;

// Setup self-ping mechanism to prevent Render from spinning down
async fn setup_self_ping(interval_seconds: u64) {
    // Try to determine the best URL to use for self-pinging
    let url = determine_ping_url();
    info!("Setting up self-ping service to {} with interval of {} seconds", url, interval_seconds);
    
    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let mut interval = time::interval(Duration::from_secs(interval_seconds));
        
        loop {
            interval.tick().await;
            match client.get(&url).send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        debug!("Self-ping successful");
                    } else {
                        warn!("Self-ping returned non-success status: {}", response.status());
                    }
                },
                Err(e) => warn!("Self-ping failed: {}", e),
            }
        }
    });
}

// Determine the best URL to use for self-pinging
fn determine_ping_url() -> String {
    // First priority: Check if a custom ping URL is specified
    if let Ok(custom_url) = std::env::var("SELF_PING_URL") {
        if !custom_url.is_empty() {
            return custom_url;
        }
    }
    
    // Second priority: Check for Render's external URL (automatically includes protocol and port)
    if let Ok(render_url) = std::env::var("RENDER_EXTERNAL_URL") {
        if !render_url.is_empty() {
            return render_url;
        }
    }
    
    // Third priority: Check for Render's external hostname with HTTPS (default for Render services)
    if let Ok(render_hostname) = std::env::var("RENDER_EXTERNAL_HOSTNAME") {
        if !render_hostname.is_empty() {
            return format!("https://{}", render_hostname);
        }
    }
    
    // Fourth priority: Use internal host/port with appropriate protocol
    // Determine if SSL is enabled
    let ssl_enabled = std::env::var("SSL_ENABLED")
        .map(|val| val == "true" || val == "1")
        .unwrap_or(false);
    
    // Get host and port
    let host = std::env::var("HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3030".to_string());
    
    // Format appropriate URL
    if ssl_enabled {
        format!("https://{}:{}/", host, port)
    } else {
        format!("http://{}:{}/", host, port)
    }
}

#[tokio::main]
async fn main() {
    // Set up environment variables
    dotenv::dotenv().ok();
    
    // Configure logger with minimal output
    let env = env_logger::Env::default()
        .filter_or("RUST_LOG", "info");
    
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}] {}",
                record.level(),
                record.args()
            )
        })
        .init();
    
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| String::from("3030"))
        .parse()
        .expect("Unable to parse PORT");

    info!("Starting Code Beautifier on port {}", port);

    // Setup self-ping if enabled
    if let Ok(enabled) = std::env::var("SELF_PING_ENABLED") {
        if enabled == "true" {
            if let Ok(interval) = std::env::var("SELF_PING_INTERVAL") {
                if let Ok(interval_seconds) = interval.parse::<u64>() {
                    setup_self_ping(interval_seconds).await;
                } else {
                    // Default to 4 minutes if parse fails
                    setup_self_ping(240).await;
                }
            } else {
                // Default to 4 minutes if env var not set
                setup_self_ping(240).await;
            }
        }
    }

    let config = ServerConfig {
        expiry_days: std::env::var("EXPIRY_DAYS")
            .unwrap_or_else(|_| String::from("1"))
            .parse()
            .expect("Unable to parse EXPIRY_DAYS"),
        database: match std::env::var("POSTGRES_URI") {
            Ok(uri) => {
                match Database::new(&uri).await {
                    Ok(db) => {
                        info!("PostgreSQL connection successful");
                        Some(db)
                    },
                    Err(e) => {
                        error!("PostgreSQL connection failed: {}", e);
                        None
                    }
                }
            },
            Err(_) => {
                warn!("No POSTGRES_URI found. Running without persistence.");
                None
            },
        },
    };

    info!("Server ready");
    warp::serve(server(config)).run(([0, 0, 0, 0], port)).await;
}
