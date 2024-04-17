#![warn(missing_docs)]

//! Shader 产物模块
//!
//! 定义着色器编译产物的数据结构，包括渲染状态、剔除模式、混合模式等，
//! 以及着色器产物的序列化与反序列化功能。

use gg_core::{GError, GErrorKind, GResult};
use naga;

use crate::serialize;

/// 剔除模式
///
/// 定义渲染时三角面的剔除方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullMode {
    /// 不剔除任何面
    None,
    /// 剔除正面
    Front,
    /// 剔除背面
    Back,
}

impl CullMode {
    /// 将剔除模式编码为 u8 标志
    pub fn to_u8(self) -> u8 {
        match self {
            CullMode::None => 0,
            CullMode::Front => 1,
            CullMode::Back => 2,
        }
    }

    /// 从 u8 标志解码剔除模式
    pub fn from_u8(val: u8) -> GResult<Self> {
        match val {
            0 => Ok(CullMode::None),
            1 => Ok(CullMode::Front),
            2 => Ok(CullMode::Back),
            _ => Err(GError { kind: GErrorKind::Other, message: format!("无效的 CullMode 值: {}", val) }),
        }
    }
}

/// 混合模式
///
/// 定义渲染时像素的混合方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    /// 不透明，无混合
    Opaque,
    /// Alpha 混合
    Alpha,
    /// 加法混合
    Additive,
    /// 乘法混合
    Multiply,
}

impl BlendMode {
    /// 将混合模式编码为 u8 标志
    pub fn to_u8(self) -> u8 {
        match self {
            BlendMode::Opaque => 0,
            BlendMode::Alpha => 1,
            BlendMode::Additive => 2,
            BlendMode::Multiply => 3,
        }
    }

    /// 从 u8 标志解码混合模式
    pub fn from_u8(val: u8) -> GResult<Self> {
        match val {
            0 => Ok(BlendMode::Opaque),
            1 => Ok(BlendMode::Alpha),
            2 => Ok(BlendMode::Additive),
            3 => Ok(BlendMode::Multiply),
            _ => Err(GError { kind: GErrorKind::Other, message: format!("无效的 BlendMode 值: {}", val) }),
        }
    }
}

/// 渲染状态
///
/// 定义着色器在渲染管线中使用的各种渲染状态配置。
#[derive(Debug, Clone)]
pub struct RenderStates {
    /// 剔除模式
    pub cull_mode: CullMode,
    /// 混合模式
    pub blend_mode: BlendMode,
    /// 是否启用深度测试
    pub depth_test: bool,
    /// 是否启用深度写入
    pub depth_write: bool,
    /// 是否启用线框模式
    pub wireframe: bool,
    /// 是否启用模板测试
    pub stencil_test: bool,
    /// 是否启用多重采样
    pub multisample: bool,
}

impl Default for RenderStates {
    fn default() -> Self {
        Self {
            cull_mode: CullMode::Back,
            blend_mode: BlendMode::Opaque,
            depth_test: true,
            depth_write: true,
            wireframe: false,
            stencil_test: false,
            multisample: false,
        }
    }
}

impl RenderStates {
    /// 根据着色器类型返回合适的默认渲染状态
    ///
    /// 不同类型的着色器有不同的渲染状态默认值：
    /// - PBR：不透明渲染，开启深度测试和写入，背面剔除
    /// - Unlit：不透明渲染，开启深度测试但不写入
    /// - UiUnlit/UiSdf/UiCustom：Alpha 混合，无深度，无剔除
    pub fn default_for_kind(kind: &str) -> Self {
        match kind {
            "PBR" => Self {
                cull_mode: CullMode::Back,
                blend_mode: BlendMode::Opaque,
                depth_test: true,
                depth_write: true,
                wireframe: false,
                stencil_test: false,
                multisample: false,
            },
            "Unlit" => Self {
                cull_mode: CullMode::Back,
                blend_mode: BlendMode::Opaque,
                depth_test: true,
                depth_write: false,
                wireframe: false,
                stencil_test: false,
                multisample: false,
            },
            "UiUnlit" | "UiSdf" | "UiCustom" => Self {
                cull_mode: CullMode::None,
                blend_mode: BlendMode::Alpha,
                depth_test: false,
                depth_write: false,
                wireframe: false,
                stencil_test: false,
                multisample: false,
            },
            _ => Self::default(),
        }
    }

