use crate::commands::prelude::*;
use crate::storages::ReqwestClient;
use lazy_static::lazy_static;
use requester::smogon::{MoveSet, SmogonRequester};
use scraper::{Html, Selector};

#[command]
#[aliases("smogon", "strategy")]
fn smogon_strategy(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let req = get_data::<ReqwestClient>(&ctx).unwrap();
    let pokemon = args.rest();
    let aliasized = pokemon.replace(" ", "-").to_lowercase();
    let title = format!("Strategies for {}", pokemon);

    let strategies = block_on(req.strategy(&aliasized, None))?;

    if strategies.is_empty() {
        msg.channel_id.send_message(&ctx, |m| {
            m.embed(|embed| {
                embed
                    .title(title)
                    .description(format!(
                        "Not found any strategy for the pokemon **{}**",
                        pokemon
                    ))
                    .color(ERROR_COLOR)
                    .timestamp(now())
            })
        })?;
        return Ok(());
    }

    let strategy = strategies.get(0).unwrap();
    let sprite = format!(
        "https://www.smogon.com/dex/media/sprites/xy/{}.gif",
        aliasized
    );

    let description = format!(
        "**Format: {}**\n{}",
        strategy.format,
        format_overview(&strategy.overview)
    );

    let fields: Vec<(String, String, bool)> = strategy
        .movesets
        .iter()
        .map(|v| (v.name.to_owned(), format_moveset(&v), false))
        .collect();

    msg.channel_id.send_message(&ctx, |m| {
        m.embed(|embed| {
            embed
                .title(title)
                .description(description)
                .fields(fields)
                .thumbnail(sprite)
                .color(INFORMATION_COLOR)
                .timestamp(now())
        })
    })?;

    Ok(())
}

fn format_overview(m: &str) -> String {
    let fragment = Html::parse_fragment(m);

    lazy_static! {
        static ref LI_SELECTOR: Selector = Selector::parse("li").unwrap();
    }

    let li = fragment
        .select(&LI_SELECTOR)
        .map(|v| v.inner_html())
        .collect::<Vec<_>>();

    if li.is_empty() {
        fragment.root_element().text().collect()
    } else {
        li.into_iter().map(|v| format!("- {}\n", v)).collect()
    }
}

fn format_moveset(m: &MoveSet) -> String {
    let mut result = format!(
        "{}\n**Item**: {}\n**Nature**: {}\n**Ability**: {}\n**EVs**: {}",
        m.moveslots
            .iter()
            .zip(1..)
            .map(|(v, i)| format!("{}. {}\n", i, v.join(" / ")))
            .collect::<String>(),
        m.items.join(" / "),
        m.nature(),
        m.ability(),
        m.ev_config().join(" | "),
    );

    if !m.ivconfigs.is_empty() {
        let iv = format!("\n**__IV__**: {}", m.iv_config().join(" | "));
        result.push_str(&iv);
    }

    result
}
