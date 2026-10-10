//! Read-only Story and Hall of Fame presentation adapters. Controllers and
//! durable score/story policy remain in their domain modules.

use super::{
    Presentation,
    help_runtime::{fill, font_text, image, image_box, outline},
};
use crate::{
    adventure::AdventureSession,
    bubbles::{BubbleConfig, BubbleManager, BubbleRect},
    highscores::screen::{
        self as hall, HallAsset, HallColor, HallDraw, HallFont, HallMetrics, ImageFrame,
        PersonalRecords, ScoreSnapshot,
    },
    profiles::Profiles,
    story::{StoryCatalog, StoryId},
    story_screen::{
        self as story, StoryAsset, StoryAssets, StoryFont, StoryPortrait, StoryTextMetrics,
    },
    timing::StepClock,
};
use macroquad::prelude::*;

pub(super) const STORY_PORTRAITS: [StoryPortrait; 33] = StoryPortrait::ALL;
pub(super) const HALL_IMAGES: [HallAsset; 4] = [
    HallAsset::Wave,
    HallAsset::ScreenTitle,
    HallAsset::ScreenTitleHole,
    HallAsset::MainButton,
];
pub(super) const STORY_FONTS: [StoryFont; 8] = [
    StoryFont::Liddie18,
    StoryFont::Liddie15,
    StoryFont::ContinuumBold14,
    StoryFont::JungleFever17Outline,
    StoryFont::JungleFever15Outline,
    StoryFont::ContinuumBold12Outline,
    StoryFont::ContinuumBold12,
    StoryFont::JungleFever10Outline,
];
pub(super) const HALL_FONTS: [HallFont; 5] = [
    HallFont::Title,
    HallFont::Page,
    HallFont::PageOutline,
    HallFont::Heading,
    HallFont::Row,
];

pub(super) const fn story_font_file(font: StoryFont) -> &'static str {
    match font {
        StoryFont::Liddie18 => "Liddie18",
        StoryFont::Liddie15 => "Liddie15",
        StoryFont::ContinuumBold14 => "ContinuumBold14",
        StoryFont::JungleFever17Outline => "JungleFever17outline",
        StoryFont::JungleFever15Outline => "JungleFever15outline",
        StoryFont::ContinuumBold12Outline => "ContinuumBold12outline",
        StoryFont::ContinuumBold12 => "ContinuumBold12",
        StoryFont::JungleFever10Outline => "JungleFever10outline",
    }
}

pub(super) const fn hall_font_file(font: HallFont) -> &'static str {
    match font {
        HallFont::Title => "JungleFever17outline",
        HallFont::Page => "ContinuumBold14",
        HallFont::PageOutline => "ContinuumBold14outback",
        HallFont::Heading => "JungleFever12outline",
        HallFont::Row => "JungleFever10outline",
    }
}

/// The caller may supply source animation cels after decoding the owned
/// metadata. A missing multi-cel cadence stays unavailable, never a guessed loop.
pub(super) struct Metrics<'a> {
    pub presentation: &'a Presentation,
    pub wave_cel: Option<u16>,
    pub portrait_cel: Option<&'a dyn Fn(StoryPortrait, u64) -> Option<u16>>,
}

impl StoryAssets for Metrics<'_> {
    fn image_metrics(&self, asset: StoryAsset) -> Option<story::StoryImageMetrics> {
        let id = asset.resource_key();
        let image = self.presentation.images.get(id)?;
        let (columns, rows) = if asset == StoryAsset::TalkShow {
            (1, 1)
        } else {
            *self.presentation.image_cells.get(id)?
        };
        if columns == 0
            || rows == 0
            || !(image.width() as u32).is_multiple_of(columns)
            || !(image.height() as u32).is_multiple_of(rows)
        {
            return None;
        }
        Some(story::StoryImageMetrics {
            cel_width: image.width() as u32 / columns,
            cel_height: image.height() as u32 / rows,
            columns: u16::try_from(columns).ok()?,
            rows: u16::try_from(rows).ok()?,
        })
    }

    fn animation_cel(&self, portrait: StoryPortrait, updates: u64) -> Option<u16> {
        if let Some(resolve) = self.portrait_cel {
            return resolve(portrait, updates);
        }
        let id = portrait.resource_id();
        if let Some(animation) = self.presentation.image_animations.get(id) {
            return u16::try_from(animation.cel_at_time(updates)?).ok();
        }
        let (columns, rows) = *self.presentation.image_cells.get(id)?;
        (columns == 1 && rows == 1).then_some(0)
    }
}

