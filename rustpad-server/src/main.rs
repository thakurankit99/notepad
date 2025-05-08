use rustpad_server::{server, database::Database, ServerConfig};
use log::{info, warn, error, LevelFilter};
use std::io::Write;

#[tokio::main]
async fn main() {
    // Set up environment variables
    dotenv::dotenv().ok();
    
    // Configure logger with explicit console output
    let env = env_logger::Env::default()
        .filter_or("RUST_LOG", "info"); // Default to info level if RUST_LOG not set
    
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}] {} - {}",
                record.level(),
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.args()
            )
        })
        .init();
    
    // Force some output to stderr and stdout for testing
    eprintln!("CODE BEAUTIFIER SERVER STARTING (stderr)");
    println!("CODE BEAUTIFIER SERVER STARTING (stdout)");
    
    info!("=== CODE BEAUTIFIER SERVER INITIALIZING ===");
    error!("Logger initialized at info level - this error log is for visibility testing");

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| String::from("3030"))
        .parse()
        .expect("Unable to parse PORT");

    info!("Starting Code Beautifier Server on port {}", port);

    // Check for PostgreSQL URI
    let postgres_uri = std::env::var("POSTGRES_URI");
    match &postgres_uri {
        Ok(uri) => info!("PostgreSQL URI is set. URI length: {} chars", uri.len()),
        Err(_) => error!("POSTGRES_URI environment variable is NOT set!"),
    }

    let config = ServerConfig {
        expiry_days: std::env::var("EXPIRY_DAYS")
            .unwrap_or_else(|_| String::from("1"))
            .parse()
            .expect("Unable to parse EXPIRY_DAYS"),
        database: match postgres_uri {
            Ok(uri) => {
                info!("PostgreSQL URI found, initializing database connection...");
                match Database::new(&uri).await {
                    Ok(db) => {
                        info!("PostgreSQL database connection SUCCESSFUL!");
                        Some(db)
                    },
                    Err(e) => {
                        error!("PostgreSQL connection FAILED: {}", e);
                        error!("Starting without database persistence");
                        None
                    }
                }
            },
            Err(_) => {
                warn!("No POSTGRES_URI environment variable found. Running without persistence!");
                None
            },
        },
    };

    info!("Code Beautifier Server initialized and ready to handle requests");
    info!("=== SERVER STARTUP COMPLETE ===");
    
    warp::serve(server(config)).run(([0, 0, 0, 0], port)).await;
}
