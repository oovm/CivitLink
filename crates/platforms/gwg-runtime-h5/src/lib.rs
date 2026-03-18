//! GWG Engine H5 (Web) 运行时
//!
//! 提供浏览器环境下的运行时支持。

use wasm_bindgen::prelude::*;
use web_sys::*;

use gwg_platform::prelude::*;
use gwg_world::prelude::*;
use gwg_schedule::prelude::*;

/// H5 运行时错误
#[derive(thiserror::Error, Debug)]
pub enum H5Error {
    /// 平台错误
    #[error("Platform error: {0}")]
    PlatformError(#[from] PlatformError),
    
    /// Web API 错误
    #[error("Web API error: {0}")]
    WebApiError(String),
    
    /// 初始化失败
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
}

/// H5 运行时结果
pub type H5Result<T> = Result<T, H5Error>;

/// H5 平台实现
pub struct H5Platform {
    /// 游戏世界
    game_world: GameWorld,
    /// 主调度器
    main_schedule: MainSchedule,
    /// 文件系统
    filesystem: H5Filesystem,
    /// 输入
    input: H5Input,
    /// 时间
    time: NullTime,
    /// 窗口
    window: H5Window,
    /// 窗口
    web_window: Option<Window>,
}

impl H5Platform {
    /// 创建新的 H5 平台
    pub fn new() -> H5Result<Self> {
        console_error_panic_hook::set_once();
        
        let window = web_sys::window().ok_or_else(|| {
            H5Error::InitializationFailed("Failed to get window".to_string())
        })?;
        
        Ok(Self {
            game_world: GameWorld::new("GWG Game".to_string()),
            main_schedule: create_default_schedule(),
            filesystem: H5Filesystem::new(),
            input: H5Input::new(),
            time: NullTime::new(),
            window: H5Window::new(),
            web_window: Some(window),
        })
    }

    /// 获取游戏世界
    pub fn game_world(&self) -> &GameWorld {
        &self.game_world
    }

    /// 获取游戏世界可变引用
    pub fn game_world_mut(&mut self) -> &mut GameWorld {
        &mut self.game_world
    }

    /// 获取主调度器
    pub fn main_schedule(&self) -> &MainSchedule {
        &self.main_schedule
    }

    /// 获取主调度器可变引用
    pub fn main_schedule_mut(&mut self) -> &mut MainSchedule {
        &mut self.main_schedule
    }

    /// 初始化平台
    pub fn initialize(&mut self) -> H5Result<()> {
        Ok(())
    }

    /// 运行主循环一帧
    pub fn run_frame(&mut self) {
        self.time.update();
        self.input.clear_state();
        
        self.main_schedule.run(self.game_world_mut().ecs_world_mut());
        
        self.window.clear_events();
    }

    /// 检查是否应该退出
    pub fn should_exit(&self) -> bool {
        false
    }

    /// 关闭平台
    pub fn shutdown(&mut self) {
    }
}

impl Default for H5Platform {
    fn default() -> Self {
        Self::new().expect("Failed to create H5 platform")
    }
}

impl Platform for H5Platform {
    fn platform_type(&self) -> PlatformType {
        PlatformType::H5
    }

    fn initialize(&mut self) -> PlatformResult<()> {
        Ok(())
    }

    fn shutdown(&mut self) {
    }

    fn poll_events(&mut self) {
    }

    fn should_exit(&self) -> bool {
        false
    }
}

/// H5 文件系统
pub struct H5Filesystem;

impl H5Filesystem {
    /// 创建新的 H5 文件系统
    pub fn new() -> Self {
        Self
    }
}

impl Default for H5Filesystem {
    fn default() -> Self {
        Self::new()
    }
}

impl Filesystem for H5Filesystem {
    fn read_file(&self, _path: &std::path::Path) -> PlatformResult<Vec<u8>> {
        Err(PlatformError::UnsupportedOperation(
            "File system not available on H5".to_string()
        ))
    }

    fn write_file(&mut self, _path: &std::path::Path, _data: &[u8]) -> PlatformResult<()> {
        Err(PlatformError::UnsupportedOperation(
            "File system not available on H5".to_string()
        ))
    }

    fn exists(&self, _path: &std::path::Path) -> bool {
        false
    }

    fn remove_file(&mut self, _path: &std::path::Path) -> PlatformResult<()> {
        Err(PlatformError::UnsupportedOperation(
            "File system not available on H5".to_string()
        ))
    }

    fn create_dir(&mut self, _path: &std::path::Path) -> PlatformResult<()> {
        Err(PlatformError::UnsupportedOperation(
            "File system not available on H5".to_string()
        ))
    }

