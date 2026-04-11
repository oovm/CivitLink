shader RoundedRectShader by Unlit {
    render_states {
        cull_mode = "none"
        blend_mode = "alpha"
        depth_test = false
        depth_write = false
    }

    structure Uniforms {
        mvp: mat44
        rect_size: vec4
        color: vec4
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
        let size = uniforms.rect_size.xy
        let radius = uniforms.rect_size.z

        let half_size = size * 0.5
        let p = uv * size - half_size

        let q = abs(p) - half_size + radius
        let dist = length(max(q, vec2(0.0, 0.0))) + min(max(q.x, q.y), 0.0) - radius

        let aa = 1.0
        let alpha = 1.0 - smoothstep(-aa, aa, dist)

        if (dist > aa) {
            discard
        }

        return vec4(uniforms.color.rgb, uniforms.color.a * alpha)
    }
}
