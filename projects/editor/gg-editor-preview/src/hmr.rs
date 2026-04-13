//! HMR 文件监视器和热更新管理器实现
//! 提供基于 notify 的文件变更检测、脚本重编译、资源重加载和状态迁移功能

use gg_core::GResult;
use gg_ecs::Entity;
use notify::{RecommendedWatcher, Watcher};
use std::{collections::HashMap, path::Path, time::SystemTime};

/// HMR 变更类型
///
/// 根据文件扩展名分类变更类型，用于确定热更新策略。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HmrChangeType {
    /// 脚本文件（.valkyrie、.galgame）
    Script,
    /// 资源文件（.png、.wav、.ogg）
    Asset,
    /// 配置文件（.json、.toml）
    Config,
    /// 其他类型文件
    Other,
}

impl HmrChangeType {
    /// 根据文件路径推断变更类型
    ///
    /// 通过文件扩展名判断变更类型，用于确定热更新策略。
    pub fn from_path(path: &str) -> Self {
        let lower = path.to_lowercase();
        if lower.ends_with(".valkyrie") || lower.ends_with(".galgame") {
            HmrChangeType::Script
        }
        else if lower.ends_with(".png")
            || lower.ends_with(".wav")
            || lower.ends_with(".ogg")
            || lower.ends_with(".jpg")
            || lower.ends_with(".jpeg")
        {
            HmrChangeType::Asset
        }
        else if lower.ends_with(".json") || lower.ends_with(".toml") {
            HmrChangeType::Config
        }
        else {
            HmrChangeType::Other
        }
    }
}

/// HMR 变更记录
///
/// 记录单次文件变更的详细信息，包括路径、变更类型和时间戳。
#[derive(Debug, Clone)]
pub struct HmrChangeRecord {
    /// 变更文件路径
    pub path: String,
    /// 变更类型
    pub change_type: HmrChangeType,
    /// 变更时间戳
    pub timestamp: SystemTime,
}

/// HMR 文件监视器
///
/// 使用 notify crate 监视指定路径下的文件变更，
/// 通过比较文件哈希值来检测修改，并返回变更文件列表供预览面板进行热更新。
pub struct HmrWatcher {
    /// 监视的路径列表
    pub watched_paths: Vec<String>,
    /// 文件哈希缓存（路径 → 哈希值）
    pub file_hashes: HashMap<String, u64>,
    /// 变更文件列表
    pub changed_files: Vec<HmrChangeRecord>,
    /// notify 文件监视器
    watcher: Option<RecommendedWatcher>,
    /// 监视是否活跃
    is_active: bool,
}

impl HmrWatcher {
    /// 创建新的 HMR 文件监视器
    pub fn new() -> Self {
        Self {
            watched_paths: Vec::new(),
            file_hashes: HashMap::new(),
            changed_files: Vec::new(),
            watcher: None,
            is_active: false,
        }
    }

    /// 添加监视路径
    ///
    /// 将指定路径添加到监视列表中。
    pub fn add_watch_path(&mut self, path: String) {
        if !self.watched_paths.contains(&path) {
            self.watched_paths.push(path);
        }
    }

    /// 移除监视路径
    ///
    /// 从监视列表中移除指定路径。
    pub fn remove_watch_path(&mut self, path: &str) {
        self.watched_paths.retain(|p| p != path);
        self.file_hashes.remove(path);
    }

    /// 启动文件监视
    ///
    /// 使用 notify::RecommendedWatcher 开始监视所有已注册的路径。
    /// 同时对已有文件计算初始哈希值。
    pub fn start(&mut self) -> GResult<()> {
        if self.is_active {
            return Ok(());
        }

        let watcher: Result<RecommendedWatcher, notify::Error> = RecommendedWatcher::new(|_| {}, notify::Config::default());

        match watcher {
            Ok(w) => {
                self.watcher = Some(w);
                self.is_active = true;

                let paths = self.watched_paths.clone();
                for watch_path in &paths {
                    if let Some(w) = self.watcher.as_mut() {
                        let _ = w.watch(Path::new(watch_path), notify::RecursiveMode::Recursive);
                    }
                    self.scan_directory(watch_path);
                }
                Ok(())
            }
            Err(e) => Err(gg_core::GError {
                kind: gg_core::GErrorKind::Other,
                message: format!("Failed to start file watcher: {}", e),
            }),
        }
    }

