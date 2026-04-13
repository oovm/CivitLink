# GG Shader Language

using gg_shader::f32::{vec2, vec4, mat44, tex2}

#? Basic UI rendering shader for buttons, panels, icons etc.
shader UiUnlitShader by UiUnlit {
    _MainTex: texture = "white"
    _Color: color = [1, 1, 1, 1]

    render_queue = "transparent"

    render_states {
        cull_mode = "none"
        blend_mode = "alpha"
        depth_test = false
        depth_write = false
    }

    vertex(
        @location(0) position: vec2,
        @location(1) uv: vec2
    ) -> @builtin(position) vec4 {
        let clip_rect = uniforms.gg_ui_clip_rect
        let pos = uniforms.gg_ui_matrix * vec4(position, 0.0, 1.0)
        return pos
    }

    fragment(
        @location(0) uv: vec2
    ) -> @location(0) vec4 {
        let tex = texture_sample(uniforms._MainTex, uv)
        let color = tex * uniforms._Color
        return color
    }

    uniforms {
        gg_ui_matrix: mat44
        gg_ui_clip_rect: vec4
        _MainTex: tex2
        _Color: vec4
    }
}
