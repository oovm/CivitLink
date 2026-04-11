use std::{path::PathBuf, process::Command};

use gg_core::{
    GError, GErrorKind, GResult,
    platform::{BuildConfig, DeviceInfo, EnvironmentReport, GenerateContext, PackageContext, Platform, PlatformId, RunContext},
};
use gg_platform::utils;

/// Android 平台构建时实现
///
/// 为 Android 提供构建、生成、打包和运行的 Platform trait 完整实现。
/// 支持完整 Gradle 项目生成、签名配置和真机调试。
pub struct AndroidPlatform;

impl AndroidPlatform {
    /// 创建 Android 平台实例
    pub fn new() -> Self {
        Self
    }
}

impl Platform for AndroidPlatform {
    fn id(&self) -> PlatformId {
        "android".to_string()
    }

    fn display_name(&self) -> &str {
        "Android"
    }

    fn configure_build(&self, config: &mut BuildConfig) {
        config.target_triple = "aarch64-linux-android".to_string();
    }

    fn generate_code(&self, ctx: &GenerateContext) -> GResult<PathBuf> {
        generate_android_code(ctx)
    }

    fn package(&self, ctx: &PackageContext) -> GResult<Vec<PathBuf>> {
        package_android(ctx)
    }

    fn run(&self, ctx: &RunContext) -> GResult<()> {
        run_android(ctx)
    }

    fn list_devices(&self) -> GResult<Vec<DeviceInfo>> {
        list_android_devices()
    }

    fn validate_environment(&self) -> GResult<EnvironmentReport> {
        validate_android_environment()
    }
}

/// 生成 Android 平台代码
///
/// 生成完整的 Gradle 项目结构，包含 AndroidManifest.xml、Java 源文件、
/// Gradle 构建脚本、gradle.properties、proguard 规则和 Gradle Wrapper。
fn generate_android_code(ctx: &GenerateContext) -> GResult<PathBuf> {
    let android_dir = ctx.output_dir.join("android");
    let app_main_dir = android_dir.join("app/src/main");
    let java_dir = app_main_dir.join("java/com/gg/game");
    let gradle_wrapper_dir = android_dir.join("gradle/wrapper");

    std::fs::create_dir_all(&java_dir).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to create android java directory '{}': {}", java_dir.display(), e),
    })?;

    std::fs::create_dir_all(&gradle_wrapper_dir).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to create gradle wrapper directory '{}': {}", gradle_wrapper_dir.display(), e),
    })?;

    let android_manifest = r#"<?xml version="1.0" encoding="utf-8"?>
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="com.gg.game">

    <uses-permission android:name="android.permission.INTERNET" />

    <application
        android:allowBackup="true"
        android:label="GG Game"
        android:hasCode="true">
        <activity
            android:name=".MainActivity"
            android:exported="true"
            android:configChanges="orientation|screenSize|keyboardHidden">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
        </activity>
    </application>

</manifest>
"#;

    let main_activity_java = r#"package com.gg.game;

import android.app.Activity;
import android.os.Bundle;

public class MainActivity extends Activity {

    static {
        System.loadLibrary("game");
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
    }

}
"#;

    let app_build_gradle = r#"android {
    compileSdk 34

    defaultConfig {
        applicationId "com.gg.game"
        minSdk 24
        targetSdk 34
        versionCode 1
        versionName "1.0"

        ndk {
            abiFilters 'arm64-v8a'
        }
    }

    signingConfigs {
        debug {
            storeFile file('debug.keystore')
            storePassword 'android'
            keyAlias 'androiddebugkey'
            keyPassword 'android'
        }
    }

    buildTypes {
        debug {
            signingConfig signingConfigs.debug
            minifyEnabled false
        }
        release {
            signingConfig signingConfigs.debug
            minifyEnabled true
            proguardFiles getDefaultProguardFile('proguard-android-optimize.txt'), 'proguard-rules.pro'
        }
    }

    sourceSets {
        main {
            jniLibs.srcDirs = ['libs']
        }
    }
}
"#;

    let root_build_gradle = r#"buildscript {
    repositories {
        google()
        mavenCentral()
    }
    dependencies {
        classpath 'com.android.tools.build:gradle:8.1.0'
    }
}

