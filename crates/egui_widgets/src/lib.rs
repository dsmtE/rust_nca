#[macro_use]
extern crate lazy_static;

mod syntax_highlighting;
mod iq_gradiant;
mod code_editor;
pub mod nalgebra_helpers;

/// Something to view
pub trait UiWidget {
    fn show(&mut self, ui: &mut egui::Ui) -> egui::Response;
}


pub use code_editor::CodeEditor;
pub use iq_gradiant::{IqGradient, IQ_GRADIENT_PRESETS};