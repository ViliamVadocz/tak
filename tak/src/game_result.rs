use takparse::Color;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameResult {
    Winner { color: Color, road: bool },
    Draw { reversible_plies: bool },
    Ongoing,
}

impl Default for GameResult {
    fn default() -> Self {
        GameResult::Ongoing
    }
}
