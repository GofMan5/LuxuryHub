//! Design tokens and primitives for the LuxuryHub desktop client.
//!
//! Everything visual is defined once, here: colours, spacing, radii and the few
//! elements that screens are built from. Screens only compose these helpers, so
//! changing the look of the product means changing this crate, not every view.

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

/// Current palette; a single place to switch to a light theme later.
pub fn palette() -> Palette {
    Palette::dark()
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
        base.bg(p.accent_soft).text_color(p.accent).child(label.into())
    } else {
        base.text_color(p.text_muted).child(label.into())
    }
}

/// Thin horizontal separator.
pub fn separator() -> Div {
    let p = palette();
    div().h(px(1.0)).w_full().bg(p.border)
}