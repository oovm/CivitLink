#? 平台跳跃游戏特效着色器
namespace gg_platformer::shaders {
    using gg_shader::f32::{vec2, vec4, mat44, tex2}

    shader PlatformerEffect by Unlit {
        #? 主纹理
        let _MainTex: texture = "white"
        #? 基础颜色
        let _Color: color = [1, 1, 1, 1]
        #? 闪烁叠加颜色（受伤效果）
        let _FlashColor: color = [1, 1, 1, 1]
        #? 闪烁强度（0=无闪烁，1=全闪烁）
        let _FlashAmount: f32 = 0.0 @range(0.0, 1.0)
        #? 淡出程度（0=完全透明，1=完全不透明）
        let _FadeAmount: f32 = 1.0 @range(0.0, 1.0)
        #? 闪烁速度（收集物微光效果，0=无闪烁）
        let _BlinkSpeed: f32 = 0.0 @range(0.0, 20.0)

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
            #? 时间
            gg_time: f32
            #? 主纹理
            _MainTex: tex2
            #? 基础颜色
            _Color: vec4
            #? 闪烁叠加颜色
            _FlashColor: vec4
            #? 闪烁强度
            _FlashAmount: f32
            #? 淡出程度
            _FadeAmount: f32
            #? 闪烁速度
            _BlinkSpeed: f32
        }

        vertex(@location(0) position: vec2, @location(1) uv: vec2) -> @builtin(position) vec4 {
            let world_pos = gg_model_matrix * vec4(position, 0.0, 1.0)
            return gg_view_projection * world_pos
        }

        fragment(@location(0) uv: vec2) -> @location(0) vec4 {
            let tex_color = texture_sample(_MainTex, uv)
            let base_color = tex_color * _Color
            let blink = sin(gg_time * _BlinkSpeed) * 0.5 + 0.5
            let blink_factor = mix(1.0, blink, step(0.001, _BlinkSpeed))
            let blinked_color = base_color * blink_factor
            let flashed_color = mix(blinked_color, _FlashColor, _FlashAmount)
            return vec4(flashed_color.rgb, flashed_color.a * _FadeAmount)
        }
    }
}
