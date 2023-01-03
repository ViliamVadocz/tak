use std::{cmp::Ordering, ops::Not};

use takparse::Color;

use crate::{game_result::Reason, Game, GameResult};

const MAX_REVERSIBLE_PLIES: u16 = 50;

impl<const N: usize, const HALF_KOMI: i8> Game<N, HALF_KOMI> {
    #[must_use]
    pub fn result(&self) -> GameResult {
        // We check the result after a move, so for the dragon clause
        // we look at the other player's path first (they just played).
        if self.board.has_road(self.to_move.not()) {
            GameResult::Winner {
                color: self.to_move.not(),
                reason: Reason::Road,
            }
        } else if self.board.has_road(self.to_move) {
            GameResult::Winner {
                color: self.to_move,
                reason: Reason::Road,
            }
        } else if self.white_reserves.depleted()
            || self.black_reserves.depleted()
            || self.board.full()
        {
            self.flat_end()
        } else if self.reversible_plies >= MAX_REVERSIBLE_PLIES {
            GameResult::Draw {
                reason: Reason::ReversiblePlies,
            }
        } else {
            GameResult::Ongoing
        }
    }

    fn flat_end(&self) -> GameResult {
        let reason = if self.white_reserves.depleted() || self.black_reserves.depleted() {
            Reason::ReservesDepleted
        } else {
            Reason::BoardFill
        };
        let flat_diff = self.board.flat_diff();
        match flat_diff.cmp(&(HALF_KOMI / 2)) {
            Ordering::Greater => GameResult::Winner {
                color: Color::White,
                reason,
            },
            Ordering::Less => GameResult::Winner {
                color: Color::Black,
                reason,
            },
            Ordering::Equal => {
                if HALF_KOMI % 2 == 0 {
                    GameResult::Draw { reason }
                } else {
                    GameResult::Winner {
                        color: if HALF_KOMI > 0 {
                            Color::Black
                        } else {
                            Color::White
                        },
                        reason,
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use takparse::Color;

    use crate::{game_result::Reason, Game, GameResult};

    #[test]
    fn dragon_clause() {
        let game = Game::<6, 0>::from_ptn_moves(&[
            "a4", "a3", "b3", "b4", "c3", "c4", "d3", "d4", "d3+", "e4", "e3", "f4", "f3", "Cb5",
            "d4-",
        ]);
        assert_eq!(game.result(), GameResult::Winner {
            color: Color::White,
            reason: Reason::Road
        });
    }

    #[test]
    fn flat_win() {
        let game =
            Game::<3, 0>::from_ptn_moves(&["a3", "c1", "c2", "c3", "b3", "b2", "b1", "a1", "a2"]);
        assert_eq!(game.result(), GameResult::Winner {
            color: Color::White,
            reason: Reason::BoardFill
        });
    }

    #[test]
    fn road_win() {
        let game = Game::<5, 0>::from_ptn_moves(&[
            "d2", "a5", "b4", "d3", "Cc3", "Cc2", "b2", "b1", "b3", "a1", "c4", "c1", "e2", "e3",
        ]);
        assert_eq!(game.result(), GameResult::Winner {
            color: Color::Black,
            reason: Reason::Road,
        });
    }

    #[test]
    fn road_beats_flats() {
        let game =
            Game::<3, 0>::from_ptn_moves(&["a1", "c1", "c2", "a2", "Sa3", "b1", "Sb3", "b2", "c3"]);
        assert_eq!(game.result(), GameResult::Winner {
            color: Color::White,
            reason: Reason::Road,
        });
    }

    #[test]
    fn reserves() {
        let game = Game::<3, 0>::from_ptn_moves(&[
            "a3", "b3", "a2", "b2", "b3<", "b2<", "b3", "b2", "c2", "c3", "2a3-", "b2<", "b3>",
            "b3", "b2", "a3", "a1", "b1", "b2<", "c1", "c2<", "b1+", "b1", "2b2+", "b2", "a3-",
            "a3", "3b3<", "b1<", "3a2-", "b1",
        ]);
        assert_eq!(game.result(), GameResult::Winner {
            color: Color::White,
            reason: Reason::ReservesDepleted
        });
    }

    #[test]
    fn board_fill_komi() {
        let moves = [
            "a1", "a2", "b1", "b2", "c2", "c1", "d1", "d2", "d3", "c3", "b3", "a3", "a4", "b4",
            "c4", "d4",
        ];
        let game = Game::<4, 0>::from_ptn_moves(&moves);
        assert_eq!(game.result(), GameResult::Draw {
            reason: Reason::BoardFill
        });
        let game = Game::<4, -1>::from_ptn_moves(&moves);
        assert_eq!(game.result(), GameResult::Winner {
            color: Color::White,
            reason: Reason::BoardFill
        });
        let game = Game::<4, 1>::from_ptn_moves(&moves);
        assert_eq!(game.result(), GameResult::Winner {
            color: Color::Black,
            reason: Reason::BoardFill,
        });
        let game = Game::<4, 2>::from_ptn_moves(&moves);
        assert_eq!(game.result(), GameResult::Winner {
            color: Color::Black,
            reason: Reason::BoardFill
        });
    }
}
