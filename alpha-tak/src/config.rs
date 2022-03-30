// game settings
pub const N: usize = 5;
pub const KOMI: i32 = 2;

// model
pub const RES_BLOCKS: usize = 8;
pub const FILTERS: i64 = 128;

// search
pub const CONTEMPT: f32 = 0.05;
pub const EXPLORATION_BASE: f32 = 500.0;
pub const EXPLORATION_INIT: f32 = 4.0;

// self-play
pub const SELF_PLAY_GAMES: usize = 500;
pub const ROLLOUTS_PER_MOVE: usize = 1000;
pub const TEMPERATURE_PLIES: u64 = 10;

pub const DIRICHLET_NOISE: f32 = 0.15;
pub const NOISE_RATIO: f32 = 0.5;

// train
pub const MAX_EXAMPLES: usize = 250_000;
pub const MAX_TRAIN_SIZE: usize = 50_000;
pub const BATCH_SIZE: i64 = 10_000;
pub const LEARNING_RATE: f64 = 1e-4;
pub const WEIGHT_DECAY: f64 = 1e-4;

// pit
pub const WIN_RATE_THRESHOLD: f64 = 0.55;
pub const PIT_MATCHES: usize = 80;
