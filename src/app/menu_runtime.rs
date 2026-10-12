//! Transient renderer/input adapter for the source-shaped GameSelector model.
//! Profile, mode admission, dialogs, navigation and saves remain caller-owned.

use super::Presentation;
use crate::{
    assets::GameAssets,
    game_selector::{
        Availability, Blend, DrawCommand, FontMetadata, GameSelector, HorizontalAlign, ImageDraw,
        ImageMetadata, ImageSizing, LockReason, ModeAvailability, Paint, PixelRect, Point,
        ProfileFacts, SelectorFacts, SelectorFont, SelectorImage, SelectorInput, SelectorIntent,
        SelectorResources, SelectorRng, SelectorScene, TextDraw, TextLayout, Trophy, TrophyMask,
        Unavailable,
    },
};
use macroquad::prelude::*;

pub(super) const REQUIRED_IMAGE_IDS: [&str; 18] = [
    "IMAGE_SELECTORSCREEN",
    "IMAGE_BLANK",
    "IMAGE_BATTLETANKBUTTON",
    "IMAGE_BATTLETANKBUTTOND",
    "IMAGE_MIDDLEBUTTON",
    "IMAGE_MIDDLEBUTTOND",
    "IMAGE_LEFTBUTTON",
    "IMAGE_RIGHTBUTTON",
    "IMAGE_MAINBUTTON",
    "IMAGE_CENTERBUTTON",
    "IMAGE_SELECTORSPEECHBUBBLE",
    "IMAGE_WELCOMETO",
    "IMAGE_WELCOMEBACK",
    "IMAGE_BONUSAWARD",
    "IMAGE_BONUSAWARD2",
    "IMAGE_MERYLBLINK",
    "IMAGE_MERYLTAIL",
    "IMAGE_SPARKLE",
];
pub(super) const REQUIRED_FONT_NAMES: [&str; 5] = [
    "JungleFever15outline",
    "JungleFever10outline",
    "BlambotPro8",
    "BlambotPro15",
    "Pix118",
];

pub(super) const fn font_file_name(font: SelectorFont) -> &'static str {
    match font {
        SelectorFont::ModeButton => "JungleFever15outline",
        SelectorFont::SmallOutline => "JungleFever10outline",
        SelectorFont::Hint => "BlambotPro8",
        SelectorFont::ProfileName => "BlambotPro15",
        SelectorFont::ProfileLink => "Pix118",
    }
}

/// Immutable menu facts from the current durable profile. `returning_player`
/// uses the project's session clock; this is a presentation heuristic.
pub(super) fn facts<'a>(
    profiles: &'a crate::profiles::Profiles,
    modal_open: bool,
    context_epoch: u64,
) -> SelectorFacts<'a> {
    let Some(entry) = profiles.current() else {
        return SelectorFacts {
            profile: None,
            modes: ModeAvailability::default(),
            modal_open,
            debug_keys_enabled: false,
            context_epoch,
        };
    };
    let session = &entry.session;
    let progress = &session.progress;
    let available = Availability::Available;
    let modes = ModeAvailability {
        adventure: available,
        time_trial: if session.time_trial_tank_available(1) {
            available
        } else {
            Availability::Locked(LockReason::CompleteAdventureTank)
        },
        challenge: if progress.adventure_completed {
            available
        } else {
            Availability::Locked(LockReason::CompleteAdventure)
        },
        virtual_tank: available,
        sandbox: if crate::sandbox::admitted(progress.purchase_cursor) {
            available
        } else {
            Availability::Locked(LockReason::CompleteBonusPurchases)
        },
    };
    let trophy = if !progress.adventure_completed {
        Trophy::None
    } else if progress.stories.cyrax_unlocked {
        Trophy::Gold
    } else {
        Trophy::Silver
    };
    SelectorFacts {
        profile: Some(ProfileFacts {
            id: entry.id,
            name: &entry.name,
            tank: progress.tank,
            level: progress.level,
            adventure_completed: progress.adventure_completed,
            returning_player: session.ticks > 0,
            cyrax_defeats: progress.adventure_completions,
            shells: progress.shell_balance,
            trophy,
        }),
        modes,
        modal_open,
        debug_keys_enabled: false,
        context_epoch,
    }
}

