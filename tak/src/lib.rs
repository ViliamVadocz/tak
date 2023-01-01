mod board;
mod colors;
mod error;
mod game;
mod game_result;
mod move_gen;
mod ptn;
mod reserves;
mod stack;
mod symm;
mod tps;
mod wins;

pub use error::{PlayError, StackError, TakeError};
pub use game::Game;
pub use game_result::GameResult;
pub use move_gen::perf_count;
pub use symm::Symmetry;

// TODO
// Code coverage
// Documentation
// Tests
// Criterion profiling
