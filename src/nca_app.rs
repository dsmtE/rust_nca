use anyhow::Result;

use wgpu::SurfaceConfiguration;
use winit::event::{Event, MouseScrollDelta, WindowEvent};

use std::time::{Duration, Instant};

use skeleton_app::{App, AppState};

use epi;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use nalgebra_glm as glm;

use crate::{
    egui_widgets::{CodeEditor, UiWidget},
    preset::{load_preset, save_preset, Preset},
    simulation_data::{InitSimulationData, SimulationData},
    utils::ping_pong_texture::PingPongTexture,
    view_data::ViewData,
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
    presets_list: HashMap<String, Preset>,
    clear_color: wgpu::Color,
    Simulation_size_state: SimulationSizeState,
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
    init: bool,

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

fn GenerateSimulationShader(activation_code: &str) -> String {
    include_str!("shaders/simulationBase.wgsl").replace("[functionTemplate]", activation_code)
}

fn GetPresets() -> HashMap<String, Preset> {
    HashMap::from([
        (
            "Game Of life".to_owned(),
            Preset {
                kernel: [1., 1., 1., 1., 9., 1., 1., 1., 1.],
                activation_code: "
fn activationFunction(kernelOutput: vec4<f32>) -> vec4<f32> {
var condition: bool = kernelOutput.x == 3.0 || kernelOutput.x == 11.0 || kernelOutput.x == 12.0;
var r: f32 = select(0.0, 1.0, condition);
return vec4<f32>(r, r, r, 1.0);
}"
                .to_owned(),
                display_frames_mode: DisplayFramesMode::All,
            },
        ),
        (
            "Slime".to_owned(),
            Preset {
                kernel: [0.8, -0.85, 0.8, -0.85, -0.2, -0.85, 0.8, -0.85, 0.8],
                activation_code: "
// an inverted gaussian function, 
// where f(0) = 0. 
// Graph: https://www.desmos.com/calculator/torawryxnq          

fn activationFunction(kernelOutput: vec4<f32>) -> vec4<f32> {
var r: f32 = -1./(0.89*pow(kernelOutput.x, 2.)+1.)+1.;
return vec4<f32>(r, r, r, 1.0);
}"
                .to_owned(),
                display_frames_mode: DisplayFramesMode::Evens,
            },
        ),
        (
            "Waves".to_owned(),
            Preset {
                kernel: [
                    0.564599, -0.715900, 0.564599, -0.715900, 0.626900, -0.715900, 0.564599,
                    -0.715900, 0.564599,
                ],
                activation_code: "
fn activationFunction(kernelOutput: vec4<f32>) -> vec4<f32> {
var r: f32 = abs(1.2*kernelOutput.x);
return vec4<f32>(r, r, r, 1.0);
}"
                .to_owned(),
                display_frames_mode: DisplayFramesMode::All,
            },
        ),
        (
            "Stars".to_owned(),
            Preset {
                kernel: [
                    0.56459, -0.71590, 0.56459, -0.75859, 0.62690, -0.75859, 0.56459, -0.71590,
                    0.56459,
                ],
                activation_code: "
fn activationFunction(kernelOutput: vec4<f32>) -> vec4<f32> {
var r: f32 = abs(kernelOutput.x);
return vec4<f32>(r, r, r, 1.0);
}"
                .to_owned(),
                display_frames_mode: DisplayFramesMode::All,
            },
        ),
        (
            "Pathways".to_owned(),
            Preset {
                kernel: [0., 1., 0., 1., 1., 1., 0., 1., 0.],
                activation_code: "
fn gaussian(x: f32, b: f32) -> f32{
return 1./pow(2., (pow(x-b, 2.)));
}

fn activationFunction(kernelOutput: vec4<f32>) -> vec4<f32> {
var r: f32 = gaussian(kernelOutput.x, 3.5);
return vec4<f32>(r, r, r, 1.0);
}"
                .to_owned(),
                display_frames_mode: DisplayFramesMode::All,
            },
        ),
        (
            "Mitosis".to_owned(),
            Preset {
                kernel: [
                    -0.939, 0.879, -0.939, 0.879, 0.4, 0.879, -0.939, 0.879, -0.939,
                ],
                activation_code: "
// an inverted gaussian function, 
// where f(0) = 0. 
// Graph: https://www.desmos.com/calculator/torawryxnq\
                                  

fn activationFunction(kernelOutput: vec4<f32>) -> vec4<f32> {
var r: f32 = -1. / (0.9*pow(kernelOutput.x, 2.)+1.)+1.;
return vec4<f32>(r, r, r, 1.0);
}"
                .to_owned(),
                display_frames_mode: DisplayFramesMode::All,
            },
        ),
        (
            "Blob".to_owned(),
            Preset {
                kernel: [
                    0.7795687913894653,
                    -0.7663648128509521,
                    0.7795687913894653,
                    -0.7663648128509521,
                    -0.29899999499320984,
                    -0.7663648128509521,
                    0.7795687913894653,
                    -0.7663648128509521,
                    0.7795687913894653,
                ],
                activation_code: "
fn activationFunction(kernelOutput: vec4<f32>) -> vec4<f32> {
var r: f32 = -1. / pow(2., (pow(kernelOutput.x, 2.)))+1.;
return vec4<f32>(r, r, r, 1.0);
}"
                .to_owned(),
                display_frames_mode: DisplayFramesMode::All,
            },
        ),
        (
            "test".to_owned(),
            Preset {
                kernel: [
                    0.5669999718666077,
                    -0.7149999737739563,
                    0.5669999718666077,
                    -0.7149999737739563,
                    0.6370000243186951,
                    -0.7149999737739563,
                    0.5669999718666077,
                    -0.7149999737739563,
                    0.5669999718666077,
                ],
                activation_code: "
fn activationFunction(kernelOutput: vec4<f32>) -> vec4<f32> {
var r: f32 = abs(kernelOutput.x);
return vec4<f32>(r, r, r, 1.0);
}"
                .to_owned(),
                display_frames_mode: DisplayFramesMode::Evens,
            },
        ),
        (
            "test2".to_owned(),
            Preset {
                kernel: [
                    91.627685546875,
                    -59.281097412109375,
                    91.627685546875,
                    -59.281097412109375,
                    -42.35920715332031,
                    -59.281097412109375,
                    91.627685546875,
                    -59.281097412109375,
                    91.627685546875,
                ],
                activation_code: "
fn activationFunction(kernelOutput: vec4<f32>) -> vec4<f32> {
var r: f32 = (exp(2.*kernelOutput.x) - 1.) / (exp(2.*kernelOutput.x) + 1.);
return vec4<f32>(r, r, r, 1.0);
}"
                .to_owned(),
                display_frames_mode: DisplayFramesMode::Evens,
            },
        ),
    ])
}

impl NcaApp {
    pub fn load_preset_from_file(&mut self, filepath: &str) -> Result<()> {
        let preset: Preset = load_preset(filepath)?;

        self.simulation_data.uniform.kernel = preset.kernel;
        self.simulation_data.need_update = true;
        Ok(())
    }

    pub fn load_preset(&mut self, preset: &Preset) -> Result<()> {
        self.simulation_data.uniform.kernel = preset.kernel;
        self.simulation_data.need_update = true;

        self.activation_code = preset.activation_code.clone();
        self.shader_state = ShaderState::Dirty;

        self.display_frames_mode = preset.display_frames_mode.clone();

        Ok(())
    }

    pub fn save_preset(&self, filepath: &str) -> std::io::Result<()> {
        let mut current_preset = Preset {
            kernel: self.simulation_data.uniform.kernel,
            activation_code: self.activation_code.clone(),
            display_frames_mode: self.display_frames_mode.clone(),
        };

        save_preset(filepath, &current_preset)
    }

    pub fn try_generate_simulation_pipeline(
        &mut self,
        device: &mut wgpu::Device,
        surface_configuration: &wgpu::SurfaceConfiguration,
    ) -> Result<(), wgpu::Error> {
        let shader_code: String = GenerateSimulationShader(&self.activation_code);

        let (tx, rx) = std::sync::mpsc::channel::<wgpu::Error>();
        device.on_uncaptured_error(move |e: wgpu::Error| {
            tx.send(e).expect("sending error failed");
        });

        let simulation_shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: Some("Simulation Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        let simulation_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Simulation Render Pipeline"),
                layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Simulation Pipeline Layout"),
                    bind_group_layouts: &[
                        &self.simulation_textures.bind_group_layout,
                        &self.simulation_data.bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                })),
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

        self.simulation_render_pipeline = simulation_render_pipeline;
        self.shader_state = ShaderState::Compiled;

        Ok(())
    }

    pub fn try_update_simulation_size(
        &mut self,
        new_simulation_size: [u32; 2],
        device: &mut wgpu::Device,
        surface_configuration: &SurfaceConfiguration,
    ) -> Result<(), wgpu::Error> {
        let (tx, rx) = std::sync::mpsc::channel::<wgpu::Error>();
        device.on_uncaptured_error(move |e: wgpu::Error| {
            tx.send(e).expect("sending error failed");
        });

        let texture_desc = wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: new_simulation_size[0],
                height: new_simulation_size[1],
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            label: None,
        };

        let simulation_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
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

        let simulation_textures =
            PingPongTexture::from_descriptor(device, &texture_desc, Some("simulation"))?;

        let (bind_group_display_ping, bind_group_display_pong) =
            simulation_textures.create_binding_group(device, &display_sampler);
        let (bind_group_simulation_ping, bind_group_simulation_pong) =
            simulation_textures.create_binding_group(device, &simulation_sampler);

        let screen_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Screen Render Pipeline"),
                layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Screen Pipeline Layout"),
                    bind_group_layouts: &[
                        &simulation_textures.bind_group_layout,
                        &self.view_data.bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                })),
                vertex: wgpu::VertexState {
                    module: &self.screen_shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &self.screen_shader,
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

        let simulation_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Simulation Render Pipeline"),
                layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Simulation Pipeline Layout"),
                    bind_group_layouts: &[
                        &simulation_textures.bind_group_layout,
                        &self.simulation_data.bind_group_layout,
                    ],
                    push_constant_ranges: &[],
                })),
                vertex: wgpu::VertexState {
                    module: &self.screen_shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &self.simulation_shader,
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

        self.Simulation_size_state = SimulationSizeState::Compiled(new_simulation_size);
        self.init = false;
        self.simulation_textures = simulation_textures;
        self.screen_render_pipeline = screen_render_pipeline;
        self.simulation_render_pipeline = simulation_render_pipeline;
        self.bind_group_display_ping = bind_group_display_ping;
        self.bind_group_display_pong = bind_group_display_pong;
        self.bind_group_simulation_ping = bind_group_simulation_ping;
        self.bind_group_simulation_pong = bind_group_simulation_pong;
        self.simulation_data
            .set_simulation_size(&new_simulation_size);
        Ok(())
    }
}

