shader BatchSpriteShader by Unlit {
    render_states {
        cull_mode = "none"
        blend_mode = "alpha"
        depth_test = false
        depth_write = false
    }

    @group(0) @binding(0)
    let tex_sampler: sampler

    @group(0) @binding(1)
    let tex: texture_2d

    @vertex
    micro vertex_main(
        @location(0) position: vec2,
        @location(1) uv: vec2,
        @location(2) mvp_row0: vec4,
        @location(3) mvp_row1: vec4,
        @location(4) mvp_row2: vec4,
        @location(5) mvp_row3: vec4,
        @location(6) tint: vec4,
        @location(7) uv_transform: vec4
    ) -> @builtin(position) vec4 {
        let mvp = mat44(mvp_row0, mvp_row1, mvp_row2, mvp_row3)
        let uv_offset = vec2(uv_transform.x, uv_transform.y)
        let uv_scale = vec2(uv_transform.z, uv_transform.w)
        let out_uv = uv_offset + uv * uv_scale
        return mvp * vec4(position, 0.0, 1.0)
    }

    @fragment
    micro fragment_main(
        @location(0) uv: vec2,
        @location(1) tint: vec4
    ) -> @location(0) vec4 {
        return textureSample(tex, tex_sampler, uv) * tint
    }
}
