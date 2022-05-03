mod nca_app;
use nca_app::NcaApp;

fn main() {

    skeleton_app::run_application::<NcaApp>(skeleton_app::AppConfig {
        is_resizable: true,
        title: "rust NCA".to_owned(),
        icon: None,
    });
}