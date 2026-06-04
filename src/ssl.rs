use tokio_rustls::{ServerConfig, TlsAcceptor};
use std::{fs::File, io, sync::Arc};

pub fn load_tls_config(cert_path: &str, key_path: &str) -> io::Result<Arc<ServerConfig>> {
    let cert_file = &mut io::BufReader::new(File::open(cert_path).unwrap());
    let key_file = &mut io::BufReader::new(File::open(key_path).unwrap());

    let certs = pem::certs(cert_file).map(|c| c.unwrap()).collect();
    let key = pem::private_key(key_file).unwrap().unwrap();

    let sconfig = match ServerConfig::builder().with_no_client_auth().with_single_cert(certs, key) {
        Ok(x) => x,
        Err(e) => {
            return Err(io::Error::new(io::ErrorKind::InvalidData, e));
        }
    };

    Ok(Arc::new(sconfig))
}

pub fn acceptor_tls(cert_path: &str, key_path: &str) -> io::Result<TlsAcceptor> {
    match load_tls_config(cert_path, key_path) {
        Ok(x) => Ok(TlsAcceptor::from(x)),
        Err(e) => Err(e)
    }
}
