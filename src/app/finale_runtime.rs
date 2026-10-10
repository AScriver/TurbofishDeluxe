//! Transient finale paint and decorative effects. Progress, unlocks, result
//! credit, music, and exit remain owned by the caller's session/controller.

use super::{Presentation, RenderedFont};
use crate::{
    adventure::AdventureSession,
    bubbles::{
        BubbleBlend, BubbleConfig, BubbleManager, BubbleRect, BubbleRenderOp, BubbleSpriteSource,
    },
    interlude::{
        CATALOG_UNAVAILABLE_MESSAGE, END_SCROLL, FinaleBlend, FinalePresentation, FinaleSprite,
        FinaleSpriteFrame, InterludeController, catalog::FinaleCatalog,
    },
};
use macroquad::prelude::*;
use std::path::Path;

pub(super) const REQUIRED_IMAGE_IDS: &[&str] = &[
    "IMAGE_EGGBRINKLEY",
    "IMAGE_EGGNOSTRADAMUS",
    "IMAGE_EGGSTANLEY",
    "IMAGE_EGGWALTER",
];
pub(super) const REQUIRED_FONT_NAMES: &[&str] = &["Liddie18", "Liddie15"];

pub(super) struct FinaleRuntime {
    catalog: Option<FinaleCatalog>,
    catalog_error: Option<String>,
    rear_bubbles: BubbleManager,
    front_bubbles: BubbleManager,
    fx_seed: u64,
    bubble_fish_start_scroll: Option<u32>,
    fish_started: bool,
    fish_settled: bool,
}

impl FinaleRuntime {
    pub(super) fn new(executable: &Path) -> Self {
        let (catalog, catalog_error) = match FinaleCatalog::load(executable) {
            Ok(catalog) => (Some(catalog), None),
            Err(error) => (
                None,
                Some(format!("{CATALOG_UNAVAILABLE_MESSAGE}: {error}")),
            ),
        };
        let mut runtime = Self {
            catalog,
            catalog_error,
            rear_bubbles: BubbleManager::new(BubbleConfig {
                bubble_bounds: BubbleRect::new(0, -20, 640, 520),
                max_bubbles: 10,
                bubble_spawn_chance: 3,
                ..BubbleConfig::default()
            }),
            front_bubbles: BubbleManager::new(BubbleConfig {
                bubble_bounds: BubbleRect::new(0, -20, 640, 520),
                fish_bounds: BubbleRect::new(0, 200, 640, 240),
                max_bubbles: 10,
                bubble_spawn_chance: 3,
                max_fish: 0,
                fish_spawn_chance: 0,
                default_fish_vy: -1.0,
                clip_fish: false,
            }),
            fx_seed: 0x6a09_e667_f3bc_c909,
            bubble_fish_start_scroll: None,
            fish_started: false,
            fish_settled: false,
        };
        let seed = &mut runtime.fx_seed;
        let mut roll = |upper| fx_roll(seed, upper);
        runtime.rear_bubbles.update_a_lot(&mut roll);
        runtime.front_bubbles.update_a_lot(&mut roll);
        runtime
    }

    /// Call before `InterludeController::advance` once per UI update. Focus
    /// freezes decorative particles along with the source scroll and halo.
    pub(super) fn advance(&mut self, controller: &InterludeController, has_focus: bool) {
        if !has_focus || controller.exited() {
            return;
        }
        let fish_start = *self
            .bubble_fish_start_scroll
            .get_or_insert_with(|| controller.presentation(|_| false).bubble_fish_start_scroll);
        if !self.fish_started && controller.scroll() >= fish_start {
            self.front_bubbles.set_fish_config(3, 3);
            self.fish_started = true;
        }
        if !self.fish_settled && controller.scroll() >= END_SCROLL {
            self.front_bubbles
                .set_fish_bounds(BubbleRect::new(0, 30, 640, 420));
            self.front_bubbles.set_default_fish_vy(0.0);
            self.fish_settled = true;
        }
        let seed = &mut self.fx_seed;
        let mut roll = |upper| fx_roll(seed, upper);
        self.rear_bubbles.update(&mut roll);
        self.front_bubbles.update(&mut roll);
    }

