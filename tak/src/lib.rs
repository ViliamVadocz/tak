pub mod board;
pub mod colour;
pub mod game;
pub mod turn;
pub mod tile;

pub type StrResult<T> = Result<T, &'static str>;
