mod board;
mod game;
mod game_result;
mod move_gen;
mod symm;
mod tile;

pub type StrResult<T> = Result<T, String>;

pub use game::{default_starting_stones, Game};
pub use game_result::GameResult;
pub use symm::Symmetry;
