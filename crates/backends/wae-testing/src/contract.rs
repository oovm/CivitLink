//! 契约测试模块
//!
//! 提供基于 Pact 的契约测试工具，支持消费者和提供者的契约定义与验证。

#[cfg(feature = "pact")]
pub use pact_impl::*;

#[cfg(feature = "pact")]
mod pact_impl {
    use pact_consumer::prelude::*;
    use pact_verifier::*;
    use std::path::PathBuf;

    /// 消费者契约构建器
    ///
    /// 用于定义消费者与提供者之间的契约。
    pub struct ConsumerPactBuilder {
        pact: PactBuilder,
    }

    impl ConsumerPactBuilder {
        /// 创建新的消费者契约构建器
        ///
        /// # 参数
        /// - `consumer_name`: 消费者名称
        /// - `provider_name`: 提供者名称
        pub fn new(consumer_name: &str, provider_name: &str) -> Self {
            let pact = PactBuilder::new(consumer_name, provider_name);
            Self { pact }
        }

        /// 创建新的 V4 版本的消费者契约构建器
        ///
        /// # 参数
        /// - `consumer_name`: 消费者名称
        /// - `provider_name`: 提供者名称
        pub fn new_v4(consumer_name: &str, provider_name: &str) -> Self {
            let pact = PactBuilder::new_v4(consumer_name, provider_name);
            Self { pact }
        }

        /// 添加 HTTP 交互
        ///
        /// # 参数
        /// - `description`: 交互描述
        /// - `test_name`: 测试名称（可选）
        /// - `f`: 构建交互的闭包
        pub fn interaction<F>(mut self, description: &str, test_name: &str, f: F) -> Self
        where
            F: FnOnce(RequestResponsePactInteraction) -> RequestResponsePactInteraction,
        {
            self.pact = self.pact.interaction(description, test_name, f);
            self
        }

        /// 开始 Mock 服务器
        ///
        /// # 参数
        /// - `addr`: 监听地址（可选）
        /// - `tls`: TLS 配置（可选）
        pub fn start_mock_server(
            self,
            addr: Option<std::net::SocketAddr>,
            tls: Option<pact_consumer::mock_server::TlsConfig>,
        ) -> pact_consumer::mock_server::MockServer {
            self.pact.start_mock_server(addr, tls)
        }

        /// 异步开始 Mock 服务器
        ///
        /// # 参数
        /// - `addr`: 监听地址（可选）
        /// - `tls`: TLS 配置（可选）
        pub async fn start_mock_server_async(
            self,
            addr: Option<std::net::SocketAddr>,
            tls: Option<pact_consumer::mock_server::TlsConfig>,
        ) -> pact_consumer::mock_server::MockServer {
            self.pact.start_mock_server_async(addr, tls).await
        }
    }

    /// 提供者验证配置
    ///
    /// 用于配置提供者契约验证的参数。
    pub struct ProviderVerifier {
        provider_info: ProviderInfo,
        pact_sources: Vec<PactSource>,
        verification_options: VerificationOptions,
    }

    impl ProviderVerifier {
        /// 创建新的提供者验证器
        ///
        /// # 参数
        /// - `provider_name`: 提供者名称
        /// - `host`: 提供者主机地址
        /// - `port`: 提供者端口
        pub fn new(provider_name: &str, host: &str, port: u16) -> Self {
            let provider_info = ProviderInfo {
                name: provider_name.to_string(),
                protocol: "http".to_string(),
                host: host.to_string(),
                port: Some(port),
                path: "/".to_string(),
            };
            Self { provider_info, pact_sources: Vec::new(), verification_options: VerificationOptions::default() }
        }

        /// 设置协议
        ///
        /// # 参数
        /// - `protocol`: 协议（http 或 https）
        pub fn protocol(mut self, protocol: &str) -> Self {
            self.provider_info.protocol = protocol.to_string();
            self
        }

        /// 设置路径
        ///
        /// # 参数
        /// - `path`: 基础路径
        pub fn path(mut self, path: &str) -> Self {
            self.provider_info.path = path.to_string();
            self
        }

        /// 添加契约源为文件
        ///
        /// # 参数
        /// - `file`: 契约文件路径
        pub fn pact_file(mut self, file: &str) -> Self {
            self.pact_sources.push(PactSource::File(PathBuf::from(file)));
            self
        }

        /// 添加契约源为文件目录
        ///
        /// # 参数
        /// - `dir`: 契约文件目录路径
        pub fn pact_directory(mut self, dir: &str) -> Self {
            self.pact_sources.push(PactSource::FileDirectory(PathBuf::from(dir)));
            self
        }

        /// 添加契约源为 Pact Broker
        ///
        /// # 参数
        /// - `broker_url`: Pact Broker 地址
        /// - `consumer_name`: 消费者名称（可选）
        pub fn pact_broker(mut self, broker_url: &str, consumer_name: Option<&str>) -> Self {
            self.pact_sources.push(PactSource::BrokerUrl {
                url: broker_url.to_string(),
                auth: None,
                consumer_name: consumer_name.map(|s| s.to_string()),
                provider_name: Some(self.provider_info.name.clone()),
            });
            self
        }

        /// 设置验证选项
        ///
        /// # 参数
        /// - `options`: 验证选项
        pub fn verification_options(mut self, options: VerificationOptions) -> Self {
            self.verification_options = options;
            self
        }

        /// 同步执行提供者验证
        pub fn verify(self) -> Result<(), Box<dyn std::error::Error>> {
            verify_provider(self.provider_info, self.pact_sources, None, None, self.verification_options)?;
            Ok(())
        }

        /// 异步执行提供者验证
        pub async fn verify_async(self) -> Result<(), Box<dyn std::error::Error>> {
            verify_provider_async(self.provider_info, self.pact_sources, None, None, self.verification_options).await?;
            Ok(())
        }
    }
}
