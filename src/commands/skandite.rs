use crate::db::skandite::Skandite;
use crate::utils::extract_url;
use crate::MessageCommand;
use anyhow::Error;
use chrono_humanize::HumanTime;
use normalize_url::normalizer::UrlNormalizer;
use serenity::async_trait;
use serenity::client::Context;
use serenity::model::prelude::Message;
use sqlx::MySqlPool;
use std::sync::Arc;

pub struct SkanditeCommand {
    pub db_pool: Arc<MySqlPool>,
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
                            Skandite::increment(&self.db_pool, skandite.id).await?;
                            let message = format!(
                                "**Skandite!** <{}> linked {} by {} ({}x).",
                                url,
                                HumanTime::from(skandite.posted_date),
                                skandite.author,
                                skandite.count
                            );
                            Ok(Some(message))
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
    if url.contains("twitter.com") {
        remove_params_regexes.push("s");
    }

    let mut normalized_url = normalizer.normalize(Some(&remove_params_regexes))?;
    if normalized_url.ends_with('/') {
        normalized_url.pop();
    }
    Ok(normalized_url)
}

fn is_ignored(skandite: &Skandite) -> bool {
    skandite.url.contains("tenor.com") || skandite.url.contains("giphy.com")
}

#[cfg(test)]
mod tests {
    use tokio_test::assert_ok;

    #[test]
    fn normalize_twitter_url() {
        let input = "https://twitter.com/fi_paris5/status/1470124228825526272?s=21";
        let output = assert_ok!(super::normalize_url(input));
        assert_eq!(
            output,
            "https://twitter.com/fi_paris5/status/1470124228825526272"
        );
    }
}
