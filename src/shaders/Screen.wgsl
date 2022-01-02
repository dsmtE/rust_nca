struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
};

var<private> positions: array<vec2<f32>, 3> = array<vec2<f32>, 3>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(3.0, -1.0),
    vec2<f32>(-1.0, 3.0)
);

[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(positions[in_vertex_index], 0.0, 1.0);
    out.uv = (out.clip_position.xy+vec2<f32>(1.0))/vec2<f32>(2.0);
    return out;
}

[[group(0), binding(0)]] var simulation_texture: texture_2d<f32>;
[[group(0), binding(1)]] var simulation_tex_sampler: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    // let new_uv = in.uv - vec2<f32>(0.5, 0.5);
    // let new_uv = in.uv;
    return textureSample(simulation_texture, simulation_tex_sampler, in.uv);
    // return vec4<f32>(in.uv, 0.0, 1.0);
}