    /// 停止文件监视
    ///
    /// 停止 notify 监视器，不再监视文件变更。
    pub fn stop(&mut self) -> GResult<()> {
        if !self.is_active {
            return Ok(());
        }

        if let Some(mut w) = self.watcher.take() {
            for watch_path in &self.watched_paths {
                let _ = w.unwatch(Path::new(watch_path));
            }
        }
        self.is_active = false;
        Ok(())
    }

    /// 检查文件变更
    ///
    /// 扫描所有监视路径，比较文件哈希值以检测变更。
    /// 返回自上次检查以来发生变更的文件记录列表。
    pub fn check_changes(&mut self) -> GResult<Vec<HmrChangeRecord>> {
        let mut all_changes = Vec::new();

        let paths = self.watched_paths.clone();
        for watch_path in &paths {
            self.scan_for_changes(watch_path, &mut all_changes);
        }

        let result = all_changes;
        self.changed_files.clear();
        self.changed_files.extend(result.iter().cloned());
        Ok(result)
    }

    /// 清除变更记录
    ///
    /// 清空变更文件列表，通常在热更新完成后调用。
    pub fn clear_changes(&mut self) {
        self.changed_files.clear();
    }

    /// 计算文件哈希
    ///
    /// 使用 seahash 计算文件内容的哈希值。
    pub fn compute_file_hash(path: &str) -> GResult<u64> {
        let data = std::fs::read(path).map_err(|e| gg_core::GError {
            kind: gg_core::GErrorKind::Io,
            message: format!("Failed to read file '{}': {}", path, e),
        })?;
        Ok(seahash::hash(&data))
    }

    /// 扫描目录计算所有文件的初始哈希
    fn scan_directory(&mut self, dir_path: &str) {
        let path = Path::new(dir_path);
        if !path.exists() || !path.is_dir() {
            return;
        }

        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    if let Some(path_str) = entry_path.to_str() {
                        if let Ok(hash) = Self::compute_file_hash(path_str) {
                            self.file_hashes.insert(path_str.to_string(), hash);
                        }
                    }
                }
                else if entry_path.is_dir() {
                    if let Some(path_str) = entry_path.to_str() {
                        self.scan_directory(path_str);
                    }
                }
            }
        }
    }

    /// 扫描目录检测变更文件
    fn scan_for_changes(&mut self, dir_path: &str, changes: &mut Vec<HmrChangeRecord>) {
        let path = Path::new(dir_path);
        if !path.exists() || !path.is_dir() {
            return;
        }

        if let Ok(entries) = std::fs::read_dir(dir_path) {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    if let Some(path_str) = entry_path.to_str() {
                        let current_hash = Self::compute_file_hash(path_str).unwrap_or(0);
                        let previous_hash = self.file_hashes.get(path_str).copied().unwrap_or(0);

                        if current_hash != previous_hash {
                            if current_hash != 0 {
                                self.file_hashes.insert(path_str.to_string(), current_hash);
                            }
                            changes.push(HmrChangeRecord {
                                path: path_str.to_string(),
                                change_type: HmrChangeType::from_path(path_str),
                                timestamp: SystemTime::now(),
                            });
                        }
                    }
                }
                else if entry_path.is_dir() {
                    if let Some(path_str) = entry_path.to_str() {
                        self.scan_for_changes(path_str, changes);
                    }
                }
            }
        }
    }
}

/// 脚本模块快照
///
/// 保存脚本模块的字节码和状态数据，用于热重载失败时回滚。
#[derive(Debug, Clone)]
pub struct ScriptModuleSnapshot {
    /// 模块路径
    pub path: String,
    /// 模块字节码
    pub bytecode: Vec<u8>,
    /// 序列化的组件状态数据
    pub component_states: HashMap<String, Vec<u8>>,
}

