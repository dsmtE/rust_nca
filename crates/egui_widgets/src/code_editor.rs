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

impl<'a> crate::UiWidget for CodeEditor<'a> {
    fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let theme = crate::syntax_highlighting::CodeTheme::from_memory(ui.ctx());
        let mut layouter = |ui: &egui::Ui, text_buffer: &dyn egui::TextBuffer, wrap_width: f32| {
            let mut layout_job = crate::syntax_highlighting::highlight(ui.ctx(), &theme, text_buffer.as_str(), self.language);
            layout_job.wrap.max_width = wrap_width;
            ui.fonts_mut(|f| f.layout_job(layout_job))
        };

        let font = egui::TextStyle::Monospace.resolve(ui.style());
        let height = ui.fonts_mut(|f| {f.row_height(&font) }) * ((self.height_row + 1) as f32);

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

impl<'a> CodeEditor<'a> {
    pub fn show_theme_selector(&mut self, label: &str, ui: &mut egui::Ui) -> Option<egui::Response> {
        let mut theme = crate::syntax_highlighting::CodeTheme::from_memory(ui.ctx());
        let response = theme.ui(label, ui);
        theme.clone().store_in_memory(ui.ctx());
        response
    }
}
