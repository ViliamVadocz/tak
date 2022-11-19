use clap::Parser;

/// Run the bot on PlayTak
#[derive(Parser, Clone, Debug)]
pub struct Args {
    /// Path to model
    pub model_path: String,
    /// PlayTak Username
    pub username: Option<String>,
    /// PlayTak Password
    pub password: Option<String>,
    /// Start as black instead of white
    #[clap(short, long)]
    pub start_as_black: bool,
    /// Initial time in seconds
    #[clap(long, default_value_t = 600)]
    pub initial_time: u64,
    /// Increment in seconds
    #[clap(long, default_value_t = 10)]
    pub increment: u64,
    /// Time to think per move
    #[clap(short, long, default_value_t = 10)]
    pub time_to_think: u64,
    /// Disable GPU usage
    #[clap(short, long)]
    pub no_gpu: bool,
}
