//! Transient title bootstrap and presentation. No profile or Board authority.

use crate::{
    assets::{DecodedImage, GameAssets},
    timing::{StepClock, wall_deadline_reached},
    title_screen::{
        Dimensions, FinishTarget, ImageDraw, LoadingReport, LoadingStatus, MaskedTitleDraw,
        PixelRect, TitleAssets, TitleEvent, TitleImage, TitleLayer, TitleMode, TitleScreen,
    },
};
use macroquad::prelude::*;
use std::{collections::HashMap, error::Error};

const VIEW_WIDTH: f32 = 640.0;
const VIEW_HEIGHT: f32 = 480.0;
const MASK_X: u32 = 384;
const MASK_Y: u32 = 388;

#[derive(Clone)]
struct LoadedImage {
    texture: Texture2D,
    cells: (u32, u32),
}

/// Live loading/title screen. Seven bootstrap uploads are actual completed
/// tasks; the caller includes them in `total_tasks` and reports each further
/// required image, font, sound or backend only after success.
pub(super) struct TitleRuntime {
    title: TitleScreen,
    images: HashMap<&'static str, LoadedImage>,
    masked_title: Texture2D,
    total_tasks: u64,
    completed_tasks: u64,
    clock: StepClock,
    pending_finish: Option<FinishTarget>,
    app_started_at: f64,
    quit_after: Option<f64>,
}

impl TitleRuntime {
    pub(super) fn bootstrap(
        assets: &GameAssets,
        mode: TitleMode,
        total_tasks: u64,
        app_started_at: f64,
        quit_after: Option<f64>,
    ) -> Result<Self, Box<dyn Error>> {
        if total_tasks < TitleImage::ALL.len() as u64 {
            return Err("title loading total excludes seven required bootstrap images".into());
        }
        if wall_deadline_reached(get_time() - app_started_at, quit_after) {
            return Err("title startup reached --quit-after before bootstrap".into());
        }
        let mut decoded = HashMap::<&'static str, DecodedImage>::new();
        for image in TitleImage::ALL {
            let id = image.resource_id();
            decoded.insert(id, assets.load_image(id)?);
            if wall_deadline_reached(get_time() - app_started_at, quit_after) {
                return Err("title startup reached --quit-after during bootstrap".into());
            }
        }
        let dimensions = |image: TitleImage| {
            let decoded = &decoded[image.resource_id()];
            Dimensions {
                width: decoded.width,
                height: decoded.height,
            }
        };
        let metadata = TitleAssets {
            title_page: dimensions(TitleImage::TitlePage),
            loader_bar: dimensions(TitleImage::LoaderBar),
            loader_bar_over: dimensions(TitleImage::LoaderBarOver),
            loading_label: dimensions(TitleImage::LoadingLabel),
            play_label: dimensions(TitleImage::PlayLabel),
            stinky: dimensions(TitleImage::Stinky),
            title_page_mask: dimensions(TitleImage::TitlePageMask),
        };
        let mut title = TitleScreen::new(mode, VIEW_WIDTH as u32, metadata)?;
        title.report_loading(LoadingReport {
            completed_tasks: TitleImage::ALL.len() as u64,
            total_tasks,
            status: LoadingStatus::Running,
        })?;
        let masked_title = compose_title_mask(
            &decoded[TitleImage::TitlePage.resource_id()],
            &decoded[TitleImage::TitlePageMask.resource_id()],
        )?;
        let mut images = HashMap::new();
        for image in TitleImage::ALL {
            let id = image.resource_id();
            let decoded = decoded.remove(id).expect("decoded bootstrap image");
            let texture = Texture2D::from_rgba8(
                u16::try_from(decoded.width)?,
                u16::try_from(decoded.height)?,
                &decoded.pixels,
            );
            texture.set_filter(FilterMode::Linear);
            images.insert(
                id,
                LoadedImage {
                    texture,
                    cells: (decoded.cols, decoded.rows),
                },
            );
        }
        Ok(Self {
            title,
            images,
            masked_title,
            total_tasks,
            completed_tasks: TitleImage::ALL.len() as u64,
            clock: StepClock::new(1),
            pending_finish: None,
            app_started_at,
            quit_after,
        })
    }

    /// Clone GPU handle and real atlas dimensions into Presentation; no second
    /// disk decode or duplicate upload is needed for shared Stinky/title art.
    pub(super) fn bootstrap_image(&self, id: &str) -> Option<(Texture2D, (u32, u32))> {
        self.images
            .get(id)
            .map(|image| (image.texture.clone(), image.cells))
    }

    pub(super) fn check_deadline(&self) -> Result<(), Box<dyn Error>> {
        if is_quit_requested()
            || wall_deadline_reached(get_time() - self.app_started_at, self.quit_after)
        {
            Err("title startup canceled before gameplay or saver entry".into())
        } else {
            Ok(())
        }
    }

