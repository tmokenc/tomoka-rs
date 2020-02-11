#![allow(unstable_name_collisions)]

use crate::commands::prelude::now;
use magic::traits::MagicIter as _;
use magic::traits::MagicStr as _;
use magic::traits::MagicBool as _;
use std::fmt::Write as _;
use magic::report_bytes;
use serenity::builder::CreateEmbed;
use chrono::{Utc, TimeZone};
use crate::utils::space_to_underscore;

/// This trait exist due to the number of rewriting thanks to my stupid code
pub trait ToEmbed {
    fn to_embed<'a>(&self, embed: &'a mut CreateEmbed) -> &'a mut CreateEmbed;

    /// This function do nothing
    /// There is nowhere use this, so it can be deleted safely
    /// Just for my own *bad* habit
    fn into_embed(&self) -> CreateEmbed {
        let mut embed = CreateEmbed::default();
        self.to_embed(&mut embed);
        embed
    }
}

impl ToEmbed for magic::sauce::SauceNao {
    fn to_embed<'a>(&self, embed: &'a mut CreateEmbed) -> &'a mut CreateEmbed {
        let mut description = Vec::new();
        let mut fields = Vec::new();

        if let Some(creator) = &self.creator {
            description.push(format!("**Creator**: {}", creator));
        }

        match self.characters.len() {
            0 => {}
            1 => {
                let content = self.characters.iter().next().unwrap();
                description.insert(0, format!("**Character**: {}", content));
            }
            _ => {
                let content = self.characters.iter().join("\n");
                fields.push(("Characters", content, false));
            }
        }

        match self.parody.len() {
            0 => {}
            1 => {
                let content = self.parody.iter().next().unwrap();
                description.insert(0, format!("**Parody**: {}", content));
            }
            _ => {
                let content = self.parody.iter().join("\n");
                fields.push(("Parody", content, false));
            }
        }

        match self.author.len() {
            0 => {}
            1 => {
                let content = self
                    .author
                    .iter()
                    .next()
                    .map(|(k, v)| format!("[{} ({})]({})", k, v.name, v.url))
                    .unwrap();

                description.push(format!("**Author**: {}", content));
            }
            _ => {
                let content = self
                    .author
                    .iter()
                    .map(|(k, v)| format!("[{} ({})]({})", k, v.name, v.url))
                    .join("\n");

                fields.push(("Author", content, false));
            }
        }

        match self.sources.len() {
            0 => {}
            1 => {
                let content = self
                    .sources
                    .iter()
                    .next()
                    .map(|(k, v)| format!("[{}]({})", k, v))
                    .unwrap();

                description.push(format!("**Source**: {}", content));
            }
            _ => {
                let content = self
                    .sources
                    .iter()
                    .map(|(k, v)| format!("[{}]({})", k, v))
                    .join("\n");

                fields.push(("Sources", content, false));
            }
        }

        match self.altenative_links.len() {
            0 => {}
            1 => {
                let content = self
                    .altenative_links
                    .iter()
                    .next()
                    .map(|(k, v)| format!("[{}]({})", k, v))
                    .unwrap();

                description.push(format!("**Altenative link**: {}", content));
            }
            _ => {
                let content = self
                    .altenative_links
                    .iter()
                    .map(|(k, v)| format!("[{}]({})", k, v))
                    .join("\n");

                fields.push(("Altenative links", content, false));
            }
        }

        if let Some(n) = &self.note {
            description.push(format!("**Note**: {}", n));
        }
        if let Some(title) = &self.title {
            embed.title(title);
        }

        let description = description.join("\n");

        embed
            .description(description)
            .url(self.url())
            .fields(fields)
            .thumbnail(self.img_url())
            .timestamp(now())
            .footer(|f| f.text("Powered by SauceNao"));

        embed
    }
}

