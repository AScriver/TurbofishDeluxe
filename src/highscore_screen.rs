//! Renderer-independent Hall of Fame, informed by pinned W1 HighScoreScreen.
//! The caller owns immutable score/profile snapshots, asset/font metrics, scene
//! clock, BubbleMgr lifetime/RNG, sounds and exit/persistence transactions.

use super::{GlobalScores, ScoreKind};
use crate::bubbles::{BubbleConfig, BubbleRect};
use std::fmt;

pub const MISSING_RECORD: &str = "-----";
pub const SCREEN: Rect = Rect::new(0.0, 0.0, 640.0, 480.0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Adventure,
    TimeTrial,
    Challenge,
    Personal,
}

impl Page {
    pub const ALL: [Self; 4] = [
        Self::Adventure,
        Self::TimeTrial,
        Self::Challenge,
        Self::Personal,
    ];

    pub fn from_index(index: usize) -> Option<Self> {
        Self::ALL.get(index).copied()
    }

    pub const fn index(self) -> usize {
        match self {
            Self::Adventure => 0,
            Self::TimeTrial => 1,
            Self::Challenge => 2,
            Self::Personal => 3,
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::Adventure => "Adventure",
            Self::TimeTrial => "Time Trial",
            Self::Challenge => "Challenge",
            Self::Personal => "Personal Records",
        }
    }
}

/// Optional values distinguish no record from an earned zero. Adventure times
/// match project u64 seconds; money/survivors remain counts, never time ticks.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PersonalRecords {
    pub adventure_seconds: [[Option<u64>; 5]; 4],
    pub time_trial_money: [Option<u32>; 4],
    pub challenge_seconds: [Option<u64>; 4],
    pub tank5_survivors: Option<u32>,
}

