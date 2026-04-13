#? 平台跳跃游戏精灵着色器
namespace gg_platformer::shaders {
    using gg_shader::f32::{vec2, vec4, mat44, tex2}

    shader PlatformerSprite by Unlit {
        #? 主精灵纹理
        let _MainTex: texture = "white"
        #? 着色颜色
        let _Color: color = [1, 1, 1, 1]
        #? 水平翻转
        let _FlipX: f32 = 0.0 @range(0.0, 1.0)
        #? 垂直翻转
        let _FlipY: f32 = 0.0 @range(0.0, 1.0)

        render_queue = "transparent"

        render_states {
            cull_mode = "none"
            blend_mode = "alpha"
            depth_test = false
            depth_write = false
        }

        uniforms {
            #? 模型矩阵
            gg_model_matrix: mat44
            #? 视图投影矩阵
            gg_view_projection: mat44
            #? 主纹理
            _MainTex: tex2
            #? 着色颜色
            _Color: vec4
            #? 水平翻转
            _FlipX: f32
            #? 垂直翻转
            _FlipY: f32
        }

        vertex(@location(0) position: vec2, @location(1) uv: vec2) -> @builtin(position) vec4 {
            let mut out_uv = uv
            if _FlipX > 0.5 {
                out_uv.x = 1.0 - out_uv.x
            }
            if _FlipY > 0.5 {
                out_uv.y = 1.0 - out_uv.y
            }
            let world_pos = gg_model_matrix * vec4(position, 0.0, 1.0)
            return gg_view_projection * world_pos
        }

        fragment(@location(0) uv: vec2) -> @location(0) vec4 {
            let tex_color = texture_sample(_MainTex, uv)
            return tex_color * _Color
        }
    }
}
