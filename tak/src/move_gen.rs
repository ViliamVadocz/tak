use arrayvec::ArrayVec;
use takparse::{Direction, Move, MoveKind, Piece, Square};

use crate::game::Game;

impl<const N: usize> Game<N> {
    pub fn possible_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        // On the first two plies the only possible moves are placing a flat.
        if self.is_swapped() {
            self.add_opening_moves(&mut moves);
            return moves;
        }

        // Go over every start position and add the possible moves.
        for x in 0..N {
            for y in 0..N {
                let square = Square::new(x as u8, y as u8);
                if let Some((_piece, color)) = self.board[square].top() {
                    if color == self.color() {
                        self.add_spreads(square, &mut moves);
                    }
                } else {
                    self.add_places(square, &mut moves);
                }
            }
        }
        moves
    }

    fn add_opening_moves(&self, moves: &mut Vec<Move>) {
        for x in 0..N {
            for y in 0..N {
                let square = Square::new(x as u8, y as u8);
                if self.board[square].is_empty() {
                    moves.push(Move::new(square, MoveKind::Place(Piece::Flat)));
                }
            }
        }
    }

    fn add_places(&self, square: Square, moves: &mut Vec<Move>) {
        let (stones, caps) = self.get_counts();
        if stones > 0 {
            moves.push(Move::new(square, MoveKind::Place(Piece::Flat)));
            moves.push(Move::new(square, MoveKind::Place(Piece::Wall)));
        }
        if caps > 0 {
            moves.push(Move::new(square, MoveKind::Place(Piece::Cap)));
        }
    }

    fn add_spreads(&self, square: Square, moves: &mut Vec<Move>) {
        struct Spread<const N: usize> {
            square: Square,
            hand: usize,
            drops: ArrayVec<u32, { N }>,
        }

        let tile = &self.board[square];
        let max_carry = std::cmp::min(tile.size(), N);

        for direction in [Direction::Up, Direction::Down, Direction::Left, Direction::Right] {
            for pickup in 1..=max_carry {
                let mut spreads = vec![Spread {
                    square,
                    hand: pickup,
                    drops: ArrayVec::<u32, N>::new(),
                }];
                while let Some(spread) = spreads.pop() {
                    if spread.hand == 0 {
                        moves.push(Move::new(
                            square,
                            MoveKind::Spread(direction, spread.drops.into_iter().collect()),
                        ));
                        continue;
                    }
                    if let Some(next) = spread.square.checked_step(direction, N as u8) {
                        let can_drop = match self.board[next].piece {
                            Piece::Flat => true,
                            Piece::Cap => false,
                            Piece::Wall => spread.hand == 1 && tile.piece == Piece::Cap,
                        };
                        if !can_drop {
                            continue;
                        }

                        for drop in 1..=(spread.hand) {
                            let mut drops = spread.drops.clone();
                            drops.push(drop as u32);
                            spreads.push(Spread {
                                square: next,
                                hand: spread.hand - drop,
                                drops,
                            });
                        }
                    }
                }
            }
        }
    }
}
