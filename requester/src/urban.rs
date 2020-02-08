use crate::Reqwest;
use crate::Result;
use serde::Deserialize;
use std::fmt::Display;

type UrbanResult = Result<Vec<UrbanDictionary>>;

#[derive(Deserialize)]
struct UrbanApi {
    list: Vec<UrbanDictionary>,
}

#[derive(Deserialize)]
pub struct UrbanDictionary {
    pub word: String,
    pub definition: String,
    pub permalink: String,
    pub author: String,
    pub example: String,
    pub thumbs_up: u32,
    pub thumbs_down: u32,
    pub written_on: String,
}

#[async_trait]
pub trait UrbanRequester {
    async fn search_word(&self, word: impl Display + Send + 'async_trait) -> UrbanResult;
    async fn get_random(&self) -> UrbanResult;
}

#[async_trait]
impl UrbanRequester for Reqwest {
    async fn get_random(&self) -> UrbanResult {
        let url = "http://api.urbandictionary.com/v0/random";
        let res: UrbanApi = self.get(url).send().await?.json().await?;

        Ok(res.list)
    }

    async fn search_word(&self, word: impl Display + Send + 'async_trait) -> UrbanResult {
        let url = format!("http://api.urbandictionary.com/v0/define?term={}", word);
        let res: UrbanApi = self.get(&url).send().await?.json().await?;

        Ok(res.list)
    }
}
