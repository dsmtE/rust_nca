pub struct PingPongTexture {
    label: Option<&'static str>,
    // texture_ping: wgpu::Texture,
    // texture_pong: wgpu::Texture,
    view_ping: wgpu::TextureView,
    view_pong: wgpu::TextureView,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub state: bool,
}

impl PingPongTexture {
    // TODO: pass SamplerDescriptor in parameter
    pub fn from_descriptor(
        device: &wgpu::Device,
        descriptor: &wgpu::TextureDescriptor,
        label: Option<&'static str>, // Optional debug label. This will show up in graphics debuggers for easy identification.
    ) -> Result<Self, wgpu::Error> {
        let texture_ping = device.create_texture(&descriptor);
        let texture_pong = device.create_texture(&descriptor);
        let view_ping = texture_ping.create_view(&wgpu::TextureViewDescriptor::default());
        let view_pong = texture_pong.create_view(&wgpu::TextureViewDescriptor::default());

        let group_layout_label: String = label.unwrap_or("").to_owned() + " bind group layout";
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { 0: wgpu::SamplerBindingType::Filtering },
                    count: None,
                },
            ],
            label: Some(&group_layout_label[..]),
        });

        Ok(Self {
            label,
            view_ping,
            view_pong,
            bind_group_layout,
            state: false,
        })
    }

    pub fn create_binding_group(&self, device: &wgpu::Device, sampler: &wgpu::Sampler) -> (wgpu::BindGroup, wgpu::BindGroup) {
        let bind_group_ping_label: String = self.label.unwrap_or("").to_owned() + " bind group ping";
        let bind_group_ping = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.view_ping),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
            label: Some(&bind_group_ping_label[..]),
        });

        let bind_group_pong_label: String = self.label.unwrap_or("").to_owned() + " bind group pong";
        let bind_group_pong = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.view_pong),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
            label: Some(&bind_group_pong_label[..]),
        });

        (bind_group_ping, bind_group_pong)
    }

    pub fn toogle_state(&mut self) { self.state = !self.state; }

    pub fn get_target_texture_view(&self) -> &wgpu::TextureView {
        if self.state {
            &self.view_ping
        } else {
            &self.view_pong
        }
    }

    pub fn get_rendered_texture_view(&self) -> &wgpu::TextureView {
        if !self.state {
            &self.view_ping
        } else {
            &self.view_pong
        }
    }
}
