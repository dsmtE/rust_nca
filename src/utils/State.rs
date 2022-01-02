use std::iter;

use winit::{
    event::*,
    window::Window,
};

use super::ping_pong_texture::PingPongTexture;

mod simulation_data;
use simulation_data::SimulationData;

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    clear_color: wgpu::Color,
    init_simulation_render_pipeline: wgpu::RenderPipeline,
    simulation_render_pipeline: wgpu::RenderPipeline,
    screen_render_pipeline: wgpu::RenderPipeline,
    simulation_textures: PingPongTexture,
    simulation_uniforms: SimulationData,
    init: bool,
}

impl State {
    pub async fn new(window: &Window, simulation_size: [u32; 2]) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::default(),
                limits: wgpu::Limits::default(),
            },
            None,
        )
        .await.unwrap();
        
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_preferred_format(&adapter).unwrap(),
            width: size.width,
            height: size.height,
            // FIFO, will cap the display rate at the displays framerate. This is essentially VSync.
            // https://docs.rs/wgpu/0.10.1/wgpu/enum.PresentMode.html
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);
        
        // Texture
        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: simulation_size[0],
                height: simulation_size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            label: None,
        };

        let simulation_textures = PingPongTexture::from_descriptor(&device, &texture_desc, Some("simulation")).unwrap();
         
        let simulation_uniforms = SimulationData::new(&device, &simulation_size);
        
        // Shaders
        let screen_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Screne Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/Screen.wgsl").into()),
        });

        let simulation_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Simulation Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/simulation.wgsl").into()),
        });

        let init_simulation_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Init Simulation Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/init_simulation.wgsl").into()),
        });

        // Pipeline
        let primitive_state = wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            ..Default::default()
        };

        let multisample_state = wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        };

        let screen_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Screen Render Pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Screen Pipeline Layout"),
                bind_group_layouts: &[&simulation_textures.bind_group_layout],
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
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: primitive_state,
            depth_stencil: None,
            multisample: multisample_state,
            multiview: None,
        });
            
        let init_simulation_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Init Simulation Render Pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Init Simulation Pipeline Layout"),
                bind_group_layouts: &[],
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
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: primitive_state,
            depth_stencil: None,
            multisample: multisample_state,
            multiview: None,
        });

        let simulation_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Simulation Render Pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Simulation Pipeline Layout"),
                bind_group_layouts: &[&simulation_textures.bind_group_layout, &simulation_uniforms.bind_group_layout],
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
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: primitive_state,
            depth_stencil: None,
            multisample: multisample_state,
            multiview: None,
        });
        
        Self {
            surface,
            device,
            queue,
            config,
            init_simulation_render_pipeline,
            simulation_render_pipeline,
            screen_render_pipeline,
            size,
            clear_color: wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 },
            simulation_textures,
            simulation_uniforms,
            init: false,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.clear_color = wgpu::Color {
                    r: position.x as f64 / self.size.width as f64,
                    g: position.y as f64 / self.size.height as f64,
                    b: 1.0,
                    a: 1.0,
                };
                true
            }
            WindowEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(_, y), .. } => {
                println!("ScrollDelta {}", y);
                true
            }
            _ => false,
        }
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output: wgpu::SurfaceTexture = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        // init if needed
        if self.init == false {
            self.init = true;
            println!("init {}", self.init);
            {
                let mut init_simulation_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Init Simulation Render Pass"),
                    color_attachments: &[
                        wgpu::RenderPassColorAttachment {
                        view: &self.simulation_textures.get_rendered_texture_view(),
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(self.clear_color),
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });
                
                init_simulation_render_pass.set_pipeline(&self.init_simulation_render_pipeline);
                init_simulation_render_pass.draw(0..3, 0..1);
            }
        }
            
        // simulation
        {
            let mut simulation_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Simulation Render Pass"),
                color_attachments: &[
                    wgpu::RenderPassColorAttachment {
                    view: self.simulation_textures.get_target_texture_view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            
            simulation_render_pass.set_pipeline(&self.simulation_render_pipeline);
            simulation_render_pass.set_bind_group(0, self.simulation_textures.get_rendered_bind_group(), &[]);
            simulation_render_pass.set_bind_group(1, &self.simulation_uniforms.bind_group, &[]);
            simulation_render_pass.draw(0..3, 0..1);
        }
    
        // render on screen
        {
            let mut screen_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Screen Render Pass"),
                color_attachments: &[
                    // This is what [[location(0)]] in the fragment shader targets
                    wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            screen_render_pass.set_pipeline(&self.screen_render_pipeline);
            screen_render_pass.set_bind_group(0, self.simulation_textures.get_target_bind_group(), &[]); // NEW!
            screen_render_pass.draw(0..3, 0..1);
        }
        
        // submit will accept anything that implements IntoIter
        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        self.simulation_textures.toogle_state();
        Ok(())
    }
}