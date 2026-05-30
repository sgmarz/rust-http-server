use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::signal;

use crate::args::Args;
use crate::client;

pub async fn run(args: Args) {
    let bind_addr = format!("{}:{}", args.address, args.port);

    let listener = match TcpListener::bind(&bind_addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: could not bind to {bind_addr}: {e}");
            std::process::exit(1);
        }
    };

    // Canonicalize the root so we can print an absolute path.
    let root_display = args
        .root
        .canonicalize()
        .unwrap_or_else(|_| args.root.clone());

    println!("http-server");
    println!("  serving : {}", root_display.display());
    println!("  address : http://{bind_addr}");
    println!("  cache   : {}s", args.cache);
    println!("  listing : {}", !args.no_dir_listing);
    println!();
    println!("Hit Ctrl-C to stop.");
    println!();

    // Share args across tasks without cloning the PathBuf every accept.
    let args = Arc::new(args);

    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, addr)) => {
                        let args = Arc::clone(&args);
                        tokio::spawn(async move {
                            client::handle(stream, addr, (*args).clone()).await;
                        });
                    }
                    Err(e) => eprintln!("accept error: {e}"),
                }
            }
            _ = signal::ctrl_c() => {
                println!("\nshutting down.");
                break;
            }
        }
    }
}
