use crate::db::connerie::Connerie;
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
use sqlx::MySqlPool;
use std::sync::Arc;

const PROC_PERCENTAGE: u8 = 3;
const MIN_RAND_TERMS_LENGTH: usize = 4;

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

    async fn should_trigger_say(&self, ctx: &Context, message: &Message) -> Result<bool, Error> {
        if self.mentions_me(ctx, message).await? {
            Ok(true)
        } else {
            Ok(rand::thread_rng().gen_range(1..100) <= PROC_PERCENTAGE)
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
            Connerie::insert(&self.db_pool, &message.author.name, &message.content).await?;
        }

        if self.should_trigger_say(ctx, message).await? {
            Connerie::random(&self.db_pool).await
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
        let connerie = Connerie::search(&self.db_pool, &tokens).await?;
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
