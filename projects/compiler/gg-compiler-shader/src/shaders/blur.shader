# using gg_shader::f32::{vec2, vec3, vec4, mat44};

#? Gaussian blur post-processing shader
shader BlurPostProcess by Unlit {
    _MainTex: texture = "white"
    _Direction: vec2 = [1, 0]
    _BlurSize: f32 = 1.0 @range(0.0, 10.0)

    render_states {
        cull_mode = "none"
        blend_mode = "alpha"
        depth_test = false
        depth_write = false
    }

    structure Uniforms {
        mvp: mat44
        direction: vec2
        blur_size: f32
        texel_size: vec2
    }

    @group(0) @binding(0)
    var<uniform> uniforms: Uniforms

    @group(1) @binding(0)
    var tex_sampler: sampler

    @group(1) @binding(1)
    var tex: texture_2d<f32>

    @vertex
    fn vertex_main(
        @location(0) position: vec2,
        @location(1) uv: vec2
    ) -> @builtin(position) vec4 {
        return uniforms.mvp * vec4(position, 0.0, 1.0)
    }

    @fragment
    fn fragment_main(
        @location(0) uv: vec2
    ) -> @location(0) vec4 {
        let weights = array<f32, 5>(
            0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216
        );

        let dir = uniforms.direction * uniforms.blur_size * uniforms.texel_size;
        var result = textureSample(tex, tex_sampler, uv) * weights[0];

        for (var i = 1; i < 5; i = i + 1) {
            let offset = dir * f32(i);
            result = result + textureSample(tex, tex_sampler, uv + offset) * weights[i];
            result = result + textureSample(tex, tex_sampler, uv - offset) * weights[i];
        }

        return result
    }
}
