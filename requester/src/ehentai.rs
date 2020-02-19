use crate::Reqwest;
use crate::Result;
use magic::traits::MagicOption;
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
    pub title: String,
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

type Tag = Option<Vec<String>>;

#[derive(Default)]
pub struct Tags {
    pub artist: Tag,
    pub characters: Tag,
    pub group: Tag,
    pub parody: Tag,
    pub language: Tag,
    pub male: Tag,
    pub female: Tag,
    pub misc: Tag,
    pub reclass: Option<String>,
}

impl Gmetadata {
    pub fn is_sfw(&self) -> bool {
        self.category.as_str() == "Non-H"
    }

    pub fn parse_tags(&self) -> Tags {
        let mut tags = Tags::default();

        for tag in self.tags.iter() {
            if tag.contains(':') {
                let mut iter = tag.split(':');
                let namespace = iter.next().unwrap();
                let value = iter.next().unwrap().to_owned();

                match namespace {
                    "artist" => tags.artist.extend_inner(value),
                    "character" => tags.characters.extend_inner(value),
                    "group" => tags.group.extend_inner(value),
                    "language" => tags.language.extend_inner(value),
                    "male" => tags.male.extend_inner(value),
                    "female" => tags.female.extend_inner(value),
                    "parody" => tags.parody.extend_inner(value),
                    "reclass" => {
                        if tags.reclass.is_none() {
                            tags.reclass = Some(value)
                        }
                    }
                    _ => (),
                }
            } else {
                tags.misc.extend_inner(tag.to_owned());
            }
        }

        tags
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
        I: IntoIterator<Item = (u32, String)> + Send + 'async_trait;

    // async fn gtoken<T>(&self, id: u64, token: T, page: u16) -> Result<Vec<Gtoken>>
    // where
    //     T: AsRef<str> + Send + 'async_trait;
}

#[async_trait]
impl EhentaiApi for Reqwest {
    async fn gmetadata<I>(&self, g: I) -> Result<Vec<Gmetadata>>
    where
        I: IntoIterator<Item = (u32, String)> + Send + 'async_trait,
    {
        let galleries = g.into_iter().collect::<Vec<_>>();
        let body = json!({
            "method": "gdata",
            "gidlist": galleries,
            "namespace": 1
        });

        let data: GmetadataRoot = self
            .post(API_ENDPOINT)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;
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
