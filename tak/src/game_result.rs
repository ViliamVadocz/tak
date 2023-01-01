use takparse::Color;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum GameResult {
    Winner {
        color: Color,
        reason: Reason,
    },
    Draw {
        reason: Reason,
    },
    #[default]
    Ongoing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Reason {
    Road,
    ReservesDepleted,
    BoardFill,
    ReversiblePlies,
}
