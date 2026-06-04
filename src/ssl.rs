use std::io;
use std::net::ToSocketAddrs;

use std::error::Error as StdError;
use std::sync::Arc;

use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio::net::TcpListener;
use tokio_rustls::{rustls, TlsAcceptor};

pub async fn create_tls_server(cert_path: &str, key_path: &str, addr: &String) -> Result<(TlsAcceptor, TcpListener), Box<dyn StdError + Send + Sync + 'static>> {
    let sockaddr = addr.to_socket_addrs()?.next().ok_or_else(|| io::Error::from(io::ErrorKind::AddrNotAvailable))?;
    let certs = CertificateDer::pem_file_iter(cert_path)?.collect::<Result<Vec<_>, _>>()?;
    let key = PrivateKeyDer::from_pem_file(key_path)?;
    let config = rustls::ServerConfig::builder().with_no_client_auth().with_single_cert(certs, key)?;
    let acceptor = TlsAcceptor::from(Arc::new(config));
    let listener = TcpListener::bind(&sockaddr).await?;
    Ok((acceptor, listener))
}

// async fn amain() -> Result<(), Box<dyn StdError + Send + Sync + 'static>> {
//     let options: Options = argh::from_env();

//     let addr = options
//         .addr
//         .to_socket_addrs()?
//         .next()
//         .ok_or_else(|| io::Error::from(io::ErrorKind::AddrNotAvailable))?;
//     let certs = CertificateDer::pem_file_iter(&options.cert)?.collect::<Result<Vec<_>, _>>()?;
//     let key = PrivateKeyDer::from_pem_file(&options.key)?;
//     let flag_echo = options.echo_mode;

//     let config = rustls::ServerConfig::builder()
//         .with_no_client_auth()
//         .with_single_cert(certs, key)?;
//     let acceptor = TlsAcceptor::from(Arc::new(config));

//     let listener = TcpListener::bind(&addr).await?;

//     loop {
//         let (stream, peer_addr) = listener.accept().await?;
//         let acceptor = acceptor.clone();

//         let fut = async move {
//             let mut stream = acceptor.accept(stream).await?;

//             if flag_echo {
//                 let (mut reader, mut writer) = split(stream);
//                 let n = copy(&mut reader, &mut writer).await?;
//                 writer.flush().await?;
//                 println!("Echo: {} - {}", peer_addr, n);
//             } else {
//                 let mut output = sink();
//                 stream
//                     .write_all(
//                         &b"HTTP/1.0 200 ok\r\n\
//                     Connection: close\r\n\
//                     Content-length: 12\r\n\
//                     \r\n\
//                     Hello world!"[..],
//                     )
//                     .await?;
//                 stream.shutdown().await?;
//                 copy(&mut stream, &mut output).await?;
//                 println!("Hello: {}", peer_addr);
//             }

//             Ok(()) as io::Result<()>
//         };

//         tokio::spawn(async move {
//             if let Err(err) = fut.await {
//                 eprintln!("{:?}", err);
//             }
//         });
//     }
// }
