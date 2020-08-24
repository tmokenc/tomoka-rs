use crate::commands::prelude::*;
use crate::traits::ChannelExt;
use pokemon_core::{Flavor, Nature, Stat};
use std::fmt::Write;
use std::str::FromStr;

#[derive(Default, Debug)]
pub struct Filter {
    natures: Vec<Nature>,
    data: Vec<FilterData>,
}

#[derive(Debug)]
pub enum FilterData {
    Increase(Stat),
    Decrease(Stat),
    Favorite(Flavor),
    Disliked(Flavor),
}

impl Filter {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.natures.is_empty() && self.data.is_empty()
    }
    
    pub fn can_pass(&self, nature: Nature) -> bool {
        (self.natures.is_empty() && self.data.is_empty())
        || self.natures.iter().any(|&v| v == nature)
        || !self.data.is_empty() && self.data.iter().all(|v| match v {
            FilterData::Increase(x) => nature.increase() == *x,
            FilterData::Decrease(x) => nature.decrease() == *x,
            FilterData::Favorite(x) => nature.favorite() == *x,
            FilterData::Disliked(x) => nature.disliked() == *x,
        })
    }
}

impl From<&str> for Filter {
    fn from(s: &str) -> Self {
        let mut filter = Self::default();
        let args = s.split_whitespace();

        for arg in args {
            if let Ok(nature) = Nature::from_str(&arg) {
                if !filter.natures.contains(&nature) {
                    filter.natures.push(nature);
                }

                continue;
            }

            let mut arg = arg;
            let plus = if arg.starts_with('-') {
                arg = &arg[1..];
                false
            } else {
                true
            };

            if arg.starts_with('+') {
                arg = &arg[1..];
            }

            if let Ok(stat) = Stat::from_str(&arg) {
                let has = filter.data.iter().any(|v| match v {
                    FilterData::Increase(x) | FilterData::Decrease(x) => stat == *x,
                    _ => false,
                });

                if !has {
                    let data = if plus {
                        FilterData::Increase(stat)
                    } else {
                        FilterData::Decrease(stat)
                    };

                    filter.data.push(data);
                }

                continue;
            }

            if let Ok(flavor) = Flavor::from_str(&arg) {
                let has = filter.data.iter().any(|v| match v {
                    FilterData::Favorite(x) | FilterData::Disliked(x) => flavor == *x,
                    _ => false,
                });

                if !has {
                    let data = if plus {
                        FilterData::Favorite(flavor)
                    } else {
                        FilterData::Disliked(flavor)
                    };

                    filter.data.push(data);
                }

                continue;
            }
        }

        filter
    }
}

#[command]
/// Get a pokemon nature information or get all of them
async fn nature(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let args = args.rest();
    let filter = Filter::from(args);
    let mut data = String::new();
    
    Nature::iter()
        .filter(|&v| filter.can_pass(v))
        .for_each(|v| write_nature(&mut data, v));

    if data.is_empty() {
        data = format!("Cannot find any nature with `{}`", args);
    }

    msg.channel_id.send_embed(ctx).with_description(data).await?;

    Ok(())
}

#[inline]
pub(super) fn write_nature(f: &mut String, nature: Nature) {
    writeln!(
        f,
        "**{}**> 🔼 {} | 🔻 {} | 👍 {} | 👎 {}",
        nature,
        nature.increase(),
        nature.decrease(),
        nature.favorite(),
        nature.disliked(),
    )
    .unwrap();
}
