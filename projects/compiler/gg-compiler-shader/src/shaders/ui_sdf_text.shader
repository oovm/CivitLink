# using gg_shader::f32::{vec2, vec3, vec4, mat44};

#? SDF text rendering shader for UI
structure Uniforms {
    gg_ui_matrix: mat44
    gg_ui_clip_rect: vec4
    gg_sdf_params: vec3
    _MainTex: tex2
    _Color: vec4
    _OutlineColor: vec4
    _OutlineWidth: f32
    _ShadowColor: vec4
    _ShadowOffset: vec2
}

shader UiSdfTextShader by UiSdf {
    _MainTex: texture = "white"
    _Color: color = [1, 1, 1, 1]
    _OutlineColor: color = [0, 0, 0, 1]
    @range(0.0, 0.5)
    _OutlineWidth: f32 = 0.1
    _ShadowColor: color = [0, 0, 0, 0.5]
    _ShadowOffset: vec2 = [1, -1]

    render_states {
        cull_mode = "none"
        blend_mode = "alpha"
        depth_test = false
        depth_write = false
    }

    uniform uniforms: Uniforms
    tex_sampler: sampler
    sdf_tex: texture_2d<f32>

    @vertex
    micro vertex_main(
        @location(0) position: vec2,
        @location(1) uv: vec2
    ) -> @builtin(position) vec4 {
        return uniforms.gg_ui_matrix * vec4(position, 0.0, 1.0)
    }

    @fragment
    micro fragment_main(
        @location(0) uv: vec2
    ) -> @location(0) vec4 {
        let dist = textureSample(sdf_tex, tex_sampler, uv).r
        let smooth_min = uniforms.gg_sdf_params.x
        let smooth_max = uniforms.gg_sdf_params.y

        let body_alpha = smoothstep(smooth_min, smooth_max, dist)
        let outline_alpha = smoothstep(smooth_min - uniforms._OutlineWidth, smooth_min, dist)

        let shadow_uv = uv - uniforms._ShadowOffset / uniforms.gg_sdf_params.z
        let shadow_dist = textureSample(sdf_tex, tex_sampler, shadow_uv).r
        let shadow_alpha = smoothstep(smooth_min, smooth_max, shadow_dist) * uniforms._ShadowColor.a

        let color = mix(uniforms._OutlineColor, uniforms._Color, body_alpha)
        let alpha = max(outline_alpha, shadow_alpha)

        return vec4(color.rgb, alpha)
    }
}
