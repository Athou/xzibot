use crate::commands::SlashCommand;
use crate::utils::google::GoogleSearcher;
use crate::utils::google::SearchMode;
use anyhow::anyhow;
use anyhow::Error;
use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::CommandDataOptionValue;
use std::sync::Arc;

pub struct GoogleImageCommand {
    pub google_searcher: Arc<GoogleSearcher>,
}

#[async_trait]
impl SlashCommand for GoogleImageCommand {
    fn register(&self, command: &mut CreateApplicationCommand) {
        command
            .name("image")
            .description("Recherche Google image")
            .create_option(|option| {
                option
                    .name("terms")
                    .description("Que chercher ?")
                    .kind(CommandOptionType::String)
                    .required(true)
            });
    }

    async fn handle(
        &self,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<Option<String>, Error> {
        if interaction.data.name != "image" {
            return Ok(None);
        }

        let option = interaction
            .data
            .options
            .get(0)
            .ok_or_else(|| anyhow!("missing terms option"))?
            .resolved
            .as_ref()
            .ok_or_else(|| anyhow!("missing terms option value"))?;

        let search_terms = match option {
            CommandDataOptionValue::String(q) => q,
            _ => return Err(anyhow!("wrong value type for terms option")),
        };

        match self
            .google_searcher
            .search(search_terms.to_string(), SearchMode::Image)
        {
            Ok(Some(r)) => Ok(Some(r.link)),
            Ok(None) => Ok(Some("Pas de résultat".to_string())),
            Err(e) => Err(e),
        }
    }
}
