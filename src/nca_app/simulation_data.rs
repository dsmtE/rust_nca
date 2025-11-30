use rand::Rng;
use serde::{Deserialize, Serialize};
use crevice::std140::AsStd140;
use glam::{Vec2, Mat3};

use oxyde::wgpu as wgpu;

use wgpu::util::DeviceExt;
#[repr(C)]
#[derive(Debug, Copy, Clone, AsStd140)]
pub struct SimulationUniforms {
    pixel_size: Vec2,
    kernel: Mat3,
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
            pixel_size: Vec2::from_slice(&simulation_size.map(|x| 1.0 / x as f32)),
            kernel: Mat3::from_cols_array(&[1.0, 1.0, 1.0, 1.0, 9.0, 1.0, 1.0, 1.0, 1.0]),
        }
    }

    // row-major order (with transpose)
    pub fn get_kernel_as_slice(&self) -> [f32; 9] { self.kernel.transpose().to_cols_array() }

    pub fn get_kernel_at(&self, col: usize, row: usize) -> f32 { self.kernel.col(col)[row] }
    pub fn get_kernel_at_mut(&mut self, col: usize, row: usize) -> &mut f32 { &mut self.kernel.col_mut(col)[row] }

    pub fn set_kernel_at(&mut self, col: usize, row: usize, value: f32) {
        *self.get_kernel_at_mut(col, row) = value;
    }

    pub fn set_kernel_from_slice(&mut self, new_kernel: [f32; 9]) {
        self.kernel = Mat3::from_cols_slice(&new_kernel);
    }

    pub fn set_kernel_at_with_symmetry(&mut self, col: usize, row: usize, value: f32, mode: KernelSymmetryMode) {
        self.set_kernel_at(col, row, value);
        self.apply_symmetry_at(col, row, mode);
    }


    pub fn apply_symmetry(&mut self, mode: KernelSymmetryMode) {
        const N: usize = 3;
        const HALF_IDX: usize = (N + 1) / 2 - 1;

        match mode {
            KernelSymmetryMode::Any => (),
            KernelSymmetryMode::Vertical =>
                for row in 0..N {
                    for col in 0..HALF_IDX {
                        self.set_kernel_at(N - 1 - col, row, self.get_kernel_at(col, row));
                    }
                },
            KernelSymmetryMode::Horizontal =>
                for col in 0..N {
                    for row in 0..HALF_IDX {
                        self.set_kernel_at(col, N - 1 - row, self.get_kernel_at(col, row));
                    }
                },
            KernelSymmetryMode::Full => {
                self.apply_symmetry(KernelSymmetryMode::Horizontal);
                self.apply_symmetry(KernelSymmetryMode::Vertical);
            },
        }
    }

    fn apply_symmetry_at(&mut self, col: usize, row: usize, mode: KernelSymmetryMode) {
        const N: usize = 3;
        const HALF_IDX: usize = (N + 1) / 2 - 1;

        if row == HALF_IDX && col == HALF_IDX { return; } // center 

        let value = self.get_kernel_at(col, row);
        match mode {
            KernelSymmetryMode::Any => (),
            KernelSymmetryMode::Vertical => 
            if col != HALF_IDX {
                *self.get_kernel_at_mut(N - 1 - col, row) = value
            }
            KernelSymmetryMode::Horizontal =>
            if row != HALF_IDX {
                *self.get_kernel_at_mut(col, N - 1 - row) = value
            },
            KernelSymmetryMode::Full => {
                
                *self.get_kernel_at_mut(col, N - 1 - row) = value;
                *self.get_kernel_at_mut(N - 1 - col, row) = value;
                *self.get_kernel_at_mut(N - 1 - col, N - 1 - row) = value;

                *self.get_kernel_at_mut(row, col) = value;
                *self.get_kernel_at_mut(N - 1 - row, col) = value;
                *self.get_kernel_at_mut(row, N - 1 - col) = value;
            },
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
            seed: rand::rng().random::<f32>(),
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
            contents: uniform.as_std140().as_bytes(),
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
        self.uniform.pixel_size = Vec2::from_slice(&new_simulation_size.map(|x| 1.0 / x as f32));
        self.need_update = true;
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(&self.buffer, 0, self.uniform.as_std140().as_bytes());
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