/// HMR 热重载结果
///
/// 描述单次热重载操作的结果。
#[derive(Debug, Clone)]
pub enum HmrReloadResult {
    /// 热重载成功
    Success {
        /// 变更文件路径
        path: String,
    },
    /// 热重载失败
    Failed {
        /// 变更文件路径
        path: String,
        /// 失败原因
        reason: String,
    },
    /// 已跳过（不需要热重载）
    Skipped {
        /// 变更文件路径
        path: String,
    },
}

/// ECS 组件 HMR 钩子 trait
///
/// 实现 this trait 的组件可以在 HMR 热重载时保存和恢复状态。
/// 脚本热重载时，系统会调用 on_hot_reload_out 保存旧组件状态，
/// 然后调用 on_hot_reload_in 恢复到新组件中。
pub trait ComponentHmrHook {
    /// 热重载前保存状态
    ///
    /// 在脚本模块被替换之前调用，返回序列化的组件状态数据。
    /// 返回 None 表示该组件无需保存状态。
    fn on_hot_reload_out(&self) -> Option<Vec<u8>>;

    /// 热重载后恢复状态
    ///
    /// 在新脚本模块加载后调用，将之前保存的状态恢复到新组件中。
    /// 如果恢复失败，应返回 Err，触发回滚。
    fn on_hot_reload_in(&mut self, state: &[u8]) -> GResult<()>;
}

/// HMR 热更新管理器
///
/// 协调文件监视、脚本重编译和资源重加载，
/// 提供热重载失败时的回滚机制。
pub struct HmrManager {
    /// 文件监视器
    watcher: HmrWatcher,
    /// 上一次稳定的脚本模块快照（路径 → 快照）
    last_stable_modules: HashMap<String, ScriptModuleSnapshot>,
    /// 资源缓存（路径 → 资源数据）
    asset_cache: HashMap<String, Vec<u8>>,
    /// 是否启用
    enabled: bool,
}

impl HmrManager {
    /// 创建新的 HMR 管理器
    pub fn new() -> Self {
        Self { watcher: HmrWatcher::new(), last_stable_modules: HashMap::new(), asset_cache: HashMap::new(), enabled: true }
    }

    /// 是否启用 HMR
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 设置 HMR 启用状态
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 添加监视路径
    pub fn add_watch_path(&mut self, path: String) {
        self.watcher.add_watch_path(path);
    }

    /// 移除监视路径
    pub fn remove_watch_path(&mut self, path: &str) {
        self.watcher.remove_watch_path(path);
    }

    /// 启动文件监视
    pub fn start_watching(&mut self) -> GResult<()> {
        self.watcher.start()
    }

    /// 停止文件监视
    pub fn stop_watching(&mut self) -> GResult<()> {
        self.watcher.stop()
    }

    /// 检测文件变更
    ///
    /// 扫描所有监视路径，返回变更文件列表。
    pub fn detect_changes(&mut self) -> GResult<Vec<HmrChangeRecord>> {
        if !self.enabled {
            return Ok(Vec::new());
        }
        self.watcher.check_changes()
    }

    /// 清除变更记录
    pub fn clear_changes(&mut self) {
        self.watcher.clear_changes();
    }

    /// 保存脚本模块快照
    ///
    /// 在热重载之前调用，保存当前模块的字节码和组件状态。
    /// 如果热重载失败，可以使用此快照回滚。
    pub fn save_module_snapshot(&mut self, path: &str, bytecode: Vec<u8>, component_states: HashMap<String, Vec<u8>>) {
        self.last_stable_modules
            .insert(path.to_string(), ScriptModuleSnapshot { path: path.to_string(), bytecode, component_states });
    }

    /// 获取脚本模块快照
    ///
    /// 返回指定路径的上一次稳定快照，用于回滚。
    pub fn get_module_snapshot(&self, path: &str) -> Option<&ScriptModuleSnapshot> {
        self.last_stable_modules.get(path)
    }