impl StoryTextMetrics for Metrics<'_> {
    fn font_metrics(&self, font: StoryFont) -> Option<story::StoryFontMetrics> {
        let layer = self
            .presentation
            .fonts
            .get(story_font_file(font))?
            .metrics
            .layers
            .first()?;
        Some(story::StoryFontMetrics {
            height: layer.point_size as f32,
            ascent: layer.ascent as f32,
            line_advance: (layer.point_size + layer.line_spacing_offset) as f32,
        })
    }

    fn text_width(&self, font: StoryFont, text: &str) -> Option<f32> {
        self.presentation
            .fonts
            .get(story_font_file(font))?
            .metrics
            .measure_width(text)
            .ok()
            .map(|width| width as f32)
    }
}

impl HallMetrics for Metrics<'_> {
    fn text_width(&self, font: HallFont, text: &str) -> f32 {
        self.presentation
            .fonts
            .get(hall_font_file(font))
            .and_then(|font| font.metrics.measure_width(text).ok())
            .map_or(f32::NAN, |width| width as f32)
    }

    fn font_height(&self, font: HallFont) -> f32 {
        self.presentation
            .fonts
            .get(hall_font_file(font))
            .and_then(|font| font.metrics.layers.first())
            .map_or(f32::NAN, |layer| layer.point_size as f32)
    }

    fn image(&self, asset: HallAsset) -> hall::ImageMetrics {
        let Some(texture) = self.presentation.images.get(asset.resource_key()) else {
            return hall::ImageMetrics {
                width: f32::NAN,
                height: f32::NAN,
                frames: 0,
            };
        };
        let (columns, rows) = self
            .presentation
            .image_cells
            .get(asset.resource_key())
            .copied()
            .unwrap_or((0, 0));
        if columns == 0 || rows == 0 {
            return hall::ImageMetrics {
                width: f32::NAN,
                height: f32::NAN,
                frames: 0,
            };
        }
        if asset == HallAsset::Wave {
            hall::ImageMetrics {
                width: texture.width() / columns as f32,
                height: texture.height() / rows as f32,
                frames: u16::try_from(columns.saturating_mul(rows)).unwrap_or(0),
            }
        } else {
            hall::ImageMetrics {
                width: texture.width(),
                height: texture.height(),
                frames: 1,
            }
        }
    }

    fn wave_frame(&self, tick: u64) -> u16 {
        self.wave_cel
            .or_else(|| {
                self.presentation
                    .image_animations
                    .get(HallAsset::Wave.resource_key())
                    .and_then(|animation| animation.cel_at_time(tick))
                    .and_then(|cel| u16::try_from(cel).ok())
            })
            .or_else(|| {
                self.presentation
                    .image_cells
                    .get(HallAsset::Wave.resource_key())
                    .filter(|&&(columns, rows)| columns == 1 && rows == 1)
                    .map(|_| 0)
            })
            .unwrap_or(u16::MAX)
    }
}

pub(super) fn personal_records(session: &AdventureSession) -> PersonalRecords {
    let mut personal = PersonalRecords::default();
    personal.adventure_seconds[0][0] = session.progress.first_stage_best_seconds;
    for result in &session.progress.later_stage_best_seconds {
        if (1..=4).contains(&result.tank) && (1..=5).contains(&result.level) {
            personal.adventure_seconds[usize::from(result.tank - 1)]
                [usize::from(result.level - 1)] = Some(result.seconds);
        }
    }
    personal.tank5_survivors = session.progress.tank5_best_survivors;
    for tank in 0..4 {
        let money = session.time_trial_scores.personal_best[tank];
        personal.time_trial_money[tank] = (money > 0).then_some(money as u32);
        personal.challenge_seconds[tank] = session.challenge_scores.personal_best[tank]
            .and_then(|seconds| u64::try_from(seconds).ok());
    }
    personal
}

