struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
};

struct SimulationUniforms {
    pixel_size: vec2<f32>;
};

[[group(0), binding(0)]] var simulation_texture: texture_2d<f32>;
[[group(0), binding(1)]] var simulation_tex_sampler: sampler;

[[group(1), binding(0)]]
var<uniform> simulation_uniforms: SimulationUniforms;

fn getCoords(coord: vec2<f32>, offset: vec2<f32>) -> vec2<f32> {
    return (coord + simulation_uniforms.pixel_size * offset) % vec2<f32>(1.0);
}

var<private> kernel: array<f32, 9> = array<f32, 9>(
    1.0,1.0,1.0,
    1.0,9.0,1.0,
    1.0,1.0,1.0,
);

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let sum: f32 =
          textureSample(simulation_texture, simulation_tex_sampler, getCoords(in.uv, vec2<f32>( 1.,-1.))).x * kernel[0] 
        + textureSample(simulation_texture, simulation_tex_sampler, getCoords(in.uv, vec2<f32>( 0.,-1.))).x * kernel[1]
        + textureSample(simulation_texture, simulation_tex_sampler, getCoords(in.uv, vec2<f32>(-1.,-1.))).x * kernel[2]
        + textureSample(simulation_texture, simulation_tex_sampler, getCoords(in.uv, vec2<f32>( 1., 0.))).x * kernel[3]
        + textureSample(simulation_texture, simulation_tex_sampler, getCoords(in.uv, vec2<f32>( 0., 0.))).x * kernel[4]
        + textureSample(simulation_texture, simulation_tex_sampler, getCoords(in.uv, vec2<f32>(-1., 0.))).x * kernel[5]
        + textureSample(simulation_texture, simulation_tex_sampler, getCoords(in.uv, vec2<f32>( 1., 1.))).x * kernel[6]
        + textureSample(simulation_texture, simulation_tex_sampler, getCoords(in.uv, vec2<f32>( 0., 1.))).x * kernel[7]
        + textureSample(simulation_texture, simulation_tex_sampler, getCoords(in.uv, vec2<f32>(-1., 1.))).x * kernel[8];

    var d: f32 = 0.1;
    var r: f32 = 0.0;

    if ( (sum > 3.0-d && sum < 3.0+d)  || (sum > 11.0-d && sum < 11.0+d) || (sum > 12.0-d && sum < 12.0+d)) {
        r = 1.0;
    }

    // if (sum == 3.0 || sum == 11.0 || sum == 12.0) {
    //     r = 1.0;
    // }

    return vec4<f32>(r, r, r, 1.0);

    ////////

    // var val: f32 = textureSample(simulation_texture, simulation_tex_sampler, in.uv ).x;
    // val = val + 0.01;
    
    // if (val > 1.0) { val = 0.0; }
    // return vec4<f32>(val, val, val, 1.0);

    // return textureSample(simulation_texture, simulation_tex_sampler, (in.uv - simulation_uniforms.pixel_size) % vec2<f32>(1.0));
}