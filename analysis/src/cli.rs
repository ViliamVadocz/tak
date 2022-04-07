use clap::Parser;

/// Train AlphaTak
#[derive(Parser)]
pub struct Args {
    /// Path to model
    pub model_path: String,
    /// How many virtual rollouts to perform per batch
    pub batch_size: Option<u32>,
    /// Disable GPU usage
    #[clap(short, long)]
    pub no_gpu: bool,
}