    fn list_dir(&self, _path: &std::path::Path) -> PlatformResult<Vec<String>> {
        Err(PlatformError::UnsupportedOperation(
            "File system not available on H5".to_string()
        ))
    }
}

/// H5 输入
pub struct H5Input;

impl H5Input {
    /// 创建新的 H5 输入
    pub fn new() -> Self {
        Self
    }
}

impl Default for H5Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Input for H5Input {
    fn is_key_pressed(&self, _key_code: KeyCode) -> bool {
        false
    }

    fn is_key_just_pressed(&self, _key_code: KeyCode) -> bool {
        false
    }

    fn is_key_just_released(&self, _key_code: KeyCode) -> bool {
        false
    }

    fn is_mouse_button_pressed(&self, _button: MouseButton) -> bool {
        false
    }

    fn is_mouse_button_just_pressed(&self, _button: MouseButton) -> bool {
        false
    }

    fn is_mouse_button_just_released(&self, _button: MouseButton) -> bool {
        false
    }

    fn mouse_position(&self) -> (f64, f64) {
        (0.0, 0.0)
    }

    fn mouse_delta(&self) -> (f64, f64) {
        (0.0, 0.0)
    }

    fn events(&self) -> Box<dyn Iterator<Item = InputEvent> + '_> {
        Box::new(std::iter::empty())
    }

    fn clear_state(&mut self) {}
}

/// H5 窗口
pub struct H5Window {
    config: WindowConfig,
    size: WindowSize,
}

impl H5Window {
    /// 创建新的 H5 窗口
    pub fn new() -> Self {
        let config = WindowConfig::default();
        let size = WindowSize::new(config.width, config.height);
        Self { config, size }
    }
}

impl Default for H5Window {
    fn default() -> Self {
        Self::new()
    }
}

impl Window for H5Window {
    fn config(&self) -> &WindowConfig {
        &self.config
    }

    fn size(&self) -> WindowSize {
        self.size
    }

    fn set_size(&mut self, size: WindowSize) -> PlatformResult<()> {
        self.size = size;
        Ok(())
    }

    fn position(&self) -> WindowPosition {
        WindowPosition::new(0, 0)
    }

    fn set_position(&mut self, _position: WindowPosition) -> PlatformResult<()> {
        Ok(())
    }

    fn title(&self) -> &str {
        &self.config.title
    }

    fn set_title(&mut self, title: &str) -> PlatformResult<()> {
        self.config.title = title.to_string();
        Ok(())
    }

    fn is_fullscreen(&self) -> bool {
        self.config.fullscreen
    }

    fn set_fullscreen(&mut self, fullscreen: bool) -> PlatformResult<()> {
        self.config.fullscreen = fullscreen;
        Ok(())
    }

    fn is_visible(&self) -> bool {
        self.config.visible
    }

    fn set_visible(&mut self, visible: bool) -> PlatformResult<()> {
        self.config.visible = visible;
        Ok(())
    }

    fn swap_buffers(&mut self) -> PlatformResult<()> {
        Ok(())
    }

    fn events(&self) -> Box<dyn Iterator<Item = WindowEvent> + '_> {
        Box::new(std::iter::empty())
    }

    fn clear_events(&mut self) {}
}

/// WASM 导出的初始化函数
#[wasm_bindgen]
pub fn init() {
    console_error_panic_hook::set_once();
}

/// WASM 导出的 H5 运行时
#[wasm_bindgen]
pub struct H5Runtime {
    platform: H5Platform,
}

#[wasm_bindgen]
impl H5Runtime {
    /// 创建新的 H5 运行时
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<H5Runtime, JsValue> {
        init();
        let platform = H5Platform::new()
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        
        Ok(Self { platform })
    }

    /// 运行一帧
    #[wasm_bindgen]
    pub fn run_frame(&mut self) {
        self.platform.run_frame();
    }

    /// 检查是否应该退出
    #[wasm_bindgen]
    pub fn should_exit(&self) -> bool {
        self.platform.should_exit()
    }
}

impl Default for H5Runtime {
    fn default() -> Self {
        Self::new().expect("Failed to create H5 runtime")
    }
}

pub mod prelude {
    //! H5 运行时的预导入模块

    pub use super::{
        H5Error, H5Filesystem, H5Input, H5Platform, H5Result, H5Runtime, H5Window,
    };
    pub use gwg_platform::prelude::*;
    pub use gwg_world::prelude::*;
    pub use gwg_schedule::prelude::*;
    pub use wasm_bindgen::prelude::*;
}
