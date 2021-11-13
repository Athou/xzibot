use crate::commands::SlashCommand;
use crate::db::rss::RssFeedEntry;
use anyhow::Error;
use feed_rs::model::Entry;
use feed_rs::parser;
use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use sqlx::MySqlPool;

use std::sync::Arc;

pub struct BuzzCommand {
    pub db_pool: Arc<MySqlPool>,
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
            if !RssFeedEntry::exists_by_guid(&self.db_pool, &e.id).await? {
                entry = Some(e);
                break;
            }
        }

        match entry {
            None => Ok(Some("Plus d'exclus pour le moment :(".to_string())),
            Some(e) => match (e.title, e.links.get(0).map(|l| &l.href)) {
                (Some(title), Some(link)) => {
                    RssFeedEntry::save(&self.db_pool, &e.id).await?;
                    Ok(Some(format!("**EXCLU!** {} - {}", title.content, link)))
                }
                _ => Ok(None),
            },
        }
    }
}
