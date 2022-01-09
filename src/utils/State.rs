use std::iter;

use winit::{
    event::*,
    event_loop::EventLoop,
    window::Window,
};

use super::ping_pong_texture::PingPongTexture;

mod simulation_data;
use simulation_data::SimulationData;

mod gui_render_wgpu;
use gui_render_wgpu::{Gui, GuiRenderWgpu, ScreenDescriptor};

#[derive(Default)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}


pub struct State {
    window_id: winit::window::WindowId,
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

    bind_group_display_ping : wgpu::BindGroup,
    bind_group_display_pong: wgpu::BindGroup,
    bind_group_simulation_ping: wgpu::BindGroup,
    bind_group_simulation_pong: wgpu::BindGroup,

    gui: Gui,
    gui_render: GuiRenderWgpu,
    demo_app: egui_demo_lib::WrapApp,
    ui_central_viewport: Viewport,
}

impl State {
    pub async fn new(window: &Window, event_loop: &EventLoop<()>, simulation_size: [u32; 2]) -> State {
        let size = window.inner_size();
        let scale_factor =  window.scale_factor();
        
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
        
        let surface_format = surface.get_preferred_format(&adapter).unwrap();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            // FIFO, will cap the display rate at the displays framerate. This is essentially VSync.
            // https://docs.rs/wgpu/0.10.1/wgpu/enum.PresentMode.html
            present_mode: wgpu::PresentMode::Mailbox,
        };
        surface.configure(&device, &config);
        
        let mut gui = Gui::new(ScreenDescriptor {
            physical_width: size.width,
            physical_height: size.height,
            scale_factor: scale_factor as f32,
        });

        let gui_render = GuiRenderWgpu::new(&device, config.format, 1);

        // Display the demo application that ships with egui.
        let demo_app = egui_demo_lib::WrapApp::default();
        
        let ui_central_viewport = Viewport {
            x: 0.0,
            y: 0.0,
            width: size.width as f32,
            height: size.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

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
        
        let simulation_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
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

        let (bind_group_display_ping, bind_group_display_pong) = simulation_textures.create_binding_group(&device, &display_sampler);
        let (bind_group_simulation_ping, bind_group_simulation_pong) = simulation_textures.create_binding_group(&device, &simulation_sampler);

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
            window_id: window.id(),
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
            
            bind_group_display_ping,
            bind_group_display_pong,
            bind_group_simulation_ping,
            bind_group_simulation_pong,
            gui,
            gui_render,
            demo_app,
            ui_central_viewport,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
        // See: https://github.com/rust-windowing/winit/issues/208
        // This solves an issue where the app would panic when minimizing on Windows.
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, event: &Event<()>) {
        self.gui.handle_event(event);
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == &self.window_id => {
                match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        self.clear_color = wgpu::Color {
                            r: position.x as f64 / self.size.width as f64,
                            g: position.y as f64 / self.size.height as f64,
                            b: 1.0,
                            a: 1.0,
                        };
                    }
                    WindowEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(_, y), .. } => {
                        println!("ScrollDelta {}", y);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    pub fn update(&mut self) {}

    pub fn render_ui_meshs(&mut self, window: &winit::window::Window) -> Vec<egui::ClippedMesh> {
        // Begin to draw the UI frame.
        let _frame_data = self.gui.start_frame(window.scale_factor() as _);
        let ctx = &self.gui.context();
        
        egui::TopBottomPanel::top("top_panel")
            .resizable(true)
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Open").clicked() {
                            // …
                        }
                        if ui.button("Quit").clicked() {
                            // TODO exit
                        }
                    });
                });

            });

        egui::SidePanel::left("left_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Left Panel");
                ui.allocate_space(ui.available_size());
            });

        egui::SidePanel::right("right_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Right Panel");
                ui.allocate_space(ui.available_size());
            });

        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .show(ctx, |ui| {
                ui.heading("Bottom Panel");
                
                ui.allocate_space(ui.available_size());
            });


        // Create manually virtual center panel to get rect used as viewport afterward
        let mut center_ui_panel_viewport = egui::Ui::new(
            ctx.clone(),
            egui::LayerId::background(),
            egui::Id::new("central_panel"),
            ctx.available_rect(),
            ctx.input().screen_rect()
        ).max_rect();

        // update ui_central_viewport
        let center_size = center_ui_panel_viewport.max - center_ui_panel_viewport.min;
        self.ui_central_viewport.x = center_ui_panel_viewport.min.x;
        self.ui_central_viewport.y = center_ui_panel_viewport.min.y;
        self.ui_central_viewport.width = center_size.x;
        self.ui_central_viewport.height = center_size.y;
        
        return self.gui.end_frame(window);
    }

    pub fn render(&mut self, window: &winit::window::Window) -> Result<(), wgpu::SurfaceError> {
        let output: wgpu::SurfaceTexture = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let window_scale_factor = window.scale_factor() as f32;

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let ui_meshes = self.render_ui_meshs(window);

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
            let bind_group: &wgpu::BindGroup = if self.simulation_textures.state { &self.bind_group_simulation_pong } else { &self.bind_group_simulation_ping };
            simulation_render_pass.set_bind_group(0, bind_group, &[]);
            simulation_render_pass.set_bind_group(1, &self.simulation_uniforms.bind_group, &[]);
            simulation_render_pass.draw(0..3, 0..1);
        }
        
        // draw UI
        encoder.insert_debug_marker("Render GUI");

        let screen_descriptor = 

         self.gui_render.render(
             self.gui.context(),
             &self.device,
             &self.queue,
             &ScreenDescriptor {
                 physical_width: self.size.width,
                 physical_height: self.size.height,
                scale_factor: window_scale_factor,
             },
             &mut encoder,
             &view,
             &ui_meshes,
         ).expect("Failed to execute gui render pass!");

        // render simulation on screen
        {
            let mut screen_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Screen Render Pass"),
                color_attachments: &[
                    // This is what [[location(0)]] in the fragment shader targets
                    wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            
            // update viewport accordingly to the Ui to display the simulation
            // it must be multiplied by window scale factor as render pass use physical pixels screen size
            
            screen_render_pass.set_viewport(
                self.ui_central_viewport.x * window_scale_factor,
                self.ui_central_viewport.y * window_scale_factor,
                self.ui_central_viewport.width * window_scale_factor,
                self.ui_central_viewport.height * window_scale_factor,
                self.ui_central_viewport.min_depth,
                self.ui_central_viewport.max_depth,
            );

            screen_render_pass.set_pipeline(&self.screen_render_pipeline);
            // let bind_group: &wgpu::BindGroup = if self.simulation_textures.state { &self.bind_group_display_ping } else { &self.bind_group_display_pong };
            // TODO: why it's blinking on switch bindgroup ?
            let bind_group: &wgpu::BindGroup = &self.bind_group_display_ping;
            screen_render_pass.set_bind_group(0, bind_group, &[]);
            screen_render_pass.draw(0..3, 0..1);
        }
       
        // */

        // submit will accept anything that implements IntoIter
        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        self.simulation_textures.toogle_state();
        Ok(())
    }
}