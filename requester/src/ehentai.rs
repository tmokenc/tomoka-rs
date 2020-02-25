use crate::Reqwest;
use crate::Result;
use magic::traits::MagicOption;
use escaper::decode_html;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;

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
    pub category: Category,
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

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum Category {
    Doujinshi,
    Manga,
    #[serde(rename = "Artist CG")]
    ArtistCG,
    #[serde(rename = "Game CG")]
    GameCG,
    Western,
    #[serde(rename = "Image Set")]
    ImageSet,
    #[serde(rename = "Non-H")]
    NonH,
    Cosplay,
    AsianPorn,
}

impl Default for Category {
    fn default() -> Self {
        Self::Doujinshi
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Category::*;
        
        match self {
            Doujinshi => write!(f, "Doujinshi"),
            Manga => write!(f, "Manga"),
            ArtistCG => write!(f, "Artist CG"),
            GameCG => write!(f, "Game CG"),
            Western => write!(f, "Western"),
            ImageSet => write!(f, "Image Set"),
            NonH => write!(f, "Non-H"),
            Cosplay => write!(f, "Cosplay"),
            AsianPorn => write!(f, "Asian Porn"),
        }
    }
}

impl Category {
    /// Based on the color of the tag on e-h
    pub fn color(&self) -> u32 {
        use Category::*;
        
        match self {
            Doujinshi => 0xf66258,
            Manga => 0xf5a718,
            ArtistCG => 0xd4d503,
            GameCG => 0x09b60e,
            Western => 0x2cdb2b,
            ImageSet => 0x4f5ce7,
            NonH => 0x0cbacf,
            Cosplay => 0x902ede,
            AsianPorn => 0xf188ef,
        }
    }
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
        self.category == Category::NonH 
    }
    
    pub fn url(&self) -> String {
        format!("https://e-hentai.org/g/{}/{}", self.gid, self. token)
    }
    
    pub fn x_url(&self) -> String {
        format!("https://exhentai.org/g/{}/{}", self.gid, self. token)
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

        let mut data: Vec<Gmetadata> = self
            .post(API_ENDPOINT)
            .json(&body)
            .send()
            .await?
            .json::<GmetadataRoot>()
            .await?
            .gmetadata;
        
        for mut d in data.iter_mut() {
            d.title = d.title
                .as_ref()
                .filter(|v| !v.is_empty())
                .and_then(|v| decode_html(v).ok());
                
            d.title_jpn = d.title_jpn
                .as_ref()
                .filter(|v| !v.is_empty())
                .and_then(|v| decode_html(v).ok());
        }
        
        Ok(data)
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
