use tak::*;

fn symmetrical_boards(seed: usize) -> Result<(), PlayError> {
    let [mut g0, mut g1, mut g2, mut g3, mut g4, mut g5, mut g6, mut g7] = Game::<5>::default().symmetries();
    while matches!(g0.result(), GameResult::Ongoing) {
        let moves = g0.possible_moves();
        let count = moves.len();
        let my_move = moves.into_iter().nth(seed % count).unwrap();
        println!("{:#?}", Symmetry::<5>::symmetries(my_move));
        let [t0, t1, t2, t3, t4, t5, t6, t7] = Symmetry::<5>::symmetries(my_move);
        g0.play(t0)?;
        g1.play(t1)?;
        g2.play(t2)?;
        g3.play(t3)?;
        g4.play(t4)?;
        g5.play(t5)?;
        g6.play(t6)?;
        g7.play(t7)?;
    }
    assert_eq!(g0.result(), g1.result());
    assert_eq!(g1.result(), g2.result());
    assert_eq!(g2.result(), g3.result());
    assert_eq!(g4.result(), g5.result());
    assert_eq!(g5.result(), g6.result());
    assert_eq!(g6.result(), g7.result());
    Ok(())
}

#[test]
fn symmetrical_boards_5915587277() -> Result<(), PlayError> {
    symmetrical_boards(5915587277)
}
#[test]
fn symmetrical_boards_1500450271() -> Result<(), PlayError> {
    symmetrical_boards(1500450271)
}
#[test]
fn symmetrical_boards_3267000013() -> Result<(), PlayError> {
    symmetrical_boards(3267000013)
}
#[test]
fn symmetrical_boards_5754853343() -> Result<(), PlayError> {
    symmetrical_boards(5754853343)
}
#[test]
fn symmetrical_boards_4093082899() -> Result<(), PlayError> {
    symmetrical_boards(4093082899)
}
#[test]
fn symmetrical_boards_9576890767() -> Result<(), PlayError> {
    symmetrical_boards(9576890767)
}
#[test]
fn symmetrical_boards_3628273133() -> Result<(), PlayError> {
    symmetrical_boards(3628273133)
}
#[test]
fn symmetrical_boards_2860486313() -> Result<(), PlayError> {
    symmetrical_boards(2860486313)
}
#[test]
fn symmetrical_boards_5463458053() -> Result<(), PlayError> {
    symmetrical_boards(5463458053)
}
#[test]
fn symmetrical_boards_3367900313() -> Result<(), PlayError> {
    symmetrical_boards(3367900313)
}
