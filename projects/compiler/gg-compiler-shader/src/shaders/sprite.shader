# using gg_shader::f32::{vec2, vec3, vec4, mat44};

shader SpriteShader by Unlit {
    render_states {
        cull_mode = "none"
        blend_mode = "alpha"
        depth_test = false
        depth_write = false
    }

    structure Uniforms {
        mvp: mat44
        tint: vec4
        uv_transform: vec4
    }

    @group(0) @binding(0)
    let uniforms: Uniforms

    @group(1) @binding(0)
    let tex_sampler: sampler

    @group(1) @binding(1)
    let tex: texture_2d

    @vertex
    micro vertex_main(
        @location(0) position: vec2,
        @location(1) uv: vec2
    ) -> @builtin(position) vec4 {
        let mvp = uniforms.mvp
        let uv_offset = vec2(uniforms.uv_transform.x, uniforms.uv_transform.y)
        let uv_scale = vec2(uniforms.uv_transform.z, uniforms.uv_transform.w)
        let out_uv = uv_offset + uv * uv_scale
        return mvp * vec4(position, 0.0, 1.0)
    }

    @fragment
    micro fragment_main(
        @location(0) uv: vec2
    ) -> @location(0) vec4 {
        return textureSample(tex, tex_sampler, uv) * uniforms.tint
    }
}
