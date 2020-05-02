use crate::commands::prelude::*;
use rand::prelude::*;

#[command]
#[min_args(1)]
/// Let me decide thing for you
async fn choose(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let mut rng = SmallRng::from_entropy();
    let chosen = args.rest().split('|').choose(&mut rng).map(|v| v.trim());

    if let Some(s) = chosen {
        msg.channel_id.say(ctx, format!("I choose **{}**", s)).await?;
    }

    Ok(())
}
