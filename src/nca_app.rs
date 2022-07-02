mod pipeline_helpers;
mod preset;
mod simulation_data;
mod view_data;

use anyhow::Result;
use rand::Rng;

use winit::event::{Event, MouseScrollDelta, WindowEvent};

use std::{
    path::Path,
    time::{Duration, Instant},
};

use skeleton_app::{App, AppState};

use epi;

use serde::{Deserialize, Serialize};

use nalgebra_glm as glm;

use pipeline_helpers::{
    build_init_simulation_pipeline,
    build_screen_pipeline,
    build_simulation_pipeline,
    get_simulation_textures_and_bind_groups,
    get_texture_descriptor,
};
use preset::{Preset, PRESETS};

use simulation_data::{InitSimulationData, SimulationData, KernelSymmetryMode};
use view_data::ViewData;

use crate::{
    egui_widgets::{CodeEditor, UiWidget, IQ_GRADIENT_PRESETS, IqGradient, DisplayableVec2},
    utils::ping_pong_texture::PingPongTexture,
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

pub enum SimulationSizeState {
    Compiled([u32; 2]),
    Dirty([u32; 2]),
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
pub enum DisplayFramesMode {
    All,
    Evens,
    Odd,
}

pub struct NcaApp {
    clear_color: wgpu::Color,
    simulation_size_state: SimulationSizeState,
    primitive_state: wgpu::PrimitiveState,
    multisample_state: wgpu::MultisampleState,

    simulation_shader: wgpu::ShaderModule,
    screen_shader: wgpu::ShaderModule,

    init_simulation_render_pipeline: wgpu::RenderPipeline,
    simulation_render_pipeline: wgpu::RenderPipeline,
    screen_render_pipeline: wgpu::RenderPipeline,
    simulation_textures: PingPongTexture,
    init_simulation_data: InitSimulationData,
    simulation_data: SimulationData,
    kernel_symmetry_mode: KernelSymmetryMode,
    init: bool,
    reset_on_randomize: bool,
    kernel_rand_range: DisplayableVec2,

    bind_group_display_ping: wgpu::BindGroup,
    bind_group_display_pong: wgpu::BindGroup,
    bind_group_simulation_ping: wgpu::BindGroup,
    bind_group_simulation_pong: wgpu::BindGroup,

    ui_central_viewport: Viewport,

    target_delta: Duration,
    last_simulation_end: Instant,

    activation_code: String,
    shader_state: ShaderState,

    display_frames_mode: DisplayFramesMode,

    view_data: ViewData,
}

fn generate_simulation_shader(activation_code: &str) -> String {
    include_str!("shaders/simulationBase.wgsl").replace("[functionTemplate]", activation_code)
}

impl NcaApp {
    #[inline(always)]
    pub fn load_preset_from_file<P: AsRef<Path>>(&mut self, filepath: &P) -> Result<()> { self.load_preset(preset::load_preset(filepath)?) }

    pub fn load_preset(&mut self, preset: Preset) -> Result<()> {
        self.simulation_data.uniform.set_kernel(preset.kernel);
        self.simulation_data.need_update = true;

        self.activation_code = preset.activation_code;
        self.shader_state = ShaderState::Dirty;

        self.display_frames_mode = preset.display_frames_mode;

        self.view_data.uniform.gradient = preset.gradient;
        self.view_data.need_update = true;

        Ok(())
    }

    pub fn save_preset<P: AsRef<Path>>(&self, filepath: &P) -> std::io::Result<()> {
        let current_preset = Preset {
            kernel: self.simulation_data.uniform.get_kernel(),
            activation_code: self.activation_code.clone(),
            display_frames_mode: self.display_frames_mode.clone(),
            gradient: self.view_data.uniform.gradient.clone(),
            kernel_symmetry_mode: self.kernel_symmetry_mode,
        };

        preset::save_preset(filepath, &current_preset)
    }

    pub fn try_generate_simulation_pipeline(
        &mut self,
        device: &mut wgpu::Device,
        surface_configuration: &wgpu::SurfaceConfiguration,
    ) -> Result<(), wgpu::Error> {
        let (tx, rx) = std::sync::mpsc::channel::<wgpu::Error>();
        device.on_uncaptured_error(move |e: wgpu::Error| {
            tx.send(e).expect("sending error failed");
        });

        let shader_code: String = generate_simulation_shader(&self.activation_code);
        let simulation_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Simulation Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        let simulation_render_pipeline = build_simulation_pipeline(
            device,
            surface_configuration,
            &self.primitive_state,
            &self.multisample_state,
            &self.screen_shader,
            &simulation_shader,
            &self.simulation_textures,
            &self.simulation_data,
        );

        device.on_uncaptured_error(|e| panic!("{}", e));

        if let Ok(err) = rx.try_recv() {
            return Err(err);
        }

        self.simulation_render_pipeline = simulation_render_pipeline;
        self.shader_state = ShaderState::Compiled;

        Ok(())
    }

    pub fn try_update_simulation_size(
        &mut self,
        new_simulation_size: [u32; 2],
        device: &mut wgpu::Device,
        surface_configuration: &wgpu::SurfaceConfiguration,
    ) -> Result<(), wgpu::Error> {
        let (tx, rx) = std::sync::mpsc::channel::<wgpu::Error>();
        device.on_uncaptured_error(move |e: wgpu::Error| {
            tx.send(e).expect("sending error failed");
        });

        let texture_desc = get_texture_descriptor(&new_simulation_size);

        let (simulation_textures, bind_group_display_ping, bind_group_display_pong, bind_group_simulation_ping, bind_group_simulation_pong) =
            get_simulation_textures_and_bind_groups(device, &texture_desc)?;

        let screen_render_pipeline = build_screen_pipeline(
            device,
            surface_configuration,
            &self.primitive_state,
            &self.multisample_state,
            &self.screen_shader,
            &self.simulation_textures,
            &self.view_data,
        );

        let simulation_render_pipeline = build_simulation_pipeline(
            device,
            surface_configuration,
            &self.primitive_state,
            &self.multisample_state,
            &self.screen_shader,
            &self.simulation_shader,
            &self.simulation_textures,
            &self.simulation_data,
        );

        device.on_uncaptured_error(|e| panic!("{}", e));

        if let Ok(err) = rx.try_recv() {
            return Err(err);
        }

        self.simulation_size_state = SimulationSizeState::Compiled(new_simulation_size);
        self.init = false;
        self.simulation_textures = simulation_textures;
        self.bind_group_display_ping = bind_group_display_ping;
        self.bind_group_display_pong = bind_group_display_pong;
        self.bind_group_simulation_ping = bind_group_simulation_ping;
        self.bind_group_simulation_pong = bind_group_simulation_pong;
        self.screen_render_pipeline = screen_render_pipeline;
        self.simulation_render_pipeline = simulation_render_pipeline;
        self.simulation_data.set_simulation_size(&new_simulation_size);
        Ok(())
    }

    fn randomise_kernel(&mut self) {
        let mut rng = rand::thread_rng();
        let range: std::ops::Range<f32> = self.kernel_rand_range.x..self.kernel_rand_range.y;
        match self.kernel_symmetry_mode {
            KernelSymmetryMode::Any => {
                for i in 0..9 {
                    self.simulation_data.uniform.set_kernel_at(i, rng.gen_range(range.clone()), KernelSymmetryMode::Any);
                }
            },
            KernelSymmetryMode::Vertical => {
                for i in 0..6 {
                    self.simulation_data.uniform.set_kernel_at((i%3)*3+i/3, rng.gen_range(range.clone()), KernelSymmetryMode::Any);
                }
                self.simulation_data.uniform.apply_symmetry(KernelSymmetryMode::Vertical);
            },
            KernelSymmetryMode::Horizontal => {
                for i in 0..6 {
                    self.simulation_data.uniform.set_kernel_at(i, rng.gen_range(range.clone()), KernelSymmetryMode::Any);
                }
                self.simulation_data.uniform.apply_symmetry(KernelSymmetryMode::Horizontal);
            },
            KernelSymmetryMode::Full => {
                for i in 0..2 {
                    for j in 0..2 {
                        self.simulation_data.uniform.set_kernel_at(j*3+i, rng.gen_range(range.clone()), KernelSymmetryMode::Any);
                    }
                }
                self.simulation_data.uniform.apply_symmetry(KernelSymmetryMode::Full);
            },
        }
        self.simulation_data.need_update = true;
    }
}

impl App for NcaApp {
    fn create(_app_state: &mut AppState) -> Self {
        let (_, default_preset) = PRESETS.iter().next().unwrap();
        let activation_code = default_preset.activation_code.clone();

        let size = _app_state.window.inner_size();

        let simulation_size: [u32; 2] = [2000, 2000];
        // Texture
        let texture_desc = get_texture_descriptor(&simulation_size);

        let init_simulation_data = InitSimulationData::new(&_app_state.device);

        let mut simulation_data = SimulationData::new(&_app_state.device, &simulation_size);
        simulation_data.uniform.set_kernel(default_preset.kernel);
        simulation_data.need_update = true;

        let view_data = ViewData::new(&_app_state.device);

        let (simulation_textures, bind_group_display_ping, bind_group_display_pong, bind_group_simulation_ping, bind_group_simulation_pong) =
            get_simulation_textures_and_bind_groups(&mut _app_state.device, &texture_desc).expect("");

        // Shaders
        let screen_shader = _app_state.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Screne Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/Screen.wgsl").into()),
        });

        let shader_code: String = generate_simulation_shader(&activation_code);
        let simulation_shader = _app_state.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Simulation Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
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
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            ..Default::default()
        };

        let multisample_state = wgpu::MultisampleState::default();

        let screen_render_pipeline = build_screen_pipeline(
            &mut _app_state.device,
            &_app_state.config,
            &primitive_state,
            &multisample_state,
            &screen_shader,
            &simulation_textures,
            &view_data,
        );

        let init_simulation_render_pipeline = build_init_simulation_pipeline(
            &mut _app_state.device,
            &_app_state.config,
            &primitive_state,
            &multisample_state,
            &screen_shader,
            &init_simulation_shader,
            &init_simulation_data,
        );

        let simulation_render_pipeline = build_simulation_pipeline(
            &mut _app_state.device,
            &_app_state.config,
            &primitive_state,
            &multisample_state,
            &screen_shader,
            &simulation_shader,
            &simulation_textures,
            &simulation_data,
        );

        Self {
            clear_color: wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 },
            simulation_size_state: SimulationSizeState::Compiled(simulation_size),
            primitive_state,
            multisample_state,

            screen_shader,
            simulation_shader,

            init_simulation_render_pipeline,
            simulation_render_pipeline,
            screen_render_pipeline,
            simulation_textures,
            init_simulation_data,
            simulation_data,
            init: false,
            reset_on_randomize: true,
            kernel_rand_range: DisplayableVec2(glm::vec2(-1.0, 1.0)),
            bind_group_display_ping,
            bind_group_display_pong,
            bind_group_simulation_ping,
            bind_group_simulation_pong,

            ui_central_viewport: Viewport {
                x: 0.0,
                y: 0.0,
                width: size.width as f32,
                height: size.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            },

            target_delta: Duration::from_secs_f64(1.0 / 30.0),
            last_simulation_end: Instant::now(),

            activation_code,
            shader_state: ShaderState::Compiled,
            display_frames_mode: DisplayFramesMode::All,
            view_data,
            kernel_symmetry_mode: KernelSymmetryMode::Any,
        }
    }

    fn handle_event(&mut self, _app_state: &mut AppState, _event: &Event<()>) -> Result<()> {
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
                },
                WindowEvent::MouseWheel {
                    delta: MouseScrollDelta::LineDelta(_, y), ..
                } => {
                    let window_scale_factor = _app_state.window.scale_factor() as f32;

                    let mouse_pos = &_app_state.input_state.mouse.position;
                    let viewport_min_position = glm::vec2(self.ui_central_viewport.x, self.ui_central_viewport.y) * window_scale_factor;
                    let viewport_size = glm::vec2(self.ui_central_viewport.width, self.ui_central_viewport.height) * window_scale_factor;
                    let normalized_mouse_pos_within_viewport = (*mouse_pos - viewport_min_position).zip_map(&viewport_size, |y, x| y / x);
                    // let mouse_pos_within_simulation = self.view_data.uniform.center + (normalized_mouse_pos_within_viewport - glm::vec2(0.5, 0.5)) * self.view_data.uniform.zoom_level;

                    let old_zoom_level = self.view_data.uniform.zoom_level;
                    self.view_data.uniform.zoom_level = (self.view_data.uniform.zoom_level * 1.08_f32.powf(-*y)).min(1.0);

                    let zoom_delta = old_zoom_level - self.view_data.uniform.zoom_level;
                    if *y > 0. {
                        self.view_data.uniform.center += (normalized_mouse_pos_within_viewport - glm::vec2(0.5, 0.5)) * zoom_delta;
                    } else {
                        if old_zoom_level != 1.0 {
                            self.view_data.uniform.center +=
                                (glm::vec2(0.5, 0.5) - self.view_data.uniform.center) / (old_zoom_level - 1.0) * zoom_delta;
                        }
                    }

                    self.view_data.need_update = true;
                },
                _ => {},
            },
            _ => {},
        };

        Ok(())
    }

    fn render_gui(&mut self, _ctx: &epi::egui::Context) -> Result<()> {
        egui::TopBottomPanel::top("top_panel").resizable(true).show(&_ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        match nfd2::open_file_dialog(Some("json"), None).expect("Unable to open the file") {
                            nfd2::Response::Okay(file_path) => {
                                let path: &Path = file_path.as_path();
                                self.load_preset_from_file(&path).unwrap_or_else(|error| {
                                    println!("Unable to load preset from the file at path {}.\n {:?}", path.display(), error);
                                });
                            },
                            nfd2::Response::OkayMultiple(_) => println!("Multiple files selection should not happen here."),
                            nfd2::Response::Cancel => (),
                        }
                    }
                    if ui.button("Save").clicked() {
                        match nfd2::open_save_dialog(Some("json"), None).expect("Unable to save the file") {
                            nfd2::Response::Okay(file_path) => {
                                let path: &Path = file_path.as_path();
                                self.save_preset(&path).unwrap_or_else(|error| {
                                    println!("Unable to save the preset at path {}.\n {:?}", path.display(), error);
                                });
                            },
                            nfd2::Response::OkayMultiple(_) => println!("Multiple files selection should not happen here."),
                            nfd2::Response::Cancel => (),
                        }
                    }
                });
                ui.menu_button("Preset", |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let mut preset_to_apply: Option<(&'static str, Preset)> = None;
                        for (name, preset) in PRESETS.iter() {
                            if ui.button(*name).clicked() {
                                preset_to_apply = Some((name, preset.clone()));
                            }
                        }
                        if let Some((preset_name, preset)) = preset_to_apply {
                            self.load_preset(preset).unwrap_or_else(|error| {
                                println!("Unable to load selected preset : {}.\n {:?}", preset_name, error);
                            });
                            ui.close_menu();
                        }
                    });
                });
            });
        });

        egui::SidePanel::left("left_panel").resizable(true).show(&_ctx, |ui| {
            ui.heading("Left Panel");

            egui::CollapsingHeader::new("Simulation settings").default_open(true).show(ui, |ui| {
                ui.add(
                    egui::DragValue::from_get_set(|optional_value: Option<f64>| {
                        if let Some(v) = optional_value {
                            self.simulation_size_state = SimulationSizeState::Dirty([v as u32, v as u32]);
                        }
                        match self.simulation_size_state {
                            SimulationSizeState::Compiled(size) => size[0] as f64,
                            SimulationSizeState::Dirty(size) => size[0] as f64,
                        }
                    })
                    .speed(1)
                    .prefix("simulation size: "),
                );
            });

            egui::CollapsingHeader::new("Starting settings").default_open(true).show(ui, |ui| {
                ui.separator();
                ui.add(
                    egui::DragValue::from_get_set(|optional_value: Option<f64>| {
                        if let Some(v) = optional_value {
                            self.init_simulation_data.uniform.seed = v as f32;
                            self.init_simulation_data.need_update = true;
                        }
                        self.init_simulation_data.uniform.seed as f64
                    })
                    .speed(0.1)
                    .prefix("seed: "),
                );

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

                ui.checkbox(&mut self.reset_on_randomize, "reset on randomize");
            });

            egui::CollapsingHeader::new("Kernel").default_open(true).show(ui, |ui| {
                egui::Grid::new("some_unique_id").show(ui, |ui| {
                    for j in 0..3 {
                        for i in 0..3 {
                            ui.add(
                                egui::DragValue::from_get_set(|optional_value: Option<f64>| {
                                    if let Some(v) = optional_value {
                                        self.simulation_data.uniform.set_kernel_at(j * 3 + i, v as f32, self.kernel_symmetry_mode);
                                        self.simulation_data.need_update = true;
                                    }
                                    self.simulation_data.uniform.get_kernel_at(j * 3 + i) as f64
                                })
                                .speed(0.1),
                            );
                        }
                        ui.end_row();
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Symmetry mode: ");
                    egui::ComboBox::from_id_source("Symmetry mode: ")
                        .selected_text(self.kernel_symmetry_mode.to_string())
                        .show_ui(ui, |ui| {
                            let mut changed: bool = false;
                            changed |= ui.selectable_value(&mut self.kernel_symmetry_mode, KernelSymmetryMode::Any, KernelSymmetryMode::Any.to_string()).changed();
                            changed |= ui.selectable_value(&mut self.kernel_symmetry_mode, KernelSymmetryMode::Vertical, KernelSymmetryMode::Vertical.to_string()).changed();
                            changed |= ui.selectable_value(&mut self.kernel_symmetry_mode, KernelSymmetryMode::Horizontal, KernelSymmetryMode::Horizontal.to_string()).changed();
                            changed |= ui.selectable_value(&mut self.kernel_symmetry_mode, KernelSymmetryMode::Full, KernelSymmetryMode::Full.to_string()).changed();
                            if changed { self.simulation_data.uniform.apply_symmetry(self.kernel_symmetry_mode); }
                        });
                });

                ui.separator();
                if ui.button("randomise kernel").clicked() {
                    self.randomise_kernel();

                    if self.reset_on_randomize {
                        self.init = false;
                    }
                }
                
                ui.horizontal(|ui| {
                    ui.label("Range: ");
                    ui.add(&mut self.kernel_rand_range);
                });
            });

            egui::CollapsingHeader::new("Simulation").default_open(true).show(ui, |ui| {
                ui.separator();
                let mut code_editor = CodeEditor::new(&mut self.activation_code, "rs", Some(15));
                code_editor.show(ui);

                let code_to_paste: Option<String> = _ctx.input().events.iter().find_map(|e| match e {
                    egui::Event::Paste(paste_content) => Some((*paste_content).to_owned()),
                    _ => None,
                });

                if let Some(new_code) = code_to_paste {
                    self.activation_code = new_code;
                }

                if let ShaderState::CompilationFail(error) = &self.shader_state {
                    ui.label(format!("Shader compile error:\n {}", error));
                }
                
                ui.menu_button("Activation Presets", |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let mut preset_to_apply: Option<&'static str> = None;
                        for (name, preset) in ACTIVATION_FUNCTIONS_PRESETS.iter() {
                            if ui.button(*name).clicked() {
                                preset_to_apply = Some(preset);
                            }
                        }
                        if let Some(preset) = preset_to_apply {
                            self.activation_code = preset.to_owned();
                            self.shader_state = ShaderState::Dirty;
                            ui.close_menu();
                        }
                    });
                });

                if ui.button("Recompile").clicked() {
                    self.shader_state = ShaderState::Dirty;
                }
            });

            egui::CollapsingHeader::new("Display Options").default_open(true).show(ui, |ui| {
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Display frame mode: ");
                    egui::ComboBox::from_id_source("display_frames_mode")
                        .selected_text(format!("{:?}", self.display_frames_mode))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.display_frames_mode, DisplayFramesMode::All, "ALl");
                            ui.selectable_value(&mut self.display_frames_mode, DisplayFramesMode::Evens, "Evens");
                            ui.selectable_value(&mut self.display_frames_mode, DisplayFramesMode::Odd, "Odd");
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Target Simulation rate: ");
                    ui.add(
                        egui::DragValue::from_get_set(|new_value: Option<f64>| {
                            if let Some(value) = new_value {
                                self.target_delta = Duration::from_secs_f64(1.0 / value.max(1.0));
                            }

                            1.0 / self.target_delta.as_secs_f64()
                        })
                        .speed(1.0)
                        .min_decimals(1)
                        .max_decimals(60),
                    );

                    ui.label(" fps");
                });

                ui.separator();

                self.view_data.uniform.gradient.show(ui);
                if self.view_data.uniform.gradient.ui_control(ui) {
                    self.view_data.need_update = true;
                }

                ui.menu_button("Gradient Presets", |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let mut preset_to_apply: Option<IqGradient> = None;
                        for (name, preset) in IQ_GRADIENT_PRESETS.iter() {
                            if ui.button(*name).clicked() {
                                preset_to_apply = Some(preset.clone());
                            }
                        }
                        if let Some(preset) = preset_to_apply {
                            self.view_data.uniform.gradient = preset;
                            self.view_data.need_update = true;
                            ui.close_menu();
                        }
                    });
                });

            });

            ui.allocate_space(ui.available_size());
        });

        egui::TopBottomPanel::bottom("bottom_panel").resizable(true).show(&_ctx, |ui| {
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
            match self.try_generate_simulation_pipeline(&mut _app_state.device, &_app_state.config) {
                Err(err) => match err {
                    wgpu::Error::OutOfMemory { .. } => {
                        anyhow::bail!("Shader compilation gpu::Error::OutOfMemory")
                    },
                    wgpu::Error::Validation { description, .. } => self.shader_state = ShaderState::CompilationFail(description),
                },
                Ok(()) => {},
            }
        }

        if let SimulationSizeState::Dirty(new_simulation_size) = self.simulation_size_state {
            match self.try_update_simulation_size(new_simulation_size, &mut _app_state.device, &_app_state.config) {
                Err(err) => match err {
                    wgpu::Error::OutOfMemory { .. } => {
                        anyhow::bail!("Shader compilation gpu::Error::OutOfMemory")
                    },
                    wgpu::Error::Validation { description, .. } => self.shader_state = ShaderState::CompilationFail(description),
                },
                Ok(()) => {},
            }
        }

        Ok(())
    }

    fn render(
        &mut self,
        _app_state: &mut AppState,
        _encoder: &mut wgpu::CommandEncoder,
        _output_view: &wgpu::TextureView,
    ) -> Result<(), wgpu::SurfaceError> {
        if self.last_simulation_end.elapsed() > self.target_delta {
            // init if needed
            if self.init == false {
                self.init = true;

                if self.init_simulation_data.need_update {
                    self.init_simulation_data.update(&_app_state.queue);
                }

                let mut init_simulation_render_pass = _encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Init Simulation Render Pass"),
                    color_attachments: &[wgpu::RenderPassColorAttachment {
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
                    color_attachments: &[wgpu::RenderPassColorAttachment {
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
                let bind_group: &wgpu::BindGroup = if self.simulation_textures.state {
                    &self.bind_group_simulation_pong
                } else {
                    &self.bind_group_simulation_ping
                };
                simulation_render_pass.set_bind_group(0, bind_group, &[]);
                simulation_render_pass.set_bind_group(1, &self.simulation_data.bind_group, &[]);
                simulation_render_pass.draw(0..3, 0..1);
            }

            self.last_simulation_end = Instant::now();
            self.simulation_textures.toogle_state();
        };

        // render simulation on screen
        {
            if self.view_data.need_update {
                self.view_data.update(&_app_state.queue);
            }

            let mut screen_render_pass = _encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Screen Render Pass"),
                color_attachments: &[
                    // This is what [[location(0)]] in the fragment shader targets
                    wgpu::RenderPassColorAttachment {
                        view: &_output_view,
                        resolve_target: None,
                        ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: true },
                    },
                ],
                depth_stencil_attachment: None,
            });

            // update viewport accordingly to the Ui to display the simulation
            // it must be multiplied by window scale factor as render pass use physical pixels screen size

            let window_scale_factor = _app_state.window.scale_factor() as f32;
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
                DisplayFramesMode::All =>
                    if self.simulation_textures.state {
                        &self.bind_group_display_pong
                    } else {
                        &self.bind_group_display_ping
                    },
                DisplayFramesMode::Evens => &self.bind_group_display_pong,
                DisplayFramesMode::Odd => &self.bind_group_display_ping,
            };

            screen_render_pass.set_bind_group(0, bind_group, &[]);
            screen_render_pass.set_bind_group(1, &self.view_data.bind_group, &[]);
            screen_render_pass.draw(0..3, 0..1);
        }

        Ok(())
    }
}


