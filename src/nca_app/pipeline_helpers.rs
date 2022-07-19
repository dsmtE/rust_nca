use oxyde::wgpu_utils::PingPongTexture;

use super::{
    simulation_data::{InitSimulationData, SimulationData},
    view_data::ViewData,
};

pub fn get_texture_descriptor(size: &[u32; 2]) -> wgpu::TextureDescriptor {
    wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: size[0],
            height: size[1],
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        label: None,
    }
}

pub fn get_simulation_textures_and_bind_groups(
    device: &mut wgpu::Device,
    texture_descriptor: &wgpu::TextureDescriptor,
) -> Result<(PingPongTexture, wgpu::BindGroup, wgpu::BindGroup, wgpu::BindGroup, wgpu::BindGroup), wgpu::Error> {
    let simulation_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::Repeat,
        address_mode_v: wgpu::AddressMode::Repeat,
        address_mode_w: wgpu::AddressMode::Repeat,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        lod_min_clamp: 0.0,
        lod_max_clamp: 0.0,
        ..Default::default()
    });

    let display_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        lod_min_clamp: 0.0,
        lod_max_clamp: 0.0,
        ..Default::default()
    });

    let simulation_textures = PingPongTexture::from_descriptor(device, &texture_descriptor, Some("simulation"))?;

    let (bind_group_display_ping, bind_group_display_pong) = simulation_textures.create_binding_group(device, &display_sampler);
    let (bind_group_simulation_ping, bind_group_simulation_pong) = simulation_textures.create_binding_group(device, &simulation_sampler);

    Ok((
        simulation_textures,
        bind_group_display_ping,
        bind_group_display_pong,
        bind_group_simulation_ping,
        bind_group_simulation_pong,
    ))
}

pub fn build_simulation_pipeline(
    device: &mut wgpu::Device,
    surface_configuration: &wgpu::SurfaceConfiguration,
    primitive_state: &wgpu::PrimitiveState,
    multisample_state: &wgpu::MultisampleState,
    screen_shader: &wgpu::ShaderModule,
    simulation_shader: &wgpu::ShaderModule,
    simulation_textures: &PingPongTexture,
    simulation_data: &SimulationData,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Simulation Render Pipeline"),
        layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Simulation Pipeline Layout"),
            bind_group_layouts: &[&simulation_textures.bind_group_layout, &simulation_data.bind_group_layout],
            push_constant_ranges: &[],
        })),
        vertex: wgpu::VertexState {
            module: &screen_shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &simulation_shader,
            entry_point: "fs_main",
            targets: &[wgpu::ColorTargetState {
                format: surface_configuration.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            }],
        }),
        primitive: *primitive_state,
        depth_stencil: None,
        multisample: *multisample_state,
        multiview: None,
    })
}

pub fn build_screen_pipeline(
    device: &mut wgpu::Device,
    surface_configuration: &wgpu::SurfaceConfiguration,
    primitive_state: &wgpu::PrimitiveState,
    multisample_state: &wgpu::MultisampleState,
    screen_shader: &wgpu::ShaderModule,
    simulation_textures: &PingPongTexture,
    view_data: &ViewData,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Screen Render Pipeline"),
        layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Screen Pipeline Layout"),
            bind_group_layouts: &[&simulation_textures.bind_group_layout, &view_data.bind_group_layout],
            push_constant_ranges: &[],
        })),
        vertex: wgpu::VertexState {
            module: &screen_shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &screen_shader,
            entry_point: "fs_main",
            targets: &[wgpu::ColorTargetState {
                format: surface_configuration.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            }],
        }),
        primitive: *primitive_state,
        depth_stencil: None,
        multisample: *multisample_state,
        multiview: None,
    })
}

pub fn build_init_simulation_pipeline(
    device: &mut wgpu::Device,
    surface_configuration: &wgpu::SurfaceConfiguration,
    primitive_state: &wgpu::PrimitiveState,
    multisample_state: &wgpu::MultisampleState,
    screen_shader: &wgpu::ShaderModule,
    init_simulation_shader: &wgpu::ShaderModule,
    init_simulation_data: &InitSimulationData,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Init Simulation Render Pipeline"),
        layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Init Simulation Pipeline Layout"),
            bind_group_layouts: &[&init_simulation_data.bind_group_layout],
            push_constant_ranges: &[],
        })),
        vertex: wgpu::VertexState {
            module: &screen_shader,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &init_simulation_shader,
            entry_point: "fs_main",
            targets: &[wgpu::ColorTargetState {
                format: surface_configuration.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            }],
        }),
        primitive: *primitive_state,
        depth_stencil: None,
        multisample: *multisample_state,
        multiview: None,
    })
}
