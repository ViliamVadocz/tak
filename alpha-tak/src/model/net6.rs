use std::path::Path;

use tak::*;
use tch::{nn, TchError, Tensor};

use super::network::{Eval, Network, Policy};
use crate::DEVICE;

const RES_BLOCKS: usize = 8;
const FILTERS: i64 = 128;
const N: i64 = 6;

#[derive(Debug)]
pub struct Net6 {
    vs: nn::VarStore,
}

impl Default for Net6 {
    fn default() -> Self {
        let vs = nn::VarStore::new(*DEVICE);
        Net6 { vs }
    }
}

impl Network<6> for Net6 {
    fn vs(&self) -> &nn::VarStore {
        &self.vs
    }

    fn save<T: AsRef<Path>>(&self, path: T) -> Result<(), TchError> {
        self.vs.save(path)?;
        Ok(())
    }

    fn load<T: AsRef<Path>>(path: T) -> Result<Self, TchError> {
        let mut nn = Self::default();
        nn.vs.load(path)?;
        Ok(nn)
    }

    fn forward_mcts(&self, input: Tensor) -> (Tensor, Tensor) {
        todo!()
    }

    fn forward_training(&self, input: Tensor) -> (Tensor, Tensor) {
        todo!()
    }

    fn policy_eval(&self, games: &[Game<6>]) -> Vec<(Policy, Eval)> {
        if games.is_empty() {
            return Vec::new();
        }
        todo!()
    }
}
