use std::path::PathBuf;

use clap::Parser;
use tracing::Level;
#[derive(Debug, Parser, Clone)]
pub struct Cli {
    #[arg(default_value_t = Level::WARN)]
    ///set the minimum log level
    log_level: Level,
    ///override the default config path (~/.config/embargo_bar/config.ron)
    override_config: Option<PathBuf>,
}
