use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;
use tracing::Level;
#[derive(Debug, Parser, Clone)]
pub struct Cli {
    #[arg(short, long, default_value_t = Level::WARN)]
    ///set the minimum log level
    pub log_level: Level,
    ///override the default config path (~/.config/embargo_bar/config.ron)
    #[arg(short = 'c', long)]
    pub override_config: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Command,
}
#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    Run,
    PrintConfig,
}
