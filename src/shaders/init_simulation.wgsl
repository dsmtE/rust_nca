struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
};

struct InitSimulationUniforms {
    seed: f32;
};

[[group(0), binding(0)]]
var<uniform> init_simulation_uniforms: InitSimulationUniforms;

fn rand(v: vec2<f32>) -> f32 {
    return fract(sin(dot(v, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let r = rand(in.uv* init_simulation_uniforms.seed);
    return vec4<f32>(r, r, r, 1.0);
}