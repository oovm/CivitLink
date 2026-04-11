//! 资产编译器适配器模块
//! 为所有资产格式提供 AssetCompiler trait 的实现，将各编译器模块的 API 适配为统一的管线接口

use std::sync::Mutex;

use crate::{
    pipeline::{AssetCompiler, CompileContext},
    types::{AssetEntry, AssetType},
};

/// VON 类资产编译器，覆盖 Animation/Config/Material/Prefab/Scene/VON 六种格式
///
/// 使用 gg-meta 的 VonCompiler 对 VON 格式资产进行编译和验证
pub struct VonAssetCompiler {
    /// 支持的资产类型列表
    supported: Vec<AssetType>,
}

impl VonAssetCompiler {
    /// 创建一个新的 VON 类资产编译器
    pub fn new() -> Self {
        Self {
            supported: vec![
                AssetType::Animation,
                AssetType::Config,
                AssetType::Material,
                AssetType::Prefab,
                AssetType::Scene,
                AssetType::Von,
            ],
        }
    }
}

impl AssetCompiler for VonAssetCompiler {
    fn name(&self) -> &str {
        "VonAssetCompiler"
    }

    fn supported_types(&self) -> &[AssetType] {
        &self.supported
    }

    fn compile(&self, entry: &AssetEntry, _context: &CompileContext) -> Result<(), String> {
        let source = std::fs::read_to_string(&entry.path)
            .map_err(|e| format!("Failed to read VON asset {}: {}", entry.path.display(), e))?;

        let compiler = gg_meta::VonCompiler::new();
        compiler.compile(&source).map_err(|e| format!("VON compilation failed for {}: {}", entry.guid, e))?;

        Ok(())
    }

    fn clone_box(&self) -> Box<dyn AssetCompiler> {
        Box::new(Self { supported: self.supported.clone() })
    }
}

impl Default for VonAssetCompiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Meta 资产编译器
///
/// 使用 gg-meta 的 MetaGenerator 对 .meta 文件进行验证和处理
pub struct MetaAssetCompiler;

impl AssetCompiler for MetaAssetCompiler {
    fn name(&self) -> &str {
        "MetaAssetCompiler"
    }

    fn supported_types(&self) -> &[AssetType] {
        static TYPES: [AssetType; 1] = [AssetType::Meta];
        &TYPES
    }

    fn compile(&self, entry: &AssetEntry, _context: &CompileContext) -> Result<(), String> {
        let content = std::fs::read_to_string(&entry.path)
            .map_err(|e| format!("Failed to read meta file {}: {}", entry.path.display(), e))?;

        let _meta: gg_meta::MetaFile =
            toml::from_str(&content).map_err(|e| format!("Meta validation failed for {}: {}", entry.guid, e))?;

        Ok(())
    }

    fn clone_box(&self) -> Box<dyn AssetCompiler> {
        Box::new(Self)
    }
}

#[cfg(feature = "gg-schema")]
mod schema_impl {
    use std::sync::Mutex;

    use crate::{
        compilers::SchemaAssetCompiler,
        pipeline::{AssetCompiler, CompileContext},
        types::{AssetEntry, AssetType},
    };

    /// Schema 资产编译器
    ///
    /// 使用 gg-schema 的 SchemaCompiler 编译 .schema 文件，
    /// 产出 SchemaIR、Valkyrie 类型绑定和数据库迁移文件
    impl SchemaAssetCompiler {
        /// 创建一个新的 Schema 资产编译器
        pub fn new() -> Self {
            Self { inner: Mutex::new(gg_schema::SchemaCompiler::new()) }
        }
    }

    impl AssetCompiler for SchemaAssetCompiler {
        fn name(&self) -> &str {
            "SchemaAssetCompiler"
        }

        fn supported_types(&self) -> &[AssetType] {
            static TYPES: [AssetType; 1] = [AssetType::Schema];
            &TYPES
        }

        fn compile(&self, entry: &AssetEntry, _context: &CompileContext) -> Result<(), String> {
            let source = std::fs::read_to_string(&entry.path)
                .map_err(|e| format!("Failed to read schema file {}: {}", entry.path.display(), e))?;

            let module_name = entry.path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");

            let compiler = self.inner.lock().map_err(|e| format!("Lock error: {}", e))?;
            compiler
                .compile(&source, module_name)
                .map_err(|e| format!("Schema compilation failed for {}: {}", entry.guid, e))?;

            Ok(())
        }

