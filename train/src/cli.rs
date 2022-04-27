use clap::Parser;

/// Train AlphaTak
#[derive(Parser)]
pub struct Args {
    /// Path to model, use "random" or leave blank if you want a new model
    pub model_path: Option<String>,
    /// Paths to example files
    pub examples: Vec<String>,
    /// Disable GPU usage
    #[clap(short, long)]
    pub no_gpu: bool,
}
