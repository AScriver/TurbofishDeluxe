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
const LOADING_BURST_SECONDS: f64 = 0.008;

/// Pure between-task schedule. Waiting for a frame is UI time, not work in the
/// next burst. Task counts and the Title clock remain independent of rendering.
struct LoadingSchedule {
    total_tasks: u64,
    completed_tasks: u64,
    burst_started_at: f64,
    ui_updated_at: f64,
    app_started_at: f64,
    quit_after: Option<f64>,
}

impl LoadingSchedule {
    fn new(total_tasks: u64, now: f64, app_started_at: f64, quit_after: Option<f64>) -> Self {
        Self {
            total_tasks,
            completed_tasks: TitleImage::ALL.len() as u64,
            burst_started_at: now,
            ui_updated_at: now,
            app_started_at,
            quit_after,
        }
    }

    fn check_deadline(&self, now: f64, quit_requested: bool) -> Result<(), Box<dyn Error>> {
        if quit_requested || wall_deadline_reached(now - self.app_started_at, self.quit_after) {
            Err("title startup canceled before gameplay or saver entry".into())
        } else {
            Ok(())
        }
    }

    fn report(&self, status: LoadingStatus) -> LoadingReport {
        LoadingReport {
            completed_tasks: self.completed_tasks,
            total_tasks: self.total_tasks,
            status,
        }
    }

    fn task_completed(
        &mut self,
        title: &mut TitleScreen,
        now: f64,
        quit_requested: bool,
    ) -> Result<bool, Box<dyn Error>> {
        self.check_deadline(now, quit_requested)?;
        let completed = self
            .completed_tasks
            .checked_add(1)
            .filter(|completed| *completed <= self.total_tasks)
            .ok_or("title loader reported more tasks than planned")?;
        title.report_loading(LoadingReport {
            completed_tasks: completed,
            total_tasks: self.total_tasks,
            status: LoadingStatus::Running,
        })?;
        self.completed_tasks = completed;
        // Flush the last completion even when the entire load fits one burst.
        Ok(completed == self.total_tasks || now - self.burst_started_at >= LOADING_BURST_SECONDS)
    }

    fn frame_seconds(&mut self, now: f64) -> f32 {
        let elapsed = (now - self.ui_updated_at).max(0.0);
        self.ui_updated_at = now;
        elapsed as f32
    }

