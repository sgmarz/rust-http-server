use clap::Parser;
use std::path::PathBuf;

/// A simple static HTTP file server.
#[derive(Parser, Debug, Clone)]
#[command(name = "http-server", version, about, long_about = None)]
pub struct Args {
    /// Root directory to serve files from.
    #[arg(default_value = ".")]
    pub root: PathBuf,

    /// Port to listen on.
    #[arg(short, long, default_value_t = 8080)]
    pub port: u16,

    /// Address to bind to.
    #[arg(short, long, default_value = "::")]
    pub address: String,

    /// Disable directory listing when no index.html is found.
    #[arg(short = 'd', long, default_value_t = false)]
    pub no_dir_listing: bool,

    /// Cache-Control max-age in seconds (0 disables caching).
    #[arg(short, long, default_value_t = 0)]
    pub cache: u64,

    /// Silences per-request log lines.
    #[arg(short, long)]
    pub silent: bool,
}
