//! Loopback TLS for the fake portal: a throwaway CA and a `127.0.0.1` leaf
//! made with the `openssl` CLI (as `tests/fixtures/oauth/server.py` does), and
//! an axum [`Listener`] that completes the handshake before handing the stream
//! over.
//!
//! TLS is not decoration: the connector refuses any OAuth endpoint that is not
//! `https`, so a plain-HTTP fake could never be discovered.

use std::net::SocketAddr;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

use axum::serve::Listener;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::server::TlsStream;

const LEAF_EXTENSIONS: &str = "subjectAltName=DNS:localhost,IP:127.0.0.1\n\
basicConstraints=CA:FALSE\n\
keyUsage=digitalSignature,keyEncipherment\n\
extendedKeyUsage=serverAuth\n";

fn openssl(dir: &Path, args: &[&str]) {
    let status = Command::new("openssl")
        .args(args)
        .current_dir(dir)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("run openssl (the fake portal needs it for its TLS certificate)");
    assert!(status.success(), "openssl {args:?} failed");
}

// Writes `ca.pem`, `server.pem` and `server.key` into `dir`.
pub(super) fn write_certificates(dir: &Path) {
    openssl(
        dir,
        &[
            "req",
            "-x509",
            "-newkey",
            "rsa:2048",
            "-nodes",
            "-keyout",
            "ca.key",
            "-out",
            "ca.pem",
            "-days",
            "1",
            "-subj",
            "/CN=Fake portal test CA",
        ],
    );
    openssl(
        dir,
        &[
            "req",
            "-newkey",
            "rsa:2048",
            "-nodes",
            "-keyout",
            "server.key",
            "-out",
            "server.csr",
            "-subj",
            "/CN=127.0.0.1",
        ],
    );
    std::fs::write(dir.join("leaf.ext"), LEAF_EXTENSIONS).expect("write leaf extensions");
    openssl(
        dir,
        &[
            "x509",
            "-req",
            "-in",
            "server.csr",
            "-CA",
            "ca.pem",
            "-CAkey",
            "ca.key",
            "-CAcreateserial",
            "-out",
            "server.pem",
            "-days",
            "1",
            "-extfile",
            "leaf.ext",
        ],
    );
}

pub(super) fn acceptor(dir: &Path) -> TlsAcceptor {
    let certs: Vec<CertificateDer<'static>> = CertificateDer::pem_file_iter(dir.join("server.pem"))
        .expect("open server.pem")
        .collect::<Result<_, _>>()
        .expect("parse server.pem");
    let key = PrivateKeyDer::from_pem_file(dir.join("server.key")).expect("parse server.key");
    let mut config = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()
    .expect("TLS protocol versions")
    .with_no_client_auth()
    .with_single_cert(certs, key)
    .expect("TLS server certificate");
    config.alpn_protocols = vec![b"http/1.1".to_vec()];
    TlsAcceptor::from(Arc::new(config))
}

struct TlsListener {
    tcp: TcpListener,
    acceptor: TlsAcceptor,
}

impl Listener for TlsListener {
    type Io = TlsStream<TcpStream>;
    type Addr = SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            let Ok((stream, addr)) = self.tcp.accept().await else {
                continue;
            };
            let handshake = self.acceptor.accept(stream);
            if let Ok(Ok(tls)) = tokio::time::timeout(Duration::from_secs(5), handshake).await {
                return (tls, addr);
            }
        }
    }

    fn local_addr(&self) -> std::io::Result<Self::Addr> {
        self.tcp.local_addr()
    }
}

pub(super) fn spawn(
    tcp: TcpListener,
    acceptor: TlsAcceptor,
    router: axum::Router,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        axum::serve(TlsListener { tcp, acceptor }, router)
            .await
            .expect("fake portal");
    })
}
