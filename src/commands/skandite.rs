use std::sync::Arc;

use anyhow::Error;
use normalize_url::normalizer::UrlNormalizer;
use serenity::async_trait;
use serenity::client::Context;
use serenity::model::prelude::{EmojiId, EmojiIdentifier, Message};
use sqlx::MySqlPool;

use crate::db::skandite::Skandite;
use crate::utils::extract_url;
use crate::MessageCommand;

pub struct SkanditeCommand {
    pub db_pool: Arc<MySqlPool>,
    pub discord_skandite_emoji_id: u64,
}

#[async_trait]
impl MessageCommand for SkanditeCommand {
    async fn handle(&self, _ctx: &Context, message: &Message) -> Result<Option<String>, Error> {
        match extract_url(&message.content) {
            None => Ok(None),
            Some(extracted_url) => {
                let url = normalize_url(extracted_url)?;
                match Skandite::find_by_url(&self.db_pool, &url).await? {
                    None => {
                        Skandite::insert(&self.db_pool, &url, &message.author.name).await?;
                        Ok(None)
                    }
                    Some(skandite) => {
                        if skandite.author == message.author.name || is_ignored(&skandite) {
                            Ok(None)
                        } else {
                            message
                                .react(
                                    _ctx,
                                    EmojiIdentifier {
                                        id: EmojiId(self.discord_skandite_emoji_id),
                                        name: "skandite".to_string(),
                                        animated: false,
                                    },
                                )
                                .await?;

                            Skandite::increment(&self.db_pool, skandite.id).await?;

                            Ok(None)
                        }
                    }
                }
            }
        }
    }
}

fn normalize_url(url: &str) -> Result<String, Error> {
    let normalizer = UrlNormalizer::new(url)?;

    let mut remove_params_regexes = vec!["utm_.*"];
    if url.contains("twitter.com/") {
        remove_params_regexes.push("s");
        remove_params_regexes.push("t");
    }
    if url.contains("youtube.com/") || url.contains("youtu.be/") {
        remove_params_regexes.push("t");
    }

    let mut normalized_url = normalizer.normalize(Some(&remove_params_regexes))?;
    if normalized_url.ends_with('/') {
        normalized_url.pop();
    }
    Ok(normalized_url)
}

fn is_ignored(skandite: &Skandite) -> bool {
    skandite.url.contains("tenor.com")
        || skandite.url.contains("giphy.com")
        || skandite.url.contains("warcraftlogs.com")
}

#[cfg(test)]
mod tests {
    use tokio_test::assert_ok;

    #[test]
    fn normalize_twitter_url() {
        let input = "https://twitter.com/fi_paris5/status/1470124228825526272?s=21&t=23";
        let output = assert_ok!(super::normalize_url(input));
        assert_eq!(
            output,
            "https://twitter.com/fi_paris5/status/1470124228825526272"
        );
    }

    #[test]
    fn normalize_youtube_com_url() {
        let input = "https://www.youtube.com/watch?v=VnxvRNbKMvA&t=36s";
        let output = assert_ok!(super::normalize_url(input));
        assert_eq!(output, "https://www.youtube.com/watch?v=VnxvRNbKMvA");
    }

    #[test]
    fn normalize_youtu_be_url() {
        let input = "https://youtu.be/VnxvRNbKMvA?t=37";
        let output = assert_ok!(super::normalize_url(input));
        assert_eq!(output, "https://youtu.be/VnxvRNbKMvA");
    }
}
