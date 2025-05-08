//! Backend PostgreSQL database handlers for persisting documents.

use anyhow::{bail, Result};
use log::{info, debug, warn, error};
use sqlx::{postgres::PgPoolOptions, PgPool};

/// Represents a document persisted in database storage.
#[derive(sqlx::FromRow, PartialEq, Eq, Clone, Debug)]
pub struct PersistedDocument {
    /// Text content of the document.
    pub text: String,
    /// Language of the document for editor syntax highlighting.
    pub language: Option<String>,
}

/// A driver for database operations wrapping a pool connection.
#[derive(Clone, Debug)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    /// Construct a new database from PostgreSQL connection URI.
    pub async fn new(uri: &str) -> Result<Self> {
        info!("Connecting to PostgreSQL database...");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(uri)
            .await?;
            
        info!("PostgreSQL connection established successfully!");
        
        // Verify connection by running a simple query
        match sqlx::query("SELECT 1").execute(&pool).await {
            Ok(_) => info!("PostgreSQL connection test successful"),
            Err(e) => {
                error!("PostgreSQL connection test failed: {}", e);
                return Err(e.into());
            }
        }
            
        // Create table if it doesn't exist (PostgreSQL doesn't support auto-migrate like SQLite)
        info!("Checking/creating document table...");
        match sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS document (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL,
                language TEXT
            )
            "#,
        )
        .execute(&pool)
        .await {
            Ok(_) => info!("Document table verified/created successfully"),
            Err(e) => {
                error!("Failed to create document table: {}", e);
                return Err(e.into());
            }
        }
        
        // Get table info for verification
        match sqlx::query("SELECT COUNT(*) FROM document").execute(&pool).await {
            Ok(_) => info!("Document table accessible, ready for operations"),
            Err(e) => {
                warn!("Could not query document table: {}", e);
                // Not returning error here since the table might be empty
            }
        }
        
        Ok(Database { pool })
    }

    /// Load the text of a document from the database.
    pub async fn load(&self, document_id: &str) -> Result<PersistedDocument> {
        debug!("Loading document with ID: {}", document_id);
        let result = sqlx::query_as(r#"SELECT text, language FROM document WHERE id = $1"#)
            .bind(document_id)
            .fetch_one(&self.pool)
            .await;
            
        match &result {
            Ok(_) => info!("Successfully loaded document: {}", document_id),
            Err(e) => warn!("Failed to load document {}: {}", document_id, e),
        }
        
        result.map_err(|e| e.into())
    }

    /// Store the text of a document in the database.
    pub async fn store(&self, document_id: &str, document: &PersistedDocument) -> Result<()> {
        debug!("Storing document with ID: {}", document_id);
        let result = sqlx::query(
            r#"
INSERT INTO
    document (id, text, language)
VALUES
    ($1, $2, $3)
ON CONFLICT(id) DO UPDATE SET
    text = excluded.text,
    language = excluded.language"#,
        )
        .bind(document_id)
        .bind(&document.text)
        .bind(&document.language)
        .execute(&self.pool)
        .await?;
        
        if result.rows_affected() != 1 {
            let msg = format!(
                "expected store() to receive 1 row affected, but it affected {} rows instead",
                result.rows_affected(),
            );
            error!("{}", msg);
            bail!(msg);
        }
        
        info!("Successfully stored document: {}", document_id);
        Ok(())
    }

    /// Count the number of documents in the database.
    pub async fn count(&self) -> Result<usize> {
        debug!("Counting documents in database");
        let row: (i64,) = sqlx::query_as("SELECT count(*) FROM document")
            .fetch_one(&self.pool)
            .await?;
            
        info!("Database contains {} documents", row.0);
        Ok(row.0 as usize)
    }
}
