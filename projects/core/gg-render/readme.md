# GG Render

GG 引擎渲染硬件抽象层（HAL），提供与具体图形 API 无关的渲染抽象接口。

## 功能

- 颜色类型（`Color`）
- 矩形区域（`Rect`）
- 二维变换（`Transform`）
- 纹理标识与描述（`TextureId`、`TextureDescriptor`、`PixelFormat`）
- 渲染表面与窗口事件（`SurfaceInfo`、`WindowEvent`）
- 绘制命令（`DrawCommand`、`TransitionKind`）
- 渲染上下文与渲染器 trait（`RenderContext`、`Renderer`）

## 架构

`gg-render` 定义了渲染后端的抽象接口，不依赖任何具体的图形 API 或 ECS 框架。
具体的渲染后端（如 Vulkan、Metal、DirectX、OpenGL 等）通过实现 `Renderer` trait
来提供实际的渲染能力。

## 使用示例

```rust
use gg_render::prelude::*;
use gg_core::GResult;

struct MyRenderer {
    surface_info: SurfaceInfo,
}

impl Renderer for MyRenderer {
    fn begin_frame(&mut self) -> GResult<()> {
        Ok(())
    }

    fn end_frame(&mut self) -> GResult<()> {
        Ok(())
    }

    fn draw(&mut self, context: &RenderContext) -> GResult<()> {
        for cmd in context.commands() {
            // 处理绘制命令
        }
        Ok(())
    }

    fn present(&mut self) -> GResult<()> {
        Ok(())
    }

    fn load_texture(&mut self, path: &std::path::Path) -> GResult<TextureId> {
        Ok(TextureId::new(1))
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.surface_info.width = width;
        self.surface_info.height = height;
    }

    fn surface_info(&self) -> &SurfaceInfo {
        &self.surface_info
    }
}
```
