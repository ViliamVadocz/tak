use tak::{
    game::Game,
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
