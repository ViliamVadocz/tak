use rustc_hash::FxHashMap;
use tak::{takparse::Move, Game};

use crate::node::Node;

#[derive(Default)]
pub struct Search<const N: usize, const HALF_KOMI: i8> {
    root: Node,
    nodes: FxHashMap<Game<N, HALF_KOMI>, Node>,
}

impl<const N: usize, const HALF_KOMI: i8> Search<N, HALF_KOMI> {
    pub fn virtual_rollout(
        &mut self,
        game: &mut Game<N, HALF_KOMI>,
        trajectory: &mut Vec<Move>,
    ) -> f32 {
        assert!(trajectory.is_empty());
        let mut node = &mut self.root;
        while !node.is_leaf() {
            let (my_move, mut edge) = node.choose();
            trajectory.push(my_move);
            edge.virtual_visits += 1;
            game.play(my_move).unwrap();
            if let Some(next) = self.nodes.get_mut(game) {
                if next.is_transposition() {
                    todo!();
                }
                if next.is_terminal() {
                    todo!();
                }
                node = next;
            }
        }
        node.expand();
        // todo value, policies
        todo!()
    }

    pub fn back_propagate(&mut self, trajectory: &mut Vec<Move>) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use tak::Game;

    use super::Search;

    #[test]
    fn terminal_solver_3x3() {
        let mut search = Search::default();
        let game: Game<3, 0> = Game::default();
        let mut trajectory = Vec::new();
        while !search.root.is_terminal() {
            search.virtual_rollout(&mut game.clone(), &mut trajectory);
            search.back_propagate(&mut trajectory);
        }
        todo!()
    }
}
