use anyhow::Error;
use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::client::Context;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::Message;

pub mod blague;
pub mod buzz;
pub mod connerie;
pub mod eight_ball;
pub mod episodes;
pub mod google;
pub mod google_image;
pub mod horoscope;
pub mod quote;
pub mod skandite;
pub mod youtube;

#[async_trait]
pub trait SlashCommand: Send + Sync {
    fn register(&self, command: &mut CreateApplicationCommand);

    async fn handle(
        &self,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<Option<String>, Error>;
}

#[async_trait]
pub trait MessageCommand: Send + Sync {
    async fn handle(&self, ctx: &Context, message: &Message) -> Result<Option<String>, Error>;
}
