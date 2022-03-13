#![feature(test)]
#![feature(thread_is_running)]

extern crate test;

use std::{
    fs::File,
    io::Write,
    sync::mpsc::channel,
    thread,
    time::{Duration, SystemTime},
};

use search::turn_map::Lut;
use tak::*;
use tch::{Cuda, Device};

use crate::{
    config::{KOMI, N},
    model::network::Network,
    player::Player,
    search::node::Node,
};

#[macro_use]
extern crate lazy_static;

pub mod model;
pub mod search;

pub mod analysis;
pub mod config;
pub mod threadpool;

pub mod agent;
pub mod example;
pub mod player;
pub mod repr;

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
