use crate::commands::prelude::now;
use magic::sauce::SauceNao;
use magic::traits::MagicIter as _;
use serenity::builder::CreateEmbed;

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

impl ToEmbed for SauceNao {
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
