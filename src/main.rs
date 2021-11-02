use crate::commands::blague::BlagueCommand;
use crate::commands::episodes::EpisodesCommand;
use crate::commands::google_image::GoogleImageCommand;
use crate::commands::horoscope::HoroscopeCommand;
use crate::commands::youtube::YoutubeCommand;
use crate::handler::Handler;
use crate::utils::google::GoogleSearcher;
use commands::SlashCommand;
use figment::providers::Env;
use figment::providers::Format;
use figment::providers::Toml;
use figment::Figment;
use serde::Deserialize;
use serenity::client::Client;
use serenity::framework::StandardFramework;
use std::sync::Arc;

mod commands;
mod handler;
mod utils;

#[derive(Deserialize)]
struct Config {
    discord_token: String,
    discord_application_id: u64,
    google_key: String,
    google_cse_id: String,
    blagues_api_token: String,
}

#[tokio::main]
async fn main() {
    let config: Config = Figment::new()
        .merge(Toml::file("xzibot.toml"))
        .merge(Env::prefixed("XZIBOT_"))
        .extract()
        .unwrap();

    let google_searcher = Arc::new(GoogleSearcher {
        google_key: config.google_key,
        google_cse_id: config.google_cse_id,
    });

    let mut slash_commands: Vec<Box<dyn SlashCommand>> = Vec::new();
    slash_commands.push(Box::new(BlagueCommand {
        blagues_api_token: config.blagues_api_token,
    }));
    slash_commands.push(Box::new(EpisodesCommand {}));
    slash_commands.push(Box::new(GoogleImageCommand {
        google_searcher: google_searcher.clone(),
    }));
    slash_commands.push(Box::new(HoroscopeCommand {}));
    slash_commands.push(Box::new(YoutubeCommand {
        google_searcher: google_searcher.clone(),
    }));

    let handler = Handler { slash_commands };

    let mut client = Client::builder(config.discord_token)
        .event_handler(handler)
        .application_id(config.discord_application_id)
        .framework(StandardFramework::new())
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
