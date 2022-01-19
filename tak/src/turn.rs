use std::cmp::min;

use arrayvec::ArrayVec;
use regex::Regex;

use crate::{
    board::Board,
    colour::Colour,
    game::Game,
    pos::{Direction, Pos},
    tile::{Piece, Shape, Tile},
    StrResult,
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
        drops: ArrayVec<(Pos<N>, Piece), N>,
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
            if self.ply >= 2 {
                turns.push(Turn::Place {
                    pos,
                    piece: Piece {
                        colour: self.to_move,
                        shape: Shape::Wall,
                    },
                });
            }
        }
        if caps > 0 && self.ply >= 2 {
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

fn abc_to_num(c: char) -> usize {
    (c as u32 - 'a' as u32) as usize
}

impl<const N: usize> Turn<N> {
    #[allow(non_snake_case)]
    pub fn from_PTN(ply: &str, board: &Board<N>, colour: Colour) -> StrResult<Turn<N>> {
        assert!(N < 10);
        // (count)(square)(direction)(drop counts)(stone)
        let re = Regex::new(r"([0-9]*)([a-z])([0-9])([<>+-])([0-9]*)[CS]?").unwrap();
        if let Some(cap) = re.captures(ply) {
            let carry_amount = cap[1].parse().unwrap_or(1);

            let x = abc_to_num(cap[2].chars().next().unwrap());
            let y = cap[3].parse::<usize>().unwrap() - 1;
            let direction = match &cap[4] {
                "<" => Direction::NegX,
                ">" => Direction::PosX,
                "+" => Direction::PosY,
                "-" => Direction::NegY,
                _ => unreachable!(),
            };

            let mut drop_counts: Vec<_> = cap[5].chars().map(|c| c.to_digit(10).unwrap()).collect();
            if drop_counts.is_empty() {
                drop_counts = vec![carry_amount];
            }

            let mut pos = Pos { x, y };
            let tile = board[pos].clone().ok_or("there is not stack on that position")?;
            let (_left, mut carry) = tile.take::<N>(carry_amount as usize)?;

            let mut drops = ArrayVec::new();
            for i in drop_counts {
                pos = pos.step(direction).ok_or("move would go off the board")?;
                for _ in 0..i {
                    drops.push((
                        pos,
                        carry
                            .pop()
                            .ok_or("not enough pieces picked up to satisfy move ply")?,
                    ))
                }
            }

            Ok(Turn::Move {
                pos: Pos { x, y },
                drops,
            })
        } else {
            // (stone)(square)
            let re = Regex::new(r"([CS]?)([a-z])([0-9])").unwrap();
            let cap = re.captures(ply).ok_or("didn't recognize place ply")?;
            let shape = match &cap[1] {
                "C" => Shape::Capstone,
                "S" => Shape::Wall,
                "" => Shape::Flat,
                _ => unreachable!(),
            };
            let x = abc_to_num(cap[2].chars().next().unwrap());
            let y = cap[3].parse::<usize>().unwrap() - 1;

            Ok(Turn::Place {
                pos: Pos { x, y },
                piece: Piece { shape, colour },
            })
        }
    }
}
