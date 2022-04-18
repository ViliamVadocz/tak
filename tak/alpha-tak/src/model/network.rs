use std::{error::Error, path::Path};

use arrayvec::ArrayVec;
use tch::nn;

use super::res_block::ResBlock;
use crate::{
    config::{FILTERS, RES_BLOCKS},
    repr::{input_channels, moves_dims},
    DEVICE,
};

#[derive(Debug)]
pub struct Network<const N: usize> {
    pub vs: nn::VarStore,
    pub initial_conv: nn::Conv2D,
    pub initial_batch_norm: nn::BatchNorm,
    pub residual_blocks: ArrayVec<ResBlock, RES_BLOCKS>,
    pub fully_connected_policy: nn::Linear,
    pub fully_connected_eval: nn::Linear,
}

impl<const N: usize> Network<N> {
    pub fn save<T: AsRef<Path>>(&self, path: T) -> Result<(), Box<dyn Error>> {
        self.vs.save(path)?;
        Ok(())
    }

    pub fn load<T: AsRef<Path>>(path: T) -> Result<Network<N>, Box<dyn Error>> {
        let mut nn = Self::default();
        nn.vs.load(path)?;
        Ok(nn)
    }
}

impl<const N: usize> Default for Network<N> {
    fn default() -> Self {
        let vs = nn::VarStore::new(*DEVICE);
        let root = &vs.root();

        let conv_config = nn::ConvConfig {
            padding: 1,
            ..Default::default()
        };

        let initial_conv = nn::conv2d(root, input_channels(N) as i64, FILTERS, 3, conv_config);
        let initial_batch_norm = nn::batch_norm2d(root, FILTERS, Default::default());

        let mut residual_blocks = ArrayVec::new();
        for _ in 0..RES_BLOCKS {
            let conv1 = nn::conv2d(root, FILTERS, FILTERS, 3, conv_config);
            let conv2 = nn::conv2d(root, FILTERS, FILTERS, 3, conv_config);
            let batch_norm1 = nn::batch_norm2d(root, FILTERS, Default::default());
            let batch_norm2 = nn::batch_norm2d(root, FILTERS, Default::default());
            residual_blocks.push(ResBlock {
                conv1,
                conv2,
                batch_norm1,
                batch_norm2,
            });
        }

        let fully_connected_policy = nn::linear(
            root,
            FILTERS * (N * N) as i64,
            moves_dims(N) as i64,
            Default::default(),
        );
        let fully_connected_eval = nn::linear(root, FILTERS * (N * N) as i64, 1, Default::default());

        Network {
            vs,
            initial_conv,
            initial_batch_norm,
            residual_blocks,
            fully_connected_policy,
            fully_connected_eval,
        }
    }
}
