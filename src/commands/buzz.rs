use crate::commands::SlashCommand;
use anyhow::Error;
use feed_rs::model::Entry;
use feed_rs::parser;
use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use sqlx::mysql::MySqlQueryResult;
use sqlx::MySqlPool;

use std::sync::Arc;

pub struct BuzzCommand {
    pub db_pool: Arc<MySqlPool>,
}
impl BuzzCommand {
    async fn entry_exists_by_guid(&self, guid: &str) -> Result<bool, Error> {
        let exists = sqlx::query("SELECT 1 FROM RSSFeed where guid = ?")
            .bind(guid)
            .fetch_optional(&*self.db_pool)
            .await?;
        Ok(exists.is_some())
    }

    async fn save_new_entry(&self, link: &str) -> Result<MySqlQueryResult, Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO RSSFeed (`guid`)
            VALUES(?)"#,
        )
        .bind(link)
        .execute(&*self.db_pool)
        .await?;
        Ok(result)
    }
}

#[async_trait]
impl SlashCommand for BuzzCommand {
    fn register(&self, command: &mut CreateApplicationCommand) {
        command.name("buzz").description("EXCLU!");
    }

    async fn handle(
        &self,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<Option<String>, Error> {
        if interaction.data.name != "buzz" {
            return Ok(None);
        }

        let xml = ureq::get("http://feeds.feedburner.com/jeanmarcmorandini/pExM?format=xml")
            .call()?
            .into_string()?;
        let feed = parser::parse(xml.as_bytes())?;

        let mut entry: Option<Entry> = None;
        for e in feed.entries {
            if !self.entry_exists_by_guid(&e.id).await? {
                entry = Some(e);
                break;
            }
        }

        match entry {
            None => Ok(None),
            Some(e) => match (e.title, e.links.get(0).map(|l| &l.href)) {
                (Some(title), Some(link)) => {
                    self.save_new_entry(&e.id).await?;
                    Ok(Some(format!("**EXCLU!** {} - {}", title.content, link)))
                }
                _ => Ok(None),
            },
        }
    }
}
