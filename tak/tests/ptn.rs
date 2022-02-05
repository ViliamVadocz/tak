use tak::{
    ptn::{FromPTN, ToPTN},
    turn::Turn,
    StrResult,
};

const PLIES: &[&str] = &[
    "a6", "f6", "Cd4", "Cc4", "Sd3", "Sc3", "d5", "c5", "d5<", "c4+", "d5", "Se5", "b5", "2c5>11*", "2d5<11",
    "a5", "b4", "a5>", "b4+", "b4", "3b5-21", "2e5<", "d4-*", "d4", "e4", "c4", "e4<", "c4>", "2d3+",
    "2d5<11", "5d4-221", "3b4>111", "2d3+11", "3c5>", "f1", "3a4-12", "5b6>32", "4c1<112"
];

#[test]
fn ptn_consistency() -> StrResult<()> {
    for ply in PLIES {
        let turn = Turn::<6>::from_ptn(ply)?;
        assert_eq!(turn, Turn::from_ptn(&turn.to_ptn())?);
    }
    Ok(())
}
