mod utils;
mod app;
mod gui_render_wgpu;
mod simulation_data;

mod nca_app;
use nca_app::NcaApp;

fn main() {

    app::run_application::<NcaApp>(app::AppConfig {
        is_resizable: true,
        title: "rust NCA".to_owned(),
        icon: None,
    });
}