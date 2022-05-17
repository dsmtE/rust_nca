struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
};

struct SimulationUniforms {
    pixel_size: vec2<f32>;
    kernel: array<f32, 9>;
    align: f32;
};

[[group(0), binding(0)]] var simulation_texture: texture_2d<f32>;
[[group(0), binding(1)]] var simulation_tex_sampler: sampler;

[[group(1), binding(0)]]
var<uniform> simulation_uniforms: SimulationUniforms;

fn getCoords(coord: vec2<f32>, offset: vec2<f32>) -> vec2<f32> {
    return (coord + simulation_uniforms.pixel_size * offset) % vec2<f32>(1.0);
}

fn activationFunction(kernelOutput: f32) -> vec4<f32> {
    var d: f32 = 0.1;
    var condition: bool = (kernelOutput > 3.0-d && kernelOutput < 3.0+d) || (kernelOutput > 11.0-d && kernelOutput < 11.0+d) || (kernelOutput > 12.0-d && kernelOutput < 12.0+d);
    var r: f32 = select(0.0, 1.0, condition);
    return vec4<f32>(r, r, r, 1.0);
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let sum: f32 =
          textureSample(simulation_texture, simulation_tex_sampler, getCoords(in.uv, vec2<f32>( 1.,-1.))).x * simulation_uniforms.kernel[0] 
        + textureSample(simulation_texture, simulation_tex_sampler, getCoords(in.uv, vec2<f32>( 0.,-1.))).x * simulation_uniforms.kernel[1]
        + textureSample(simulation_texture, simulation_tex_sampler, getCoords(in.uv, vec2<f32>(-1.,-1.))).x * simulation_uniforms.kernel[2]
        + textureSample(simulation_texture, simulation_tex_sampler, getCoords(in.uv, vec2<f32>( 1., 0.))).x * simulation_uniforms.kernel[3]
        + textureSample(simulation_texture, simulation_tex_sampler, getCoords(in.uv, vec2<f32>( 0., 0.))).x * simulation_uniforms.kernel[4]
        + textureSample(simulation_texture, simulation_tex_sampler, getCoords(in.uv, vec2<f32>(-1., 0.))).x * simulation_uniforms.kernel[5]
        + textureSample(simulation_texture, simulation_tex_sampler, getCoords(in.uv, vec2<f32>( 1., 1.))).x * simulation_uniforms.kernel[6]
        + textureSample(simulation_texture, simulation_tex_sampler, getCoords(in.uv, vec2<f32>( 0., 1.))).x * simulation_uniforms.kernel[7]
        + textureSample(simulation_texture, simulation_tex_sampler, getCoords(in.uv, vec2<f32>(-1., 1.))).x * simulation_uniforms.kernel[8];

    return activationFunction(sum);
}