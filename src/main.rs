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

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut args = Args::parse();
    args.root = match fs::canonicalize(args.root) {
        Ok(c) => c,
        Err(x) => {
            eprintln!("Unable to canonicalize serve directory: {}", x);
            process::exit(1);
        }
    };
    // Check to make sure that the root is a directory, not a file.
    {
        let md = fs::metadata(&args.root).unwrap();
        if !md.file_type().is_dir() {
            eprintln!("Specified ROOT is not a directory.");
            process::exit(1);
        }
    }
    server::run(args).await;
}
