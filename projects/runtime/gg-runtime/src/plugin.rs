use gg_bytecode::BytecodeModule;
use gg_core::{GError, GResult};
use gg_script::ScriptLoader;
use std::path::Path;

/// 插件标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PluginId(
    /// 插件唯一标识
    pub u64,
);

/// 插件配置
#[derive(Debug, Clone)]
pub struct PluginConfig {
    /// 插件名称
    pub name: String,
    /// 插件版本
    pub version: String,
    /// 插件描述
    pub description: String,
    /// 依赖插件列表
    pub dependencies: Vec<String>,
    /// 脚本入口函数
    pub entry_function: String,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            name: "unnamed".to_string(),
            version: "1.0.0".to_string(),
            description: "".to_string(),
            dependencies: Vec::new(),
            entry_function: "init".to_string(),
        }
    }
}

impl PluginConfig {
    /// 创建新的插件配置
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), ..Default::default() }
    }
}

/// 插件模块
#[derive(Debug, Clone)]
pub struct PluginModule {
    /// 插件配置
    pub config: PluginConfig,
    /// 字节码模块
    pub bytecode: BytecodeModule,
}

/// 插件管理器
///
/// 管理插件的加载、初始化和执行
pub struct PluginManager {
    /// 插件加载器
    loader: ScriptLoader,
    /// 已加载的插件
    plugins: Vec<PluginModule>,
    /// 插件名称到索引的映射
    plugin_map: std::collections::HashMap<String, usize>,
}

impl PluginManager {
    /// 创建新的插件管理器
    pub fn new() -> Self {
        Self { loader: ScriptLoader::new(), plugins: Vec::new(), plugin_map: std::collections::HashMap::new() }
    }

    /// 加载插件
    ///
    /// 从文件加载 Valkyrie 脚本并编译为插件模块
    pub fn load_plugin(&mut self, path: &Path, config: PluginConfig) -> GResult<PluginId> {
        let bytecode = self.loader.load_file(path)?;

        let plugin = PluginModule { config, bytecode };

        let plugin_id = PluginId(self.plugins.len() as u64);
        self.plugin_map.insert(plugin.config.name.clone(), self.plugins.len());
        self.plugins.push(plugin);

        Ok(plugin_id)
    }

    /// 从字符串加载插件
    ///
    /// 从字符串加载 Valkyrie 脚本并编译为插件模块
    pub fn load_plugin_from_string(&mut self, source: &str, config: PluginConfig) -> GResult<PluginId> {
        let bytecode = self.loader.load_string(source, &config.name)?;

        let plugin = PluginModule { config, bytecode };

        let plugin_id = PluginId(self.plugins.len() as u64);
        self.plugin_map.insert(plugin.config.name.clone(), self.plugins.len());
        self.plugins.push(plugin);

        Ok(plugin_id)
    }

    /// 获取插件配置
    pub fn get_plugin_config(&self, plugin_id: PluginId) -> Option<&PluginConfig> {
        self.plugins.get(plugin_id.0 as usize).map(|p| &p.config)
    }

    /// 获取插件字节码模块
    pub fn get_plugin_module(&self, plugin_id: PluginId) -> Option<&BytecodeModule> {
        self.plugins.get(plugin_id.0 as usize).map(|p| &p.bytecode)
    }

    /// 根据名称获取插件 ID
    pub fn get_plugin_id(&self, name: &str) -> Option<PluginId> {
        self.plugin_map.get(name).map(|&idx| PluginId(idx as u64))
    }

    /// 获取所有插件
    pub fn plugins(&self) -> &[PluginModule] {
        &self.plugins
    }

    /// 检查插件是否已加载
    pub fn has_plugin(&self, name: &str) -> bool {
        self.plugin_map.contains_key(name)
    }

    /// 获取插件数量
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