pub(super) fn hall_button_height(presentation: &Presentation) -> Option<f32> {
    let image = presentation
        .images
        .get(HallAsset::MainButton.resource_key())?;
    Some(image.height())
}

fn rgba(value: [u8; 4]) -> Color {
    Color::from_rgba(value[0], value[1], value[2], value[3])
}

fn hall_color(value: HallColor) -> Color {
    match value {
        HallColor::Rgb(red, green, blue) => Color::from_rgba(red, green, blue, 255),
        HallColor::Hsl(hue, saturation, luminance) => macroquad::color::hsl_to_rgb(
            hue as f32 / 255.0,
            saturation as f32 / 255.0,
            luminance as f32 / 255.0,
        ),
    }
}

fn story_rect(value: story::StoryRect) -> Rect {
    Rect::new(value.x, value.y, value.width, value.height)
}

fn hall_rect(value: hall::Rect) -> Rect {
    Rect::new(value.x, value.y, value.width, value.height)
}

fn sprite(
    presentation: &Presentation,
    id: &str,
    source: Rect,
    destination: Rect,
    clip: Rect,
    mirror: bool,
    additive: bool,
) {
    let Some(texture) = presentation.images.get(id) else {
        return;
    };
    if additive {
        gl_use_material(&presentation.additive);
    }
    image(texture, source, destination, clip, WHITE, mirror);
    if additive {
        gl_use_default_material();
    }
}

fn story_sprite(presentation: &Presentation, image: story::StorySprite) {
    let Some(metrics) = (Metrics {
        presentation,
        wave_cel: None,
        portrait_cel: None,
    })
    .image_metrics(image.asset) else {
        return;
    };
    let source = Rect::new(
        image.frame.column as f32 * metrics.cel_width as f32,
        image.frame.row as f32 * metrics.cel_height as f32,
        metrics.cel_width as f32,
        metrics.cel_height as f32,
    );
    sprite(
        presentation,
        image.asset.resource_key(),
        source,
        story_rect(image.destination),
        story_rect(image.clip),
        image.frame.mirror_x,
        false,
    );
}

fn story_label(presentation: &Presentation, label: &story::StoryLabel) {
    if let Some(font) = presentation.fonts.get(story_font_file(label.font)) {
        font_text(
            font,
            &label.text,
            label.baseline.x,
            label.baseline.y,
            rgba(label.color),
            story_rect(label.clip),
        );
    }
}

pub(super) fn draw_story_view(
    presentation: &Presentation,
    view: &story::StoryScreenView,
    pointer: Vec2,
    mut bubbles: impl FnMut(),
) {
    match view.background {
        story::StoryBackground::Ordinary { color, .. } => {
            draw_rectangle(0.0, 0.0, 640.0, 480.0, rgba(color));
            bubbles();
        }
        story::StoryBackground::TalkShow(image) => story_sprite(presentation, image),
    }
    story_label(presentation, &view.heading);
    if let Some(title) = &view.title {
        story_label(presentation, title);
    }
    for image in &view.portraits {
        story_sprite(presentation, *image);
    }
    if let Some(body) = &view.body {
        for line in &body.lines {
            story_label(presentation, line);
        }
    }
    for band in &view.chrome {
        let bounds = story_rect(band.bounds);
        draw_rectangle(bounds.x, bounds.y, bounds.w, bounds.h, rgba(band.color));
    }
    story_label(presentation, &view.page_counter);
    for control in &view.controls {
        if !control.visible {
            continue;
        }
        let hovered = control.bounds.contains(story::StoryPoint {
            x: pointer.x,
            y: pointer.y,
        });
        if let Some(font) = presentation.fonts.get(story_font_file(control.font)) {
            let width = font.metrics.measure_width(control.label).unwrap_or(0) as f32;
            let ascent = font.metrics.layers.first().map_or(0, |layer| layer.ascent) as f32;
            font_text(
                font,
                control.label,
                control.bounds.x + (control.bounds.width - width) / 2.0,
                control.bounds.y + (control.bounds.height + ascent) / 2.0,
                rgba(if hovered {
                    control.hover_color
                } else {
                    control.normal_color
                }),
                story_rect(control.bounds),
            );
        }
    }
    if view.catalog_unavailable || !view.asset_issues.is_empty() {
        draw_text(
            if view.catalog_unavailable {
                "Story catalog unavailable"
            } else {
                "Story art unavailable"
            },
            16.0,
            425.0,
            17.0,
            YELLOW,
        );
    }
}

