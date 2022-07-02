use rand::Rng;
use serde::{Deserialize, Serialize};
use wgpu::util::DeviceExt;
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimulationUniforms {
    pixel_size: [f32; 2],
    kernel: [f32; 9],
    align: f32,
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq)]
pub enum KernelSymmetryMode {
    Any,
    Vertical,
    Horizontal,
    Full,
}

// TODO: learn to make macro for that
impl std::fmt::Display for KernelSymmetryMode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            KernelSymmetryMode::Any => write!(f, "Any"),
            KernelSymmetryMode::Vertical => write!(f, "Vertical"),
            KernelSymmetryMode::Horizontal => write!(f, "Horizontal"),
            KernelSymmetryMode::Full => write!(f, "Full"),
        }
    }
}

impl SimulationUniforms {
    pub fn new(simulation_size: &[u32; 2]) -> Self {
        Self {
            pixel_size: simulation_size.map(|x| 1.0 / x as f32),
            kernel: [1.0, 1.0, 1.0, 1.0, 9.0, 1.0, 1.0, 1.0, 1.0],
            align: 0.0,
        }
    }

    pub fn set_kernel_at(&mut self, index: usize, value: f32, mode: KernelSymmetryMode) {
        self.kernel[index] = value;
        self.apply_symmetry_at(index, mode);
    }

    pub fn get_kernel_at(&self, index: usize) -> f32 { self.kernel[index] }

    pub fn get_kernel(&self) -> [f32; 9] { self.kernel.clone() }

    pub fn set_kernel(&mut self, new_kernel: [f32; 9]) { self.kernel = new_kernel; }

    pub fn apply_symmetry(&mut self, mode: KernelSymmetryMode) {
        const N: usize = 3;
        const HALF_IDX: usize = (N + 1) / 2 - 1;

        match mode {
            KernelSymmetryMode::Any => (),
            KernelSymmetryMode::Vertical =>
                for i in 0..HALF_IDX {
                    for j in 0..N {
                        self.kernel[j * N + (N - 1 - i % N)] = self.kernel[j * N + i];
                    }
                },
            KernelSymmetryMode::Horizontal =>
                for i in 0..N {
                    for j in 0..HALF_IDX {
                        self.kernel[(N - 1 - j / N) * N + i] = self.kernel[j * N + i];
                    }
                },
            KernelSymmetryMode::Full => {
                self.apply_symmetry(KernelSymmetryMode::Horizontal);
                self.apply_symmetry(KernelSymmetryMode::Vertical);
            },
        }
    }

    fn apply_symmetry_at(&mut self, index: usize, mode: KernelSymmetryMode) {
        const N: usize = 3;
        const HALF_IDX: usize = (N + 1) / 2 - 1;

        if index == HALF_IDX * N + HALF_IDX {
            return;
        } // center

        match mode {
            KernelSymmetryMode::Any => (),
            KernelSymmetryMode::Vertical =>
                if index % N != HALF_IDX {
                    self.kernel[vertical_symmetry_idx(index)] = self.kernel[index];
                },
            KernelSymmetryMode::Horizontal =>
                if index / N != HALF_IDX {
                    self.kernel[horizontal_symmetry_idx(index)] = self.kernel[index];
                },
            KernelSymmetryMode::Full => {
                self.kernel[vertical_symmetry_idx(index)] = self.kernel[index];
                self.kernel[horizontal_symmetry_idx(index)] = self.kernel[index];
                self.kernel[vertical_symmetry_idx(horizontal_symmetry_idx(index))] = self.kernel[index];

                // Apply symmetry on rotated index
                let rotated_idx = rot_anticlockwise_idx(index);
                self.kernel[rotated_idx] = self.kernel[index];
                self.kernel[vertical_symmetry_idx(rotated_idx)] = self.kernel[rotated_idx];
                self.kernel[horizontal_symmetry_idx(rotated_idx)] = self.kernel[rotated_idx];
                self.kernel[vertical_symmetry_idx(horizontal_symmetry_idx(rotated_idx))] = self.kernel[rotated_idx];
            },
        }

        #[inline(always)]
        fn vertical_symmetry_idx(i: usize) -> usize { (i / N) * N + (N - 1 - i % N) }
        #[inline(always)]
        fn horizontal_symmetry_idx(i: usize) -> usize { (N - 1 - i / N) * N + i % N }
        #[inline(always)]
        fn rot_anticlockwise_idx(i: usize) -> usize { (N - 1 - i % N) * N + i / N }
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

// TODO: generic getSet fonctionnal

impl SimulationData {
    pub fn new(device: &wgpu::Device, simulation_size: &[u32; 2]) -> Self {
        let uniform = SimulationUniforms::new(&simulation_size);

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Simulation uniforms Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("simulation uniforms bind group layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
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

    pub fn set_simulation_size(&mut self, new_simulation_size: &[u32; 2]) {
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
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("Init Simulation uniforms bind group layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
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
