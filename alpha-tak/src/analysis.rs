use arrayvec::ArrayVec;
use tak::*;

use crate::{config::KOMI, search::node::Node};

const MAX_BRANCH_LENGTH: usize = 10;
const BRANCH_MIN_VISITS: u32 = 100;
const CANDIDATE_MOVE_RATIO: f32 = 0.7;

#[derive(Default)]
pub struct Analysis<const N: usize> {
    played_turns: Vec<Turn<N>>,
    eval: Vec<Option<f32>>,
    branches: Vec<Branch<N>>,
}

impl<const N: usize> Analysis<N> {
    pub fn from_opening(opening: Vec<Turn<N>>) -> Self {
        Analysis {
            eval: vec![None; opening.len()],
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
        let eval_perspective = if ply % 2 == 0 {1.} else {-1.};
        for (candidate, node) in candidates {
            if candidate == &played_turn {
                // following engine line
                continue;
            }

            // create branch from continuation
            let mut continuation = node.continuation(BRANCH_MIN_VISITS, MAX_BRANCH_LENGTH - 1);
            continuation.push_front(candidate.clone());
            self.branches.push(Branch {
                ply,
                line: continuation.into_iter().collect(),
                eval: eval_perspective * node.expected_reward,
            });
        }

        let child = children.get(&played_turn).unwrap();
        self.eval.push(Some(eval_perspective * child.expected_reward));
        self.played_turns.push(played_turn)
    }
}

impl<const N: usize> ToPTN for Analysis<N> {
    fn to_ptn(&self) -> String {
        let mut out = format!("[Size \"{N}\"]\n[Komi \"{KOMI}\"]\n");
        let mut turn_iter = self.played_turns.iter();
        let mut eval_iter = self.eval.iter();
        let mut move_num = 1;
        while let Some(white) = turn_iter.next() {
            // add white turn
            out.push_str(&format!("{move_num}. "));
            out.push_str(&white.to_ptn());

            // maybe add eval
            if let Some(Some(eval)) = eval_iter.next() {
                out.push_str(&format!(" {{{eval}}}"));
            }
            out.push(' ');

            // maybe add black move
            if let Some(black) = turn_iter.next() {
                out.push_str(&black.to_ptn());
                // maybe add eval
                if let Some(Some(eval)) = eval_iter.next() {
                    out.push_str(&format!(" {{{eval}}}"));
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
    eval: f32,
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
                self.eval,
                turn_iter.next().unwrap_or_default()
            ));
        } else {
            out.push_str(&format!(
                "{move_num}. -- {} {{{}}}\n",
                turn_iter.next().unwrap(),
                self.eval
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
