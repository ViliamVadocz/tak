use std::{collections::HashMap, iter::repeat};

use tak::{
    board::Board,
    colour::Colour,
    game::Game,
    pos::Pos,
    tile::{Piece, Shape, Tile},
    turn::Turn,
};

lazy_static! {
    static ref LUT_3: HashMap<Turn<3>, usize> = generate_turn_map::<3>();
    static ref LUT_4: HashMap<Turn<4>, usize> = generate_turn_map::<4>();
    static ref LUT_5: HashMap<Turn<5>, usize> = generate_turn_map::<5>();
    static ref LUT_6: HashMap<Turn<6>, usize> = generate_turn_map::<6>();
    static ref LUT_7: HashMap<Turn<7>, usize> = generate_turn_map::<7>();
    static ref LUT_8: HashMap<Turn<8>, usize> = generate_turn_map::<8>();
}

fn generate_turn_map<const N: usize>() -> HashMap<Turn<N>, usize>
where
    [[Option<Tile>; N]; N]: Default,
{
    let mut map = HashMap::new();
    // create empty game and add all place moves
    let game = Game {
        ply: 4, // to bypass first move swap
        to_move: Colour::White,
        ..Default::default()
    };
    let mut i = 0;
    for turn in game.move_gen() {
        assert!(matches!(turn, Turn::Place { .. }));
        map.insert(turn, i);
        i += 1;
    }

    // create a board where all the spots are filled
    // with tall stacks controlled by white
    let mut board = Board::default();
    for y in 0..N {
        for x in 0..N {
            let pos = Pos { x, y };
            board[pos] = Some(Tile {
                top: Piece {
                    colour: Colour::White,
                    shape: Shape::Flat,
                },
                stack: Some(repeat(Colour::White).take(N).collect()),
            });
        }
    }
    let game = Game {
        board,
        ply: 4, // to bypass first move swap
        to_move: Colour::White,
        ..Default::default()
    };

    for turn in game.move_gen() {
        assert!(matches!(turn, Turn::Move { .. }));
        map.insert(turn, i);
        i += 1;
    }
    map
}

fn to_white(piece: &Piece) -> Piece {
    Piece {
        shape: piece.shape,
        colour: Colour::White,
    }
}

pub trait LUT {
    fn turn_map(&self) -> usize;
}

macro_rules! impl_lut {
    ($n:literal, $lut:ident) => {
        impl LUT for Turn<$n> {
            fn turn_map(&self) -> usize {
                let key = match self {
                    Turn::Place { pos, piece } => Turn::Place {
                        pos: *pos,
                        piece: to_white(piece),
                    },
                    Turn::Move { pos, drops } => Turn::Move {
                        pos: *pos,
                        // all moves with the same drop distributions map
                        // to the same thing regardless of the piece types they drop
                        drops: drops
                            .clone()
                            .into_iter()
                            .map(|(pos, _piece)| {
                                (pos, Piece {
                                    shape: Shape::Flat,
                                    colour: Colour::White,
                                })
                            })
                            .collect(),
                    },
                };
                *$lut.get(&key).expect(&format!(
                    "could not map turn to index. {:?}, {:?}",
                    self, key
                ))
            }
        }
    };
}

impl_lut!(3, LUT_3);
impl_lut!(4, LUT_4);
impl_lut!(5, LUT_5);
impl_lut!(6, LUT_6);
impl_lut!(7, LUT_7);
impl_lut!(8, LUT_8);
