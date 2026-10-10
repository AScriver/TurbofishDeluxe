//! Transient title/loading presentation, informed by pinned W1 TitleScreen.
//!
//! Loading and asset readiness belong to the caller. This controller consumes
//! actual task reports, advances once per caller-owned UI tick and emits draw
//! descriptions and transition requests. See `docs/title-screen.md` for source
//! qualifications, safety policies and integration responsibilities.

use std::{error::Error, fmt};

const BAR_X: i32 = 180;
const BAR_Y: i32 = 415;
const MASK_X: i32 = 384;
const MASK_Y: i32 = 388;
const STINKY_START_X: i32 = 160;
const STINKY_DRAW_END: i32 = 510;
const SAVER_FINISH_X: i32 = 511;
const STINKY_CELL: u32 = 80;
const BLEND_TICKS: u8 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleImage {
    TitlePage,
    LoaderBar,
    LoaderBarOver,
    LoadingLabel,
    PlayLabel,
    Stinky,
    TitlePageMask,
}

impl TitleImage {
    pub const ALL: [Self; 7] = [
        Self::TitlePage,
        Self::LoaderBar,
        Self::LoaderBarOver,
        Self::LoadingLabel,
        Self::PlayLabel,
        Self::Stinky,
        Self::TitlePageMask,
    ];

    pub const fn resource_id(self) -> &'static str {
        match self {
            Self::TitlePage => "IMAGE_TITLEPAGE",
            Self::LoaderBar => "IMAGE_LOADERBAR",
            Self::LoaderBarOver => "IMAGE_LOADERBAROVER",
            Self::LoadingLabel => "IMAGE_LOADERBARLOADING",
            Self::PlayLabel => "IMAGE_LOADERPLAY",
            Self::Stinky => "IMAGE_STINKY",
            Self::TitlePageMask => "IMAGE_TITLEPAGEMASK",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}

impl Dimensions {
    // Matches the current renderer's checked u16 texture-dimension boundary.
    fn valid(self) -> bool {
        self.width > 0
            && self.height > 0
            && self.width <= u32::from(u16::MAX)
            && self.height <= u32::from(u16::MAX)
    }

    fn rect(self, x: i32, y: i32) -> PixelRect {
        PixelRect {
            x,
            y,
            width: self.width,
            height: self.height,
        }
    }
}

/// Actual decoded bootstrap image dimensions, not guessed resource readiness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TitleAssets {
    pub title_page: Dimensions,
    pub loader_bar: Dimensions,
    pub loader_bar_over: Dimensions,
    pub loading_label: Dimensions,
    pub play_label: Dimensions,
    pub stinky: Dimensions,
    pub title_page_mask: Dimensions,
}

