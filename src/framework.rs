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
    "translate",
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
        .group(&OSU_GROUP)
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
        // eika: 
        // NDMyMDYzMjM4Nzg4ODc0MjUw.WshkPw.lwk5nZC0UMpk5Yit4lq0vyQ5YXw
    
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
            animated: false,
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
        let response = data
            .get::<AIStore>()
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
    
    if msg.author.id != UserId(314444746959355905) {
        return Ok(())
    }

    let config = crate::read_config().await;
    let rgb = match config.rgb.as_ref() {
        Some(r) => r,
        None => return Ok(()),
    };
    
    if !matches!(msg.guild_id, Some(v) if rgb.tu_server.contains(&v)) {
        return Ok(())
    }

    if !msg.content
        .to_lowercase()
        .split_whitespace()
        .map(SmallString::from)
        .any(|v| rgb.tu.contains(&v))
    {
        return Ok(());
    }

    let mut rng = SmallRng::from_entropy();
    let num = rng.gen::<f32>();

    msg.channel_id
        .send_embed(ctx)
        .with_color((num * 0xffffff as f32 - 1.0) as u32)
        .with_image("https://cdn.discordapp.com/attachments/418811018698031107/661658331613495297/2019-09-15_220414.png")
        .await?;
        
    // msg
    // .channel_id
    // .send_message(&ctx, |m| m.embed(|embed| {
    //     embed
    //     .color((num * 0xffffff as f32 - 1.0) as u32)
    //     .image("https://cdn.discordapp.com/attachments/418811018698031107/661658331613495297/2019-09-15_220414.png")
    // }))
    // .await?;

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

    let data = sauces.get(0).cloned();

    let fut_a = async {
        if let Some(sauce) = data {
            post_to_fb(ctx, msg, sauce, req).await?;
        }

        crate::Result::Ok(())
    };

    let fut_b = react_to_pagination(ctx, msg, reaction, timeout, sauces);

    let (a, b) = future::join(fut_a, fut_b).await;
    a.and(b)?;

    Ok(())
}

use requester::saucenao::SauceNao;
use serde::Deserialize;
use serenity::http::Http;
use serenity::model::id::{ChannelId, GuildId};
use serenity::prelude::TypeMapKey;

#[inline]
fn is_acceptable_size(msg: &Message, data: &SauceNao) -> bool {
    const MAX_FB_IMG_SIZE: u64 = 4 * 1024 * 1024;

    msg.attachments
        .iter()
        .find(|v| v.url.as_str() == data.img_url())
        .filter(|v| v.size < MAX_FB_IMG_SIZE)
        .is_some()
}

fn facebook_description(author: &str, data: &SauceNao) -> String {
    let mut description = String::new();

    if let Some(title) = &data.title {
        description.push_str(&title);
    }

    description.push_str(" <3\n");

    let parodies = data
        .parody
        .iter()
        .filter(|v| v.as_str() != "original")
        .collect::<Vec<_>>();
    if !parodies.is_empty() {
        description.push_str("\nParody:\n");
        for parody in parodies {
            writeln!(&mut description, "{}", parody).ok();
        }
    }

    description.push_str("\nSource:\n");
    for (name, sauce) in data.sources.iter() {
        // writeln!(&mut description, "[{}]({})", name, sauce).ok();
        writeln!(&mut description, "{}: {}", name, sauce).ok();
    }

    write!(&mut description, "\n#{}", author).ok();

    description
}

async fn post_to_fb(
    ctx: &Context,
    msg: &Message,
    data: Ref<SauceNao>,
    req: <ReqwestClient as TypeMapKey>::Value,
) -> Result<()> {
    if data.sources.is_empty() || !is_acceptable_size(msg, &*data) {
        return Ok(());
    }

    if !matches!(msg.guild_id, Some(GuildId(418811018244784129))) {
        return Ok(());
    }

    if is_nsfw_channel(ctx, msg.channel_id).await {
        return Ok(());
    }

    let reaction = ReactionType::Unicode(String::from("💟"));
    let timeout = Duration::from_secs(30);

    let author = match wait_for_reaction(ctx, msg, reaction, timeout).await? {
        Some(UserId(239825449637642240)) => "tmokenc",
        Some(UserId(353026384601284609)) => "myon",
        Some(UserId(303146279884685314)) => "Kai",
        _ => return Ok(()),
    };

    let (url, query) = {
        let config = crate::read_config().await;
        match config.apikeys.facebook.as_ref() {
            Some(page) => {
                let url = format!("https://graph.facebook.com/{}/photos", page.id);
                let mut query = std::collections::HashMap::new();

                query.insert("url", data.img_url().to_owned());
                query.insert("access_token", page.token.to_owned());
                query.insert("caption", facebook_description(author, &*data));

                (url, query)
            }

            None => return Ok(()),
        }
    };

    #[derive(Deserialize)]
    struct PagePhotoPost {
        id: String,
        post_id: String,
    }

    let content = format!("Posting to Loli Chronicle as **#{}**", author);

    let mess = msg
        .channel_id
        .send_embed(ctx)
        .with_description(&content)
        .with_thumbnail(data.img_url())
        .with_color(crate::read_config().await.color.information)
        .with_current_timestamp();

    let post = async { req.post(&url).query(&query).send().await?.text().await };

    let (mess, post) = future::join(mess, post).await;
    let mut embed = serenity::builder::CreateEmbed::default();

    embed.timestamp(now());
    embed.thumbnail(data.img_url());

    let text = post?;
    let post = serde_json::from_str::<PagePhotoPost>(&text);

    match post {
        Ok(_post) => {
            embed.description(format!("Successfully posted as **#{}**!!!", author));
            embed.color(crate::read_config().await.color.success);

            match mess {
                Ok(mess) => {
                    mess.channel_id
                        .0
                        .edit_message(ctx, mess.id)
                        .with_embed(embed)
                        .await?
                }

                Err(_) => {
                    msg.channel_id
                        .send_embed(ctx)
                        .with_embedable_object(embed)
                        .await?
                }
            };
        }

        Err(why) => {
            log::error!("Error while posting image to facebook\n{:#?}", text);
            embed.description(format!("Error while posting the image```{:#?}```", why));
            embed.color(crate::read_config().await.color.error);
            match mess {
                Ok(mess) => {
                    mess.channel_id
                        .0
                        .edit_message(ctx, mess.id)
                        .with_embed(embed)
                        .await?
                }

                Err(_) => {
                    msg.channel_id
                        .send_embed(ctx)
                        .with_embedable_object(embed)
                        .await?
                }
            };
        }
    }

    Ok(())
}

