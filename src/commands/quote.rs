use crate::commands::SlashCommand;
use anyhow::anyhow;
use anyhow::Error;
use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::model::interactions::application_command::ApplicationCommandInteractionDataOptionValue;
use serenity::model::interactions::application_command::ApplicationCommandOptionType;
use serenity::model::interactions::application_command::ApplicationCommandType;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use sqlx::MySqlPool;
use sqlx::Row;
use std::sync::Arc;

#[derive(sqlx::FromRow)]
pub struct Quote {
    pub id: i64,
    pub quote: String,
    pub number: i64,
}

pub struct QuoteCommand {
    pub db_pool: Arc<MySqlPool>,
}

impl QuoteCommand {
    async fn find_quote_by_number(&self, number: i64) -> Result<Option<String>, Error> {
        let quote = sqlx::query_as::<_, Quote>("SELECT * FROM Quote where number = ?")
            .bind(&number)
            .fetch_optional(&*self.db_pool)
            .await?;
        Ok(quote.map(|q| q.quote))
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
                    .description("get")
                    .kind(ApplicationCommandOptionType::SubCommand)
                    .create_sub_option(|sub_option| {
                        sub_option
                            .name("id")
                            .description("id")
                            .kind(ApplicationCommandOptionType::String)
                            .required(true)
                    })
            });
    }

    async fn handle(
        &self,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<Option<String>, Error> {
        if interaction.data.name != "quote" {
            return Ok(None);
        }

        let option = &interaction
            .data
            .options
            .get(0)
            .ok_or(anyhow!("missing command option"))?
            .options
            .get(0)
            .ok_or(anyhow!("missing command sub option"))?
            .resolved
            .as_ref()
            .ok_or(anyhow!("missing command sub option value"))?;

        let number = match option {
            ApplicationCommandInteractionDataOptionValue::String(s) => s,
            _ => return Err(anyhow!("wrong value type for ommand sub option")),
        };

        let quote = self.find_quote_by_number(number.parse::<i64>()?).await?;
        match quote {
            None => Ok(Some("Pas de résultat!".to_string())),
            Some(q) => Ok(Some(q)),
        }
    }
}

pub struct QuoteAddCommand {
    pub db_pool: Arc<MySqlPool>,
}

impl QuoteAddCommand {
    async fn count(&self) -> Result<i64, Error> {
        let count: i64 = sqlx::query("SELECT count(*) from Quote")
            .fetch_one(&*self.db_pool)
            .await?
            .get(0);
        Ok(count)
    }

    async fn save_quote(&self, quote: &str) -> Result<i64, Error> {
        let number = self.count().await? + 1;

        sqlx::query(
            r#"
            INSERT INTO Quote (`quote`, `number`)
            VALUES(?, ?)"#,
        )
        .bind(quote)
        .bind(number)
        .execute(&*self.db_pool)
        .await?;

        Ok(number)
    }
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
        let message = interaction
            .data
            .resolved
            .messages
            .values()
            .next()
            .ok_or(anyhow!("messages map is empty"))?;

        let quote = format!("<{}> {}", message.author.name, message.content);
        let i = self.save_quote(&quote).await?;
        let reply = format!("Quote {} ajoutée : {}", i, quote);

        Ok(Some(reply))
    }
}
