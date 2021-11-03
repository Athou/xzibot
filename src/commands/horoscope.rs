use crate::commands::SlashCommand;
use anyhow::anyhow;
use anyhow::Error;
use scraper::Html;
use scraper::Selector;
use serenity::async_trait;
use serenity::builder::CreateApplicationCommand;
use serenity::model::interactions::application_command::ApplicationCommandInteractionDataOptionValue;
use serenity::model::interactions::application_command::ApplicationCommandOptionType;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;

pub struct HoroscopeCommand {}

#[async_trait]
impl SlashCommand for HoroscopeCommand {
    fn register(&self, command: &mut CreateApplicationCommand) {
        command
            .name("horoscope")
            .description("Horoscope du jour")
            .create_option(|option| {
                option
                    .name("sign")
                    .description("Votre signe astrologique")
                    .kind(ApplicationCommandOptionType::String)
                    .required(true)
            });
    }

    async fn handle(
        &self,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<Option<String>, Error> {
        if interaction.data.name != "horoscope" {
            return Ok(None);
        }

        let option = interaction
            .data
            .options
            .get(0)
            .ok_or(anyhow!("missing sign option"))?
            .resolved
            .as_ref()
            .ok_or(anyhow!("missing sign option value"))?;

        let sign = match option {
            ApplicationCommandInteractionDataOptionValue::String(q) => q.to_lowercase(),
            _ => return Err(anyhow!("wrong value type for sign option")),
        };
        let sign_number =
            map_sign(&sign).ok_or(anyhow!("cannot find sign mapping for {}", &sign))?;
        let url = format!(
            "https://www.horoscope.com/us/horoscopes/general/horoscope-general-daily-today.aspx?sign={}",
            sign_number
        );

        let html = ureq::get(&url).call()?.into_string()?;
        let document = Html::parse_document(&html);
        let selector = Selector::parse(".main-horoscope > p:first-of-type").unwrap();
        let element = document
            .select(&selector)
            .next()
            .ok_or(anyhow!("found no element matching selector in html"))?;

        let horoscope = element
            .text()
            .nth(1)
            .ok_or(anyhow!("cannot extract text of node"))?;
        Ok(Some(horoscope.to_string()))
    }
}

fn map_sign(sign: &str) -> Option<u8> {
    match sign {
        "aries" | "belier" => Some(1),
        "taurus" | "taureau" => Some(2),
        "gemini" | "gemeaux" => Some(3),
        "cancer" => Some(4),
        "leo" | "lion" => Some(5),
        "virgo" | "vierge" => Some(6),
        "libra" | "balance" => Some(7),
        "scorpio" | "scorpion" => Some(8),
        "sagittarius" | "sagittaire" => Some(9),
        "capricorn" | "capricorne" => Some(10),
        "aquarius" | "verseau" => Some(11),
        "pisces" | "poisson" | "poissons" => Some(12),
        _ => None,
    }
}
