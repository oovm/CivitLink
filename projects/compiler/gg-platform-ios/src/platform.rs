use std::{path::PathBuf, process::Command};

use gg_core::{
    GError, GErrorKind, GResult,
    platform::{BuildConfig, DeviceInfo, EnvironmentReport, GenerateContext, PackageContext, Platform, PlatformId, RunContext},
};
use gg_platform::utils;

/// iOS 平台构建时实现
///
/// 为 iOS 提供构建、生成、打包和运行的 Platform trait 完整实现。
/// 支持生成完整 Xcode 项目结构、签名管理和真机调试。
pub struct IOSPlatform;

impl IOSPlatform {
    /// 创建 iOS 平台实例
    pub fn new() -> Self {
        Self
    }
}

impl Platform for IOSPlatform {
    fn id(&self) -> PlatformId {
        "ios".to_string()
    }

    fn display_name(&self) -> &str {
        "iOS"
    }

    fn configure_build(&self, config: &mut BuildConfig) {
        config.target_triple = "aarch64-apple-ios".to_string();
    }

    fn generate_code(&self, ctx: &GenerateContext) -> GResult<PathBuf> {
        generate_ios_code(ctx)
    }

    fn package(&self, ctx: &PackageContext) -> GResult<Vec<PathBuf>> {
        package_ios(ctx)
    }

    fn run(&self, ctx: &RunContext) -> GResult<()> {
        run_ios(ctx)
    }

    fn list_devices(&self) -> GResult<Vec<DeviceInfo>> {
        list_ios_devices()
    }

    fn validate_environment(&self) -> GResult<EnvironmentReport> {
        validate_ios_environment()
    }
}

/// 生成 iOS 平台代码
///
/// 生成完整的 Xcode 项目结构，包含 .xcodeproj 目录和所有源文件。
fn generate_ios_code(ctx: &GenerateContext) -> GResult<PathBuf> {
    let ios_dir = ctx.output_dir.join("ios");
    let sources_dir = ios_dir.join("Sources");
    let xcodeproj_dir = ios_dir.join("GGGame.xcodeproj");

    std::fs::create_dir_all(&sources_dir).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to create ios/Sources directory '{}': {}", sources_dir.display(), e),
    })?;

    std::fs::create_dir_all(&xcodeproj_dir).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to create xcodeproj directory '{}': {}", xcodeproj_dir.display(), e),
    })?;

    let info_plist = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>com.gg.game</string>
    <key>CFBundleName</key>
    <string>GG Game</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0</string>
    <key>CFBundleExecutable</key>
    <string>game</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>MinimumOSVersion</key>
    <string>14.0</string>
    <key>UILaunchStoryboardName</key>
    <string>LaunchScreen</string>
    <key>UISupportedInterfaceOrientations</key>
    <array>
        <string>UIInterfaceOrientationPortrait</string>
        <string>UIInterfaceOrientationLandscapeLeft</string>
        <string>UIInterfaceOrientationLandscapeRight</string>
    </array>
</dict>
</plist>
"#;

    let main_m = r#"#import <UIKit/UIKit.h>
#import "AppDelegate.h"

int main(int argc, char * argv[]) {
    NSString * appDelegateClassName = NSStringFromClass([AppDelegate class]);
    return UIApplicationMain(argc, argv, nil, appDelegateClassName);
}
"#;

    let app_delegate_h = r#"#import <UIKit/UIKit.h>

@interface AppDelegate : UIResponder <UIApplicationDelegate>

@property (strong, nonatomic) UIWindow *window;

@end
"#;

    let app_delegate_m = r#"#import "AppDelegate.h"
#import "ViewController.h"

@implementation AppDelegate

- (BOOL)application:(UIApplication *)application didFinishLaunchingWithOptions:(NSDictionary *)launchOptions {
    self.window = [[UIWindow alloc] initWithFrame:[UIScreen mainScreen].bounds];
    self.window.rootViewController = [[ViewController alloc] init];
    [self.window makeKeyAndVisible];
    return YES;
}

