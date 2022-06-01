mod nca_app;

mod egui_widgets;
mod syntax_highlighting;
mod utils;

use nca_app::NcaApp;
fn main() {
    skeleton_app::run_application::<NcaApp>(skeleton_app::AppConfig {
        is_resizable: true,
        title: "rust NCA".to_owned(),
        icon: None,
    })
    .unwrap();
}
