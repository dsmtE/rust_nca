use anyhow::Result;
pub struct PingPongTexture {
    texture_ping: wgpu::Texture,
    texture_pong: wgpu::Texture,
    view_ping: wgpu::TextureView,
    view_pong: wgpu::TextureView,
    pub bind_group_layout: wgpu::BindGroupLayout,
    bind_group_ping: wgpu::BindGroup,
    bind_group_pong: wgpu::BindGroup,
    sampler: wgpu::Sampler,
    state: bool,
}

impl PingPongTexture {

    pub fn from_descriptor(
        device: &wgpu::Device,
        descriptor: &wgpu::TextureDescriptor,
        name: Option<&str>, // Optional debug label. This will show up in graphics debuggers for easy identification.
    ) -> Result<Self> {

        let texture_ping = device.create_texture(&descriptor);
        let texture_pong = device.create_texture(&descriptor);
        let view_ping = texture_ping.create_view(&wgpu::TextureViewDescriptor::default());
        let view_pong = texture_pong.create_view(&wgpu::TextureViewDescriptor::default());
        
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let group_layout_label: String = name.unwrap_or("").to_owned() + " bind group layout";
        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
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
                        ty: wgpu::BindingType::Sampler {
                            0: wgpu::SamplerBindingType::NonFiltering,
                        },
                        count: None,
                    },
                ],
                label: Some(&group_layout_label[..]),
            }
        );

        let bind_group_ping_label: String = name.unwrap_or("").to_owned() + " bind group ping";
        let bind_group_ping = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view_ping),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    }
                ],
                label: Some(&bind_group_ping_label[..]),
            }
        );

        let bind_group_pong_label: String = name.unwrap_or("").to_owned() + " bind group pong";
        let bind_group_pong = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view_pong),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    }
                ],
                label: Some(&bind_group_pong_label[..]),
            }
        );

        Ok(Self { texture_ping, texture_pong, view_ping, view_pong, bind_group_layout, bind_group_ping, bind_group_pong, sampler, state: false })
    }

    pub fn toogle_state(&mut self) {
        self.state = !self.state;
    }

    pub fn get_target_texture_view(&self) -> &wgpu::TextureView {
        if self.state { &self.view_ping } else { &self.view_pong }
    }

    pub fn get_rendered_texture_view(&self) -> &wgpu::TextureView {
        if !self.state { &self.view_ping } else { &self.view_pong }
    }

    // pub fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout { &self.bind_group_layout }

    pub fn get_target_bind_group(&self) -> &wgpu::BindGroup {
        if self.state { &self.bind_group_ping } else { &self.bind_group_pong }
    }

    pub fn get_rendered_bind_group(&self) -> &wgpu::BindGroup {
        if !self.state { &self.bind_group_ping } else { &self.bind_group_pong }
    }
}