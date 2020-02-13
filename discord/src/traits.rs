#![allow(unstable_name_collisions)]

use crate::commands::prelude::now;
use crate::utils::{split_message, space_to_underscore};
use chrono::{TimeZone, Utc};
use magic::report_bytes;
use magic::traits::MagicIter as _;
use magic::traits::MagicStr as _;
use serenity::builder::CreateEmbed;
use std::fmt::Write as _;

/// This trait exist due to the number of rewriting thanks to my stupid code
pub trait ToEmbed {
    fn to_embed(&self, embed: &mut CreateEmbed);
}

impl ToEmbed for magic::sauce::SauceNao {
    fn to_embed(&self, embed: &mut CreateEmbed) {
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

    }
}

impl ToEmbed for requester::ehentai::Gmetadata {
    fn to_embed(&self, embed: &mut CreateEmbed) {
        let tags = self.parse_tags();
        let mut info = String::new();

        match (&self.title, &self.title_jpn) {
            (Some(ref title), None) | (None, Some(ref title)) => {
                embed.title(title);
            }
            
            (Some(ref title), Some(ref title_jpn)) => {
                embed.title(title);
                writeln!(&mut info, "**Title Jpn:** {}", title_jpn).unwrap();
            }
            
            _ => {}
        }
        
        fn write_info(mut info: &mut String, key: &str, data: Option<Vec<String>>) {
            if let Some(value) = data {
                write!(&mut info, "**{}:**", key).unwrap();
                for i in value {
                    write!(&mut info, "`{}` | ", i).unwrap();
                }
                info.truncate(info.len() - 3);
                info.push('\n');
            }
        };
        
        fn write_info_normal(mut info: &mut String, key: &str, data: Option<Vec<String>>) {
            if let Some(value) = data {
                write!(&mut info, "**{}:**", key).unwrap();
                for i in value {
                    info.push_str(&i);
                    info.push_str(" | ");
                }
                info.truncate(info.len() - 3);
                info.push('\n');
            }
        };

        write_info_normal(&mut info, "Language", tags.language);
        write_info(&mut info, "Parody", tags.parody);
        write_info(&mut info, "Characters", tags.characters);
        write_info(&mut info, "Artist", tags.artist);
        write_info_normal(&mut info, "Circle", tags.group);
        
        writeln!(&mut info, "**Gallery type**: {}", &self.category).unwrap();
        writeln!(&mut info, "**Total files**: {} ({})", &self.filecount, report_bytes(self.filesize)).unwrap();
        write!(&mut info, "**Rating**: {} / 5", &self.rating).unwrap();
        
        if !self.tags.is_empty() {
            info.push_str("\n\n***TAGs***");
        }

        embed.description(info);

        &[("Male", tags.male), ("Female", tags.female), ("Misc", tags.misc)]
            .iter()
            .filter_map(|(k, v)| v.as_ref().map(|x| (k, x)))
            .filter_map(|(k, v)| {
                v.iter()
                    .map(|v| (v, space_to_underscore(&v)))
                    .map(|(v, u)| format!("[{}](https://ehwiki.org/wiki/{})", v, u))
                    .join(" | ")
                    .to_option()
                    .map(|v| (k, v))
            })
            .for_each(|(k, v)| {
                if v.len() > 1024 {
                    let mut splited = split_message(v, 1024, "|").into_iter();
                    embed.field(k, splited.next().unwrap(), false);
                    for later in splited {
                        embed.field('\u{200B}', later, false);
                    }
                } else { 
                    embed.field(k, v, false);
                }
            });

        let time = self
            .posted
            .parse::<i64>()
            .map(|v| Utc.timestamp(v, 0))
            .unwrap_or_else(|_| Utc::now())
            .to_rfc3339();

        embed.timestamp(time);
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
        
        
        let url = format!("https://e-hentai.org/g/{}/{}", self.gid, self.token);

        embed.url(&url);
        embed.footer(|f| f.icon_url("https://cdn.discordapp.com/emojis/676135471566290985.png").text(url));
    }
}
