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
        piece: Piece,
    },
    Move {
        pos: Pos<N>,
        // at most N drops because of carry limit and you have to drop at least one
        drops: ArrayVec<(Pos<N>, Piece), N>, // TODO simplify
    },
}

impl<const N: usize> Game<N> {
    fn add_moves(&self, turns: &mut Vec<Turn<N>>, pos: Pos<N>, tile: &Tile) {
        for neighbour in pos.neighbors() {
            let direction = (neighbour - pos).unwrap();
            let max_carry = min(tile.size(), N);
            for i in 0..=(max_carry - 1) {
                let mut carry: Vec<_> = tile
                    .stack
                    .iter()
                    .map(|&colour| Piece {
                        colour,
                        shape: Shape::Flat,
                    })
                    .skip(tile.stack.len() - i)
                    .collect();
                carry.push(tile.top);
                let possible_drops = self.try_drop(neighbour, direction, &carry);
                turns.extend(
                    possible_drops
                        .into_iter()
                        .filter(|drops| !drops.is_empty())
                        .map(|drops| Turn::Move { pos, drops }),
                );
            }
        }
    }

    fn add_places(&self, turns: &mut Vec<Turn<N>>, pos: Pos<N>) {
        let (stones, caps) = self.get_counts();
        if stones > 0 {
            turns.push(Turn::Place {
                pos,
                piece: Piece {
                    colour: self.to_move,
                    shape: Shape::Flat,
                },
            });
            turns.push(Turn::Place {
                pos,
                piece: Piece {
                    colour: self.to_move,
                    shape: Shape::Wall,
                },
            });
        }
        if caps > 0 {
            turns.push(Turn::Place {
                pos,
                piece: Piece {
                    colour: self.to_move,
                    shape: Shape::Capstone,
                },
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
                        piece: Piece {
                            colour: self.to_move.next(),
                            shape: Shape::Flat,
                        },
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

    // size of Vec is technically bounded by number of partitions of carry
    // but it's too much effort to try and calculate that
    fn try_drop(
        &self,
        pos: Pos<N>,
        direction: Direction,
        carry: &[Piece],
    ) -> Vec<ArrayVec<(Pos<N>, Piece), N>> {
        let mut all_drops = Vec::new();

        #[rustfmt::skip]
        let can_drop = match self.board[pos] {
            None => true,
            Some(Tile {top: Piece {shape: Shape::Flat, ..}, ..}) => true,
            Some(Tile {top: Piece {shape: Shape::Wall, ..}, ..})
                if carry.len() == 1 && carry[0].shape == Shape::Capstone => true,
            _ => false,
        };

        if can_drop {
            for i in 1..=(carry.len()) {
                let (drops, sub_carry) = carry.split_at(i);
                let here_drops: ArrayVec<_, N> = drops.iter().map(|&piece| (pos, piece)).collect();
                if sub_carry.is_empty() {
                    all_drops.push(here_drops);
                } else if let Some(next) = pos.step(direction) {
                    let possible_drops = self.try_drop(next, direction, sub_carry);
                    debug_assert!(possible_drops.iter().all(|v| v.len() == sub_carry.len()));
                    for possible in possible_drops {
                        let mut clone = here_drops.clone();
                        clone.extend(possible);
                        all_drops.push(clone);
                    }
                }
            }
        }

        all_drops
    }
}
