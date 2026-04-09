//! GG 引擎核心模块
//! 提供基础类型、错误处理和平台抽象接口

/// 错误类型枚举
#[derive(Debug)]
pub enum GErrorKind {
    /// IO 错误
    Io,
    /// 资源错误
    Asset,
    /// 平台错误
    Platform,
    /// ECS 错误
    Ecs,
    /// 插件错误
    Plugin,
    /// 运行时错误
    Runtime,
    /// 其他错误
    Other,
}

/// GG 引擎错误类型
#[derive(Debug)]
pub struct GError {
    /// 错误类型
    pub kind: GErrorKind,
    /// 错误消息
    pub message: String,
}

impl std::fmt::Display for GError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}

impl std::error::Error for GError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

/// GG 引擎结果类型
pub type GResult<T> = std::result::Result<T, GError>;

/// 平台抽象层
pub mod platform {
    /// 平台标识
    pub type PlatformId = String;
    
    /// 构建配置
    pub struct BuildConfig {
        // 构建配置字段
    }
    
    /// 生成上下文
    pub struct GenerateContext {
        // 生成上下文字段
    }
    
    /// 打包上下文
    pub struct PackageContext {
        // 打包上下文字段
    }
    
    /// 运行上下文
    pub struct RunContext {
        // 运行上下文字段
    }
    
    /// 平台 trait
    pub trait Platform {
        /// 获取平台标识
        fn id(&self) -> PlatformId;
        
        /// 获取平台显示名称
        fn display_name(&self) -> &str;
        
        /// 配置构建
        fn configure_build(&self, config: &mut BuildConfig);
        
        /// 生成平台特定代码
        fn generate_code(&self, ctx: &GenerateContext) -> super::GResult<std::path::PathBuf>;
        
        /// 打包最终产物
        fn package(&self, ctx: &PackageContext) -> super::GResult<Vec<std::path::PathBuf>>;
        
        /// 本地运行/部署
        fn run(&self, ctx: &RunContext) -> super::GResult<()> {
            Ok(())
        }
    }
}

/// 插件系统
pub mod plugin {
    /// 插件 trait
    pub trait Plugin {
        /// 插件名称
        fn name(&self) -> &str;
        
        /// 初始化插件
        fn initialize(&self) -> super::GResult<()>;
        
        /// 关闭插件
        fn shutdown(&self) -> super::GResult<()>;
    }
}
