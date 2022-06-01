use crevice::std140::AsStd140;
use nalgebra_glm as glm;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Debug, Copy, Clone, AsStd140)]
pub struct ViewParameters {
    pub center: glm::Vec2,
    pub zoom_level: f32,
}

pub struct ViewData {
    pub need_update: bool,
    pub uniform: ViewParameters,
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl ViewData {
    pub fn new(device: &wgpu::Device) -> Self {
        let uniform = ViewParameters {
            center: glm::vec2(0.5, 0.5),
            zoom_level: 1.0,
        };

        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("View uniforms Buffer"),
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
            label: Some("View uniforms bind group layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some("View uniforms bind group"),
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
        queue.write_buffer(&self.buffer, 0, self.uniform.as_std140().as_bytes());
        self.need_update = false;
    }
}
