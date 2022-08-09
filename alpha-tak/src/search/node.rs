use tak::*;

#[derive(Clone, Debug, Default)]
pub struct Node {
    pub policy: f32,
    pub expected_reward: f32,
    pub result: GameResult,
    pub visits: u32,
    pub virtual_visits: u32,
    pub children: Box<[(Move, Node)]>,
}

impl Node {
    pub fn new(policy: f32) -> Self {
        Node {
            policy,
            ..Default::default()
        }
    }

    /// Check whether this node has been visited at least once
    /// and that the children are initialized.
    pub fn is_initialized(&self) -> bool {
        self.visits != 0 || self.virtual_visits != 0
    }

    /// Get the visit count of this node, including virtual visits.
    pub fn visit_count(&self) -> f32 {
        (self.visits + self.virtual_visits) as f32
    }

    /// Get the expected reward, accounting for virtual losses.
    pub fn expected_reward_with_losses(&self) -> f32 {
        if !self.is_initialized() {
            return 0.0;
        }
        (self.expected_reward * self.visits as f32 - self.virtual_visits as f32) / self.visit_count()
    }
}
