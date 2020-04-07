use crate::commands::prelude::*;

#[command]
#[min_args(1)]
#[max_args(2)]
#[only_in(guilds)]
#[example = "5"]
#[usage = "<how_many> ?[-m | --manual]"]
#[required_permissions("MANAGE_MESSAGES")]
/// Delete x number of messages
/// I doesn't have the permission for delete the messages that are older than 2 weeks
/// But it be done by pass the __\"-m\"__ or __\"--manual\"__ to delete them one by one, which is slower but it works
async fn prune(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let channel = msg.channel_id;
    let total = args.find::<u64>()?;
    let manual = args.iter::<String>().find(|v| {
        v.as_ref()
            .ok()
            .map_or(false, |s| s == "-m" || s == "--manual")
    });
    
    let msgs = channel.messages(&ctx.http, |m| m.before(msg.id.0).limit(total)).await?;
    let count = msgs.len();

    msg.delete(&ctx).await?;
    if manual.is_some() {
        futures::future::join_all(msgs.iter().map(|v| v.delete(&ctx))).await;
    } else {
        channel.delete_messages(&ctx, msgs).await?;
    }

    info!("Deleted {} messages", count);
    Ok(())
}
