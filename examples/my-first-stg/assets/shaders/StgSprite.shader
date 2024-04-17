shader StgSprite by Sprite

uniforms {
    texture: texture_2d<f32>,
    tint: vec4<f32>,
    alpha: f32,
}

vertex fn main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = projection_matrix * model_matrix * vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}

fragment fn main(in: FragmentOutput) -> vec4<f32> {
    let tex_color = textureSample(texture, sampler_linear, in.uv);
    let final_color = tex_color * tint;
    let final_alpha = final_color.a * alpha;
    if final_alpha <= 0.01 { discard; }
    return vec4<f32>(final_color.rgb, final_alpha);
}
