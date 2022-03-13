use tak::ToPTN;

#[derive(Default, Debug, Clone)]
pub struct MoveInfo {
    pub eval: f32,
    pub policy: f32,
    pub visits: u32,
}

impl ToPTN for MoveInfo {
    fn to_ptn(&self) -> String {
        format!("e: {:.4}, p: {:.4}, v: {}", self.eval, self.policy, self.visits)
    }
}