    fn resume_after_yield(&mut self, now: f64) {
        self.burst_started_at = now;
    }
}

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
    loading: LoadingSchedule,
    clock: StepClock,
    pending_finish: Option<FinishTarget>,
    ui_evidence: Option<super::UiEvidence>,
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
            loading: LoadingSchedule::new(total_tasks, get_time(), app_started_at, quit_after),
            clock: StepClock::new(1),
            pending_finish: None,
            ui_evidence: None,
        })
    }

    pub(super) fn observe_in(
        &mut self,
        root: Option<&std::path::Path>,
    ) -> Result<(), Box<dyn Error>> {
        self.ui_evidence = root.map(super::UiEvidence::open).transpose()?;
        Ok(())
    }

    pub(super) fn take_ui_evidence(&mut self) -> Option<super::UiEvidence> {
        self.ui_evidence.take()
    }

    fn draw_and_observe(&mut self) -> Result<(), Box<dyn Error>> {
        self.draw();
        if let Some(observer) = self.ui_evidence.as_mut() {
            observer.record_draw(
                "Title",
                Some(&self.title.view()),
                None,
                get_time() - self.loading.app_started_at,
            )?;
        }
        Ok(())
    }

    /// Clone GPU handle and real atlas dimensions into Presentation; no second
    /// disk decode or duplicate upload is needed for shared Stinky/title art.
    pub(super) fn bootstrap_image(&self, id: &str) -> Option<(Texture2D, (u32, u32))> {
        self.images
            .get(id)
            .map(|image| (image.texture.clone(), image.cells))
    }

    pub(super) fn check_deadline(&self) -> Result<(), Box<dyn Error>> {
        self.loading.check_deadline(get_time(), is_quit_requested())
    }

    /// Call only after one real required task completed successfully. Drawing
    /// and frame yielding are cooperative; no sleep or fabricated task count.
    pub(super) async fn task_completed(&mut self) -> Result<(), Box<dyn Error>> {
        if self
            .loading
            .task_completed(&mut self.title, get_time(), is_quit_requested())?
        {
            self.draw_and_yield().await?;
        }
        Ok(())
    }

    /// Caller asserts all required resources/backends are usable. Model
    /// rejects any incomplete task plan before admitting the Start button.
    pub(super) fn succeed(&mut self) -> Result<(), Box<dyn Error>> {
        self.check_deadline()?;
        self.title
            .report_loading(self.loading.report(LoadingStatus::Succeeded))?;
        Ok(())
    }

    /// Terminal failure: title can neither start nor later be marked ready.
    pub(super) fn fail(&mut self) -> Result<(), Box<dyn Error>> {
        self.title
            .report_loading(self.loading.report(LoadingStatus::Failed))?;
        Ok(())
    }

    /// Cooperative frame between long caller-owned tasks. UI uses a separate
    /// 1x 28 ms clock and never advances the gameplay session.
    pub(super) async fn draw_and_yield(&mut self) -> Result<(), Box<dyn Error>> {
        self.check_deadline()?;
        self.advance_clock();
        self.draw_and_observe()?;
        next_frame().await;
        self.loading.resume_after_yield(get_time());
        self.check_deadline()
    }

    fn advance_clock(&mut self) {
        self.clock
            .add_frame(self.loading.frame_seconds(get_time()), false);
        while self.clock.step_due() {
            self.clock.consume_step(false);
            if let Some(TitleEvent::Finished(target)) = self.title.advance() {
                self.pending_finish = Some(target);
            }
        }
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
            self.advance_clock();
            if let Some(target) = self.pending_finish.take() {
                return Ok(target);
            }
            self.draw_and_observe()?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::title_screen::TitleError;

    fn fixture(
        total: u64,
        now: f64,
        app_start: f64,
        limit: Option<f64>,
    ) -> (LoadingSchedule, TitleScreen) {
        // Synthetic metadata; no textures, assets, window or game process.
        let bar = Dimensions {
            width: 280,
            height: 40,
        };
        let mut title = TitleScreen::new(
            TitleMode::Application,
            640,
            TitleAssets {
                title_page: Dimensions {
                    width: 640,
                    height: 480,
                },
                loader_bar: bar,
                loader_bar_over: bar,
                loading_label: Dimensions {
                    width: 100,
                    height: 40,
                },
                play_label: bar,
                stinky: Dimensions {
                    width: 800,
                    height: 160,
                },
                title_page_mask: Dimensions {
                    width: 300,
                    height: 100,
                },
            },
        )
        .unwrap();
        let schedule = LoadingSchedule::new(total, now, app_start, limit);
        title
            .report_loading(schedule.report(LoadingStatus::Running))
            .unwrap();
        (schedule, title)
    }

    #[test]
    fn fast_completions_update_counts_immediately_and_flush_only_at_end() {
        let (mut schedule, mut title) = fixture(192, 0.0, 0.0, None);
        for completed in 8..=192 {
            assert_eq!(
                schedule.task_completed(&mut title, 0.0, false).unwrap(),
                completed == 192
            );
            assert_eq!(
                schedule.report(LoadingStatus::Running).completed_tasks,
                completed
            );
            assert_eq!(schedule.report(LoadingStatus::Running).total_tasks, 192);
            assert_eq!(
                title.view().loading.progress().unwrap(),
                completed as f64 / 192.0
            );
            assert!(!title.can_start());
        }
    }

    #[test]
    fn wall_budget_yields_at_boundary_and_waiting_does_not_spend_next_burst() {
        let (mut schedule, mut title) = fixture(20, 0.0, 0.0, None);
        assert!(
            !schedule
                .task_completed(&mut title, 0.007_999, false)
                .unwrap()
        );
        assert!(schedule.task_completed(&mut title, 0.008, false).unwrap());
        // A deliberately slow frame wait must not make the next cheap task yield.
        schedule.resume_after_yield(1.0);
        assert!(!schedule.task_completed(&mut title, 1.001, false).unwrap());
        assert!(schedule.task_completed(&mut title, 1.125, false).unwrap());
    }

    #[test]
    fn quit_and_application_deadline_abort_even_before_burst_is_due() {
        let (mut schedule, mut title) = fixture(20, 11.99, 10.0, Some(2.0));
        assert!(!schedule.task_completed(&mut title, 11.991, false).unwrap());
        let before = title.view();
        assert!(schedule.task_completed(&mut title, 11.992, true).is_err());
        assert_eq!(schedule.completed_tasks, 8);
        assert_eq!(title.view(), before);
        assert!(schedule.task_completed(&mut title, 12.0, false).is_err());
        assert_eq!(schedule.completed_tasks, 8);
        assert_eq!(title.view(), before);
        // This is also the post-yield check: no new task is necessary to abort.
        assert!(schedule.check_deadline(12.0, false).is_err());
        assert!(schedule.check_deadline(11.993, true).is_err());
    }

    #[test]
    fn callbacks_do_not_tick_title_and_draws_count_work_and_wait_once() {
        let (mut schedule, mut title) = fixture(200, 0.0, 0.0, None);
        let mut clock = StepClock::new(1);
        for task in 1..=100 {
            schedule
                .task_completed(&mut title, f64::from(task) / 2048.0, false)
                .unwrap();
        }
        assert!(!clock.step_due());
        assert_eq!(title.view().displayed_fill, 0);
        clock.add_frame(schedule.frame_seconds(0.0625), false);
        let mut ticks = 0;
        while clock.step_due() {
            clock.consume_step(false);
            title.advance();
            ticks += 1;
        }
        assert_eq!(ticks, 2); // floor(62.5 / 28), independent of 100 callbacks.
        assert_eq!(title.view().displayed_fill, 3); // Existing 1,2,3 fill contract.
        assert_eq!(schedule.frame_seconds(0.0625), 0.0);
        schedule.resume_after_yield(0.078125);
        clock.add_frame(schedule.frame_seconds(0.09375), false);
        while clock.step_due() {
            clock.consume_step(false);
            title.advance();
            ticks += 1;
        }
        assert_eq!(ticks, 3); // floor(93.75 / 28), including work and frame wait.
        assert_eq!(title.view().displayed_fill, 6);
    }

    #[test]
    fn title_stall_keeps_existing_one_x_clock_clamp() {
        let (mut schedule, _) = fixture(20, 0.0, 0.0, None);
        let mut clock = StepClock::new(1);
        clock.add_frame(schedule.frame_seconds(10.0), false);
        let mut ticks = 0;
        while clock.step_due() {
            clock.consume_step(false);
            ticks += 1;
        }
        assert_eq!(ticks, 7); // floor(200 / 28), not 10 s or accelerated time.
        assert_eq!(schedule.frame_seconds(10.0), 0.0);
    }

    #[test]
    fn running_full_counts_and_incomplete_success_cannot_admit_start() {
        let (mut schedule, mut title) = fixture(9, 0.0, 0.0, None);
        schedule.task_completed(&mut title, 0.0, false).unwrap();
        let before = title.view();
        assert_eq!(
            title.report_loading(schedule.report(LoadingStatus::Succeeded)),
            Err(TitleError::IncompleteSuccess)
        );
        assert_eq!(title.view(), before);
        assert!(schedule.task_completed(&mut title, 0.0, false).unwrap());
        for _ in 0..200 {
            title.advance();
        }
        assert!(!title.can_start());
        assert_eq!(title.pointer_press(181.0, 416.0), None);
        title
            .report_loading(schedule.report(LoadingStatus::Succeeded))
            .unwrap();
        // Source model permits readiness only after the independent UI update.
        assert!(!title.can_start());
        title.advance();
        assert!(title.can_start());
        assert_eq!(
            title.pointer_press(181.0, 416.0),
            Some(TitleEvent::StartPressed)
        );
        assert_eq!(
            title.pointer_release(181.0, 416.0),
            Some(TitleEvent::Finished(FinishTarget::GameSelector))
        );
    }

    #[test]
    fn failure_is_terminal_and_rejected_completion_keeps_last_valid_counts() {
        let (mut schedule, mut title) = fixture(9, 0.0, 0.0, None);
        schedule.task_completed(&mut title, 0.0, false).unwrap();
        title
            .report_loading(schedule.report(LoadingStatus::Failed))
            .unwrap();
        let failed = title.view();
        assert!(schedule.task_completed(&mut title, 0.0, false).is_err());
        assert_eq!(schedule.completed_tasks, 8);
        assert_eq!(title.view(), failed);
        assert_eq!(
            title.report_loading(LoadingReport {
                completed_tasks: 9,
                total_tasks: 9,
                status: LoadingStatus::Succeeded
            }),
            Err(TitleError::LoadingAlreadyFailed)
        );
        for _ in 0..200 {
            title.advance();
        }
        assert_eq!(title.view(), failed);
        assert!(!title.can_start());
    }

    #[test]
    fn excess_completion_is_rejected_atomically() {
        let (mut schedule, mut title) = fixture(8, 0.0, 0.0, None);
        assert!(schedule.task_completed(&mut title, 0.0, false).unwrap());
        let before = title.view();
        assert!(schedule.task_completed(&mut title, 0.0, false).is_err());
        assert_eq!(schedule.completed_tasks, 8);
        assert_eq!(title.view(), before);
    }

    fn synthetic_yields(tasks: u64, work_seconds: f64) -> usize {
        let (mut schedule, mut title) = fixture(tasks + 7, 0.0, 0.0, None);
        let mut now = 0.0;
        let mut yields = 0;
        for _ in 0..tasks {
            now += work_seconds;
            if schedule.task_completed(&mut title, now, false).unwrap() {
                schedule.frame_seconds(now);
                yields += 1;
                now += 1.0 / 60.0; // Fake presentation wait; no real sleep.
                schedule.resume_after_yield(now);
            }
        }
        assert_eq!(schedule.completed_tasks, tasks + 7);
        title
            .report_loading(schedule.report(LoadingStatus::Succeeded))
            .unwrap();
        yields
    }

    #[test]
    fn synthetic_resource_schedules_reduce_yields_and_keep_slow_control() {
        for (tasks, burst_yields) in [(185, 21), (140, 16)] {
            // 1/1024 s = 0.9765625 ms; nine tasks cross the 8 ms budget.
            let actual = synthetic_yields(tasks, 1.0 / 1024.0);
            assert_eq!(actual, burst_yields);
            assert_eq!(synthetic_yields(tasks, 0.0), 1);
            assert_eq!(synthetic_yields(tasks, 1.0 / 64.0), tasks as usize);
            println!(
                "synthetic loading tasks={tasks} before_yields={tasks} after_yields={actual} instant_after=1 slow_after={tasks}"
            );
        }
    }
}
