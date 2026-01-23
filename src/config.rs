use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(name = "takumi-http")]
#[command(author, version, about = "HTTP server for Takumi image rendering")]
pub struct Config {
    #[arg(short, long, default_value_t = 3000, env = "TAKUMI_PORT")]
    pub port: u16,

    #[arg(short = 'd', long, env = "TAKUMI_FONT_DIR")]
    pub font_dir: Option<PathBuf>,

    /// Load embedded default fonts (Geist, Geist Mono, Twemoji). Set to false to only use fonts from --font-dir.
    #[arg(long, default_value_t = true, env = "TAKUMI_LOAD_DEFAULT_FONTS", action = clap::ArgAction::Set)]
    pub load_default_fonts: bool,

    #[arg(long, default_value_t = 50_000_000, env = "TAKUMI_BODY_LIMIT")]
    pub body_limit: usize,

    #[arg(long, default_value_t = true, env = "TAKUMI_ENABLE_CACHE")]
    pub enable_cache: bool,

    #[arg(long, default_value = "info", env = "TAKUMI_LOG_LEVEL")]
    pub log_level: String,
}

impl Config {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
