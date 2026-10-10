//! Presentation adapter for Help's transient, read-only draw plan.

use super::{Presentation, RenderedFont};
use crate::help::{
    HelpController,
    demo::{
        HelpAction, HelpAmbient, HelpAssets, HelpBlend, HelpControl, HelpDraw, HelpFont, HelpLayer,
        HelpPoint, HelpPrimitive, HelpRect, HelpRenderPlan, HelpSprite, HelpTint, ImageMetrics,
        TextAlign, TextDraw, TileAxis,
    },
};
use macroquad::prelude::*;

pub(super) const REQUIRED_IMAGES: [HelpSprite; 35] = HelpSprite::ALL;
pub(super) const REQUIRED_FONTS: [HelpFont; 5] = HelpFont::ALL;

/// `HelpFont::asset_id` is the resource identity; these are the existing
/// installed font file stems used by `RenderedFont::load`.
pub(super) const fn font_file_name(font: HelpFont) -> &'static str {
    match font {
        HelpFont::ContinuumBold14 => "ContinuumBold14",
        HelpFont::ContinuumBold14Outline => "ContinuumBold14outback",
        HelpFont::ContinuumBold12 => "ContinuumBold12",
        HelpFont::JungleFever10Outline => "JungleFever10outline",
        HelpFont::JungleFever17Outline => "JungleFever17outline",
    }
}

struct Assets<'a>(&'a Presentation);

impl HelpAssets for Assets<'_> {
    fn image_metrics(&self, sprite: HelpSprite) -> Option<ImageMetrics> {
        let id = sprite.asset_id();
        let texture = self.0.images.get(id)?;
        let (columns, rows) = *self.0.image_cells.get(id)?;
        ImageMetrics::new(
            texture.width() as u32,
            texture.height() as u32,
            columns,
            rows,
        )
    }

    fn wave_cell(&self, updates: u64) -> Option<u16> {
        let cel = self
            .0
            .image_animations
            .get("IMAGE_WAVE")?
            .cel_at_time(updates)?;
        u16::try_from(cel).ok()
    }
}

pub(super) fn plan(presentation: &Presentation, help: &HelpController) -> Option<HelpRenderPlan> {
    help.render_plan(&Assets(presentation))
}

pub(super) fn hit_control(
    presentation: &Presentation,
    help: &HelpController,
    point: Vec2,
) -> Option<HelpAction> {
    plan(presentation, help)?.control_at(HelpPoint::new(point.x as i32, point.y as i32))
}

pub(super) fn draw(presentation: &Presentation, help: &HelpController) {
    let Some(plan) = plan(presentation, help) else {
        return;
    };
    draw_plan(presentation, &plan);
}

/// Root may insert its separately owned bubbles between background and ambient.
pub(super) fn draw_plan(presentation: &Presentation, plan: &HelpRenderPlan) {
    draw_plan_with_ambient(presentation, plan, |_| {});
}

pub(super) fn draw_plan_with_ambient(
    presentation: &Presentation,
    plan: &HelpRenderPlan,
    ambient: impl FnOnce(&HelpAmbient),
) {
    let mut ambient = Some(ambient);
    for draw in &plan.draws {
        if draw.layer > HelpLayer::Background
            && let Some(render) = ambient.take()
        {
            render(&plan.ambient);
        }
        draw_command(presentation, draw);
    }
    if let Some(render) = ambient {
        render(&plan.ambient);
    }
    for control in &plan.controls {
        draw_control(presentation, control);
    }
    if !plan.asset_issues.is_empty() {
        let y = if plan.page == 0 { 387.0 } else { 113.0 };
        draw_rectangle(335.0, y - 14.0, 295.0, 18.0, Color::new(0.0, 0.0, 0.0, 0.8));
        draw_text("Help illustration unavailable", 341.0, y, 15.0, YELLOW);
    }
}

