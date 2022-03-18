mod move_info;
mod branch;

use tak::*;

use crate::{config::KOMI, search::node::Node};

use self::{move_info::MoveInfo, branch::Branch};

const MAX_BRANCH_LENGTH: usize = 10;
const BRANCH_MIN_VISITS: u32 = 100;
const CANDIDATE_MOVE_RATIO: f32 = 0.7;

#[derive(Default)]
pub struct Analysis<const N: usize> {
    komi: i32,
    played_turns: Vec<Turn<N>>,
    move_info: Vec<Option<MoveInfo>>,
    branches: Vec<Branch<N>>,
}

impl<const N: usize> Analysis<N> {
    pub fn from_opening(opening: Vec<Turn<N>>, komi: i32) -> Self {
        Analysis {
            move_info: vec![None; opening.len()],
            played_turns: opening,
            komi,
            ..Default::default()
        }
    }

    pub fn update(&mut self, node: &Node<N>, played_turn: Turn<N>) {
        // find other candidate moves for branches
        let children = node.children.as_ref().unwrap();
        let top_visits = children
            .iter()
            .max_by_key(|(_, node)| node.visited_count)
            .unwrap()
            .1
            .visited_count;
        let candidates: Vec<_> = children
            .iter()
            .filter(|(_, node)| CANDIDATE_MOVE_RATIO < node.visited_count as f32 / top_visits as f32)
            .collect();

        let ply = self.played_turns.len();
        let eval_perspective = if ply % 2 == 0 { 1. } else { -1. };
        for (candidate, candidate_node) in candidates {
            if candidate == &played_turn {
                // following engine line
                continue;
            }

            // create branch from continuation
            let mut continuation = candidate_node.continuation(BRANCH_MIN_VISITS, MAX_BRANCH_LENGTH - 1);
            continuation.push_front(candidate.clone());
            self.branches.push(Branch {
                ply,
                line: continuation.into_iter().collect(),
                info: MoveInfo {
                    eval: eval_perspective * candidate_node.expected_reward,
                    policy: candidate_node.policy,
                    visits: candidate_node.visited_count,
                },
            });
        }

        let child = children.get(&played_turn).unwrap();
        self.move_info.push(Some(MoveInfo {
            eval: eval_perspective * child.expected_reward,
            policy: child.policy,
            visits: child.visited_count,
        }));
        self.played_turns.push(played_turn)
    }
}

impl<const N: usize> ToPTN for Analysis<N> {
    fn to_ptn(&self) -> String {
        let mut out = format!("[Size \"{N}\"]\n[Komi \"{}\"]\n", self.komi);
        let mut turn_iter = self.played_turns.iter();
        let mut info_iter = self.move_info.iter();
        let mut move_num = 1;
        while let Some(white) = turn_iter.next() {
            // add white turn
            out.push_str(&format!("{move_num}. "));
            out.push_str(&white.to_ptn());

            // maybe add eval
            if let Some(Some(info)) = info_iter.next() {
                out.push_str(&format!(" {{{}}}", info.to_ptn()));
            }
            out.push(' ');

            // maybe add black move
            if let Some(black) = turn_iter.next() {
                out.push_str(&black.to_ptn());
                // maybe add eval
                if let Some(Some(info)) = info_iter.next() {
                    out.push_str(&format!(" {{{}}}", info.to_ptn()));
                }
            }
            out.push('\n');
            move_num += 1;
        }

        for branch in self.branches.iter() {
            out.push('\n'); // empty line before branch
            out.push_str(&branch.to_ptn());
        }
        out
    }
}
