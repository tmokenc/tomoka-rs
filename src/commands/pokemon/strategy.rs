use crate::commands::prelude::*;
use requester::smogon::{MoveSet, SmogonApi as _};
use scraper::{Html, Selector};
use magic::traits::MagicIter;

#[command]
#[aliases("smogon", "strategy")]
async fn smogon_strategy(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let pokemon = args.rest();
    let aliasized = pokemon.replace(" ", "-").to_lowercase();
    let title = format!("Strategies for {}", pokemon);

    let strategies = get_data::<ReqwestClient>(&ctx)
        .await
        .unwrap()
        .strategy(&aliasized, Default::default())
        .await?;

    if strategies.is_empty() {
        let color = crate::read_config().await.color.error;
        
        msg.channel_id.send_message(&ctx, |m| {
            m.embed(|embed| {
                embed.title(title);
                embed.description(format!(
                    "Not found any strategy for the pokemon **{}**",
                    pokemon
                ));
                embed.timestamp(now());
                embed.color(color);
                
                embed
            })
        }).await?;
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
    
    let color = {
        let config = crate::read_config().await;
        config.color.information
    };

    msg.channel_id.send_message(&ctx, |m| {
        m.embed(|embed| {
            embed.title(title);
            embed.description(description);
            embed.fields(fields);
            embed.thumbnail(sprite);
            embed.timestamp(now());
            embed.color(color);
            
            embed
        })
    }).await?;

    Ok(())
}

fn format_overview(m: &str) -> String {
    let fragment = Html::parse_fragment(m);
    let li_selector: Selector = Selector::parse("li").unwrap();

    let li = fragment
        .select(&li_selector)
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
            .map(|(v, i)| {
                let moves = v.iter().map(|v| {
                    let mut res = v.name.to_owned();
                    if let Some(ref typ) = v.typ {
                        res.push(' ');
                        res.push_str(typ);
                    }
                    res
                }).join(" / ");
                
                format!("{}. {}\n", i, moves)
                
            })
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
