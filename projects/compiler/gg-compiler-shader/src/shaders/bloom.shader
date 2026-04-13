# using gg_shader::f32::{vec2, vec3, vec4, mat44};

#? Bloom post-processing shader (brightness extraction + combine)
shader BloomPostProcess by Unlit {
    _MainTex: texture = "white"
    _BloomTex: texture = "black"
    _Threshold: f32 = 0.8 @range(0.0, 2.0)
    _Intensity: f32 = 1.0 @range(0.0, 3.0)

    render_states {
        cull_mode = "none"
        blend_mode = "alpha"
        depth_test = false
        depth_write = false
    }

    structure Uniforms {
        mvp: mat44
        threshold: f32
        intensity: f32
        texel_size: vec2
    }

    @group(0) @binding(0)
    var<uniform> uniforms: Uniforms

    @group(1) @binding(0)
    var tex_sampler: sampler

    @group(1) @binding(1)
    var main_tex: texture_2d<f32>

    @group(2) @binding(0)
    var bloom_sampler: sampler

    @group(2) @binding(1)
    var bloom_tex: texture_2d<f32>

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
        let main_color = textureSample(main_tex, tex_sampler, uv);
        let bloom_color = textureSample(bloom_tex, bloom_sampler, uv);

        let brightness = dot(main_color.rgb, vec3(0.2126, 0.7152, 0.0722));
        let contribution = step(uniforms.threshold, brightness);

        let result = main_color + bloom_color * uniforms.intensity;

        return vec4(result.rgb, main_color.a)
    }
}
