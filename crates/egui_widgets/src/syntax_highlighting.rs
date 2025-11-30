use egui::text::LayoutJob;

impl egui::util::cache::ComputerMut<(&CodeTheme, &str, &str), LayoutJob> for Highlighter {
    fn compute(&mut self, (theme, code, lang): (&CodeTheme, &str, &str)) -> LayoutJob { self.highlight(theme, code, lang) }
}

/// Memoized Code highlighting
pub fn highlight(ctx: &egui::Context, theme: &CodeTheme, code: &str, language: &str) -> LayoutJob {
    type HighlightCache<'a> = egui::util::cache::FrameCache<LayoutJob, Highlighter>;

    ctx.memory_mut(|memory| {
        let highlight_cache = memory.caches.cache::<HighlightCache<'_>>();
        highlight_cache.get((theme, code, language))
    })
}

// ----------------------------------------------------------------------------

#[cfg(not(feature = "syntax_highlighting"))]
#[derive(Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(enum_map::Enum)]
enum TokenType {
    Comment,
    Keyword,
    Literal,
    StringLiteral,
    Punctuation,
    Whitespace,
}

#[cfg(feature = "syntax_highlighting")]
#[derive(Clone, Copy, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
enum SyntectTheme {
    Base16EightiesDark,
    Base16MochaDark,
    Base16OceanDark,
    Base16OceanLight,
    InspiredGitHub,
    SolarizedDark,
    SolarizedLight,
}

#[cfg(feature = "syntax_highlighting")]
impl SyntectTheme {
    fn all() -> impl ExactSizeIterator<Item = Self> {
        [
            Self::Base16EightiesDark,
            Self::Base16MochaDark,
            Self::Base16OceanDark,
            Self::Base16OceanLight,
            Self::InspiredGitHub,
            Self::SolarizedDark,
            Self::SolarizedLight,
        ]
        .iter()
        .copied()
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Base16EightiesDark => "Base16 Eighties (dark)",
            Self::Base16MochaDark => "Base16 Mocha (dark)",
            Self::Base16OceanDark => "Base16 Ocean (dark)",
            Self::Base16OceanLight => "Base16 Ocean (light)",
            Self::InspiredGitHub => "InspiredGitHub (light)",
            Self::SolarizedDark => "Solarized (dark)",
            Self::SolarizedLight => "Solarized (light)",
        }
    }

    fn syntect_key_name(&self) -> &'static str {
        match self {
            Self::Base16EightiesDark => "base16-eighties.dark",
            Self::Base16MochaDark => "base16-mocha.dark",
            Self::Base16OceanDark => "base16-ocean.dark",
            Self::Base16OceanLight => "base16-ocean.light",
            Self::InspiredGitHub => "InspiredGitHub",
            Self::SolarizedDark => "Solarized (dark)",
            Self::SolarizedLight => "Solarized (light)",
        }
    }

    pub fn is_dark(&self) -> bool {
        match self {
            Self::Base16EightiesDark | Self::Base16MochaDark | Self::Base16OceanDark | Self::SolarizedDark => true,
            Self::Base16OceanLight | Self::InspiredGitHub | Self::SolarizedLight => false,
        }
    }
}

#[derive(Clone, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct CodeTheme {
    #[cfg(feature = "syntax_highlighting")]
    syntect_theme: SyntectTheme,

    #[cfg(not(feature = "syntax_highlighting"))]
    formats: enum_map::EnumMap<TokenType, egui::TextFormat>,
}

impl CodeTheme {
    fn default(dark_mode: bool) -> Self { CodeTheme{
        syntect_theme: if dark_mode { SyntectTheme::Base16MochaDark } else { SyntectTheme::SolarizedLight } 
    }}

    fn internal_id(dark_mode: bool) -> egui::Id {
        egui::Id::new(format!("code_theme_{:}", if dark_mode { "dark" } else { "light" }))
    }

    pub fn from_memory(ctx: &egui::Context) -> Self {
        let dark_mode = ctx.style().visuals.dark_mode;
        ctx.data_mut(|data| {
            data.get_persisted(Self::internal_id(dark_mode)).unwrap_or(Self::default(dark_mode))
        })
    }

    pub fn store_in_memory(self, ctx: &egui::Context) {
        let dark_mode = ctx.style().visuals.dark_mode;
        ctx.data_mut(|data| {
            data.insert_persisted(Self::internal_id(dark_mode), self);
        });
    }
}

