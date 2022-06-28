use anyhow::{Context, Result};
use egui_wgpu_backend::RenderPass;

use egui_winit_platform::{Platform, PlatformDescriptor};
use std::{sync::Arc, time::Instant};
use wgpu::CommandEncoder;
use winit::{event::Event, window::Window};

pub use egui_wgpu_backend::ScreenDescriptor;

// We repaint the UI every frame, so no custom repaint signal is needed
struct RepaintSignal;
impl epi::backend::RepaintSignal for RepaintSignal {
    fn request_repaint(&self) {}
}

pub struct Gui {
    platform: Platform,
    repaint_signal: Arc<RepaintSignal>,
    start_time: Instant,
    last_frame_start: Instant,
    previous_frame_time: Option<f32>,
}

impl Gui {
    pub fn new(screen_descriptor: ScreenDescriptor) -> Self {
        // We use the egui_winit_platform crate as the platform.
        let platform = Platform::new(PlatformDescriptor {
            physical_width: screen_descriptor.physical_width,
            physical_height: screen_descriptor.physical_height,
            scale_factor: screen_descriptor.scale_factor as f64,
            font_definitions: epi::egui::FontDefinitions::default(),
            style: Default::default(),
        });

        Self {
            platform,
            repaint_signal: std::sync::Arc::new(RepaintSignal {}),
            start_time: Instant::now(),
            previous_frame_time: None,
            last_frame_start: Instant::now(),
        }
    }

    pub fn handle_event(&mut self, event: &Event<()>) { self.platform.handle_event(&event); }

    pub fn context(&self) -> epi::egui::Context { self.platform.context() }

    pub fn start_frame<'a>(&mut self, scale_factor: f32) -> epi::backend::FrameData {
        self.platform.update_time(self.start_time.elapsed().as_secs_f64());

        // Begin to draw the UI frame.
        self.last_frame_start = Instant::now();
        self.platform.begin_frame();
        let app_output = epi::backend::AppOutput::default();

        epi::backend::FrameData {
            info: epi::IntegrationInfo {
                name: "egui_frame",
                web_info: None,
                cpu_usage: self.previous_frame_time,
                native_pixels_per_point: Some(scale_factor),
                prefer_dark_mode: None,
            },
            output: app_output,
            repaint_signal: self.repaint_signal.clone(),
        }
    }

    pub fn end_frame(&mut self, window: &Window) -> egui::FullOutput {
        let frame_time = self.last_frame_start.elapsed().as_secs_f32();
        self.previous_frame_time = Some(frame_time);

        self.platform.end_frame(Some(&window))
    }
}

pub struct GuiRenderWgpu {
    pub renderpass: RenderPass,
}

impl GuiRenderWgpu {
    pub fn new(device: &wgpu::Device, output_format: wgpu::TextureFormat, msaa_samples: u32) -> Self {
        Self {
            renderpass: RenderPass::new(device, output_format, msaa_samples),
        }
    }

    pub fn render(
        &mut self,
        context: epi::egui::Context,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_descriptor: &ScreenDescriptor,
        encoder: &mut CommandEncoder,
        output_view: &wgpu::TextureView,
        gui_output: egui::FullOutput,
    ) -> Result<()> {
        // TODO: how handle not repaint ui if isn't needed
        // if gui_output.needs_repaint {

        self.renderpass.add_textures(&device, &queue, &gui_output.textures_delta)?;

        let paint_jobs = context.tessellate(gui_output.shapes);

        self.renderpass.update_buffers(&device, &queue, &paint_jobs, &screen_descriptor);

        self.renderpass
            .execute(encoder, &output_view, &paint_jobs, &screen_descriptor, None)
            .context("Failed to execute egui renderpass!")?;

        // Remove unused textures
        self.renderpass.remove_textures(gui_output.textures_delta).unwrap();

        Ok(())
    }
}
