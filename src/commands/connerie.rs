use crate::utils::extract_url;
use crate::MessageCommand;
use crate::SlashCommand;
use anyhow::anyhow;
use anyhow::Error;
use rand::Rng;
use regex::Regex;
use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::interactions::application_command::ApplicationCommandInteractionDataOptionValue;
use serenity::model::interactions::application_command::ApplicationCommandOptionType;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::Message;
use sql_builder::SqlBuilder;
use sqlx::mysql::MySqlQueryResult;
use sqlx::MySqlPool;
use sqlx::Row;
use std::sync::Arc;

const PROC_PERCENTAGE: u8 = 3;
const MIN_RAND_TERMS_LENGTH: usize = 4;

#[derive(sqlx::FromRow)]
pub struct Connerie {
    pub id: i64,
    pub value: String,
    pub author: Option<String>,
}

pub struct ConnerieCommand {
    pub bot_name: Arc<String>,
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
        let count = self.count().await?;
        if count <= 0 {
            Ok(None)
        } else {
            let offset = rand::thread_rng().gen_range(0..count);
            let connerie = sqlx::query_as::<_, Connerie>("SELECT * FROM Connerie LIMIT 1 OFFSET ?")
                .bind(&offset)
                .fetch_one(&*self.db_pool)
                .await?;
            Ok(Some(connerie.value))
        }
    }

    async fn count(&self) -> Result<i64, Error> {
        let count: i64 = sqlx::query("SELECT count(*) from Connerie")
            .fetch_one(&*self.db_pool)
            .await?
            .get(0);
        Ok(count)
    }

    fn build_search_sql(&self, tokens: &Vec<&str>, with_spaces: bool) -> Result<String, Error> {
        let mut sql = SqlBuilder::select_from("Connerie");
        sql.field("*");
        for token in tokens {
            let like_pattern = if with_spaces {
                format!("% {} %", token.to_lowercase())
            } else {
                format!("%{}%", token.to_lowercase())
            };
            sql.and_where_like("LOWER(value)", like_pattern);
        }
        sql.sql()
    }

    async fn search(&self, tokens: &Vec<&str>) -> Result<Option<String>, Error> {
        let mut conneries = sqlx::query_as::<_, Connerie>(&self.build_search_sql(tokens, true)?)
            .fetch_all(&*self.db_pool)
            .await?;

        if conneries.is_empty() {
            conneries = sqlx::query_as::<_, Connerie>(&self.build_search_sql(tokens, false)?)
                .fetch_all(&*self.db_pool)
                .await?;
        }

        if conneries.is_empty() {
            return Ok(None);
        }

        let i = rand::thread_rng().gen_range(0..conneries.len());
        let connerie = conneries.into_iter().nth(i).map(|c| c.value);
        Ok(connerie)
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

#[async_trait]
impl SlashCommand for ConnerieCommand {
    fn register(&self, command: &mut CreateApplicationCommand) {
        command
            .name("rand")
            .description("Une phrase au hasard")
            .create_option(|option| {
                option
                    .name("terms")
                    .description("Que chercher ?")
                    .kind(ApplicationCommandOptionType::String)
                    .required(true)
            });
    }

    async fn handle(
        &self,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<Option<String>, Error> {
        if interaction.data.name != "rand" {
            return Ok(None);
        }

        let option = interaction
            .data
            .options
            .get(0)
            .ok_or(anyhow!("missing terms option"))?
            .resolved
            .as_ref()
            .ok_or(anyhow!("missing terms option value"))?;

        let search_terms = match option {
            ApplicationCommandInteractionDataOptionValue::String(q) => q,
            _ => return Err(anyhow!("wrong value type for terms option")),
        };

        if search_terms.chars().count() < MIN_RAND_TERMS_LENGTH {
            return Ok(Some(format!(
                "Requête trop courte, minimum {} caractères",
                MIN_RAND_TERMS_LENGTH
            )));
        }

        let tokens = search_terms.split(" ").collect();
        let connerie = self.search(&tokens).await?;
        match connerie {
            None => Ok(Some("Pas de résultat".to_string())),
            Some(c) => Ok(Some(c)),
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
