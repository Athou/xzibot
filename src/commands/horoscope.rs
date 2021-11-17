use crate::commands::SlashCommand;
use anyhow::anyhow;
use anyhow::Error;
use linked_hash_map::LinkedHashMap;
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
                    .required(true);
                for sign in build_sign_map().keys() {
                    option.add_string_choice(sign, sign);
                }
                option
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
            .ok_or_else(|| anyhow!("missing sign option"))?
            .resolved
            .as_ref()
            .ok_or_else(|| anyhow!("missing sign option value"))?;

        let sign = match option {
            ApplicationCommandInteractionDataOptionValue::String(s) => s,
            _ => return Err(anyhow!("wrong value type for sign option")),
        };
        let sign_map = build_sign_map();
        let sign_number = sign_map
            .get(sign)
            .ok_or_else(|| anyhow!("cannot find sign mapping for {}", &sign))?;
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
            .ok_or_else(|| anyhow!("found no element matching selector in html"))?;

        let horoscope = element
            .text()
            .nth(1)
            .ok_or_else(|| anyhow!("cannot extract text of node"))?;
        Ok(Some(format!("{}{}", sign, horoscope.to_string())))
    }
}

fn build_sign_map() -> LinkedHashMap<String, String> {
    let mut map = LinkedHashMap::new();
    map.insert("Bélier".to_string(), "1".to_string());
    map.insert("Taureau".to_string(), "2".to_string());
    map.insert("Gémeaux".to_string(), "3".to_string());
    map.insert("Cancer".to_string(), "4".to_string());
    map.insert("Lion".to_string(), "5".to_string());
    map.insert("Vierge".to_string(), "6".to_string());
    map.insert("Balance".to_string(), "7".to_string());
    map.insert("Scorpion".to_string(), "8".to_string());
    map.insert("Sagittaire".to_string(), "9".to_string());
    map.insert("Capricorne".to_string(), "10".to_string());
    map.insert("Verseau".to_string(), "11".to_string());
    map.insert("Poissons".to_string(), "12".to_string());
    map
}
