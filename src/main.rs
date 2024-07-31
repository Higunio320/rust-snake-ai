use crate::game::play_game;

mod snake_game;
mod game;
mod neural_network;
mod genetic_algorithm;

fn main() {
    if let Err(error) = play_game() {
        eprintln!("Error happened during game: {}", error)
    }
}
