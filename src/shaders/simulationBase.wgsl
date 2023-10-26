struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

struct SimulationUniforms {
    pixel_size: vec2<f32>,
    kernel_first_row: vec3<f32>,
    kernel_second_row: vec3<f32>,
    kernel_third_row: vec3<f32>,
};

@group(0) @binding(0) var simulation_texture: texture_2d<f32>;
@group(0) @binding(1) var simulation_tex_sampler: sampler;

@group(1) @binding(0)
var<uniform> simulation_uniforms: SimulationUniforms;

fn getCoords(coord: vec2<f32>, offset: vec2<f32>) -> vec2<f32> {
    return (coord + simulation_uniforms.pixel_size * offset) % vec2<f32>(1.0);
}

[functionTemplate]

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {

    let textureUv: vec2<f32> = vec2<f32>(in.uv.x, 1.0 - in.uv.y);
    let sum: vec4<f32> =
          textureSample(simulation_texture, simulation_tex_sampler, getCoords(textureUv, vec2<f32>( 1.,-1.))) * simulation_uniforms.kernel_first_row[0] 
        + textureSample(simulation_texture, simulation_tex_sampler, getCoords(textureUv, vec2<f32>( 0.,-1.))) * simulation_uniforms.kernel_first_row[1]
        + textureSample(simulation_texture, simulation_tex_sampler, getCoords(textureUv, vec2<f32>(-1.,-1.))) * simulation_uniforms.kernel_first_row[2]
        + textureSample(simulation_texture, simulation_tex_sampler, getCoords(textureUv, vec2<f32>( 1., 0.))) * simulation_uniforms.kernel_second_row[0]
        + textureSample(simulation_texture, simulation_tex_sampler, getCoords(textureUv, vec2<f32>( 0., 0.))) * simulation_uniforms.kernel_second_row[1]
        + textureSample(simulation_texture, simulation_tex_sampler, getCoords(textureUv, vec2<f32>(-1., 0.))) * simulation_uniforms.kernel_second_row[2]
        + textureSample(simulation_texture, simulation_tex_sampler, getCoords(textureUv, vec2<f32>( 1., 1.))) * simulation_uniforms.kernel_third_row[0]
        + textureSample(simulation_texture, simulation_tex_sampler, getCoords(textureUv, vec2<f32>( 0., 1.))) * simulation_uniforms.kernel_third_row[1]
        + textureSample(simulation_texture, simulation_tex_sampler, getCoords(textureUv, vec2<f32>(-1., 1.))) * simulation_uniforms.kernel_third_row[2];

    return activationFunction(sum);
}