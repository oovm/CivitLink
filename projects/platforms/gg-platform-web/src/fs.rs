use std::path::Path;

use gg_core::platform::{DirEntry, FileMetadata, FileSystem};
use gg_core::{GError, GErrorKind, GResult};

/// Web 平台文件系统实现
///
/// 基于 HTTP fetch 和 localStorage 提供 Web 环境的文件访问能力。
/// 读取操作通过同步 XHR 从服务器获取文件，写入操作使用 localStorage 模拟。
#[allow(dead_code)]
pub struct WebFileSystem {
    /// 资源基础 URL
    base_url: String,
}

impl WebFileSystem {
    /// 创建新的 Web 文件系统实例
    ///
    /// # 参数
    ///
    /// - `base_url` - 资源文件的基础 URL 路径
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }
}

impl FileSystem for WebFileSystem {
    fn exists(&self, path: &Path) -> bool {
        #[cfg(target_arch = "wasm32")]
        {
            let url = format!("{}/{}", self.base_url, path.to_string_lossy());
            js_sys::eval(&format!(
                "(() => {{ try {{ var x = new XMLHttpRequest(); x.open('HEAD', '{}', false); x.send(); return x.status === 200; }} catch(e) {{ return false; }} }})()",
                url
            ))
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = path;
            false
        }
    }

    fn read(&self, path: &Path) -> GResult<Vec<u8>> {
        #[cfg(target_arch = "wasm32")]
        {
            let url = format!("{}/{}", self.base_url, path.to_string_lossy());
            js_sys::eval(&format!(
                "(() => {{ var x = new XMLHttpRequest(); x.open('GET', '{}', false); x.responseType = 'arraybuffer'; x.send(); if (x.status === 200) {{ return Array.from(new Uint8Array(x.response)); }} else {{ throw new Error('HTTP ' + x.status); }} }})()",
                url
            ))
            .map_err(|_| GError {
                kind: GErrorKind::Platform,
                message: format!("Failed to fetch file '{}'", path.display()),
            })
            .and_then(|v| {
                let arr: Vec<u8> = js_sys::Array::from(&v)
                    .iter()
                    .filter_map(|x| x.as_f64().map(|f| f as u8))
                    .collect();
                Ok(arr)
            })
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Err(GError {
                kind: GErrorKind::Platform,
                message: format!(
                    "WebFileSystem is only available on wasm32 target, cannot read '{}'",
                    path.display()
                ),
            })
        }
    }

    fn read_to_string(&self, path: &Path) -> GResult<String> {
        let bytes = self.read(path)?;
        String::from_utf8(bytes).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to convert file to UTF-8: {}", e),
        })
    }

    fn write(&self, _path: &Path, _content: &[u8]) -> GResult<()> {
        #[cfg(target_arch = "wasm32")]
        {
            let key = path.to_string_lossy().to_string();
            let value = js_sys::Uint8Array::from(content).to_vec();
            let value_str =
                js_sys::JSON::stringify(&js_sys::Array::from_iter(
                    value.iter().map(|&b| js_sys::Number::from(b)),
                ))
                .map_err(|_| GError {
                    kind: GErrorKind::Platform,
                    message: "Failed to serialize data for localStorage".to_string(),
                })?;

            let window = web_sys::window().ok_or_else(|| GError {
                kind: GErrorKind::Platform,
                message: "No window object available".to_string(),
            })?;

            let storage = window
                .local_storage()
                .map_err(|_| GError {
                    kind: GErrorKind::Platform,
                    message: "Failed to access localStorage".to_string(),
                })?
                .ok_or_else(|| GError {
                    kind: GErrorKind::Platform,
                    message: "localStorage not available".to_string(),
                })?;

            storage
                .set_item(&key, &value_str.into_string().unwrap_or_default())
                .map_err(|_| GError {
                    kind: GErrorKind::Platform,
                    message: format!("Failed to write to localStorage: '{}'", key),
                })?;

            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Err(GError {
                kind: GErrorKind::Platform,
                message: "WebFileSystem is only available on wasm32 target".to_string(),
            })
        }
    }

    fn create_dir_all(&self, _path: &Path) -> GResult<()> {
        Ok(())
    }

    fn read_dir(&self, _path: &Path) -> GResult<Vec<DirEntry>> {
        Err(GError {
            kind: GErrorKind::Platform,
            message: "Directory listing is not supported in Web environment".to_string(),
        })
    }

    fn metadata(&self, _path: &Path) -> GResult<FileMetadata> {
        Err(GError {
            kind: GErrorKind::Platform,
            message: "File metadata is not supported in Web environment".to_string(),
        })
    }

    fn remove_file(&self, _path: &Path) -> GResult<()> {
        #[cfg(target_arch = "wasm32")]
        {
            let key = path.to_string_lossy().to_string();
            let window = web_sys::window().ok_or_else(|| GError {
                kind: GErrorKind::Platform,
                message: "No window object available".to_string(),
            })?;

            let storage = window
                .local_storage()
                .map_err(|_| GError {
                    kind: GErrorKind::Platform,
                    message: "Failed to access localStorage".to_string(),
                })?
                .ok_or_else(|| GError {
                    kind: GErrorKind::Platform,
                    message: "localStorage not available".to_string(),
                })?;

            storage.remove_item(&key).map_err(|_| GError {
                kind: GErrorKind::Platform,
                message: format!("Failed to remove from localStorage: '{}'", key),
            })?;

            Ok(())
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Err(GError {
                kind: GErrorKind::Platform,
                message: "WebFileSystem is only available on wasm32 target".to_string(),
            })
        }
    }
}