lazy_static! {
    pub static ref ACTIVATION_FUNCTIONS_PRESETS: std::collections::HashMap<&'static str, &'static str> = std::collections::HashMap::from([
        (
            "Identity","
fn activationFunction(kernelOutput: vec4<f32>) -> vec4<f32> {
var r: f32 = kernelOutput.x;
return vec4<f32>(r, r, r, 1.0);
}"
        ),
        (
            "Sin","
fn activationFunction(kernelOutput: vec4<f32>) -> vec4<f32> {
var r: f32 = sin(kernelOutput.x);
return vec4<f32>(r, r, r, 1.0);
}"
        ),
        (
            "Abs","
fn activationFunction(kernelOutput: vec4<f32>) -> vec4<f32> {
var r: f32 = abs(kernelOutput.x);
return vec4<f32>(r, r, r, 1.0);
}"
        ),
        (
            "Power","
fn activationFunction(kernelOutput: vec4<f32>) -> vec4<f32> {
var r: f32 = pow(kernelOutput.x, 2.0);
return vec4<f32>(r, r, r, 1.0);
}"
        ),
        (
            "Tanh","
fn activationFunction(kernelOutput: vec4<f32>) -> vec4<f32> {
var r: f32 = (exp(2. * kernelOutput.x) -1.) / (exp(2. * kernelOutput.x) + 1.);
return vec4<f32>(r, r, r, 1.0);
}"
        ),
        (
            " inverted gaussian","
// an inverted gaussian function, 
// where f(0) = 0. 
// Graph: https://www.desmos.com/calculator/torawryxnq
fn activationFunction(kernelOutput: vec4<f32>) -> vec4<f32> {
var r: f32 = -1./(0.89*pow(kernelOutput.x, 2.)+1.)+1.;
return vec4<f32>(r, r, r, 1.0);
}"
        ),
        
        
    ]);
}
