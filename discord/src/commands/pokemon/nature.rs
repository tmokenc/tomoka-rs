use crate::commands::prelude::*;
use pokemon_core::Nature;
use std::fmt::Write;

#[command]
/// Get a pokemon nature information or get all of them
fn nature(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    // let filter = args.rest().split_whitespace();

    let mut data = String::new();

    for nature in Nature::iter() {
        write_nature(&mut data, nature);
    }

    msg.channel_id.say(&ctx, data)?;

    Ok(())
}

fn write_nature(f: &mut String, nature: Nature) {
    writeln!(
        f,
        "{} {} {} {} {}",
        nature,
        nature.increase(),
        nature.decrease(),
        nature.favorite(),
        nature.dislike(),
    )
    .unwrap();
}