#[derive(Clone, Copy, Debug)]
pub struct ScoreSnapshot<'a> {
    pub global: &'a GlobalScores,
    pub personal: &'a PersonalRecords,
    pub finished_game: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScoreUnit {
    ElapsedSeconds,
    Money,
    SurvivingPets,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordScope {
    Global,
    Personal,
}

/// Preserve the source's unit-specific formatting. The denominator18 is W1
/// presentation metadata, not a bound/clamp on supplied physical survivors.
pub fn format_value(unit: ScoreUnit, scope: RecordScope, value: Option<u64>) -> String {
    let Some(value) = value else {
        return MISSING_RECORD.into();
    };
    match unit {
        ScoreUnit::ElapsedSeconds => {
            let hours = value / 3600;
            let minutes = value / 60 % 60;
            let seconds = value % 60;
            if hours == 0 {
                format!("{minutes}:{seconds:02}")
            } else {
                format!("{hours:02}:{minutes:02}:{seconds:02}")
            }
        }
        ScoreUnit::Money if scope == RecordScope::Personal => {
            let digits = value.to_string();
            let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
            for (index, digit) in digits.bytes().enumerate() {
                if index > 0 && (digits.len() - index).is_multiple_of(3) {
                    grouped.push(',');
                }
                grouped.push(char::from(digit));
            }
            grouped
        }
        ScoreUnit::Money => value.to_string(),
        ScoreUnit::SurvivingPets if scope == RecordScope::Personal => {
            format!("{value} / 18 pets saved")
        }
        ScoreUnit::SurvivingPets => format!("{value} / 18"),
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(self, point: Point) -> bool {
        point.x.is_finite()
            && point.y.is_finite()
            && point.x >= self.x
            && point.x < self.x + self.width
            && point.y >= self.y
            && point.y < self.y + self.height
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HallAsset {
    Wave,
    ScreenTitle,
    ScreenTitleHole,
    MainButton,
}

impl HallAsset {
    pub const fn resource_key(self) -> &'static str {
        match self {
            Self::Wave => "IMAGE_WAVE",
            Self::ScreenTitle => "IMAGE_SCREENTITLE",
            Self::ScreenTitleHole => "IMAGE_SCREENTITLEHOLE",
            Self::MainButton => "IMAGE_MAINBUTTON",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HallFont {
    Title,
    Page,
    PageOutline,
    Heading,
    Row,
}

impl HallFont {
    pub const fn resource_key(self) -> &'static str {
        match self {
            Self::Title => "FONT_JUNGLEFEVER17OUTLINE",
            Self::Page => "FONT_CONTINUUMBOLD14",
            Self::PageOutline => "FONT_CONTINUUMBOLD14OUTLINE",
            Self::Heading => "FONT_JUNGLEFEVER12OUTLINE",
            Self::Row => "FONT_JUNGLEFEVER10OUTLINE",
        }
    }
}

/// Wave dimensions are per cel; other image dimensions are the complete image.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageMetrics {
    pub width: f32,
    pub height: f32,
    pub frames: u16,
}

pub trait HallMetrics {
    fn text_width(&self, font: HallFont, text: &str) -> f32;
    fn font_height(&self, font: HallFont) -> f32;
    fn image(&self, asset: HallAsset) -> ImageMetrics;
    /// Resolve through owned resource animation metadata, not a guessed rate.
    fn wave_frame(&self, tick: u64) -> u16;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HallClock {
    pub tick: u64,
    /// W1 uses text bobbing when 3D acceleration is enabled. This controls that
    /// presentation effect; it is unrelated to gameplay test-speed flags.
    pub bob_text: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HallColor {
    Rgb(u8, u8, u8),
    /// Source byte-valued HSL inputs; caller uses its normal HSL conversion.
    Hsl(u8, u8, u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    Background,
    Decoration,
    Content,
    Header,
    Controls,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageFrame {
    Cel(u16),
    /// Existing frontend image-box/nine-slice treatment, not arbitrary scaling.
    ImageBox,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextRun {
    pub text: String,
    pub font: HallFont,
    pub outline: Option<HallFont>,
    pub baseline: Point,
    pub color: HallColor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
    Tab(Page),
    Menu,
}

impl Control {
    const ALL: [Self; 5] = [
        Self::Tab(Page::Adventure),
        Self::Tab(Page::TimeTrial),
        Self::Tab(Page::Challenge),
        Self::Tab(Page::Personal),
        Self::Menu,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Tab(Page::Personal) => "Personal",
            Self::Tab(page) => page.title(),
            Self::Menu => "Menu",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ButtonView {
    pub control: Control,
    pub bounds: Rect,
    pub asset: HallAsset,
    pub font: HallFont,
    pub selected: bool,
    pub focused: bool,
    pub hovered: bool,
    pub pressed: bool,
}

/// Hall-specific paint intentions, in paint order. Bubble rendering is supplied
/// at its marker through the existing BubbleMgr public render contract.
#[derive(Clone, Debug, PartialEq)]
pub enum HallDraw {
    Fill {
        bounds: Rect,
        color: HallColor,
    },
    CallerBubbles,
    Image {
        asset: HallAsset,
        frame: ImageFrame,
        bounds: Rect,
        additive: bool,
    },
    Text(TextRun),
    Button(ButtonView),
}

#[derive(Clone, Debug, PartialEq)]
pub struct DrawOp {
    pub layer: Layer,
    pub clip: Rect,
    pub draw: HallDraw,
}

#[derive(Clone, Debug, PartialEq)]
pub struct HallScene {
    pub page: Page,
    pub bounds: Rect,
    pub ops: Vec<DrawOp>,
}

/// Configure/warm the caller's scene-local manager on entry; update it once per
/// caller tick. This module does not construct a manager or draw/consume RNG.
pub fn bubble_config() -> BubbleConfig {
    BubbleConfig {
        bubble_bounds: BubbleRect::new(0, 60, 640, 420),
        fish_bounds: BubbleRect::new(0, 80, 640, 350),
        max_bubbles: 20,
        bubble_spawn_chance: 3,
        max_fish: 3,
        fish_spawn_chance: 3,
        ..BubbleConfig::default()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HallInput {
    PointerMove(Point),
    PointerPress(Point),
    PointerRelease(Point),
    ButtonPress(Control),
    ButtonRelease(Control),
    CancelPress,
    FocusNext { backward: bool },
    Activate(Control),
    ActivateFocused,
    Escape,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundIntent {
    ButtonClick,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Intentions {
    pub sound: Option<SoundIntent>,
    /// Caller retains this transient page for the next screen entry.
    pub remember_page: Option<Page>,
    pub exit_to_menu: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HallOfFame {
    page: Page,
    focus: Control,
    hover: Option<Control>,
    pressed: Option<Control>,
}

impl Default for HallOfFame {
    fn default() -> Self {
        Self::new(Page::TimeTrial)
    }
}

impl HallOfFame {
    pub fn new(remembered_page: Page) -> Self {
        Self {
            page: remembered_page,
            focus: Control::Tab(remembered_page),
            hover: None,
            pressed: None,
        }
    }

    pub fn page(&self) -> Page {
        self.page
    }

    /// Mouse coordinates are in the640x480 logical scene. Button height comes
    /// from the same owned MainButton metric used by `view`.
    pub fn input(&mut self, input: HallInput, button_height: f32) -> Intentions {
        match input {
            HallInput::PointerMove(point) => {
                self.hover = hit_test(point, button_height);
                Intentions::default()
            }
            HallInput::PointerPress(point) => {
                let control = hit_test(point, button_height);
                self.hover = control;
                self.press(control)
            }
            HallInput::PointerRelease(point) => {
                let control = hit_test(point, button_height);
                self.hover = control;
                self.release(control)
            }
            HallInput::ButtonPress(control) => self.press(Some(control)),
            HallInput::ButtonRelease(control) => self.release(Some(control)),
            HallInput::CancelPress => {
                self.pressed = None;
                self.hover = None;
                Intentions::default()
            }
            HallInput::FocusNext { backward } => {
                let index = match self.focus {
                    Control::Tab(page) => page.index(),
                    Control::Menu => 4,
                };
                self.focus = Control::ALL[(index + if backward { 4 } else { 1 }) % 5];
                self.pressed = None;
                Intentions::default()
            }
            HallInput::Activate(control) => self.activate(control, true),
            HallInput::ActivateFocused => self.activate(self.focus, true),
            HallInput::Escape => self.activate(Control::Menu, true),
        }
    }

    fn press(&mut self, control: Option<Control>) -> Intentions {
        let changed = control.is_some() && self.pressed != control;
        self.pressed = control;
        if let Some(control) = control {
            self.focus = control;
        }
        Intentions {
            sound: changed.then_some(SoundIntent::ButtonClick),
            ..Intentions::default()
        }
    }

    fn release(&mut self, control: Option<Control>) -> Intentions {
        let pressed = self.pressed.take();
        match control.filter(|_| pressed == control) {
            Some(control) => self.activate(control, false),
            None => Intentions::default(),
        }
    }

    fn activate(&mut self, control: Control, sound: bool) -> Intentions {
        self.pressed = None;
        self.focus = control;
        let mut intentions = Intentions {
            sound: sound.then_some(SoundIntent::ButtonClick),
            ..Intentions::default()
        };
        match control {
            Control::Tab(page) => {
                self.page = page;
                intentions.remember_page = Some(page);
            }
            Control::Menu => intentions.exit_to_menu = true,
        }
        intentions
    }

    pub fn view(
        &self,
        snapshot: ScoreSnapshot<'_>,
        metrics: &impl HallMetrics,
        clock: HallClock,
    ) -> Result<HallScene, ViewError> {
        snapshot
            .global
            .validate()
            .map_err(ViewError::InvalidScores)?;
        let button = checked_image(metrics, HallAsset::MainButton)?;
        if !valid_button_height(button.height) {
            return Err(ViewError::InvalidMetric("MainButton height"));
        }
        let title = checked_image(metrics, HallAsset::ScreenTitle)?;
        let hole = checked_image(metrics, HallAsset::ScreenTitleHole)?;
        let wave = checked_image(metrics, HallAsset::Wave)?;
        let wave_frame = metrics.wave_frame(clock.tick);
        if wave_frame >= wave.frames {
            return Err(ViewError::InvalidMetric("Wave animation cel"));
        }
        for font in [
            HallFont::Title,
            HallFont::Page,
            HallFont::PageOutline,
            HallFont::Heading,
            HallFont::Row,
        ] {
            let height = metrics.font_height(font);
            if !height.is_finite() || !(0.0..=80.0).contains(&height) || height == 0.0 {
                return Err(ViewError::InvalidMetric(font.resource_key()));
            }
        }
        let mut builder = ViewBuilder {
            metrics,
            clock,
            ops: Vec::new(),
        };
        for band in 0..120 {
            builder.push(
                Layer::Background,
                SCREEN,
                HallDraw::Fill {
                    bounds: Rect::new(0.0, (band * 4) as f32, 640.0, 4.0),
                    color: HallColor::Hsl(151, 138, (107_i32 - band).max(50) as u8),
                },
            );
        }
        builder.push(Layer::Decoration, SCREEN, HallDraw::CallerBubbles);
        for tile in 0..4 {
            builder.push(
                Layer::Decoration,
                SCREEN,
                HallDraw::Image {
                    asset: HallAsset::Wave,
                    frame: ImageFrame::Cel(wave_frame),
                    bounds: Rect::new((tile * 160) as f32, 60.0, wave.width, wave.height),
                    additive: true,
                },
            );
        }
        if self.page == Page::Personal {
            builder.personal(snapshot.personal)?;
        } else {
            builder.global(self.page, snapshot.global, snapshot.finished_game)?;
        }
        let mut page_style = TextStyle::new(HallFont::Page, HallColor::Rgb(255, 255, 255));
        page_style.outline = Some(HallFont::PageOutline);
        page_style.bob = false;
        builder.text(self.page.title(), Anchor::Center(320.0), 60.0, page_style)?;
        for (asset, bounds) in [
            (
                HallAsset::ScreenTitle,
                Rect::new(0.0, 0.0, 640.0, title.height),
            ),
            (
                HallAsset::ScreenTitleHole,
                Rect::new(524.0, 3.0, 82.0, hole.height),
            ),
        ] {
            builder.push(
                Layer::Header,
                SCREEN,
                HallDraw::Image {
                    asset,
                    frame: ImageFrame::ImageBox,
                    bounds,
                    additive: false,
                },
            );
        }
        let mut title_style = TextStyle::new(HallFont::Title, HallColor::Rgb(255, 200, 0));
        title_style.layer = Layer::Header;
        title_style.bob = false;
        builder.text("Hall of Fame", Anchor::Center(320.0), 25.0, title_style)?;
        for control in Control::ALL {
            let bounds = control_bounds(control, button.height);
            // Validate label metrics as well as row metrics before handing any
            // draw intentions to the caller. Actual button chrome stays there.
            checked_width(metrics, HallFont::Heading, control.label())?;
            builder.push(
                Layer::Controls,
                SCREEN,
                HallDraw::Button(ButtonView {
                    control,
                    bounds,
                    asset: HallAsset::MainButton,
                    font: HallFont::Heading,
                    selected: control == Control::Tab(self.page),
                    focused: control == self.focus,
                    hovered: Some(control) == self.hover,
                    pressed: Some(control) == self.pressed,
                }),
            );
        }
        Ok(HallScene {
            page: self.page,
            bounds: SCREEN,
            ops: builder.ops,
        })
    }
}

fn valid_button_height(height: f32) -> bool {
    height.is_finite() && height > 0.0 && height <= 80.0
}

pub fn control_bounds(control: Control, button_height: f32) -> Rect {
    match control {
        Control::Tab(page) => Rect::new(
            20.0 + page.index() as f32 * 160.0,
            470.0 - button_height,
            120.0,
            button_height,
        ),
        Control::Menu => Rect::new(525.0, 4.0, 80.0, button_height),
    }
}

pub fn hit_test(point: Point, button_height: f32) -> Option<Control> {
    if !valid_button_height(button_height) {
        return None;
    }
    Control::ALL
        .into_iter()
        .find(|&control| control_bounds(control, button_height).contains(point))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ViewError {
    InvalidScores(String),
    InvalidMetric(&'static str),
}

impl fmt::Display for ViewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidScores(error) => write!(formatter, "Hall of Fame score snapshot: {error}"),
            Self::InvalidMetric(metric) => write!(
                formatter,
                "Hall of Fame metric unavailable/invalid: {metric}"
            ),
        }
    }
}

impl std::error::Error for ViewError {}

fn checked_image(metrics: &impl HallMetrics, asset: HallAsset) -> Result<ImageMetrics, ViewError> {
    let image = metrics.image(asset);
    if !image.width.is_finite()
        || !image.height.is_finite()
        || !(0.0..=16_384.0).contains(&image.width)
        || image.width == 0.0
        || !(0.0..=480.0).contains(&image.height)
        || image.height == 0.0
        || image.frames == 0
    {
        return Err(ViewError::InvalidMetric(asset.resource_key()));
    }
    Ok(image)
}

fn checked_width(metrics: &impl HallMetrics, font: HallFont, text: &str) -> Result<f32, ViewError> {
    let width = metrics.text_width(font, text);
    if !width.is_finite() || !(0.0..=65_536.0).contains(&width) {
        return Err(ViewError::InvalidMetric(font.resource_key()));
    }
    Ok(width)
}

#[derive(Clone, Copy)]
enum Anchor {
    Left(f32),
    Center(f32),
    Right(f32),
}

#[derive(Clone, Copy)]
struct TextStyle {
    font: HallFont,
    outline: Option<HallFont>,
    color: HallColor,
    layer: Layer,
    clip: Rect,
    bob: bool,
}

impl TextStyle {
    fn new(font: HallFont, color: HallColor) -> Self {
        Self {
            font,
            outline: None,
            color,
            layer: Layer::Content,
            clip: SCREEN,
            bob: true,
        }
    }
}

struct ViewBuilder<'a, M: HallMetrics> {
    metrics: &'a M,
    clock: HallClock,
    ops: Vec<DrawOp>,
}

impl<M: HallMetrics> ViewBuilder<'_, M> {
    fn push(&mut self, layer: Layer, clip: Rect, draw: HallDraw) {
        self.ops.push(DrawOp { layer, clip, draw });
    }

    fn text(
        &mut self,
        text: impl Into<String>,
        anchor: Anchor,
        baseline: f32,
        style: TextStyle,
    ) -> Result<(), ViewError> {
        let text = text.into();
        let width = checked_width(self.metrics, style.font, &text)?;
        let x = match anchor {
            Anchor::Left(x) => x,
            Anchor::Center(x) => x - width / 2.0,
            Anchor::Right(x) => x - width,
        };
        let bob = if style.bob && self.clock.bob_text {
            ((f64::from(baseline) * 2.0 + (self.clock.tick % 120) as f64 * 3.0)
                .to_radians()
                .sin()
                * 3.0) as f32
        } else {
            0.0
        };
        self.push(
            style.layer,
            style.clip,
            HallDraw::Text(TextRun {
                text,
                font: style.font,
                outline: style.outline,
                baseline: Point {
                    x,
                    y: baseline + bob,
                },
                color: style.color,
            }),
        );
        Ok(())
    }

    fn heading(
        &mut self,
        text: impl Into<String>,
        center: f32,
        baseline: f32,
    ) -> Result<(), ViewError> {
        self.text(
            text,
            Anchor::Center(center),
            baseline,
            TextStyle::new(HallFont::Heading, HallColor::Rgb(255, 255, 100)),
        )
    }

    fn row_style(&self, baseline: f32) -> TextStyle {
        let phase = ((self.clock.tick % 200) + baseline as u64) % 200;
        let triangle = if phase > 100 { 200 - phase } else { phase };
        TextStyle::new(
            HallFont::Row,
            HallColor::Hsl(159, 119, (triangle * 70 / 100 + 180) as u8),
        )
    }

    fn named_row(
        &mut self,
        entry: Option<&super::ScoreEntry>,
        unit: ScoreUnit,
        x: f32,
        baseline: f32,
    ) -> Result<(), ViewError> {
        let value = format_value(unit, RecordScope::Global, entry.map(|row| row.score as u64));
        let width = checked_width(self.metrics, HallFont::Row, &value)?;
        let mut name_style = self.row_style(baseline);
        name_style.clip = Rect::new(x, baseline - 30.0, (200.0 - width - 8.0).max(0.0), 40.0);
        self.text(
            entry.map_or(MISSING_RECORD, |row| row.name.as_str()),
            Anchor::Left(x),
            baseline,
            name_style,
        )?;
        let mut score_style = self.row_style(baseline);
        score_style.clip = Rect::new(x, baseline - 30.0, 200.0, 40.0);
        self.text(value, Anchor::Right(x + 200.0), baseline, score_style)
    }

    fn global(
        &mut self,
        page: Page,
        scores: &GlobalScores,
        finished: bool,
    ) -> Result<(), ViewError> {
        let bottom = if page == Page::Adventure && finished {
            240.0
        } else {
            280.0
        };
        let kind = match page {
            Page::Adventure => ScoreKind::Adventure,
            Page::TimeTrial => ScoreKind::TimeTrial,
            _ => ScoreKind::Challenge,
        };
        let unit = if page == Page::TimeTrial {
            ScoreUnit::Money
        } else {
            ScoreUnit::ElapsedSeconds
        };
        for tank in 1..=4 {
            let x = if tank % 2 == 1 { 60.0 } else { 380.0 };
            let mut baseline = if tank <= 2 { 100.0 } else { bottom };
            self.heading(format!("Tank {tank}"), x + 100.0, baseline)?;
            baseline += self.metrics.font_height(HallFont::Heading);
            for row in 0..5 {
                let entry = if page == Page::Adventure {
                    let style = self.row_style(baseline);
                    self.text(
                        format!("{tank}-{}", row + 1),
                        Anchor::Left(x - 30.0),
                        baseline,
                        style,
                    )?;
                    scores
                        .rows(kind, tank, Some(row + 1))
                        .map_err(ViewError::InvalidScores)?
                        .first()
                } else {
                    scores
                        .rows(kind, tank, None)
                        .map_err(ViewError::InvalidScores)?
                        .get(usize::from(row))
                };
                self.named_row(entry, unit, x, baseline)?;
                baseline += 15.0;
            }
        }
        if page == Page::Adventure && finished {
            self.heading("Tank 5 · Pets saved", 320.0, 365.0)?;
            let mut baseline = 365.0 + self.metrics.font_height(HallFont::Heading);
            for entry in scores
                .rows(ScoreKind::Adventure, 5, Some(1))
                .map_err(ViewError::InvalidScores)?
            {
                self.named_row(Some(entry), ScoreUnit::SurvivingPets, 220.0, baseline)?;
                baseline += 15.0;
            }
        }
        Ok(())
    }

    fn personal(&mut self, records: &PersonalRecords) -> Result<(), ViewError> {
        self.heading("Adventure", 320.0, 110.0)?;
        self.heading("Time Trial", 173.0, 290.0)?;
        self.heading("Challenge", 466.0, 290.0)?;
        for (tank, levels) in records.adventure_seconds.iter().enumerate() {
            let x = 20.0 + tank as f32 * 166.0;
            for (level, value) in levels.iter().enumerate() {
                let baseline = 140.0 + level as f32 * 15.0;
                self.personal_row(
                    format!("{}-{}", tank + 1, level + 1),
                    *value,
                    ScoreUnit::ElapsedSeconds,
                    Rect::new(x, baseline, 100.0, 15.0),
                )?;
            }
        }
        if let Some(survivors) = records.tank5_survivors {
            let style = self.row_style(220.0);
            self.text(
                "Final boss",
                Anchor::Left(220.0),
                220.0,
                TextStyle {
                    color: HallColor::Rgb(255, 255, 136),
                    ..style
                },
            )?;
            self.text(
                format_value(
                    ScoreUnit::SurvivingPets,
                    RecordScope::Personal,
                    Some(u64::from(survivors)),
                ),
                Anchor::Left(320.0),
                220.0,
                style,
            )?;
        }
        for tank in 0..4 {
            let baseline = 315.0 + tank as f32 * 15.0;
            self.personal_row(
                format!("Tank {}", tank + 1),
                records.time_trial_money[tank].map(u64::from),
                ScoreUnit::Money,
                Rect::new(83.0, baseline, 170.0, 15.0),
            )?;
            self.personal_row(
                format!("Tank {}", tank + 1),
                records.challenge_seconds[tank],
                ScoreUnit::ElapsedSeconds,
                Rect::new(376.0, baseline, 170.0, 15.0),
            )?;
        }
        Ok(())
    }

    fn personal_row(
        &mut self,
        label: String,
        value: Option<u64>,
        unit: ScoreUnit,
        row: Rect,
    ) -> Result<(), ViewError> {
        let style = self.row_style(row.y);
        let label_width = checked_width(self.metrics, HallFont::Row, &label)?;
        self.text(
            label,
            Anchor::Left(row.x),
            row.y,
            TextStyle {
                color: HallColor::Rgb(255, 255, 136),
                ..style
            },
        )?;
        // Clip unusually wide values within this column, leaving label readable.
        let clip = Rect::new(
            row.x + label_width + 4.0,
            row.y - 30.0,
            (row.width - label_width - 4.0).max(0.0),
            40.0,
        );
        self.text(
            format_value(unit, RecordScope::Personal, value),
            Anchor::Right(row.x + row.width),
            row.y,
            TextStyle { clip, ..style },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Metrics;

    impl HallMetrics for Metrics {
        fn text_width(&self, _: HallFont, text: &str) -> f32 {
            text.chars().count() as f32 * 6.0
        }
        fn font_height(&self, _: HallFont) -> f32 {
            18.0
        }
        fn image(&self, asset: HallAsset) -> ImageMetrics {
            ImageMetrics {
                width: 160.0,
                height: if asset == HallAsset::MainButton {
                    30.0
                } else {
                    40.0
                },
                frames: 12,
            }
        }
        fn wave_frame(&self, tick: u64) -> u16 {
            (tick % 12) as u16
        }
    }

    fn texts(scene: &HallScene) -> Vec<&str> {
        scene
            .ops
            .iter()
            .filter_map(|op| match &op.draw {
                HallDraw::Text(run) => Some(run.text.as_str()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn score_formats_preserve_zero_hours_money_and_survivor_units() {
        let time =
            |value| format_value(ScoreUnit::ElapsedSeconds, RecordScope::Global, Some(value));
        assert_eq!(time(0), "0:00");
        assert_eq!(time(3300), "55:00");
        assert_eq!(time(3599), "59:59");
        assert_eq!(time(3600), "01:00:00");
        assert_eq!(time(3661), "01:01:01");
        assert_eq!(
            format_value(ScoreUnit::Money, RecordScope::Global, Some(1000)),
            "1000"
        );
        assert_eq!(
            format_value(ScoreUnit::Money, RecordScope::Personal, Some(1000)),
            "1,000"
        );
        assert_eq!(
            format_value(ScoreUnit::Money, RecordScope::Personal, Some(u64::MAX)),
            "18,446,744,073,709,551,615"
        );
        assert_eq!(
            format_value(ScoreUnit::SurvivingPets, RecordScope::Global, Some(22)),
            "22 / 18"
        );
        assert_eq!(
            format_value(ScoreUnit::ElapsedSeconds, RecordScope::Personal, None),
            MISSING_RECORD
        );
    }

    #[test]
    fn views_use_immutable_snapshots_and_keep_empty_records_safe() {
        let mut global = GlobalScores::default();
        global
            .record(ScoreKind::Adventure, 1, Some(1), "Player", 61)
            .unwrap();
        global
            .record(ScoreKind::Adventure, 5, Some(1), "Saved", 18)
            .unwrap();
        global
            .record(ScoreKind::TimeTrial, 1, None, "Money", 1000)
            .unwrap();
        let personal = PersonalRecords {
            time_trial_money: [Some(0), Some(1000), None, None],
            challenge_seconds: [Some(61), None, None, None],
            ..PersonalRecords::default()
        };
        let original = global.clone();
        let mut snapshot = ScoreSnapshot {
            global: &global,
            personal: &personal,
            finished_game: false,
        };
        let screen = HallOfFame::new(Page::Adventure);
        let scene = screen
            .view(snapshot, &Metrics, HallClock::default())
            .unwrap();
        assert!(texts(&scene).contains(&"1:01"));
        assert!(texts(&scene).contains(&MISSING_RECORD));
        assert!(!texts(&scene).contains(&"18 / 18"));
        snapshot.finished_game = true;
        let finished = screen
            .view(
                snapshot,
                &Metrics,
                HallClock {
                    tick: u64::MAX,
                    bob_text: true,
                },
            )
            .unwrap();
        assert!(texts(&finished).contains(&"18 / 18"));
        let scene = HallOfFame::new(Page::Personal)
            .view(snapshot, &Metrics, HallClock::default())
            .unwrap();
        assert!(texts(&scene).contains(&"0"));
        assert!(texts(&scene).contains(&"1,000"));
        assert!(texts(&scene).contains(&"1:01"));
        assert_eq!(
            texts(&scene)
                .iter()
                .filter(|&&text| text == MISSING_RECORD)
                .count(),
            25
        );
        assert_eq!(global, original);
        let invalid: GlobalScores = serde_json::from_value(serde_json::json!({
            "adventure": vec![Vec::<super::super::ScoreEntry>::new(); 21],
            "time_trial": [[{"name":"Bad", "score":-1}], [], [], []],
            "challenge": [[], [], [], []]
        }))
        .unwrap();
        assert!(matches!(
            screen.view(
                ScoreSnapshot {
                    global: &invalid,
                    ..snapshot
                },
                &Metrics,
                HallClock::default()
            ),
            Err(ViewError::InvalidScores(_))
        ));
    }

    #[test]
    fn tab_menu_gestures_emit_once_and_remember_selected_page() {
        let mut screen = HallOfFame::default();
        let personal = Point { x: 510.0, y: 450.0 };
        let first = screen.input(HallInput::PointerPress(personal), 30.0);
        assert_eq!(first.sound, Some(SoundIntent::ButtonClick));
        assert_eq!(
            screen.input(HallInput::PointerPress(personal), 30.0),
            Intentions::default()
        );
        assert_eq!(screen.page(), Page::TimeTrial);
        let released = screen.input(HallInput::PointerRelease(personal), 30.0);
        assert_eq!(released.remember_page, Some(Page::Personal));
        assert_eq!(released.sound, None);
        assert_eq!(HallOfFame::new(screen.page()).page(), Page::Personal);
        screen.input(HallInput::ButtonPress(Control::Menu), 30.0);
        assert!(
            !screen
                .input(HallInput::PointerRelease(personal), 30.0)
                .exit_to_menu
        );
        screen.input(HallInput::FocusNext { backward: true }, 30.0);
        screen.input(HallInput::FocusNext { backward: false }, 30.0);
        assert!(screen.input(HallInput::ActivateFocused, 30.0).exit_to_menu);
        assert!(screen.input(HallInput::Escape, 30.0).exit_to_menu);
        assert_eq!(
            hit_test(
                Point {
                    x: f32::NAN,
                    y: 450.0
                },
                30.0
            ),
            None
        );
        assert_eq!(hit_test(personal, f32::NAN), None);
        assert_eq!(Page::from_index(4), None);
    }
}
