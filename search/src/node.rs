use tak::takparse::Move;

use crate::{edge::Edge, exploration::upper_confidence_bound};

#[derive(Default)]
pub struct Node {
    value: f32,
    visits: u32, // N(s_t)
    parents: u32,
    edges: Box<[(Move, Edge)]>,
}

impl Node {
    pub fn is_leaf(&self) -> bool {
        todo!()
    }

    pub fn is_terminal(&self) -> bool {
        todo!()
    }

    pub fn choose(&mut self) -> (Move, &mut Edge) {
        let sum_of_action_visits = self
            .edges
            .iter()
            .map(|(_, edge)| edge.total_visits())
            .sum::<u32>() as f32;
        let uct = |edge: &Edge| -> f32 {
            upper_confidence_bound(
                edge.q_value,
                edge.policy,
                sum_of_action_visits,
                edge.total_visits() as f32,
            )
        };

        let (my_move, edge) = self
            .edges
            .iter_mut()
            .max_by(|(_, a), (_, b)| uct(a).partial_cmp(&uct(b)).unwrap())
            .unwrap();

        (*my_move, edge)
    }

    pub fn is_transposition(&self) -> bool {
        // edge has fewer visits than this node
        todo!()
    }

    pub fn expand(&mut self) {
        todo!()
    }
}
