//! App projection for the transient four-card Time Trial/Challenge chooser.
//! The controller owns captures; AdventureSession remains the eligibility and
//! action authority.

use super::{Presentation, RenderedFont};
use crate::{
    adventure::{AdventurePhase, AdventureSession},
    music::{MusicOwner, MusicReport},
    sim::{Action, Event},
    tank_selection::{
        DrawKind, ImageDraw, ImageMetadata, ImageProjection, Rect as TankRect, SceneUnavailable,
        SelectionMode, TankFacts, TankFont, TankImage, TankInput, TankIntent, TankResources,
        TankScene, TankSelection, TextDraw, TextPlacement,
    },
};
use macroquad::prelude::*;

pub(super) const REQUIRED_IMAGE_IDS: &[&str] = &[
    "IMAGE_HORZCREASE",
    "IMAGE_VERTCREASE",
    "IMAGE_BOLT",
    "IMAGE_SCREENTITLEHOLE",
    "IMAGE_TANKCHOOSER",
    "IMAGE_TANKBACKDROP1",
    "IMAGE_TANKBACKDROP2",
    "IMAGE_TANKBACKDROP4",
    "IMAGE_TANKBACKDROP5",
    "IMAGE_DIALOGBUTTON",
];

impl TankResources for Presentation {
    fn image(&self, image: TankImage) -> Option<ImageMetadata> {
        let id = image.resource_key();
        let texture = self.images.get(id)?;
        let &(cols, rows) = self.image_cells.get(id)?;
        let width = texture.width();
        let height = texture.height();
        if !width.is_finite()
            || !height.is_finite()
            || width.fract() != 0.0
            || height.fract() != 0.0
            || width < 0.0
            || height < 0.0
        {
            return None;
        }
        Some(ImageMetadata {
            width: width as u32,
            height: height as u32,
            cols,
            rows,
        })
    }

    fn font(&self, font: TankFont) -> Option<&crate::font::BitmapFont> {
        self.fonts
            .get(font.script_stem())
            .map(|rendered| &rendered.metrics)
    }
}

pub(super) fn mode_for_phase(phase: &AdventurePhase) -> Option<SelectionMode> {
    match phase {
        AdventurePhase::TimeTrialTankSelection => Some(SelectionMode::TimeTrial),
        AdventurePhase::ChallengeTankSelection => Some(SelectionMode::Challenge),
        _ => None,
    }
}

/// Snapshot current authoritative progress before each controller input.
pub(super) fn facts(session: &AdventureSession, mode: SelectionMode) -> TankFacts {
    let progress = &session.progress;
    let available = match mode {
        SelectionMode::TimeTrial => {
            std::array::from_fn(|index| session.time_trial_tank_available((index + 1) as u8))
        }
        SelectionMode::Challenge => [
            progress.adventure_completed,
            progress.challenge_completed[0],
            progress.challenge_completed[1],
            progress.challenge_completed[2],
        ],
    };
    TankFacts {
        available,
        has_ordinary_stories: progress.stories.seen_mask != 0,
    }
}

/// Model input stays typed. Caller feeds move/down/up and focus/modal/cancel;
/// action and sound intents are returned without applying game state here.
pub(super) fn input(
    chooser: &mut TankSelection,
    presentation: &Presentation,
    event: TankInput,
) -> Result<Vec<TankIntent>, SceneUnavailable> {
    chooser.input(event, presentation)
}

/// Only selection and Menu map to existing session actions. Stories remains
/// the app's earned Story-browser route, and click audio stays separate.
pub(super) fn action_for_intent(intent: TankIntent) -> Option<Action> {
    match intent {
        TankIntent::SelectTank { mode, tank } => Some(match mode {
            SelectionMode::TimeTrial => Action::SelectTimeTrialTank {
                tank: tank.number(),
            },
            SelectionMode::Challenge => Action::SelectChallengeTank {
                tank: tank.number(),
            },
        }),
        TankIntent::Menu => Some(Action::OpenMenu),
        TankIntent::Stories | TankIntent::PlayButtonSound(_) => None,
    }
}

