use crate::commands::SlashCommand;
use anyhow::Error;
use rand::Rng;
use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;

pub struct EightBallCommand {}

#[async_trait]
impl SlashCommand for EightBallCommand {
    fn register(&self, command: &mut CreateApplicationCommand) {
        command.name("8ball").description("Magic 8-Ball");
    }

    async fn handle(
        &self,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<Option<String>, Error> {
        if interaction.data.name != "8ball" {
            return Ok(None);
        }

        let answers = [
            "Essaye plus tard",
            "Essaye encore",
            "Pas d'avis",
            "C'est ton destin",
            "Le sort en est jeté",
            "Une chance sur deux",
            "Repose ta question",
            "D'après moi oui",
            "C'est certain",
            "Oui absolument",
            "Tu peux compter dessus",
            "Sans aucun doute",
            "Très probable",
            "Oui",
            "C'est bien parti",
            "C'est non",
            "Peu probable",
            "Faut pas rêver",
            "N'y compte pas",
            "Impossible",
        ];

        let i = rand::thread_rng().gen_range(0..answers.len());
        Ok(Some(format!("🎱 {} 🎱", answers[i])))
    }
}
