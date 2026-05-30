mod args;
mod client;
mod http;
mod server;
mod mime;

use args::Args;
use clap::Parser;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    server::run(args).await;
}
