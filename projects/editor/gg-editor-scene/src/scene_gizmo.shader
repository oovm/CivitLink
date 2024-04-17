# using gg_shader::f32::{vec2, vec4, mat44};

shader SceneGizmoShader by UiCustom {
    render_states {
        cull_mode = "none"
        blend_mode = "alpha"
        depth_test = false
        depth_write = false
    }

    let gizmo_mode: u32 = 0

    structure Uniforms {
        gg_ui_matrix: mat44
        gg_ui_clip_rect: vec4
        axis_x_color: vec4
        axis_y_color: vec4
        center_color: vec4
        active_color: vec4
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
            return uniforms.axis_x_color
        }
        if vertex_mode == 1u {
            return uniforms.axis_y_color
        }
        if vertex_mode == 2u {
            return uniforms.center_color
        }
        if vertex_mode == 3u {
            return uniforms.active_color
        }
        return uniforms.center_color
    }
}
