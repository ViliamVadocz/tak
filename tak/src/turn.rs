use std::cmp::min;

use arrayvec::ArrayVec;

use crate::{
    game::Game,
    pos::{Direction, Pos},
    tile::{Piece, Shape},
};

#[derive(Clone, Debug)]
pub enum Turn<const N: usize> {
    Place {
        pos: Pos,
        piece: Piece,
    },
    Move {
        pos: Pos,
        // at most N drops because of carry limit and you have to drop at least one
        drops: ArrayVec<(Pos, Piece), N>,
    },
}

impl<const N: usize> Game<N> {
    pub fn move_gen(&self) -> Vec<Turn<N>> {
        let (stones, caps) = self.get_counts();

        let mut turns = Vec::new();

        for x in 0..N {
            for y in 0..N {
                let pos = Pos { x, y };
                if let Some(tile) = &self.board[pos] {
                    if tile.top.colour == self.to_move {
                        for neighbour in pos.neighbors::<N>() {
                            let direction = (neighbour - pos).unwrap();
                            let max_carry = min(tile.size(), N);
                            for i in 0..=max_carry {
                                let mut carry = vec![tile.top];
                                if let Some(stack) = &tile.stack {
                                    carry.extend(
                                        stack
                                            .iter()
                                            .map(|&colour| Piece {
                                                colour,
                                                shape: Shape::Flat,
                                            })
                                            .skip(stack.len() - i)
                                            .rev(),
                                    );
                                }
                                let possible_drops = self.try_drop(neighbour, direction, &carry);
                                turns.extend(
                                    possible_drops.into_iter().map(|drops| Turn::Move { pos, drops }),
                                );
                            }
                        }
                    }
                } else {
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
            }
        }

        turns
    }

    // size of Vec is technically bounded by number of partitions of carry
    // but it's too much effort to try and calculate that
    fn try_drop(&self, pos: Pos, direction: Direction, carry: &[Piece]) -> Vec<ArrayVec<(Pos, Piece), N>> {
        let mut all_drops = Vec::new();
        if let Some(next) = pos
            .neighbors::<N>()
            .into_iter()
            .find(|&n| (n - pos).unwrap() == direction)
        {
            for i in 1..=(carry.len()) {
                let (drops, sub_carry) = carry.split_at(i);
                let possible_drops = self.try_drop(next, direction, sub_carry);
                let here_drops: ArrayVec<_, N> = drops.iter().map(|&piece| (pos, piece)).collect();
                for possible_drop in possible_drops {
                    let mut clone = here_drops.clone();
                    clone.extend(possible_drop);
                    all_drops.push(clone);
                }
            }
        }
        all_drops
    }
}