pub(super) fn draw_hall_scene(
    presentation: &Presentation,
    scene: &hall::HallScene,
    mut bubbles: impl FnMut(),
) {
    for operation in &scene.ops {
        let clip = hall_rect(operation.clip);
        match &operation.draw {
            HallDraw::Fill { bounds, color } => {
                let bounds = hall_rect(*bounds);
                fill(bounds, clip, hall_color(*color));
            }
            HallDraw::CallerBubbles => bubbles(),
            HallDraw::Image {
                asset,
                frame,
                bounds,
                additive,
            } => {
                let Some(texture) = presentation.images.get(asset.resource_key()) else {
                    continue;
                };
                let (columns, rows) = presentation
                    .image_cells
                    .get(asset.resource_key())
                    .copied()
                    .unwrap_or((1, 1));
                if columns == 0 || rows == 0 {
                    continue;
                }
                let cell = Rect::new(
                    0.0,
                    0.0,
                    texture.width() / columns as f32,
                    texture.height() / rows as f32,
                );
                match frame {
                    ImageFrame::Cel(index) => {
                        let source = Rect::new(
                            (*index as u32 % columns) as f32 * cell.w,
                            (*index as u32 / columns) as f32 * cell.h,
                            cell.w,
                            cell.h,
                        );
                        sprite(
                            presentation,
                            asset.resource_key(),
                            source,
                            hall_rect(*bounds),
                            clip,
                            false,
                            *additive,
                        );
                    }
                    ImageFrame::ImageBox => image_box(
                        texture,
                        Rect::new(0.0, 0.0, texture.width(), texture.height()),
                        hall_rect(*bounds),
                        clip,
                    ),
                }
            }
            HallDraw::Text(run) => {
                let color = hall_color(run.color);
                if let Some(outline) = run
                    .outline
                    .and_then(|font| presentation.fonts.get(hall_font_file(font)))
                {
                    font_text(
                        outline,
                        &run.text,
                        run.baseline.x,
                        run.baseline.y,
                        BLACK,
                        clip,
                    );
                }
                if let Some(font) = presentation.fonts.get(hall_font_file(run.font)) {
                    font_text(font, &run.text, run.baseline.x, run.baseline.y, color, clip);
                }
            }
            HallDraw::Button(button) => {
                let bounds = hall_rect(button.bounds);
                if let Some(texture) = presentation.images.get(button.asset.resource_key()) {
                    image_box(
                        texture,
                        Rect::new(0.0, 0.0, texture.width(), texture.height()),
                        bounds,
                        clip,
                    );
                }
                let stroke = if button.pressed {
                    ORANGE
                } else if button.hovered || button.focused {
                    YELLOW
                } else {
                    WHITE
                };
                if button.selected || button.pressed || button.hovered || button.focused {
                    outline(bounds, clip, 2.0, stroke);
                }
                if let Some(font) = presentation.fonts.get(hall_font_file(button.font)) {
                    let width = font
                        .metrics
                        .measure_width(button.control.label())
                        .unwrap_or(0) as f32;
                    font_text(
                        font,
                        button.control.label(),
                        bounds.x + (bounds.w - width) / 2.0,
                        bounds.y + bounds.h * 0.7,
                        WHITE,
                        clip,
                    );
                }
            }
        }
    }
}

/// Screen-local presentation state. It never owns a profile, Board, score,
/// catalog, or gameplay RNG; the caller retains all transition authority.
pub(super) struct RecordsRuntime {
    story: Option<story::StoryScreenController>,
    hall: Option<hall::HallOfFame>,
    remembered_story: StoryId,
    remembered_hall_page: hall::Page,
    story_bubbles: BubbleManager,
    story_bubble_spec: Option<story::StoryBubbleSpec>,
    hall_bubbles: BubbleManager,
    clock: StepClock,
    hall_ticks: u64,
    fx_seed: u64,
}