// Simply a clone of the find_sauce due to similar functionality
async fn find_sadkaede(ctx: &Context, msg: &Message) -> Result<()> {
    if msg.content.len() < 20 {
        return Ok(());
    }

    if msg.is_own(&ctx).await {
        return Ok(());
    }

    let config = crate::read_config().await;

    // let is_watching_channel = msg
    //     .guild_id
    //     .and_then(|v| config.guilds.get(&v))
    //     .filter(|v| v.find_sadkaede.enable)
    //     .filter(|v| v.find_sadkaede.all || v.find_sadkaede.channels.contains(&msg.channel_id.0))
    //     .is_some();

    // if !is_watching_channel || msg.is_own(&ctx).await {

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

    let req = get_data::<ReqwestClient>(&ctx).await.unwrap();
    let data = req.gmetadata(gids.into_iter().take(10)).await?;

    let is_channel_nsfw = is_nsfw_channel(&ctx, msg.channel_id).await;
    let data: Vec<_> = data
        .into_iter()
        .filter(|data| is_channel_nsfw || data.is_sfw())
        .map(Ref)
        .collect();

    if data.is_empty() {
        return Ok(());
    }

    let nhen = {
        let data = &data[0];
        let query = panda2spider_query(data);

        dbg!(&query);

        async {
            let query = query; // to take ownership
            use requester::nhentai::NhentaiScraper;
            let data = req.search(&query).await?;
            if let Some(id) = data.results.first() {
                let config = crate::read_config().await;
                let timeout = Duration::from_secs(config.nhentai.wait_duration as u64);
                let reaction: ReactionType = match config.emoji.nhentai.parse() {
                    Ok(r) => r,
                    Err(_) => return Ok(()),
                };

                drop(config);

                let res = req.gallery_by_id(id.id).await?.map(Ref).unwrap();
                react_to_embed_then_pagination(ctx, msg, reaction, timeout, res).await?;
            }

            Result::Ok(())
        }
    };

    let kaede = react_to_pagination(ctx, msg, reaction, timeout, data);

    let (kaede, nhen) = future::join(kaede, nhen).await;
    kaede.and(nhen)?;

    Ok(())
}

fn panda2spider_query(kaede: &requester::ehentai::Gmetadata) -> String {
    let mut res = kaede
        .title
        .as_ref()
        .or_else(|| kaede.title_jpn.as_ref())
        .map(String::from)
        .unwrap_or_default();

    if res.ends_with('}') {
        if let Some(index) = res.rfind('{') {
            res.truncate(index);
        }
    }

    write!(&mut res, " pages:{}", kaede.filecount).unwrap();

    let tags = kaede.parse_tags();

    if let Some(lang) = tags.language.as_ref() {
        for tag in lang.iter() {
            write!(&mut res, " language:{}", tag).unwrap();
        }
    }

    res
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

    let (reaction, timeout) = {
        let config = crate::read_config().await;
        let timeout = Duration::from_secs(config.nhentai.wait_duration as u64);
        let reaction: ReactionType = match config.emoji.nhentai.parse() {
            Ok(r) => r,
            Err(_) => return Ok(()),
        };

        (reaction, timeout)
    };

    //let is_watching_channel = msg
    //    .guild_id
    //    .and_then(|v| config.guilds.get(&v))
    //    .filter(|v| v.find_sadkaede.enable)
    //    .filter(|v| v.find_sadkaede.all || v.find_sadkaede.channels.contains(&msg.channel_id.0))
    //    .is_some();

    //if !is_watching_channel {
    //    return Ok(());
    //}

    let data = get_data::<ReqwestClient>(ctx)
        .await
        .unwrap()
        .gallery_by_id(id)
        .await?
        .map(Ref);

    if let Some(gallery) = data {
        react_to_embed_then_pagination(ctx, msg, reaction, timeout, gallery).await?;
    }

    Ok(())
}