    /// 将渲染状态编码为 u8 标志位数组
    ///
    /// 编码布局：
    /// - byte 0: cull_mode (0-2)
    /// - byte 1: blend_mode (0-3)
    /// - byte 2: 位标志 (bit0: depth_test, bit1: depth_write, bit2: wireframe, bit3: stencil_test, bit4: multisample)
    fn to_bytes(&self) -> [u8; 3] {
        let flags = (self.depth_test as u8)
            | ((self.depth_write as u8) << 1)
            | ((self.wireframe as u8) << 2)
            | ((self.stencil_test as u8) << 3)
            | ((self.multisample as u8) << 4);
        [self.cull_mode.to_u8(), self.blend_mode.to_u8(), flags]
    }

    /// 从 u8 标志位数组解码渲染状态
    fn from_bytes(bytes: [u8; 3]) -> GResult<Self> {
        let cull_mode = CullMode::from_u8(bytes[0])?;
        let blend_mode = BlendMode::from_u8(bytes[1])?;
        let flags = bytes[2];
        Ok(Self {
            cull_mode,
            blend_mode,
            depth_test: (flags & 0x01) != 0,
            depth_write: (flags & 0x02) != 0,
            wireframe: (flags & 0x04) != 0,
            stencil_test: (flags & 0x08) != 0,
            multisample: (flags & 0x10) != 0,
        })
    }
}

/// 回退信息
///
/// 定义着色器回退条件和回退目标着色器。
#[derive(Debug, Clone)]
pub struct FallbackInfo {
    /// 回退条件描述
    pub condition: String,
    /// 回退目标着色器名称
    pub fallback_shader: String,
}

/// 着色器模块条目
///
/// 单个着色器块的编译产物，包含名称、类型、
/// 编译后的 naga IR 模块、渲染状态配置和可选的回退信息。
#[derive(Debug, Clone)]
pub struct ShaderModuleEntry {
    /// 着色器名称
    pub name: String,
    /// 着色器类型
    pub kind: String,
    /// 编译后的 naga IR 模块
    pub module: naga::Module,
    /// 渲染状态配置
    pub render_states: RenderStates,
    /// 可选的回退信息
    pub fallback: Option<FallbackInfo>,
}

/// 着色器产物
///
/// 着色器编译的最终产物，包含所有编译后的着色器模块。
#[derive(Debug, Clone)]
pub struct ShaderArtifact {
    /// 所有编译后的着色器模块
    pub modules: Vec<ShaderModuleEntry>,
}

/// 序列化魔数标识
const MAGIC: [u8; 4] = [b'G', b'G', b'S', b'A'];