impl RecordsRuntime {
    pub(super) fn new(fx_seed: u64) -> Self {
        Self {
            story: None,
            hall: None,
            remembered_story: StoryId::new(0).expect("story zero is valid"),
            remembered_hall_page: hall::Page::TimeTrial,
            story_bubbles: BubbleManager::new(BubbleConfig::default()),
            story_bubble_spec: None,
            hall_bubbles: BubbleManager::new(hall::bubble_config()),
            clock: StepClock::new(1),
            hall_ticks: 0,
            fx_seed,
        }
    }

    pub(super) fn current_story(&self) -> Option<StoryId> {
        self.story
            .as_ref()
            .map(story::StoryScreenController::current)
    }

    pub(super) fn remembered_story(&self) -> StoryId {
        self.remembered_story
    }

    pub(super) fn hall_open(&self) -> bool {
        self.hall.is_some()
    }

    pub(super) fn open_story(&mut self, session: &AdventureSession, preferred: StoryId) -> bool {
        let Some(browser) = session.progress.stories.open_browser(preferred) else {
            return false;
        };
        self.close_hall();
        self.story = Some(story::StoryScreenController::new(browser));
        self.story_bubbles = BubbleManager::new(BubbleConfig::default());
        self.story_bubble_spec = None;
        self.clock.clear_backlog();
        true
    }

    /// Challenge owns the earned page in the Session. This only projects it.
    pub(super) fn sync_challenge_story(&mut self, session: &AdventureSession, id: StoryId) -> bool {
        if !session.progress.stories.is_unlocked(id) {
            return false;
        }
        let Some(browser) = session.progress.stories.open_browser(id) else {
            return false;
        };
        if let Some(controller) = &mut self.story {
            controller.sync_browser(browser);
        } else {
            self.open_story(session, id);
        }
        true
    }

    pub(super) fn close_story(&mut self) {
        if let Some(controller) = self.story.take() {
            self.remembered_story = controller.current();
        }
        self.story_bubbles.clear();
        self.story_bubble_spec = None;
        self.clock.clear_backlog();
    }

    pub(super) fn open_hall(&mut self) {
        self.close_story();
        self.hall = Some(hall::HallOfFame::new(self.remembered_hall_page));
        self.hall_bubbles = BubbleManager::new(hall::bubble_config());
        let seed = &mut self.fx_seed;
        self.hall_bubbles
            .update_a_lot(&mut |upper| fx_roll(seed, upper));
        self.hall_ticks = 0;
        self.clock.clear_backlog();
    }

    pub(super) fn close_hall(&mut self) {
        if let Some(screen) = self.hall.take() {
            self.remembered_hall_page = screen.page();
        }
        self.hall_bubbles.clear();
        self.clock.clear_backlog();
    }

    pub(super) fn input_story(
        &mut self,
        presentation: &Presentation,
        session: &AdventureSession,
        catalog: Option<&StoryCatalog>,
        input: story::StoryInput,
        focused: bool,
    ) -> story::StoryIntent {
        let Some(controller) = &mut self.story else {
            return story::StoryIntent::default();
        };
        if !focused {
            controller.advance(0, false, story::StoryClockPolicy::FocusedOnly);
            return story::StoryIntent::default();
        }
        let metrics = Metrics {
            presentation,
            wave_cel: None,
            portrait_cel: None,
        };
        let view = match controller.view(
            &session.progress.stories,
            catalog.map(|catalog| catalog.entry(controller.current())),
            &metrics,
            &metrics,
        ) {
            Ok(view) => view,
            Err(_) => return story::StoryIntent::default(),
        };
        let intent = controller.input(input, &session.progress.stories, &view);
        self.remembered_story = controller.current();
        intent
    }

    pub(super) fn input_hall(
        &mut self,
        presentation: &Presentation,
        input: hall::HallInput,
        focused: bool,
    ) -> hall::Intentions {
        let Some(screen) = &mut self.hall else {
            return hall::Intentions::default();
        };
        if !focused {
            screen.input(hall::HallInput::CancelPress, 1.0);
            return hall::Intentions::default();
        }
        let Some(button_height) = hall_button_height(presentation) else {
            return hall::Intentions::default();
        };
        let intent = screen.input(input, button_height);
        self.remembered_hall_page = screen.page();
        intent
    }

