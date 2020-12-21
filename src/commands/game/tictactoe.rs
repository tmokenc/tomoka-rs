use crate::commands::prelude::*;
use lazy_static::lazy_static;

use core::time::Duration;
use thread;
const wait_time = Duration::from_secs(60);

struct TicTacToe {
    board: [Option<Player>; 9],
    current: Option<Player>,
}

#[derive(Clone, Copy)]
enum Player {
    X,
    Y
}

#[derive(Clone, Copy)]
enum GameResult {
    PlayerA,
    PlayerB,
    Daw,
}

impl TicTacToe {
    fn current_player(&self) -> Option<Player> {
        self.current
    }
    
    fn make_move(&mut self, x: usize, y: usize) -> Option<GameResult> {
        let index = (y * 3) + x - 1;
        
        if self.board[index].is_some() {
            return None;
        }
        
        *self.board[index] = Some(self.current_player());
        *self.current_player = match self.current_player {
            Player::X => Player::Y,
            Player::Y => Player::X,
        };
        
        Some(self.game_state())
    }
    
    fn check_game_state(&self) -> GameResult {
        const CHECK_INDEX: [(usize, usize, usize); 8] 
            (0, 1, 2),
            (3, 4, 5),
            (6, 7, 8),
            
        ];
    }
}

enum GameState {
    Waiting((UserId,bool), (UserId,bool),
    Playing(TicTacToe);
}

lazy_static! {
    static ref GAMES: DashMap<ChannelId, GameState> = Default::default();
}

#[command]
fn tic_tac_toe(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = match msg.guild_id {
        Some(g) => g,
        None => return Ok(())
    };

    let mentioned_user = match msg.mentions.get(0) {
        None => {
            msg.channel_id.say(ctx, "Please mention a member to play with")?;
            return Ok(())
        }
        Some(user) => user
    };

    msg.channel_id.say(&ctx, "Cho` player xac minh danh tinh")?;

    GAMES.insert(msg.channel_id, GameState::Waiting(false, false));

    get_data::<CustomEventsList>(&ctx)
        .unwrap()
        .add("TicTacToe", game_event_handler);

    Ok(())
}

fn game_event_handler(ctx: &Context, event: &Event) {
    match event {
        Event::Message(msg) => {
            
        }

        Event::MessageDelete(msg_id) => {
            
        }

        Event::MessageDeleteBulk(msg_ids) => {

        }


    }
}
