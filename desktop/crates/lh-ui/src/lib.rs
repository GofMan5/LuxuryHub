//! Design tokens and primitives for the LuxuryHub desktop client.
//!
//! Everything visual is defined once, here: colours, spacing, radii and the few
//! elements that screens are built from. Screens only compose these helpers, so
//! changing the look of the product means changing this crate, not every view.

use std::path::Path;
use std::sync::{OnceLock, RwLock};

use gpui::{Div, FontWeight, ParentElement, Rgba, SharedString, Styled, div, px, rgb};

/// Colour palette of the dark theme.
///
/// The product is a premium marketplace, so the base is a near-black canvas with
/// warm off-white type and a muted gold accent; saturated colours are reserved
/// for state (success, warning, danger) and are not part of the skeleton yet.
#[derive(Clone, Copy, Debug)]
pub struct Palette {
    /// Window background.
    pub background: Rgba,
    /// Sidebar and panel background.
    pub surface: Rgba,
    /// Raised card background.
    pub surface_raised: Rgba,
    /// Hairline borders and separators.
    pub border: Rgba,
    /// Primary text.
    pub text: Rgba,
    /// Secondary text and hints.
    pub text_muted: Rgba,
    /// Accent used for selection and calls to action.
    pub accent: Rgba,
    /// Accent with low contrast, for badges and highlights.
    pub accent_soft: Rgba,
}

impl Palette {
    /// Dark theme used by the client.
    pub fn dark() -> Self {
        Self {
            background: rgb(0x0E0F12),
            surface: rgb(0x13151A),
            surface_raised: rgb(0x1A1D24),
            border: rgb(0x262A33),
            text: rgb(0xE9E4DA),
            text_muted: rgb(0x9A958C),
            accent: rgb(0xC8A24A),
            accent_soft: rgb(0x2A2415),
        }
    }
}

/// Spacing scale in logical pixels: 4 / 8 / 12 / 16 / 24 / 32 / 48.
pub mod space {
    /// 4 px.
    pub const XS: f32 = 4.0;
    /// 8 px.
    pub const SM: f32 = 8.0;
    /// 12 px.
    pub const MD: f32 = 12.0;
    /// 16 px.
    pub const LG: f32 = 16.0;
    /// 24 px.
    pub const XL: f32 = 24.0;
    /// 32 px.
    pub const XXL: f32 = 32.0;
    /// 48 px.
    pub const XXXL: f32 = 48.0;
}

/// Width of the navigation column.
pub const SIDEBAR_WIDTH: f32 = 248.0;

/// Palette that rendering currently uses.
///
/// It lives in a lock rather than in a constant because the client can swap it
/// while running: that is what makes a live theme file possible.
static ACTIVE: OnceLock<RwLock<Palette>> = OnceLock::new();

fn active() -> &'static RwLock<Palette> {
    ACTIVE.get_or_init(|| RwLock::new(Palette::dark()))
}

/// Current palette; a single place to switch to a light theme later.
pub fn palette() -> Palette {
    match active().read() {
        Ok(guard) => *guard,
        Err(poisoned) => *poisoned.into_inner(),
    }
}

/// Swaps the palette in place. The next frame renders with it, so the caller
/// only has to repaint its windows.
pub fn set_palette(palette: Palette) {
    match active().write() {
        Ok(mut guard) => *guard = palette,
        Err(poisoned) => *poisoned.into_inner() = palette,
    }
}

/// Outcome of reading a theme file.
#[derive(Clone, Debug)]
pub struct LoadedTheme {
    /// Palette to install: the built-in dark theme plus the file's overrides.
    pub palette: Palette,
    /// Problems that did not stop the load, one message per offending line.
    pub warnings: Vec<String>,
}

/// Reads `key = #RRGGBB` lines on top of [`Palette::dark`].
///
/// Unknown keys and unparsable colours do not fail the load: they are reported
/// through [`LoadedTheme::warnings`], so a half-typed value keeps the client
/// usable instead of blanking the window. A line starting with `#` is a comment;
/// the `#` inside a colour is not.
pub fn load_palette(path: &Path) -> std::io::Result<LoadedTheme> {
    let text = std::fs::read_to_string(path)?;
    let mut palette = Palette::dark();
    let mut warnings = Vec::new();

    for (index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            warnings.push(format!("строка {}: нет знака «=»", index + 1));
            continue;
        };

        let key = key.trim();
        let raw = value.split_whitespace().next().unwrap_or_default();
        let Some(colour) = parse_colour(raw) else {
            warnings.push(format!(
                "строка {}: «{raw}» — не цвет вида #RRGGBB",
                index + 1
            ));
            continue;
        };

        let target = match key {
            "background" => &mut palette.background,
            "surface" => &mut palette.surface,
            "surface_raised" => &mut palette.surface_raised,
            "border" => &mut palette.border,
            "text" => &mut palette.text,
            "text_muted" => &mut palette.text_muted,
            "accent" => &mut palette.accent,
            "accent_soft" => &mut palette.accent_soft,
            other => {
                warnings.push(format!("строка {}: неизвестный ключ «{other}»", index + 1));
                continue;
            }
        };

        *target = colour;
    }

    Ok(LoadedTheme { palette, warnings })
}

fn parse_colour(raw: &str) -> Option<Rgba> {
    let hex = raw.trim_start_matches('#');
    if hex.len() != 6 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let value = u32::from_str_radix(hex, 16).ok()?;
    Some(rgb(value))
}

/// Small uppercase label above a heading.
pub fn eyebrow(text: impl Into<SharedString>) -> Div {
    let p = palette();
    div()
        .text_sm()
        .font_weight(FontWeight::BOLD)
        .text_color(p.accent)
        .child(text.into())
}

/// Screen heading.
pub fn heading(text: impl Into<SharedString>) -> Div {
    let p = palette();
    div()
        .text_2xl()
        .font_weight(FontWeight::BOLD)
        .text_color(p.text)
        .child(text.into())
}

/// Secondary text: captions, hints, empty-state explanations.
pub fn muted(text: impl Into<SharedString>) -> Div {
    let p = palette();
    div().text_sm().text_color(p.text_muted).child(text.into())
}

/// Raised container used for cards and panels.
pub fn card() -> Div {
    let p = palette();
    div()
        .flex()
        .flex_col()
        .gap_3()
        .p_6()
        .rounded_lg()
        .bg(p.surface_raised)
        .border_1()
        .border_color(p.border)
        .shadow_sm()
}

/// Small pill with the accent colour, for statuses and counts.
pub fn badge(text: impl Into<SharedString>) -> Div {
    let p = palette();
    div()
        .px_3()
        .py_1()
        .rounded_md()
        .bg(p.accent_soft)
        .text_sm()
        .text_color(p.accent)
        .child(text.into())
}

/// Navigation entry. The active one is highlighted with a soft accent background.
pub fn nav_item(label: impl Into<SharedString>, active: bool) -> Div {
    let p = palette();
    let base = div()
        .flex()
        .items_center()
        .gap_2()
        .px_3()
        .py_2()
        .rounded_md()
        .text_base();

    if active {
        base.bg(p.accent_soft)
            .text_color(p.accent)
            .child(label.into())
    } else {
        base.text_color(p.text_muted).child(label.into())
    }
}

/// Thin horizontal separator.
pub fn separator() -> Div {
    let p = palette();
    div().h(px(1.0)).w_full().bg(p.border)
}
