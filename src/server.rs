//! Main server loop and connection handling.
//! Stephen Marz
//! 5-Jun-2026
use crate::{args::Args, client, http, ssl};
use std::sync::Arc;
use tokio::{io::AsyncWriteExt, net::TcpListener, signal::ctrl_c};

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
            println!("    ┠https: {}", args.https);
            println!("    ┠cert : {}", args.cert.as_ref().unwrap().display());
            println!("    ┗key  : {}", args.key.as_ref().unwrap().display());
        }
        println!();
        println!("Hit Ctrl-C to stop.");
        println!();
    }
    match args.tls {
        true => run_tls(args).await,
        false => run_http(args).await,
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
                    Err(e) => eprintln!("error: accept: {e}"),
                }
            }
            _ = ctrl_c() => {
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
    // We are going to share the args across tasks, so we don't want it
    // to drop it. So, add an atomic reference count (Arc) around it.
    let args = Arc::new(args);
    // We have acceptor: TlsAcceptor and listener: TcpListener at this point.
    loop {
        tokio::select! {
            result = listener.accept() => {
                match result {
                    Ok((mut stream, addr)) => {
                        let acceptor = acceptor.clone();
                        // Clone here increases the reference count. It being
                        // dropped at the end will decrement the reference count.
                        let args = Arc::clone(&args);
                        tokio::spawn(async move {
                            // See if we need to redirect to HTTPS before accepting TLS.
                            // If the client is trying to connect without TLS, we can redirect
                            // them to the HTTPS URL instead of just dropping the connection.
                            if args.https {
                                let mut buf = [0u8; 1];
                                stream.peek(&mut buf).await.unwrap_or(0);
                                if buf[0] != ssl::TLS_HANDSHAKE_HELLO {
                                    // Not TLS handshake, so redirect to HTTPS URL.
                                    let location = format!("https://{}:{}", args.address, args.port);
                                    let response = http::Response::redirect(&location);
                                    if !args.quiet && !args.silent {
                                        println!(
                                            "{addr} HTTP -> HTTPS ({} {}) ({} bytes).",
                                            response.status,
                                            http::response_name(response.status),
                                            response.body.len(),
                                        );
                                    }
                                    let bytes = response.into_bytes();
                                    if let Err(e) = stream.write_all(&bytes).await {
                                        eprintln!("[{addr}] write error: {e}");
                                    }
                                    stream.shutdown().await.ok();
                                    return;
                                }
                            }
                            let stream = match acceptor.accept(stream).await {
                                Ok(s) => s,
                                Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
                                    // Typically a certificate error or TLS handshake failure.
                                    // Log it and drop the connection.
                                    return;
                                }
                                Err(e) => {
                                    eprintln!("[{addr}] TLS accept error: {e}");
                                    return;
                                }
                            };
                            client::handle_tls(stream, addr, (*args).clone()).await;
                        });
                    }
                    Err(e) => eprintln!("accept error: {e}"),
                }
            }
            _ = ctrl_c() => {
                if !args.quiet {
                    println!("\nshutting down.");
                }
                break;
            }
        }
    }
}
