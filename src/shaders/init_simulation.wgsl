struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct InitSimulationUniforms {
    seed: f32,
    initialisation_mode: u32,
};

@group(0) @binding(0)
var<uniform> init_simulation_uniforms: InitSimulationUniforms;

fn rand(v: vec2<f32>) -> f32 {
    return fract(sin(dot(v, vec2<f32>(12.9898 - init_simulation_uniforms.seed, 78.233 +  init_simulation_uniforms.seed))) * 43758.5453);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var r: f32 = rand(in.uv);

    if(init_simulation_uniforms.initialisation_mode == 0u) {
        r = round(r);
    }

    return vec4<f32>(r, r, r, 1.0);
}