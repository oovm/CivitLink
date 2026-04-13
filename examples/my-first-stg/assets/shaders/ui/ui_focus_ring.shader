# GG Shader Language

using gg_shader::f32::{vec2, vec4, mat44, tex2}

#? Focus ring rendering shader for accessibility focus indicator
shader UiFocusRingShader by UiCustom {
    _Color: color = [0, 0.4, 1, 1]
    _Width: f32 = 2.0
    _Offset: f32 = 2.0
    _CornerRadius: f32 = 4.0

    render_queue = "overlay"

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
        return uniforms.gg_ui_matrix * vec4(position, 0.0, 1.0)
    }

    fragment(
        @location(0) uv: vec2
    ) -> @location(0) vec4 {
        let rect = uniforms.gg_ui_clip_rect
        let offset = uniforms._Offset
        let width = uniforms._Width

        let outer_left = rect.x - offset
        let outer_top = rect.y - offset
        let outer_right = rect.x + rect.z + offset
        let outer_bottom = rect.y + rect.w + offset

        let inner_left = outer_left + width
        let inner_top = outer_top + width
        let inner_right = outer_right - width
        let inner_bottom = outer_bottom - width

        let is_outer = uv.x >= outer_left && uv.x <= outer_right && uv.y >= outer_top && uv.y <= outer_bottom
        let is_inner = uv.x >= inner_left && uv.x <= inner_right && uv.y >= inner_top && uv.y <= inner_bottom
        let is_ring = is_outer && !is_inner

        if !is_ring {
            discard
        }

        return uniforms._Color
    }

    uniforms {
        gg_ui_matrix: mat44
        gg_ui_clip_rect: vec4
        _Color: vec4
        _Width: f32
        _Offset: f32
        _CornerRadius: f32
    }
}
