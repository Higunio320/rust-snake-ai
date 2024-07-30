use crate::game::play_game;

mod snake_game;
mod game;

fn main() {
    if let Err(error) = play_game() {
        eprint!("Error happened during game: {}", error)
    }
}