@end
"#;

    let view_controller_h = r#"#import <UIKit/UIKit.h>

@interface ViewController : UIViewController

@end
"#;

    let view_controller_m = r#"#import "ViewController.h"
#import <MetalKit/MetalKit.h>

@implementation ViewController

- (void)loadView {
    MTKView *mtkView = [[MTKView alloc] initWithFrame:CGRectZero];
    mtkView.autoresizingMask = UIViewAutoresizingFlexibleWidth | UIViewAutoresizingFlexibleHeight;
    self.view = mtkView;
}

@end
"#;

    utils::write_file(&sources_dir.join("Info.plist"), info_plist)?;
    utils::write_file(&sources_dir.join("main.m"), main_m)?;
    utils::write_file(&sources_dir.join("AppDelegate.h"), app_delegate_h)?;
    utils::write_file(&sources_dir.join("AppDelegate.m"), app_delegate_m)?;
    utils::write_file(&sources_dir.join("ViewController.h"), view_controller_h)?;
    utils::write_file(&sources_dir.join("ViewController.m"), view_controller_m)?;

    let pbxproj_content = generate_pbxproj();
    utils::write_file(&xcodeproj_dir.join("project.pbxproj"), &pbxproj_content)?;

    Ok(ios_dir)
}

/// 生成 Xcode project.pbxproj 文件内容
fn generate_pbxproj() -> String {
    r#"// !$*UTF8*$!
{
    archiveVersion = 1;
    classes = {
    };
    objectVersion = 56;
    objects = {

/* Begin PBXFileReference section */
        8A1 /* main.m */ = {isa = PBXFileReference; fileEncoding = 4; lastKnownFileType = sourcecode.c.objc; path = main.m; sourceTree = "<group>"; };
        8A2 /* AppDelegate.h */ = {isa = PBXFileReference; fileEncoding = 4; lastKnownFileType = sourcecode.c.h; path = AppDelegate.h; sourceTree = "<group>"; };
        8A3 /* AppDelegate.m */ = {isa = PBXFileReference; fileEncoding = 4; lastKnownFileType = sourcecode.c.objc; path = AppDelegate.m; sourceTree = "<group>"; };
        8A4 /* ViewController.h */ = {isa = PBXFileReference; fileEncoding = 4; lastKnownFileType = sourcecode.c.h; path = ViewController.h; sourceTree = "<group>"; };
        8A5 /* ViewController.m */ = {isa = PBXFileReference; fileEncoding = 4; lastKnownFileType = sourcecode.c.objc; path = ViewController.m; sourceTree = "<group>"; };
        8A6 /* Info.plist */ = {isa = PBXFileReference; fileEncoding = 4; lastKnownFileType = text.plist.xml; path = Info.plist; sourceTree = "<group>"; };
/* End PBXFileReference section */

/* Begin PBXGroup section */
        8A10 = {
            isa = PBXGroup;
            children = (
                8A1,
                8A2,
                8A3,
                8A4,
                8A5,
                8A6,
            );
            path = Sources;
            sourceTree = "<group>";
        };
/* End PBXGroup section */

/* Begin XCBuildConfiguration section */
        8A20 /* Debug */ = {
            isa = XCBuildConfiguration;
            buildSettings = {
                ARCHS = arm64;
                IPHONEOS_DEPLOYMENT_TARGET = 14.0;
                PRODUCT_BUNDLE_IDENTIFIER = com.gg.game;
                PRODUCT_NAME = "$(TARGET_NAME)";
                SDKROOT = iphoneos;
                TARGETED_DEVICE_FAMILY = "1,2";
            };
            name = Debug;
        };
        8A21 /* Release */ = {
            isa = XCBuildConfiguration;
            buildSettings = {
                ARCHS = arm64;
                IPHONEOS_DEPLOYMENT_TARGET = 14.0;
                PRODUCT_BUNDLE_IDENTIFIER = com.gg.game;
                PRODUCT_NAME = "$(TARGET_NAME)";
                SDKROOT = iphoneos;
                TARGETED_DEVICE_FAMILY = "1,2";
            };
            name = Release;
        };
/* End XCBuildConfiguration section */

    };
    rootObject = 8A10;
}
"#.to_string()
}

