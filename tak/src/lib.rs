#![feature(array_zip)]

#[macro_use]
extern crate lazy_static;

pub mod board;
pub mod colour;
pub mod direction;
pub mod game;
pub mod pos;
pub mod ptn;
pub mod symm;
pub mod tile;
pub mod turn;

pub type StrResult<T> = Result<T, String>;
