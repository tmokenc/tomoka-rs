use scraper::{ElementRef, Html, Selector};
use std::collections::{HashMap, HashSet};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

type Title = Option<String>;
type Parody = HashSet<String>;
type Creator = Option<String>;
type Sources = HashMap<String, String>;
type Authors = HashMap<String, Author>;
type Characters = HashSet<String>;
type AltenativeLinks = HashMap<String, String>;
type Note = Option<String>;

type MainData = (Sources, Authors, Characters, Parody, Note);

#[inline]
pub fn get_sauce(img_url: impl AsRef<str>, similarity: impl Into<Option<f32>>) -> Result<SauceNao> {
    SauceNao::get(img_url, similarity)
}

#[derive(Debug)]
pub struct SauceNao {
    pub title: Title,
    pub parody: Parody,
    pub characters: Characters,
    pub creator: Creator,
    pub author: Authors,
    pub sources: Sources,
    pub altenative_links: AltenativeLinks,
    pub note: Note,
    pub url: String,
    pub img_url: String,
}

#[derive(Debug)]
pub struct Author {
    pub name: String,
    pub url: String,
}

impl Author {
    fn new(name: impl ToString, url: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
        }
    }
}

impl SauceNao {
    pub fn get(img_url: impl AsRef<str>, similarity: impl Into<Option<f32>>) -> Result<Self> {
        let url = format!(
            "http://saucenao.com/search.php?db=999&url={}",
            img_url.as_ref()
        );
        let req = {
            let mut rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async { reqwest::get(&url).await?.text().await })?
        };

        let html = req.split("Low similarity results").next().unwrap();

        let html = Html::parse_fragment(html);

        let result_selector = Selector::parse(".result").unwrap();
        let title_selector = Selector::parse(".resulttitle").unwrap();
        let selector = Selector::parse(".resultcontentcolumn").unwrap();
        let alt_links_selector = Selector::parse(".resultmiscinfo > a").unwrap();

        let similarity = similarity.into();
        let data = html
            .select(&result_selector)
            .filter(move |v| filter_low_similarity(v, similarity))
            .collect::<Vec<_>>();

        let (title, creator) = data
            .iter()
            .flat_map(|v| v.select(&title_selector))
            .fold(Default::default(), get_title_creator);

        let (sources, author, characters, parody, note) = data
            .iter()
            .flat_map(|v| v.select(&selector))
            .fold(Default::default(), get_main_data);

        let altenative_links = data
            .iter()
            .flat_map(|v| v.select(&alt_links_selector))
            .filter_map(|v| v.value().attr("href"))
            .fold(Default::default(), get_altenative_links);

        Ok(Self {
            title,
            parody,
            characters,
            creator,
            author,
            sources,
            altenative_links,
            note,
            url,
            img_url: img_url.as_ref().to_owned(),
        })
    }

    pub fn not_found(&self) -> bool {
        self.title.is_none()
            && self.creator.is_none()
            && self.parody.is_empty()
            && self.altenative_links.is_empty()
            && self.characters.is_empty()
            && self.sources.is_empty()
    }

    pub fn found(&self) -> bool {
        !self.not_found()
    }

    pub fn url(&self) -> String {
        self.url.to_owned()
    }

    pub fn img_url(&self) -> String {
        self.img_url.to_owned()
    }
}

fn filter_low_similarity<'a>(element: &ElementRef<'a>, s: impl Into<Option<f32>>) -> bool {
    let selector = Selector::parse(".resultsimilarityinfo").unwrap();
    let similarity = s.into().unwrap_or(82.0);

    if similarity < 50.0 {
        return true;
    }

    element
        .select(&selector)
        .next()
        .and_then(|v| {
            let mut s = v.text().collect::<String>();
            s.truncate(s.len() - 1);
            s.parse::<f32>().ok()
        })
        .filter(|&v| v > similarity)
        .is_some()
}

