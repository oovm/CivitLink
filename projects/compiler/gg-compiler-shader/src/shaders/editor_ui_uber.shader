# using gg_shader::f32::{vec2, vec3, vec4, mat44};

shader EditorUiUberShader by UiUnlit {
    render_states {
        cull_mode = "none"
        blend_mode = "alpha"
        depth_test = false
        depth_write = false
    }

    let mode: u32 = 0
    let atlas: texture = "white"

    structure Uniforms {
        gg_ui_matrix: mat44
        gg_ui_clip_rect: vec4
        mode: u32
        rect_color: vec4
        text_color: vec4
        tint_color: vec4
        sdf_params: vec3
    }

    @group(0) @binding(0)
    var<uniform> uniforms: Uniforms

    @group(1) @binding(0)
    var atlas_sampler: sampler

    @group(1) @binding(1)
    var atlas: texture_2d<f32>

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
            return uniforms.rect_color
        }
        if vertex_mode == 1u {
            let tex_color = textureSample(atlas, atlas_sampler, uv)
            let distance = tex_color.x
            let smooth_min = uniforms.sdf_params.x
            let smooth_max = uniforms.sdf_params.y
            let alpha = smoothstep(smooth_min, smooth_max, distance)
            return vec4(uniforms.text_color.x, uniforms.text_color.y, uniforms.text_color.z, alpha)
        }
        if vertex_mode == 2u {
            return textureSample(atlas, atlas_sampler, uv)
        }
        if vertex_mode == 3u {
            let tex_color = textureSample(atlas, atlas_sampler, uv)
            return tex_color * uniforms.tint_color
        }
        return uniforms.rect_color
    }
}
