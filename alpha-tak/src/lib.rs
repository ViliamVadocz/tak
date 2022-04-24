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
