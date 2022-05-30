use wgpu::util::DeviceExt;
use rand::Rng;
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimulationUniforms {
    pixel_size: [f32; 2],
    pub kernel: [f32; 9],
    align: f32,
}

impl SimulationUniforms {
    pub fn new(simulation_size : &[u32; 2]) -> Self {
        Self {
            pixel_size: simulation_size.map(|x| 1.0 / x as f32),
            kernel: [1.0, 1.0, 1.0, 1.0, 9.0, 1.0, 1.0, 1.0, 1.0],
            align: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InitSimulationUniforms {
    pub seed: f32,
    pub initialisation_mode: u32,
}

impl InitSimulationUniforms {
    pub fn new() -> Self {
        Self {
            seed: rand::thread_rng().gen::<f32>(),
            initialisation_mode: 0,
        }
    }
}

pub struct SimulationData {
    pub need_update: bool,
    pub uniform: SimulationUniforms,
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

pub struct InitSimulationData {
    pub need_update: bool,
    pub uniform: InitSimulationUniforms,
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

//TODO: generic getSet fonctionnal

impl SimulationData {
    pub fn new(device: &wgpu::Device, simulation_size : &[u32; 2]) -> Self {
        
        let uniform = SimulationUniforms::new(&simulation_size);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Simulation uniforms Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
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
            need_update: false,
            uniform,
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn set_simulation_size(&mut self, new_simulation_size : &[u32; 2]) {
        self.uniform.pixel_size = new_simulation_size.map(|x| 1.0 / x as f32);
        self.need_update = true;
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
        self.need_update = false;
    }
}

impl InitSimulationData {
    pub fn new(device: &wgpu::Device) -> Self {
        
        let uniform = InitSimulationUniforms::new();

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Init Simulation uniforms Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
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
            label: Some("Init Simulation uniforms bind group layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }
        ],
        label: Some("Init Simulation uniforms bind group"),
        });

        Self {
            need_update: false,
            uniform,
            buffer,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]));
        self.need_update = false;
    }
}

