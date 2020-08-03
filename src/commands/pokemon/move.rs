use crate::commands::prelude::*;
use super::{PokeKey, PokeKeyKind, process_data, parse_args};

#[command("move")]
async fn moves(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let (moves, gen) = match parse_args(args.rest()) {
        Some(v) => v,
        None => return Err("Please specify a move".into())
    };
    
    let key = PokeKey::new(&moves, gen, PokeKeyKind::Move);
    process_data(ctx, key, msg, None).await?;
    Ok(())
}
