use crate::commands::prelude::*;
use regex::Regex;
use crate::traits::ToEmbed;
use requester::ehentai::EhentaiApi;
use lazy_static::lazy_static;

#[command]
#[owners_only]
/// Get the e-h gallery info
fn info(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    msg.channel_id.broadcast_typing(&ctx)?;
    
    let content = args.rest();
    let data = match parse_eh_token(content) {
        Some(d) => d,
        None => return Ok(())
    };
    let reqwest = get_data::<ReqwestClient>(&ctx).unwrap();
    let data = block_on(reqwest.gmetadata(Some(data)))?;
    
    for d in data {
        msg.channel_id.send_message(&ctx, |m| m.embed(|mut embed| {
            d.to_embed(&mut embed);
            embed
        }))?;
    }
    
    
    Ok(())
}

fn parse_eh_token(content: &str) -> Option<(u32, String)> {
    lazy_static! {
        static ref KAEDE_REG: Regex = Regex::new(r"e(x|\-)hentai.org/g/(\d+)/([[:alnum:]]+)").unwrap();
    }
    
    let res = match KAEDE_REG.captures(content) {
        Some(e) => e,
        None => return None
    };
    res.get(2)
        .and_then(|v| v.as_str().parse::<u32>().ok())
        .and_then(|v| res.get(3).map(|x| (v, x)))
        .map(|(v, x)| (v, x.as_str().to_string()))
}