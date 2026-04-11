//! TLS/SSL 安全连接模块
//! 提供基于 tokio-rustls 的 TLS 服务端和客户端支持

use std::{fs::File, future::Future, io::BufReader, pin::Pin, sync::Arc};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_rustls::rustls::{
    ClientConfig, DigitallySignedStruct, RootCertStore, ServerConfig, SignatureScheme,
    client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    crypto::ring::default_provider,
    pki_types::{CertificateDer, ServerName, UnixTime},
};

use gg_core::{GError, GErrorKind, GResult};

use crate::async_driver::AsyncConnection;

/// TLS 配置
///
/// 包含 TLS 服务端所需的证书和私钥路径，以及是否接受无效证书的选项。
pub struct TlsConfig {
    /// 证书文件路径（PEM 格式）
    pub cert_path: String,
    /// 私钥文件路径（PEM 格式）
    pub key_path: String,
    /// 是否接受无效证书（仅开发模式使用）
    pub accept_invalid_certs: bool,
}

/// TLS 接受器
///
/// 用于服务端接受 TLS 连接。从 PEM 文件加载证书和私钥，
/// 创建 TLS 接受器以处理入站 TLS 握手。
pub struct TlsAcceptor {
    /// tokio-rustls TLS 接受器
    acceptor: tokio_rustls::TlsAcceptor,
    /// TLS 配置
    _config: TlsConfig,
}

impl TlsAcceptor {
    /// 创建新的 TLS 接受器
    ///
    /// 从配置中指定的路径加载证书和私钥文件，创建 TLS 接受器。
    ///
    /// # 参数
    /// - `config`: TLS 配置，包含证书路径、私钥路径和验证选项
    pub fn new(config: TlsConfig) -> GResult<Self> {
        let certs = rustls_pemfile::certs(&mut BufReader::new(
            File::open(&config.cert_path)
                .map_err(|e| GError { kind: GErrorKind::Network, message: format!("TLS 证书文件打开失败: {}", e) })?,
        ))
        .collect::<Result<Vec<CertificateDer<'static>>, _>>()
        .map_err(|e| GError { kind: GErrorKind::Network, message: format!("TLS 证书文件读取失败: {}", e) })?;

        let key = rustls_pemfile::private_key(&mut BufReader::new(
            File::open(&config.key_path)
                .map_err(|e| GError { kind: GErrorKind::Network, message: format!("TLS 私钥文件打开失败: {}", e) })?,
        ))
        .map_err(|e| GError { kind: GErrorKind::Network, message: format!("TLS 私钥文件读取失败: {}", e) })?
        .ok_or_else(|| GError { kind: GErrorKind::Network, message: "TLS 私钥文件中未找到私钥".to_string() })?;

        let _ = default_provider().install_default();

        let server_config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| GError { kind: GErrorKind::Network, message: format!("TLS 服务端配置创建失败: {}", e) })?;

        Ok(Self { acceptor: tokio_rustls::TlsAcceptor::from(Arc::new(server_config)), _config: config })
    }

    /// 接受 TLS 连接
    ///
    /// 将 TCP 流升级为 TLS 连接，完成 TLS 握手。
    ///
    /// # 参数
    /// - `stream`: TCP 流
    pub async fn accept(
        &self,
        stream: tokio::net::TcpStream,
    ) -> GResult<tokio_rustls::server::TlsStream<tokio::net::TcpStream>> {
        self.acceptor
            .accept(stream)
            .await
            .map_err(|e| GError { kind: GErrorKind::Network, message: format!("TLS 接受连接失败: {}", e) })
    }
}

/// 危险证书验证器
///
/// 接受所有证书，仅用于开发模式。在生产环境中使用此验证器会导致安全风险。
#[derive(Debug)]
struct DangerousVerifier;

impl ServerCertVerifier for DangerousVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, tokio_rustls::rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, tokio_rustls::rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, tokio_rustls::rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::ED25519,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
        ]
    }
}

/// TLS 连接器
///
/// 用于客户端发起 TLS 连接。支持自定义域名和是否接受无效证书。
pub struct TlsConnector {
    /// tokio-rustls TLS 连接器
    connector: tokio_rustls::TlsConnector,
    /// 连接域名
    domain: String,
    /// 是否接受无效证书
    _accept_invalid_certs: bool,
}

