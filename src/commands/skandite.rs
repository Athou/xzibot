use crate::utils::extract_url;
use crate::MessageCommand;
use anyhow::Error;
use chrono::DateTime;
use chrono::Utc;
use chrono_humanize::HumanTime;
use normalize_url::normalizer::UrlNormalizer;
use serenity::async_trait;
use serenity::client::Context;
use serenity::model::prelude::Message;
use sqlx::mysql::MySqlQueryResult;
use sqlx::mysql::MySqlRow;
use sqlx::MySqlPool;
use sqlx::Row;
use std::sync::Arc;

pub struct Skandite {
    pub id: i64,
    pub author: String,
    pub posted_date: DateTime<Utc>,
    pub url: String,
    pub count: i64,
}

pub struct SkanditeCommand {
    pub db_pool: Arc<MySqlPool>,
}
impl SkanditeCommand {
    async fn find_skandite_by_url(&self, url: &str) -> Result<Option<Skandite>, Error> {
        match sqlx::query("SELECT * FROM Skandite where url = ?")
            .bind(url)
            .map(|row: MySqlRow| Skandite {
                id: row.get("id"),
                author: row.get("author"),
                posted_date: row.get("postedDate"),
                url: row.get("url"),
                count: row.get("count"),
            })
            .fetch_optional(&*self.db_pool)
            .await?
        {
            Some(skandite) => Ok(Some(skandite)),
            None => Ok(None),
        }
    }

    async fn save_new_skandite(&self, url: &str, author: &str) -> Result<MySqlQueryResult, Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO Skandite (`url`, `postedDate`, `author`, `count`)
            VALUES(?, ?, ?, ?)"#,
        )
        .bind(&url)
        .bind(Utc::now())
        .bind(&author)
        .bind(1)
        .execute(&*self.db_pool)
        .await?;
        Ok(result)
    }

    async fn increment_skandite(&self, skandite: &Skandite) -> Result<MySqlQueryResult, Error> {
        let result = sqlx::query("UPDATE Skandite set count = count + 1 where id = ?")
            .bind(skandite.id)
            .execute(&*self.db_pool)
            .await?;
        Ok(result)
    }
}

#[async_trait]
impl MessageCommand for SkanditeCommand {
    async fn handle(&self, _ctx: &Context, message: &Message) -> Result<Option<String>, Error> {
        match extract_url(&message.content) {
            None => Ok(None),
            Some(extracted_url) => {
                let url = normalize_url(extracted_url)?;
                match self.find_skandite_by_url(&url).await? {
                    None => {
                        self.save_new_skandite(&url, &message.author.name).await?;
                        Ok(None)
                    }
                    Some(skandite) => {
                        if skandite.author == message.author.name || is_ignored(&skandite) {
                            Ok(None)
                        } else {
                            self.increment_skandite(&skandite).await?;
                            let message = format!(
                                "**Skandite!** <{}> linked {} by {} ({}x).",
                                url,
                                HumanTime::from(skandite.posted_date),
                                skandite.author,
                                skandite.count
                            );
                            Ok(Some(message))
                        }
                    }
                }
            }
        }
    }
}

fn normalize_url(url: &str) -> Result<String, Error> {
    let normalizer = UrlNormalizer::new(url)?;
    let mut normalized_url = normalizer.normalize(Some(&["utm_.*"]))?;
    if normalized_url.ends_with("/") {
        normalized_url.pop();
    }
    Ok(normalized_url)
}

fn is_ignored(skandite: &Skandite) -> bool {
    skandite.url.contains("tenor.com") || skandite.url.contains("giphy.com")
}
