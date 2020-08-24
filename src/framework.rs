#![allow(unstable_name_collisions)]

use serenity::client::Context;
use serenity::framework::standard::macros::{help, hook};
use serenity::framework::{
    standard::{help_commands, Args, CommandGroup, CommandResult, Configuration, HelpOptions},
    Framework, StandardFramework,
};
use serenity::model::{
    channel::{Message, ReactionType},
    id::{MessageId, UserId},
    misc::EmojiIdentifier,
};

use crate::{
    commands::*,
    storages::{AIStore, InforKey, ReqwestClient},
    traits::{ChannelExt, Embedable},
    types::Ref,
    utils::*,
    Result,
};

use chrono::{DateTime, Utc};
use colorful::Colorful;
use core::time::Duration;
use dashmap::DashMap;
use futures::future;
use lazy_static::lazy_static;
use magic::has_external_command;
use requester::ehentai::EhentaiApi;
use smallstr::SmallString;
use std::collections::HashSet;
use std::sync::Arc;

use magic::traits::MagicBool as _;
use magic::traits::MagicIter as _;
use magic::traits::MagicStr as _;
use std::fmt::Write as _;

const TYPING_LIST: &[&str] = &[
    "leaderboard",
    "diancie",
    "say",
    "flip",
    "rotate",
    "saucenao",
    "info",
    "pokemon",
];

lazy_static! {
    static ref EXECUTION_LIST: DashMap<MessageId, DateTime<Utc>> = DashMap::new();
}

#[help]
#[individual_command_tip = "Hello! こんにちは！\n\
If you want more information about a specific command, just pass the command as argument."]
#[command_not_found_text = "Could not find: `{}`."]
#[embed_success_colour(MEIBE_PINK)]
#[embed_error_colour(ROSEWATER)]
#[max_levenshtein_distance(3)]
#[indention_prefix = "+"]
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]
async fn stolen_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

pub fn get_framework() -> impl Framework {
    let mut framework = StandardFramework::new()
        // .bucket("basic", |b| b.delay(2).time_span(10).limit(3))
        // .await
        .group(&MASTER_GROUP)
        .group(&GENERAL_GROUP)
        .group(&GUILDMASTER_GROUP)
        .group(&ADMINISTRATION_GROUP)
        .group(&CORONA_GROUP)
        .group(&GAME_GROUP)
        .group(&POKEMON_GROUP)
        .group(&UTILITY_GROUP)
        .group(&IMAGE_GROUP)
        .group(&RGB_GROUP)
        .help(&STOLEN_HELP)
        .configure(framwork_config)
        .before(before_cmd)
        .after(after_cmd)
        .normal_message(normal_message);

    //if has_external_command("ffmpeg") {
    //    framework.group_add(&MUSIC_GROUP);
    //}

    framework
}

fn framwork_config(config: &mut Configuration) -> &mut Configuration {
    let mut owners = HashSet::new();
    let mut disabled_commands = HashSet::new();

    owners.insert(UserId(239825449637642240));

    if !has_external_command("ffmpeg") {
        disabled_commands.insert(String::from("touhou_music_quiz"));
    }

    if !has_external_command("youtube-dl") {
        disabled_commands.insert(String::from("play"));
    }

    config
        .owners(owners)
        .disabled_commands(disabled_commands)
        .case_insensitivity(true)
        .by_space(true)
        .no_dm_prefix(true)
        .dynamic_prefix(normal_prefix)
        .dynamic_prefix(master_prefix)
}

#[hook]
async fn normal_prefix(_ctx: &Context, msg: &Message) -> Option<String> {
    let config = crate::read_config().await;

    msg.guild_id
        .and_then(|guild| config.guilds.get(&guild))
        .and_then(|guild| guild.prefix.to_owned())
        .or_else(|| Some(config.prefix.to_owned()))
        .map(|v| v.to_string())
}

