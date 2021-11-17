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
            .ok_or_else(|| anyhow!("missing command sub option"))?
            .resolved
            .as_ref()
            .ok_or_else(|| anyhow!("missing command sub option value"))?;

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

    async fn trigger_find(
        &self,
        command: &ApplicationCommandInteractionDataOption,
    ) -> Result<Option<String>, Error> {
        let option = command
            .options
            .get(0)
            .ok_or_else(|| anyhow!("missing command sub option"))?
            .resolved
            .as_ref()
            .ok_or_else(|| anyhow!("missing command sub option value"))?;

        let search_terms = match option {
            ApplicationCommandInteractionDataOptionValue::String(s) => s,
            _ => return Err(anyhow!("wrong value type for command sub option")),
        };

        let tokens: Vec<&str> = search_terms.split(' ').collect();
        let quotes = Quote::search(&self.db_pool, &tokens[..]).await?;
        if quotes.is_empty() {
            return Ok(Some("Pas de résultat.".to_string()));
        }

        let quote_ids = quotes
            .iter()
            .map(|q| q.number.to_string())
            .collect::<Vec<String>>();

        let message = format!(
            "Quotes correspondants à la recherche : {}",
            quote_ids.join(", ")
        );
        Ok(Some(message))
    }

    async fn trigger_random(&self) -> Result<Option<String>, Error> {
        let quote = Quote::random(&self.db_pool).await?;
        match quote {
            None => Ok(Some("Pas de résultat!".to_string())),
            Some(q) => Ok(Some(QuoteCommand::format_quote(&q))),
        }
    }

    async fn trigger_count(&self) -> Result<Option<String>, Error> {
        let count = Quote::count(&self.db_pool).await?;
        Ok(Some(format!(
            "Il y a {} citations dans la base de données.",
            count
        )))
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
                    .name("find")
                    .description("trouver une citation avec des mots clés")
                    .kind(ApplicationCommandOptionType::SubCommand)
                    .create_sub_option(|sub_option| {
                        sub_option
                            .name("terms")
                            .description("mots clés")
                            .kind(ApplicationCommandOptionType::String)
                            .required(true)
                    })
            })
            .create_option(|option| {
                option
                    .name("random")
                    .description("une citation au hasard")
                    .kind(ApplicationCommandOptionType::SubCommand)
            })
            .create_option(|option| {
                option
                    .name("count")
                    .description("combien de citations il y a dans la base de données")
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
            .ok_or_else(|| anyhow!("missing command option"))?;

        match command.name.as_str() {
            "get" => self.trigger_get(command).await,
            "find" => self.trigger_find(command).await,
            "random" => self.trigger_random().await,
            "count" => self.trigger_count().await,
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
            .ok_or_else(|| anyhow!("messages map is empty"))?;

        let quote = format!("<{}> {}", message.author.name, message.content);
        let i = Quote::save(&self.db_pool, &quote).await?;
        let reply = format!("Quote {} ajoutée : {}", i, quote);

        Ok(Some(reply))
    }
}