    /// Call only after one real required task completed successfully. Drawing
    /// and frame yielding are cooperative; no sleep or fabricated task count.
    pub(super) async fn task_completed(&mut self) -> Result<(), Box<dyn Error>> {
        self.check_deadline()?;
        let completed = self.completed_tasks.saturating_add(1);
        if completed > self.total_tasks {
            return Err("title loader reported more tasks than planned".into());
        }
        self.title.report_loading(LoadingReport {
            completed_tasks: completed,
            total_tasks: self.total_tasks,
            status: LoadingStatus::Running,
        })?;
        self.completed_tasks = completed;
        self.draw_and_yield().await
    }

    /// Caller asserts all required resources/backends are usable. Model
    /// rejects any incomplete task plan before admitting the Start button.
    pub(super) fn succeed(&mut self) -> Result<(), Box<dyn Error>> {
        self.check_deadline()?;
        self.title.report_loading(LoadingReport {
            completed_tasks: self.completed_tasks,
            total_tasks: self.total_tasks,
            status: LoadingStatus::Succeeded,
        })?;
        Ok(())
    }

    /// Terminal failure: title can neither start nor later be marked ready.
    pub(super) fn fail(&mut self) -> Result<(), Box<dyn Error>> {
        self.title.report_loading(LoadingReport {
            completed_tasks: self.completed_tasks,
            total_tasks: self.total_tasks,
            status: LoadingStatus::Failed,
        })?;
        Ok(())
    }

    /// Cooperative frame between long caller-owned tasks. UI uses a separate
    /// 1x 28 ms clock and never advances the gameplay session.
    pub(super) async fn draw_and_yield(&mut self) -> Result<(), Box<dyn Error>> {
        self.check_deadline()?;
        self.clock.add_frame(get_frame_time(), false);
        while self.clock.step_due() {
            self.clock.consume_step(false);
            if let Some(TitleEvent::Finished(target)) = self.title.advance() {
                self.pending_finish = Some(target);
            }
        }
        self.draw();
        next_frame().await;
        self.check_deadline()
    }

    /// Ordinary title waits for captured Start release; saver auto-finishes
    /// after its source walk. Play the press cue at the actual press boundary.
    pub(super) async fn wait_start(
        &mut self,
        presentation: &super::Presentation,
        music: Option<&crate::music::MusicOwner>,
    ) -> Result<FinishTarget, Box<dyn Error>> {
        loop {
            self.check_deadline()?;
            if let Some(target) = self.pending_finish.take() {
                return Ok(target);
            }
            let scale = (screen_width() / VIEW_WIDTH).min(screen_height() / VIEW_HEIGHT);
            if scale <= 0.0 {
                return Err("title viewport has no drawable area".into());
            }
            let offset = vec2(
                (screen_width() - VIEW_WIDTH * scale) / 2.0,
                (screen_height() - VIEW_HEIGHT * scale) / 2.0,
            );
            let (mouse_x, mouse_y) = mouse_position();
            let pointer = (vec2(mouse_x, mouse_y) - offset) / scale;
            self.title
                .pointer_move(f64::from(pointer.x), f64::from(pointer.y));
            if is_mouse_button_pressed(MouseButton::Left)
                && self
                    .title
                    .pointer_press(f64::from(pointer.x), f64::from(pointer.y))
                    == Some(TitleEvent::StartPressed)
                && let Some(owner) = music
            {
                let click = presentation
                    .sounds
                    .get("SOUND_BUTTONCLICK")
                    .ok_or("title click sound missing after successful loading")?;
                if let Some(report) = owner.play_effect(click.clone()) {
                    eprintln!("Title click sound: {report:?}");
                }
            }
            if is_mouse_button_released(MouseButton::Left)
                && let Some(TitleEvent::Finished(target)) = self
                    .title
                    .pointer_release(f64::from(pointer.x), f64::from(pointer.y))
            {
                return Ok(target);
            }
            self.clock.add_frame(get_frame_time(), false);
            while self.clock.step_due() {
                self.clock.consume_step(false);
                if let Some(TitleEvent::Finished(target)) = self.title.advance() {
                    return Ok(target);
                }
            }
            self.draw();
            next_frame().await;
        }
    }

