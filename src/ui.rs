//! Drawing and text geometry shared by the existing screen adapters.
//! Skins, clipping, font metrics and input state remain caller-owned.

use macroquad::prelude::*;

/// Inset text bounds without producing negative dimensions for narrow controls.
pub fn inset(bounds: Rect, amount: f32) -> Rect {
    Rect::new(
        bounds.x + amount,
        bounds.y + amount,
        (bounds.w - 2.0 * amount).max(0.0),
        (bounds.h - 2.0 * amount).max(0.0),
    )
}

/// Records and main-button labels may extend equally beyond both edges.
pub fn centered_text_x(bounds: Rect, text_width: f32) -> f32 {
    bounds.x + centered_text_offset(bounds.w, text_width)
}

/// VT labels keep their left edge inside the text bounds when a word overflows.
pub fn centered_text_x_clamped(bounds: Rect, text_width: f32) -> f32 {
    bounds.x + centered_text_offset(bounds.w, text_width).max(0.0)
}

fn centered_text_offset(width: f32, text_width: f32) -> f32 {
    (width - text_width) / 2.0
}

/// Fixed screen-center labels retain their original subtraction association.
pub fn centered_label_x(center: f32, text_width: f32) -> f32 {
    center - text_width / 2.0
}

/// Story controls center their baseline using the selected font's first ascent.
pub fn centered_ascent_baseline(bounds: Rect, ascent: f32) -> f32 {
    bounds.y + (bounds.h + ascent) / 2.0
}

/// Place the first VT line; overflowing blocks retain their original top edge.
pub fn text_block_baseline(
    bounds: Rect,
    ascent: f32,
    line_height: f32,
    line_count: usize,
    centered: bool,
) -> f32 {
    bounds.y
        + ascent
        + if centered {
            ((bounds.h - line_height * line_count as f32) / 2.0).max(0.0)
        } else {
            0.0
        }
}

/// The established fallback skin; textured screens retain their own paint path.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FallbackButtonStyle {
    pub fill: Color,
    pub outline: Color,
    pub outline_width: f32,
    pub label_color: Color,
    pub font_size: f32,
    pub label_inset: f32,
    pub baseline_fraction: f32,
}

impl FallbackButtonStyle {
    pub fn label_position(self, bounds: Rect) -> Vec2 {
        vec2(
            bounds.x + self.label_inset,
            bounds.y + bounds.h * self.baseline_fraction,
        )
    }
}

pub const FALLBACK_BUTTON_STYLE: FallbackButtonStyle = FallbackButtonStyle {
    fill: Color::new(0.05, 0.2, 0.26, 0.96),
    outline: YELLOW,
    outline_width: 1.0,
    label_color: WHITE,
    font_size: 18.0,
    label_inset: 8.0,
    baseline_fraction: 0.7,
};

/// Paint in the original fill, outline, label order, without adding clipping.
pub fn fallback_button(bounds: Rect, label: &str) {
    let style = FALLBACK_BUTTON_STYLE;
    draw_rectangle(bounds.x, bounds.y, bounds.w, bounds.h, style.fill);
    draw_rectangle_lines(
        bounds.x,
        bounds.y,
        bounds.w,
        bounds.h,
        style.outline_width,
        style.outline,
    );
    let position = style.label_position(bounds);
    draw_text(
        label,
        position.x,
        position.y,
        style.font_size,
        style.label_color,
    );
}
