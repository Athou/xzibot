use crate::commands::SlashCommand;
use anyhow::anyhow;
use anyhow::Error;
use chrono::{DateTime, Utc};
use chrono_humanize::HumanTime;
use serde::Deserialize;
use serenity::builder::CreateApplicationCommand;
use serenity::model::interactions::application_command::ApplicationCommandInteractionDataOptionValue;
use serenity::model::interactions::application_command::ApplicationCommandOptionType;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;

#[derive(Deserialize)]
struct TVMazeSearch {
    name: String,
    externals: TVMazeExternals,
    #[serde(rename = "_embedded")]
    embedded: TVMazeEmbedded,
}

#[derive(Deserialize)]
struct TVMazeExternals {
    imdb: String,
}

#[derive(Deserialize)]
struct TVMazeEmbedded {
    episodes: Vec<TVMazeEpisode>,
}

#[derive(Deserialize)]
struct TVMazeEpisode {
    name: String,
    season: u8,
    number: u8,
    airstamp: Option<DateTime<Utc>>,
}

pub struct EpisodesCommand {}
impl SlashCommand for EpisodesCommand {
    fn register(&self, command: &mut CreateApplicationCommand) {
        command
            .name("next")
            .description("Chercher la date de diffusion du prochain épisode d'une série")
            .create_option(|option| {
                option
                    .name("tv_show")
                    .description("Le nom de la série")
                    .kind(ApplicationCommandOptionType::String)
                    .required(true)
            });
    }

    fn handle(&self, interaction: &ApplicationCommandInteraction) -> Result<Option<String>, Error> {
        if interaction.data.name != "next" {
            return Ok(None);
        }

        let option = interaction
            .data
            .options
            .get(0)
            .ok_or(anyhow!("missing tv_show option"))?
            .resolved
            .as_ref()
            .ok_or(anyhow!("missing tv_show option value"))?;

        let search_terms = match option {
            ApplicationCommandInteractionDataOptionValue::String(q) => q,
            _ => return Err(anyhow!("wrong value type for tv_show option")),
        };

        let url = format!(
            "https://api.tvmaze.com/singlesearch/shows?q={}&embed=episodes",
            search_terms
        );

        let mut lines: Vec<String> = Vec::new();
        match ureq::get(&url).call() {
            Ok(r) => {
                let search_result = r.into_json::<TVMazeSearch>()?;
                lines.push(format!(
                    "{} <http://www.imdb.com/title/{}>",
                    search_result.name, search_result.externals.imdb
                ));

                let (previous, next) =
                    find_previous_and_next_episodes(&search_result.embedded.episodes);

                lines.push(match next {
                    None => "Next Episode: N/A".to_string(),
                    Some(ep) => format!("Next Episode: {}", build_episode_line(ep)),
                });
                lines.push(match previous {
                    None => "Previous Episode: N/A".to_string(),
                    Some(ep) => format!("Previous Episode: {}", build_episode_line(ep)),
                });
            }
            Err(e) => {
                if let ureq::Error::Status(404, _) = e {
                    lines.push("Pas de résultat".to_string());
                } else {
                    return Err(anyhow!("{}", e));
                }
            }
        }

        Ok(Some(lines.join("\n")))
    }
}

fn find_previous_and_next_episodes(
    episodes: &Vec<TVMazeEpisode>,
) -> (Option<&TVMazeEpisode>, Option<&TVMazeEpisode>) {
    let mut previous: Option<&TVMazeEpisode> = None;
    let mut next: Option<&TVMazeEpisode> = None;

    let now = Utc::now();
    for episode in episodes {
        if let Some(d) = episode.airstamp {
            if d < now {
                previous = Some(episode);
            } else if let None = next {
                next = Some(episode);
            }
        }
    }

    (previous, next)
}

fn build_episode_line(episode: &TVMazeEpisode) -> String {
    let episode_id = format!("S{:02}E{:02}", episode.season, episode.number);
    match episode.airstamp {
        None => format!("{} - {} (?)", episode_id, episode.name),
        Some(date) => {
            let formatted_airdate = date.format("%Y-%m-%d %H:%M").to_string();
            let human_airdate = format!("{}", HumanTime::from(date));
            format!(
                "{} - {} ({}, {})",
                episode_id, episode.name, formatted_airdate, human_airdate
            )
        }
    }
}
