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
use std::{
    env::current_exe,
    fs::{canonicalize, metadata},
    process::{Command, Stdio, exit},
};
use tokio::runtime::Builder;

fn main() {
    let mut args = Args::parse();
    args.root = match canonicalize(args.root) {
        Ok(c) => c,
        Err(x) => {
            eprintln!("Error with ROOT: {}", x);
            exit(1);
        }
    };
    // Check to make sure that the root is a directory, not a file.
    {
        let md = metadata(&args.root).unwrap();
        if !md.file_type().is_dir() {
            eprintln!("Error with ROOT: Is not a directory.");
            exit(1);
        }
    }

    if args.background {
        daemonize(&args);
    }

    Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(server::run(args));
}

fn daemonize(args: &Args) {
    let exe = current_exe().expect("cannot find current exe");

    let mut cmd = Command::new(exe);

    // Forward all args except --background / -b
    if let Some(root) = &args.root.to_str() {
        cmd.arg(root);
    }

    // Detach stdin/stdout/stderr so the child is fully independent
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    match cmd.spawn() {
        Ok(child) => {
            if !args.quiet {
                println!("http-server running in background (pid {})", child.id());
            }
            exit(0); // parent exits immediately
        }
        Err(e) => {
            if !args.quiet {
                eprintln!("error: failed to background process: {e}");
            }
            exit(1);
        }
    }
}