        fn clone_box(&self) -> Box<dyn AssetCompiler> {
            Box::new(Self::new())
        }
    }

    impl Default for SchemaAssetCompiler {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(not(feature = "gg-schema"))]
mod schema_impl {
    use crate::{
        compilers::SchemaAssetCompiler,
        pipeline::{AssetCompiler, CompileContext},
        types::{AssetEntry, AssetType},
    };

    impl SchemaAssetCompiler {
        /// 创建一个新的 Schema 资产编译器（stub，需启用 gg-schema feature）
        pub fn new() -> Self {
            Self { inner: std::sync::Mutex::new(()) }
        }
    }

    impl AssetCompiler for SchemaAssetCompiler {
        fn name(&self) -> &str {
            "SchemaAssetCompiler"
        }

        fn supported_types(&self) -> &[AssetType] {
            static TYPES: [AssetType; 1] = [AssetType::Schema];
            &TYPES
        }

        fn compile(&self, _entry: &AssetEntry, _context: &CompileContext) -> Result<(), String> {
            Err("Schema compiler not available: enable gg-schema feature".to_string())
        }

        fn clone_box(&self) -> Box<dyn AssetCompiler> {
            Box::new(Self::new())
        }
    }

    impl Default for SchemaAssetCompiler {
        fn default() -> Self {
            Self::new()
        }
    }
}

/// Schema 资产编译器
///
/// 使用 gg-schema 的 SchemaCompiler 编译 .schema 文件，
/// 产出 SchemaIR、Valkyrie 类型绑定和数据库迁移文件
pub struct SchemaAssetCompiler {
    /// 内部编译器
    #[cfg(feature = "gg-schema")]
    inner: Mutex<gg_schema::SchemaCompiler>,
    #[cfg(not(feature = "gg-schema"))]
    #[allow(dead_code)]
    inner: Mutex<()>,
}

#[cfg(feature = "gg-script")]
mod script_impl {
    use std::sync::Mutex;

    use crate::{
        compilers::ScriptAssetCompiler,
        pipeline::{AssetCompiler, CompileContext},
        types::{AssetEntry, AssetType},
    };

    /// Script 资产编译器
    ///
    /// 使用 gg-script 的 ScriptCompiler 编译 Valkyrie 脚本文件，
    /// 产出 BytecodeModule 字节码产物
    impl ScriptAssetCompiler {
        /// 创建一个新的 Script 资产编译器
        pub fn new() -> Self {
            Self { inner: Mutex::new(gg_script::ScriptCompiler::new()) }
        }
    }

    impl AssetCompiler for ScriptAssetCompiler {
        fn name(&self) -> &str {
            "ScriptAssetCompiler"
        }

        fn supported_types(&self) -> &[AssetType] {
            static TYPES: [AssetType; 1] = [AssetType::Script];
            &TYPES
        }

        fn compile(&self, entry: &AssetEntry, _context: &CompileContext) -> Result<(), String> {
            let source = std::fs::read_to_string(&entry.path)
                .map_err(|e| format!("Failed to read script file {}: {}", entry.path.display(), e))?;

            let module_name = entry.path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");

            let compiler = self.inner.lock().map_err(|e| format!("Lock error: {}", e))?;
            compiler
                .compile(&source, module_name)
                .map_err(|e| format!("Script compilation failed for {}: {}", entry.guid, e))?;

            Ok(())
        }

        fn clone_box(&self) -> Box<dyn AssetCompiler> {
            Box::new(Self::new())
        }
    }

    impl Default for ScriptAssetCompiler {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(not(feature = "gg-script"))]
mod script_impl {
    use crate::{
        compilers::ScriptAssetCompiler,
        pipeline::{AssetCompiler, CompileContext},
        types::{AssetEntry, AssetType},
    };

    impl ScriptAssetCompiler {
        /// 创建一个新的 Script 资产编译器（stub，需启用 gg-script feature）
        pub fn new() -> Self {
            Self { inner: std::sync::Mutex::new(()) }
        }
    }

