use crate::commands::SlashCommand;
use crate::utils::google::GoogleSearcher;
use crate::utils::google::SearchMode;
use anyhow::anyhow;
use anyhow::Error;
use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::model::interactions::application_command::ApplicationCommandInteractionDataOptionValue;
use serenity::model::interactions::application_command::ApplicationCommandOptionType;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use std::sync::Arc;

pub struct GoogleCommand {
    pub google_searcher: Arc<GoogleSearcher>,
}

#[async_trait]
impl SlashCommand for GoogleCommand {
    fn register(&self, command: &mut CreateApplicationCommand) {
        command
            .name("google")
            .description("Recherche Google")
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
        if interaction.data.name != "google" {
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
            ApplicationCommandInteractionDataOptionValue::String(q) => q,
            _ => return Err(anyhow!("wrong value type for terms option")),
        };

        match self
            .google_searcher
            .search(search_terms.to_string(), SearchMode::Web)
        {
            Ok(Some(r)) => Ok(Some(format!("{} - {}", r.title, r.link))),
            Ok(None) => Ok(Some("Pas de r??sultat".to_string())),
            Err(e) => Err(e),
        }
    }
}
