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
use std::{process, fs};

#[tokio::main]
async fn main() {
    let mut args = Args::parse();
    // HACK: The safe resolver pops . out of the PathBuf, leaving an empty
    // pathbuf, which causes nothing to match when it is resolved.
    if args.root.as_os_str() == "." {
        match fs::canonicalize(args.root) {
            Ok(c) => {
                args.root = c;
            }
            Err(x) => {
                eprintln!("Unable to canonicalize serve directory: {}", x);
                process::exit(1);
            }
        }
    }
    else if args.root.to_string_lossy().contains("..") {
        eprintln!("Root directory cannot contain '..' for security reasons. Use absolute paths.");
        process::exit(1);
    }
    server::run(args).await;
}