    /// Poll native input once for the active Story surface. Down and Up stay
    /// separate so the model's captured-control rule decides activation.
    pub(super) fn input_story_frame(
        &mut self,
        presentation: &Presentation,
        session: &AdventureSession,
        catalog: Option<&StoryCatalog>,
        pointer: Vec2,
        focused: bool,
    ) -> Vec<story::StoryIntent> {
        if self.story.is_none() {
            return Vec::new();
        }
        if !focused {
            self.input_story(
                presentation,
                session,
                catalog,
                story::StoryInput::CancelPress,
                false,
            );
            return Vec::new();
        }
        let mut inputs = Vec::new();
        if is_key_pressed(KeyCode::Escape) {
            inputs.push(story::StoryInput::Activate(story::StoryControlId::Menu));
        } else {
            let point = story::StoryPoint {
                x: pointer.x,
                y: pointer.y,
            };
            if is_mouse_button_pressed(MouseButton::Left) {
                inputs.push(story::StoryInput::PointerDown(point));
            }
            if is_mouse_button_released(MouseButton::Left) {
                inputs.push(story::StoryInput::PointerUp(point));
            }
            if is_key_pressed(KeyCode::Left) {
                inputs.push(story::StoryInput::Activate(story::StoryControlId::Previous));
            } else if is_key_pressed(KeyCode::Right) {
                inputs.push(story::StoryInput::Activate(story::StoryControlId::Next));
            }
            let wheel = mouse_wheel().1;
            if wheel > 0.0 {
                inputs.push(story::StoryInput::ScrollLines(-3));
            } else if wheel < 0.0 {
                inputs.push(story::StoryInput::ScrollLines(3));
            }
            if is_key_pressed(KeyCode::Up) {
                inputs.push(story::StoryInput::ScrollLines(-3));
            } else if is_key_pressed(KeyCode::Down) {
                inputs.push(story::StoryInput::ScrollLines(3));
            }
        }
        let mut intents = Vec::new();
        for input in inputs {
            let intent = self.input_story(presentation, session, catalog, input, true);
            if intent != story::StoryIntent::default() {
                intents.push(intent);
            }
            if intent.action.is_some() {
                break;
            }
        }
        intents
    }

