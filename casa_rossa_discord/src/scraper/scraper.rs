use super::scraper_fetch;
use crate::config;

pub struct MenuScraper {
    pub base_url: String,
    pub client: reqwest::Client,
}

impl Default for MenuScraper {
    fn default() -> Self {
        Self {
            base_url: "http://www.casarossailfuturista.it".into(),
            client: reqwest::Client::new(),
        }
    }
}

#[rustfmt::skip]
impl MenuScraper {
    pub fn new(config: &config::Config) -> Self {
        Self { base_url: config.menu_base_url.clone(), client: reqwest::Client::new() }
    }
    pub async fn fetch(&self) -> crate::Result<Vec<super::ImageEntry>> {
        scraper_fetch::fetch(self).await
    }
}
