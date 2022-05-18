use anyhow::Result;

use winit::{
    event::{Event, WindowEvent, MouseScrollDelta},
};

use std::time::{Instant, Duration};

use skeleton_app::{App, AppState};

use epi;

use crate::{
    utils::ping_pong_texture::PingPongTexture,
    simulation_data::{SimulationData, InitSimulationData},
    egui_widgets::{UiWidget, CodeEditor},
};

#[derive(Default)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub min_depth: f32,
    pub max_depth: f32,
}

pub enum ShaderState {
    Compiled,
    Dirty,
    CompilationFail(String),
}

#[derive(Debug, PartialEq)]
pub enum DisplayFramesMode {
    All,
    Evens,
    Odd
}

pub struct NcaApp {
    clear_color: wgpu::Color,

    primitive_state: wgpu::PrimitiveState,
    multisample_state: wgpu::MultisampleState,

    screen_shader: wgpu::ShaderModule,

    init_simulation_render_pipeline: wgpu::RenderPipeline,
    simulation_render_pipeline: wgpu::RenderPipeline,
    screen_render_pipeline: wgpu::RenderPipeline,
    simulation_textures: PingPongTexture,
    init_simulation_data: InitSimulationData,
    simulation_data: SimulationData,
    init: bool,

    bind_group_display_ping : wgpu::BindGroup,
    bind_group_display_pong: wgpu::BindGroup,
    bind_group_simulation_ping: wgpu::BindGroup,
    bind_group_simulation_pong: wgpu::BindGroup,

    ui_central_viewport: Viewport,

    target_delta: Duration,
    last_simulation_end: Instant,

    code: String,
    shader_state: ShaderState,

    display_frames_mode: DisplayFramesMode,
}

impl NcaApp {

    pub fn set_target_rate<R: Into<f64>>(&mut self, target_rate: R) {
        self.target_delta = Duration::from_secs_f64(1.0 / target_rate.into());
    }

    pub fn target_rate(&self) -> f64 {
        1.0 / self.target_delta.as_secs_f64()
    }

    pub fn generate_simulation_pipeline(&mut self, device: &mut wgpu::Device, surface_configuration: &wgpu::SurfaceConfiguration) -> Result<wgpu::RenderPipeline, wgpu::Error> {
        let shader_code: String = include_str!("shaders/simulationBase.wgsl").replace("[functionTemplate]", &self.code);
            
        let (tx, rx) = std::sync::mpsc::channel::<wgpu::Error>();
        device.on_uncaptured_error(move |e: wgpu::Error| {
            tx.send(e).expect("sending error failed");
        });

        let simulation_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Simulation Shader"),
            source: wgpu::ShaderSource::Wgsl(
                shader_code.into(),
            ),
        });

        let simulation_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Simulation Render Pipeline"),
            layout: Some(&device.create_pipeline_layout(
                &wgpu::PipelineLayoutDescriptor {
                    label: Some("Simulation Pipeline Layout"),
                    bind_group_layouts: &[
                        &self.simulation_textures.bind_group_layout,
                        &self.simulation_data.bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                },
            )),
            vertex: wgpu::VertexState {
                module: &self.screen_shader,
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
            primitive: self.primitive_state,
            depth_stencil: None,
            multisample: self.multisample_state,
            multiview: None,
        });
        
        device.on_uncaptured_error(|e| panic!("{}", e));
        

        if let Ok(err) = rx.try_recv() {
            return Err(err);
        }

        Ok(simulation_render_pipeline)
    }
}

impl App for NcaApp {

