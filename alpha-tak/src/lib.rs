use std::time::SystemTime;

use tch::{Cuda, Device};

#[macro_use]
extern crate lazy_static;

mod analysis;
mod example;
mod model;
mod player;
mod repr;
mod search;

pub use analysis::Analysis;
pub use example::{Example, IncompleteExample};
pub use model::{net5::Net5, net6::Net6, network::Network};
pub use player::Player;
pub use search::{MoveInfo, Node, NodeDebugInfo};

lazy_static! {
    static ref DEVICE: Device = Device::cuda_if_available();
}

/// Try initializing CUDA.
/// Returns whether CUDA is available.
pub fn use_cuda() -> bool {
    tch::maybe_init_cuda();
    Cuda::is_available()
}

/// Get UNIX time in seconds.
pub fn sys_time() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