fn get_title_creator(
    (mut title, mut creator): (Title, Creator),
    data: ElementRef<'_>,
) -> (Title, Creator) {
    let data = data.text().collect::<Vec<_>>();

    match (data.len(), &title, &creator) {
        (1, None, _) => title = data.get(0).map(|&v| v.to_string()),
        (2, _, None) => creator = data.get(1).map(|&v| v.to_string()),
        _ => (),
    };

    (title, creator)
}

fn get_main_data(
    (mut sources, mut author, mut characters, mut parody, mut note): MainData,
    element: ElementRef<'_>,
) -> MainData {
    let a = Selector::parse("a").unwrap();

    let mut watching_parody = false;
    let mut watching_character = false;

    for data in element.inner_html().split("<br>") {
        if data.is_empty() {
            continue;
        }

        if data.starts_with("<small>") && note.is_none() {
            let element = Html::parse_fragment(&data);
            note = Some(element.root_element().text().collect());
            continue;
        }

        if !data.starts_with("<strong>") {
            if watching_character {
                characters.insert(data.to_string());
            } else if watching_parody {
                parody.insert(data.to_string());
            }

            continue;
        }

        if data.starts_with("<strong>Characters") {
            watching_character = true;
            watching_parody = false;
        }

        if data.starts_with("<strong>Material:") {
            if let Some(p) = data.split("</strong>").nth(1) {
                parody.insert(p.to_owned());
            }

            watching_parody = true;
            watching_character = false;
            continue;
        }

        if data.starts_with("<strong>Member:") {
            let fragment = Html::parse_fragment(&data);

            let element = match fragment.select(&a).next() {
                Some(l) => l,
                None => continue,
            };

            if let Some(link) = element.value().attr("href") {
                let key: &str = match link {
                    n if n.contains("nico") => "Niconico Seiga",
                    n if n.contains("pixiv") => "Pixiv",
                    _n => {
                        // dbg!(n);
                        continue;
                    }
                };

                let name = element.text().collect::<String>();
                let aut = Author::new(name, link);
                author.insert(key.to_owned(), aut);
            }

            continue;
        }

        if data.starts_with("<strong>Source:") {
            let fragment = Html::parse_fragment(&data);

            for source in fragment.select(&a) {
                if let Some(value) = source.value().attr("href") {
                    let key = source.text().collect();
                    sources.insert(key, value.to_owned());
                }
            }

            continue;
        }

        if data.starts_with("<strong>Seiga ID:") {
            let fragment = Html::parse_fragment(&data);

            if let Some(e) = fragment.select(&a).next() {
                let id = e.text().collect::<String>();
                let key = format!("Seiga #{}", &id);
                let value = format!("https://seiga.nicovideo.jp/seiga/im{}", id);
                sources.insert(key, value);
            }

            continue;
        }

        // if data.starts_with("<strong>Pixiv ID:") {
        //     let fragment = Html::parse_fragment(&data);

        //     if let Some(e) = fragment.select(&a).next() {
        //         let id = e.text().collect::<String>();
        //         let key = format!("Pixiv #{}", &id);

        //         if !sources.contains_key(&key) {
        //             let link = format!("https://www.pixiv.net/member_illust.php?mode=medium&illust_id={}", id);
        //             sources.insert(key, link);
        //         }
        //     }

        //     continue
        // }

        // dbg!(data);
        // dbg!();
    }

    (sources, author, characters, parody, note)
}

fn get_altenative_links(mut links: AltenativeLinks, data: &str) -> AltenativeLinks {
    let key = match &data {
        n if n.contains("sankakucomplex") => "SankakuChan",
        n if n.contains("gelbooru") => "Geibooru",
        n if n.contains("danbooru") => "Danbooru",
        n if n.contains("yande.re") => "Yandere",
        n if n.contains("mangaupdates") => "MangaUpdates",
        n => n.split('/').nth(2).unwrap(),
    };

    links.insert(key.to_owned(), data.to_owned());
    links
}
