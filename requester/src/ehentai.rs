use crate::Reqwest;
use serde::{Deserialize, Serialize};

const API_ENDPOINT: &str = "https://api.e-hentai.org/api.php";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GmetadataRoot {
    pub gmetadata: Vec<Gmetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GtokenRoot {
    pub token_list: Vec<Gtoken>,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Gmetadata {
    pub gid: u64,
    pub token: String,
    #[serde(rename = "archiver_key")]
    pub archiver_key: String,
    pub title: String,
    #[serde(rename = "title_jpn")]
    pub title_jpn: String,
    pub category: String,
    pub thumb: String,
    pub uploader: String,
    pub posted: String,
    pub filecount: String,
    pub filesize: u64,
    pub expunged: bool,
    pub rating: String,
    pub torrentcount: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Gtoken {
    pub gid: u64,
    pub token: String,
}



#[async_trait]
pub trait EhentaiApi {
    async fn gmetadata<T>(&self, id: u64, token: T) -> Result<Gmetadata>
    where T: AsRef<str> + Send + 'async_trait;
    
    async fn gtoken<T>(&self, id: u64, token: T, page: u16) -> Result<Gtoken>
    where
        T: AsRef<str> + Send + 'async_trait;
}

#[async_trait]
impl EhentaiApi for Reqwest {
    async fn gmetadata<T>(&self, id: u64, token: T) -> Result<Gmetadata>
    where T: AsRef<str> + Send + 'async_trait
    {
        let body = json!({
            "method": "gdata",
            "gidlist": [id, token.as_ref()],
            "namespace": 1
        });

        let data: GmetadataRoot = self.get(API_ENDPOINT).body(&body).send().await?.json().await?;
        Ok(data.gmetadatal)
    }
    
    async fn gtoken<T, P>(&self, id: u64, token: T, page: u16) -> Result<Gtoken>
    where
        T: AsRef<str> + Send + 'async_trait,
        P: IntoIterator<Item=u16> + Send + 'async_trait,
    {
        let pages = page.into_iter().map(|v| (id, token,as_ref(), v)).collect::<Vec<_>>();
        let body = json!({
            "method": "gtoken",
            "pagelist": pages
        });

        let data: GtokenRoot = self.get(API_ENDPOINT).body(&body).send().await?.json().await?;
        Ok(data.token_list)
    }
};