    impl AssetCompiler for ScriptAssetCompiler {
        fn name(&self) -> &str {
            "ScriptAssetCompiler"
        }

        fn supported_types(&self) -> &[AssetType] {
            static TYPES: [AssetType; 1] = [AssetType::Script];
            &TYPES
        }

        fn compile(&self, _entry: &AssetEntry, _context: &CompileContext) -> Result<(), String> {
            Err("Script compiler not available: enable gg-script feature".to_string())
        }

        fn clone_box(&self) -> Box<dyn AssetCompiler> {
            Box::new(Self::new())
        }
    }

    impl Default for ScriptAssetCompiler {
        fn default() -> Self {
            Self::new()
        }
    }
}

/// Script 资产编译器
///
/// 使用 gg-script 的 ScriptCompiler 编译 Valkyrie 脚本文件，
/// 产出 BytecodeModule 字节码产物
pub struct ScriptAssetCompiler {
    /// 内部编译器
    #[cfg(feature = "gg-script")]
    inner: Mutex<gg_script::ScriptCompiler>,
    #[cfg(not(feature = "gg-script"))]
    #[allow(dead_code)]
    inner: Mutex<()>,
}

#[cfg(feature = "gg-compiler-shader")]
mod shader_impl {
    use std::sync::Mutex;

    use crate::{
        compilers::ShaderAssetCompiler,
        pipeline::{AssetCompiler, CompileContext},
        types::{AssetEntry, AssetType},
    };

    /// Shader 资产编译器
    ///
    /// 使用 gg-compiler-shader 的 ShaderCompiler 编译 GG Shader (gs) 格式文件，
    /// 产出 ShaderArtifact 产物。注意：不使用 WGSL 作为源格式
    impl ShaderAssetCompiler {
        /// 创建一个新的 Shader 资产编译器
        pub fn new() -> Self {
            Self { inner: Mutex::new(gg_compiler_shader::ShaderCompiler::new()) }
        }
    }

    impl AssetCompiler for ShaderAssetCompiler {
        fn name(&self) -> &str {
            "ShaderAssetCompiler"
        }

        fn supported_types(&self) -> &[AssetType] {
            static TYPES: [AssetType; 1] = [AssetType::Shader];
            &TYPES
        }

        fn compile(&self, entry: &AssetEntry, _context: &CompileContext) -> Result<(), String> {
            let source = std::fs::read_to_string(&entry.path)
                .map_err(|e| format!("Failed to read shader file {}: {}", entry.path.display(), e))?;

            let compiler = self.inner.lock().map_err(|e| format!("Lock error: {}", e))?;
            compiler.compile(&source).map_err(|e| format!("Shader compilation failed for {}: {}", entry.guid, e))?;

            Ok(())
        }

        fn clone_box(&self) -> Box<dyn AssetCompiler> {
            Box::new(Self::new())
        }
    }

    impl Default for ShaderAssetCompiler {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(not(feature = "gg-compiler-shader"))]
mod shader_impl {
    use crate::{
        compilers::ShaderAssetCompiler,
        pipeline::{AssetCompiler, CompileContext},
        types::{AssetEntry, AssetType},
    };

    impl ShaderAssetCompiler {
        /// 创建一个新的 Shader 资产编译器（stub，需启用 gg-compiler-shader feature）
        pub fn new() -> Self {
            Self { inner: std::sync::Mutex::new(()) }
        }
    }

    impl AssetCompiler for ShaderAssetCompiler {
        fn name(&self) -> &str {
            "ShaderAssetCompiler"
        }

        fn supported_types(&self) -> &[AssetType] {
            static TYPES: [AssetType; 1] = [AssetType::Shader];
            &TYPES
        }

        fn compile(&self, _entry: &AssetEntry, _context: &CompileContext) -> Result<(), String> {
            Err("Shader compiler not available: enable gg-compiler-shader feature".to_string())
        }

        fn clone_box(&self) -> Box<dyn AssetCompiler> {
            Box::new(Self::new())
        }
    }

