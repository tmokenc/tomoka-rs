use crate::commands::prelude::*;
use super::{PokeKey, PokeKeyKind, process_data, parse_args};


#[command]
async fn ability(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let (ability, gen) = match parse_args(args.rest()) {
        Some(v) => v,
        None => return Err("Please specify a ability".into())
    };
    
    let key = PokeKey::new(&ability, gen, PokeKeyKind::Ability);
    process_data(ctx, key, msg, None).await?;
    Ok(())
}