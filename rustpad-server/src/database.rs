//! Backend PostgreSQL database handlers for persisting documents.

use anyhow::{bail, Result};
use log::{info, error, debug};
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
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(uri)
            .await?;
            
        info!("PostgreSQL connection established");
        
        // Create table if it doesn't exist
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
        
        // Verify table is accessible
        sqlx::query("SELECT COUNT(*) FROM document")
            .execute(&pool)
            .await?;
        
        Ok(Database { pool })
    }

    /// Load the text of a document from the database.
    pub async fn load(&self, document_id: &str) -> Result<PersistedDocument> {
        debug!("Loading document: {}", document_id);
        let result = sqlx::query_as(r#"SELECT text, language FROM document WHERE id = $1"#)
            .bind(document_id)
            .fetch_one(&self.pool)
            .await;
        
        if result.is_err() {
            debug!("Document not found: {}", document_id);
        }
        
        result.map_err(|e| e.into())
    }

    /// Store the text of a document in the database.
    pub async fn store(&self, document_id: &str, document: &PersistedDocument) -> Result<()> {
        debug!("Storing document: {}", document_id);
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
                "expected 1 row affected, but got {} rows",
                result.rows_affected(),
            );
            error!("{}", msg);
            bail!(msg);
        }
        
        Ok(())
    }

    /// Count the number of documents in the database.
    pub async fn count(&self) -> Result<usize> {
        let row: (i64,) = sqlx::query_as("SELECT count(*) FROM document")
            .fetch_one(&self.pool)
            .await?;
            
        Ok(row.0 as usize)
    }
}
