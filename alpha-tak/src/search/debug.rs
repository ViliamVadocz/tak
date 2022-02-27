use std::collections::VecDeque;

use tak::*;

use super::node::Node;

impl<const N: usize> Node<N> {
    pub fn debug(&self, limit: Option<usize>) -> String {
        const MAX_CONTINUATION_LEN: u8 = 5;
        format!("turn      visited   reward   policy | continuation\n{}", {
            let mut p: Vec<_> = self.children.as_ref().unwrap().iter().collect();
            p.sort_by_key(|(_turn, node)| node.visited_count);
            p.reverse();
            p.iter()
                .take(limit.unwrap_or(usize::MAX))
                .map(|(turn, node)| {
                    let continuation = node
                        .continuation(MAX_CONTINUATION_LEN)
                        .into_iter()
                        .map(|t| t.to_ptn())
                        .collect::<Vec<_>>()
                        .join(" ");
                    format!(
                        "{: <8} {: >8} {: >8.4} {: >8.4} | {}\n",
                        turn.to_ptn(),
                        node.visited_count,
                        node.expected_reward,
                        node.policy,
                        continuation,
                    )
                })
                .collect::<String>()
        })
    }

    pub fn continuation(&self, depth: u8) -> VecDeque<Turn<N>> {
        const MIN_VISIT_COUNT: u32 = 10;
        if depth == 0 || self.children.is_none() || self.visited_count <= MIN_VISIT_COUNT {
            return VecDeque::new();
        }
        let turn = self.pick_move(true);
        let children = self.children.as_ref().unwrap();
        let node = children.get(&turn).unwrap();
        let mut turns = node.continuation(depth - 1);
        turns.push_front(turn);
        turns
    }
}