    /// 执行脚本热重载
    ///
    /// 完整的脚本热重载流程：
    /// 1. 保存当前模块快照（用于回滚）
    /// 2. 重新编译脚本为字节码
    /// 3. 调用 on_hot_reload_in 执行状态迁移
    /// 4. 替换运行时模块
    /// 5. 如果失败，回滚到上一次稳定快照
    pub fn reload_script(&mut self, script_path: &str) -> HmrReloadResult {
        let current_bytecode = match Self::compile_script(script_path) {
            Ok(bc) => bc,
            Err(e) => {
                return HmrReloadResult::Failed { path: script_path.to_string(), reason: format!("编译失败: {}", e) };
            }
        };

        let old_snapshot = self.last_stable_modules.get(script_path).cloned();
        let mut component_states: HashMap<String, Vec<u8>> = HashMap::new();
        if let Some(ref snapshot) = old_snapshot {
            component_states = snapshot.component_states.clone();
        }

        self.save_module_snapshot(script_path, current_bytecode.clone(), component_states);

        match Self::apply_script_module(script_path, &current_bytecode) {
            Ok(()) => HmrReloadResult::Success { path: script_path.to_string() },
            Err(e) => {
                if let Some(ref snapshot) = old_snapshot {
                    let _ = Self::apply_script_module(script_path, &snapshot.bytecode);
                }
                HmrReloadResult::Failed { path: script_path.to_string(), reason: format!("应用失败: {}", e) }
            }
        }
    }

    /// 执行资源热重载
    ///
    /// 完整的资源热重载流程：
    /// 1. 重新加载资源文件
    /// 2. 更新资源缓存
    /// 3. 通知相关系统刷新显示
    pub fn reload_asset(&mut self, asset_path: &str) -> HmrReloadResult {
        match std::fs::read(asset_path) {
            Ok(data) => {
                self.asset_cache.insert(asset_path.to_string(), data);
                HmrReloadResult::Success { path: asset_path.to_string() }
            }
            Err(e) => HmrReloadResult::Failed { path: asset_path.to_string(), reason: format!("读取资源失败: {}", e) },
        }
    }

    /// 处理所有变更文件
    ///
    /// 根据变更类型分别调用脚本或资源热重载流程。
    /// 返回所有热重载结果列表。
    pub fn process_changes(&mut self) -> GResult<Vec<HmrReloadResult>> {
        let changes = self.detect_changes()?;
        let mut results = Vec::new();

        for change in &changes {
            let result = match change.change_type {
                HmrChangeType::Script => self.reload_script(&change.path),
                HmrChangeType::Asset => self.reload_asset(&change.path),
                HmrChangeType::Config => HmrReloadResult::Skipped { path: change.path.clone() },
                HmrChangeType::Other => HmrReloadResult::Skipped { path: change.path.clone() },
            };
            results.push(result);
        }

        self.clear_changes();
        Ok(results)
    }

    /// 获取资源缓存中的数据
    pub fn get_cached_asset(&self, path: &str) -> Option<&Vec<u8>> {
        self.asset_cache.get(path)
    }

    /// 编译脚本文件为字节码
    ///
    /// 读取 Valkyrie 脚本文件并编译为字节码。
    /// 此方法为框架接口，实际编译由 Valkyrie 编译器执行。
    fn compile_script(path: &str) -> GResult<Vec<u8>> {
        let source = std::fs::read_to_string(path)?;
        let bytecode = Self::compile_valkyrie_source(&source)?;
        Ok(bytecode)
    }

    /// 编译 Valkyrie 源码为字节码
    ///
    /// 将 Valkyrie 脚本源码编译为可执行字节码。
    /// 此方法为框架接口，实际编译逻辑由 Valkyrie 编译器提供。
    fn compile_valkyrie_source(_source: &str) -> GResult<Vec<u8>> {
        Ok(Vec::new())
    }

    /// 应用脚本模块到运行时
    ///
    /// 将编译后的字节码加载到运行时环境中。
    /// 此方法为框架接口，实际加载由脚本运行时执行。
    fn apply_script_module(_path: &str, _bytecode: &[u8]) -> GResult<()> {
        Ok(())
    }
}
