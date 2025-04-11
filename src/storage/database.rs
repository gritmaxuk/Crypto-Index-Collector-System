use sqlx::{Pool, Postgres, postgres::PgPoolOptions, Row};
use chrono::{DateTime, Utc};
use tracing::info;

use crate::models::FeedData;
use crate::error::AppResult;

#[derive(Clone)]
pub struct Database {
    pool: Pool<Postgres>,
    enabled: bool,
}

impl Database {
    pub async fn new(db_url: &str, enabled: bool) -> AppResult<Self> {
        if !enabled {
            info!("[DATABASE] Persistence disabled in configuration");
            return Ok(Self {
                pool: Pool::connect(db_url).await?,
                enabled: false,
            });
        }

        info!("[DATABASE] Connecting to database at {}", db_url);
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(db_url)
            .await?;

        // Initialize the database schema
        Self::init_schema(&pool).await?;

        info!("[DATABASE] Connection established successfully");

        Ok(Self {
            pool,
            enabled,
        })
    }

    async fn init_schema(pool: &Pool<Postgres>) -> AppResult<()> {
        // First ensure the extension is available
        sqlx::query("CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;")
            .execute(pool)
            .await?;

        // Drop the existing table if it exists but is not a hypertable
        // This is needed because we can't convert an existing table with constraints to a hypertable
        sqlx::query(
            r#"
                DO $$
                BEGIN
                    IF EXISTS (
                        SELECT 1 FROM pg_tables WHERE tablename = 'raw_price_data'
                    ) AND NOT EXISTS (
                        SELECT 1 FROM timescaledb_information.hypertables WHERE hypertable_name = 'raw_price_data'
                    ) THEN
                        DROP TABLE raw_price_data CASCADE;
                        RAISE NOTICE 'Dropped existing non-hypertable to recreate as hypertable';
                    END IF;
                END;
                $$;
            "#
        )
        .execute(pool)
        .await?;

        // Create the table if it doesn't exist
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS raw_price_data (
                id SERIAL,
                feed_id TEXT NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL,
                price DOUBLE PRECISION NOT NULL,
                PRIMARY KEY (id, timestamp)
            );
            "#
        )
        .execute(pool)
        .await?;

        // Try to convert to hypertable
        sqlx::query(
            r#"
            SELECT create_hypertable('raw_price_data', 'timestamp',
                                   chunk_time_interval => INTERVAL '1 day',
                                   if_not_exists => TRUE);
            "#
        )
        .execute(pool)
        .await?;

        // Create indexes
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_raw_price_data_timestamp ON raw_price_data (timestamp);
            "#
        )
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            CREATE UNIQUE INDEX IF NOT EXISTS idx_raw_price_data_feed_timestamp
            ON raw_price_data (feed_id, timestamp);
            "#
        )
        .execute(pool)
        .await?;

        info!("[DATABASE] Schema initialized with TimescaleDB hypertable");
        Ok(())
    }

    pub async fn save_price_data(&self, data: &FeedData) -> AppResult<()> {
        if !self.enabled {
            return Ok(());
        }

        // Use ON CONFLICT to handle duplicates
        sqlx::query(
            r#"
            INSERT INTO raw_price_data (feed_id, timestamp, price)
            VALUES ($1, $2, $3)
            ON CONFLICT (feed_id, timestamp)
            DO UPDATE SET price = EXCLUDED.price
            "#
        )
        .bind(&data.feed_id)
        .bind(data.timestamp)
        .bind(data.price)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn setup_retention_policy(&self, days: u32) -> AppResult<()> {
        if !self.enabled {
            return Ok(());
        }

        // Construct the SQL directly with the interval value
        let sql = format!(
            "SELECT add_retention_policy('raw_price_data', INTERVAL '{} days', if_not_exists => TRUE);",
            days
        );

        // Execute without parameter binding
        sqlx::query(&sql)
            .execute(&self.pool)
            .await?;

        info!("[DATABASE] Retention policy set to {} days", days);
        Ok(())
    }

    pub async fn get_recent_prices(&self, feed_id: &str, limit: i64) -> AppResult<Vec<(DateTime<Utc>, f64)>> {
        if !self.enabled {
            return Ok(Vec::new());
        }

        let rows = sqlx::query(
            "SELECT timestamp, price FROM raw_price_data WHERE feed_id = $1 ORDER BY timestamp DESC LIMIT $2"
        )
        .bind(feed_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let results = rows.into_iter()
            .map(|row| {
                let timestamp: DateTime<Utc> = row.try_get("timestamp").unwrap();
                let price: f64 = row.try_get("price").unwrap();
                (timestamp, price)
            })
            .collect();

        Ok(results)
    }
}
