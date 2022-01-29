use std::cmp::min;

use arrayvec::ArrayVec;

use crate::{
    game::Game,
    pos::{Direction, Pos},
    tile::{Piece, Shape, Tile},
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Turn<const N: usize> {
    Place {
        pos: Pos<N>,
        shape: Shape,
    },
    Move {
        pos: Pos<N>,
        direction: Direction,
        moves: ArrayVec<bool, N>, // true means we moved one step for the next drop
    },
}

impl<const N: usize> Game<N> {
    fn add_moves(&self, turns: &mut Vec<Turn<N>>, pos: Pos<N>, tile: &Tile) {
        for neighbour in pos.neighbors() {
            let direction = (neighbour - pos).unwrap();
            let max_carry = min(tile.size(), N);
            for drop_choices in 0..max_carry {
                let capstone = matches!(tile.top.shape, Shape::Capstone);
                let mut tries = vec![(neighbour, drop_choices, ArrayVec::new())];
                let mut possible_moves = Vec::new();
                while let Some((current, drop_choices, mut moves)) = tries.pop() {
                    #[rustfmt::skip]
                    let can_drop = match self.board[current] {
                        None => true,
                        Some(Tile {top: Piece {shape: Shape::Flat, ..}, ..}) => true,
                        Some(Tile {top: Piece {shape: Shape::Wall, ..}, ..})
                            if drop_choices == 0 && capstone => true,
                        _ => false,
                    };

                    if !can_drop {
                        continue;
                    }
                    if drop_choices == 0 {
                        moves.push(false);
                        possible_moves.push(moves);
                        continue;
                    }

                    if let Some(next) = current.step(direction) {
                        let mut copy = moves.clone();
                        copy.push(true);
                        tries.push((next, drop_choices - 1, copy));
                    }
                    moves.push(false);
                    tries.push((current, drop_choices - 1, moves));
                }

                turns.extend(possible_moves.into_iter().map(|moves| Turn::Move {
                    pos,
                    direction,
                    moves,
                }));
            }
        }
    }

    fn add_places(&self, turns: &mut Vec<Turn<N>>, pos: Pos<N>) {
        let (stones, caps) = self.get_counts();
        if stones > 0 {
            turns.push(Turn::Place {
                pos,
                shape: Shape::Flat,
            });
            turns.push(Turn::Place {
                pos,
                shape: Shape::Wall,
            });
        }
        if caps > 0 {
            turns.push(Turn::Place {
                pos,
                shape: Shape::Capstone,
            });
        }
    }

    pub fn move_gen(&self) -> Vec<Turn<N>> {
        let mut turns = Vec::new();

        // can only place opponent's flat on the first two plies
        if self.swap() {
            for pos in (0..N).flat_map(|x| (0..N).map(move |y| Pos { x, y })) {
                if self.board[pos].is_none() {
                    turns.push(Turn::Place {
                        pos,
                        shape: Shape::Flat,
                    });
                }
            }
            return turns;
        }

        for pos in (0..N).flat_map(|x| (0..N).map(move |y| Pos { x, y })) {
            if let Some(tile) = &self.board[pos] {
                if tile.top.colour == self.to_move {
                    self.add_moves(&mut turns, pos, tile);
                }
            } else {
                self.add_places(&mut turns, pos);
            }
        }
        turns
    }
}
