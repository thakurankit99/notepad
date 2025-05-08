//! Backend PostgreSQL database handlers for persisting documents.

use anyhow::{bail, Result};
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
        log::info!("Initializing PostgreSQL database connection...");
        
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(uri)
            .await?;
        
        log::info!("PostgreSQL connection pool established");
            
        // Create table if it doesn't exist (PostgreSQL doesn't support auto-migrate like SQLite)
        log::info!("Ensuring document table exists...");
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS document (
                id TEXT PRIMARY KEY,
                text TEXT NOT NULL,
                language TEXT
            )
            "#,
        )
        .execute(&pool)
        .await?;
        
        log::info!("Document table ready");
        
        Ok(Database { pool })
    }

    /// Load the text of a document from the database.
    pub async fn load(&self, document_id: &str) -> Result<PersistedDocument> {
        log::debug!("Loading document with ID: {}", document_id);
        let document = sqlx::query_as(r#"SELECT text, language FROM document WHERE id = $1"#)
            .bind(document_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                log::error!("Failed to load document {}: {}", document_id, e);
                anyhow::Error::from(e)
            })?;
        
        log::debug!("Successfully loaded document: {}", document_id);
        Ok(document)
    }

    /// Store the text of a document in the database.
    pub async fn store(&self, document_id: &str, document: &PersistedDocument) -> Result<()> {
        log::debug!("Storing document with ID: {}", document_id);
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
            log::error!("{}", msg);
            bail!(msg);
        }
        
        log::debug!("Successfully stored document: {}", document_id);
        Ok(())
    }

    /// Count the number of documents in the database.
    pub async fn count(&self) -> Result<usize> {
        log::debug!("Counting documents in database");
        let row: (i64,) = sqlx::query_as("SELECT count(*) FROM document")
            .fetch_one(&self.pool)
            .await?;
        
        let count = row.0 as usize;
        log::debug!("Document count: {}", count);
        Ok(count)
    }
}
