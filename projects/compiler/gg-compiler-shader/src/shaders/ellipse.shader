shader EllipseShader by Unlit {
    render_states {
        cull_mode = "none"
        blend_mode = "alpha"
        depth_test = false
        depth_write = false
    }

    structure Uniforms {
        mvp: mat44
        ellipse_params: vec4
        fill_color: vec4
        border_params: vec4
    }

    @group(0) @binding(0)
    let uniforms: Uniforms

    @vertex
    micro vertex_main(
        @location(0) position: vec2,
        @location(1) uv: vec2
    ) -> @builtin(position) vec4 {
        let mvp = uniforms.mvp
        return mvp * vec4(position, 0.0, 1.0)
    }

    @fragment
    micro fragment_main(
        @location(0) uv: vec2
    ) -> @location(0) vec4 {
        let center = uniforms.ellipse_params.xy
        let radii = uniforms.ellipse_params.zw
        let border_width = uniforms.border_params.x
        let border_opacity = uniforms.border_params.y

        let d = length((uv - center) / radii) - 1.0

        if d > border_width {
            discard
        }

        let aa = 0.01
        let outer_alpha = smoothstep(border_width, border_width - aa, d)
        let inner_alpha = smoothstep(0.0, -aa, d)
        let border_t = smoothstep(border_width, 0.0, d)

        let alpha = mix(uniforms.fill_color.a * border_opacity * border_t, uniforms.fill_color.a, inner_alpha) * outer_alpha

        return vec4(uniforms.fill_color.rgb, alpha)
    }
}
