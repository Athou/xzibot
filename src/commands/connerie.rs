use crate::utils::extract_url;
use crate::MessageCommand;
use anyhow::Error;
use rand::Rng;
use regex::Regex;
use serenity::async_trait;
use serenity::client::Context;
use serenity::model::prelude::Message;
use sqlx::mysql::MySqlQueryResult;
use sqlx::MySqlPool;
use sqlx::Row;
use std::sync::Arc;

const PROC_PERCENTAGE: u8 = 3;

#[derive(sqlx::FromRow)]
pub struct Connerie {
    pub id: i64,
    pub value: String,
    pub author: Option<String>,
}

pub struct ConnerieCommand {
    pub bot_name: String,
    pub db_pool: Arc<MySqlPool>,
}
impl ConnerieCommand {
    async fn should_trigger_save(&self, ctx: &Context, message: &Message) -> Result<bool, Error> {
        let trigger = message.content.chars().count() > 9
            && !message.mention_everyone
            && message.mention_roles.is_empty()
            && message.mention_channels.is_empty()
            && message.mentions.is_empty()
            && !has_url(&message.content)
            && !contains_emoji(&message.content)
            && !self.mentions_me(ctx, message).await?;
        Ok(trigger)
    }

    async fn trigger_save(&self, message: &Message) -> Result<MySqlQueryResult, Error> {
        let result = sqlx::query(
            r#"
            INSERT INTO Connerie (`author`, `value`)
            VALUES(?, ?)"#,
        )
        .bind(&message.author.name)
        .bind(&message.content)
        .execute(&*self.db_pool)
        .await?;
        Ok(result)
    }

    async fn should_trigger_say(&self, ctx: &Context, message: &Message) -> Result<bool, Error> {
        if self.mentions_me(ctx, message).await? {
            Ok(true)
        } else {
            Ok(rand::thread_rng().gen_range(1..100) <= PROC_PERCENTAGE)
        }
    }

    async fn trigger_say(
        &self,
        _ctx: &Context,
        _message: &Message,
    ) -> Result<Option<String>, Error> {
        let count: i64 = sqlx::query("SELECT count(*) from Connerie")
            .fetch_one(&*self.db_pool)
            .await?
            .get(0);

        if count <= 0 {
            Ok(None)
        } else {
            let id = rand::thread_rng().gen_range(0..count);
            let connerie = sqlx::query_as::<_, Connerie>("SELECT * FROM Connerie LIMIT 1 OFFSET ?")
                .bind(&id)
                .fetch_one(&*self.db_pool)
                .await?;
            Ok(Some(connerie.value))
        }
    }

    async fn mentions_me(&self, ctx: &Context, message: &Message) -> Result<bool, Error> {
        let mentions_me = message
            .content
            .to_lowercase()
            .contains(&self.bot_name.to_lowercase())
            || message.mentions_me(&ctx.http).await?;
        Ok(mentions_me)
    }
}

#[async_trait]
impl MessageCommand for ConnerieCommand {
    async fn handle(&self, ctx: &Context, message: &Message) -> Result<Option<String>, Error> {
        if self.should_trigger_save(ctx, message).await? {
            self.trigger_save(message).await?;
        }

        if self.should_trigger_say(ctx, message).await? {
            self.trigger_say(ctx, message).await
        } else {
            Ok(None)
        }
    }
}

fn has_url(input: &str) -> bool {
    extract_url(input).is_some()
}

fn contains_emoji(input: &str) -> bool {
    let re = Regex::new(r"\p{Emoji}").unwrap();
    re.is_match(input)
}