    pub(super) fn draw(
        &self,
        presentation: &Presentation,
        session: &AdventureSession,
        controller: &InterludeController,
    ) {
        let finale = controller.presentation(|pet| session.progress.has_pet(pet));
        clear_background(Color::from_rgba(
            finale.background_rgba[0],
            finale.background_rgba[1],
            finale.background_rgba[2],
            finale.background_rgba[3],
        ));
        for op in self.rear_bubbles.render_ops() {
            draw_bubble_op(presentation, op);
        }
        if let Some(catalog) = &self.catalog {
            draw_rows(presentation, catalog, &finale);
            for sprite in &finale.sprites {
                draw_finale_sprite(presentation, sprite, finale.clip);
            }
        } else {
            draw_text(CATALOG_UNAVAILABLE_MESSAGE, 135.0, 215.0, 24.0, WHITE);
            if let Some(error) = &self.catalog_error {
                let diagnostic: String = error.chars().take(70).collect();
                draw_text(&diagnostic, 32.0, 245.0, 14.0, WHITE);
            }
        }
        for op in self.front_bubbles.render_ops() {
            draw_bubble_op(presentation, op);
        }
        // Source graphics masks keep bubbles, text, and partial sprites inside
        // the 40..440 content band without leaving a global scissor/material.
        draw_rectangle(0.0, 0.0, 640.0, 40.0, BLACK);
        draw_rectangle(0.0, 440.0, 640.0, 40.0, BLACK);
        let height = presentation.images["IMAGE_MAINBUTTON"].height();
        presentation.main_button(Rect::new(186.0, 445.0, 264.0, height), "Main Menu");
    }
}

fn fx_roll(seed: &mut u64, upper: u64) -> u64 {
    debug_assert!(upper > 0);
    let mut next = *seed;
    if next == 0 {
        next = 0x9e37_79b9_7f4a_7c15;
    }
    next ^= next >> 12;
    next ^= next << 25;
    next ^= next >> 27;
    *seed = next;
    next.wrapping_mul(0x2545_f491_4f6c_dd1d) % upper
}

fn draw_rows(presentation: &Presentation, catalog: &FinaleCatalog, finale: &FinalePresentation) {
    for row in &finale.rows {
        let name = match row.font {
            crate::interlude::FinaleFont::Heading => REQUIRED_FONT_NAMES[0],
            crate::interlude::FinaleFont::Body => REQUIRED_FONT_NAMES[1],
        };
        let Some(font) = presentation.fonts.get(name) else {
            continue;
        };
        let text = row.text.resolve(catalog);
        let Ok(width) = font.metrics.measure_width(&text) else {
            continue;
        };
        let x = row.column.text_x(width) as f32;
        let [r, g, b, a] = row.font.rgba();
        draw_clipped_font(
            font,
            &text,
            x,
            row.y as f32,
            Color::from_rgba(r, g, b, a),
            finale.clip,
        );
    }
}