/// Presentation-only streams. They never consume Board or Profile RNG.
struct MenuRandom {
    characters: u64,
    effects: u64,
}

impl MenuRandom {
    fn new(seed: u64) -> Self {
        Self {
            characters: seed ^ 0x5345_4c45_4354_4f52,
            effects: seed ^ 0x5452_4f50_4859_4658,
        }
    }

    fn draw(state: &mut u64) -> u32 {
        *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = *state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        (value ^ (value >> 31)) as u32
    }
}

impl SelectorRng for MenuRandom {
    fn character_u32(&mut self) -> u32 {
        Self::draw(&mut self.characters)
    }

    fn effect_u32(&mut self) -> u32 {
        Self::draw(&mut self.effects)
    }
}

struct SilverMask {
    width: u32,
    height: u32,
    alpha: Vec<u8>,
}

impl SilverMask {
    fn load(assets: &GameAssets) -> Option<Self> {
        let image = assets
            .load_image(SelectorImage::SilverTrophy.resource_id())
            .ok()?;
        let alpha = image.pixels.chunks_exact(4).map(|pixel| pixel[3]).collect();
        Some(Self {
            width: image.width,
            height: image.height,
            alpha,
        })
    }
}

impl TrophyMask for Option<SilverMask> {
    fn available(&self, image: SelectorImage) -> bool {
        image == SelectorImage::SilverTrophy && self.is_some()
    }

    fn alpha(&self, image: SelectorImage, x: u32, y: u32) -> Option<u8> {
        if image != SelectorImage::SilverTrophy {
            return None;
        }
        let mask = self.as_ref()?;
        if x >= mask.width || y >= mask.height {
            return None;
        }
        mask.alpha.get((y * mask.width + x) as usize).copied()
    }
}

struct Resources<'a>(&'a Presentation);

impl SelectorResources for Resources<'_> {
    fn image(&self, image: SelectorImage) -> Option<ImageMetadata> {
        let id = image.resource_id();
        let texture = self.0.images.get(id)?;
        let (columns, rows) = *self.0.image_cells.get(id)?;
        Some(ImageMetadata {
            width: texture.width() as u32,
            height: texture.height() as u32,
            columns,
            rows,
        })
    }

    fn font(&self, font: SelectorFont) -> Option<FontMetadata> {
        let font = self.0.fonts.get(font_file_name(font))?;
        let layer = font.metrics.layers.first()?;
        // Declared point size is the owned script's line-height authority here.
        Some(FontMetadata {
            height: u32::try_from(font.metrics.default_point_size).ok()?,
            ascent: u32::try_from(layer.ascent).ok()?,
            ascent_padding: u32::try_from(layer.ascent_padding).ok()?,
        })
    }

    fn text_width(&self, font: SelectorFont, text: &str) -> Option<u32> {
        let font = self.0.fonts.get(font_file_name(font))?;
        u32::try_from(font.metrics.measure_width(text).ok()?).ok()
    }

    fn animation_cel(&self, image: SelectorImage, countdown: u16) -> Option<u32> {
        self.0
            .image_animations
            .get(image.resource_id())?
            .cel_at_time(u64::from(countdown))
    }

    fn animation_is_hidden(&self, image: SelectorImage, countdown: u16) -> bool {
        self.0
            .image_animations
            .get(image.resource_id())
            .is_some_and(|animation| animation.cel_at_time(u64::from(countdown)).is_none())
    }
}

/// Construct on each selector entry. Drop on navigation; no durable state.
pub(super) struct MenuRuntime {
    selector: GameSelector,
    random: MenuRandom,
    silver_mask: Option<SilverMask>,
    was_focused: bool,
    pointer_inside: bool,
}

impl MenuRuntime {
    pub(super) fn new(seed: u64, assets: &GameAssets) -> Self {
        let mut random = MenuRandom::new(seed);
        let selector = GameSelector::new(&mut random);
        Self {
            selector,
            random,
            silver_mask: SilverMask::load(assets),
            was_focused: true,
            pointer_inside: false,
        }
    }