allprojects {
    repositories {
        google()
        mavenCentral()
    }
}
"#;

    let settings_gradle = r#"include ':app'
rootProject.name = "GGGame"
"#;

    let gradle_properties = r#"org.gradle.jvmargs=-Xmx2048m -Dfile.encoding=UTF-8
android.useAndroidX=true
android.enableJetifier=true
android.nonTransitiveRClass=true
"#;

    let proguard_rules = r#"# GG Game Engine ProGuard Rules

# Keep engine classes
-keep class gg.** { *; }

# Keep native method declarations
-keepclasseswithmembernames class * {
    native <methods>;
}

# Don't warn about missing references
-dontwarn com.gg.**
-dontwarn javax.**

# Keep activity classes
-keep class * extends android.app.Activity { *; }
"#;

    let gradle_wrapper_properties = r#"distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-8.5-bin.zip
networkTimeout=10000
validateDistributionUrl=true
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists
"#;

    let local_properties = if let Ok(sdk_root) = std::env::var("ANDROID_HOME") {
        format!("sdk.dir={}", sdk_root.replace('\\', "/"))
    }
    else if let Ok(sdk_root) = std::env::var("ANDROID_SDK_ROOT") {
        format!("sdk.dir={}", sdk_root.replace('\\', "/"))
    }
    else {
        "# sdk.dir=/path/to/android/sdk".to_string()
    };

    utils::write_file(&app_main_dir.join("AndroidManifest.xml"), android_manifest)?;
    utils::write_file(&java_dir.join("MainActivity.java"), main_activity_java)?;
    utils::write_file(&android_dir.join("app/build.gradle"), app_build_gradle)?;
    utils::write_file(&android_dir.join("build.gradle"), root_build_gradle)?;
    utils::write_file(&android_dir.join("settings.gradle"), settings_gradle)?;
    utils::write_file(&android_dir.join("gradle.properties"), gradle_properties)?;
    utils::write_file(&android_dir.join("app/proguard-rules.pro"), proguard_rules)?;
    utils::write_file(&gradle_wrapper_dir.join("gradle-wrapper.properties"), gradle_wrapper_properties)?;
    utils::write_file(&android_dir.join("local.properties"), &local_properties)?;

    Ok(android_dir)
}