fn tint(value: HelpTint) -> Color {
    match value {
        HelpTint::Rgba(red, green, blue, alpha) => Color::from_rgba(red, green, blue, alpha),
        HelpTint::Hsl {
            hue,
            saturation,
            luminance,
        } => macroquad::color::hsl_to_rgb(
            f32::from(hue) / 255.0,
            f32::from(saturation) / 255.0,
            f32::from(luminance) / 255.0,
        ),
    }
}

fn rect(value: HelpRect) -> Rect {
    Rect::new(
        value.x as f32,
        value.y as f32,
        value.width as f32,
        value.height as f32,
    )
}

fn intersection(area: Rect, clip: Rect) -> Option<Rect> {
    let left = area.x.max(clip.x);
    let top = area.y.max(clip.y);
    let right = (area.x + area.w).min(clip.x + clip.w);
    let bottom = (area.y + area.h).min(clip.y + clip.h);
    (right > left && bottom > top).then(|| Rect::new(left, top, right - left, bottom - top))
}

fn crop(mut source: Rect, destination: Rect, clip: Rect, mirror: bool) -> Option<(Rect, Rect)> {
    let visible = intersection(destination, clip)?;
    let sx = source.w / destination.w;
    let sy = source.h / destination.h;
    source.x += if mirror {
        (destination.x + destination.w - visible.x - visible.w) * sx
    } else {
        (visible.x - destination.x) * sx
    };
    source.y += (visible.y - destination.y) * sy;
    source.w = visible.w * sx;
    source.h = visible.h * sy;
    Some((source, visible))
}

pub(super) fn image(
    texture: &Texture2D,
    source: Rect,
    destination: Rect,
    clip: Rect,
    color: Color,
    mirror: bool,
) {
    if let Some((source, destination)) = crop(source, destination, clip, mirror) {
        draw_texture_ex(
            texture,
            destination.x,
            destination.y,
            color,
            DrawTextureParams {
                source: Some(source),
                dest_size: Some(vec2(destination.w, destination.h)),
                flip_x: mirror,
                ..Default::default()
            },
        );
    }
}

pub(super) fn fill(area: Rect, clip: Rect, color: Color) {
    if let Some(area) = intersection(area, clip) {
        draw_rectangle(area.x, area.y, area.w, area.h, color);
    }
}

pub(super) fn outline(area: Rect, clip: Rect, width: f32, color: Color) {
    fill(Rect::new(area.x, area.y, area.w, width), clip, color);
    fill(
        Rect::new(area.x, area.y + area.h - width, area.w, width),
        clip,
        color,
    );
    fill(Rect::new(area.x, area.y, width, area.h), clip, color);
    fill(
        Rect::new(area.x + area.w - width, area.y, width, area.h),
        clip,
        color,
    );
}

fn line(start: Vec2, end: Vec2, clip: Rect, color: Color) {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let mut low: f32 = 0.0;
    let mut high: f32 = 1.0;
    for (p, q) in [
        (-dx, start.x - clip.x),
        (dx, clip.x + clip.w - start.x),
        (-dy, start.y - clip.y),
        (dy, clip.y + clip.h - start.y),
    ] {
        if p == 0.0 {
            if q < 0.0 {
                return;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                low = low.max(t);
            } else {
                high = high.min(t);
            }
        }
    }
    if low <= high {
        draw_line(
            start.x + low * dx,
            start.y + low * dy,
            start.x + high * dx,
            start.y + high * dy,
            1.0,
            color,
        );
    }
}