/// Consume this chooser frame outside the fixed-step pause gate. Only this
/// mode's choices and Menu may reach Session; eligibility remains authoritative
/// there. A transition consumes the rest of the frame without applying it to
/// the newly opened screen. Blocked input is discarded rather than deferred.
pub(super) fn dispatch_actions(
    session: &mut AdventureSession,
    actions: &mut Vec<Action>,
    focused: bool,
    blocked: bool,
) -> (Vec<Event>, bool) {
    let actions = std::mem::take(actions);
    let Some(mode) = mode_for_phase(&session.phase) else {
        return (Vec::new(), false);
    };
    if !focused || blocked {
        return (Vec::new(), false);
    }
    let mut events = Vec::new();
    for action in actions {
        if !matches!(
            (mode, &action),
            (_, Action::OpenMenu)
                | (SelectionMode::TimeTrial, Action::SelectTimeTrialTank { .. })
                | (SelectionMode::Challenge, Action::SelectChallengeTank { .. })
        ) {
            continue;
        }
        events.extend(session.apply_actions(std::slice::from_ref(&action)));
        if mode_for_phase(&session.phase) != Some(mode) {
            return (events, true);
        }
    }
    (events, false)
}

/// One sound request is emitted on the model's first valid pointer down.
pub(super) fn play_intent_sound(
    intent: TankIntent,
    presentation: &Presentation,
    music: Option<&MusicOwner>,
) -> Option<MusicReport> {
    let TankIntent::PlayButtonSound(sound) = intent else {
        return None;
    };
    presentation
        .sounds
        .get(sound.resource_key())
        .zip(music)
        .and_then(|(bytes, owner)| owner.play_effect(bytes.clone()))
}

pub(super) fn draw(
    presentation: &Presentation,
    chooser: &TankSelection,
) -> Result<TankScene, SceneUnavailable> {
    let scene = chooser.scene(presentation)?;
    clear_background(BLACK);
    for command in &scene.commands {
        match &command.kind {
            DrawKind::Image(image) => draw_image(presentation, *image)?,
            DrawKind::Text(text) => draw_text_command(presentation, text)?,
        }
    }
    Ok(scene)
}

fn draw_image(presentation: &Presentation, image: ImageDraw) -> Result<(), SceneUnavailable> {
    let id = image.resource.resource_key();
    let texture = presentation
        .images
        .get(id)
        .ok_or(SceneUnavailable::MissingImage(image.resource))?;
    let source = rect(image.crop);
    let destination = rect(image.destination);
    match image.projection {
        ImageProjection::ImageBox => presentation.image_box(id, source, destination),
        ImageProjection::Natural => {
            let flip_x = image.resource == TankImage::Bolt && image.destination.x > 320;
            let flip_y = image.resource == TankImage::Bolt && image.destination.y >= 322;
            draw_unscaled(
                texture,
                source,
                destination.x,
                destination.y,
                flip_x,
                flip_y,
            );
        }
        ImageProjection::HorizontalTile => {
            let mut used = 0.0;
            while used < destination.w {
                let width = source.w.min(destination.w - used);
                draw_unscaled(
                    texture,
                    Rect::new(source.x, source.y, width, source.h),
                    destination.x + used,
                    destination.y,
                    false,
                    false,
                );
                used += width;
            }
        }
        ImageProjection::VerticalTile => {
            let mut used = 0.0;
            while used < destination.h {
                let height = source.h.min(destination.h - used);
                draw_unscaled(
                    texture,
                    Rect::new(source.x, source.y, source.w, height),
                    destination.x,
                    destination.y + used,
                    false,
                    false,
                );
                used += height;
            }
        }
    }
    Ok(())
}