#[hook]
async fn master_prefix(_ctx: &Context, msg: &Message) -> Option<String> {
    let config = crate::read_config().await;

    config
        .masters
        .iter()
        .any(|&v| v == msg.author.id)
        .then(|| config.master_prefix.to_string())
}

#[hook]
async fn before_cmd(ctx: &Context, msg: &Message, cmd_name: &str) -> bool {
    info!("Found command {}", cmd_name.bold().underlined());

    if TYPING_LIST.contains(&cmd_name) {
        typing(&ctx, msg.channel_id);
    }

    if crate::read_config()
        .await
        .cmd_blacklist
        .contains(&cmd_name.into())
    {
        return false;
    }

    EXECUTION_LIST.insert(msg.id, Utc::now());
    true
}

#[hook]
async fn after_cmd(ctx: &Context, msg: &Message, cmd: &str, err: CommandResult) {
    let start_time = match EXECUTION_LIST.remove(&msg.id) {
        Some((_, v)) => v.timestamp_millis(),
        None => msg.timestamp.timestamp_millis(),
    };

    match err {
        Ok(_) => {
            let now = Utc::now().timestamp_millis();
            info!(
                "Successfully executed the command {}, time passed {}ms",
                cmd.cyan(),
                now - start_time
            );
        }
        Err(why) => {
            error!("Couldn't execute the command {}\n{:#?}", cmd.magenta(), why);
            let mess = format!("Cannot execute the command **__{}__**```{}```", cmd, why);

            msg.channel_id
                .send_embed(ctx)
                .with_color(crate::read_config().await.color.error)
                .with_description(mess)
                .await
                .ok();
        }
    }

    if let Some(info) = ctx.data.read().await.get::<InforKey>() {
        info.executed_one();
    }
}

#[hook]
async fn normal_message(ctx: &Context, msg: &Message) {
    if msg.author.bot {
        return;
    }

    let config = crate::read_config().await;
    let mut futs = Vec::new();
    let mut names = Vec::new();

    macro_rules! exec_func {
        ( $( $x:ident ),* ) => {
            $(
                let func = SmallString::from(stringify!($x));

                if !config.disable_auto_cmd.contains(&func) {
                    futs.push($x(&ctx, &msg).boxed());
                    names.push(func);
                }
            )*
        };
    }

    exec_func! {
        mention_rgb,
        repeat_words,
        respect,
        eliza_response,
        rgb_tu,
        find_sauce,
        find_sadkaede,
        find_nhentai
    }

    drop(config);
    future::join_all(futs)
        .await
        .into_iter()
        .zip(names)
        .filter_map(|(func, name)| func.err().map(|e| (e, name)))
        .for_each(|(err, name)| error!("Cannot exec the {} autocmd \n{:#?}", name, err));
}

async fn mention_rgb(ctx: &Context, msg: &Message) -> Result<()> {
    let guild_id = match msg.guild_id {
        Some(v) => v,
        None => return Ok(()),
    };

    if !msg.content.contains("@rgb") {
        return Ok(());
    }

    let to_say = crate::read_config()
        .await
        .guilds
        .get(&guild_id)
        .as_deref()
        .and_then(|v| v.rgblized.as_ref())
        .map(|v| {
            let mut res = String::with_capacity(v.len() * 22);
            for role in v {
                write!(&mut res, "{}", role).unwrap();
            }
            res
        });

    if let Some(m) = to_say {
        for roles in m.split_at_limit(2000, ">") {
            msg.channel_id.say(&ctx, roles).await?;
        }
    }

    Ok(())
}

async fn repeat_words(ctx: &Context, msg: &Message) -> Result<()> {
    let guild_id = match msg.guild_id {
        Some(g) => g,
        None => return Ok(()),
    };

    let config = crate::read_config().await;
    let guild = match config.guilds.get(&guild_id) {
        Some(d) => d,
        None => return Ok(()),
    };

    if guild.repeat_words.enable && !guild.repeat_words.words.is_empty() {
        let mess: String = msg
            .content
            .split_whitespace()
            .filter(|v| guild.repeat_words.words.contains(&v.to_lowercase()))
            .map(|v| format!("**{}**", v))
            .join(", ");

        if !mess.is_empty() {
            msg.channel_id.say(ctx, mess).await?;
        }
    }

    Ok(())
}

