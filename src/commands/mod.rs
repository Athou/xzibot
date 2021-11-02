use anyhow::Error;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;

pub mod blague;
pub mod episodes;
pub mod google_image;
pub mod horoscope;
pub mod youtube;

pub trait SlashCommand: Send + Sync {
    fn register(&self, command: &mut CreateApplicationCommand);

    fn handle(&self, interaction: &ApplicationCommandInteraction) -> Result<Option<String>, Error>;
}
