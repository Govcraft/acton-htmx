//! Test database utilities for SQLx
//!
//! Provides helpers for creating and managing test databases in integration tests.

use sqlx::PgPool;
use std::sync::Arc;

/// Test database helper for SQLx integration tests
///
/// This helper creates a temporary test database, runs migrations, and provides
/// a connection pool for testing. The database is automatically dropped when the
/// helper is dropped.
///
/// # Example
///
/// ```rust,no_run
/// use acton_htmx::testing::TestDatabase;
///
/// #[tokio::test]
/// async fn test_user_creation() {
///     let test_db = TestDatabase::new().await.unwrap();
///     let pool = test_db.pool();
///
///     // Run your tests with the pool
///     // Database will be dropped automatically when test_db goes out of scope
/// }
/// ```
pub struct TestDatabase {
    pool: Arc<PgPool>,
    database_name: String,
    postgres_url: String,
}

impl TestDatabase {
    /// Create a new test database with migrations
    ///
    /// This creates a temporary database with a unique name, runs all migrations,
    /// and returns a connection pool.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Cannot connect to PostgreSQL
    /// - Cannot create database
    /// - Migrations fail
    pub async fn new() -> anyhow::Result<Self> {
        Self::with_migrations(true).await
    }

    /// Create a new test database without running migrations
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Cannot connect to PostgreSQL
    /// - Cannot create database
    pub async fn without_migrations() -> anyhow::Result<Self> {
        Self::with_migrations(false).await
    }

    async fn with_migrations(run_migrations: bool) -> anyhow::Result<Self> {
        // Generate unique database name
        let database_name = format!("test_db_{}", uuid::Uuid::new_v4().simple());

        // Connect to postgres database
        let postgres_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/postgres".to_string());

        let pool = PgPool::connect(&postgres_url).await?;

        // Create test database
        sqlx::query(&format!("CREATE DATABASE {database_name}"))
            .execute(&pool)
            .await?;

        // Connect to the new database
        let test_db_url = postgres_url.replace("/postgres", &format!("/{database_name}"));
        let test_pool = PgPool::connect(&test_db_url).await?;

        // Run migrations if requested
        if run_migrations {
            sqlx::migrate!("../migrations")
                .run(&test_pool)
                .await?;
        }

        Ok(Self {
            pool: Arc::new(test_pool),
            database_name,
            postgres_url,
        })
    }

    /// Get a connection pool to the test database
    #[must_use]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Get the database name
    #[must_use]
    pub fn name(&self) -> &str {
        &self.database_name
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        // Drop the database asynchronously in a blocking context
        let database_name = self.database_name.clone();
        let postgres_url = self.postgres_url.clone();

        // Close the connection pool before dropping the database
        // This is important because PostgreSQL won't drop a database with active connections
        let pool = Arc::clone(&self.pool);
        std::mem::drop(pool);

        // Use a blocking task to drop the database
        // This is acceptable in Drop because it only runs during test cleanup
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime for cleanup");
            rt.block_on(async {
                match PgPool::connect(&postgres_url).await {
                    Ok(pool) => {
                        // Force disconnect all connections to the test database
                        let force_disconnect = format!(
                            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{database_name}'"
                        );
                        let _ = sqlx::query(&force_disconnect).execute(&pool).await;

                        // Drop the database
                        let drop_query = format!("DROP DATABASE IF EXISTS {database_name}");
                        match sqlx::query(&drop_query).execute(&pool).await {
                            Ok(_) => {
                                tracing::debug!("Successfully dropped test database: {database_name}");
                            }
                            Err(e) => {
                                tracing::warn!("Failed to drop test database {database_name}: {e}");
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to connect for cleanup of {database_name}: {e}");
                    }
                }
            });
        });
    }
}

/// Create a test database pool for SQLite (in-memory)
///
/// This is useful for fast tests that don't need PostgreSQL.
///
/// # Example
///
/// ```rust,no_run
/// use acton_htmx::testing::create_sqlite_pool;
///
/// #[tokio::test]
/// async fn test_with_sqlite() {
///     let pool = create_sqlite_pool().await.unwrap();
///     // Use pool for testing
/// }
/// ```
///
/// # Errors
///
/// Returns an error if the SQLite pool cannot be created
#[cfg(feature = "sqlite")]
pub async fn create_sqlite_pool() -> anyhow::Result<sqlx::SqlitePool> {
    use sqlx::sqlite::SqlitePoolOptions;

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(":memory:")
        .await?;

    Ok(pool)
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "sqlite")]
    #[tokio::test]
    async fn test_sqlite_pool() {
        use super::*;
        let pool = create_sqlite_pool().await.unwrap();

        // Verify we can query the database
        let result: (i32,) = sqlx::query_as("SELECT 1")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(result.0, 1);
    }
}
