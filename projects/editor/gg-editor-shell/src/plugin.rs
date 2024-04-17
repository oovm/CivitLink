//! 编辑器插件系统
//!
//! 提供插件的热加载/卸载支持，包括插件状态管理、依赖拓扑排序和卸载保护。

use std::{collections::HashMap, path::Path};

use crate::{
    context::EditorContext,
    dynamic_loader::{DynamicPluginLoader, PluginManifest},
};
use gg_core::{GError, GErrorKind, GResult};

/// 插件状态枚举
///
/// 描述插件在生命周期中所处的状态。
/// 由于 `Error` 变体包含 `String`，本类型无法派生 `Copy`。
#[derive(Clone, Debug, PartialEq)]
pub enum PluginState {
    /// 插件已卸载
    Unloaded,
    /// 插件已加载但尚未激活
    Loaded,
    /// 插件已激活并正在运行
    Active,
    /// 插件处于错误状态，包含错误描述
    Error(String),
}

/// 插件描述符
///
/// 包含插件的元数据信息，如名称、版本和依赖列表。
#[derive(Clone, Debug)]
pub struct PluginDescriptor {
    /// 插件名称
    pub name: String,
    /// 插件版本
    pub version: String,
    /// 插件依赖列表
    pub dependencies: Vec<String>,
}

/// 编辑器插件 trait
///
/// 插件可以在编辑器启动时初始化、在关闭时清理，
/// 通过 `EditorContext` 访问服务注册表、命令管理器和事件总线。
/// 新增的方法均提供默认实现，确保现有实现无需修改即可编译。
pub trait EditorPlugin {
    /// 插件名称
    fn name(&self) -> &str;

    /// 初始化插件
    fn initialize(&mut self, context: &mut EditorContext);

    /// 关闭插件
    fn shutdown(&mut self, context: &mut EditorContext);

    /// 插件加载时调用
    fn on_load(&mut self, _context: &mut EditorContext) {}

    /// 插件卸载时调用
    fn on_unload(&mut self, _context: &mut EditorContext) {}

    /// 返回依赖的插件名称列表
    fn dependencies(&self) -> Vec<&str> {
        Vec::new()
    }

    /// 返回初始化优先级（数值越小越优先）
    fn priority(&self) -> u32 {
        100
    }
}

/// 插件条目
///
/// 持有插件实例、描述符和当前状态，作为插件管理器的基本存储单元。
pub struct PluginEntry {
    /// 插件实例
    pub plugin: Box<dyn EditorPlugin>,
    /// 插件描述符
    pub descriptor: PluginDescriptor,
    /// 插件当前状态
    pub state: PluginState,
}

/// 插件管理器
///
/// 管理编辑器插件的注册、激活、停用、卸载和重载，
/// 支持基于依赖关系的拓扑排序和被依赖插件的卸载保护。
pub struct PluginManager {
    /// 已注册的插件条目
    entries: Vec<PluginEntry>,
    /// 已加载的动态库句柄（插件名称 → 动态库）
    #[cfg(not(target_arch = "wasm32"))]
    loaded_libraries: HashMap<String, libloading::Library>,
}

