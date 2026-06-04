use rustls::pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
use std::{error::Error as StdError, io, net::ToSocketAddrs, sync::Arc};
use tokio::net::TcpListener;
use tokio_rustls::{TlsAcceptor, rustls};

pub async fn create_tls_server(
    cert_path: &str,
    key_path: &str,
    addr: &String,
) -> Result<(TlsAcceptor, TcpListener), Box<dyn StdError + Send + Sync + 'static>> {
    let sockaddr = addr
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::from(io::ErrorKind::AddrNotAvailable))?;
    let certs = CertificateDer::pem_file_iter(cert_path)?.collect::<Result<Vec<_>, _>>()?;
    let key = PrivateKeyDer::from_pem_file(key_path)?;
    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    let acceptor = TlsAcceptor::from(Arc::new(config));
    let listener = TcpListener::bind(&sockaddr).await?;
    Ok((acceptor, listener))
}