/// Android 打包
///
/// 创建 .apk.dir 目录结构并复制 .so 和 .dex 文件。
/// 如果 Gradle 可用，构建正式 APK。
fn package_android(ctx: &PackageContext) -> GResult<Vec<PathBuf>> {
    let apk_dir = ctx.package_output_dir.join("game.apk.dir");
    std::fs::create_dir_all(&apk_dir).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to create apk directory '{}': {}", apk_dir.display(), e),
    })?;

    let lib_dir = apk_dir.join("lib/arm64-v8a");
    std::fs::create_dir_all(&lib_dir).map_err(|e| GError {
        kind: GErrorKind::Io,
        message: format!("Failed to create lib directory '{}': {}", lib_dir.display(), e),
    })?;

    let _ = utils::copy_files_by_ext(&ctx.build_output_dir, &lib_dir, ".so");
    let _ = utils::copy_files_by_ext(&ctx.build_output_dir, &apk_dir, ".dex");

    if ctx.assets_dir.exists() {
        let assets_dst = apk_dir.join("assets");
        utils::copy_dir_recursive(&ctx.assets_dir, &assets_dst)?;
    }

    let manifest_src = ctx.build_output_dir.join("AndroidManifest.xml");
    if manifest_src.exists() {
        let manifest_dst = apk_dir.join("AndroidManifest.xml");
        std::fs::copy(&manifest_src, &manifest_dst).map_err(|e| GError {
            kind: GErrorKind::Io,
            message: format!("Failed to copy AndroidManifest.xml to '{}': {}", manifest_dst.display(), e),
        })?;
    }

    let gradle_available = utils::is_command_available("gradle");

    let gradlew_available = if !gradle_available {
        let android_dir = ctx.build_output_dir.join("android");
        let gradlew_path = if cfg!(windows) { android_dir.join("gradlew.bat") } else { android_dir.join("gradlew") };
        gradlew_path.exists()
    }
    else {
        false
    };

    if gradle_available || gradlew_available {
        let android_src = ctx.build_output_dir.join("android");
        let temp_android_dir = ctx.package_output_dir.join("android_build");

        if temp_android_dir.exists() {
            let _ = std::fs::remove_dir_all(&temp_android_dir);
        }

        if android_src.exists() {
            utils::copy_dir_recursive(&android_src, &temp_android_dir)?;

            let jni_libs_dir = temp_android_dir.join("app/libs/arm64-v8a");
            std::fs::create_dir_all(&jni_libs_dir).map_err(|e| GError {
                kind: GErrorKind::Io,
                message: format!("Failed to create jniLibs directory '{}': {}", jni_libs_dir.display(), e),
            })?;

            let _ = utils::copy_files_by_ext(&ctx.build_output_dir, &jni_libs_dir, ".so");

            let assets_dir = temp_android_dir.join("app/src/main/assets");
            if ctx.assets_dir.exists() {
                std::fs::create_dir_all(&assets_dir).map_err(|e| GError {
                    kind: GErrorKind::Io,
                    message: format!("Failed to create assets directory '{}': {}", assets_dir.display(), e),
                })?;
                utils::copy_dir_recursive(&ctx.assets_dir, &assets_dir)?;
            }

            generate_debug_keystore(&temp_android_dir.join("app"))?;

            let build_result = if gradle_available {
                Command::new("gradle").arg("assembleDebug").current_dir(&temp_android_dir).status()
            }
            else {
                let gradlew_path =
                    if cfg!(windows) { temp_android_dir.join("gradlew.bat") } else { temp_android_dir.join("gradlew") };
                Command::new(gradlew_path).arg("assembleDebug").current_dir(&temp_android_dir).status()
            };

            match build_result {
                Ok(status) if status.success() => {
                    let apk_output = temp_android_dir.join("app/build/outputs/apk/debug/app-debug.apk");
                    if apk_output.exists() {
                        let final_apk_path = ctx.package_output_dir.join("game.apk");
                        std::fs::copy(&apk_output, &final_apk_path).map_err(|e| GError {
                            kind: GErrorKind::Io,
                            message: format!("Failed to copy APK to '{}': {}", final_apk_path.display(), e),
                        })?;
                        let _ = std::fs::remove_dir_all(&temp_android_dir);
                        let _ = std::fs::remove_dir_all(&apk_dir);
                        return Ok(vec![final_apk_path]);
                    }
                    else {
                        eprintln!(
                            "Warning: gradle build succeeded but APK not found at expected path, falling back to .apk.dir output"
                        );
                    }
                }
                Ok(_) => {
                    eprintln!("Warning: gradle build failed, falling back to .apk.dir output");
                }
                Err(e) => {
                    eprintln!("Warning: failed to run gradle command: {}, falling back to .apk.dir output", e);
                }
            }

            let _ = std::fs::remove_dir_all(&temp_android_dir);
        }
        else {
            eprintln!("Warning: android project directory not found, skipping APK packaging. Outputting .apk.dir as fallback.");
        }
    }
    else {
        eprintln!("Warning: neither gradle nor gradlew is available, skipping APK packaging. Outputting .apk.dir as fallback.");
    }

    Ok(vec![apk_dir])
}

/// 生成调试用 keystore
///
/// 如果 keytool 可用且 app 目录下不存在 debug.keystore，则自动生成。
fn generate_debug_keystore(app_dir: &PathBuf) -> GResult<()> {
    let keystore_path = app_dir.join("debug.keystore");
    if keystore_path.exists() {
        return Ok(());
    }

    if !utils::is_command_available("keytool") {
        eprintln!("Warning: keytool not found, skipping debug keystore generation");
        return Ok(());
    }

    let result = Command::new("keytool")
        .args([
            "-genkeypair",
            "-v",
            "-keystore",
            &keystore_path.to_string_lossy(),
            "-alias",
            "androiddebugkey",
            "-storepass",
            "android",
            "-keypass",
            "android",
            "-keyalg",
            "RSA",
            "-keysize",
            "2048",
            "-validity",
            "10000",
            "-dname",
            "CN=Android Debug,O=Android,C=US",
        ])
        .status();

    match result {
        Ok(status) if status.success() => {
            println!("Generated debug keystore at '{}'", keystore_path.display());
        }
        Ok(_) => {
            eprintln!("Warning: keytool failed to generate debug keystore");
        }
        Err(e) => {
            eprintln!("Warning: keytool execution failed: {}", e);
        }
    }

    Ok(())
}