impl ToEmbed for requester::ehentai::Gmetadata {
    fn to_embed<'a>(&self, embed: &'a mut CreateEmbed) -> &'a mut CreateEmbed {
        let mut artist = Vec::new();
        let mut character = Vec::new();
        let mut female = Vec::new();
        let mut group = Vec::new();
        let mut language = Vec::new();
        let mut male = Vec::new();
        let mut parody = Vec::new();
        let mut misc = Vec::new();
        
        for tag in self.tags.iter() {
            if tag.contains(":") {
                let mut iter = tag.split(":");
                let namespace = iter.next().unwrap();
                let value = iter.next().unwrap();
                
                match namespace.as_ref() {
                    "artist" => artist.push(value.to_owned()),
                    "character" => character.push(value.to_owned()),
                    "group" => group.push(value.to_owned()),
                    "language" => language.push(value.to_owned()),
                    "male" => male.push(value.to_owned()),
                    "female" => female.push(value.to_owned()),
                    "parody" => parody.push(value.to_owned()),
                    _ => ()
                }
            } else {
                misc.push(tag.to_owned());
            }
        }
        
        let tags = &[
            ("Male", male),
            ("Female", female),
            ("Misc", misc),
        ];
        
        let mut title = String::new();
        
        if !self.title.is_empty() {
            write!(&mut title, "**Title**: {}", &self.title).unwrap();
        }
        
        if !self.title_jpn.is_empty() {
            if !title.is_empty() {
                title.push('\n');
            }
            write!(&mut title, "**Title Jpn**: {}", &self.title_jpn).unwrap();
        }
         
        let language = (!language.is_empty())
            .then(|| format!("\n**Language**: {}", language.join(" ")));
            
        let parody = parody
            .into_iter()
            .map(|v| format!("`{}`", v))
            .join(" ")
            .to_option()
            .map(|v| format!("\n**Parody**: {}", v));
            
        let characters = character
            .into_iter()
            .map(|v| format!("`{}`", v))
            .join(" ")
            .to_option()
            .map(|v| format!("\n**Characters**: {}", v));
            
        let circle = (!group.is_empty())
            .then(|| format!("\n**Circle**: {}", group.join(", ")));
            
        tags.into_iter()
            .filter_map(|(k, v)| {
                v.into_iter()
                    .map(|v| (v, space_to_underscore(&v)))
                    .map(|(v, u)| format!("[{}](https://ehwiki.org/wiki/{})", v, u))
                    .join(" ")
                    .to_option()
                    .map(|v| (k, v))
            })
            .for_each(|(k, v)| {
                embed.field(k, v, false);    
            });
            
        let description = format!(
            "{title} {language} {parody} {characters}
            **Artist**: {artist} {circle}
            **Category**: {category}
            **Total files**: {count} ({size})
            **Rating 👍**: {rating} / 5
            
            ***TAGS***",
            
            title = title,
            language = language.unwrap_or_default(),
            parody = parody.unwrap_or_default(),
            characters = characters.unwrap_or_default(),
            artist = artist.into_iter().map(|v| format!("`{}`", v)).join(" "),
            circle = circle.unwrap_or_default(),
            category = &self.category,
            count = &self.filecount,
            size = report_bytes(self.filesize),
            rating = &self.rating,
        );
           
        embed.description(description);
        
        let time = self
            .posted
            .parse::<i64>()
            .unwrap_or(Utc::now().timestamp_millis());
            
        embed.timestamp(Utc.timestamp_millis(time).to_rfc3339());
        // embed.author(|a| a.name(&self.uploader));
        embed.thumbnail(&self.thumb);
        
        embed.color(match self.category.as_str() {
            "Doujinshi" => 0xf66258,
            "Manga" => 0xf5a718,
            "Artist CG" => 0xd4d503,
            "Game CG" => 0x09b60e,
            "Western" => 0x2cdb2b,
            "Image Set" => 0x4f5ce7,
            "Non-H" => 0x0cbacf,
            "Cosplay" => 0x902ede,
            "Asian Porn" => 0xf188ef,
            _ => 0x8a8a8a,
        });
        
        embed.footer(|f| f.text(format!("https://e-hentai.org/g/{}/{}", self.gid, self.token)));
        
        embed
    }
}