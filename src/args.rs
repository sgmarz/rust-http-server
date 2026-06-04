use clap::Parser;
use std::path::PathBuf;

/// A simple static HTTP/HTTPS file server.
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

    /// Always produce a dir listing even if index.html is found.
    #[arg(
        short = 'i',
        long,
        default_value_t = false,
        conflicts_with = "no_dir_listing"
    )]
    pub no_index: bool,

    /// Cache-Control max-age in seconds (0 disables caching).
    #[arg(short, long, default_value_t = 0)]
    pub cache: u64,

    /// Silences per-request log lines.
    #[arg(short, long, default_value_t = false)]
    pub silent: bool,

    /// Silences all non-error output.
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,

    /// Enable TLS. If true, you must specify the key and cert files.
    #[arg(
        short,
        long,
        default_value_t = false,
        requires = "key",
        requires = "cert"
    )]
    pub tls: bool,

    /// Key PEM file for TLS
    #[arg(long, requires = "tls")]
    pub key: Option<PathBuf>,

    /// Cert PEM file for TLS
    #[arg(long, requires = "tls")]
    pub cert: Option<PathBuf>,

    /// Redirect HTTP to HTTPS when using TLS.
    #[arg(long, default_value_t = false, requires = "tls")]
    pub https: bool,
}
