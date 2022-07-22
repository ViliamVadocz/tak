use tak::*;
use tch::{kind::FLOAT_CPU, Tensor};

#[allow(unused_imports)]
use crate::repr::{
    board::{board_channels, board_repr},
    game::game_repr,
};

#[test]
fn empty_board() {
    let board = Board::<5>::default();
    assert_eq!(
        board_repr(&board, Color::White),
        Tensor::zeros(&[board_channels(5) as i64, 5, 5], FLOAT_CPU)
    );
}

#[test]
fn complicated_board() {
    let game = Game::<5>::from_ptn_moves(&[
        "e3", "e2", "d2", "Sd3", "d4", "c4", "Cb3", "Cb4", "c3", "c2", "c3-", "c3", "b3>", "b3", "a3", "b2",
        "a3>", "a3", "a1", "a3>", "Sb1", "a2", "Se5", "a3", "b1<", "a3-", "2a1+", "a4", "c5", "b5", "d5",
        "b5>", "Sb1", "b5", "b1>", "b5>", "d5<", "d1", "c1<", "c1", "b1<", "d1<", "a1>", "d1", "b1>",
    ])
    .unwrap();
    let board = game.board;
    let (x, o) = (true, false);
    #[rustfmt::skip]
        let handmade = Tensor::cat(&[
            Tensor::of_slice(&[
                // my flats
                o, o, o, o, o,
                o, o, x, x, x,
                o, o, o, o, o,
                o, o, o, x, o,
                o, o, x, o, o,
                // en flats
                o, o, o, x, o,
                o, x, o, o, o,
                o, x, o, o, x,
                x, o, x, o, o,
                o, o, o, o, o,
                // my walls
                o, o, x, o, o,
                x, o, o, o, o,
                o, o, o, o, o,
                o, o, o, o, o,
                o, o, o, o, x,
                // en walls
                o, o, o, o, o,
                o, o, o, o, o,
                o, o, o, x, o,
                o, o, o, o, o,
                o, o, o, o, o,
                // my caps
                o, o, o, o, o,
                o, o, o, o, o,
                o, o, x, o, o,
                o, o, o, o, o,
                o, o, o, o, o,
                // en caps
                o, o, o, o, o,
                o, o, o, o, o,
                o, o, o, o, o,
                o, x, o, o, o,
                o, o, o, o, o,
                // my second layer
                o, o, o, o, o,
                x, o, o, o, o,
                o, x, o, o, o,
                o, o, o, o, o,
                o, o, o, o, o,
                // en second layer
                o, o, x, o, o,
                o, o, x, o, o,
                o, o, x, o, o,
                o, o, o, o, o,
                o, o, x, o, o,
                // my third layer
                o, o, o, o, o,
                o, o, o, o, o,
                o, o, o, o, o,
                o, o, o, o, o,
                o, o, o, o, o,
                // en third layer
                o, o, x, o, o,
                x, o, o, o, o,
                o, x, o, o, o,
                o, o, o, o, o,
                o, o, x, o, o,
                // my fourth layer
                o, o, o, o, o,
                o, o, o, o, o,
                o, o, o, o, o,
                o, o, o, o, o,
                o, o, x, o, o,
                // en fourth layer
                o, o, o, o, o,
                x, o, o, o, o,
                o, o, o, o, o,
                o, o, o, o, o,
                o, o, o, o, o,
            ]).view([12, 5, 5]),
            Tensor::zeros(&[board_channels(5) as  i64 - 12, 5, 5], FLOAT_CPU)
        ], 0);

    let a: Vec<f32> = handmade.into();
    let b: Vec<f32> = board_repr(&board, Color::White).into();
    assert_eq!(a, b);
}

// #[bench]
// fn game_repr_bench(b: &mut Bencher) {
// let game = Game::<5>::from_ptn_moves(&[
// "b3", "c3", "Cc4", "Sd3", "c3<", "c3", "c4-", "Cb4", "b2", "c2", "d2", "a2",
// "Sb1", "a1", "b1+", "a1+", "2b2<", "b2", "a5", "b5", "c5", "d5", "Se5", "a4",
// "a5>", "d5<", "2b5>", "c4", "c1", "d1", "c1+", "c1", "d4", "b1", "Sa1", "a3",
// "a1>", "a3>", "2b1>", "e3", "e2", ])
// .unwrap();
//
// b.iter(|| game_repr(&game));
// }
