use serenity::framework::standard::macros::group;
use serenity::model::event::Event;
use serenity::prelude::*;
use magic::import_all;

import_all! {
    pause,
    resume,
    stop,
    radio
}

#[group]
#[commands(radio, pause, resume, stop)]
struct Music;

async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    Ok(())
}

async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    Ok(())
}

async fn play(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    Ok(())
}