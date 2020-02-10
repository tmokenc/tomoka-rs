use crate::commands::prelude::*;
use serenity::model::event::Event;
use lazy_static::lazy_static;
use tetris_core::Game as Tetris;

lazy_static! {
    static ref GAME_STATE: DashMap<Tetris> = Default::default();
}



#[command]
/// Play Tetris on discord.
/// Use reaction or keyword to play it
/// ```
/// A = Left   | J = Rotate left
/// D = Right  | K = Rotate right
/// W = Up     | L = Hard drop
/// S = Down   | I = Hold
///
/// Q = Quit
/// ```
fn tetris(ctx: &mut Context, msg: &Message) -> CommandResult {
    
    
    
    Ok(())
}

fn tetris_event_handler(ctx: &Context, ev: &Event) {
    match ev {
        Event::Message(msg) => {
            
        }
        
        Event::ReactionAdd(reaction) => {
            
        }
        
        Event::MessageDelete(msg) => {
            
        }
        
        Event::MessageDeleteBulk(msgs) => {
            
        }
        
        _ => {}
    }
}