    impl Default for ShaderAssetCompiler {
        fn default() -> Self {
            Self::new()
        }
    }
}

/// Shader 资产编译器
///
/// 使用 gg-compiler-shader 的 ShaderCompiler 编译 GG Shader (gs) 格式文件，
/// 产出 ShaderArtifact 产物。注意：不使用 WGSL 作为源格式
pub struct ShaderAssetCompiler {
    /// 内部编译器
    #[cfg(feature = "gg-compiler-shader")]
    inner: Mutex<gg_compiler_shader::ShaderCompiler>,
    #[cfg(not(feature = "gg-compiler-shader"))]
    #[allow(dead_code)]
    inner: Mutex<()>,
}

#[cfg(feature = "gg-compiler-widget")]
mod widget_impl {
    use std::sync::Mutex;

    use crate::{
        compilers::WidgetAssetCompiler,
        pipeline::{AssetCompiler, CompileContext},
        types::{AssetEntry, AssetType},
    };

    /// Widget 资产编译器
    ///
    /// 使用 gg-compiler-widget 的 WidgetCompiler 编译 .widget 单文件组件，
    /// 解析 template/script/style 三部分，产出 WidgetArtifact 二进制产物
    impl WidgetAssetCompiler {
        /// 创建一个新的 Widget 资产编译器
        pub fn new() -> Self {
            Self { inner: Mutex::new(gg_compiler_widget::WidgetCompiler::new()) }
        }
    }

    impl AssetCompiler for WidgetAssetCompiler {
        fn name(&self) -> &str {
            "WidgetAssetCompiler"
        }

        fn supported_types(&self) -> &[AssetType] {
            static TYPES: [AssetType; 1] = [AssetType::Widget];
            &TYPES
        }

        fn compile(&self, entry: &AssetEntry, _context: &CompileContext) -> Result<(), String> {
            let source = std::fs::read_to_string(&entry.path)
                .map_err(|e| format!("Failed to read widget file {}: {}", entry.path.display(), e))?;

            let module_name = entry.path.file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");

            let mut compiler = self.inner.lock().map_err(|e| format!("Lock error: {}", e))?;
            compiler
                .compile(&source, module_name)
                .map_err(|e| format!("Widget compilation failed for {}: {}", entry.guid, e))?;

            Ok(())
        }

        fn clone_box(&self) -> Box<dyn AssetCompiler> {
            Box::new(Self::new())
        }
    }

    impl Default for WidgetAssetCompiler {
        fn default() -> Self {
            Self::new()
        }
    }
}

#[cfg(not(feature = "gg-compiler-widget"))]
mod widget_impl {
    use crate::{
        compilers::WidgetAssetCompiler,
        pipeline::{AssetCompiler, CompileContext},
        types::{AssetEntry, AssetType},
    };

    impl WidgetAssetCompiler {
        /// 创建一个新的 Widget 资产编译器（stub，需启用 gg-compiler-widget feature）
        pub fn new() -> Self {
            Self { inner: std::sync::Mutex::new(()) }
        }
    }

    impl AssetCompiler for WidgetAssetCompiler {
        fn name(&self) -> &str {
            "WidgetAssetCompiler"
        }

        fn supported_types(&self) -> &[AssetType] {
            static TYPES: [AssetType; 1] = [AssetType::Widget];
            &TYPES
        }

        fn compile(&self, _entry: &AssetEntry, _context: &CompileContext) -> Result<(), String> {
            Err("Widget compiler not available: enable gg-compiler-widget feature".to_string())
        }

        fn clone_box(&self) -> Box<dyn AssetCompiler> {
            Box::new(Self::new())
        }
    }

    impl Default for WidgetAssetCompiler {
        fn default() -> Self {
            Self::new()
        }
    }
}

/// Widget 资产编译器
///
/// 使用 gg-compiler-widget 的 WidgetCompiler 编译 .widget 单文件组件，
/// 解析 template/script/style 三部分，产出 WidgetArtifact 二进制产物
pub struct WidgetAssetCompiler {
    /// 内部编译器
    #[cfg(feature = "gg-compiler-widget")]
    inner: Mutex<gg_compiler_widget::WidgetCompiler>,
    #[cfg(not(feature = "gg-compiler-widget"))]
    #[allow(dead_code)]
    inner: Mutex<()>,
}
