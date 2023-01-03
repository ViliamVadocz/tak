use takparse::{Direction, Move, MoveKind, Piece, Square};

use crate::{game::Game, reserves::Reserves, stack::Stack, GameResult};

impl<const N: usize, const HALF_KOMI: i8> Game<N, HALF_KOMI> {
    /// Populate moves vector with all possible moves for the current position.
    ///
    /// # Panics
    ///
    /// If the move vector is not empty.
    pub fn possible_moves(&self, moves: &mut Vec<Move>) {
        assert!(moves.is_empty());
        let n = N as u8;
        // On the first two plies the only possible moves are placing a flat.
        if self.is_swapped() {
            self.add_opening_moves(moves);
            return;
        }

        // Go over every start position and add the possible moves.
        for x in 0..n {
            for y in 0..n {
                let square = Square::new(x, y);
                match self.board.get(square).and_then(Stack::top) {
                    Some((piece, color)) => {
                        if color == self.to_move {
                            self.add_spreads(square, piece, moves);
                        }
                    }
                    None => self.add_places(square, moves),
                }
            }
        }
    }

    fn add_opening_moves(&self, moves: &mut Vec<Move>) {
        let n = N as u8;
        for x in 0..n {
            for y in 0..n {
                let square = Square::new(x, y);
                let Some(stack) = self.board.get(square) else { continue; };
                if stack.is_empty() {
                    moves.push(Move::new(square, MoveKind::Place(Piece::Flat)));
                }
            }
        }
    }

    fn add_places(&self, square: Square, moves: &mut Vec<Move>) {
        let Reserves { stones, caps } = self.get_reserves();
        if stones > 0 {
            moves.push(Move::new(square, MoveKind::Place(Piece::Flat)));
            moves.push(Move::new(square, MoveKind::Place(Piece::Wall)));
        }
        if caps > 0 {
            moves.push(Move::new(square, MoveKind::Place(Piece::Cap)));
        }
    }

    fn add_spreads(&self, square: Square, piece: Piece, moves: &mut Vec<Move>) {
        // Struct to store unfinished spread moves.
        struct Spread<const N: usize> {
            square: Square,
            hand: u8,
            drops: [u8; N],
            drop_counts: usize,
        }

        let n = N as u8;

        let Some(stack) = self.board.get(square) else {return;};
        let max_carry: u8 = stack.size().min(N as u32) as u8;

        let mut spreads = Vec::new();
        for pickup in 1..=max_carry {
            for direction in [
                Direction::Up,
                Direction::Down,
                Direction::Left,
                Direction::Right,
            ] {
                assert!(spreads.is_empty());
                // Start by picking up `pickup` amount.
                spreads.push(Spread {
                    square,
                    hand: pickup,
                    drops: [0; N],
                    drop_counts: 0,
                });
                while let Some(mut spread) = spreads.pop() {
                    if let Some(next) = spread.square.checked_step(direction, n) {
                        // check if it is possible to drop on the next square
                        if !match self.board.get(next).and_then(Stack::top) {
                            None => true,
                            Some((Piece::Flat, _color)) => true,
                            Some((Piece::Cap, _color)) => false,
                            Some((Piece::Wall, _color)) => spread.hand == 1 && piece == Piece::Cap,
                        } {
                            continue;
                        }

                        for drop in 1..spread.hand {
                            let mut drops = spread.drops;
                            drops[spread.drop_counts] = drop;
                            spreads.push(Spread {
                                square: next,
                                hand: spread.hand - drop,
                                drops,
                                drop_counts: spread.drop_counts + 1,
                            });
                        }

                        // Drop the rest
                        spread.drops[spread.drop_counts] = spread.hand;
                        moves.push(Move::new(
                            square,
                            MoveKind::Spread(
                                direction,
                                spread
                                    .drops
                                    .into_iter()
                                    .take(spread.drop_counts + 1)
                                    .map(u32::from)
                                    .collect(),
                            ),
                        ));
                    }
                }
            }
        }
    }
}

