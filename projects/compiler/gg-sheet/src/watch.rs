//! 配置表文件监听模块
//! 监听配置表目录的文件变更，自动触发增量编译

use std::{path::PathBuf, sync::mpsc, time::Duration};

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::{
    compiler::SheetCompiler,
    error::{SheetError, SheetResult},
};

/// 监听事件类型
#[derive(Debug)]
pub enum WatchEvent {
    /// 文件被创建
    Created(PathBuf),
    /// 文件被修改
    Modified(PathBuf),
}

/// 配置表文件监听器
/// 持有编译器实例，监听配置表目录变更后自动增量编译
pub struct SheetWatcher {
    /// 配置表编译器实例
    compiler: SheetCompiler,
}

impl SheetWatcher {
    /// 创建新的配置表文件监听器
    pub fn new(compiler: SheetCompiler) -> Self {
        Self { compiler }
    }

    /// 启动文件监听，持续运行直到用户按 Ctrl+C
    pub fn watch(&mut self) -> SheetResult<()> {
        let (tx, rx) = mpsc::channel();

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            },
            Config::default().with_poll_interval(Duration::from_secs(1)),
        )
        .map_err(|e| SheetError::Io {
            path: self.compiler.sheet_dir().to_path_buf(),
            message: format!("无法创建文件监听器: {}", e),
        })?;

        let watch_dir = self.compiler.sheet_dir().to_path_buf();
        watcher
            .watch(&watch_dir, RecursiveMode::Recursive)
            .map_err(|e| SheetError::Io { path: watch_dir.clone(), message: format!("无法监听目录: {}", e) })?;

        println!("正在监听配置表目录: {}", watch_dir.display());
        println!("按 Ctrl+C 停止监听");

        while let Ok(event) = rx.recv() {
            if let Some(watch_event) = Self::filter_event(&event) {
                Self::handle_event(self, &watch_event);
            }
        }

        let _ = watcher.unwatch(self.compiler.sheet_dir());
        Ok(())
    }

    /// 过滤文件系统事件，仅保留配置表文件的创建和修改事件
    fn filter_event(event: &Event) -> Option<WatchEvent> {
        let kind = match event.kind {
            EventKind::Create(_) => WatchEventKind::Create,
            EventKind::Modify(_) => WatchEventKind::Modify,
            _ => return None,
        };

        for path in &event.paths {
            if Self::is_temp_file(path) {
                continue;
            }

            if Self::is_supported_file(path) {
                return Some(match kind {
                    WatchEventKind::Create => WatchEvent::Created(path.clone()),
                    WatchEventKind::Modify => WatchEvent::Modified(path.clone()),
                });
            }
        }

        None
    }

    /// 判断文件是否为支持的配置表格式
    fn is_supported_file(path: &std::path::Path) -> bool {
        let ext = path.extension().and_then(|e| e.to_str()).map(|e| e.to_lowercase());
        matches!(ext.as_deref(), Some("xlsx") | Some("csv") | Some("tsv"))
    }

    /// 判断文件是否为临时文件（Excel 以 ~$ 开头的临时文件）
    fn is_temp_file(path: &std::path::Path) -> bool {
        path.file_name().and_then(|n| n.to_str()).map(|n| n.starts_with("~$")).unwrap_or(false)
    }

    /// 处理监听事件，执行增量编译
    fn handle_event(&mut self, event: &WatchEvent) {
        let (path, label) = match event {
            WatchEvent::Created(p) => (p, "创建"),
            WatchEvent::Modified(p) => (p, "修改"),
        };

        println!("[{}] {}", label, path.display());

        match self.compiler.compile_file(path) {
            Ok(()) => println!("  ✓ 编译成功"),
            Err(e) => println!("  ✗ 编译失败: {}", e),
        }
    }
}

/// 监听事件类型内部表示
enum WatchEventKind {
    /// 创建事件
    Create,
    /// 修改事件
    Modify,
}
