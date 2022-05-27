struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
};

struct ViewParameters {
    center: vec2<f32>;
    zoom_level: f32;
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

[[group(1), binding(0)]]
var<uniform> view_parameters: ViewParameters;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let flipped_Center = vec2<f32>(view_parameters.center.x, 1.0-view_parameters.center.y);
    var uv = (in.uv - 0.5) * view_parameters.zoom_level + 0.5 + (flipped_Center - 0.5);
    return textureSample(simulation_texture, simulation_tex_sampler, uv);
}