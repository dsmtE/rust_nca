mod nca_app;

mod utils;
mod simulation_data;

use nca_app::NcaApp;
fn main() {

    skeleton_app::run_application::<NcaApp>(skeleton_app::AppConfig {
        is_resizable: true,
        title: "rust NCA".to_owned(),
        icon: None,
    }).unwrap();
}