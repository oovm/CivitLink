/// 目标平台枚举，定义 AOT 编译支持的目标平台
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TargetPlatform {
    /// Windows x86_64
    WindowsX64,
    /// macOS ARM64 (Apple Silicon)
    MacOSArm64,
    /// Linux x86_64
    LinuxX64,
    /// WebAssembly 32-bit
    WebWasm32,
    /// Android ARM64
    AndroidArm64,
    /// iOS ARM64
    IOSArm64,
}

impl TargetPlatform {
    /// 获取目标平台的人类可读名称
    pub fn name(&self) -> &str {
        match self {
            TargetPlatform::WindowsX64 => "Windows x86_64",
            TargetPlatform::MacOSArm64 => "macOS ARM64",
            TargetPlatform::LinuxX64 => "Linux x86_64",
            TargetPlatform::WebWasm32 => "WebAssembly 32-bit",
            TargetPlatform::AndroidArm64 => "Android ARM64",
            TargetPlatform::IOSArm64 => "iOS ARM64",
        }
    }

    /// 获取 LLVM 风格的目标三元组
    pub fn triple(&self) -> &str {
        match self {
            TargetPlatform::WindowsX64 => "x86_64-pc-windows-msvc",
            TargetPlatform::MacOSArm64 => "aarch64-apple-darwin",
            TargetPlatform::LinuxX64 => "x86_64-unknown-linux-gnu",
            TargetPlatform::WebWasm32 => "wasm32-unknown-unknown",
            TargetPlatform::AndroidArm64 => "aarch64-linux-android",
            TargetPlatform::IOSArm64 => "aarch64-apple-ios",
        }
    }

    /// 获取所有支持的目标平台列表
    pub fn all() -> &'static [TargetPlatform] {
        &[
            TargetPlatform::WindowsX64,
            TargetPlatform::MacOSArm64,
            TargetPlatform::LinuxX64,
            TargetPlatform::WebWasm32,
            TargetPlatform::AndroidArm64,
            TargetPlatform::IOSArm64,
        ]
    }

    /// 从字符串名称解析目标平台
    pub fn from_name(name: &str) -> Option<TargetPlatform> {
        match name {
            "WindowsX64" | "windows-x64" | "x86_64-pc-windows-msvc" => {
                Some(TargetPlatform::WindowsX64)
            }
            "MacOSArm64" | "macos-arm64" | "aarch64-apple-darwin" => {
                Some(TargetPlatform::MacOSArm64)
            }
            "LinuxX64" | "linux-x64" | "x86_64-unknown-linux-gnu" => {
                Some(TargetPlatform::LinuxX64)
            }
            "WebWasm32" | "web-wasm32" | "wasm32-unknown-unknown" => {
                Some(TargetPlatform::WebWasm32)
            }
            "AndroidArm64" | "android-arm64" | "aarch64-linux-android" => {
                Some(TargetPlatform::AndroidArm64)
            }
            "IOSArm64" | "ios-arm64" | "aarch64-apple-ios" => {
                Some(TargetPlatform::IOSArm64)
            }
            _ => None,
        }
    }
}