    /// Hover and press/release use the Hall controller's own hit test and
    /// MainButton metric. Keyboard focus remains controller-owned.
    pub(super) fn input_hall_frame(
        &mut self,
        presentation: &Presentation,
        pointer: Vec2,
        focused: bool,
    ) -> Vec<hall::Intentions> {
        if self.hall.is_none() {
            return Vec::new();
        }
        if !focused {
            self.input_hall(presentation, hall::HallInput::CancelPress, false);
            return Vec::new();
        }
        let mut inputs = Vec::new();
        if is_key_pressed(KeyCode::Escape) {
            inputs.push(hall::HallInput::Escape);
        } else {
            let point = hall::Point {
                x: pointer.x,
                y: pointer.y,
            };
            inputs.push(hall::HallInput::PointerMove(point));
            if is_mouse_button_pressed(MouseButton::Left) {
                inputs.push(hall::HallInput::PointerPress(point));
            }
            if is_mouse_button_released(MouseButton::Left) {
                inputs.push(hall::HallInput::PointerRelease(point));
            }
            if is_key_pressed(KeyCode::Tab) {
                inputs.push(hall::HallInput::FocusNext {
                    backward: is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift),
                });
            }
            if is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::Space) {
                inputs.push(hall::HallInput::ActivateFocused);
            }
        }
        let mut intents = Vec::new();
        for input in inputs {
            let intent = self.input_hall(presentation, input, true);
            if intent != hall::Intentions::default() {
                intents.push(intent);
            }
            if intent.exit_to_menu {
                break;
            }
        }
        intents
    }

    /// One-timescale, active-screen-only presentation clock. The parent never
    /// forwards these ticks to a retained Adventure/Virtual Board.
    pub(super) fn advance(&mut self, frame_seconds: f32, focused: bool) {
        if !focused || (self.story.is_none() && self.hall.is_none()) {
            self.clock.clear_backlog();
            if let Some(story) = &mut self.story {
                story.advance(0, false, story::StoryClockPolicy::FocusedOnly);
            }
            if let Some(hall) = &mut self.hall {
                hall.input(hall::HallInput::CancelPress, 1.0);
            }
            return;
        }
        self.clock.add_frame(frame_seconds, false);
        while self.clock.step_due() {
            self.clock.consume_step(false);
            if let Some(story) = &mut self.story {
                let update = story.advance(1, true, story::StoryClockPolicy::FocusedOnly);
                let seed = &mut self.fx_seed;
                for _ in 0..update.bubble_updates {
                    if self.story_bubble_spec.is_some() {
                        self.story_bubbles.update(&mut |upper| fx_roll(seed, upper));
                    }
                }
            } else if self.hall.is_some() {
                self.hall_ticks = self.hall_ticks.saturating_add(1);
                let seed = &mut self.fx_seed;
                self.hall_bubbles.update(&mut |upper| fx_roll(seed, upper));
            }
        }
    }

    fn prepare_story_bubbles(&mut self, background: story::StoryBackground) {
        let story::StoryBackground::Ordinary { bubbles, .. } = background else {
            return;
        };
        if self.story_bubble_spec.is_some() {
            return;
        }
        self.story_bubbles.set_bubble_bounds(BubbleRect::new(
            bubbles.bounds.x as i32,
            bubbles.bounds.y as i32,
            bubbles.bounds.width as i32,
            bubbles.bounds.height as i32,
        ));
        self.story_bubbles
            .set_bubble_config(bubbles.max_bubbles as usize, bubbles.spawn_chance as i32);
        if bubbles.prewarm_once {
            let seed = &mut self.fx_seed;
            self.story_bubbles
                .update_a_lot(&mut |upper| fx_roll(seed, upper));
        }
        self.story_bubble_spec = Some(bubbles);
    }

    pub(super) fn draw_story(
        &mut self,
        presentation: &Presentation,
        session: &AdventureSession,
        catalog: Option<&StoryCatalog>,
        pointer: Vec2,
    ) {
        let Some(controller) = &self.story else {
            return;
        };
        let metrics = Metrics {
            presentation,
            wave_cel: None,
            portrait_cel: None,
        };
        let view = match controller.view(
            &session.progress.stories,
            catalog.map(|catalog| catalog.entry(controller.current())),
            &metrics,
            &metrics,
        ) {
            Ok(view) => view,
            Err(error) => {
                draw_text(
                    format!("Story layout unavailable: {error}").as_str(),
                    20.0,
                    230.0,
                    18.0,
                    YELLOW,
                );
                return;
            }
        };
        self.prepare_story_bubbles(view.background);
        draw_story_view(presentation, &view, pointer, || {
            for op in self.story_bubbles.render_ops() {
                super::bubble_runtime::draw_op(presentation, op);
            }
        });
    }

    pub(super) fn draw_hall(&self, presentation: &Presentation, profiles: &Profiles) {
        let Some(screen) = &self.hall else { return };
        let personal = profiles
            .current()
            .map_or_else(PersonalRecords::default, |profile| {
                personal_records(&profile.session)
            });
        let snapshot = ScoreSnapshot {
            global: &profiles.global_scores,
            personal: &personal,
            finished_game: profiles
                .current()
                .is_some_and(|profile| profile.session.progress.adventure_completed),
        };
        let metrics = Metrics {
            presentation,
            wave_cel: None,
            portrait_cel: None,
        };
        let scene = match screen.view(
            snapshot,
            &metrics,
            hall::HallClock {
                tick: self.hall_ticks,
                bob_text: false,
            },
        ) {
            Ok(scene) => scene,
            Err(error) => {
                draw_text(
                    format!("Hall of Fame unavailable: {error}").as_str(),
                    20.0,
                    230.0,
                    18.0,
                    YELLOW,
                );
                return;
            }
        };
        draw_hall_scene(presentation, &scene, || {
            for op in self.hall_bubbles.render_ops() {
                super::bubble_runtime::draw_op(presentation, op);
            }
        });
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
