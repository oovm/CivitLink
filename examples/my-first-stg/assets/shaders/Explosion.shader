shader Explosion by Effect

uniforms {
    time: f32,
    duration: f32,
    center: vec2<f32>,
    scale: f32,
}

vertex fn main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let progress = time / duration;
    let expand = 1.0 + progress * 2.0 * scale;
    let pos = (in.position - center) * expand + center;
    out.position = projection_matrix * view_matrix * vec4<f32>(pos, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}

fragment fn main(in: FragmentOutput) -> vec4<f32> {
    let progress = time / duration;
    let dist = distance(in.uv, vec2<f32>(0.5, 0.5));
    let alpha = (1.0 - progress) * (1.0 - dist);
    if alpha <= 0.0 { discard; }
    let r = 1.0;
    let g = 1.0 - progress * 0.7;
    let b = 0.2 * (1.0 - progress);
    return vec4<f32>(r, g, b, alpha);
}
