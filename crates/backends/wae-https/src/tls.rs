//! TLS 配置模块
//!
//! 提供 TLS/HTTPS 支持的工具函数，包括 HTTP/2 ALPN 协商支持。

use std::{fs::File, io::BufReader, sync::Arc};
use tokio_rustls::{
    TlsAcceptor,
    rustls::{
        RootCertStore, ServerConfig,
        pki_types::{CertificateDer, PrivateKeyDer},
        server::WebPkiClientVerifier,
    },
};

use crate::{WaeError, WaeResult};

/// ALPN 协议标识符
pub mod alpn {
    /// HTTP/1.1 ALPN 协议标识
    pub const HTTP_1_1: &[u8] = b"http/1.1";
    /// HTTP/2 ALPN 协议标识
    pub const HTTP_2: &[u8] = b"h2";
}

/// 创建 TLS 接受器
///
/// 从 PEM 格式的证书和私钥文件创建 TLS 接受器。
/// 默认支持 HTTP/1.1 协议。
pub fn create_tls_acceptor(cert_path: &str, key_path: &str) -> WaeResult<TlsAcceptor> {
    create_tls_acceptor_with_http2(cert_path, key_path, false)
}

/// 创建支持 HTTP/2 的 TLS 接受器
///
/// 从 PEM 格式的证书和私钥文件创建 TLS 接受器，
/// 支持 HTTP/1.1 和 HTTP/2 的 ALPN 协商。
///
/// 参数：
/// - `cert_path`: 证书文件路径
/// - `key_path`: 私钥文件路径
/// - `enable_http2`: 是否启用 HTTP/2 支持
pub fn create_tls_acceptor_with_http2(cert_path: &str, key_path: &str, enable_http2: bool) -> WaeResult<TlsAcceptor> {
    let certs = load_certs(cert_path)?;
    let key = load_private_key(key_path)?;

    let alpn_protocols =
        if enable_http2 { vec![alpn::HTTP_2.to_vec(), alpn::HTTP_1_1.to_vec()] } else { vec![alpn::HTTP_1_1.to_vec()] };

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| WaeError::internal(format!("Failed to create TLS config: {}", e)))?;

    let mut config = Arc::new(config);
    Arc::get_mut(&mut config).expect("Config should be unique").alpn_protocols = alpn_protocols;

    Ok(TlsAcceptor::from(config))
}

/// 创建支持客户端证书验证的 TLS 接受器
///
/// 从 PEM 格式的证书和私钥文件创建 TLS 接受器，
/// 同时验证客户端证书。
///
/// 参数：
/// - `cert_path`: 服务端证书文件路径
/// - `key_path`: 服务端私钥文件路径
/// - `ca_path`: CA 证书文件路径（用于验证客户端证书）
/// - `enable_http2`: 是否启用 HTTP/2 支持
pub fn create_tls_acceptor_with_client_auth(
    cert_path: &str,
    key_path: &str,
    ca_path: &str,
    enable_http2: bool,
) -> WaeResult<TlsAcceptor> {
    let certs = load_certs(cert_path)?;
    let key = load_private_key(key_path)?;
    let ca_certs = load_certs(ca_path)?;

    let mut root_cert_store = RootCertStore::empty();
    for cert in ca_certs {
        root_cert_store.add(cert).map_err(|e| WaeError::internal(format!("Failed to add CA cert: {}", e)))?;
    }

    let client_verifier = WebPkiClientVerifier::builder(Arc::new(root_cert_store))
        .build()
        .map_err(|e| WaeError::internal(format!("Failed to create client verifier: {}", e)))?;

    let alpn_protocols =
        if enable_http2 { vec![alpn::HTTP_2.to_vec(), alpn::HTTP_1_1.to_vec()] } else { vec![alpn::HTTP_1_1.to_vec()] };

    let config = ServerConfig::builder()
        .with_client_cert_verifier(client_verifier)
        .with_single_cert(certs, key)
        .map_err(|e| WaeError::internal(format!("Failed to create TLS config: {}", e)))?;

    let mut config = Arc::new(config);
    Arc::get_mut(&mut config).expect("Config should be unique").alpn_protocols = alpn_protocols;

    Ok(TlsAcceptor::from(config))
}

/// 从 PEM 文件加载证书
fn load_certs(path: &str) -> WaeResult<Vec<CertificateDer<'static>>> {
    let file = File::open(path).map_err(|e| WaeError::internal(format!("Failed to open cert file {}: {}", path, e)))?;
    let mut reader = BufReader::new(file);

    rustls_pemfile::certs(&mut reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| WaeError::internal(format!("Failed to parse cert file {}: {}", path, e)))
}

/// 从 PEM 文件加载私钥
fn load_private_key(path: &str) -> WaeResult<PrivateKeyDer<'static>> {
    let file = File::open(path).map_err(|e| WaeError::internal(format!("Failed to open key file {}: {}", path, e)))?;
    let mut reader = BufReader::new(file);

    let keys: Vec<PrivateKeyDer<'static>> = rustls_pemfile::private_key(&mut reader)
        .map_err(|e| WaeError::internal(format!("Failed to parse key file {}: {}", path, e)))?
        .into_iter()
        .collect();

    keys.into_iter().next().ok_or_else(|| WaeError::internal(format!("No private key found in {}", path)))
}

/// TLS 配置构建器
pub struct TlsConfigBuilder {
    cert_path: Option<String>,
    key_path: Option<String>,
    ca_path: Option<String>,
    enable_http2: bool,
}

impl TlsConfigBuilder {
    /// 创建新的 TLS 配置构建器
    pub fn new() -> Self {
        Self { cert_path: None, key_path: None, ca_path: None, enable_http2: true }
    }

    /// 设置证书文件路径
    pub fn cert_path(mut self, path: impl Into<String>) -> Self {
        self.cert_path = Some(path.into());
        self
    }

    /// 设置私钥文件路径
    pub fn key_path(mut self, path: impl Into<String>) -> Self {
        self.key_path = Some(path.into());
        self
    }

    /// 设置 CA 证书文件路径（用于客户端证书验证）
    pub fn ca_path(mut self, path: impl Into<String>) -> Self {
        self.ca_path = Some(path.into());
        self
    }

    /// 设置是否启用 HTTP/2
    pub fn enable_http2(mut self, enable: bool) -> Self {
        self.enable_http2 = enable;
        self
    }

    /// 构建 TLS 接受器
    pub fn build(self) -> WaeResult<TlsAcceptor> {
        let cert_path = self.cert_path.ok_or_else(|| WaeError::internal("Certificate path is required"))?;
        let key_path = self.key_path.ok_or_else(|| WaeError::internal("Key path is required"))?;

        match self.ca_path {
            Some(ca_path) => create_tls_acceptor_with_client_auth(&cert_path, &key_path, &ca_path, self.enable_http2),
            None => create_tls_acceptor_with_http2(&cert_path, &key_path, self.enable_http2),
        }
    }
}

impl Default for TlsConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}
