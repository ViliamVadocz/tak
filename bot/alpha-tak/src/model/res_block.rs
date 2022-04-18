use std::ops::Add;

use tch::{nn, Tensor};

#[derive(Debug)]
pub struct ResBlock {
    pub conv1: nn::Conv2D,
    pub conv2: nn::Conv2D,
    pub batch_norm1: nn::BatchNorm,
    pub batch_norm2: nn::BatchNorm,
}

impl ResBlock {
    pub fn forward(&self, input: Tensor, train: bool) -> Tensor {
        input
            .apply_t(&self.conv1, train)
            .apply_t(&self.batch_norm1, train)
            .relu_()
            .apply_t(&self.conv2, train)
            .apply_t(&self.batch_norm2, train)
            .add(&input)
            .relu_()
    }
}