impl TlsConnector {
    /// 创建新的 TLS 连接器
    ///
    /// 根据是否接受无效证书，创建使用系统根证书或危险验证器的 TLS 连接器。
    ///
    /// # 参数
    /// - `domain`: 连接的目标域名
    /// - `accept_invalid_certs`: 是否接受无效证书（仅开发模式使用）
    pub fn new(domain: &str, accept_invalid_certs: bool) -> GResult<Self> {
        let _ = default_provider().install_default();

        let client_config = if accept_invalid_certs {
            ClientConfig::builder()
                .dangerous()
                .with_custom_certificate_verifier(Arc::new(DangerousVerifier))
                .with_no_client_auth()
        }
        else {
            let mut root_store = RootCertStore::empty();
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
            ClientConfig::builder().with_root_certificates(root_store).with_no_client_auth()
        };

        Ok(Self {
            connector: tokio_rustls::TlsConnector::from(Arc::new(client_config)),
            domain: domain.to_string(),
            _accept_invalid_certs: accept_invalid_certs,
        })
    }

    /// 发起 TLS 连接
    ///
    /// 将 TCP 流升级为 TLS 连接，完成 TLS 握手。
    ///
    /// # 参数
    /// - `stream`: TCP 流
    pub async fn connect(
        &self,
        stream: tokio::net::TcpStream,
    ) -> GResult<tokio_rustls::client::TlsStream<tokio::net::TcpStream>> {
        let domain = ServerName::try_from(self.domain.as_str())
            .map_err(|e| GError { kind: GErrorKind::Network, message: format!("TLS 域名解析失败: {}", e) })?
            .to_owned();
        self.connector
            .connect(domain, stream)
            .await
            .map_err(|e| GError { kind: GErrorKind::Network, message: format!("TLS 连接失败: {}", e) })
    }
}

/// TLS 连接
///
/// 基于 tokio-rustls 的 TLS 服务端连接实现，实现 `AsyncConnection` trait。
pub struct TlsConnection {
    /// TLS 数据流
    stream: Option<tokio_rustls::server::TlsStream<tokio::net::TcpStream>>,
    /// 对端地址
    peer: Option<String>,
}

impl TlsConnection {
    /// 创建新的 TLS 连接
    ///
    /// # 参数
    /// - `stream`: TLS 服务端流
    pub fn new(stream: tokio_rustls::server::TlsStream<tokio::net::TcpStream>) -> Self {
        let peer = stream.get_ref().0.peer_addr().ok().map(|a| a.to_string());
        Self { stream: Some(stream), peer }
    }
}

impl AsyncConnection for TlsConnection {
    fn send<'a>(&'a mut self, data: &'a [u8]) -> Pin<Box<dyn Future<Output = GResult<()>> + Send + 'a>> {
        Box::pin(async move {
            let stream = self
                .stream
                .as_mut()
                .ok_or_else(|| GError { kind: GErrorKind::Network, message: "TLS 连接已关闭".to_string() })?;
            stream
                .write_all(data)
                .await
                .map_err(|e| GError { kind: GErrorKind::Network, message: format!("TLS 发送数据失败: {}", e) })
        })
    }

    fn recv(&mut self) -> Pin<Box<dyn Future<Output = GResult<Vec<u8>>> + Send + '_>> {
        Box::pin(async move {
            let stream = self
                .stream
                .as_mut()
                .ok_or_else(|| GError { kind: GErrorKind::Network, message: "TLS 连接已关闭".to_string() })?;
            let mut buf = vec![0u8; 4096];
            let n = stream
                .read(&mut buf)
                .await
                .map_err(|e| GError { kind: GErrorKind::Network, message: format!("TLS 接收数据失败: {}", e) })?;
            buf.truncate(n);
            Ok(buf)
        })
    }

    fn close(&mut self) -> Pin<Box<dyn Future<Output = GResult<()>> + Send + '_>> {
        Box::pin(async move {
            if let Some(stream) = self.stream.as_mut() {
                stream
                    .shutdown()
                    .await
                    .map_err(|e| GError { kind: GErrorKind::Network, message: format!("TLS 关闭连接失败: {}", e) })?;
            }
            self.stream = None;
            Ok(())
        })
    }

    fn peer_addr(&self) -> Option<String> {
        self.peer.clone()
    }
}