fn draw_clipped_font(
    font: &RenderedFont,
    text: &str,
    x: f32,
    baseline: f32,
    tint: Color,
    clip: [i32; 4],
) {
    let characters: Vec<_> = text.chars().collect();
    for (layer, atlas) in font.metrics.layers.iter().zip(&font.atlases) {
        let mut pen = x;
        for (index, character) in characters.iter().enumerate() {
            if let Some(glyph) = layer.glyphs.get(character) {
                if let Some(rect) = glyph.rect {
                    let source = Rect::new(
                        rect.x as f32,
                        rect.y as f32,
                        rect.width as f32,
                        rect.height as f32,
                    );
                    let glyph_x = pen + glyph.offset_x as f32;
                    let glyph_y = baseline - (layer.ascent - glyph.offset_y) as f32;
                    if let Some((draw_x, draw_y, source)) =
                        crop_source(glyph_x, glyph_y, source, clip, false)
                    {
                        draw_texture_ex(
                            atlas,
                            draw_x,
                            draw_y,
                            tint,
                            DrawTextureParams {
                                source: Some(source),
                                dest_size: Some(vec2(source.w, source.h)),
                                ..Default::default()
                            },
                        );
                    }
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

fn draw_finale_sprite(presentation: &Presentation, sprite: &FinaleSprite, clip: [i32; 4]) {
    let id = sprite.image.resource_key();
    let Some(texture) = presentation.images.get(id) else {
        return;
    };
    let source = match sprite.frame {
        FinaleSpriteFrame::WholeImage => Rect::new(0.0, 0.0, texture.width(), texture.height()),
        FinaleSpriteFrame::Cell { column, row } => {
            let Some(&(cols, rows)) = presentation.image_cells.get(id) else {
                return;
            };
            if cols == 0 || rows == 0 || u32::from(column) >= cols || u32::from(row) >= rows {
                return;
            }
            let width = texture.width() / cols as f32;
            let height = texture.height() / rows as f32;
            Rect::new(column as f32 * width, row as f32 * height, width, height)
        }
    };
    let Some((x, y, source)) = crop_source(
        sprite.x as f32,
        sprite.y as f32,
        source,
        clip,
        sprite.mirror,
    ) else {
        return;
    };
    let [r, g, b, a] = sprite.rgba;
    if sprite.blend == FinaleBlend::Additive {
        gl_use_material(&presentation.additive);
    }
    draw_texture_ex(
        texture,
        x,
        y,
        Color::from_rgba(r, g, b, a),
        DrawTextureParams {
            source: Some(source),
            dest_size: Some(vec2(source.w, source.h)),
            flip_x: sprite.mirror,
            ..Default::default()
        },
    );
    if sprite.blend == FinaleBlend::Additive {
        gl_use_default_material();
    }
}

fn draw_bubble_op(presentation: &Presentation, op: BubbleRenderOp) {
    let id = op.sheet.resource_key();
    let Some(texture) = presentation.images.get(id) else {
        return;
    };
    let source = match op.source {
        BubbleSpriteSource::Cel(column) => {
            let Some(&(cols, rows)) = presentation.image_cells.get(id) else {
                return;
            };
            if cols == 0 || rows == 0 || u32::from(column) >= cols {
                return;
            }
            let width = texture.width() / cols as f32;
            let height = texture.height() / rows as f32;
            Rect::new(column as f32 * width, 0.0, width, height)
        }
        BubbleSpriteSource::Rect(rect) => Rect::new(
            rect.x as f32,
            rect.y as f32,
            rect.width as f32,
            rect.height as f32,
        ),
    };
    let clipped = op.clip.map_or([0, 40, 640, 400], |rect| {
        [rect.x, rect.y, rect.width, rect.height]
    });
    let Some((x, y, source)) = crop_source(op.x, op.y, source, clipped, op.mirror_x) else {
        return;
    };
    let tint = op.tint.map_or(WHITE, |color| {
        Color::from_rgba(color.red, color.green, color.blue, 255)
    });
    if op.blend == BubbleBlend::Additive {
        gl_use_material(&presentation.additive);
    }
    draw_texture_ex(
        texture,
        x,
        y,
        tint,
        DrawTextureParams {
            source: Some(source),
            dest_size: Some(vec2(source.w, source.h)),
            flip_x: op.mirror_x,
            ..Default::default()
        },
    );
    if op.blend == BubbleBlend::Additive {
        gl_use_default_material();
    }
}

fn crop_source(
    x: f32,
    y: f32,
    mut source: Rect,
    clip: [i32; 4],
    mirror_x: bool,
) -> Option<(f32, f32, Rect)> {
    let left = x.max(clip[0] as f32);
    let top = y.max(clip[1] as f32);
    let right = (x + source.w).min((clip[0] + clip[2]) as f32);
    let bottom = (y + source.h).min((clip[1] + clip[3]) as f32);
    if right <= left || bottom <= top {
        return None;
    }
    source.x += if mirror_x {
        x + source.w - right
    } else {
        left - x
    };
    source.y += top - y;
    source.w = right - left;
    source.h = bottom - top;
    Some((left, top, source))
}
