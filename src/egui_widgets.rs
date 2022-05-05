
/// Something to view
pub trait UiWidget {
    fn show(&mut self, ui: &mut egui::Ui);
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct CodeEditor<'a> {
    language: &'a mut String,
    code: &'a mut String,
    height_row: usize,
}

impl<'a> CodeEditor<'a> {
    pub fn new(language: &'a mut String, code: &'a mut String) -> Self {
        Self {
            language,
            code,
            height_row: 10,
        }
    }
}

impl<'a> UiWidget for CodeEditor<'a> {
    fn show(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.set_height(0.0);
            ui.label("An example of syntax highlighting in a TextEdit.");
        });

        if cfg!(feature = "syntect") {
            ui.horizontal(|ui| {
                ui.label("Language:");
                ui.text_edit_singleline(self.language);
            });
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("Syntax highlighting powered by ");
                ui.hyperlink_to("syntect", "https://github.com/trishume/syntect");
                ui.label(".");
            });
        } else {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.label("Compile the demo with the ");
                ui.code("syntax_highlighting");
                ui.label(" feature to enable more accurate syntax highlighting using ");
                ui.hyperlink_to("syntect", "https://github.com/trishume/syntect");
                ui.label(".");
            });
        }

        let mut theme = crate::syntax_highlighting::CodeTheme::from_memory(ui.ctx());
        ui.collapsing("Theme", |ui| {
            ui.group(|ui| {
                theme.ui(ui);
                theme.clone().store_in_memory(ui.ctx());
            });
        });

        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job = crate::syntax_highlighting::highlight(ui.ctx(), &theme, string, self.language);
            layout_job.wrap_width = wrap_width;
            ui.fonts().layout_job(layout_job)
        };

        let font = egui::TextStyle::Monospace.resolve(ui.style());
        let height = ui.fonts().row_height(&font)*((self.height_row+1) as f32);
        
        egui::ScrollArea::vertical().max_height(height).show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(self.code)
                    .font(font) // for cursor height
                    .code_editor()
                    .desired_rows(self.height_row)
                    .lock_focus(true)
                    .desired_width(f32::INFINITY)
                    .layouter(&mut layouter),
            );
        });
    }
}