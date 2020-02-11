use crate::commands::prelude::*;
use serenity::model::event::Event;
use serenity::model::id::UserId;
use lazy_static::lazy_static;
use rand::prelude::*;
use tetris_core::{Size, Randomizer, Game as Tetris};

lazy_static! {
    static ref GAME_STATE: DashMap<ChannelId, TetrisInstance> = Default::default();
}

struct TetrisInstance {
    player: UserId,
    game: Tetris,
    message_id: MessageId,
}

struct Rand;

impl Randomizer for Rand {
    fn random_between(&self, lower: i32, higher: i32) -> i32 {
        let mut rng = SmallRng::from_entropy();
        rng.gen_range(lower, higher)
    }
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
    if GAME_STATE.contains_key(&msg.channel_id) {
        msg.channel_id.say(ctx, "A game is activating on this channel")?;
        return Ok(())
    }
    
    let size = Size {
        width: 10,
        height: 20,
    };
    
    let r = Rand {};
    
    let game = Tetris::new(size,r);
    
    let message_id = msg.channel_id.send_message(ctx, |m| {
        m.embed(|embed| {
            embed.description("The game will start here");
            
            embed
        });
        
        m.reactions(&[
            "⏬ ",
            "↩️",
            "⬅️",
            "➡️",
            "↪️",
            "⬇️",
        ]);
        m
    })?;
    
    let instance = TetrisInstance {
        message_id: msg.id,
        player: msg.author.id,
        game,
    };
    
    GAME_STATE.insert(msg.channel_id, instance);
    

    
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