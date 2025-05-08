use rustpad_server::{server, database::Database, ServerConfig};
use log::{info, warn};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| String::from("3030"))
        .parse()
        .expect("Unable to parse PORT");

    info!("Starting Code Beautifier Server on port {}", port);

    let config = ServerConfig {
        expiry_days: std::env::var("EXPIRY_DAYS")
            .unwrap_or_else(|_| String::from("1"))
            .parse()
            .expect("Unable to parse EXPIRY_DAYS"),
        database: match std::env::var("POSTGRES_URI") {
            Ok(uri) => {
                info!("PostgreSQL URI found, initializing database connection...");
                let db = Database::new(&uri)
                    .await
                    .expect("Unable to connect to POSTGRES_URI");
                info!("PostgreSQL database connection established successfully!");
                Some(db)
            },
            Err(_) => {
                warn!("No POSTGRES_URI environment variable found. Running without persistence!");
                None
            },
        },
    };

    info!("Code Beautifier Server initialized and ready to handle requests");
    warp::serve(server(config)).run(([0, 0, 0, 0], port)).await;
}
