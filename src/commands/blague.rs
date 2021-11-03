use crate::commands::SlashCommand;
use anyhow::Error;
use serde::Deserialize;
use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;

#[derive(Deserialize)]
struct Joke {
    joke: String,
    answer: Option<String>,
}

pub struct BlagueCommand {
    pub blagues_api_token: String,
}

#[async_trait]
impl SlashCommand for BlagueCommand {
    fn register(&self, command: &mut CreateApplicationCommand) {
        command.name("blague").description("une blague au hasard!");
    }

    async fn handle(
        &self,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<Option<String>, Error> {
        if interaction.data.name != "blague" {
            return Ok(None);
        }

        let joke = ureq::get("https://www.blagues-api.fr/api/random")
            .set(
                "Authorization",
                &format!("Bearer {}", self.blagues_api_token),
            )
            .call()?
            .into_json::<Joke>()?;

        let mut lines: Vec<String> = Vec::new();
        lines.push(joke.joke);
        if let Some(a) = joke.answer {
            lines.push(a);
        }

        Ok(Some(lines.join("\n")))
    }
}
