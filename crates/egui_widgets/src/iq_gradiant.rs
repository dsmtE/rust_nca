use crevice::std140::AsStd140;
use glam::Vec3;

// Vec3 to mut slice
fn vec3_as_mut_slice(v: &mut Vec3) -> &mut [f32; 3] {
    unsafe { &mut *(v as *mut Vec3 as *mut [f32; 3]) }
}

// https://iquilezles.org/articles/palettes/
#[repr(C)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, Debug, PartialEq, AsStd140)]
pub struct IqGradient {
    a: Vec3,
    b: Vec3,
    c: Vec3,
    d: Vec3,
}

impl Default for IqGradient {
    fn default() -> Self { IQ_GRADIENT_PRESETS["Grey"] }
}

impl IqGradient {
    pub fn evalue(&self, t: f32) -> Vec3 {
        let angle: Vec3 = std::f32::consts::TAU * (self.c * t + self.d);
        let cos: Vec3 = angle.map(|x| x.cos());
        self.a + self.b * cos
    }
}

impl IqGradient {
    pub fn ui_control(&mut self, ui: &mut egui::Ui) -> bool {
        let mut changed: bool = false;
        ui.collapsing("gradient settings", |ui| {
            ui.label("color(t) = a + b.cos(2π(c.t+d))");
            ui.hyperlink_to("read more about this", "https://iquilezles.org/articles/palettes/");
            egui::Grid::new("gradient settings").show(ui, |ui| {
                ui.label("a:");
                changed |= egui::color_picker::color_edit_button_rgb(ui, vec3_as_mut_slice(&mut self.a)).changed();
                ui.label("b:");
                changed |= egui::color_picker::color_edit_button_rgb(ui, vec3_as_mut_slice(&mut self.b)).changed();
                ui.end_row();

                ui.label("c:");
                changed |= egui::color_picker::color_edit_button_rgb(ui, vec3_as_mut_slice(&mut self.c)).changed();
                ui.label("d:");
                changed |= egui::color_picker::color_edit_button_rgb(ui, vec3_as_mut_slice(&mut self.d)).changed();
                ui.end_row();
            });
        });
        changed
    }
}

const N: u32 = 6 * 6;
impl crate::UiWidget for IqGradient {
    fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let desired_size = egui::vec2(ui.spacing().slider_width * 2.0, ui.spacing().interact_size.y * 2.0);
        let (rect, response) = ui.allocate_at_least(desired_size, egui::Sense::click());

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);

            {
                let mut mesh = egui::Mesh::default();
                for i in 0..=N {
                    let t = i as f32 / (N as f32);
                    let color: Vec3 = self.evalue(t);
                    let color32 = egui::Color32::from_rgb(
                        ecolor::gamma_u8_from_linear_f32(color[0]),
                        ecolor::gamma_u8_from_linear_f32(color[1]),
                        ecolor::gamma_u8_from_linear_f32(color[2]),
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

lazy_static! {
    pub static ref IQ_GRADIENT_PRESETS: std::collections::HashMap<&'static str, IqGradient> = std::collections::HashMap::from([
        (
            "Grey",
            IqGradient {
                a: Vec3::new(0.63, 0.63, 0.63),
                b: Vec3::new(1.0, 1.0, 1.0),
                c: Vec3::new(0.172, 0.172, 0.172),
                d: Vec3::new(0.641, 0.641, 0.641),
            }
        ),
        (
            "Colorful",
            IqGradient {
                a: Vec3::new(0.5, 0.5, 0.5),
                b: Vec3::new(0.5, 0.5, 0.5),
                c: Vec3::new(1.0, 1.0, 1.0),
                d: Vec3::new(0.0, 0.33, 0.67),
            }
        ),
        (
            "BlueAndSand",
            IqGradient {
                a: Vec3::new(0.091, 0.363, 0.406),
                b: Vec3::new(0.405, 0.242, 0.363),
                c: Vec3::new(0.314, 0.304, 0.243),
                d: Vec3::new(0.697, 0.707, 1.0),
            }
        ),
    ]);
}
