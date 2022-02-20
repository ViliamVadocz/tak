use tak::{
    board::Board,
    game::Game,
    pos::Pos,
    ptn::FromPTN,
    tps::{FromTPS, ToTPS},
    StrResult,
};

#[test]
fn empty_board_tps() {
    let board = Board::<6>::default();
    assert_eq!("x6/x6/x6/x6/x6/x6", board.to_tps());
}

#[test]
fn no_stacks_tps() {
    let game = Game::<4>::from_ptn(
        "1. a4 c3
    2. b2 d1
    3. Sc2 Sb3",
    )
    .unwrap();
    assert_eq!("2,x3/x,2S,1,x/x,1,1S,x/x3,2", game.board.to_tps());
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
    assert_eq!("x5/x5/1,x4/1,x4/2121C,x3,122S", game.board.to_tps());
}

#[test]
fn tps_consistency() -> StrResult<()> {
    let mut game = Game::<5>::default();
    for _ in 0..100 {
        game.nth_move(9576890767)?; // some 10 digit prime to seed pseudo-random moves

        let copy = Board::from_tps(&game.board.to_tps())?;
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
