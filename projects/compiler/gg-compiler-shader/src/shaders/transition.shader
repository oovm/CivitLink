shader TransitionShader by Unlit {
    render_states {
        cull_mode = "none"
        blend_mode = "alpha"
        depth_test = false
        depth_write = false
    }

    structure Uniforms {
        mvp: mat44
        params: vec4
        tint: vec4
    }

    @group(0) @binding(0)
    let uniforms: Uniforms

    @group(1) @binding(0)
    let tex_sampler: sampler

    @group(1) @binding(1)
    let old_tex: texture_2d

    @group(1) @binding(2)
    let new_tex: texture_2d

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
        let old_color = textureSample(old_tex, tex_sampler, uv)
        let new_color = textureSample(new_tex, tex_sampler, uv)
        let progress = uniforms.params.x
        let mixed_color = mix(old_color, new_color, progress)
        return mixed_color * uniforms.tint
    }
}
