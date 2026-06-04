use crate::{args::Args, client, ssl};
use std::sync::Arc;
use tokio::{net::TcpListener, signal};

pub async fn run(args: Args) {
    let bind_addr = format!("{}:{}", args.address, args.port);

    // Canonicalize the root so we can print an absolute path.
    let root_display = args
        .root
        .canonicalize()
        .unwrap_or_else(|_| args.root.clone());

    if !args.quiet {
        println!("http-server");
        println!("  serving : {}", root_display.display());
        println!("  address : http://{bind_addr}");
        println!("  cache   : {}s", args.cache);
        println!("  listing : {}", !args.no_dir_listing);
        println!("  index   : {}", !args.no_index);
        if args.tls {
            println!("    ┏TLS  : {}", args.tls);
            println!("    ┠cert : {}", args.cert.as_ref().unwrap().display());
            println!("    ┗key  : {}", args.key.as_ref().unwrap().display());
        }
        println!();
        println!("Hit Ctrl-C to stop.");
        println!();
    }

    if !args.tls {
        run_http(args).await;
    }
    else {
        run_tls(args).await;
    }
}

async fn run_http(args: Args) {
    let bind_addr = format!("{}:{}", args.address, args.port);
    let listener = match TcpListener::bind(&bind_addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: could not bind to {bind_addr}: {e}");
            std::process::exit(1);
        }
    };
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
                if !args.quiet {
                    println!("\nshutting down.");
                }
                break;
            }
        }
    }
}

async fn run_tls(args: Args) {
    // Args should error check this for us.
    let cert_file = args.cert.clone().unwrap().to_string_lossy().into_owned();
    let key_file = args.key.clone().unwrap().to_string_lossy().into_owned();
    let addr = format!("{}:{}", args.address, args.port);

    let (acceptor, listener) = match ssl::create_tls_server(&cert_file, &key_file, &addr).await {
        Ok((x, y)) => (x, y),
        Err(e) => {
            eprintln!("ERROR: {:?}", e.as_ref());
            return;
        }
    };
    // We have acceptor: TlsAcceptor and listener: TcpListener at this point.
    let args = Arc::new(args);
    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((stream, addr)) => {
                        let acceptor = acceptor.clone();
                        let args = Arc::clone(&args);
                        tokio::spawn(async move {
                            let stream = match acceptor.accept(stream).await {
                                Ok(s) => s,
                                Err(_) => return,
                            };
                            client::handle_tls(stream, addr, (*args).clone()).await;
                        });
                    }
                    Err(e) => eprintln!("accept error: {e}"),
                }
            }
            _ = signal::ctrl_c() => {
                if !args.quiet {
                    println!("\nshutting down.");
                }
                break;
            }
        }
        // let (stream, peer_addr) = match listener.accept().await {
        //     Ok((stream , addr)) => (stream, addr),
        //     Err(e) => {
        //         eprintln!("{}", e);
        //         continue;
        //     }
        // };
        // let acceptor = acceptor.clone();

        // let fut = async move {
        //     let mut stream = acceptor.accept(stream).await?;

        //     let mut output = sink();
        //     stream
        //         .write_all(
        //             &b"HTTP/1.0 200 ok\r\n\
        //         Connection: close\r\n\
        //         Content-length: 12\r\n\
        //         \r\n\
        //         Hello world!"[..],
        //         )
        //         .await?;
        //     stream.shutdown().await?;
        //     copy(&mut stream, &mut output).await?;
        //     println!("Hello: {}", peer_addr);

        //     Ok(()) as io::Result<()>
        // };

        // tokio::spawn(async move {
        //     if let Err(err) = fut.await {
        //         eprintln!("{:?}", err);
        //     }
        // });
    }
}
