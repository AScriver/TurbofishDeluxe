//! Owned-art drawing shared by the concrete normal-game modal frontends.

use super::{Presentation, help_runtime};
use macroquad::prelude::*;

pub(super) const REQUIRED_IMAGE_IDS: &[&str] = &[
    "IMAGE_DIALOG",
    "IMAGE_DIALOGBUTTON",
    "IMAGE_EDITBOX",
    "IMAGE_SLIDERTRACK",
    "IMAGE_SLIDERTHUMB",
    "IMAGE_UNCHECKED",
    "IMAGE_CHECKED",
];

const SCREEN: Rect = Rect::new(0.0, 0.0, 640.0, 480.0);

fn skin(presentation: &Presentation, id: &str, source: Option<Rect>, bounds: Rect) {
    if bounds.w <= 0.0 || bounds.h <= 0.0 {
        return;
    }
    let texture = &presentation.images[id];
    let source = source.unwrap_or(Rect::new(0.0, 0.0, texture.width(), texture.height()));
    help_runtime::image_box(texture, source, bounds, SCREEN);
}

pub(super) fn draw_panel(presentation: &Presentation, bounds: Rect) {
    skin(presentation, "IMAGE_DIALOG", None, bounds);
}

fn label(
    presentation: &Presentation,
    text: &str,
    bounds: Rect,
    font: &str,
    color: Color,
    centered: bool,
) {
    if bounds.w <= 0.0 || bounds.h <= 0.0 {
        return;
    }
    let rendered = &presentation.fonts[font];
    let ascent = rendered
        .metrics
        .layers
        .iter()
        .map(|layer| layer.ascent)
        .max()
        .unwrap_or(1)
        .max(1) as f32;
    let width = rendered.metrics.measure_width(text).unwrap_or(0) as f32;
    let x = if centered {
        crate::ui::centered_text_x_clamped(bounds, width)
    } else {
        bounds.x
    };
    let baseline = bounds.y + (bounds.h - ascent) / 2.0 + ascent;
    help_runtime::font_text(rendered, text, x, baseline, color, bounds);
}

pub(super) fn draw_title(presentation: &Presentation, text: &str, bounds: Rect) {
    label(
        presentation,
        text,
        bounds,
        "JungleFever15outline",
        Color::from_rgba(255, 200, 0, 255),
        true,
    );
}

pub(super) fn draw_body(presentation: &Presentation, text: &str, bounds: Rect, centered: bool) {
    let rendered = &presentation.fonts["JungleFever10outline"];
    let ascent = rendered
        .metrics
        .layers
        .iter()
        .map(|layer| layer.ascent)
        .max()
        .unwrap_or(1)
        .max(1) as f32;
    let line_height = ascent + 7.0;
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            let candidate = if line.is_empty() {
                word.to_owned()
            } else {
                format!("{line} {word}")
            };
            if !line.is_empty()
                && rendered.metrics.measure_width(&candidate).unwrap_or(0) as f32 > bounds.w
            {
                lines.push(std::mem::take(&mut line));
                line.push_str(word);
            } else {
                line = candidate;
            }
        }
        lines.push(line);
    }
    for (index, line) in lines.iter().enumerate() {
        let y = bounds.y + index as f32 * line_height;
        if y >= bounds.bottom() {
            break;
        }
        label(
            presentation,
            line,
            Rect::new(bounds.x, y, bounds.w, line_height.min(bounds.bottom() - y)),
            "JungleFever10outline",
            WHITE,
            centered,
        );
    }
}

pub(super) fn draw_button(
    presentation: &Presentation,
    bounds: Rect,
    text: &str,
    enabled: bool,
    pressed: bool,
    focused: bool,
) {
    let texture = &presentation.images["IMAGE_DIALOGBUTTON"];
    let width = texture.width() / 3.0;
    let state = if enabled && pressed {
        2.0
    } else if enabled && focused {
        1.0
    } else {
        0.0
    };
    let offset = if enabled && pressed { 1.0 } else { 0.0 };
    let translated = Rect::new(bounds.x + offset, bounds.y + offset, bounds.w, bounds.h);
    skin(
        presentation,
        "IMAGE_DIALOGBUTTON",
        Some(Rect::new(width * state, 0.0, width, texture.height())),
        translated,
    );
    let text_bounds = Rect::new(translated.x - 2.0, translated.y, translated.w, translated.h);
    label(
        presentation,
        text,
        text_bounds,
        "JungleFever12outline",
        if enabled {
            Color::from_rgba(255, 255, 100, 255)
        } else {
            GRAY
        },
        true,
    );
}

pub(super) fn draw_edit_box(presentation: &Presentation, bounds: Rect, text: &str, focused: bool) {
    skin(presentation, "IMAGE_EDITBOX", None, bounds);
    let content = crate::ui::inset(bounds, 8.0);
    label(presentation, text, content, "ContinuumBold12", WHITE, false);
    if focused {
        let width = presentation.fonts["ContinuumBold12"]
            .metrics
            .measure_width(text)
            .unwrap_or(0) as f32;
        let x = (content.x + width + 1.0).min(content.right() - 1.0);
        draw_line(x, content.y + 1.0, x, content.bottom() - 1.0, 1.0, YELLOW);
    }
}

pub(super) fn draw_checkbox(
    presentation: &Presentation,
    bounds: Rect,
    checked: bool,
    enabled: bool,
    focused: bool,
) {
    let id = if checked {
        "IMAGE_CHECKED"
    } else {
        "IMAGE_UNCHECKED"
    };
    let texture = &presentation.images[id];
    draw_texture_ex(
        texture,
        bounds.x,
        bounds.y,
        if enabled { WHITE } else { GRAY },
        DrawTextureParams {
            dest_size: Some(Vec2::new(bounds.w, bounds.h)),
            ..Default::default()
        },
    );
    if focused && enabled {
        draw_rectangle_lines(
            bounds.x - 2.0,
            bounds.y - 2.0,
            bounds.w + 4.0,
            bounds.h + 4.0,
            1.0,
            YELLOW,
        );
    }
}

pub(super) fn draw_slider(
    presentation: &Presentation,
    bounds: Rect,
    value: f32,
    enabled: bool,
    focused: bool,
) {
    let track = &presentation.images["IMAGE_SLIDERTRACK"];
    let track_bounds = Rect::new(
        bounds.x,
        bounds.y + (bounds.h - track.height()) / 2.0,
        bounds.w,
        track.height(),
    );
    skin(presentation, "IMAGE_SLIDERTRACK", None, track_bounds);
    let thumb = &presentation.images["IMAGE_SLIDERTHUMB"];
    let value = if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let thumb_x = bounds.x + value * (bounds.w - thumb.width()).max(0.0);
    draw_texture(
        thumb,
        thumb_x,
        bounds.y + (bounds.h - thumb.height()) / 2.0,
        if enabled { WHITE } else { GRAY },
    );
    if focused && enabled {
        draw_rectangle_lines(
            bounds.x - 2.0,
            bounds.y - 2.0,
            bounds.w + 4.0,
            bounds.h + 4.0,
            1.0,
            YELLOW,
        );
    }
}
