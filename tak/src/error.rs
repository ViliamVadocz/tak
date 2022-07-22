use std::{error::Error, fmt::Display};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
}

impl Display for PlayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use PlayError::*;
        if let StackError(stack_error) = self {
            stack_error.fmt(f)
        } else if let TakeError(take_error) = self {
            take_error.fmt(f)
        } else {
            write!(f, "{}", match self {
                OutOfBounds => "given square is not on the board",
                AlreadyOccupied => "cannot place a piece in that position because it is already occupied",
                NoCapstone => "there is not a capstone left to play",
                NoStones => "there are no more stones left to play",
                OpeningNonFlat => "cannot play a wall or capstone on the first two plies",
                EmptySquare => "cannot move from an empty square",
                StackNotOwned => "cannot move a stack that you do not own",
                SpreadOutOfBounds => "spread would leave the board",
                _ => unreachable!(),
            })
        }
    }
}

impl Error for PlayError {}

impl From<TakeError> for PlayError {
    fn from(e: TakeError) -> Self {
        PlayError::TakeError(e)
    }
}

impl From<StackError> for PlayError {
    fn from(e: StackError) -> Self {
        PlayError::StackError(e)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StackError {
    Wall,
    Cap,
}

impl Display for StackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            StackError::Wall => "can only flatten a wall with a capstone",
            StackError::Cap => "cannot create a stack on top of a capstone",
        })
    }
}

impl Error for StackError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TakeError {
    Zero,
    CarryLimit,
    StackSize(usize),
}

impl Display for TakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TakeError::Zero => write!(f, "cannot take 0 from a tile"),
            TakeError::CarryLimit => write!(f, "cannot take more than the carry limit"),
            TakeError::StackSize(stack) => {
                write!(
                    f,
                    "cannot take more pieces than there are on the tile (stack size: {stack})"
                )
            }
        }
    }
}
