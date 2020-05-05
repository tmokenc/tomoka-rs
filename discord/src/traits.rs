#![allow(unstable_name_collisions)]

use crate::utils::now;
use chrono::{TimeZone, Utc};
use magic::report_bytes;
use magic::traits::MagicIter as _;
use magic::traits::MagicStr as _;
use serenity::builder::CreateEmbed;
use std::collections::HashMap;
use std::fmt::{Display, Write as _};

/// This trait exist due to the number of rewriting thanks to my stupid code
pub trait Embedable: Send {
    fn append_to<'a>(&self, embed: &'a mut CreateEmbed) -> &'a mut CreateEmbed;

    fn embed_data(&self) -> HashMap<&'static str, serde_json::Value> {
        let mut embed = CreateEmbed::default();
        self.append_to(&mut embed);
        embed.0
    }
}

// #[async_trait]
// pub trait Paginator {
//     fn len(&self) -> usize;
//     fn message_data<'a>(
//         &mut self, 
//         msg: &mut CreateMessage<'a>
//     ) -> &mut CreateMessage<'a>;
//     
//     async fn update(&mut self) -> Result<()> {
//         Ok(())
//     }
// }

impl Embedable for requester::saucenao::SauceNao {
    fn append_to<'a>(&self, embed: &'a mut CreateEmbed) -> &'a mut CreateEmbed {
        let mut info = String::new();

        match self.characters.len() {
            0 => {}
            1 => {
                let content = self.characters.iter().next().unwrap();
                writeln!(&mut info, "**Character**: {}", content).unwrap();
            }
            _ => {
                let content = self.characters.iter().join("\n");
                embed.field("Characters", content, false);
            }
        }

        match self.parody.len() {
            0 => {}
            1 => {
                let content = self.parody.iter().next().unwrap();
                writeln!(&mut info, "**Parody**: {}", content).unwrap();
            }
            _ => {
                let content = self.parody.iter().join("\n");
                embed.field("Parody", content, false);
            }
        }

        if let Some(creator) = &self.creator {
            writeln!(&mut info, "**Creator**: {}", creator).unwrap();
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

                writeln!(&mut info, "**Author**: {}", content).unwrap();
            }
            _ => {
                let content = self
                    .author
                    .iter()
                    .map(|(k, v)| format!("[{} ({})]({})", k, v.name, v.url))
                    .join("\n");

                embed.field("Author", content, false);
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

                writeln!(&mut info, "**Source**: {}", content).unwrap();
            }
            _ => {
                let content = self
                    .sources
                    .iter()
                    .map(|(k, v)| format!("[{}]({})", k, v))
                    .join("\n");

                embed.field("Sources", content, false);
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

                writeln!(&mut info, "**Altenative link**: {}", content).unwrap();
            }
            _ => {
                let content = self
                    .altenative_links
                    .iter()
                    .map(|(k, v)| format!("[{}]({})", k, v))
                    .join("\n");

                embed.field("Altenative links", content, false);
            }
        }

        if let Some(n) = &self.note {
            writeln!(&mut info, "**Note**: {}", n).unwrap();
        }
        if let Some(title) = &self.title {
            embed.title(title);
        }

        embed
            .description(info)
            .url(self.url())
            .thumbnail(self.img_url())
            .timestamp(now())
            .footer(|f| f.text("Powered by SauceNao"))
    }
}

impl Embedable for requester::ehentai::Gmetadata {
    fn append_to<'a>(&self, embed: &'a mut CreateEmbed) -> &'a mut CreateEmbed {
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

        fn write_info<D: Display>(mut info: &mut String, key: &str, data: Option<Vec<D>>) {
            if let Some(value) = data {
                write!(&mut info, "**{}:** ", key).unwrap();
                for i in value {
                    write!(&mut info, "`{}` | ", i).unwrap();
                }
                info.truncate(info.len() - 3);
                info.push('\n');
            }
        };

        fn write_info_normal<D: Display>(mut info: &mut String, key: &str, data: Option<Vec<D>>) {
            if let Some(value) = data {
                write!(&mut info, "**{}:** ", key).unwrap();
                for i in value {
                    write!(&mut info, "{} | ", i).unwrap();
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
        writeln!(
            &mut info,
            "**Total files**: {} ({})",
            &self.filecount,
            report_bytes(self.filesize)
        )
        .unwrap();
        write!(&mut info, "**Rating**: {} / 5", &self.rating).unwrap();

        if self.expunged {
            info.push_str("\n>>>>> ***EXPUNGED*** <<<<<");
        }

        if !self.tags.is_empty() {
            info.push_str("\n\n***TAGs***");
        }

        embed.description(info);

        [
            ("Male", tags.male),
            ("Female", tags.female),
            ("Misc", tags.misc),
        ]
        .iter()
        .filter_map(|(k, v)| v.as_ref().map(|v| (k, v)))
        .map(|(key, v)| {
            let mut content = String::with_capacity(45 * v.len());

            for tag in v {
                write!(&mut content, "[{}]({}) | ", tag, tag.wiki_url()).unwrap();
            }

            content.truncate(content.len() - 3);
            (key, content)
        })
        .for_each(|(k, v)| {
            let mut splited = v.split_at_limit(1024, "|");

            if let Some(data) = splited.next() {
                embed.field(k, data, false);

                for later in splited {
                    embed.field('\u{200B}', later, false);
                }
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

        embed.color(self.category.color());

        let url = self.url();

        embed.url(&url);
        embed.footer(|f| {
            f.icon_url("https://cdn.discordapp.com/emojis/676135471566290985.png")
                .text(url)
        });

        embed
    }
}
