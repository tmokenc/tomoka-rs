use crate::Reqwest;
use crate::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;

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


#[derive(Default, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Gmetadata {
    pub gid: u64,
    pub token: String,
    pub archiver_key: String,
    pub title: Option<String>,
    pub title_jpn: Option<String>,
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

impl Gmetadata {
    pub fn is_sfw(&self) -> bool {
        self.category.as_str() == "Non-H"
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Gtoken {
    pub gid: u64,
    pub token: String,
}

#[async_trait]
pub trait EhentaiApi {
    async fn gmetadata<I>(&self, g: I) -> Result<Vec<Gmetadata>>
    where 
        I: IntoIterator<Item=(u32, String)> + Send + 'async_trait;
    
    // async fn gtoken<T>(&self, id: u64, token: T, page: u16) -> Result<Vec<Gtoken>>
    // where
    //     T: AsRef<str> + Send + 'async_trait;
}

#[async_trait]
impl EhentaiApi for Reqwest {
    async fn gmetadata<I>(&self, g: I) -> Result<Vec<Gmetadata>>
    where 
        I: IntoIterator<Item = (u32, String)> + Send + 'async_trait
    {
        let galleries = g.into_iter().collect::<Vec<_>>();
        let body = json!({
            "method": "gdata",
            "gidlist": galleries,
            "namespace": 1
        });

        let data: GmetadataRoot = self.post(API_ENDPOINT).json(&body).send().await?.json().await?;
        Ok(data.gmetadata)
    }
    
    // async fn gtoken<T, P>(&self, id: u64, token: T, page: u16) -> Result<Vec<Gtoken>>
    // where
    //     T: AsRef<str> + Send + 'async_trait,
    //     P: IntoIterator<Item=u16> + Send + 'async_trait,
    // {
    //     let pages = page.into_iter().map(|v| (id, token,as_ref(), v)).collect::<Vec<_>>();
    //     let body = json!({
    //         "method": "gtoken",
    //         "pagelist": pages
    //     });

    //     let data: GtokenRoot = self.get(API_ENDPOINT).body(&body).send().await?.json().await?;
    //     Ok(data.token_list)
    // }
}
