use tak::*;
use takparse::Tps;

#[test]
fn complicated_board() {
    let game = Game::<6>::from_ptn_moves(&[
        "e1", "f2", "Sb5", "Cd6", "d3", "d4", "Sc1", "c3", "Ca6", "f6", "b1", "Sb4", "b3", "b2", "d5", "e1>",
        "d3>", "b2<", "Se2", "f4", "f2-", "c3-", "e4", "Sa5", "c3", "c5", "b5>", "a2-", "Sb5", "e6",
        "2c5-11", "d6>", "d5<", "b2", "b3-", "b3", "e3+", "e6>", "a4", "Sf5", "d6", "e6-", "f1+", "d4<",
        "d3", "d4", "b2>", "e3", "2e4+11", "a1>", "2c3>11", "Sc6", "d3-", "e4", "d5", "a2", "d5-", "a2+",
        "2c2+11", "c2", "d1", "c3>", "3c4-", "2d3-11", "Sa2", "c4", "2d2<11", "Sd2", "d3", "b3-", "f2+",
        "b3", "a1", "e4+", "d5", "2e5<11", "2d4>", "2b2>", "d5-", "d2+", "e4+", "d2", "c3<", "c3<", "e2<",
        "c2+", "c2<", "e2", "d5>", "c3<", "b2>", "d5", "d4>", "d5+", "c2<", "d5", "b2-", "d5>", "c2+", "b3>",
        "2d2<", "d2", "3c2+21", "d4", "e4<", "d5", "c2",
    ])
    .unwrap();

    let tps: Tps = game.into();
    assert_eq!(
        tps.to_string(),
        "1C,x,2S,12,1,22C/2S,1S,12,2,2112,2S/1,2S,21S,21,2,2/2,212,21222,12S,21S,1/1S,2,1,2,2,x/1,121,1S,12,\
         x,2 2 54"
    )
}

fn tps_consistency(seed: usize) -> Result<(), PlayError> {
    let mut game = Game::<5>::default();
    while game.result() == GameResult::Ongoing {
        let moves = game.possible_moves();
        let count = moves.len();
        let my_move = moves.into_iter().nth(seed % count).unwrap();

        game.play(my_move)?;

        println!("{}", game.ply);
        let tps: Tps = game.clone().into();
        let tps_game: Game<5> = tps.into();

        assert_eq!(game.board, tps_game.board, "board does not equal");
        assert_eq!(game.to_move, tps_game.to_move, "to_move does not equal");
        assert_eq!(game.ply, tps_game.ply, "ply does not equal");
        assert_eq!(game.white_caps, tps_game.white_caps, "white caps do not equal");
        assert_eq!(
            game.white_stones, tps_game.white_stones,
            "white stones do not equal"
        );
        assert_eq!(game.black_caps, tps_game.black_caps, "black caps do not equal");
        assert_eq!(
            game.black_stones, tps_game.black_stones,
            "black stones do not equal"
        );
    }

    Ok(())
}

#[test]
fn tps_consistency_5915587277() -> Result<(), PlayError> {
    tps_consistency(5915587277)
}
#[test]
fn tps_consistency_1500450271() -> Result<(), PlayError> {
    tps_consistency(1500450271)
}
#[test]
fn tps_consistency_3267000013() -> Result<(), PlayError> {
    tps_consistency(3267000013)
}
#[test]
fn tps_consistency_5754853343() -> Result<(), PlayError> {
    tps_consistency(5754853343)
}
#[test]
fn tps_consistency_4093082899() -> Result<(), PlayError> {
    tps_consistency(4093082899)
}
#[test]
fn tps_consistency_9576890767() -> Result<(), PlayError> {
    tps_consistency(9576890767)
}
#[test]
fn tps_consistency_3628273133() -> Result<(), PlayError> {
    tps_consistency(3628273133)
}
#[test]
fn tps_consistency_2860486313() -> Result<(), PlayError> {
    tps_consistency(2860486313)
}
#[test]
fn tps_consistency_5463458053() -> Result<(), PlayError> {
    tps_consistency(5463458053)
}
#[test]
fn tps_consistency_3367900313() -> Result<(), PlayError> {
    tps_consistency(3367900313)
}
