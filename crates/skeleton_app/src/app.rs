use std::iter;

use winit::{
    event::{ElementState, Event, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Icon, WindowBuilder},
};

use anyhow::Result;

use crate::{
    gui_render_wgpu::{Gui, GuiRenderWgpu, ScreenDescriptor},
    input::{InputsState, SystemState, WinitEventHandler},
};

use spin_sleep::LoopHelper;

pub struct AppState {
    pub window: winit::window::Window,

    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub clear_color: wgpu::Color,

    pub gui: Gui,
    pub gui_render: GuiRenderWgpu,

    pub input_state: InputsState,
    pub system_state: SystemState,

    pub loop_helper: LoopHelper,
}

impl AppState {
    pub fn set_fullscreen(&mut self) {
        self.window
            .set_fullscreen(Some(winit::window::Fullscreen::Borderless(self.window.primary_monitor())));
    }
}

pub trait App {
    fn create(_app_state: &mut AppState) -> Self;

    fn update(&mut self, _app_state: &mut AppState) -> Result<()> { Ok(()) }

    fn render_gui(&mut self, _ctx: &epi::egui::Context) -> Result<()> { Ok(()) }

    fn render(
        &mut self,
        _app_state: &mut AppState,
        _encoder: &mut wgpu::CommandEncoder,
        _output_view: &wgpu::TextureView,
    ) -> Result<(), wgpu::SurfaceError> {
        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> { Ok(()) }

    fn on_mouse(&mut self, _app_state: &mut AppState, _button: &MouseButton, _button_state: &ElementState) -> Result<()> { Ok(()) }
    fn on_key(&mut self, _app_state: &mut AppState, _input: KeyboardInput) -> Result<()> { Ok(()) }

    fn handle_event(&mut self, _app_state: &mut AppState, _event: &Event<()>) -> Result<()> { Ok(()) }
}

pub struct AppConfig {
    pub is_resizable: bool,
    pub title: String,
    pub icon: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            is_resizable: false,
            title: "Application".to_string(),
            icon: None,
        }
    }
}

pub fn run_application<T: App + 'static>(config: AppConfig) -> Result<()> {
    let event_loop = EventLoop::new();

    let mut window_builder = WindowBuilder::new()
        .with_decorations(true)
        .with_resizable(config.is_resizable)
        .with_transparent(false)
        .with_title(config.title.to_string());

    if let Some(icon_path) = config.icon.as_ref() {
        let image = image::io::Reader::open(icon_path)?.decode()?.into_rgba8();
        let (width, height) = image.dimensions();
        let icon = Icon::from_rgba(image.into_raw(), width, height)?;
        window_builder = window_builder.with_window_icon(Some(icon));
    }

    // if let Some(default_dimension) = config.default_dimension {
    //     let (width, height) = default_dimension;
    //     window_builder = window_builder.with_inner_size(PhysicalSize::new(width, height));
    // }

    let window = window_builder.build(&event_loop)?;

    let window_dimensions = window.inner_size();

    // TODO: add Input system (for mouse etc)

    // TODO : encapsulate renderer initialisation
    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let surface = unsafe { instance.create_surface(&window) };

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .unwrap();

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::default(),
            limits: wgpu::Limits::default(),
        },
        None,
    ))?;
    // .ok_or(Err(anyhow::anyhow!("Unable to request device")));

    let surface_format = surface.get_preferred_format(&adapter).unwrap();
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: window_dimensions.width,
        height: window_dimensions.height,
        // FIFO, will cap the display rate at the displays framerate. This is essentially VSync.
        // https://docs.rs/wgpu/0.10.1/wgpu/enum.PresentMode.html
        present_mode: wgpu::PresentMode::Mailbox,
    };
    surface.configure(&device, &config);

    let gui = Gui::new(ScreenDescriptor {
        physical_width: window_dimensions.width,
        physical_height: window_dimensions.height,
        scale_factor: window.scale_factor() as f32,
    });

    let gui_render = GuiRenderWgpu::new(&device, config.format, 1);

    let loop_helper = LoopHelper::builder().report_interval_s(1.0).build_with_target_rate(60);

    let mut app_state = AppState {
        window,

        surface,
        device,
        queue,
        config,
        clear_color: wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 },

        gui,
        gui_render,

        input_state: InputsState::default(),
        system_state: SystemState::new(window_dimensions),

        loop_helper,
    };

    let mut app = T::create(&mut app_state);

    // Run
    event_loop.run(move |event, _, control_flow| {
        if let Err(error) = run_loop(&mut app, &mut app_state, event, control_flow) {
            eprintln!("Application Error: {}", error);
        }
    });
}