impl PluginManager {
    /// 创建空的插件管理器
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            loaded_libraries: HashMap::new(),
        }
    }

    /// 注册插件
    ///
    /// 将插件添加到管理器中，初始状态设为 `Loaded`。
    pub fn register(&mut self, plugin: Box<dyn EditorPlugin>, descriptor: PluginDescriptor) {
        self.entries.push(PluginEntry { plugin, descriptor, state: PluginState::Loaded });
    }

    /// 激活插件
    ///
    /// 调用插件的 `on_load` 和 `initialize` 方法，将状态设为 `Active`。
    /// 仅当插件处于 `Loaded` 状态时才能激活。
    pub fn activate(&mut self, name: &str, context: &mut EditorContext) -> GResult<()> {
        let entry = self
            .entries
            .iter_mut()
            .find(|e| e.plugin.name() == name)
            .ok_or_else(|| GError::with_kind(GErrorKind::Plugin, &format!("插件 '{}' 不存在", name)))?;

        if entry.state != PluginState::Loaded {
            return Err(GError::with_kind(
                GErrorKind::Plugin,
                &format!("插件 '{}' 当前状态为 {:?}，无法激活，需要处于 Loaded 状态", name, entry.state),
            ));
        }

        entry.plugin.on_load(context);
        entry.plugin.initialize(context);
        entry.state = PluginState::Active;
        Ok(())
    }

    /// 停用插件
    ///
    /// 调用插件的 `shutdown` 和 `on_unload` 方法，将状态设为 `Loaded`。
    /// 仅当插件处于 `Active` 状态时才能停用。
    pub fn deactivate(&mut self, name: &str, context: &mut EditorContext) -> GResult<()> {
        let entry = self
            .entries
            .iter_mut()
            .find(|e| e.plugin.name() == name)
            .ok_or_else(|| GError::with_kind(GErrorKind::Plugin, &format!("插件 '{}' 不存在", name)))?;

        if entry.state != PluginState::Active {
            return Err(GError::with_kind(
                GErrorKind::Plugin,
                &format!("插件 '{}' 当前状态为 {:?}，无法停用，需要处于 Active 状态", name, entry.state),
            ));
        }

        entry.plugin.shutdown(context);
        entry.plugin.on_unload(context);
        entry.state = PluginState::Loaded;
        Ok(())
    }

    /// 卸载插件
    ///
    /// 先检查是否有其他活跃插件依赖该插件，若有则返回错误。
    /// 否则若插件处于活跃状态则调用 `shutdown` 和 `on_unload`，将状态设为 `Unloaded`。
    pub fn unload(&mut self, name: &str, context: &mut EditorContext) -> GResult<()> {
        let has_dependent = self.entries.iter().any(|e| {
            if e.state != PluginState::Active {
                return false;
            }
            e.plugin.dependencies().iter().any(|dep| *dep == name)
        });

        if has_dependent {
            return Err(GError::with_kind(GErrorKind::Plugin, &format!("插件 '{}' 被其他活跃插件依赖，无法卸载", name)));
        }

        let entry = self
            .entries
            .iter_mut()
            .find(|e| e.plugin.name() == name)
            .ok_or_else(|| GError::with_kind(GErrorKind::Plugin, &format!("插件 '{}' 不存在", name)))?;

        if entry.state == PluginState::Active {
            entry.plugin.shutdown(context);
            entry.plugin.on_unload(context);
        }

        entry.state = PluginState::Unloaded;
        Ok(())
    }

    /// 重载插件
    ///
    /// 先停用插件再重新激活，等效于 `deactivate` + `activate`。
    pub fn reload(&mut self, name: &str, context: &mut EditorContext) -> GResult<()> {
        self.deactivate(name, context)?;
        self.activate(name, context)
    }

    /// 查询插件状态
    ///
    /// 返回指定名称插件的当前状态，若插件不存在则返回 `None`。
    pub fn get_state(&self, name: &str) -> Option<PluginState> {
        self.entries.iter().find(|e| e.plugin.name() == name).map(|e| e.state.clone())
    }

    /// 按拓扑排序激活所有已注册插件
    ///
    /// 使用 Kahn 算法对插件依赖关系进行拓扑排序，
    /// 按优先级和依赖顺序依次激活所有处于 `Loaded` 状态的插件。
    /// 若检测到循环依赖则返回错误。
    pub fn activate_all(&mut self, context: &mut EditorContext) -> GResult<()> {
        let loaded_indices: Vec<usize> =
            self.entries.iter().enumerate().filter(|(_, e)| e.state == PluginState::Loaded).map(|(i, _)| i).collect();

        if loaded_indices.is_empty() {
            return Ok(());
        }

        let name_to_idx: HashMap<String, usize> =
            self.entries.iter().enumerate().map(|(i, e)| (e.plugin.name().to_string(), i)).collect();

        let mut in_degree: HashMap<usize, usize> = HashMap::new();
        let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();

        for &i in &loaded_indices {
            in_degree.insert(i, 0);
            adj.insert(i, Vec::new());
        }

        for &i in &loaded_indices {
            for dep in self.entries[i].plugin.dependencies() {
                if let Some(&dep_idx) = name_to_idx.get(dep) {
                    if in_degree.contains_key(&dep_idx) {
                        adj.get_mut(&dep_idx).unwrap().push(i);
                        *in_degree.get_mut(&i).unwrap() += 1;
                    }
                }
            }
        }

        let mut ready: Vec<usize> = loaded_indices.iter().filter(|&&i| in_degree[&i] == 0).copied().collect();
        ready.sort_by_key(|&i| self.entries[i].plugin.priority());

        let mut sorted: Vec<usize> = Vec::new();

        while !ready.is_empty() {
            let idx = ready.remove(0);
            sorted.push(idx);

            let mut new_ready: Vec<usize> = Vec::new();
            for &dependent in &adj[&idx] {
                let deg = in_degree.get_mut(&dependent).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    new_ready.push(dependent);
                }
            }
            ready.extend(new_ready);
            ready.sort_by_key(|&i| self.entries[i].plugin.priority());
        }

        if sorted.len() != loaded_indices.len() {
            return Err(GError::with_kind(GErrorKind::Plugin, "插件依赖关系中存在循环依赖，无法完成拓扑排序"));
        }

        let sorted_names: Vec<String> = sorted.into_iter().map(|i| self.entries[i].plugin.name().to_string()).collect();

        for name in sorted_names {
            self.activate(&name, context)?;
        }

        Ok(())
    }

    /// 按逆拓扑排序停用所有插件
    ///
    /// 按依赖关系的逆序停用所有活跃插件，确保被依赖的插件最后停用。
    pub fn deactivate_all(&mut self, context: &mut EditorContext) {
        let active_indices: Vec<usize> =
            self.entries.iter().enumerate().filter(|(_, e)| e.state == PluginState::Active).map(|(i, _)| i).collect();

        if active_indices.is_empty() {
            return;
        }

        let name_to_idx: HashMap<String, usize> =
            self.entries.iter().enumerate().map(|(i, e)| (e.plugin.name().to_string(), i)).collect();

        let mut in_degree: HashMap<usize, usize> = HashMap::new();
        let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();

        for &i in &active_indices {
            in_degree.insert(i, 0);
            adj.insert(i, Vec::new());
        }

        for &i in &active_indices {
            for dep in self.entries[i].plugin.dependencies() {
                if let Some(&dep_idx) = name_to_idx.get(dep) {
                    if in_degree.contains_key(&dep_idx) {
                        adj.get_mut(&dep_idx).unwrap().push(i);
                        *in_degree.get_mut(&i).unwrap() += 1;
                    }
                }
            }
        }

        let mut ready: Vec<usize> = active_indices.iter().filter(|&&i| in_degree[&i] == 0).copied().collect();

        let mut sorted: Vec<usize> = Vec::new();

        while !ready.is_empty() {
            let idx = ready.remove(0);
            sorted.push(idx);

            for &dependent in &adj[&idx] {
                let deg = in_degree.get_mut(&dependent).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    ready.push(dependent);
                }
            }
        }

        sorted.reverse();

        let sorted_names: Vec<String> = sorted.into_iter().map(|i| self.entries[i].plugin.name().to_string()).collect();

        for name in sorted_names {
            let _ = self.deactivate(&name, context);
        }
    }

    /// 迭代所有活跃插件
    pub fn iter_active(&self) -> impl Iterator<Item = &dyn EditorPlugin> {
        self.entries.iter().filter(|e| e.state == PluginState::Active).map(|e| &*e.plugin)
    }

    /// 对所有活跃插件执行可变回调
    ///
    /// 对每个处于 `Active` 状态的插件调用提供的回调函数，
    /// 避免返回可变引用时的生命周期问题。
    pub fn for_each_active_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut dyn EditorPlugin),
    {
        for entry in &mut self.entries {
            if entry.state == PluginState::Active {
                f(&mut *entry.plugin);
            }
        }
    }

    /// 按名称查找插件
    pub fn find(&self, name: &str) -> Option<&dyn EditorPlugin> {
        self.entries.iter().find(|e| e.plugin.name() == name).map(|e| &*e.plugin)
    }

    /// 按名称可变查找插件
    pub fn find_mut(&mut self, name: &str) -> Option<&mut (dyn EditorPlugin + '_)> {
        self.entries.iter_mut().find(|e| e.plugin.name() == name).map(move |e| &mut *e.plugin as &mut (dyn EditorPlugin + '_))
    }

    /// 从指定路径加载动态库插件
    ///
    /// 解析路径下的 `plugin.json` 清单文件，加载动态库并获取插件实例，
    /// 注册到管理器中并激活。
    #[cfg(not(target_arch = "wasm32"))]
    pub fn load_dynamic(&mut self, path: &Path, context: &mut EditorContext) -> GResult<()> {
        let manifest_path = path.join("plugin.json");
        let manifest = PluginManifest::from_file(&manifest_path)?;

        let library_path = path.join(&manifest.entry_library);
        let (library, plugin_ptr) = unsafe { DynamicPluginLoader::load(&library_path)? };

        let plugin: Box<dyn EditorPlugin> = unsafe { Box::from_raw(plugin_ptr as *mut dyn EditorPlugin) };

        let name = plugin.name().to_string();
        let descriptor = PluginDescriptor {
            name: name.clone(),
            version: manifest.version.clone(),
            dependencies: manifest.dependencies.clone(),
        };

        self.register(plugin, descriptor);
        self.loaded_libraries.insert(name.clone(), library);

        self.activate(&name, context)?;

        Ok(())
    }

    /// 卸载动态库插件
    ///
    /// 先停用插件，再释放动态库句柄。
    /// 若有其他活跃插件依赖该插件则返回错误。
    #[cfg(not(target_arch = "wasm32"))]
    pub fn unload_dynamic(&mut self, name: &str, context: &mut EditorContext) -> GResult<()> {
        self.unload(name, context)?;

        if let Some(library) = self.loaded_libraries.remove(name) {
            unsafe {
                let _ = DynamicPluginLoader::unload(library);
            }
        }

        Ok(())
    }
}
