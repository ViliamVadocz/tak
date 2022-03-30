use clap::Parser;

/// Train AlphaTak
#[derive(Parser)]
pub struct Args {
    /// Path to model
    pub model_path: String,
    /// Disable GPU usage
    #[clap(short, long)]
    pub no_gpu: bool,
}
