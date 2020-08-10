pub use tomo_serenity_ext::*;

use crate::types::Ref;
use crate::utils::now;
use chrono::{TimeZone, Utc};
use core::num::NonZeroUsize;
use magic::dark_magic::report_bytes;
use magic::traits::MagicIter as _;
use magic::traits::MagicStr as _;
use serenity::builder::CreateEmbed;
use std::fmt::{Display, Write};

impl Embedable for Ref<requester::saucenao::SauceNao> {
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

impl Embedable for Ref<requester::ehentai::Gmetadata> {
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

impl Embedable for Ref<requester::nhentai::NhentaiGallery> {
    fn append_to<'a>(&self, embed: &'a mut CreateEmbed) -> &'a mut CreateEmbed {
        let metadata = self.metadata();
        let mut description = format!(
            "**Category**: {}\n**Language**: {}\n**Total Pages**: {}\n",
            metadata.categories.join(", "),
            metadata.languages.join(", "),
            (&**self).total_pages(),
        );

        if !self.scanlator.is_empty() {
            let data = format!("**Scanlator**: {}\n", &self.scanlator);
            description.push_str(&data);
        }

        if let Some(parodies) = metadata.parodies {
            let data = format!("**Parody**: {}\n", parodies.join(", "));
            description.push_str(&data);
        }

        if let Some(characters) = metadata.characters {
            let data = format!("**Character**: {}\n", characters.join(", "));
            description.push_str(&data);
        }

        if let Some(groups) = metadata.groups {
            let data = format!("**Group**: {}\n", groups.join(", "));
            description.push_str(&data);
        }

        if let Some(artists) = metadata.artists {
            let data = format!("**Artist**: {}\n", artists.join(", "));
            description.push_str(&data);
        }

        let color = {
            let num_length = (self.id as f32 + 1.0).log10().ceil() as u64;
            self.media_id * num_length + self.id
        };

        embed.title(&self.title.pretty);
        embed.url(self.url());
        embed.thumbnail(self.thumbnail());
        embed.description(description);
        embed.color(color & 0xffffff);
        embed.timestamp(Utc.timestamp(self.upload_date as _, 0).to_rfc3339());

        if let Some(tags) = metadata.tags {
            embed.field("Tags", tags.join(", "), false);
        }

        embed
    }
}

impl Paginator for Ref<requester::nhentai::NhentaiGallery> {
    fn append_page(&mut self, page: NonZeroUsize, embed: &mut CreateEmbed) {
        let total_pages = (&**self).total_pages();
        let page = page.get();
        let color = {
            let num_length = (self.id as f32 + 1.0).log10().ceil() as u64;
            self.media_id * num_length + self.id
        };

        embed.title(&self.title.pretty);
        embed.url(self.url());
        embed.color(color);

        embed.footer(|f| {
            f.text(format!(
                "ID: {} | Page: {} / {}",
                self.id, page, total_pages
            ))
        });

        match self.page(page) {
            Some(p) => embed.image(p),
            None => embed.field(
                "Error",
                format!("Out of page, this gallery has only {} pages", total_pages),
                false,
            ),
        };
    }

    fn total_pages(&self) -> Option<usize> {
        Some((&**self).total_pages())
    }
}
