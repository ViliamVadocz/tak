#![feature(array_zip)]

#[macro_use]
extern crate lazy_static;

mod board;
mod colour;
mod direction;
mod game;
mod pos;
mod ptn;
mod symm;
mod tile;
mod tps;
mod turn;

pub type StrResult<T> = Result<T, String>;

// re-export so you can star import everything important
pub use board::Board;
pub use colour::Colour;
pub use game::{Game, GameResult};
pub use pos::Pos;
pub use ptn::{FromPTN, ToPTN};
pub use symm::Symmetry;
pub use tile::Tile;
pub use tps::{FromTPS, ToTPS};
pub use turn::Turn;
