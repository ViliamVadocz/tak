#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    PosX,
    PosY,
    NegX,
    NegY,
}

impl Direction {
    /// rotates a direction 1 quarter turn counterclockwise
    #[must_use]
    pub const fn rotate(&self) -> Self {
        match self {
            Direction::PosX => Direction::NegY,
            Direction::PosY => Direction::PosX,
            Direction::NegX => Direction::PosY,
            Direction::NegY => Direction::NegX,
        }
    }

    /// mirror along the x axis
    #[must_use]
    pub const fn mirror(&self) -> Self {
        match self {
            Direction::PosX => Direction::PosX,
            Direction::PosY => Direction::NegY,
            Direction::NegX => Direction::NegX,
            Direction::NegY => Direction::PosY,
        }
    }
}
