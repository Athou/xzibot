use anyhow::Error;
use chrono::{DateTime, Utc};
use sqlx::{
    mysql::{MySqlQueryResult, MySqlRow},
    MySqlPool, Row,
};

pub struct Skandite {
    pub id: i64,
    pub author: String,
    pub posted_date: DateTime<Utc>,
    pub url: String,
    pub count: i64,
}

impl Skandite {
    pub async fn find_by_url(pool: &MySqlPool, url: &str) -> Result<Option<Skandite>, Error> {
        match sqlx::query("SELECT * FROM Skandite where url = ?")
            .bind(url)
            .map(|row: MySqlRow| Skandite {
                id: row.get("id"),
                author: row.get("author"),
                posted_date: row.get("postedDate"),
                url: row.get("url"),
                count: row.get("count"),
            })
            .fetch_optional(pool)
            .await?
        {
            Some(skandite) => Ok(Some(skandite)),
            None => Ok(None),
        }
    }

    pub async fn insert(
        pool: &MySqlPool,
        url: &str,
        author: &str,
    ) -> Result<MySqlQueryResult, Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO Skandite (`url`, `postedDate`, `author`, `count`)
            VALUES(?, ?, ?, ?)"#,
        )
        .bind(url)
        .bind(Utc::now())
        .bind(author)
        .bind(1)
        .execute(pool)
        .await?;
        Ok(result)
    }

    pub async fn increment(pool: &MySqlPool, id: i64) -> Result<MySqlQueryResult, Error> {
        let result = sqlx::query("UPDATE Skandite set count = count + 1 where id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(result)
    }
}
