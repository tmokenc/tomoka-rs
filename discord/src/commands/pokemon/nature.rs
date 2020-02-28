use crate::commands::prelude::*;
use pokemon_core::{Flavor, Nature, Stat};
use std::fmt::Write;
use std::str::FromStr;

#[derive(Default)]
pub struct Filter {
    natures: Vec<Nature>,
    data: Vec<FilterData>,
}

pub enum FilterData {
    Increase(Stat),
    Decrease(Stat),
    Favorite(Flavor),
    Disliked(Flavor),
}

#[command]
/// Get a pokemon nature information or get all of them
fn nature(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let args = args.rest().split_whitespace();
    let mut filter = Filter::default();

    for arg in args {
        if let Ok(nature) = Nature::from_str(&arg) {
            if !filter.natures.contains(&nature) {
                filter.natures.push(nature);
            }

            continue;
        }

        let mut arg = arg;
        let mut plus = true;

        if arg.starts_with('-') {
            arg = &arg[1..];
            plus = false;
        }

        if arg.starts_with('+') {
            arg = &arg[1..];
        }

        if let Ok(stat) = Stat::from_str(&arg) {
            let has = filter.data.iter().any(|v| match v {
                FilterData::Increase(x) | FilterData::Decrease(x) => stat == *x,
                _ => false,
            });

            if !has {
                let data = match plus {
                    true => FilterData::Increase(stat),
                    false => FilterData::Decrease(stat),
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

    let mut data = String::new();

    for nature in Nature::iter() {
        if is_in_filter(nature, &filter) {
            write_nature(&mut data, nature);
        }
    }

    msg.channel_id.say(&ctx, data)?;

    Ok(())
}

fn is_in_filter(nature: Nature, filter: &Filter) -> bool {
    (filter.natures.is_empty() && filter.data.is_empty())
        || filter.natures.iter().any(|&v| v == nature)
        || filter.data.iter().all(|v| match v {
            FilterData::Increase(x) => nature.increase() == *x,
            FilterData::Decrease(x) => nature.decrease() == *x,
            FilterData::Favorite(x) => nature.favorite() == *x,
            FilterData::Disliked(x) => nature.disliked() == *x,
        })
}

fn write_nature(f: &mut String, nature: Nature) {
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
