use std::{collections::VecDeque, fmt::Display};

use tak::*;

use super::node::Node;

impl Node {
    /// Get debug info for this node.
    pub fn debug(&self, depth: usize) -> NodeDebugInfo {
        let mut moves: Vec<_> = self
            .children
            .iter()
            .map(|(mov, node)| MoveInfo {
                mov: *mov,
                visits: node.visits,
                reward: node.expected_reward,
                policy: node.policy,
                continuation: node.continuation(depth),
            })
            .collect();
        moves.sort_unstable_by_key(|info| info.visits);
        moves.reverse();
        NodeDebugInfo(moves)
    }

    pub fn continuation(&self, depth: usize) -> VecDeque<(Move, u32)> {
        if depth == 0 || self.children.is_empty() {
            return VecDeque::new();
        }
        let my_move = self.pick_move(true);
        let (_mov, node) = self.children.iter().find(|(mov, _node)| mov == &my_move).unwrap();
        let mut turns = node.continuation(depth - 1);
        turns.push_front((my_move, node.visits));
        turns
    }
}

/// The inner Vec should always be sorted in descending order of visits.
#[derive(Debug, Clone)]
pub struct NodeDebugInfo(pub(crate) Vec<MoveInfo>);

impl NodeDebugInfo {
    pub fn eval(&self) -> f32 {
        self.0
            .first()
            .map(|move_info| move_info.reward)
            .unwrap_or_default()
    }
}

impl Display for NodeDebugInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            return write!(f, "Node has no children");
        }
        writeln!(f, "turn      visited   reward   policy | continuation")?;
        for move_info in &self.0 {
            move_info.fmt(f)?
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MoveInfo {
    pub mov: Move,
    pub visits: u32,
    pub reward: f32,
    pub policy: f32,
    pub continuation: VecDeque<(Move, u32)>,
}

impl MoveInfo {
    pub fn ptn_comment(&self, flip_reward: bool) -> String {
        let eval = if flip_reward { -self.reward } else { self.reward };
        format!(" {{e: {:+.3}, p: {:.4}, v: {}}}", eval, self.policy, self.visits)
    }
}

impl Display for MoveInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{: <8} {: >8} {: >+8.4} {: >8.4} | {}",
            self.mov,
            self.visits,
            self.reward,
            self.policy,
            self.continuation
                .iter()
                .map(|(mov, visits)| mov.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}