async fn respect(ctx: &Context, msg: &Message) -> Result<()> {
    if msg
        .content
        .split_whitespace()
        .next()
        .filter(|&v| v == "f" || v == "F")
        .is_none()
    {
        return Ok(());
    }

    let mut content = format!("**{}** has paid their respects", msg.author.name);

    if msg.content.len() > 2 {
        let arg = remove_emote(&msg.content[2..]);
        if !arg.is_empty() {
            write!(&mut content, " for **{}**", arg.trim())?;
        }
    }

    content.push('.');

    let emoji = match crate::read_config().await.respect_emoji {
        None => ReactionType::from('🇫'),
        Some(id) => ReactionType::from(EmojiIdentifier {
            id,
            name: "f_".to_string(),
        }),
    };

    msg.channel_id
        .send_message(ctx, |message| {
            message.content(content);
            message.reactions(Some(emoji));
            message
        })
        .await?;

    Ok(())
}

async fn eliza_response(ctx: &Context, msg: &Message) -> Result<()> {
    let data = ctx.data.read().await;
    let me = data.get::<InforKey>().unwrap().user_id;

    if msg.mentions_user_id(me) {
        let input = remove_mention(&msg.content);
        let response = get_data::<AIStore>(&ctx)
            .await
            .expect("Expected brain in ShareMap.")
            .lock()
            .await
            .respond(&input);

        drop(data);

        msg.channel_id.say(&ctx.http, response).await?;
    }

    Ok(())
}

async fn rgb_tu(ctx: &Context, msg: &Message) -> Result<()> {
    use rand::prelude::*;

    let config = crate::read_config().await;
    let rgb = match config.rgb.as_ref() {
        Some(r) => r,
        None => return Ok(()),
    };

    if msg.guild_id.is_none()
        || msg.author.id.0 != 314444746959355905
        || !msg
            .content
            .to_lowercase()
            .split_whitespace()
            .map(SmallString::from)
            .any(|v| rgb.tu.contains(&v))
    {
        return Ok(());
    }

    let mut rng = SmallRng::from_entropy();
    let num = rng.gen::<f32>();

    msg
    .channel_id
    .send_message(&ctx, |m| m.embed(|embed| {
        embed
        .color((num * 0xffffff as f32 - 1.0) as u32)
        .image("https://cdn.discordapp.com/attachments/418811018698031107/661658331613495297/2019-09-15_220414.png")
    }))
    .await?;

    if num < 0.05 {
        use tokio::fs;
        use tokio::stream::StreamExt;

        let path = &rgb.evidence;

        let evi = fs::read_dir(path)
            .await?
            .filter_map(|v| v.ok())
            .collect::<Vec<_>>()
            .await
            .choose(&mut rng)
            .map(|v| v.path());

        drop(config);

        if let Some(evi) = evi {
            msg.channel_id
                .send_message(&ctx, |m| m.add_file(&evi))
                .await?;
        }
    }

    Ok(())
}

async fn find_sauce(ctx: &Context, msg: &Message) -> Result<()> {
    use requester::SauceNaoScraper;

    let config = crate::read_config().await;

    let is_watching_channel = msg
        .guild_id
        .and_then(|v| config.guilds.get(&v))
        .filter(|v| v.find_sauce.enable)
        .filter(|v| v.find_sauce.all || v.find_sauce.channels.contains(&msg.channel_id.0))
        .is_some();

    if !is_watching_channel || msg.is_own(&ctx).await {
        return Ok(());
    }

    let timeout = Duration::from_secs(config.sauce.wait_duration as u64);
    let reaction = match config.emoji.sauce.parse() {
        Ok(r) => r,
        Err(_) => return Ok(()),
    };

    drop(config);

    let req = get_data::<ReqwestClient>(&ctx).await.unwrap();
    let sauces = msg
        .attachments
        .iter()
        .filter(|v| v.width.is_some())
        .filter(|v| !v.url.ends_with(".gif"))
        .map(|v| {
            let req = Arc::clone(&req);
            async move { req.saucenao(&v.url, None).await }
        });

    let sauces: Vec<_> = future::join_all(sauces)
        .await
        .into_iter()
        .filter_map(|v| v.ok())
        .filter(|v| v.found())
        .map(Ref)
        .collect();

    if sauces.is_empty() {
        return Ok(());
    }

    drop(req);
    react_to_pagination(ctx, msg, reaction, timeout, sauces).await?;
    Ok(())
}