impl ShaderArtifact {
    /// 将着色器产物序列化为二进制数据
    ///
    /// 二进制格式布局：
    /// - 4 字节魔数 "GGSA"
    /// - 2 字节模块数量
    /// - 对于每个模块：
    ///   - 2 字节名称长度 + 名称 UTF-8 字节
    ///   - 2 字节类型长度 + 类型 UTF-8 字节
    ///   - 3 字节渲染状态标志
    ///   - 1 字节回退标志 (1=存在, 0=不存在)
    ///     - 若存在：2 字节条件长度 + 条件 UTF-8 字节 + 2 字节着色器长度 + 着色器 UTF-8 字节
    ///   - 4 字节 naga 数据长度 + naga SPIR-V 数据
    pub fn serialize(&self) -> GResult<Vec<u8>> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&MAGIC);

        let module_count = self.modules.len() as u16;
        buf.extend_from_slice(&module_count.to_le_bytes());

        for entry in &self.modules {
            let naga_bytes = serialize::serialize_module(&entry.module)?;

            let name_bytes = entry.name.as_bytes();
            let kind_bytes = entry.kind.as_bytes();

            let name_len = name_bytes.len() as u16;
            let kind_len = kind_bytes.len() as u16;
            let naga_len = naga_bytes.len() as u32;

            buf.extend_from_slice(&name_len.to_le_bytes());
            buf.extend_from_slice(name_bytes);
            buf.extend_from_slice(&kind_len.to_le_bytes());
            buf.extend_from_slice(kind_bytes);
            buf.extend_from_slice(&entry.render_states.to_bytes());

            if let Some(ref fb) = entry.fallback {
                buf.push(1u8);
                let cond_bytes = fb.condition.as_bytes();
                let shader_bytes = fb.fallback_shader.as_bytes();
                let cond_len = cond_bytes.len() as u16;
                let shader_len = shader_bytes.len() as u16;
                buf.extend_from_slice(&cond_len.to_le_bytes());
                buf.extend_from_slice(cond_bytes);
                buf.extend_from_slice(&shader_len.to_le_bytes());
                buf.extend_from_slice(shader_bytes);
            }
            else {
                buf.push(0u8);
            }

            buf.extend_from_slice(&naga_len.to_le_bytes());
            buf.extend_from_slice(&naga_bytes);
        }

        Ok(buf)
    }

    /// 从二进制数据反序列化着色器产物
    ///
    /// 解析由 `serialize` 方法生成的二进制数据，重建着色器产物。
    pub fn deserialize(bytes: &[u8]) -> GResult<Self> {
        let mut offset = 0usize;

        if bytes.len() < 4 {
            return Err(GError { kind: GErrorKind::Other, message: "数据过短，无法读取魔数".to_string() });
        }

        if bytes[offset..offset + 4] != MAGIC {
            return Err(GError { kind: GErrorKind::Other, message: "无效的 ShaderArtifact 魔数".to_string() });
        }
        offset += 4;

        if bytes.len() < offset + 2 {
            return Err(GError { kind: GErrorKind::Other, message: "数据过短，无法读取模块数量".to_string() });
        }

        let module_count = u16::from_le_bytes(
            bytes[offset..offset + 2]
                .try_into()
                .map_err(|_| GError { kind: GErrorKind::Other, message: "读取模块数量失败".to_string() })?,
        ) as usize;
        offset += 2;

        let mut modules = Vec::with_capacity(module_count);

        for _ in 0..module_count {
            let name_len = u16::from_le_bytes(
                bytes[offset..offset + 2]
                    .try_into()
                    .map_err(|_| GError { kind: GErrorKind::Other, message: "读取名称长度失败".to_string() })?,
            ) as usize;
            offset += 2;

            let name = String::from_utf8(bytes[offset..offset + name_len].to_vec())
                .map_err(|e| GError { kind: GErrorKind::Other, message: format!("名称 UTF-8 解码失败: {}", e) })?;
            offset += name_len;

            let kind_len = u16::from_le_bytes(
                bytes[offset..offset + 2]
                    .try_into()
                    .map_err(|_| GError { kind: GErrorKind::Other, message: "读取类型长度失败".to_string() })?,
            ) as usize;
            offset += 2;

            let kind = String::from_utf8(bytes[offset..offset + kind_len].to_vec())
                .map_err(|e| GError { kind: GErrorKind::Other, message: format!("类型 UTF-8 解码失败: {}", e) })?;
            offset += kind_len;

            let render_states_bytes: [u8; 3] = bytes[offset..offset + 3]
                .try_into()
                .map_err(|_| GError { kind: GErrorKind::Other, message: "读取渲染状态失败".to_string() })?;
            offset += 3;

            let render_states = RenderStates::from_bytes(render_states_bytes)?;

            let has_fallback = bytes[offset];
            offset += 1;

            let fallback = if has_fallback == 1 {
                let cond_len = u16::from_le_bytes(
                    bytes[offset..offset + 2]
                        .try_into()
                        .map_err(|_| GError {
                            kind: GErrorKind::Other, message: "读取回退条件长度失败".to_string()
                        })?,
                ) as usize;
                offset += 2;

                let condition = String::from_utf8(bytes[offset..offset + cond_len].to_vec())
                    .map_err(|e| GError {
                        kind: GErrorKind::Other, message: format!("回退条件 UTF-8 解码失败: {}", e)
                    })?;
                offset += cond_len;

                let shader_len = u16::from_le_bytes(
                    bytes[offset..offset + 2]
                        .try_into()
                        .map_err(|_| GError {
                            kind: GErrorKind::Other, message: "读取回退着色器长度失败".to_string()
                        })?,
                ) as usize;
                offset += 2;

                let fallback_shader = String::from_utf8(bytes[offset..offset + shader_len].to_vec())
                    .map_err(|e| GError {
                        kind: GErrorKind::Other, message: format!("回退着色器 UTF-8 解码失败: {}", e)
                    })?;
                offset += shader_len;

                Some(FallbackInfo { condition, fallback_shader })
            }
            else {
                None
            };

            let naga_len = u32::from_le_bytes(
                bytes[offset..offset + 4]
                    .try_into()
                    .map_err(|_| GError { kind: GErrorKind::Other, message: "读取 naga 数据长度失败".to_string() })?,
            ) as usize;
            offset += 4;

            let naga_bytes = &bytes[offset..offset + naga_len];
            let module = serialize::deserialize_module(naga_bytes)?;
            offset += naga_len;

            modules.push(ShaderModuleEntry { name, kind, module, render_states, fallback });
        }

        Ok(Self { modules })
    }

    /// 返回第一个着色器模块条目的引用
    pub fn first_module(&self) -> Option<&ShaderModuleEntry> {
        self.modules.first()
    }

    /// 返回第一个着色器模块条目，消费自身
    pub fn into_first_module(self) -> Option<ShaderModuleEntry> {
        self.modules.into_iter().next()
    }
}
