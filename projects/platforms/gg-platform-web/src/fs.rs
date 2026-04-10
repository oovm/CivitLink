use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;

use gg_core::{
    GError, GErrorKind, GResult,
    platform::{DirEntry, FileMetadata, FileSystem},
};

/// Web 平台文件系统实现
///
/// 基于 HTTP fetch API 和内存缓存提供 Web 环境的文件访问能力。
/// 读取操作通过 fetch API 从服务器获取文件并缓存到内存，
/// 写入操作使用 localStorage 模拟。
pub struct WebFileSystem {
    /// 资源基础 URL
    base_url: String,
    /// 文件内容内存缓存（路径 -> 字节数据）
    cache: RefCell<HashMap<String, Vec<u8>>>,
}

impl WebFileSystem {
    /// 创建新的 Web 文件系统实例
    ///
    /// # 参数
    ///
    /// - `base_url` - 资源文件的基础 URL 路径
    pub fn new(base_url: impl Into<String>) -> Self {
        Self { base_url: base_url.into(), cache: RefCell::new(HashMap::new()) }
    }

    /// 构建完整的 URL
    fn build_url(&self, path: &Path) -> String {
        format!("{}/{}", self.base_url, path.to_string_lossy())
    }

    /// 通过 fetch API 获取文件数据
    ///
    /// 优先从内存缓存中读取，若缓存未命中则通过 fetch API 获取。
    #[cfg(target_arch = "wasm32")]
    fn fetch_data(&self, path: &Path) -> GResult<Vec<u8>> {
        let path_str = path.to_string_lossy().to_string();

        if let Some(cached) = self.cache.borrow().get(&path_str) {
            return Ok(cached.clone());
        }

        let url = self.build_url(path);

        let window = web_sys::window().ok_or_else(|| GError {
            kind: GErrorKind::Platform,
            message: "No window object available".to_string(),
        })?;

        let promise = window.fetch_with_str(&url);

        let resp = wasm_bindgen_futures::futures::block_on(wasm_bindgen_futures::JsFuture::from(promise))
            .map_err(|_| GError { kind: GErrorKind::Io, message: format!("Failed to fetch '{}'", url) })?;

        let response = web_sys::Response::from(resp);

        if !response.ok() {
            return Err(GError {
                kind: GErrorKind::Io,
                message: format!("HTTP error fetching '{}': {}", url, response.status()),
            });
        }

        let array_buffer_promise = response.array_buffer()
            .map_err(|_| GError { kind: GErrorKind::Io, message: format!("Failed to get array buffer for '{}'", url) })?;

        let array_buffer = wasm_bindgen_futures::futures::block_on(wasm_bindgen_futures::JsFuture::from(array_buffer_promise))
            .map_err(|_| GError { kind: GErrorKind::Io, message: format!("Failed to read array buffer for '{}'", url) })?;

        let js_array = js_sys::Uint8Array::new(&array_buffer);
        let mut data = vec![0u8; js_array.length() as usize];
        js_array.copy_to(&mut data);

        self.cache.borrow_mut().insert(path_str, data.clone());

        Ok(data)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn fetch_data(&self, path: &Path) -> GResult<Vec<u8>> {
        Err(GError {
            kind: GErrorKind::Platform,
            message: format!("WebFileSystem is only available on wasm32 target, cannot fetch '{}'", path.display()),
        })
    }

    /// 通过 fetch HEAD 请求检查文件是否存在
    #[cfg(target_arch = "wasm32")]
    fn fetch_exists(&self, path: &Path) -> bool {
        let url = self.build_url(path);

        let window = match web_sys::window() {
            Some(w) => w,
            None => return false,
        };

        let mut opts = web_sys::RequestInit::new();
        opts.method("HEAD");

        let request = web_sys::Request::new_with_str_and_init(&url, &opts);
        let request = match request {
            Ok(r) => r,
            Err(_) => return false,
        };

        let promise = match window.fetch_with_request(&request) {
            Ok(p) => p,
            Err(_) => return false,
        };

        match wasm_bindgen_futures::futures::block_on(wasm_bindgen_futures::JsFuture::from(promise)) {
            Ok(resp) => {
                let response = web_sys::Response::from(resp);
                response.ok()
            }
            Err(_) => false,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn fetch_exists(&self, _path: &Path) -> bool {
        false
    }
}

impl FileSystem for WebFileSystem {
    fn exists(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_string();
        if self.cache.borrow().contains_key(&path_str) {
            return true;
        }
        self.fetch_exists(path)
    }

    fn read(&self, path: &Path) -> GResult<Vec<u8>> {
        self.fetch_data(path)
    }

    fn read_to_string(&self, path: &Path) -> GResult<String> {
        let bytes = self.read(path)?;
        String::from_utf8(bytes)
            .map_err(|e| GError { kind: GErrorKind::Io, message: format!("Failed to convert file to UTF-8: {}", e) })
    }

    fn write(&self, _path: &Path, _content: &[u8]) -> GResult<()> {
        #[cfg(target_arch = "wasm32")]
        {
            let key = _path.to_string_lossy().to_string();
            let value = js_sys::Uint8Array::from(_content).to_vec();
            let value_str =
                js_sys::JSON::stringify(&js_sys::Array::from_iter(value.iter().map(|&b| js_sys::Number::from(b)))).map_err(
                    |_| GError { kind: GErrorKind::Platform, message: "Failed to serialize data for localStorage".to_string() },
                )?;

            let window = web_sys::window()
                .ok_or_else(|| GError { kind: GErrorKind::Platform, message: "No window object available".to_string() })?;

            let storage = window
                .local_storage()
                .map_err(|_| GError { kind: GErrorKind::Platform, message: "Failed to access localStorage".to_string() })?
                .ok_or_else(|| GError { kind: GErrorKind::Platform, message: "localStorage not available".to_string() })?;

            storage.set_item(&key, &value_str.into_string().unwrap_or_default()).map_err(|_| GError {
                kind: GErrorKind::Platform,
                message: format!("Failed to write to localStorage: '{}'", key),
            })?;

            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Err(GError { kind: GErrorKind::Platform, message: "WebFileSystem is only available on wasm32 target".to_string() })
        }
    }

    fn create_dir_all(&self, _path: &Path) -> GResult<()> {
        Ok(())
    }

    fn read_dir(&self, _path: &Path) -> GResult<Vec<DirEntry>> {
        Err(GError { kind: GErrorKind::Platform, message: "Directory listing is not supported in Web environment".to_string() })
    }

    fn metadata(&self, _path: &Path) -> GResult<FileMetadata> {
        Err(GError { kind: GErrorKind::Platform, message: "File metadata is not supported in Web environment".to_string() })
    }

    fn remove_file(&self, _path: &Path) -> GResult<()> {
        #[cfg(target_arch = "wasm32")]
        {
            let key = _path.to_string_lossy().to_string();

            self.cache.borrow_mut().remove(&key);

            let window = web_sys::window()
                .ok_or_else(|| GError { kind: GErrorKind::Platform, message: "No window object available".to_string() })?;

            let storage = window
                .local_storage()
                .map_err(|_| GError { kind: GErrorKind::Platform, message: "Failed to access localStorage".to_string() })?
                .ok_or_else(|| GError { kind: GErrorKind::Platform, message: "localStorage not available".to_string() })?;

            storage.remove_item(&key).map_err(|_| GError {
                kind: GErrorKind::Platform,
                message: format!("Failed to remove from localStorage: '{}'", key),
            })?;

            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Err(GError { kind: GErrorKind::Platform, message: "WebFileSystem is only available on wasm32 target".to_string() })
        }
    }
}
