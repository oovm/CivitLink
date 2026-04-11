shader StgBullet by InstancedSprite

uniforms {
    texture: texture_2d<f32>,
    instance_count: u32,
    tint: vec4<f32>,
    glow_intensity: f32,
}

vertex fn main(in: InstancedVertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = in.instance_position + in.position * in.instance_scale;
    let rotated_pos = vec2<f32>(
        world_pos.x * cos(in.instance_rotation) - world_pos.y * sin(in.instance_rotation),
        world_pos.x * sin(in.instance_rotation) + world_pos.y * cos(in.instance_rotation)
    );
    out.position = projection_matrix * view_matrix * vec4<f32>(rotated_pos, 0.0, 1.0);
    out.uv = in.uv;
    out.color = in.instance_color;
    return out;
}

fragment fn main(in: FragmentOutput) -> vec4<f32> {
    let tex_color = textureSample(texture, sampler_linear, in.uv);
    let glow = tex_color * glow_intensity;
    let base = tex_color * tint * in.color;
    let final_color = base + glow;
    let final_alpha = max(base.a, glow.a);
    if final_alpha <= 0.01 { discard; }
    return vec4<f32>(final_color.rgb, final_alpha);
}
