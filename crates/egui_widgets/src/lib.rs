#[macro_use]
extern crate lazy_static;

mod code_editor;
mod iq_gradiant;
pub mod nalgebra_helpers;
mod syntax_highlighting;

/// Something to view
pub trait UiWidget {
    fn show(&mut self, ui: &mut egui::Ui) -> egui::Response;
}

pub use code_editor::CodeEditor;
pub use iq_gradiant::{IqGradient, IQ_GRADIENT_PRESETS};