impl App for NcaApp {
    fn create(_app_state: &mut AppState) -> Self {
        let presets_list = GetPresets();

        let (_, default_preset) = presets_list.iter().next().unwrap();
        let activation_code = default_preset.activation_code.clone();

        let size = _app_state.window.inner_size();

        let ui_central_viewport = Viewport {
            x: 0.0,
            y: 0.0,
            width: size.width as f32,
            height: size.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let simulation_size: [u32; 2] = [2000, 2000];
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

        let init_simulation_data = InitSimulationData::new(&_app_state.device);

        let simulation_textures =
            PingPongTexture::from_descriptor(&_app_state.device, &texture_desc, Some("simulation"))
                .unwrap();

        let mut simulation_data = SimulationData::new(&_app_state.device, &simulation_size);
        simulation_data.uniform.kernel = default_preset.kernel;
        simulation_data.need_update = true;

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

        let view_data = ViewData::new(&_app_state.device);

        let (bind_group_display_ping, bind_group_display_pong) =
            simulation_textures.create_binding_group(&_app_state.device, &display_sampler);
        let (bind_group_simulation_ping, bind_group_simulation_pong) =
            simulation_textures.create_binding_group(&_app_state.device, &simulation_sampler);

        // Shaders
        let screen_shader = _app_state
            .device
            .create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: Some("Screne Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shaders/Screen.wgsl").into()),
            });

        let shader_code: String = GenerateSimulationShader(&activation_code);
        let simulation_shader =
            _app_state
                .device
                .create_shader_module(&wgpu::ShaderModuleDescriptor {
                    label: Some("Simulation Shader"),
                    source: wgpu::ShaderSource::Wgsl(shader_code.into()),
                });

        let init_simulation_shader =
            _app_state
                .device
                .create_shader_module(&wgpu::ShaderModuleDescriptor {
                    label: Some("Init Simulation Shader"),
                    source: wgpu::ShaderSource::Wgsl(
                        include_str!("shaders/init_simulation.wgsl").into(),
                    ),
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

        let screen_render_pipeline =
            _app_state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Screen Render Pipeline"),
                    layout: Some(&_app_state.device.create_pipeline_layout(
                        &wgpu::PipelineLayoutDescriptor {
                            label: Some("Screen Pipeline Layout"),
                            bind_group_layouts: &[
                                &simulation_textures.bind_group_layout,
                                &view_data.bind_group_layout,
                            ],
                            push_constant_ranges: &[],
                        },
                    )),
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

        let init_simulation_render_pipeline =
            _app_state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Init Simulation Render Pipeline"),
                    layout: Some(&_app_state.device.create_pipeline_layout(
                        &wgpu::PipelineLayoutDescriptor {
                            label: Some("Init Simulation Pipeline Layout"),
                            bind_group_layouts: &[&init_simulation_data.bind_group_layout],
                            push_constant_ranges: &[],
                        },
                    )),
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

        let simulation_render_pipeline =
            _app_state
                .device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Simulation Render Pipeline"),
                    layout: Some(&_app_state.device.create_pipeline_layout(
                        &wgpu::PipelineLayoutDescriptor {
                            label: Some("Simulation Pipeline Layout"),
                            bind_group_layouts: &[
                                &simulation_textures.bind_group_layout,
                                &simulation_data.bind_group_layout,
                            ],
                            push_constant_ranges: &[],
                        },
                    )),
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
            presets_list,
            clear_color: wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 },
            Simulation_size_state: SimulationSizeState::Compiled(simulation_size),
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

            bind_group_display_ping,
            bind_group_display_pong,
            bind_group_simulation_ping,
            bind_group_simulation_pong,

            ui_central_viewport,

            target_delta: Duration::from_secs_f64(1.0 / 30.0),
            last_simulation_end: Instant::now(),

            activation_code,
            shader_state: ShaderState::Compiled,
            display_frames_mode: DisplayFramesMode::All,
            view_data,
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
                }
                WindowEvent::MouseWheel {
                    delta: MouseScrollDelta::LineDelta(_, y), ..
                } => {
                    let window_scale_factor = _app_state.window.scale_factor() as f32;

                    let mouse_pos = &_app_state.input_state.mouse.position;
                    let viewport_min_position =
                        glm::vec2(self.ui_central_viewport.x, self.ui_central_viewport.y)
                            * window_scale_factor;
                    let viewport_size =
                        glm::vec2(self.ui_central_viewport.width, self.ui_central_viewport.height)
                            * window_scale_factor;
                    let normalized_mouse_pos_within_viewport =
                        (*mouse_pos - viewport_min_position).zip_map(&viewport_size, |y, x| y / x);
                    // let mouse_pos_within_simulation = self.view_data.uniform.center + (normalized_mouse_pos_within_viewport - glm::vec2(0.5, 0.5)) * self.view_data.uniform.zoom_level;

                    let old_zoom_level = self.view_data.uniform.zoom_level;
                    self.view_data.uniform.zoom_level =
                        (self.view_data.uniform.zoom_level * 1.08_f32.powf(-*y)).min(1.0);

                    let zoom_delta = old_zoom_level - self.view_data.uniform.zoom_level;
                    if *y > 0. {
                        self.view_data.uniform.center += (normalized_mouse_pos_within_viewport
                            - glm::vec2(0.5, 0.5))
                            * zoom_delta;
                    } else {
                        if old_zoom_level != 1.0 {
                            self.view_data.uniform.center += (glm::vec2(0.5, 0.5)
                                - self.view_data.uniform.center)
                                / (old_zoom_level - 1.0)
                                * zoom_delta;
                        }
                    }

                    self.view_data.need_update = true;
                }
                _ => {}
            },
            _ => {}
        };

        Ok(())
    }

    fn render_gui(&mut self, _ctx: &epi::egui::Context) -> Result<()> {
        egui::TopBottomPanel::top("top_panel")
            .resizable(true)
            .show(&_ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Open").clicked() {
                            self.load_preset_from_file("testSave.json");
                        }
                        if ui.button("Save").clicked() {
                            self.save_preset("testSave.json");
                        }
                    });

                    ui.menu_button("Preset", |ui| {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            let mut preset_to_apply: Option<Preset> = None;
                            for (name, preset) in self.presets_list.iter() {
                                if ui.button(name).clicked() {
                                    preset_to_apply = Some(preset.clone());
                                }
                            }

                            if let Some(preset) = preset_to_apply {
                                self.load_preset(&preset);
                            }
                        });
                    });
                });
            });

        egui::SidePanel::left("left_panel")
            .resizable(true)
            .show(&_ctx, |ui| {
                ui.heading("Left Panel");

                egui::CollapsingHeader::new("Simulation settings")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.add(
                            egui::DragValue::from_get_set(|optional_value: Option<f64>| {
                                if let Some(v) = optional_value {
                                    self.Simulation_size_state =
                                        SimulationSizeState::Dirty([v as u32, v as u32]);
                                }
                                match self.Simulation_size_state {
                                    SimulationSizeState::Compiled(size) => size[0] as f64,
                                    SimulationSizeState::Dirty(size) => size[0] as f64,
                                }
                            })
                            .speed(1)
                            .prefix("simulation size: "),
                        );
                    });

                egui::CollapsingHeader::new("Starting settings")
                    .default_open(true)
                    .show(ui, |ui| {
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
                    });

                egui::CollapsingHeader::new("Kernel")
                    .default_open(true)
                    .show(ui, |ui| {
                        egui::Grid::new("some_unique_id").show(ui, |ui| {
                            for j in 0..3 {
                                for i in 0..3 {
                                    ui.add(
                                        egui::DragValue::from_get_set(
                                            |optional_value: Option<f64>| {
                                                if let Some(v) = optional_value {
                                                    self.simulation_data.uniform.kernel
                                                        [j * 3 + i] = v as f32;
                                                    self.simulation_data.need_update = true;
                                                }
                                                self.simulation_data.uniform.kernel[j * 3 + i]
                                                    as f64
                                            },
                                        )
                                        .speed(0.1),
                                    );
                                }
                                ui.end_row();
                            }
                        });
                    });

                egui::CollapsingHeader::new("Simulation")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.separator();
                        let mut code_editor =
                            CodeEditor::new(&mut self.activation_code, "rs", Some(15));
                        code_editor.show(ui);

                        let code_to_paste: Option<String> =
                            _ctx.input().events.iter().find_map(|e| match e {
                                egui::Event::Paste(paste_content) => {
                                    Some((*paste_content).to_owned())
                                }
                                _ => None,
                            });

                        if let Some(new_code) = code_to_paste {
                            self.activation_code = new_code;
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
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label("Display frame mode: ");
                            egui::ComboBox::from_id_source("display_frames_mode")
                                .selected_text(format!("{:?}", self.display_frames_mode))
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.display_frames_mode,
                                        DisplayFramesMode::All,
                                        "ALl",
                                    );
                                    ui.selectable_value(
                                        &mut self.display_frames_mode,
                                        DisplayFramesMode::Evens,
                                        "Evens",
                                    );
                                    ui.selectable_value(
                                        &mut self.display_frames_mode,
                                        DisplayFramesMode::Odd,
                                        "Odd",
                                    );
                                });
                        });

                        ui.horizontal(|ui| {
                            ui.label("Target Simulation rate: ");
                            ui.add(
                                egui::DragValue::from_get_set(|new_value: Option<f64>| {
                                    if let Some(value) = new_value {
                                        self.target_delta =
                                            Duration::from_secs_f64(1.0 / value.max(1.0));
                                    }

                                    1.0 / self.target_delta.as_secs_f64()
                                })
                                .speed(1.0)
                                .min_decimals(1)
                                .max_decimals(60),
                            );

                            ui.label(" fps");
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
            match self.try_generate_simulation_pipeline(&mut _app_state.device, &_app_state.config)
            {
                Err(err) => match err {
                    wgpu::Error::OutOfMemory { .. } => {
                        anyhow::bail!("Shader compilation gpu::Error::OutOfMemory")
                    }
                    wgpu::Error::Validation { description, .. } => {
                        self.shader_state = ShaderState::CompilationFail(description)
                    }
                },
                Ok(()) => {}
            }
        }

        if let SimulationSizeState::Dirty(new_simulation_size) = self.Simulation_size_state {
            match self.try_update_simulation_size(
                new_simulation_size,
                &mut _app_state.device,
                &_app_state.config,
            ) {
                Err(err) => match err {
                    wgpu::Error::OutOfMemory { .. } => {
                        anyhow::bail!("Shader compilation gpu::Error::OutOfMemory")
                    }
                    wgpu::Error::Validation { description, .. } => {
                        self.shader_state = ShaderState::CompilationFail(description)
                    }
                },
                Ok(()) => {}
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

                let mut init_simulation_render_pass =
                    _encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                init_simulation_render_pass.set_bind_group(
                    0,
                    &self.init_simulation_data.bind_group,
                    &[],
                );
                init_simulation_render_pass.draw(0..3, 0..1);
            }

            // simulation
            {
                if self.simulation_data.need_update {
                    self.simulation_data.update(&_app_state.queue);
                }

                let mut simulation_render_pass =
                    _encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
                DisplayFramesMode::All => {
                    if self.simulation_textures.state {
                        &self.bind_group_display_pong
                    } else {
                        &self.bind_group_display_ping
                    }
                }
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
