use crevice::std140::AsStd140;
use nalgebra_glm as glm;
use serde::{Deserialize, Serialize};

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
            let mut layout_job = crate::syntax_highlighting::highlight(ui.ctx(), &theme, string, self.language);
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

// https://iquilezles.org/articles/palettes/
#[repr(C)]
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, AsStd140)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct IqGradient {
    a: glm::Vec3,
    b: glm::Vec3,
    c: glm::Vec3,
    d: glm::Vec3,
}

impl Default for IqGradient {
    fn default() -> Self {
        IqGradient::grey()
    }
}

impl IqGradient {
    pub fn evalue(&self, t: f32) -> glm::Vec3 {
        let angle: glm::Vec3 = std::f32::consts::TAU * (self.c * t + self.d);
        let cos: glm::Vec3 = glm::Vec3::from_iterator(angle.as_slice().into_iter().map(|x| x.cos()).into_iter());
        self.a + self.b.component_mul(&cos)
    }

    fn grey() -> Self {
        Self {
            a: glm::vec3(0.63, 0.63, 0.63),
            b: glm::vec3(1.0, 1.0, 1.0),
            c: glm::vec3(0.172, 0.172, 0.172),
            d: glm::vec3(0.641, 0.641, 0.641),
        }
    }
}

impl IqGradient {
    pub fn ui_control(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed: bool = false;
        ui.collapsing("gradient settings", |ui| {
            let slicde_err_msg = "slice with incorrect length";
            ui.label("color(t) = a + b.cos(2π(c.t+d))");
            ui.hyperlink_to("read more about this", "https://iquilezles.org/articles/palettes/");
            egui::Grid::new("gradient settings").show(ui, |ui| {
                ui.label("a:");
                changed |= egui::color_picker::color_edit_button_rgb(ui, self.a.as_mut_slice().try_into().expect(slicde_err_msg)).changed();
                ui.label("b:");
                changed |= egui::color_picker::color_edit_button_rgb(ui, self.b.as_mut_slice().try_into().expect(slicde_err_msg)).changed();
                ui.end_row();

                ui.label("c:");
                changed |= egui::color_picker::color_edit_button_rgb(ui, self.c.as_mut_slice().try_into().expect(slicde_err_msg)).changed();
                ui.label("d:");
                changed |= egui::color_picker::color_edit_button_rgb(ui, self.d.as_mut_slice().try_into().expect(slicde_err_msg)).changed();
                ui.end_row();
            });
        });
        changed
    }
}

const N: u32 = 6 * 6;
impl UiWidget for IqGradient {
    fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let desired_size = egui::vec2(ui.spacing().slider_width * 2.0, ui.spacing().interact_size.y * 2.0);
        let (rect, response) = ui.allocate_at_least(desired_size, egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);

            {
                let mut mesh = egui::Mesh::default();
                for i in 0..=N {
                    let t = i as f32 / (N as f32);
                    let color: glm::Vec3 = self.evalue(t);
                    let color32 = egui::color::Color32::from_rgb(
                        egui::color::gamma_u8_from_linear_f32(color[0]),
                        egui::color::gamma_u8_from_linear_f32(color[1]),
                        egui::color::gamma_u8_from_linear_f32(color[2]),
                    );
                    let x = egui::lerp(rect.left()..=rect.right(), t);
                    mesh.colored_vertex(egui::pos2(x, rect.top()), color32);
                    mesh.colored_vertex(egui::pos2(x, rect.bottom()), color32);
                    if i < N {
                        mesh.add_triangle(2 * i + 0, 2 * i + 1, 2 * i + 2);
                        mesh.add_triangle(2 * i + 1, 2 * i + 2, 2 * i + 3);
                    }
                }
                ui.painter().add(egui::Shape::mesh(mesh));
            }

            ui.painter().rect_stroke(rect, 0.0, visuals.bg_stroke); // outline
        }

        response.on_hover_text("computed from the formula:\ncolor(t) = a + b.cos(2π(c.t+d))")
    }
}