fn run_loop(app: &mut impl App, app_state: &mut AppState, event: Event<()>, control_flow: &mut ControlFlow) -> Result<()> {
    *control_flow = ControlFlow::Poll;

    app_state.input_state.handle_event(&event);
    app_state.system_state.handle_event(&event);
    app_state.gui.handle_event(&event);
    app.handle_event(app_state, &event)?;

    match event {
        Event::WindowEvent { ref event, .. } => match event {
            WindowEvent::Resized(physical_size) => {
                // Resize with 0 width and height is used by winit to signal a minimize event on Windows.
                // See: https://github.com/rust-windowing/winit/issues/208
                // This solves an issue where the app would panic when minimizing on Windows.
                app_state.config.width = physical_size.width;
                app_state.config.height = physical_size.height;
                if physical_size.width > 0 && physical_size.height > 0 {
                    app_state.surface.configure(&app_state.device, &app_state.config);
                }
            },
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            WindowEvent::MouseInput { button, state, .. } => app.on_mouse(app_state, button, state)?,
            WindowEvent::KeyboardInput { input, .. } => {
                app.on_key(app_state, *input)?;
            },
            _ => (),
        },
        Event::RedrawRequested(_) => {
            // TODO move that
            // TODO: fix render method here by calling sub app render features
            let full_output = {
                let _frame_data = app_state.gui.start_frame(app_state.window.scale_factor() as _);
                app.render_gui(&app_state.gui.context())?;
                app_state.gui.end_frame(&mut app_state.window)
            };

            match render_app(app, app_state, full_output) {
                Ok(_) => {},
                // TODO: Reconfigure the surface if lost
                // Err(wgpu::SurfaceError::Lost) => { }
                // The system is out of memory, we should probably quit
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                // All other errors (Outdated, Timeout) should be resolved by the next frame
                Err(e) => eprintln!("{:?}", e),
            }
        },
        Event::MainEventsCleared => {
            app.update(app_state)?;

            // RedrawRequested will only trigger once, unless we manually
            // request it.
            // window.request_redraw();
            app_state.window.request_redraw();

            if let Some(_fps) = app_state.loop_helper.report_rate() {
                // println!("fps report : {}", fps);
            }

            app_state.loop_helper.loop_sleep_no_spin(); // or `loop_sleep_no_spin()` to save battery
            app_state.loop_helper.loop_start();
        },
        Event::LoopDestroyed => {
            app.cleanup()?;
        },
        _ => (),
    }

    Ok(())
}

pub fn render_app(app: &mut impl App, app_state: &mut AppState, gui_output: egui::FullOutput) -> Result<(), wgpu::SurfaceError> {
    let output: wgpu::SurfaceTexture = app_state.surface.get_current_texture()?;
    let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder: wgpu::CommandEncoder = app_state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Render Encoder") });

    app.render(app_state, &mut encoder, &view)?;

    // draw UI
    encoder.insert_debug_marker("Render GUI");

    let window_dimensions = app_state.window.inner_size();

    let screen_descriptor = ScreenDescriptor {
        physical_width: window_dimensions.width,
        physical_height: window_dimensions.height,
        scale_factor: app_state.window.scale_factor() as f32,
    };

    app_state
        .gui_render
        .render(
            app_state.gui.context(),
            &app_state.device,
            &app_state.queue,
            &screen_descriptor,
            &mut encoder,
            &view,
            gui_output,
        )
        .expect("Failed to execute gui render pass!");

    // submit will accept anything that implements IntoIter
    app_state.queue.submit(iter::once(encoder.finish()));
    output.present();

    Ok(())
}
