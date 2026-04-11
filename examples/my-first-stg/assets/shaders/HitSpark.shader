shader HitSpark by Effect

uniforms {
    time: f32,
    duration: f32,
    center: vec2<f32>,
    intensity: f32,
}

vertex fn main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let jitter = vec2<f32>(
        sin(time * 50.0 + in.position.x) * 3.0 * intensity,
        cos(time * 40.0 + in.position.y) * 3.0 * intensity
    );
    let pos = in.position + jitter;
    out.position = projection_matrix * view_matrix * vec4<f32>(pos, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}

fragment fn main(in: FragmentOutput) -> vec4<f32> {
    let progress = time / duration;
    let alpha = (1.0 - progress) * intensity;
    if alpha <= 0.0 { discard; }
    let brightness = 1.5 - progress * 0.8;
    return vec4<f32>(brightness, brightness, brightness * 0.9, alpha);
}
