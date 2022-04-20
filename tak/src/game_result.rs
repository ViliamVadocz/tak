use takparse::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameResult {
    Winner { color: Color, road: bool },
    Draw { turn_limit: bool },
    Ongoing,
}

impl Default for GameResult {
    fn default() -> Self {
        GameResult::Ongoing
    }
}
