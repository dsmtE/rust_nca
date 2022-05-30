/// Something to view
pub trait UiWidget {
    fn show(&mut self, ui: &mut egui::Ui) -> egui::Response;
}

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct CodeEditor<'a> {
    code: &'a mut String,
    language: &'static str,
    height_row: usize,
}

impl<'a> CodeEditor<'a> {
    pub fn new(code: &'a mut String, language: &'static str, height_row: Option<usize>) -> Self {
        Self {
            code,
            language,
            height_row: height_row.unwrap_or(10),
        }
    }
}

impl<'a> UiWidget for CodeEditor<'a> {
    fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let mut theme = crate::syntax_highlighting::CodeTheme::from_memory(ui.ctx());
        ui.collapsing("Theme", |ui| {
            ui.group(|ui| {
                theme.ui(ui);
                theme.clone().store_in_memory(ui.ctx());
            });
        });

        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job =
                crate::syntax_highlighting::highlight(ui.ctx(), &theme, string, self.language);
            layout_job.wrap_width = wrap_width;
            ui.fonts().layout_job(layout_job)
        };

        let font = egui::TextStyle::Monospace.resolve(ui.style());
        let height = ui.fonts().row_height(&font) * ((self.height_row + 1) as f32);

        egui::ScrollArea::vertical()
            .max_height(height)
            .show(ui, |ui| -> egui::Response {
                ui.add(
                    egui::TextEdit::multiline(self.code)
                    .font(font) // for cursor height
                    .lock_focus(true)
                    .desired_rows(self.height_row)
                    .desired_width(f32::INFINITY)
                    .layouter(&mut layouter),
                )
            })
            .inner
    }
}