/// iOS 打包
///
/// 创建 .app 目录并复制可执行文件和资源。
/// 如果 xcrun 可用，进一步打包为 .ipa 文件。
fn package_ios(ctx: &PackageContext) -> GResult<Vec<PathBuf>> {
    let app_dir = ctx.package_output_dir.join("game.app");
    std::fs::create_dir_all(&app_dir).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to create app directory '{}': {}", app_dir.display(), e),
    })?;

    let exe_src = utils::find_executable(&ctx.build_output_dir, "game")?;
    let exe_dst = app_dir.join("game");
    std::fs::copy(&exe_src, &exe_dst).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to copy executable to '{}': {}", exe_dst.display(), e),
    })?;

    let info_plist_src = ctx.build_output_dir.join("Info.plist");
    if info_plist_src.exists() {
        let info_plist_dst = app_dir.join("Info.plist");
        std::fs::copy(&info_plist_src, &info_plist_dst).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to copy Info.plist to '{}': {}", info_plist_dst.display(), e),
        })?;
    }

    if ctx.assets_dir.exists() {
        let assets_dst = app_dir.join("assets");
        utils::copy_dir_recursive(&ctx.assets_dir, &assets_dst)?;
    }

    if utils::is_command_available("codesign") {
        let identity = detect_signing_identity();
        let app_str = app_dir.to_string_lossy().to_string();
        let args = vec!["--deep", "--force", "--sign", &identity, &app_str];

        if let Ok(status) = Command::new("codesign").args(&args).status() {
            if !status.success() {
                eprintln!("Warning: codesign failed for iOS app");
            }
        }
    }
    else {
        eprintln!("Warning: codesign not found, skipping iOS code signing");
    }

    if utils::is_command_available("xcrun") {
        let payload_dir = ctx.package_output_dir.join("Payload");
        let payload_app_dir = payload_dir.join("game.app");

        if payload_app_dir.exists() {
            let _ = std::fs::remove_dir_all(&payload_app_dir);
        }

        std::fs::create_dir_all(&payload_dir).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to create Payload directory '{}': {}", payload_dir.display(), e),
        })?;

        utils::copy_dir_recursive(&app_dir, &payload_app_dir)?;

        let ipa_path = ctx.package_output_dir.join("game.ipa");
        if ipa_path.exists() {
            let _ = std::fs::remove_file(&ipa_path);
        }

        let zip_status = Command::new("zip")
            .args(["-r", "game.ipa", "Payload"])
            .current_dir(&ctx.package_output_dir)
            .status()
            .map_err(|e| GError { kind: GErrorKind::Platform, message: format!("Failed to run zip command: {}", e) })?;

        if zip_status.success() {
            let _ = std::fs::remove_dir_all(&payload_dir);
            let _ = std::fs::remove_dir_all(&app_dir);
            return Ok(vec![ipa_path]);
        }
        else {
            eprintln!("Warning: zip command failed, falling back to .app directory output");
            let _ = std::fs::remove_dir_all(&payload_dir);
        }
    }
    else {
        eprintln!("Warning: xcrun is not available, skipping IPA packaging. Outputting .app directory as fallback.");
    }

    Ok(vec![app_dir])
}