#[cfg(feature = "syntax_highlighting")]
impl CodeTheme {
    pub fn ui(&mut self, label: &str, ui: &mut egui::Ui) -> Option<egui::Response> {
        let dark_mode = ui.ctx().style().visuals.dark_mode;

        ui.menu_button(label, |ui| {
            SyntectTheme::all()
            .filter(|theme| theme.is_dark() == dark_mode)
            .map(|theme| {
                ui.selectable_value(&mut self.syntect_theme, theme, theme.name())
            })
            .reduce(|acc, e| acc.union(e)).unwrap()
        }).inner

        // egui::ComboBox::from_id_source("CodeTheme")
        // .selected_text(self.syntect_theme.name())
        // .show_ui(ui, |ui| {
        //     SyntectTheme::all()
        //     .filter(|theme| theme.is_dark() == dark_mode)
        //     .map(|theme| {
        //         ui.selectable_value(&mut self.syntect_theme, theme, theme.name())
        //     })
        //     .reduce(|acc, e| acc.union(e)).unwrap()
        // }).inner
    }
}

#[cfg(not(feature = "syntax_highlighting"))]
impl CodeTheme {
    fn default(dark_mode: bool) -> Self { CodeTheme{
        syntect_theme: if dark_mode { self::dark() } else { self::light() } 
    }}

    pub fn dark() -> Self {
        let font_id = egui::FontId::monospace(12.0);
        use egui::{Color32, TextFormat};
        Self {
            formats: enum_map::enum_map![
                TokenType::Comment => TextFormat::simple(font_id.clone(), Color32::from_gray(120)),
                TokenType::Keyword => TextFormat::simple(font_id.clone(), Color32::from_rgb(255, 100, 100)),
                TokenType::Literal => TextFormat::simple(font_id.clone(), Color32::from_rgb(87, 165, 171)),
                TokenType::StringLiteral => TextFormat::simple(font_id.clone(), Color32::from_rgb(109, 147, 226)),
                TokenType::Punctuation => TextFormat::simple(font_id.clone(), Color32::LIGHT_GRAY),
                TokenType::Whitespace => TextFormat::simple(font_id.clone(), Color32::TRANSPARENT),
            ],
        }
    }

    pub fn light() -> Self {
        let font_id = egui::FontId::monospace(12.0);
        use egui::{Color32, TextFormat};
        Self {
            dark_mode: false,
            #[cfg(not(feature = "syntax_highlighting"))]
            formats: enum_map::enum_map![
                TokenType::Comment => TextFormat::simple(font_id.clone(), Color32::GRAY),
                TokenType::Keyword => TextFormat::simple(font_id.clone(), Color32::from_rgb(235, 0, 0)),
                TokenType::Literal => TextFormat::simple(font_id.clone(), Color32::from_rgb(153, 134, 255)),
                TokenType::StringLiteral => TextFormat::simple(font_id.clone(), Color32::from_rgb(37, 203, 105)),
                TokenType::Punctuation => TextFormat::simple(font_id.clone(), Color32::DARK_GRAY),
                TokenType::Whitespace => TextFormat::simple(font_id.clone(), Color32::TRANSPARENT),
            ],
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_top(|ui| {
            let selected_id = egui::Id::null();
            
            let mut token_type : TokenType = ui.data_mut(|map| *map.get_temp_mut_or(selected_id, TokenType::Comment));

            ui.vertical(|ui| {
                ui.set_width(150.0);
                egui::widgets::global_dark_light_mode_buttons(ui);

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ui.scope(|ui| {
                    for (tt, tt_name) in [
                        (TokenType::Comment, "// comment"),
                        (TokenType::Keyword, "keyword"),
                        (TokenType::Literal, "literal"),
                        (TokenType::StringLiteral, "\"string literal\""),
                        (TokenType::Punctuation, "punctuation ;"),
                        // (TokenType::Whitespace, "whitespace"),
                    ] {
                        let format = &mut self.formats[tt];
                        ui.style_mut().override_font_id = Some(format.font_id.clone());
                        ui.visuals_mut().override_text_color = Some(format.color);
                        ui.radio_value(token_type, tt, tt_name);
                    }
                });

                let reset_value = if self.dark_mode {
                    CodeTheme::dark()
                } else {
                    CodeTheme::light()
                };

                if ui.add_enabled(*self != reset_value, egui::Button::new("Reset theme")).clicked() {
                    *self = reset_value;
                }
            });

            ui.add_space(16.0);

            ui.data_mut(|map| map.insert_temp(selected_id, token_type));

            egui::Frame::group(ui.style()).inner_margin(egui::Vec2::splat(2.0)).show(ui, |ui| {
                // ui.group(|ui| {
                ui.style_mut().override_text_style = Some(egui::TextStyle::Small);
                ui.spacing_mut().slider_width = 128.0; // Controls color picker size
                egui::widgets::color_picker::color_picker_color32(ui, &mut self.formats[*selected_tt].color, egui::color_picker::Alpha::Opaque);
            });
        });
    }
}

// ----------------------------------------------------------------------------

#[cfg(feature = "syntax_highlighting")]
struct Highlighter {
    ps: syntect::parsing::SyntaxSet,
    ts: syntect::highlighting::ThemeSet,
}

#[cfg(feature = "syntax_highlighting")]
impl Default for Highlighter {
    fn default() -> Self {
        Self {
            ps: syntect::parsing::SyntaxSet::load_defaults_newlines(),
            ts: syntect::highlighting::ThemeSet::load_defaults(),
        }
    }
}

#[cfg(feature = "syntax_highlighting")]
impl Highlighter {
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn highlight(&self, theme: &CodeTheme, code: &str, lang: &str) -> LayoutJob {
        self.highlight_impl(theme, code, lang).unwrap_or_else(|| {
            // Fallback:
            LayoutJob::simple(
                code.into(),
                egui::FontId::monospace(14.0),
                egui::Color32::LIGHT_GRAY,
                f32::INFINITY,
            )
        })
    }

