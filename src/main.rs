use crate::commands::blague::BlagueCommand;
use crate::commands::buzz::BuzzCommand;
use crate::commands::connerie::ConnerieCommand;
use crate::commands::eight_ball::EightBallCommand;
use crate::commands::episodes::EpisodesCommand;
use crate::commands::google::GoogleCommand;
use crate::commands::google_image::GoogleImageCommand;
use crate::commands::horoscope::HoroscopeCommand;
use crate::commands::skandite::SkanditeCommand;
use crate::commands::youtube::YoutubeCommand;
use crate::commands::MessageCommand;
use crate::handler::Handler;
use crate::utils::google::GoogleSearcher;
use commands::quote::QuoteAddCommand;
use commands::quote::QuoteCommand;
use commands::SlashCommand;
use figment::providers::Env;
use figment::providers::Format;
use figment::providers::Toml;
use figment::Figment;
use serde::Deserialize;
use serenity::client::Client;
use serenity::framework::StandardFramework;
use sqlx::MySqlPool;
use std::sync::Arc;

mod commands;
mod db;
mod handler;
mod utils;

#[derive(Deserialize)]
struct Config {
    bot_name: String,
    database_url: String,
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

    let bot_name = Arc::new(config.bot_name);
    let db_pool = Arc::new(MySqlPool::connect(&config.database_url).await.unwrap());

    let google_searcher = Arc::new(GoogleSearcher {
        google_key: config.google_key,
        google_cse_id: config.google_cse_id,
    });

    let slash_commands: Vec<Box<dyn SlashCommand>> = vec![
        Box::new(BlagueCommand {
            blagues_api_token: config.blagues_api_token,
        }),
        Box::new(BuzzCommand {
            db_pool: db_pool.clone(),
        }),
        Box::new(ConnerieCommand {
            bot_name: bot_name.clone(),
            db_pool: db_pool.clone(),
        }),
        Box::new(EightBallCommand {}),
        Box::new(EpisodesCommand {}),
        Box::new(GoogleCommand {
            google_searcher: google_searcher.clone(),
        }),
        Box::new(GoogleImageCommand {
            google_searcher: google_searcher.clone(),
        }),
        Box::new(HoroscopeCommand {}),
        Box::new(QuoteCommand {
            db_pool: db_pool.clone(),
        }),
        Box::new(QuoteAddCommand {
            db_pool: db_pool.clone(),
        }),
        Box::new(YoutubeCommand {
            google_searcher: google_searcher.clone(),
        }),
    ];

    let message_commands: Vec<Box<dyn MessageCommand>> = vec![
        Box::new(ConnerieCommand {
            bot_name: bot_name.clone(),
            db_pool: db_pool.clone(),
        }),
        Box::new(SkanditeCommand {
            db_pool: db_pool.clone(),
        }),
    ];

    let handler = Handler {
        slash_commands,
        message_commands,
    };

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
