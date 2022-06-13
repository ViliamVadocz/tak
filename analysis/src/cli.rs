use clap::Parser;

/// Train AlphaTak
#[derive(Parser)]
pub struct Args {
    /// Board size
    pub board_size: usize,
    /// Path to model
    pub model_path: String,
    /// How many virtual rollouts to perform per batch
    #[clap(short, long, default_value_t = 64)]
    pub batch_size: u32,
    /// Path to PTN game file
    #[clap(short, long)]
    pub ptn_file: Option<String>,
    /// Start analysis from a position
    /// Format: "TPS;white_stones;white_caps;black_stones;black_caps;half_komi"
    /// Otherwise will assume reserve counts from TPS and Komi 2.
    #[clap(short, long)]
    pub from_position: Option<String>,
    /// Run an example game
    #[clap(short, long)]
    pub example_game: bool,
    /// Number of seconds to think
    #[clap(short, long, default_value_t = 15)]
    pub think_seconds: u64,
    /// Disable GPU usage
    #[clap(short, long)]
    pub no_gpu: bool,
}
