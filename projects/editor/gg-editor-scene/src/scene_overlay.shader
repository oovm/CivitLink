# using gg_shader::f32::{vec2, vec4, mat44};

shader SceneOverlayShader by UiCustom {
    render_states {
        cull_mode = "none"
        blend_mode = "alpha"
        depth_test = false
        depth_write = false
    }

    let mode: u32 = 0

    structure Uniforms {
        gg_ui_matrix: mat44
        gg_ui_clip_rect: vec4
        overlay_color: vec4
        line_width: f32
    }

    @group(0) @binding(0)
    var<uniform> uniforms: Uniforms

    @vertex
    fn vertex_main(
        @location(0) position: vec2,
        @location(1) uv: vec2,
        @location(2) vertex_mode: u32
    ) -> @builtin(position) vec4 {
        let ui_matrix = uniforms.gg_ui_matrix
        return ui_matrix * vec4(position, 0.0, 1.0)
    }

    @fragment
    fn fragment_main(
        @location(0) uv: vec2,
        @location(1) vertex_mode: u32
    ) -> @location(0) vec4 {
        if vertex_mode == 0u {
            return uniforms.overlay_color
        }
        if vertex_mode == 1u {
            let distance = min(uv.x, min(uv.y, min(1.0 - uv.x, 1.0 - uv.y)))
            let half_width = uniforms.line_width * 0.5
            let alpha = 1.0 - smoothstep(half_width - 1.0, half_width + 1.0, distance)
            return vec4(uniforms.overlay_color.x, uniforms.overlay_color.y, uniforms.overlay_color.z, alpha * uniforms.overlay_color.w)
        }
        if vertex_mode == 2u {
            let grid_alpha = 0.3
            return vec4(uniforms.overlay_color.x, uniforms.overlay_color.y, uniforms.overlay_color.z, grid_alpha)
        }
        return uniforms.overlay_color
    }
}