    fn draw(&self) {
        clear_background(BLACK);
        let scale = (screen_width() / VIEW_WIDTH).min(screen_height() / VIEW_HEIGHT);
        let offset = vec2(
            (screen_width() - VIEW_WIDTH * scale) / 2.0,
            (screen_height() - VIEW_HEIGHT * scale) / 2.0,
        );
        let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, VIEW_WIDTH, VIEW_HEIGHT));
        camera.zoom.y = 2.0 / VIEW_HEIGHT;
        camera.viewport = Some((
            offset.x as i32,
            offset.y as i32,
            (VIEW_WIDTH * scale) as i32,
            (VIEW_HEIGHT * scale) as i32,
        ));
        set_camera(&camera);
        for layer in self.title.view().layers {
            match layer {
                TitleLayer::Image(draw) => self.draw_image(draw),
                TitleLayer::MaskedTitle(draw) => self.draw_masked_title(draw),
            }
        }
        set_default_camera();
    }

    fn draw_image(&self, draw: ImageDraw) {
        let Some(image) = self.images.get(draw.image.resource_id()) else {
            return;
        };
        let source = rect(draw.source);
        let destination = rect(draw.destination);
        let Some((source, destination)) = crop_layer(source, destination, draw.clip, draw.mirror_x)
        else {
            return;
        };
        draw_texture_ex(
            &image.texture,
            destination.x,
            destination.y,
            Color::new(1.0, 1.0, 1.0, f32::from(draw.alpha) / 255.0),
            DrawTextureParams {
                source: Some(source),
                dest_size: Some(vec2(destination.w, destination.h)),
                flip_x: draw.mirror_x,
                ..Default::default()
            },
        );
    }

    fn draw_masked_title(&self, draw: MaskedTitleDraw) {
        let source = Rect::new(
            0.0,
            0.0,
            draw.mask_source.width as f32,
            draw.mask_source.height as f32,
        );
        let destination = rect(draw.destination);
        let viewport = PixelRect {
            x: 0,
            y: 0,
            width: VIEW_WIDTH as u32,
            height: VIEW_HEIGHT as u32,
        };
        if let Some((source, destination)) = crop_layer(source, destination, Some(viewport), false)
        {
            draw_texture_ex(
                &self.masked_title,
                destination.x,
                destination.y,
                WHITE,
                DrawTextureParams {
                    source: Some(source),
                    dest_size: Some(vec2(destination.w, destination.h)),
                    ..Default::default()
                },
            );
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

/// Crop draw coordinates instead of changing caller scissor/material state.
fn crop_layer(
    mut source: Rect,
    destination: Rect,
    clip: Option<PixelRect>,
    mirror_x: bool,
) -> Option<(Rect, Rect)> {
    let viewport = Rect::new(0.0, 0.0, VIEW_WIDTH, VIEW_HEIGHT);
    let clip = clip.map_or(viewport, |clip| {
        let clip = rect(clip);
        Rect::new(
            clip.x.max(viewport.x),
            clip.y.max(viewport.y),
            (clip.x + clip.w).min(viewport.x + viewport.w) - clip.x.max(viewport.x),
            (clip.y + clip.h).min(viewport.y + viewport.h) - clip.y.max(viewport.y),
        )
    });
    let left = destination.x.max(clip.x);
    let top = destination.y.max(clip.y);
    let right = (destination.x + destination.w).min(clip.x + clip.w);
    let bottom = (destination.y + destination.h).min(clip.y + clip.h);
    if right <= left || bottom <= top || destination.w <= 0.0 || destination.h <= 0.0 {
        return None;
    }
    let sx = source.w / destination.w;
    let sy = source.h / destination.h;
    source.x += if mirror_x {
        (destination.x + destination.w - right) * sx
    } else {
        (left - destination.x) * sx
    };
    source.y += (top - destination.y) * sy;
    source.w = (right - left) * sx;
    source.h = (bottom - top) * sy;
    Some((source, Rect::new(left, top, right - left, bottom - top)))
}

fn compose_title_mask(
    title: &DecodedImage,
    mask: &DecodedImage,
) -> Result<Texture2D, Box<dyn Error>> {
    let width = mask.width.min(title.width.saturating_sub(MASK_X));
    let height = mask.height.min(title.height.saturating_sub(MASK_Y));
    if width == 0 || height == 0 {
        return Err("title mask does not intersect title-page RGB".into());
    }
    let mut pixels = vec![0u8; width as usize * height as usize * 4];
    for y in 0..height as usize {
        for x in 0..width as usize {
            let source = ((y + MASK_Y as usize) * title.width as usize + x + MASK_X as usize) * 4;
            let mask_source = (y * mask.width as usize + x) * 4;
            let target = (y * width as usize + x) * 4;
            pixels[target..target + 3].copy_from_slice(&title.pixels[source..source + 3]);
            pixels[target + 3] = mask.pixels[mask_source + 3];
        }
    }
    let texture = Texture2D::from_rgba8(u16::try_from(width)?, u16::try_from(height)?, &pixels);
    texture.set_filter(FilterMode::Linear);
    Ok(texture)
}
