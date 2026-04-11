# GG Shader Language

using gg_shader::f32::{vec2, vec3, vec4, mat44, tex2}

#? SDF text rendering shader for high-quality text with outline, shadow and glow
shader UiSdfShader by UiSdf {
    _MainTex: texture = "white"
    _Color: color = [1, 1, 1, 1]
    _OutlineColor: color = [0, 0, 0, 1]
    _OutlineWidth: f32 = 0.1 @range(0.0, 0.5)
    _ShadowColor: color = [0, 0, 0, 0.5]
    _ShadowOffset: vec2 = [1, -1]

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
        return uniforms.gg_ui_matrix * vec4(position, 0.0, 1.0)
    }

    fragment(
        @location(0) uv: vec2
    ) -> @location(0) vec4 {
        let dist = texture_sample(uniforms._MainTex, uv).r
        let smooth_min = uniforms.gg_sdf_params.x
        let smooth_max = uniforms.gg_sdf_params.y

        let outline_dist = smooth_min - uniforms._OutlineWidth
        let body_alpha = smoothstep(smooth_min, smooth_max, dist)
        let outline_alpha = smoothstep(outline_dist, smooth_min, dist)

        let shadow_uv = uv - uniforms._ShadowOffset / uniforms.gg_sdf_params.z
        let shadow_dist = texture_sample(uniforms._MainTex, shadow_uv).r
        let shadow_alpha = smoothstep(smooth_min, smooth_max, shadow_dist) * uniforms._ShadowColor.a

        let color = mix(uniforms._OutlineColor, uniforms._Color, body_alpha)
        let alpha = max(outline_alpha, shadow_alpha)

        return vec4(color.rgb, alpha)
    }

    uniforms {
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
}
