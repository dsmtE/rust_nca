mod nca_app;

#[macro_use]
extern crate lazy_static;

use nca_app::NcaApp;
fn main() {
    oxyde::run_application::<NcaApp>(oxyde::AppConfig {
        is_resizable: true,
        title: "rust NCA".to_owned(),
        icon: None,
    })
    .unwrap();
}
