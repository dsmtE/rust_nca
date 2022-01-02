use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SimulationUniforms {
    pixel_size: [f32; 2],
}

impl SimulationUniforms {
    pub fn new(simulation_size : &[u32; 2]) -> Self {
        Self {
            pixel_size: simulation_size.map(|x| 1.0 / x as f32),
        }
    }
}

pub struct SimulationData {
    uniform: SimulationUniforms,
    buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl SimulationData {

    pub fn new(device: &wgpu::Device, simulation_size : &[u32; 2]) -> Self {
        
        let uniform = SimulationUniforms::new(&simulation_size);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Simulation uniforms Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("simulation uniforms bind group layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }
        ],
        label: Some("Simulation uniforms bind group"),
        });

        Self {
            uniform,
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    fn update_pixel_size(&mut self, simulation_size : &[u32; 2]) {
        self.uniform.pixel_size = simulation_size.map(|x| 1.0 / x as f32);
    }
}
