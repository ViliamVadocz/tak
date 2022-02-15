use tak::{
    board::Board,
    game::Game,
    pos::Pos,
    ptn::{FromPTN, ToPTN},
    turn::Turn,
    StrResult,
};

const PLIES: &[&str] = &[
    "a6", "f6", "Cd4", "Cc4", "Sd3", "Sc3", "d5", "c5", "d5<", "c4+", "d5", "Se5", "b5", "2c5>11*", "2d5<11",
    "a5", "b4", "a5>", "b4+", "b4", "3b5-21", "2e5<", "d4-*", "d4", "e4", "c4", "e4<", "c4>", "2d3+",
    "2d5<11", "5d4-221", "3b4>111", "2d3+11", "3c5>", "f1", "3a4-12", "5b6>32", "4c1<112",
];

#[test]
fn ptn_consistency() -> StrResult<()> {
    for ply in PLIES {
        let turn = Turn::<6>::from_ptn(ply)?;
        assert_eq!(turn, Turn::from_ptn(&turn.to_ptn())?);
    }
    Ok(())
}

#[test]
fn move_gen_ptn_consistency() -> StrResult<()> {
    let game = Game::<6>::from_ptn(
        "1. c4 d4
        2. d3 Sc3
        3. d3+ d3
        4. 2d4<11 b5
        5. 2c4< b5-
        6. c4 b3
        7. Sd4 e4
        8. e3 e5
        9. d5 e5<
        10. Cc5 2d5>
        11. d5 2e5<
        12. c5> b5
        13. b6 c5
        14. b6- c5<
        15. c4< 3b5-
        16. b2 c2
        17. d2 e2
        18. d2+ e2+
        19. 2d3> Ce2",
    )?;

    for turn in game.possible_turns() {
        if matches!(turn, Turn::Move { .. }) {
            println!("{} {turn:?}", turn.to_ptn());
        }
        // consistent for white moves
        assert_eq!(turn, Turn::from_ptn(&turn.to_ptn())?);
        let mut g = game.clone();
        g.play(turn)?;
        for turn in g.possible_turns() {
            // consistent for black moves
            assert_eq!(turn, Turn::from_ptn(&turn.to_ptn())?);
        }
    }
    Ok(())
}

#[test]
fn empty_board_tps() {
    let board = Board::<6>::default();
    assert_eq!("x6/x6/x6/x6/x6/x6", board.to_ptn());
}

#[test]
fn no_stacks_tps() {
    let game = Game::<4>::from_ptn(
        "1. a4 c3
    2. b2 d1
    3. Sc2 Sb3",
    )
    .unwrap();
    assert_eq!("2,x3/x,2S,1,x/x,1,1S,x/x3,2", game.board.to_ptn());
}

#[test]
fn with_stacks_tps() {
    let game = Game::<5>::from_ptn(
        "1. a1 e1
        2. b1 a2
        3. b1< a2-
        4. Ca2 Se2
        5. a2- d1
        6. a2  d1>
        7. a3  e2-",
    )
    .unwrap();
    assert_eq!("x5/x5/1,x4/1,x4/2121C,x3,122S", game.board.to_ptn());
}

#[test]
fn tps_consistency() -> StrResult<()> {
    let mut game = Game::<5>::default();
    for _ in 0..100 {
        game.nth_move(9576890767)?; // some 10 digit prime to seed pseudo-random moves

        let copy = Board::from_ptn(&game.board.to_ptn())?;
        println!("board\n{}copy\n{}", game.board, copy);
        for y in 0..5 {
            for x in 0..5 {
                let pos = Pos { x, y };
                assert_eq!(game.board[pos], copy[pos]);
            }
        }
    }
    Ok(())
}