pub fn perf_count<const N: usize, const HALF_KOMI: i8>(
    game: Game<N, HALF_KOMI>,
    depth: usize,
) -> usize {
    if depth == 0 || game.result() != GameResult::Ongoing {
        1
    } else if depth == 1 {
        let mut moves = Vec::new();
        game.possible_moves(&mut moves);
        moves.len()
    } else {
        let mut moves = Vec::new();
        game.possible_moves(&mut moves);
        moves
            .into_iter()
            .map(|m| {
                let mut clone = game;
                if clone.play(m).is_err() {
                    return 0;
                };
                perf_count(clone, depth - 1)
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use crate::{move_gen::perf_count, Game};

    #[test]
    fn move_stack_perft() {
        let game = Game::<5, 0>::from_ptn_moves(&["d3", "c3", "c4", "1d3<", "1c4-", "Sc4"]);
        assert_eq!(perf_count(game, 1), 87);
        assert_eq!(perf_count(game, 2), 6_155);
        assert_eq!(perf_count(game, 3), 461_800);
    }

    #[test]
    fn respect_carry_limit_perft() {
        let game = Game::<5, 0>::from_ptn_moves(&[
            "c2", "c3", "d3", "b3", "c4", "1c2+", "1d3<", "1b3>", "1c4-", "Cc2", "a1", "1c2+", "a2",
        ]);
        assert_eq!(perf_count(game, 1), 104);
        assert_eq!(perf_count(game, 2), 7_743);
        assert_eq!(perf_count(game, 3), 592_645);
    }

    #[test]
    fn suicide_perft() {
        let game = Game::<5, 0>::from_ptn_moves(&[
            "c4", "c2", "d2", "c3", "b2", "d3", "1d2+", "b3", "d2", "b4", "1c2+", "1b3>", "2d3<",
            "1c4-", "d4", "5c3<23", "c2", "c4", "1d4<", "d3", "1d2+", "1c3+", "Cc3", "2c4>",
            "1c3<", "d2", "c3", "1d2+", "1c3+", "1b4>", "2b3>11", "3c4-12", "d2", "c4", "b4", "c5",
            "1b3>", "1c4<", "3c3-", "e5", "e2",
        ]);
        assert_eq!(perf_count(game, 1), 85);
        assert_eq!(perf_count(game, 2), 11_206);
        assert_eq!(perf_count(game, 3), 957_000);
    }

    #[test]
    fn endgame_perft() {
        let game = Game::<5, 0>::from_ptn_moves(&[
            "a5", "b4", "c3", "d2", "e1", "d1", "c2", "d3", "c1", "d4", "d5", "c4", "c5", "b3",
            "b2", "a2", "Sb1", "a3", "Ce4", "Cb5", "a4", "a1", "e5", "e3", "c3<", "Sc3", "c1>",
            "c1", "2d1+", "c3-", "c3", "a3>", "a3", "d1", "e4<", "2c2>", "c2", "e2", "b2+", "b2",
        ]);
        assert_eq!(perf_count(game, 1), 65);
        assert_eq!(perf_count(game, 2), 4_072);
        assert_eq!(perf_count(game, 3), 272_031);
        assert_eq!(perf_count(game, 4), 16_642_760);
    }

    #[test]
    fn reserves_perft() {
        let game = Game::<5, 0>::from_ptn_moves(&[
            "a1", "b1", "c1", "d1", "e1", "e2", "d2", "c2", "b2", "a2", "a3", "b3", "c3", "d3",
            "e3", "a4", "b4", "c4", "d4", "e4", "a5", "a4-", "b4-", "c4-", "d4-", "e4-", "a4",
            "b4", "c4", "d4", "e4", "2a3>", "c4>", "2e3<", "a3", "4b3-", "b3", "c4", "e3", "d5",
            "d2<", "d2", "2d4-", "d4", "c5", "b5", "2c2>", "d1+", "c2", "e2+", "d1", "e2", "c5<",
            "c5", "e4<", "Se4", "2b5-", "e4-", "a3-",
        ]);
        assert_eq!(perf_count(game, 1), 152);
        assert_eq!(perf_count(game, 2), 15_356);
        assert_eq!(perf_count(game, 3), 1_961_479);
        // assert_eq!(perf_count(&game, 4), 197_434_816);
    }

    #[test]
    fn perft_5() {
        assert_eq!(perf_count(Game::<5, 0>::default(), 0), 1);
        assert_eq!(perf_count(Game::<5, 0>::default(), 1), 25);
        assert_eq!(perf_count(Game::<5, 0>::default(), 2), 600);
        assert_eq!(perf_count(Game::<5, 0>::default(), 3), 43_320);
        assert_eq!(perf_count(Game::<5, 0>::default(), 4), 2_999_784);
    }

    #[test]
    fn perft_6() {
        assert_eq!(perf_count(Game::<6, 0>::default(), 0), 1);
        assert_eq!(perf_count(Game::<6, 0>::default(), 1), 36);
        assert_eq!(perf_count(Game::<6, 0>::default(), 2), 1_260);
        assert_eq!(perf_count(Game::<6, 0>::default(), 3), 132_720);
        assert_eq!(perf_count(Game::<6, 0>::default(), 4), 13_586_048);
        // assert_eq!(perf_count(&Game::<6>::default(), 5), 1_253_506_520);
    }
}
