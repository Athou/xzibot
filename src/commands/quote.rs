use crate::commands::SlashCommand;
use crate::db::quote::Quote;
use anyhow::anyhow;
use anyhow::Error;
use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::model::interactions::application_command::ApplicationCommandInteractionDataOption;
use serenity::model::interactions::application_command::ApplicationCommandInteractionDataOptionValue;
use serenity::model::interactions::application_command::ApplicationCommandOptionType;
use serenity::model::interactions::application_command::ApplicationCommandType;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use sqlx::MySqlPool;
use std::sync::Arc;

pub struct QuoteCommand {
    pub db_pool: Arc<MySqlPool>,
}

impl QuoteCommand {
    fn format_quote(quote: &Quote) -> String {
        format!("{}. {}", quote.number, quote.quote)
    }

    async fn trigger_get(
        &self,
        command: &ApplicationCommandInteractionDataOption,
    ) -> Result<Option<String>, Error> {
        let option = command
            .options
            .get(0)
            .ok_or(anyhow!("missing command sub option"))?
            .resolved
            .as_ref()
            .ok_or(anyhow!("missing command sub option value"))?;

        let number = match option {
            ApplicationCommandInteractionDataOptionValue::String(s) => s,
            _ => return Err(anyhow!("wrong value type for command sub option")),
        };

        let quote = Quote::find_by_number(&self.db_pool, number.parse::<i64>()?).await?;
        match quote {
            None => Ok(Some("Pas de résultat!".to_string())),
            Some(q) => Ok(Some(QuoteCommand::format_quote(&q))),
        }
    }

    async fn trigger_random(&self) -> Result<Option<String>, Error> {
        let quote = Quote::random(&self.db_pool).await?;
        match quote {
            None => Ok(Some("Pas de résultat!".to_string())),
            Some(q) => Ok(Some(QuoteCommand::format_quote(&q))),
        }
    }
}

#[async_trait]
impl SlashCommand for QuoteCommand {
    fn register(&self, command: &mut CreateApplicationCommand) {
        command
            .name("quote")
            .description("Citations")
            .create_option(|option| {
                option
                    .name("get")
                    .description("trouver une citation avec son identifiant")
                    .kind(ApplicationCommandOptionType::SubCommand)
                    .create_sub_option(|sub_option| {
                        sub_option
                            .name("id")
                            .description("id")
                            .kind(ApplicationCommandOptionType::String)
                            .required(true)
                    })
            })
            .create_option(|option| {
                option
                    .name("random")
                    .description("une citation au hasard")
                    .kind(ApplicationCommandOptionType::SubCommand)
            });
    }

    async fn handle(
        &self,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<Option<String>, Error> {
        if interaction.data.name != "quote" {
            return Ok(None);
        }

        let command = interaction
            .data
            .options
            .get(0)
            .ok_or(anyhow!("missing command option"))?;

        match command.name.as_str() {
            "get" => self.trigger_get(command).await,
            "random" => self.trigger_random().await,
            e => Err(anyhow!("unknown command {}", e)),
        }
    }
}

pub struct QuoteAddCommand {
    pub db_pool: Arc<MySqlPool>,
}

#[async_trait]
impl SlashCommand for QuoteAddCommand {
    fn register(&self, command: &mut CreateApplicationCommand) {
        command
            .name("Add Quote")
            .kind(ApplicationCommandType::Message);
    }

    async fn handle(
        &self,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<Option<String>, Error> {
        if interaction.data.name != "Add Quote" {
            return Ok(None);
        }

        let message = interaction
            .data
            .resolved
            .messages
            .values()
            .next()
            .ok_or(anyhow!("messages map is empty"))?;

        let quote = format!("<{}> {}", message.author.name, message.content);
        let i = Quote::save(&self.db_pool, &quote).await?;
        let reply = format!("Quote {} ajoutée : {}", i, quote);

        Ok(Some(reply))
    }
}
