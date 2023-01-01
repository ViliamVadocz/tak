use std::{error::Error, fmt::Display};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum PlayError {
    OutOfBounds,
    AlreadyOccupied,
    NoCapstone,
    NoStones,
    OpeningNonFlat,
    EmptySquare,
    StackNotOwned,
    StackError(StackError),
    TakeError(TakeError),
    SpreadOutOfBounds,
    GameOver,
}

impl Display for PlayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Self::StackError(stack_error) = self {
            stack_error.fmt(f)
        } else if let Self::TakeError(take_error) = self {
            take_error.fmt(f)
        } else {
            match self {
                Self::OutOfBounds => "given square is not on the board",
                Self::AlreadyOccupied => {
                    "cannot place a piece in that position because it is already occupied"
                }
                Self::NoCapstone => "there is not a capstone left to play",
                Self::NoStones => "there are no more stones left to play",
                Self::OpeningNonFlat => "cannot play a wall or capstone on the first two plies",
                Self::EmptySquare => "cannot move from an empty square",
                Self::StackNotOwned => "cannot move a stack that you do not own",
                Self::SpreadOutOfBounds => "spread would leave the board",
                Self::GameOver => "cannot play a move after the game is over",
                Self::StackError(_) | Self::TakeError(_) => unreachable!(),
            }
            .fmt(f)
        }
    }
}

impl Error for PlayError {}

impl From<TakeError> for PlayError {
    fn from(e: TakeError) -> Self {
        Self::TakeError(e)
    }
}

impl From<StackError> for PlayError {
    fn from(e: StackError) -> Self {
        Self::StackError(e)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum StackError {
    Wall,
    Cap,
}

impl Display for StackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cap => "cannot stack on top of a capstone",
            Self::Wall => "can only flatten a wall with a capstone",
        }
        .fmt(f)
    }
}

impl Error for StackError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TakeError {
    Zero,
    CarryLimit,
    StackSize,
}

impl Display for TakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Zero => "cannot take 0 from a stack",
            Self::CarryLimit => "cannot take more than the carry limit",
            Self::StackSize => "cannot take more pieces than there are on the stack",
        }
        .fmt(f)
    }
}

impl Error for TakeError {}