/// Android 运行
///
/// 通过 adb 安装 APK 并启动 Activity。
/// 支持通过 RunContext args 指定目标设备序列号。
fn run_android(ctx: &RunContext) -> GResult<()> {
    if !utils::is_command_available("adb") {
        return Err(GError {
            kind: GErrorKind::Platform,
            message: "adb is not available. Please install Android SDK platform-tools.".to_string(),
        });
    }

    let device_serial = ctx.args.first();

    let apk_path = ctx.executable_path.display().to_string();

    let install_result = if let Some(serial) = device_serial {
        Command::new("adb").args(["-s", serial, "install", "-r", &apk_path]).status()
    }
    else {
        Command::new("adb").args(["install", "-r", &apk_path]).status()
    };

    let install_status = install_result
        .map_err(|e| GError { kind: GErrorKind::Platform, message: format!("Failed to run adb install: {}", e) })?;

    if !install_status.success() {
        return Err(GError { kind: GErrorKind::Platform, message: "Failed to install APK on device".to_string() });
    }

    let launch_result = if let Some(serial) = device_serial {
        Command::new("adb").args(["-s", serial, "shell", "am", "start", "-n", "com.gg.game/.MainActivity"]).status()
    }
    else {
        Command::new("adb").args(["shell", "am", "start", "-n", "com.gg.game/.MainActivity"]).status()
    };

    let launch_status = launch_result
        .map_err(|e| GError { kind: GErrorKind::Platform, message: format!("Failed to launch activity: {}", e) })?;

    if !launch_status.success() {
        return Err(GError { kind: GErrorKind::Platform, message: "Failed to launch activity on device".to_string() });
    }

    Ok(())
}

/// 列出 Android 设备
///
/// 通过 adb devices -l 获取已连接设备列表。
fn list_android_devices() -> GResult<Vec<DeviceInfo>> {
    if !utils::is_command_available("adb") {
        return Ok(Vec::new());
    }

    let output = Command::new("adb")
        .args(["devices", "-l"])
        .output()
        .map_err(|e| GError { kind: GErrorKind::Platform, message: format!("Failed to list devices: {}", e) })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for line in stdout.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let id = parts[0].to_string();
            let state = parts[1].to_string();
            let name =
                if parts.len() > 3 { parts[3].trim_start_matches("model:").to_string() } else { "Android Device".to_string() };

            if state == "device" || state == "offline" {
                devices.push(DeviceInfo { id, name, platform: "android".to_string(), state });
            }
        }
    }

    Ok(devices)
}

/// 验证 Android 开发环境
fn validate_android_environment() -> GResult<EnvironmentReport> {
    let mut available = Vec::new();
    let mut missing = Vec::new();
    let mut warnings = Vec::new();

    let tools = ["adb", "java", "gradle", "keytool"];

    for tool in tools {
        if utils::is_command_available(tool) {
            available.push(tool.to_string());
        }
        else {
            missing.push(tool.to_string());
            if tool == "adb" {
                warnings
                    .push("adb is not available, Android device deployment requires Android SDK platform-tools".to_string());
            }
            else if tool == "java" {
                warnings.push("java is not available, Gradle builds require JDK".to_string());
            }
        }
    }

    if std::env::var("ANDROID_HOME").is_err() && std::env::var("ANDROID_SDK_ROOT").is_err() {
        warnings.push("ANDROID_HOME or ANDROID_SDK_ROOT environment variable is not set".to_string());
    }

    Ok(EnvironmentReport { available_tools: available, missing_tools: missing, warnings })
}