    fn highlight_impl(&self, theme: &CodeTheme, text: &str, language: &str) -> Option<LayoutJob> {
        use syntect::{easy::HighlightLines, highlighting::FontStyle, util::LinesWithEndings};

        let syntax = self
            .ps
            .find_syntax_by_name(language)
            .or_else(|| self.ps.find_syntax_by_extension(language))?;

        let theme = theme.syntect_theme.syntect_key_name();
        let mut h = HighlightLines::new(syntax, &self.ts.themes[theme]);

        use egui::text::{LayoutSection, TextFormat};

        let mut job = LayoutJob { text: text.into(), ..Default::default() };

        for line in LinesWithEndings::from(text) {
            for (style, range) in h.highlight_line(line, &self.ps).unwrap() {
                let fg = style.foreground;
                let text_color = egui::Color32::from_rgb(fg.r, fg.g, fg.b);
                let italics = style.font_style.contains(FontStyle::ITALIC);
                let underline = style.font_style.contains(FontStyle::ITALIC);
                let underline = if underline {
                    egui::Stroke::new(1.0, text_color)
                } else {
                    egui::Stroke::NONE
                };
                job.sections.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: as_byte_range(text, range),
                    format: TextFormat {
                        font_id: egui::FontId::monospace(14.0),
                        color: text_color,
                        italics,
                        underline,
                        ..Default::default()
                    },
                });
            }
        }

        Some(job)
    }
}

#[cfg(feature = "syntax_highlighting")]
fn as_byte_range(whole: &str, range: &str) -> std::ops::Range<usize> {
    let whole_start = whole.as_ptr() as usize;
    let range_start = range.as_ptr() as usize;
    assert!(whole_start <= range_start);
    assert!(range_start + range.len() <= whole_start + whole.len());
    let offset = range_start - whole_start;
    offset..(offset + range.len())
}

// ----------------------------------------------------------------------------

#[cfg(not(feature = "syntax_highlighting"))]
#[derive(Default)]
struct Highlighter {}

#[cfg(not(feature = "syntax_highlighting"))]
impl Highlighter {
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn highlight(&self, theme: &CodeTheme, mut text: &str, _language: &str) -> LayoutJob {
        // Extremely simple syntax highlighter for when we compile without syntect

        let mut job = LayoutJob::default();

        while !text.is_empty() {
            if text.starts_with("//") {
                let end = text.find('\n').unwrap_or(text.len());
                job.append(&text[..end], 0.0, theme.formats[TokenType::Comment].clone());
                text = &text[end..];
            } else if text.starts_with('"') {
                let end = text[1..].find('"').map(|i| i + 2).or_else(|| text.find('\n')).unwrap_or(text.len());
                job.append(&text[..end], 0.0, theme.formats[TokenType::StringLiteral].clone());
                text = &text[end..];
            } else if text.starts_with(|c: char| c.is_ascii_alphanumeric()) {
                let end = text[1..].find(|c: char| !c.is_ascii_alphanumeric()).map_or_else(|| text.len(), |i| i + 1);
                let word = &text[..end];
                let tt = if is_keyword(word) {
                    TokenType::Keyword
                } else {
                    TokenType::Literal
                };
                job.append(word, 0.0, theme.formats[tt].clone());
                text = &text[end..];
            } else if text.starts_with(|c: char| c.is_ascii_whitespace()) {
                let end = text[1..].find(|c: char| !c.is_ascii_whitespace()).map_or_else(|| text.len(), |i| i + 1);
                job.append(&text[..end], 0.0, theme.formats[TokenType::Whitespace].clone());
                text = &text[end..];
            } else {
                let mut it = text.char_indices();
                it.next();
                let end = it.next().map_or(text.len(), |(idx, _chr)| idx);
                job.append(&text[..end], 0.0, theme.formats[TokenType::Punctuation].clone());
                text = &text[end..];
            }
        }

        job
    }
}

#[cfg(not(feature = "syntax_highlighting"))]
fn is_keyword(word: &str) -> bool {
    matches!(
        word,
        "as" | "async"
            | "await"
            | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
    )
}