    /// Caller supplies current immutable profile/lock/modal facts on each event.
    pub(super) fn input(
        &mut self,
        input: SelectorInput,
        facts: &SelectorFacts<'_>,
        presentation: &Presentation,
    ) -> Vec<SelectorIntent> {
        self.selector.input(input, facts, &Resources(presentation))
    }

    /// Forward one native frame to the typed controller in event order.
    /// Caller owns viewport conversion, modal priority and all resulting routes.
    pub(super) fn input_frame(
        &mut self,
        facts: &SelectorFacts<'_>,
        presentation: &Presentation,
        pointer: Vec2,
        focused: bool,
    ) -> Vec<SelectorIntent> {
        let mut intentions = Vec::new();
        if !focused {
            if self.was_focused {
                intentions.extend(self.input(SelectorInput::FocusLost, facts, presentation));
            }
            self.was_focused = false;
            self.pointer_inside = false;
            while get_char_pressed().is_some() {}
            return intentions;
        }
        self.was_focused = true;
        if !pointer.x.is_finite() || !pointer.y.is_finite() {
            if self.pointer_inside {
                intentions.extend(self.input(SelectorInput::Cancel, facts, presentation));
            }
            self.pointer_inside = false;
        } else {
            let point = logical_point(pointer)
                .filter(|point| point.x >= 0 && point.x < 640 && point.y >= 0 && point.y < 480);
            if let Some(point) = point {
                self.pointer_inside = true;
                intentions.extend(self.input(
                    SelectorInput::PointerMove(point),
                    facts,
                    presentation,
                ));
                if is_mouse_button_pressed(MouseButton::Left) {
                    intentions.extend(self.input(
                        SelectorInput::PrimaryDown(point),
                        facts,
                        presentation,
                    ));
                }
                if is_mouse_button_released(MouseButton::Left) {
                    intentions.extend(self.input(
                        SelectorInput::PrimaryUp(point),
                        facts,
                        presentation,
                    ));
                }
            } else if self.pointer_inside {
                self.pointer_inside = false;
                intentions.extend(self.input(SelectorInput::PointerLeft, facts, presentation));
            }
        }
        let mut had_character = false;
        while let Some(character) = get_char_pressed() {
            had_character = true;
            if !facts.modal_open {
                intentions.extend(self.input(
                    SelectorInput::Character(character),
                    facts,
                    presentation,
                ));
            }
        }
        if !had_character && get_last_key_pressed().is_some() {
            intentions.extend(self.input(SelectorInput::NonCharacterKey, facts, presentation));
        }
        intentions
    }

    /// Existing EntryCode owns its sequence. Call only when it fires.
    pub(super) fn sandbox_activated(&self, facts: &SelectorFacts<'_>) -> Option<SelectorIntent> {
        self.selector.sandbox_activated(facts)
    }

    /// Exactly one call per 1x selector UI tick, never from draw or input polling.
    pub(super) fn tick(
        &mut self,
        facts: &SelectorFacts<'_>,
        presentation: &Presentation,
    ) -> Vec<SelectorIntent> {
        self.selector.update(
            facts,
            &Resources(presentation),
            &mut self.random,
            &self.silver_mask,
        )
    }

    /// Draw within caller's logical 640x480 camera, beneath modal overlays.
    /// Return explicit resource gaps so caller can report them once.
    pub(super) fn draw(
        &self,
        facts: &SelectorFacts<'_>,
        presentation: &Presentation,
    ) -> Vec<Unavailable> {
        let scene = self.selector.view(facts, &Resources(presentation));
        draw_scene(presentation, &scene);
        scene.unavailable
    }
}

/// Caller converts viewport pointer coordinates once; no duplicate OS events.
pub(super) fn logical_point(pointer: Vec2) -> Option<Point> {
    (pointer.x.is_finite() && pointer.y.is_finite()).then(|| Point {
        x: pointer.x.floor() as i32,
        y: pointer.y.floor() as i32,
    })
}

