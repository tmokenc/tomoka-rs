use crate::commands::prelude::*;
use crate::types::GuildConfig;
use magic::traits::MagicIter as _;

#[command]
#[aliases("channel", "channels")]
#[only_in("guilds")]
#[required_permissions(MANAGE_GUILD)]
/// Add channel(s) to be watcing for sauce
/// *This* command will automatically enable the sauce machine, even when it is disabled 
fn add(ctx: &mut Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(id) => id,
        None => return Ok(())
    };

    let channels = extract_channel_ids(&msg.content);
    
    if channels.is_empty() {
            msg.channel_id.send_message(ctx, |m| {
                m.content("Please *mention* some channel to be watched")
            })?;
            return Ok(());
    }
    
    let config = crate::read_config();

    let mut guild = config
        .guilds
        .entry(guild_id)
        .or_insert_with(|| GuildConfig::new(guild_id));

    guild.enable_find_sauce();

    let (added, existed) = channels
        .iter()
        .fold((Vec::new(), Vec::new()), |mut v, x| {
            if !guild.find_sauce.channels.contains(&x) {
                guild.add_sauce_channel(x);
                v.0.push(x);
            } else {
                v.1.push(x);
            }
            v
        });
        
    update_guild_config(&ctx, &guild)?;

    msg.channel_id.send_message(ctx, |m| m.embed(|embed| {
        embed.title("Saucing information");
        embed.thumbnail("https://www.daringgourmet.com/wp-content/uploads/2017/04/Sweet-Sour-Sauce-1.jpg");
        embed.color(INFORMATION_COLOR);
        embed.timestamp(now());
        
        let mess = match (channels.len(), added.len()) {
            (1, 0) => String::from("I'm watching this channel already"),
            (_, 0) => String::from("I'm watching these channels already"),
            (_, 1) => String::from("Added a channel to be watching"),
            (v, x) if v - x == 1 => {
                let channel = existed.iter().next().unwrap();
                format!("Added {} channels to be watching, <#{}> already exists", x, channel) 
            }
            (v, x) if v > x => {
                let exist = existed
                    .into_iter()
                    .map(|v| format!("<#{}>", v))
                    .join(" ");
                    
                embed.field("Exist channel", exist, true);
                format!("Added {} channels to be watching, {} channels already exist", x, v - x) 
            },
            (_, x) => format!("Added {} channels to be watching", x)
        };

        if added.len() > 0 {
            let added = added
                .into_iter()
                .map(|v| format!("<#{}>", v))
                .join(" ");
            
            embed.field("Added channels", added, true);
        }

        embed.description(mess);
        embed
    }))?;

    Ok(())
}
