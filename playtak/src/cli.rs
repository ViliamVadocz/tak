use clap::Parser;

/// Train AlphaTak
#[derive(Parser)]
pub struct Args {
    /// Path to model
    pub model_path: String,
    /// PlayTak Username
    pub username: String,
    /// PlayTak Password
    pub password: String,
    /// Disable GPU usage
    #[clap(short, long)]
    pub no_gpu: bool,
}