// Simply a clone of the find_sauce due to similar functionality
async fn find_sadkaede(ctx: &Context, msg: &Message) -> Result<()> {
    if msg.content.len() < 20 {
        return Ok(());
    }

    let config = crate::read_config().await;

    let is_watching_channel = msg
        .guild_id
        .and_then(|v| config.guilds.get(&v))
        .filter(|v| v.find_sadkaede.enable)
        .filter(|v| v.find_sadkaede.all || v.find_sadkaede.channels.contains(&msg.channel_id.0))
        .is_some();

    if !is_watching_channel || msg.is_own(&ctx).await {
        return Ok(());
    }

    let gids = parse_eh_token(&msg.content);
    if gids.is_empty() {
        return Ok(());
    }

    let reaction = match config.emoji.sadkaede.parse() {
        Ok(r) => r,
        Err(_) => return Ok(()),
    };

    let timeout = Duration::from_secs(config.sadkaede.wait_duration as u64);

    drop(config);

    let data = get_data::<ReqwestClient>(&ctx)
        .await
        .unwrap()
        .gmetadata(gids.into_iter().take(25))
        .await?;

    let is_channel_nsfw = is_nsfw_channel(&ctx, msg.channel_id).await;
    let data: Vec<_> = data
        .into_iter()
        .filter(|data| is_channel_nsfw || data.is_sfw())
        .map(Ref)
        .collect();

    if data.is_empty() {
        return Ok(());
    }

    react_to_pagination(ctx, msg, reaction, timeout, data).await?;

    Ok(())
}

// Simply a clone of the find_sauce due to similar functionality
async fn find_nhentai(ctx: &Context, msg: &Message) -> Result<()> {
    use requester::nhentai::NhentaiScraper;

    let id = match msg.content.parse::<u64>() {
        Ok(id) => id,
        Err(_) => return Ok(()),
    };

    let check_nsfw = is_nsfw_channel(&ctx, msg.channel_id);
    let check_own_msg = msg.is_own(&ctx);
    let (nsfw, own) = future::join(check_nsfw, check_own_msg).await;

    if !nsfw || own {
        return Ok(());
    }

    let config = crate::read_config().await;

    let is_watching_channel = msg
        .guild_id
        .and_then(|v| config.guilds.get(&v))
        .filter(|v| v.find_sadkaede.enable)
        .filter(|v| v.find_sadkaede.all || v.find_sadkaede.channels.contains(&msg.channel_id.0))
        .is_some();

    if !is_watching_channel {
        return Ok(());
    }

    let reaction: ReactionType = match config.emoji.nhentai.parse() {
        Ok(r) => r,
        Err(_) => return Ok(()),
    };

    let timeout = Duration::from_secs(config.nhentai.wait_duration as u64);

    drop(config);

    let data = get_data::<ReqwestClient>(ctx)
        .await
        .unwrap()
        .gallery_by_id(id)
        .await?
        .map(Ref);

    if let Some(gallery) = data {
        if wait_for_reaction(ctx, msg, reaction, timeout)
            .await?
            .is_some()
        {
            let reaction = ReactionType::Unicode(String::from("📖"));
            let message = gallery.send_embed(ctx, msg.channel_id).await?;
            react_to_pagination(ctx, &message, reaction, timeout, gallery).await?;
        }
    }

    Ok(())
}
