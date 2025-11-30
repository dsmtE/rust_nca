mod nca_app;

#[macro_use]
extern crate lazy_static;

use nca_app::NcaApp;

use oxyde::app::{run_application, AppConfig, RenderingConfig};
fn main() {
    run_application::<NcaApp>(AppConfig {
        is_resizable: true,
        title: "rust NCA",
        control_flow: oxyde::winit::event_loop::ControlFlow::Wait,
    },
    RenderingConfig::default())
    .unwrap();
}
