use anyhow::Error;
use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::Message;

pub mod blague;
pub mod connerie;
pub mod episodes;
pub mod google;
pub mod google_image;
pub mod horoscope;
pub mod skandite;
pub mod youtube;

pub trait SlashCommand: Send + Sync {
    fn register(&self, command: &mut CreateApplicationCommand);

    fn handle(&self, interaction: &ApplicationCommandInteraction) -> Result<Option<String>, Error>;
}

#[async_trait]
pub trait MessageCommand: Send + Sync {
    async fn handle(&self, ctx: &Context, message: &Message) -> Result<Option<String>, Error>;
}
