pub mod board;
pub mod colour;
pub mod game;
pub mod pos;
pub mod tile;
pub mod turn;

pub type StrResult<T> = Result<T, &'static str>;