pub(super) fn image_box(texture: &Texture2D, source: Rect, destination: Rect, clip: Rect) {
    let edge_x = (source.w / 3.0).floor().min(destination.w / 2.0);
    let edge_y = (source.h / 3.0).floor().min(destination.h / 2.0);
    let sx = [source.x, source.x + edge_x, source.x + source.w - edge_x];
    let sy = [source.y, source.y + edge_y, source.y + source.h - edge_y];
    let sw = [edge_x, source.w - 2.0 * edge_x, edge_x];
    let sh = [edge_y, source.h - 2.0 * edge_y, edge_y];
    let dx = [
        destination.x,
        destination.x + edge_x,
        destination.x + destination.w - edge_x,
    ];
    let dy = [
        destination.y,
        destination.y + edge_y,
        destination.y + destination.h - edge_y,
    ];
    let dw = [edge_x, destination.w - 2.0 * edge_x, edge_x];
    let dh = [edge_y, destination.h - 2.0 * edge_y, edge_y];
    for row in 0..3 {
        for column in 0..3 {
            if sw[column] <= 0.0 || sh[row] <= 0.0 {
                continue;
            }
            let mut y = 0.0;
            while y < dh[row] {
                let height = sh[row].min(dh[row] - y);
                let mut x = 0.0;
                while x < dw[column] {
                    let width = sw[column].min(dw[column] - x);
                    image(
                        texture,
                        Rect::new(sx[column], sy[row], width, height),
                        Rect::new(dx[column] + x, dy[row] + y, width, height),
                        clip,
                        WHITE,
                        false,
                    );
                    x += sw[column];
                }
                y += sh[row];
            }
        }
    }
}

pub(super) fn font_text(
    font: &RenderedFont,
    text: &str,
    x: f32,
    baseline: f32,
    color: Color,
    clip: Rect,
) {
    let characters: Vec<_> = text.chars().collect();
    for (layer, atlas) in font.metrics.layers.iter().zip(&font.atlases) {
        let mut pen = x;
        for (index, character) in characters.iter().enumerate() {
            if let Some(glyph) = layer.glyphs.get(character) {
                if let Some(bounds) = glyph.rect {
                    let source = Rect::new(
                        bounds.x as f32,
                        bounds.y as f32,
                        bounds.width as f32,
                        bounds.height as f32,
                    );
                    image(
                        atlas,
                        source,
                        Rect::new(
                            pen + glyph.offset_x as f32,
                            baseline - (layer.ascent - glyph.offset_y) as f32,
                            source.w,
                            source.h,
                        ),
                        clip,
                        color,
                        false,
                    );
                }
                pen += (glyph.advance + layer.spacing) as f32;
                if let Some(next) = characters.get(index + 1) {
                    pen += layer
                        .kerning
                        .get(&(*character, *next))
                        .copied()
                        .unwrap_or(0) as f32;
                }
            }
        }
    }
}

fn draw_command(presentation: &Presentation, draw: &HelpDraw) {
    if draw.clip.width <= 0 || draw.clip.height <= 0 {
        return;
    }
    let clip = rect(draw.clip);
    let color = tint(draw.tint);
    match &draw.primitive {
        HelpPrimitive::Fill(area) => fill(rect(*area), clip, color),
        HelpPrimitive::Outline(area) => outline(rect(*area), clip, 1.0, color),
        HelpPrimitive::Line { start, end } => line(
            vec2(start.x as f32, start.y as f32),
            vec2(end.x as f32, end.y as f32),
            clip,
            color,
        ),
        HelpPrimitive::Sprite(sprite) => {
            let id = sprite.sprite.asset_id();
            if let Some(texture) = presentation.images.get(id) {
                if draw.blend == HelpBlend::Additive {
                    gl_use_material(&presentation.additive);
                }
                image(
                    texture,
                    rect(sprite.source),
                    rect(sprite.destination),
                    clip,
                    color,
                    sprite.mirrored,
                );
                if draw.blend == HelpBlend::Additive {
                    gl_use_default_material();
                }
            }
        }
        HelpPrimitive::ImageBox {
            sprite,
            destination,
        } => {
            if let Some(texture) = presentation.images.get(sprite.asset_id()) {
                image_box(
                    texture,
                    Rect::new(0.0, 0.0, texture.width(), texture.height()),
                    rect(*destination),
                    clip,
                );
            }
        }
        HelpPrimitive::Tile {
            sprite,
            destination,
            axis,
        } => draw_tiles(presentation, *sprite, *destination, *axis, color, clip),
        HelpPrimitive::Text(text) => draw_text_block(presentation, text, color, clip),
    }
}

