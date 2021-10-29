use anyhow::Error;
use serde::Deserialize;
use url::Url;

pub enum SearchMode {
    Web,
    Image,
}

#[derive(Deserialize)]
struct GoogleSearch {
    items: Option<Vec<GoogleSearchItem>>,
}

#[derive(Deserialize)]
pub struct GoogleSearchItem {
    pub title: String,
    pub link: String,
    pub snippet: String,
}

pub struct GoogleSearcher {
    pub google_key: String,
    pub google_cse_id: String,
}
impl GoogleSearcher {
    pub fn search(
        &self,
        terms: String,
        mode: SearchMode,
    ) -> Result<Option<GoogleSearchItem>, Error> {
        let mut url = Url::parse("https://www.googleapis.com/customsearch/v1")?;
        url.query_pairs_mut()
            .append_pair("key", &self.google_key)
            .append_pair("cx", &self.google_cse_id)
            .append_pair("q", &terms);
        if let SearchMode::Image = mode {
            url.query_pairs_mut().append_pair("searchType", "image");
        }

        let item = ureq::get(url.as_str())
            .call()?
            .into_json::<GoogleSearch>()?
            .items
            .unwrap_or(vec![])
            .into_iter()
            .nth(0);
        Ok(item)
    }
}