fn draw_scene(presentation: &Presentation, scene: &SelectorScene) {
    for command in &scene.commands {
        match command {
            DrawCommand::Image(draw) => draw_image(presentation, draw),
            DrawCommand::Text(draw) => draw_text(presentation, draw),
        }
    }
}

fn rect(rect: PixelRect) -> Rect {
    Rect::new(
        rect.x as f32,
        rect.y as f32,
        rect.width as f32,
        rect.height as f32,
    )
}

fn intersection(first: Rect, second: Rect) -> Option<Rect> {
    let left = first.x.max(second.x);
    let top = first.y.max(second.y);
    let right = (first.x + first.w).min(second.x + second.w);
    let bottom = (first.y + first.h).min(second.y + second.h);
    (right > left && bottom > top).then_some(Rect::new(left, top, right - left, bottom - top))
}

fn draw_tile(texture: &Texture2D, source: Rect, destination: Rect, paint: Paint, color: Color) {
    let Some(clip) = intersection(rect(paint.clip), Rect::new(0.0, 0.0, 640.0, 480.0)) else {
        return;
    };
    let Some(clipped) = intersection(destination, clip) else {
        return;
    };
    if source.w <= 0.0 || source.h <= 0.0 || destination.w <= 0.0 || destination.h <= 0.0 {
        return;
    }
    let sx = source.w / destination.w;
    let sy = source.h / destination.h;
    let source = Rect::new(
        source.x + (clipped.x - destination.x) * sx,
        source.y + (clipped.y - destination.y) * sy,
        clipped.w * sx,
        clipped.h * sy,
    );
    draw_texture_ex(
        texture,
        clipped.x,
        clipped.y,
        color,
        DrawTextureParams {
            source: Some(source),
            dest_size: Some(vec2(clipped.w, clipped.h)),
            ..Default::default()
        },
    );
}

fn draw_image(presentation: &Presentation, draw: &ImageDraw) {
    let Some(texture) = presentation.images.get(draw.image.resource_id()) else {
        return;
    };
    let source = rect(draw.source);
    let destination = rect(draw.destination);
    let color = Color::new(1.0, 1.0, 1.0, f32::from(draw.paint.alpha) / 255.0);
    if draw.paint.blend == Blend::Additive {
        gl_use_material(&presentation.additive);
    }
    if draw.sizing == ImageSizing::ImageBox {
        image_box(texture, source, destination, draw.paint, color);
    } else {
        draw_tile(texture, source, destination, draw.paint, color);
    }
    if draw.paint.blend == Blend::Additive {
        gl_use_default_material();
    }
}

/// W1 image-box tiles source thirds; it does not stretch the button corners.
fn image_box(texture: &Texture2D, source: Rect, dest: Rect, paint: Paint, color: Color) {
    let edge_x = (source.w / 3.0).floor().min(dest.w / 2.0);
    let edge_y = (source.h / 3.0).floor().min(dest.h / 2.0);
    let src_x = [source.x, source.x + edge_x, source.x + source.w - edge_x];
    let src_y = [source.y, source.y + edge_y, source.y + source.h - edge_y];
    let src_w = [edge_x, source.w - 2.0 * edge_x, edge_x];
    let src_h = [edge_y, source.h - 2.0 * edge_y, edge_y];
    let dst_x = [dest.x, dest.x + edge_x, dest.x + dest.w - edge_x];
    let dst_y = [dest.y, dest.y + edge_y, dest.y + dest.h - edge_y];
    let dst_w = [edge_x, dest.w - 2.0 * edge_x, edge_x];
    let dst_h = [edge_y, dest.h - 2.0 * edge_y, edge_y];
    for row in 0..3 {
        for col in 0..3 {
            if src_w[col] <= 0.0 || src_h[row] <= 0.0 {
                continue;
            }
            let mut y = 0.0;
            while y < dst_h[row] {
                let height = src_h[row].min(dst_h[row] - y);
                let mut x = 0.0;
                while x < dst_w[col] {
                    let width = src_w[col].min(dst_w[col] - x);
                    draw_tile(
                        texture,
                        Rect::new(src_x[col], src_y[row], width, height),
                        Rect::new(dst_x[col] + x, dst_y[row] + y, width, height),
                        paint,
                        color,
                    );
                    x += width;
                }
                y += height;
            }
        }
    }
}