fn draw_unscaled(texture: &Texture2D, source: Rect, x: f32, y: f32, flip_x: bool, flip_y: bool) {
    let Some((x, y, source)) =
        crop_source(x, y, source, TankRect::new(0, 0, 640, 480), flip_x, flip_y)
    else {
        return;
    };
    draw_texture_ex(
        texture,
        x,
        y,
        WHITE,
        DrawTextureParams {
            source: Some(source),
            dest_size: Some(vec2(source.w, source.h)),
            flip_x,
            flip_y,
            ..Default::default()
        },
    );
}

fn rect(source: TankRect) -> Rect {
    Rect::new(
        source.x as f32,
        source.y as f32,
        source.width as f32,
        source.height as f32,
    )
}

fn draw_text_command(presentation: &Presentation, text: &TextDraw) -> Result<(), SceneUnavailable> {
    let font = presentation
        .fonts
        .get(text.font.script_stem())
        .ok_or(SceneUnavailable::MissingFont(text.font))?;
    let [r, g, b, a] = text.color;
    let tint = Color::from_rgba(r, g, b, a);
    match text.placement {
        TextPlacement::Baseline { x, y } => draw_font_clipped(
            font,
            &text.text,
            x as f32,
            y as f32,
            tint,
            TankRect::new(0, 0, 640, 480),
        ),
        TextPlacement::WrappedCentered { bounds } => {
            let lines = wrap_lines(font, &text.text, bounds.width as i32, text.font)?;
            let layer = &font.metrics.layers[0];
            let line_height = layer
                .ascent
                .saturating_add(layer.line_spacing_offset)
                .max(1);
            for (index, line) in lines.iter().enumerate() {
                let baseline = bounds
                    .y
                    .saturating_add(layer.ascent)
                    .saturating_add((index as i32).saturating_mul(line_height));
                if baseline > bounds.y + bounds.height as i32 {
                    break;
                }
                let width = font
                    .metrics
                    .measure_width(line)
                    .map_err(|error| SceneUnavailable::InvalidFont(text.font, error.to_string()))?;
                let x = bounds.x + (bounds.width as i32 - width) / 2;
                draw_font_clipped(font, line, x as f32, baseline as f32, tint, bounds);
            }
        }
    }
    Ok(())
}

fn wrap_lines(
    font: &RenderedFont,
    text: &str,
    width: i32,
    identity: TankFont,
) -> Result<Vec<String>, SceneUnavailable> {
    let mut lines = Vec::new();
    for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            let proposed = if line.is_empty() {
                word.to_owned()
            } else {
                format!("{line} {word}")
            };
            let measured = font
                .metrics
                .measure_width(&proposed)
                .map_err(|error| SceneUnavailable::InvalidFont(identity, error.to_string()))?;
            if measured <= width || line.is_empty() {
                line = proposed;
            } else {
                lines.push(std::mem::take(&mut line));
                line = word.to_owned();
            }
        }
        lines.push(line);
    }
    Ok(lines)
}

fn draw_font_clipped(
    font: &RenderedFont,
    text: &str,
    x: f32,
    baseline: f32,
    tint: Color,
    clip: TankRect,
) {
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
        if let Some((draw_x, draw_y, source)) =
            crop_source(glyph.x, glyph.y, source, clip, false, false)
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
    });
}

fn crop_source(
    x: f32,
    y: f32,
    mut source: Rect,
    clip: TankRect,
    flip_x: bool,
    flip_y: bool,
) -> Option<(f32, f32, Rect)> {
    let left = x.max(clip.x as f32);
    let top = y.max(clip.y as f32);
    let right = (x + source.w).min(clip.x as f32 + clip.width as f32);
    let bottom = (y + source.h).min(clip.y as f32 + clip.height as f32);
    if right <= left || bottom <= top {
        return None;
    }
    source.x += if flip_x {
        x + source.w - right
    } else {
        left - x
    };
    source.y += if flip_y {
        y + source.h - bottom
    } else {
        top - y
    };
    source.w = right - left;
    source.h = bottom - top;
    Some((left, top, source))
}
