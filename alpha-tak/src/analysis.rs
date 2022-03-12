use arrayvec::ArrayVec;
use tak::*;

use crate::{config::KOMI, search::node::Node};

const MAX_BRANCH_LENGTH: usize = 10;
const BRANCH_MIN_VISITS: u32 = 100;
const CANDIDATE_MOVE_RATIO: f32 = 0.7;

#[derive(Default)]
pub struct Analysis<const N: usize> {
    played_turns: Vec<Turn<N>>,
    move_info: Vec<Option<MoveInfo>>,
    branches: Vec<Branch<N>>,
}

impl<const N: usize> Analysis<N> {
    pub fn from_opening(opening: Vec<Turn<N>>) -> Self {
        Analysis {
            move_info: vec![None; opening.len()],
            played_turns: opening,
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
        let mut out = format!("[Size \"{N}\"]\n[Komi \"{KOMI}\"]\n");
        let mut turn_iter = self.played_turns.iter();
        let mut info_iter = self.move_info.iter();
        let mut move_num = 1;
        while let Some(white) = turn_iter.next() {
            // add white turn
            out.push_str(&format!("{move_num}. "));
            out.push_str(&white.to_ptn());

            // maybe add eval
            if let Some(Some(info)) = info_iter.next() {
                out.push_str(&info.to_ptn());
            }
            out.push(' ');

            // maybe add black move
            if let Some(black) = turn_iter.next() {
                out.push_str(&black.to_ptn());
                // maybe add eval
                if let Some(Some(info)) = info_iter.next() {
                    out.push_str(&info.to_ptn());
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

struct Branch<const N: usize> {
    ply: usize,
    line: ArrayVec<Turn<N>, MAX_BRANCH_LENGTH>,
    info: MoveInfo,
}

impl<const N: usize> ToPTN for Branch<N> {
    fn to_ptn(&self) -> String {
        let mut out = format!("{{{}_{}}}\n", self.ply, self.line.first().unwrap().to_ptn());

        let mut turn_iter = self.line.iter().map(|t| t.to_ptn());
        let mut move_num = 1 + self.ply / 2;

        // first move includes eval comment so it is handled differently
        if self.ply % 2 == 0 {
            out.push_str(&format!(
                "{move_num}. {} {{{}}} {}\n",
                turn_iter.next().unwrap(),
                self.info.to_ptn(),
                turn_iter.next().unwrap_or_default()
            ));
        } else {
            out.push_str(&format!(
                "{move_num}. -- {} {{{}}}\n",
                turn_iter.next().unwrap(),
                self.info.to_ptn()
            ));
        }
        move_num += 1;

        // add the rest of the turns
        while let Some(white) = turn_iter.next() {
            out.push_str(&format!(
                "{move_num}. {white} {}\n",
                turn_iter.next().unwrap_or_default()
            ));
            move_num += 1;
        }

        out
    }
}

#[derive(Default, Debug, Clone)]
struct MoveInfo {
    eval: f32,
    policy: f32,
    visits: u32,
}

impl ToPTN for MoveInfo {
    fn to_ptn(&self) -> String {
        format!("e: {:.4}, p: {:.4}, v: {}", self.eval, self.policy, self.visits)
    }
}
