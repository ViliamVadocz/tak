use tak::*;

#[derive(Clone, Debug, Default)]
pub struct Node {
    pub policy: f32,
    pub expected_reward: f32,
    pub result: GameResult,
    pub visits: u32,
    pub virtual_visits: u32,
    pub children: Vec<(Move, Node)>,
}

impl Node {
    pub fn new(policy: f32) -> Self {
        Node {
            policy,
            ..Default::default()
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.visits != 0 || self.virtual_visits != 0
    }

    pub fn visit_count(&self) -> f32 {
        (self.visits + self.virtual_visits) as f32
    }
}