fn draw_tiles(
    presentation: &Presentation,
    sprite: HelpSprite,
    destination: HelpRect,
    axis: TileAxis,
    color: Color,
    clip: Rect,
) {
    let Some(texture) = presentation.images.get(sprite.asset_id()) else {
        return;
    };
    let (tile_width, tile_height) = (texture.width(), texture.height());
    if tile_width <= 0.0 || tile_height <= 0.0 {
        return;
    }
    let mut cursor = 0.0;
    let length = match axis {
        TileAxis::Horizontal => destination.width as f32,
        TileAxis::Vertical => destination.height as f32,
    };
    let step = match axis {
        TileAxis::Horizontal => tile_width,
        TileAxis::Vertical => tile_height,
    };
    while cursor < length {
        let remainder = (length - cursor).min(step);
        let source = match axis {
            TileAxis::Horizontal => Rect::new(0.0, 0.0, remainder, tile_height),
            TileAxis::Vertical => Rect::new(0.0, 0.0, tile_width, remainder),
        };
        let (x, y) = match axis {
            TileAxis::Horizontal => (destination.x as f32 + cursor, destination.y as f32),
            TileAxis::Vertical => (destination.x as f32, destination.y as f32 + cursor),
        };
        image(
            texture,
            source,
            Rect::new(x, y, source.w, source.h),
            clip,
            color,
            false,
        );
        cursor += step;
    }
}

fn draw_text_block(presentation: &Presentation, text: &TextDraw, color: Color, clip: Rect) {
    let font = presentation.fonts.get(font_file_name(text.font));
    let mut lines = Vec::<String>::new();
    for paragraph in text.text.split('\n') {
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            let candidate = if current.is_empty() {
                word.to_owned()
            } else {
                format!("{current} {word}")
            };
            let width = font.map_or(candidate.len() as i32 * 8, |font| {
                font.metrics.measure_width(&candidate).unwrap_or(0)
            });
            if text.wrap && !current.is_empty() && width > text.bounds.width {
                lines.push(std::mem::take(&mut current));
                current.push_str(word);
            } else {
                current = candidate;
            }
        }
        lines.push(current);
    }
    for (index, line) in lines.iter().enumerate() {
        let baseline = text.baseline + index as i32 * i32::from(text.line_height);
        if baseline > text.bounds.y + text.bounds.height {
            break;
        }
        let width = font.map_or(line.len() as i32 * 8, |font| {
            font.metrics.measure_width(line).unwrap_or(0)
        });
        let x = match text.align {
            TextAlign::Left => text.bounds.x,
            TextAlign::Center => text.bounds.x + (text.bounds.width - width) / 2,
        } as f32;
        if let Some(outline) = text
            .outline
            .and_then(|outline| presentation.fonts.get(font_file_name(outline)))
        {
            font_text(outline, line, x, baseline as f32, BLACK, clip);
        }
        if let Some(font) = font {
            font_text(font, line, x, baseline as f32, color, clip);
        }
    }
}

fn draw_control(presentation: &Presentation, control: &HelpControl) {
    let bounds = rect(control.bounds);
    if let Some(texture) = presentation.images.get(control.sprite.asset_id()) {
        image_box(
            texture,
            Rect::new(0.0, 0.0, texture.width(), texture.height()),
            bounds,
            bounds,
        );
    } else {
        fill(bounds, bounds, DARKGRAY);
    }
    if control.selected {
        outline(bounds, bounds, 2.0, ORANGE);
    }
    let font = presentation.fonts.get(font_file_name(control.font));
    let width = font.map_or(control.label.len() as i32 * 8, |font| {
        font.metrics.measure_width(&control.label).unwrap_or(0)
    });
    let x = bounds.x + (bounds.w - width as f32) / 2.0;
    let baseline = bounds.y + bounds.h * 0.7;
    if let Some(font) = font {
        font_text(font, &control.label, x, baseline, WHITE, bounds);
    }
}
