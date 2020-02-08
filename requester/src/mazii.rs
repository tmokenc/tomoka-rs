use crate::Reqwest;
use crate::Result;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;

const API_END_POINT: &str = "http://mazii.net/api/search";

#[derive(Deserialize)]
pub struct MaziiKanjiSearch {
    pub status: u16,
    pub results: Vec<MaziiKanji>,
}

// Type Kanji
#[derive(Deserialize)]
pub struct MaziiKanji {
    pub kanji: char,
    pub mean: String,
    pub on: String,
    pub kun: Option<String>,
    pub detail: Option<String>,
    pub comp: Option<String>,
    pub level: Option<char>,
    pub stoke_count: Option<char>,
    pub example_on: Option<HashMap<String, Vec<Example>>>,
    pub example_kun: Option<HashMap<String, Vec<Example>>>,
}

#[derive(Deserialize)]
pub struct Example {
    #[serde(alias = "w")]
    pub word: String,
    #[serde(alias = "p")]
    pub phonetic: String,
    #[serde(alias = "m")]
    pub meaning: String,
}

impl MaziiKanji {
    pub fn normal_detail(&self) -> Option<String> {
        self.detail.as_ref().map(|e| {
            e.split("##")
                .map(|v| format!("- {}\n", v))
                .collect::<String>()
        })
    }

    pub fn normal_on(&self) -> String {
        self.on.split(' ').collect::<Vec<&str>>().join("、")
    }

    pub fn normal_kun(&self) -> Option<String> {
        self.kun
            .as_ref()
            .map(|e| e.split(' ').collect::<Vec<_>>().join("、"))
    }
}

#[async_trait]
pub trait MaziiRequester: Sync {
    async fn kanji<K: AsRef<str> + Send + 'async_trait>(&self, kanji: K)
        -> Result<Vec<MaziiKanji>>;
}

#[async_trait]
impl MaziiRequester for Reqwest {
    async fn kanji<K: AsRef<str> + Send + 'async_trait>(
        &self,
        kanji: K,
    ) -> Result<Vec<MaziiKanji>> {
        let kanji = kanji.as_ref();
        let body = json!({
            "dict": "javi",
            "type": "kanji",
            "query": kanji,
            "page": 1
        });

        let mut result = self
            .post(API_END_POINT)
            .json(&body)
            .send()
            .await?
            .json::<MaziiKanjiSearch>()
            .await?
            .results;

        result.sort_by_key(|a| kanji.chars().position(|x| x == a.kanji));

        Ok(result)
    }
}
