use crate::commands::prelude::*;
use lazy_static::lazy_static;

#[derive(Debug)]
struct TicTacToe {
    board: [Option<Mark>; 9],
    move_count: u8,
}

#[derive(Debug)]
enum Mark {
    X,
    O,
}

#[derive(Debug)]
enum GameOver {
    Hoa,
    Player1(Coordinate),
    PlayerTheSecond(Coordinate),
}

type Coordinate = (u8, u8);

impl TicTacToe {
    fn new_game(p1: UserId, p2: UserId) -> Self {
        Self { 
            board: [None; 9],
            move_count: 0,
        }
    }

    fn whose_move(&self) -> Mark {
        if self.move_count % 2 == 0 {
            Mark::O
        } else {
            Mark::X
        }
    }

    fn parse_input(s: impl AsRef<str>) -> Option<Coordinate> {
        let mut input = s.as_ref().bytes();
        let x = input.next();
        let y = input.next();

        if let (Some(x), Some(y)) = (x, y) {
            let x = match x {
                1 | 97 => 1,
                2 | 98 => 2,
                3 | 99 => 3,
                _ => return None,
            };

            if y != 1 || y != 2 || y != 3 {
                return None
            }

            Some((x, y))
        } else {
            None
        }
    }

    fn play(&mut self, input: impl AsRef<str>) -> bool {
        let coor: Coordinate = match Self::parse_input(input) {
            Some(c) => c,
            None => return None,
        };

        let index = (coor.0 * coor.1) + coor.1; 

        let mark = self.whose_move();

        if let Some(cell) =  board.get_mut(index as usize) {
            match cell {
                Some(_) => return false,
                None => {
                    *cell = Some(mark);
                }
            }
        }

        self.move_count += 1;
        true
    }

    #[inline]
    fn board(&self) -> [Option<Mark>; 9] {
        self.board
    }

    fn is_game_over(&self) -> Option<GameOver> {
        if self.move_count < 5 {
            return None    
        }
        
        let winner_hang = self
            .board
            .windows(3)
            .find(|v| v.dedup() == 1);

        if winner_hang.is_some() {
           return winner_hang;
        }

        let winner_cot = (0..3)
            .map(|v| [self.board[v], self.board[v + 3], self.board[v + 3 + 3]])
            .find(|v| v.dedup() == 1);

        if winner_cot.is_some() {
            return winner_cot;
        }

        let check_cheo1 = self.board[0] == self.board[4] == self.board[8];
        let check_cheo2 = self.board[2] == self.board[4] == self.board[6];

        if check_cheo1 || check_cheo2 {
            return self.board[4]
        }

        if self.board.iter().all(|v| v.is_some()) {
            GameOver::Hoa
        }
    }
}

use core::time::Duration;
use thread;
const wait_time = Duration::from_secs(60);

enum GameState {
    Waiting((UserId,bool), (UserId,bool),
    Playing(TicTacToe);
}

lazy_static! {
    static ref GAMES: DashMap<ChannelId, GameState> = Default::default();
}

#[command]
fn tic_tac_toe(ctx: &mut Context, msg: &Message) -> CommandResult {
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