fn draw_text(presentation: &Presentation, draw: &TextDraw) {
    let Some(font) = presentation.fonts.get(font_file_name(draw.font)) else {
        return;
    };
    let color = Color::from_rgba(draw.rgb[0], draw.rgb[1], draw.rgb[2], draw.paint.alpha);
    match draw.layout {
        TextLayout::Baseline { anchor, align } => {
            let width = font.metrics.measure_width(&draw.text).unwrap_or(0) as f32;
            let x = anchor.x as f32
                - if align == HorizontalAlign::Center {
                    width / 2.0
                } else {
                    0.0
                };
            draw_line(
                presentation,
                draw.font,
                &draw.text,
                x,
                anchor.y as f32,
                color,
                draw,
            );
        }
        TextLayout::WrappedCenter {
            bounds,
            line_spacing,
        } => {
            let lines = wrap_lines(presentation, draw.font, &draw.text, bounds.width);
            let height = lines.len() as f32 * line_spacing as f32;
            let baseline = bounds.y as f32
                + (bounds.height as f32 - height) / 2.0
                + font.metrics.layers.first().map_or(0, |layer| layer.ascent) as f32;
            for (index, line) in lines.iter().enumerate() {
                let width = font.metrics.measure_width(line).unwrap_or(0) as f32;
                draw_line(
                    presentation,
                    draw.font,
                    line,
                    bounds.x as f32 + (bounds.width as f32 - width) / 2.0,
                    baseline + index as f32 * line_spacing as f32,
                    color,
                    draw,
                );
            }
        }
    }
}

fn wrap_lines(
    presentation: &Presentation,
    face: SelectorFont,
    text: &str,
    max_width: u32,
) -> Vec<String> {
    let Some(font) = presentation.fonts.get(font_file_name(face)) else {
        return Vec::new();
    };
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if !line.is_empty()
            && font
                .metrics
                .measure_width(&candidate)
                .is_ok_and(|width| width > max_width as i32)
        {
            lines.push(std::mem::take(&mut line));
            line.push_str(word);
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// Clip each atlas glyph in logical coordinates; no caller scissor state leaks.
fn draw_line(
    presentation: &Presentation,
    face: SelectorFont,
    text: &str,
    x: f32,
    baseline: f32,
    color: Color,
    draw: &TextDraw,
) {
    let Some(font) = presentation.fonts.get(font_file_name(face)) else {
        return;
    };
    let Some(clip) = intersection(rect(draw.paint.clip), Rect::new(0.0, 0.0, 640.0, 480.0)) else {
        return;
    };
    font.metrics.for_each_glyph(text, x, baseline, |glyph| {
        let Some(atlas) = font.atlases.get(glyph.layer_index) else {
            return;
        };
        let source = Rect::new(
            glyph.source.x as f32,
            glyph.source.y as f32,
            glyph.source.width as f32,
            glyph.source.height as f32,
        );
        let destination = Rect::new(glyph.x, glyph.y, source.w, source.h);
        if let Some(clipped) = intersection(destination, clip) {
            draw_texture_ex(
                atlas,
                clipped.x,
                clipped.y,
                color,
                DrawTextureParams {
                    source: Some(Rect::new(
                        source.x + clipped.x - destination.x,
                        source.y + clipped.y - destination.y,
                        clipped.w,
                        clipped.h,
                    )),
                    ..Default::default()
                },
            );
        }
    });
    if draw.underline {
        let width = font.metrics.measure_width(text).unwrap_or(0).max(0) as f32;
        if let Some(line) = intersection(Rect::new(x, baseline + 1.0, width, 1.0), clip) {
            draw_rectangle(line.x, line.y, line.w, line.h, color);
        }
    }
}
