shader PowerUpGlow by Effect

uniforms {
    time: f32,
    duration: f32,
    center: vec2<f32>,
    color: vec3<f32>,
}

vertex fn main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let pulse = 1.0 + sin(time * 6.0) * 0.15;
    let pos = (in.position - center) * pulse + center;
    out.position = projection_matrix * view_matrix * vec4<f32>(pos, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}

fragment fn main(in: FragmentOutput) -> vec4<f32> {
    let progress = time / duration;
    let dist = distance(in.uv, vec2<f32>(0.5, 0.5));
    let glow = exp(-dist * 4.0) * (1.0 - progress);
    let pulse_alpha = 0.6 + sin(time * 10.0) * 0.3;
    let alpha = glow * pulse_alpha;
    if alpha <= 0.01 { discard; }
    return vec4<f32>(color.r, color.g, color.b, alpha);
}
