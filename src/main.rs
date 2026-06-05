//! HTTP/HTTPS Server in Rust using Tokio and Rustls.
//! Stephen Marz
//! 5-Jun-2026
mod args;
mod client;
mod http;
mod mime;
mod server;
mod ssl;

use args::Args;
use clap::Parser;
use std::fs::canonicalize;

#[tokio::main]
async fn main() {
    let mut args = Args::parse();
    // HACK: The safe resolver pops . out of the PathBuf, leaving an empty
    // pathbuf, which causes nothing to match when it is resolved.
    if args.root.as_os_str() == "." {
        match canonicalize(args.root) {
            Ok(c) => {
                args.root = c;
            }
            Err(x) => {
                eprintln!("Unable to canonicalize serve directory: {}", x);
                std::process::exit(1);
            }
        }
    }
    server::run(args).await;
}
