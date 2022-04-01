use clap::Parser;

/// Run the bot on PlayTak
#[derive(Parser)]
pub struct Args {
    /// Path to model
    pub model_path: String,
    /// PlayTak Username
    pub username: String,
    /// PlayTak Password
    /// Whether to seek as white
    #[clap(short, long)]
    pub seek_as_white: bool,
    /// Disable GPU usage
    #[clap(short, long)]
    pub no_gpu: bool,
}
