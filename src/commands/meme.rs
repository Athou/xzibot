use crate::commands::SlashCommand;
use crate::db::connerie::Connerie;
use anyhow::anyhow;
use anyhow::Error;
use rand::Rng;
use serde::Deserialize;
use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::CommandDataOptionValue;
use sqlx::MySqlPool;
use std::sync::Arc;
use unidecode::unidecode;

#[derive(Deserialize)]
struct GetMemesResponse {
    data: GetMemeData,
}

#[derive(Deserialize)]
struct GetMemeData {
    memes: Vec<Meme>,
}

#[derive(Deserialize)]
struct Meme {
    id: String,
    box_count: u32,
}

#[derive(Deserialize)]
struct CaptionImageResponse {
    data: CaptionImageData,
}

#[derive(Deserialize)]
struct CaptionImageData {
    url: String,
}

pub struct MemeCommand {
    pub db_pool: Arc<MySqlPool>,
    pub imgflip_username: String,
    pub imgflip_password: String,
}

impl MemeCommand {
    async fn get_meme_text(
        &self,
        interaction: &ApplicationCommandInteraction,
        i: usize,
    ) -> Result<Option<String>, Error> {
        let text = match interaction.data.options.get(i) {
            Some(option_data) => {
                let option_data_value = option_data
                    .resolved
                    .as_ref()
                    .ok_or_else(|| anyhow!("missing option value"))?;
                let option_data_text = match option_data_value {
                    CommandDataOptionValue::String(q) => q,
                    _ => return Err(anyhow!("wrong value type for terms option")),
                };
                let tokens: Vec<&str> = option_data_text.split(' ').collect();
                match Connerie::search(&self.db_pool, &tokens[..]).await? {
                    Some(t) => Some(t),
                    None => Connerie::random(&self.db_pool).await?,
                }
            }
            _ => Connerie::random(&self.db_pool).await?,
        };

        match text {
            Some(t) => Ok(Some(unidecode(&t))),
            None => Ok(None),
        }
    }
}

#[async_trait]
impl SlashCommand for MemeCommand {
    fn register(&self, command: &mut CreateApplicationCommand) {
        command
            .name("meme")
            .description("Créer un meme")
            .create_option(|option| {
                option
                    .name("term1")
                    .description("phrase 1 au hasard contenant ce terme")
                    .kind(CommandOptionType::String)
            })
            .create_option(|option| {
                option
                    .name("term2")
                    .description("phrase 2 au hasard contenant ce terme")
                    .kind(CommandOptionType::String)
            });
    }

    async fn handle(
        &self,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<Option<String>, Error> {
        if interaction.data.name != "meme" {
            return Ok(None);
        }

        let get_memes_response = ureq::get("https://api.imgflip.com/get_memes")
            .call()?
            .into_json::<GetMemesResponse>()?;
        let memes: Vec<&Meme> = get_memes_response
            .data
            .memes
            .iter()
            .filter(|m| m.box_count == 2)
            .collect();
        let i = rand::thread_rng().gen_range(0..memes.len());
        let meme = memes.get(i).unwrap();

        let text0 = self.get_meme_text(interaction, 0).await?;
        let text1 = self.get_meme_text(interaction, 1).await?;

        let caption_image_response = ureq::post("https://api.imgflip.com/caption_image")
            .send_form(&[
                ("username", &self.imgflip_username),
                ("password", &self.imgflip_password),
                ("template_id", &meme.id),
                ("text0", &text0.unwrap_or_default()),
                ("text1", &text1.unwrap_or_default()),
            ])?
            .into_json::<CaptionImageResponse>()?;
        let url = caption_image_response.data.url;
        Ok(Some(url))
    }
}
