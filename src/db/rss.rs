use anyhow::Error;
use sqlx::{mysql::MySqlQueryResult, MySqlPool};

pub struct RssFeedEntry;

impl RssFeedEntry {
    pub async fn save(pool: &MySqlPool, guid: &str) -> Result<MySqlQueryResult, Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO RSSFeed (`guid`)
            VALUES(?)"#,
        )
        .bind(guid)
        .execute(pool)
        .await?;
        Ok(result)
    }

    pub async fn exists_by_guid(pool: &MySqlPool, guid: &str) -> Result<bool, Error> {
        let exists = sqlx::query("SELECT 1 FROM RSSFeed where guid = ?")
            .bind(guid)
            .fetch_optional(pool)
            .await?;
        Ok(exists.is_some())
    }
}