/// iOS 运行
///
/// 通过 xcrun simctl 在 iOS 模拟器上运行应用。
/// 支持通过 RunContext args 指定目标设备 ID。
fn run_ios(ctx: &RunContext) -> GResult<()> {
    if !utils::is_command_available("xcrun") {
        return Err(GError {
            kind: GErrorKind::Platform,
            message: "xcrun is not available. Please install Xcode command line tools.".to_string(),
        });
    }

    let device_id = ctx.args.first().map(|s| s.as_str()).unwrap_or("iPhone 15");

    let boot_status = Command::new("xcrun")
        .args(["simctl", "boot", device_id])
        .status()
        .map_err(|e| GError { kind: GErrorKind::Platform, message: format!("Failed to run xcrun simctl boot: {}", e) })?;

    if !boot_status.success() {
        let stderr_output = Command::new("xcrun")
            .args(["simctl", "boot", device_id])
            .output()
            .map_err(|e| GError { kind: GErrorKind::Platform, message: format!("Failed to run xcrun simctl boot: {}", e) })?;
        let stderr = String::from_utf8_lossy(&stderr_output.stderr);
        if !stderr.contains("Unable to boot device in current state: Booted") {
            return Err(GError {
                kind: GErrorKind::Platform,
                message: format!("Failed to boot simulator '{}': {}", device_id, stderr.trim()),
            });
        }
    }

    let app_path = ctx.executable_path.display().to_string();
    let install_status = Command::new("xcrun")
        .args(["simctl", "install", "booted", &app_path])
        .status()
        .map_err(|e| GError { kind: GErrorKind::Platform, message: format!("Failed to install app: {}", e) })?;

    if !install_status.success() {
        return Err(GError { kind: GErrorKind::Platform, message: "Failed to install app on simulator".to_string() });
    }

    let launch_status = Command::new("xcrun")
        .args(["simctl", "launch", "booted", "com.gg.game"])
        .status()
        .map_err(|e| GError { kind: GErrorKind::Platform, message: format!("Failed to launch app: {}", e) })?;

    if !launch_status.success() {
        return Err(GError { kind: GErrorKind::Platform, message: "Failed to launch app on simulator".to_string() });
    }

    Ok(())
}

/// 列出 iOS 设备
///
/// 通过 xcrun xctrace list devices 获取可用设备列表。
fn list_ios_devices() -> GResult<Vec<DeviceInfo>> {
    if !utils::is_command_available("xcrun") {
        return Ok(Vec::new());
    }

    let output = Command::new("xcrun")
        .args(["xctrace", "list", "devices"])
        .output()
        .map_err(|e| GError { kind: GErrorKind::Platform, message: format!("Failed to list devices: {}", e) })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with('=') || line.is_empty() {
            continue;
        }

        if let Some(open_paren) = line.rfind('(') {
            if let Some(close_paren) = line.rfind(')') {
                let name = line[..open_paren].trim();
                let identifier = &line[open_paren + 1..close_paren];
                let state = if line.contains("Simulator") { "simulator" } else { "connected" };
                devices.push(DeviceInfo {
                    id: identifier.to_string(),
                    name: name.to_string(),
                    platform: "ios".to_string(),
                    state: state.to_string(),
                });
            }
        }
    }

    Ok(devices)
}

/// 验证 iOS 开发环境
fn validate_ios_environment() -> GResult<EnvironmentReport> {
    let mut available = Vec::new();
    let mut missing = Vec::new();
    let mut warnings = Vec::new();

    let tools = ["xcodebuild", "xcrun", "codesign", "swiftc"];

    for tool in tools {
        if utils::is_command_available(tool) {
            available.push(tool.to_string());
        }
        else {
            missing.push(tool.to_string());
            warnings.push(format!("{} is not available, iOS development requires Xcode", tool));
        }
    }

    Ok(EnvironmentReport { available_tools: available, missing_tools: missing, warnings })
}

/// 自动检测签名身份
///
/// 尝试通过 security find-identity 检测可用的签名身份。
/// 如果检测失败，返回默认的 ad-hoc 签名标识。
fn detect_signing_identity() -> String {
    if let Ok(output) = Command::new("security").args(["find-identity", "-v", "-p", "codesigning"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains("Apple Development") || line.contains("Apple Distribution") || line.contains("iPhone Developer") {
                if let Some(start) = line.find('"') {
                    if let Some(end) = line[start + 1..].find('"') {
                        let identity = &line[start + 1..start + 1 + end];
                        return identity.to_string();
                    }
                }
            }
        }
    }
    "-".to_string()
}
