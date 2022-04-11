use clap::Parser;

/// Run the bot on PlayTak
#[derive(Parser)]
pub struct Args {
    /// Path to model
    pub model_path: String,
    /// PlayTak Username
    pub username: Option<String>,
    /// PlayTak Password
    pub password: Option<String>,
    /// Disable GPU usage
    #[clap(short, long)]
    pub no_gpu: bool,
}