    fn create(_app_state: &mut AppState) -> Self {
        
        let size = _app_state.window.inner_size();

        let ui_central_viewport = Viewport {
            x: 0.0,
            y: 0.0,
            width: size.width as f32,
            height: size.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let simulation_size: [u32; 2] = [200, 200];
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

        let init_simulation_uniforms = InitSimulationData::new(&_app_state.device);
        
        let simulation_textures = PingPongTexture::from_descriptor(&_app_state.device, &texture_desc, Some("simulation")).unwrap();
        
        let simulation_uniforms = SimulationData::new(&_app_state.device, &simulation_size);

        let simulation_sampler = _app_state.device.create_sampler(&wgpu::SamplerDescriptor {
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

        let display_sampler = _app_state.device.create_sampler(&wgpu::SamplerDescriptor {
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

        let (bind_group_display_ping, bind_group_display_pong) = simulation_textures.create_binding_group(&_app_state.device, &display_sampler);
        let (bind_group_simulation_ping, bind_group_simulation_pong) = simulation_textures.create_binding_group(&_app_state.device, &simulation_sampler);

        // Shaders
        let screen_shader = _app_state.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Screne Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/Screen.wgsl").into()),
        });

        let simulation_shader = _app_state.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Simulation Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/simulation.wgsl").into()),
        });

        let init_simulation_shader = _app_state.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Init Simulation Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/init_simulation.wgsl").into()),
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

        let screen_render_pipeline = _app_state.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Screen Render Pipeline"),
            layout: Some(&_app_state.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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
                    format: _app_state.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: primitive_state,
            depth_stencil: None,
            multisample: multisample_state,
            multiview: None,
        });
            
        let init_simulation_render_pipeline = _app_state.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Init Simulation Render Pipeline"),
            layout: Some(&_app_state.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Init Simulation Pipeline Layout"),
                bind_group_layouts: &[&init_simulation_uniforms.bind_group_layout],
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
                    format: _app_state.config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }],
            }),
            primitive: primitive_state,
            depth_stencil: None,
            multisample: multisample_state,
            multiview: None,
        });

        let simulation_render_pipeline = _app_state.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Simulation Render Pipeline"),
            layout: Some(&_app_state.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
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
                    format: _app_state.config.format,
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
            clear_color: wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 },

            primitive_state,
            multisample_state,

            screen_shader,

            init_simulation_render_pipeline,
            simulation_render_pipeline,
            screen_render_pipeline,
            simulation_textures,
            init_simulation_data: init_simulation_uniforms,
            simulation_data: simulation_uniforms,
            init: false,
        
            bind_group_display_ping,
            bind_group_display_pong,
            bind_group_simulation_ping,
            bind_group_simulation_pong,
        
            ui_central_viewport,

            target_delta: Duration::from_secs_f64(1.0 / 30.0),
            last_simulation_end: Instant::now(),

            code: "fn activationFunction(kernelOutput: f32) -> vec4<f32> {
                var condition: bool = kernelOutput == 3.0 || kernelOutput == 11.0 || kernelOutput == 12.0;
                var r: f32 = select(0.0, 1.0, condition);
                return vec4<f32>(r, r, r, 1.0);
            }".to_owned(),
            shader_state : ShaderState::Compiled,
            display_frames_mode: DisplayFramesMode::All,
        }
    }
    
    fn handle_events(&mut self, _app_state: &mut AppState, _event: &Event<()>) -> Result<()> {
        match _event {
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::CursorMoved { position, .. } => {
                    let size = _app_state.window.inner_size();
                    self.clear_color = wgpu::Color {
                        r: position.x as f64 / size.width as f64,
                        g: position.y as f64 / size.height as f64,
                        b: 1.0,
                        a: 1.0,
                    };
                }
                WindowEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(_, y), .. } => {
                    println!("ScrollDelta {}", y);
                }
                _ => {}
            }
            _ => {}
        };

        Ok(())
    }

    fn render_gui(&mut self, _ctx: & epi::egui::Context) -> Result<()> {
        egui::TopBottomPanel::top("top_panel")
            .resizable(true)
            .show(&_ctx, |ui| {
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
            .show(&_ctx, |ui| {
                ui.heading("Left Panel");
                
                egui::CollapsingHeader::new("Starting settings")
                .default_open(true)
                .show(ui, |ui| {
                    ui.add(egui::DragValue::from_get_set(|optional_value: Option<f64>| {
                        if let Some(v) = optional_value {
                            self.init_simulation_data.uniform.seed = v as f32;
                            self.init_simulation_data.need_update = true;
                        }
                        self.init_simulation_data.uniform.seed as f64
                    }).speed(0.1).prefix("seed: "));

                    if ui.button("randoms float").clicked() {
                        self.init = false;
                        self.init_simulation_data.uniform.initialisation_mode = 1;
                        self.init_simulation_data.need_update = true;
                    }

                    if ui.button("randoms ints").clicked() {
                        self.init = false;
                        self.init_simulation_data.uniform.initialisation_mode = 0;
                        self.init_simulation_data.need_update = true;
                    }
                });
                
                egui::CollapsingHeader::new("Kernel")
                .default_open(true)
                .show(ui, |ui| {
                    for j in 0..3 {
                        ui.horizontal(|ui| {
                            for i in 0..3 {
                                ui.add(egui::DragValue::from_get_set(|optional_value: Option<f64>| {
                                    if let Some(v) = optional_value {
                                        self.simulation_data.uniform.kernel[j*3+i] = v as f32;
                                        self.simulation_data.need_update = true;
                                    }
                                    self.simulation_data.uniform.kernel[j*3+i] as f64
                                }).speed(0.1));
                            }
                        });
                    }
                });
                
                egui::CollapsingHeader::new("Simulation")
                .default_open(true)
                .show(ui, |ui| {
                    let mut code_editor = CodeEditor::new(&mut self.code, "rs", Some(15));
                    code_editor.show(ui);

                    let code_to_paste: Option<String> = _ctx.input().events.iter().find_map(|e| match e {
                        egui::Event::Paste(paste_content)=> Some((*paste_content).to_owned()),
                        _ => None,
                    });
            
                    if let Some(new_code) = code_to_paste {
                        self.code = new_code;
                    }
            
                    if let ShaderState::CompilationFail(error) = &self.shader_state {
                        ui.label(format!("Shader compile error:\n {}", error));
                    }

                    if ui.button("Recompile").clicked() {
                        self.shader_state = ShaderState::Dirty;
                    }
                });
                
                egui::CollapsingHeader::new("Display Options")
                .default_open(true)
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Display frame mode: ");
                        egui::ComboBox::from_id_source("display_frames_mode")
                            .selected_text(format!("{:?}", self.display_frames_mode))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.display_frames_mode, DisplayFramesMode::All, "ALl");
                                ui.selectable_value(&mut self.display_frames_mode, DisplayFramesMode::Evens, "Evens");
                                ui.selectable_value(&mut self.display_frames_mode, DisplayFramesMode::Odd, "Odd");
                            }
                        );
                    });
                });

                ui.allocate_space(ui.available_size());
            });
        
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .show(&_ctx, |ui| {
                ui.heading("Bottom Panel");
                
                ui.allocate_space(ui.available_size());
            });

        let center_rect = _ctx.available_rect();

        // update ui_central_viewport
        let center_size = center_rect.max - center_rect.min;
        self.ui_central_viewport.x = center_rect.min.x;
        self.ui_central_viewport.y = center_rect.min.y;
        self.ui_central_viewport.width = center_size.x;
        self.ui_central_viewport.height = center_size.y;

        Ok(())
    }

    fn update(&mut self, _app_state: &mut AppState) -> Result<()> {
        
        if let ShaderState::Dirty = self.shader_state {
            match self.generate_simulation_pipeline(&mut _app_state.device, &_app_state.config) {
                Err(err) => match err {
                    wgpu::Error::OutOfMemory {..} => anyhow::bail!("Shader compilation gpu::Error::OutOfMemory"),
                    wgpu::Error::Validation { description, .. } => self.shader_state = ShaderState::CompilationFail(description),
                }
                Ok(simulation_render_pipeline) => {
                    self.simulation_render_pipeline = simulation_render_pipeline;
                    self.shader_state = ShaderState::Compiled;
                }
            }
        }

        Ok(())
    }

    fn render(&mut self, _app_state: &mut AppState, _encoder: &mut wgpu::CommandEncoder, _output_view: &wgpu::TextureView) -> Result<(), wgpu::SurfaceError> {
        
        if self.last_simulation_end.elapsed() > self.target_delta {
            
            // init if needed
            if self.init == false {
                self.init = true;

                if self.init_simulation_data.need_update {
                    self.init_simulation_data.update(&_app_state.queue);
                }

                let mut init_simulation_render_pass = _encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                init_simulation_render_pass.set_bind_group(0, &self.init_simulation_data.bind_group, &[]);
                init_simulation_render_pass.draw(0..3, 0..1);
            }

            // simulation
            {
                if self.simulation_data.need_update {
                    self.simulation_data.update(&_app_state.queue);
                }

                let mut simulation_render_pass = _encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                simulation_render_pass.set_bind_group(1, &self.simulation_data.bind_group, &[]);
                simulation_render_pass.draw(0..3, 0..1);
            }

            self.last_simulation_end = Instant::now();
            self.simulation_textures.toogle_state();
        };

        // render simulation on screen
        {
            let mut screen_render_pass = _encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Screen Render Pass"),
                color_attachments: &[
                    // This is what [[location(0)]] in the fragment shader targets
                    wgpu::RenderPassColorAttachment {
                    view: &_output_view,
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
            
            let window_scale_factor =  _app_state.window.scale_factor() as f32;
            screen_render_pass.set_viewport(
                self.ui_central_viewport.x * window_scale_factor,
                self.ui_central_viewport.y * window_scale_factor,
                self.ui_central_viewport.width * window_scale_factor,
                self.ui_central_viewport.height * window_scale_factor,
                self.ui_central_viewport.min_depth,
                self.ui_central_viewport.max_depth,
            );

            screen_render_pass.set_pipeline(&self.screen_render_pipeline);

            let bind_group: &wgpu::BindGroup = match self.display_frames_mode {
                DisplayFramesMode::All => if self.simulation_textures.state { &self.bind_group_display_pong } else { &self.bind_group_display_ping }
                DisplayFramesMode::Evens => &self.bind_group_display_pong,
                DisplayFramesMode::Odd => &self.bind_group_display_ping,
            };

            // TODO: why it's blinking on switch bindgroup ?
            // let bind_group: &wgpu::BindGroup = &self.bind_group_display_ping;
            screen_render_pass.set_bind_group(0, bind_group, &[]);
            screen_render_pass.draw(0..3, 0..1);
        }
        
        Ok(())
    }
}