impl TitleAssets {
    pub const fn dimensions(self, image: TitleImage) -> Dimensions {
        match image {
            TitleImage::TitlePage => self.title_page,
            TitleImage::LoaderBar => self.loader_bar,
            TitleImage::LoaderBarOver => self.loader_bar_over,
            TitleImage::LoadingLabel => self.loading_label,
            TitleImage::PlayLabel => self.play_label,
            TitleImage::Stinky => self.stinky,
            TitleImage::TitlePageMask => self.title_page_mask,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LoadingStatus {
    #[default]
    Waiting,
    Running,
    Succeeded,
    Failed,
}

/// Count tasks only when the real caller-owned operation has completed.
/// `Succeeded` also asserts that all required resources are usable by the caller.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LoadingReport {
    pub completed_tasks: u64,
    pub total_tasks: u64,
    pub status: LoadingStatus,
}

impl LoadingReport {
    fn validate(self) -> Result<(), TitleError> {
        if self.completed_tasks > self.total_tasks {
            return Err(TitleError::TaskCountExceedsTotal);
        }
        if self.status == LoadingStatus::Waiting && self.completed_tasks != 0 {
            return Err(TitleError::WaitingWithCompletedTasks);
        }
        if self.status == LoadingStatus::Succeeded && self.completed_tasks != self.total_tasks {
            return Err(TitleError::IncompleteSuccess);
        }
        Ok(())
    }

    /// Source-style bounded progress. Successful zero-work loading is explicit;
    /// a waiting/running report with no known tasks stays at zero.
    pub fn progress(self) -> Result<f64, TitleError> {
        self.validate()?;
        Ok(self.fraction())
    }

    fn fraction(self) -> f64 {
        if self.status == LoadingStatus::Succeeded {
            1.0
        } else if self.status == LoadingStatus::Waiting || self.total_tasks == 0 {
            0.0
        } else {
            (self.completed_tasks as f64 / self.total_tasks as f64).clamp(0.0, 1.0)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleMode {
    Application,
    Screensaver,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FinishTarget {
    GameSelector,
    ScreensaverBoard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleEvent {
    /// Route to the existing sound owner; pressing does not finish the title.
    StartPressed,
    /// Admitted once; the caller owns removal, routing and backlog reset.
    Finished(FinishTarget),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleError {
    InvalidImageDimensions(TitleImage),
    InvalidViewportWidth,
    StinkySheetTooSmall,
    EmptyMaskIntersection,
    TaskCountExceedsTotal,
    WaitingWithCompletedTasks,
    IncompleteSuccess,
    LoadingAlreadyFailed,
    LoadingAlreadySucceeded,
    AlreadyFinished,
}

impl fmt::Display for TitleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidImageDimensions(image) => {
                write!(
                    formatter,
                    "invalid title image dimensions: {}",
                    image.resource_id()
                )
            }
            Self::InvalidViewportWidth => {
                formatter.write_str("invalid logical title viewport width")
            }
            Self::StinkySheetTooSmall => {
                formatter.write_str("title Stinky needs ten 80x80 cells in row zero")
            }
            Self::EmptyMaskIntersection => {
                formatter.write_str("title mask does not intersect the title page")
            }
            Self::TaskCountExceedsTotal => {
                formatter.write_str("completed loading tasks exceed total tasks")
            }
            Self::WaitingWithCompletedTasks => {
                formatter.write_str("waiting loading report contains completed tasks")
            }
            Self::IncompleteSuccess => {
                formatter.write_str("successful loading report has incomplete tasks")
            }
            Self::LoadingAlreadyFailed => {
                formatter.write_str("failed title loading requires a fresh controller")
            }
            Self::LoadingAlreadySucceeded => {
                formatter.write_str("successful title loading cannot return to pending work")
            }
            Self::AlreadyFinished => {
                formatter.write_str("title transition has already been admitted")
            }
        }
    }
}

impl Error for TitleError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PixelRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl PixelRect {
    fn contains(self, x: f64, y: f64) -> bool {
        x.is_finite()
            && y.is_finite()
            && x >= f64::from(self.x)
            && y >= f64::from(self.y)
            && x < f64::from(self.x) + f64::from(self.width)
            && y < f64::from(self.y) + f64::from(self.height)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageDraw {
    pub image: TitleImage,
    pub source: PixelRect,
    pub destination: PixelRect,
    /// Logical screen-space clip; intersect with the caller's viewport.
    pub clip: Option<PixelRect>,
    pub alpha: u8,
    pub mirror_x: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaskedTitleDraw {
    /// RGB comes from this title-page crop, not the mask's RGB.
    pub title_source: PixelRect,
    /// Alpha comes from IMAGE_TITLEPAGEMASK at this local rectangle.
    pub mask_source: PixelRect,
    pub destination: PixelRect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleLayer {
    Image(ImageDraw),
    MaskedTitle(MaskedTitleDraw),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StartButtonView {
    pub bounds: PixelRect,
    pub visible: bool,
    pub hovered: bool,
    /// Pressed appearance requires a captured press and current hover.
    pub pressed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TitleView {
    /// Draw in this order. A finished title has no layers.
    pub layers: Vec<TitleLayer>,
    pub start: StartButtonView,
    pub loading: LoadingReport,
    pub displayed_fill: u32,
    pub ready: bool,
    pub finished: bool,
}

/// App-lived presentation state. It contains no simulation or persistent data.
#[derive(Debug)]
pub struct TitleScreen {
    assets: TitleAssets,
    mode: TitleMode,
    viewport_width: i32,
    loading: LoadingReport,
    fill: u32,
    speed: u32,
    stinky_x: i32,
    animation_tick: u8,
    blend: u8,
    ready: bool,
    hovered: bool,
    captured_press: bool,
    finished: bool,
}

impl TitleScreen {
    /// Construct only after the caller has loaded the seven bootstrap images.
    /// Coordinates remain in the source's logical 640x480 space; the caller
    /// scales rendering/input. Width controls only the source's post-ready walk.
    pub fn new(
        mode: TitleMode,
        viewport_width: u32,
        assets: TitleAssets,
    ) -> Result<Self, TitleError> {
        for image in TitleImage::ALL {
            if !assets.dimensions(image).valid() {
                return Err(TitleError::InvalidImageDimensions(image));
            }
        }
        if viewport_width == 0
            || viewport_width > u32::from(u16::MAX)
            || (mode == TitleMode::Screensaver && viewport_width < SAVER_FINISH_X as u32)
        {
            return Err(TitleError::InvalidViewportWidth);
        }
        if assets.stinky.width < STINKY_CELL * 10 || assets.stinky.height < STINKY_CELL {
            return Err(TitleError::StinkySheetTooSmall);
        }
        if assets.title_page.width <= MASK_X as u32 || assets.title_page.height <= MASK_Y as u32 {
            return Err(TitleError::EmptyMaskIntersection);
        }
        Ok(Self {
            assets,
            mode,
            viewport_width: viewport_width as i32,
            loading: LoadingReport::default(),
            fill: 0,
            speed: 0,
            stinky_x: STINKY_START_X,
            animation_tick: 0,
            blend: 0,
            ready: false,
            hovered: false,
            captured_press: false,
            finished: false,
        })
    }

    /// Validate atomically. Counts/totals may change while pending: the source
    /// never shrinks already displayed progress. Failure can revoke readiness
    /// before finish, and cannot be reversed by a later success callback.
    pub fn report_loading(&mut self, report: LoadingReport) -> Result<(), TitleError> {
        if self.finished {
            return Err(TitleError::AlreadyFinished);
        }
        report.validate()?;
        if self.loading.status == LoadingStatus::Failed && report != self.loading {
            return Err(TitleError::LoadingAlreadyFailed);
        }
        if self.loading.status == LoadingStatus::Succeeded
            && report.status != LoadingStatus::Failed
            && report != self.loading
        {
            return Err(TitleError::LoadingAlreadySucceeded);
        }
        self.loading = report;
        if report.status == LoadingStatus::Failed {
            self.ready = false;
            self.blend = 0;
            self.cancel_pointer();
        }
        Ok(())
    }

    pub fn can_start(&self) -> bool {
        self.mode == TitleMode::Application
            && self.ready
            && !self.finished
            && self.loading.status == LoadingStatus::Succeeded
    }

    /// One source-style UI update, not a wall timer or a loading task. The caller
    /// supplies its existing fixed-step UI clock and must not step the session.
    pub fn advance(&mut self) -> Option<TitleEvent> {
        if self.finished || self.loading.status == LoadingStatus::Failed {
            return None;
        }
        self.animation_tick = (self.animation_tick + 1) % 40;
        self.blend = self.blend.saturating_sub(1);
        let width = self.assets.loader_bar.width;
        if f64::from(self.fill) < self.loading.fraction() * f64::from(width) {
            self.speed = (self.speed + 1).min(3);
            self.fill += self.speed;
            // W1 sets Stinky's position before clamping a whole-bar overshoot.
            self.stinky_x = STINKY_START_X + self.fill as i32;
            if self.fill > width {
                self.fill = width;
                self.speed -= 1;
            }
        } else if !self.ready {
            if self.loading.status == LoadingStatus::Succeeded {
                self.ready = true;
                if self.mode == TitleMode::Application {
                    self.blend = BLEND_TICKS;
                }
            }
        } else {
            if self.stinky_x < self.viewport_width {
                self.speed = (self.speed + 1).min(3);
                self.stinky_x += self.speed as i32;
            }
            if self.mode == TitleMode::Screensaver && self.stinky_x >= SAVER_FINISH_X {
                return self.finish(FinishTarget::ScreensaverBoard);
            }
        }
        None
    }

    /// Logical coordinates. Nonfinite coordinates behave as outside the button.
    pub fn pointer_move(&mut self, x: f64, y: f64) {
        self.hovered = self.can_start() && self.button_bounds().contains(x, y);
    }

    /// Route only the primary-button press. A repeated down does not replay audio.
    pub fn pointer_press(&mut self, x: f64, y: f64) -> Option<TitleEvent> {
        self.pointer_move(x, y);
        if !self.hovered || self.captured_press {
            return None;
        }
        self.captured_press = true;
        Some(TitleEvent::StartPressed)
    }

    /// Only a captured, ready, inside release admits the ordinary transition.
    pub fn pointer_release(&mut self, x: f64, y: f64) -> Option<TitleEvent> {
        self.pointer_move(x, y);
        let activate = self.captured_press && self.hovered;
        self.captured_press = false;
        if activate {
            self.finish(FinishTarget::GameSelector)
        } else {
            None
        }
    }

    /// Use on focus loss, pointer cancellation, modal interruption or exit.
    pub fn cancel_pointer(&mut self) {
        self.hovered = false;
        self.captured_press = false;
    }

    pub fn view(&self) -> TitleView {
        let visible = self.can_start();
        let hovered = visible && self.hovered;
        let pressed = hovered && self.captured_press;
        let mut view = TitleView {
            layers: Vec::with_capacity(7),
            start: StartButtonView {
                bounds: self.button_bounds(),
                visible,
                hovered,
                pressed,
            },
            loading: self.loading,
            displayed_fill: self.fill,
            ready: self.ready,
            finished: self.finished,
        };
        if self.finished {
            return view;
        }
        let x = BAR_X + i32::from(pressed);
        let y = BAR_Y + i32::from(pressed);
        let bar = if hovered {
            TitleImage::LoaderBarOver
        } else {
            TitleImage::LoaderBar
        };
        let clip = PixelRect {
            x,
            y,
            width: self.fill,
            height: self.assets.dimensions(bar).height,
        };
        view.layers
            .push(self.image(TitleImage::TitlePage, 0, 0, None, 255));
        view.layers.push(self.image(bar, x, y, Some(clip), 255));
        if !visible {
            view.layers
                .push(self.image(TitleImage::LoadingLabel, 230, BAR_Y, Some(clip), 255));
        } else {
            let loading_alpha = (u16::from(self.blend) * 255 / u16::from(BLEND_TICKS)) as u8;
            if loading_alpha > 0 {
                view.layers.push(self.image(
                    TitleImage::LoadingLabel,
                    230,
                    BAR_Y,
                    None,
                    loading_alpha,
                ));
            }
            view.layers
                .push(self.image(TitleImage::PlayLabel, x, y, None, 255 - loading_alpha));
        }
        if self.stinky_x < STINKY_DRAW_END {
            let cell = Dimensions {
                width: STINKY_CELL,
                height: STINKY_CELL,
            };
            view.layers.push(TitleLayer::Image(ImageDraw {
                image: TitleImage::Stinky,
                source: cell.rect(i32::from(self.animation_tick / 4) * STINKY_CELL as i32, 0),
                destination: cell.rect(self.stinky_x, y - 30),
                clip: None,
                alpha: 255,
                mirror_x: true,
            }));
        }
        let mask = Dimensions {
            width: self
                .assets
                .title_page_mask
                .width
                .min(self.assets.title_page.width - MASK_X as u32),
            height: self
                .assets
                .title_page_mask
                .height
                .min(self.assets.title_page.height - MASK_Y as u32),
        };
        view.layers.push(TitleLayer::MaskedTitle(MaskedTitleDraw {
            title_source: mask.rect(MASK_X, MASK_Y),
            mask_source: mask.rect(0, 0),
            destination: mask.rect(MASK_X, MASK_Y),
        }));
        view
    }

    fn button_bounds(&self) -> PixelRect {
        self.assets.loader_bar.rect(BAR_X, BAR_Y)
    }

    fn image(
        &self,
        image: TitleImage,
        x: i32,
        y: i32,
        clip: Option<PixelRect>,
        alpha: u8,
    ) -> TitleLayer {
        let size = self.assets.dimensions(image);
        TitleLayer::Image(ImageDraw {
            image,
            source: size.rect(0, 0),
            destination: size.rect(x, y),
            clip,
            alpha,
            mirror_x: false,
        })
    }

    fn finish(&mut self, target: FinishTarget) -> Option<TitleEvent> {
        if self.finished || !self.ready || self.loading.status != LoadingStatus::Succeeded {
            return None;
        }
        self.finished = true;
        self.cancel_pointer();
        Some(TitleEvent::Finished(target))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Synthetic dimensions exercise contracts; they do not claim retail artwork.
    fn assets(bar_width: u32) -> TitleAssets {
        let bar = Dimensions {
            width: bar_width,
            height: 40,
        };
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
        }
    }

    fn report(completed: u64, total: u64, status: LoadingStatus) -> LoadingReport {
        LoadingReport {
            completed_tasks: completed,
            total_tasks: total,
            status,
        }
    }

    fn image(view: &TitleView, wanted: TitleImage) -> ImageDraw {
        view.layers
            .iter()
            .find_map(|layer| match layer {
                TitleLayer::Image(draw) if draw.image == wanted => Some(*draw),
                _ => None,
            })
            .unwrap()
    }

    fn make_ready(screen: &mut TitleScreen) {
        screen
            .report_loading(report(1, 1, LoadingStatus::Succeeded))
            .unwrap();
        for _ in 0..500 {
            assert_eq!(screen.advance(), None);
            if screen.view().ready {
                return;
            }
        }
        panic!("synthetic title did not reach readiness within the bound");
    }

    #[test]
    fn source_fill_acceleration_overshoot_and_readiness_branch_order() {
        let mut screen = TitleScreen::new(TitleMode::Application, 640, assets(8)).unwrap();
        screen
            .report_loading(report(1, 4, LoadingStatus::Running))
            .unwrap();
        assert_eq!(screen.advance(), None);
        assert_eq!(screen.view().displayed_fill, 1);
        screen.advance();
        assert_eq!(screen.view().displayed_fill, 3); // Can pass fractional target2.
        screen.advance();
        assert_eq!(screen.view().displayed_fill, 3);
        assert!(!screen.can_start());
        screen
            .report_loading(report(4, 4, LoadingStatus::Succeeded))
            .unwrap();
        screen.advance();
        assert_eq!(screen.view().displayed_fill, 6);
        screen.advance();
        let filled = screen.view();
        assert_eq!(filled.displayed_fill, 8);
        assert_eq!(image(&filled, TitleImage::Stinky).destination.x, 169);
        assert!(!filled.ready); // Clamping and readiness occur on different ticks.
        screen.advance();
        let ready = screen.view();
        assert!(ready.start.visible);
        assert_eq!(image(&ready, TitleImage::LoadingLabel).alpha, 255);
        assert_eq!(image(&ready, TitleImage::PlayLabel).alpha, 0);
        screen.advance();
        assert_eq!(image(&screen.view(), TitleImage::Stinky).destination.x, 172);
    }

    #[test]
    fn source_blend_clips_pose_hover_and_mask_layer_order() {
        let mut screen = TitleScreen::new(TitleMode::Application, 640, assets(280)).unwrap();
        let loading = screen.view();
        let label = image(&loading, TitleImage::LoadingLabel);
        assert_eq!(
            label.clip.unwrap(),
            PixelRect {
                x: 180,
                y: 415,
                width: 0,
                height: 40
            }
        );
        let snail = image(&loading, TitleImage::Stinky);
        assert_eq!(
            snail.destination,
            PixelRect {
                x: 160,
                y: 385,
                width: 80,
                height: 80
            }
        );
        assert!(snail.mirror_x);
        for _ in 0..4 {
            screen.advance();
        }
        assert_eq!(image(&screen.view(), TitleImage::Stinky).source.x, 80);
        for _ in 0..36 {
            screen.advance();
        }
        assert_eq!(image(&screen.view(), TitleImage::Stinky).source.x, 0);
        make_ready(&mut screen);
        assert_eq!(
            screen.pointer_press(180.0, 415.0),
            Some(TitleEvent::StartPressed)
        );
        let pressed = screen.view();
        assert_eq!(pressed.start.bounds.x, 180); // Hit box stays fixed.
        assert_eq!(
            image(&pressed, TitleImage::LoaderBarOver).destination.x,
            181
        );
        assert_eq!(image(&pressed, TitleImage::PlayLabel).destination.y, 416);
        assert_eq!(image(&pressed, TitleImage::LoadingLabel).destination.x, 230);
        assert_eq!(image(&pressed, TitleImage::LoadingLabel).clip, None);
        assert_eq!(image(&pressed, TitleImage::Stinky).destination.y, 386);
        assert_eq!(
            pressed.layers.last(),
            Some(&TitleLayer::MaskedTitle(MaskedTitleDraw {
                title_source: PixelRect {
                    x: 384,
                    y: 388,
                    width: 256,
                    height: 92
                },
                mask_source: PixelRect {
                    x: 0,
                    y: 0,
                    width: 256,
                    height: 92
                },
                destination: PixelRect {
                    x: 384,
                    y: 388,
                    width: 256,
                    height: 92
                },
            }))
        );
        for expected_loading_alpha in [223, 191, 159, 127, 95, 63, 31] {
            screen.advance();
            let view = screen.view();
            assert_eq!(
                image(&view, TitleImage::LoadingLabel).alpha,
                expected_loading_alpha
            );
            assert_eq!(
                image(&view, TitleImage::PlayLabel).alpha,
                255 - expected_loading_alpha
            );
        }
        screen.advance();
        assert_eq!(image(&screen.view(), TitleImage::PlayLabel).alpha, 255);
        assert!(!screen.view().layers.iter().any(|layer| matches!(layer,
            TitleLayer::Image(draw) if draw.image == TitleImage::LoadingLabel)));
    }

    #[test]
    fn safety_no_clock_or_full_count_can_invent_success() {
        let mut screen = TitleScreen::new(TitleMode::Application, 640, assets(8)).unwrap();
        for _ in 0..100 {
            screen.advance();
        }
        assert_eq!(screen.view().displayed_fill, 0);
        assert!(!screen.can_start());
        assert_eq!(screen.pointer_press(181.0, 416.0), None);
        screen
            .report_loading(report(2, 2, LoadingStatus::Running))
            .unwrap();
        for _ in 0..100 {
            assert_eq!(screen.advance(), None);
        }
        assert_eq!(screen.view().displayed_fill, 8);
        assert!(!screen.can_start());
        screen
            .report_loading(report(1, 10, LoadingStatus::Running))
            .unwrap();
        screen.advance();
        assert_eq!(screen.view().displayed_fill, 8); // Source does not rewind fill.
        make_ready(&mut screen);
        assert_eq!(screen.pointer_release(181.0, 416.0), None); // Old pending press did not arm.
        assert_eq!(screen.pointer_release(f64::NAN, 416.0), None);
    }

    #[test]
    fn safety_capture_cancel_edges_failure_and_once_only_finish() {
        let mut screen = TitleScreen::new(TitleMode::Application, 640, assets(8)).unwrap();
        make_ready(&mut screen);
        assert_eq!(screen.pointer_press(188.0, 416.0), None); // Right edge exclusive.
        assert_eq!(screen.pointer_press(181.0, 455.0), None); // Bottom edge exclusive.
        assert_eq!(screen.pointer_press(f64::INFINITY, 416.0), None);
        assert_eq!(
            screen.pointer_press(181.0, 416.0),
            Some(TitleEvent::StartPressed)
        );
        assert_eq!(screen.pointer_press(181.0, 416.0), None);
        screen.pointer_move(0.0, 0.0);
        assert!(!screen.view().start.pressed);
        assert_eq!(screen.pointer_release(0.0, 0.0), None);
        screen.pointer_press(181.0, 416.0);
        screen.cancel_pointer();
        assert_eq!(screen.pointer_release(181.0, 416.0), None);
        screen.pointer_press(181.0, 416.0);
        assert_eq!(
            screen.pointer_release(181.0, 416.0),
            Some(TitleEvent::Finished(FinishTarget::GameSelector))
        );
        assert_eq!(screen.pointer_release(181.0, 416.0), None);
        assert_eq!(screen.pointer_press(181.0, 416.0), None);
        assert_eq!(screen.advance(), None);
        assert!(screen.view().layers.is_empty());
        assert_eq!(
            screen.report_loading(report(1, 1, LoadingStatus::Succeeded)),
            Err(TitleError::AlreadyFinished)
        );

        let mut failed = TitleScreen::new(TitleMode::Application, 640, assets(8)).unwrap();
        make_ready(&mut failed);
        failed.pointer_press(181.0, 416.0);
        failed
            .report_loading(report(1, 1, LoadingStatus::Failed))
            .unwrap();
        assert!(!failed.can_start());
        assert_eq!(failed.pointer_release(181.0, 416.0), None);
        for _ in 0..100 {
            assert_eq!(failed.advance(), None);
        }
        assert_eq!(
            failed.report_loading(report(1, 1, LoadingStatus::Succeeded)),
            Err(TitleError::LoadingAlreadyFailed)
        );
    }

    #[test]
    fn source_saver_finishes_after_ready_threshold_once_without_start_button() {
        let mut screen = TitleScreen::new(TitleMode::Screensaver, 640, assets(350)).unwrap();
        screen
            .report_loading(report(1, 1, LoadingStatus::Succeeded))
            .unwrap();
        for _ in 0..500 {
            assert_eq!(screen.advance(), None);
            if screen.view().ready {
                break;
            }
        }
        assert!(screen.view().ready);
        assert!(!screen.view().start.visible);
        // Fill's overshoot placed Stinky at511; readiness's own tick does not finish.
        assert!(!screen.view().layers.iter().any(|layer| matches!(layer,
            TitleLayer::Image(draw) if draw.image == TitleImage::Stinky)));
        assert_eq!(screen.pointer_press(181.0, 416.0), None);
        assert_eq!(
            screen.advance(),
            Some(TitleEvent::Finished(FinishTarget::ScreensaverBoard))
        );
        for _ in 0..100 {
            assert_eq!(screen.advance(), None);
        }
    }

    #[test]
    fn safety_loading_reports_and_asset_metadata_validate_atomically() {
        let mut screen = TitleScreen::new(TitleMode::Application, 640, assets(8)).unwrap();
        let before = screen.view();
        for (bad, expected) in [
            (
                report(1, 0, LoadingStatus::Running),
                TitleError::TaskCountExceedsTotal,
            ),
            (
                report(1, 2, LoadingStatus::Waiting),
                TitleError::WaitingWithCompletedTasks,
            ),
            (
                report(1, 2, LoadingStatus::Succeeded),
                TitleError::IncompleteSuccess,
            ),
        ] {
            assert_eq!(screen.report_loading(bad), Err(expected));
            assert_eq!(screen.view(), before);
        }
        assert_eq!(report(0, 0, LoadingStatus::Running).progress(), Ok(0.0));
        assert_eq!(report(0, 0, LoadingStatus::Succeeded).progress(), Ok(1.0));
        assert_eq!(
            report(u64::MAX, u64::MAX, LoadingStatus::Running).progress(),
            Ok(1.0)
        );
        screen
            .report_loading(report(0, 0, LoadingStatus::Succeeded))
            .unwrap();
        assert_eq!(
            screen.report_loading(report(0, 1, LoadingStatus::Running)),
            Err(TitleError::LoadingAlreadySucceeded)
        );
        assert!(matches!(
            TitleScreen::new(TitleMode::Application, 0, assets(8)),
            Err(TitleError::InvalidViewportWidth)
        ));
        assert!(matches!(
            TitleScreen::new(TitleMode::Screensaver, 510, assets(8)),
            Err(TitleError::InvalidViewportWidth)
        ));
        let mut bad_assets = assets(0);
        assert!(matches!(
            TitleScreen::new(TitleMode::Application, 640, bad_assets),
            Err(TitleError::InvalidImageDimensions(TitleImage::LoaderBar))
        ));
        bad_assets = assets(8);
        bad_assets.stinky.width = 799;
        assert!(matches!(
            TitleScreen::new(TitleMode::Application, 640, bad_assets),
            Err(TitleError::StinkySheetTooSmall)
        ));
        bad_assets = assets(8);
        bad_assets.title_page.height = 388;
        assert!(matches!(
            TitleScreen::new(TitleMode::Application, 640, bad_assets),
            Err(TitleError::EmptyMaskIntersection)
        ));
    }
}
