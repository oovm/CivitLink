shader BossAura by Effect

uniforms {
    time: f32,
    duration: f32,
    center: vec2<f32>,
    radius: f32,
}

vertex fn main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let angle = time * 2.0;
    let rotate = mat2x2<f32>(cos(angle), -sin(angle), sin(angle), cos(angle));
    let offset = in.position - center;
    let rotated = rotate * offset;
    let ring_scale = 1.0 + sin(time * 3.0) * 0.1;
    let pos = rotated * ring_scale + center;
    out.position = projection_matrix * view_matrix * vec4<f32>(pos, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}

fragment fn main(in: FragmentOutput) -> vec4<f32> {
    let progress = time / duration;
    let dist = distance(in.uv, vec2<f32>(0.5, 0.5));
    let ring = smoothstep(0.35, 0.45, dist) * (1.0 - smoothstep(0.45, 0.55, dist));
    let inner_glow = exp(-dist * 3.0) * 0.5;
    let combined = (ring + inner_glow) * (1.0 - progress * 0.3);
    let r = 0.7 + sin(time * 4.0) * 0.3;
    let g = 0.2 + sin(time * 3.0 + 1.0) * 0.15;
    let b = 0.9 + cos(time * 5.0) * 0.1;
    let alpha = combined;
    if alpha <= 0.01 { discard; }
    return vec4<f32>(r, g, b, alpha);
}
