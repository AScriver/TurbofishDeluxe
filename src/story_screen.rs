//! Transient Story screen composition, informed by pinned W1 StoryScreen.
//! Earned eligibility remains in `story`; this module never writes game state.
//! See docs/story-screen.md for source references and deliberate project policies.

use crate::story::{StoryBrowser, StoryEntry, StoryId, StoryState};
use std::fmt;

/// Project work limits. The recognized catalog is already limited to 80/1024
/// bytes per title/body; these also bound independently supplied UI content.
pub const MAX_TITLE_BYTES: usize = 256;
pub const MAX_BODY_BYTES: usize = 8_192;
pub const MAX_BODY_LINES: usize = 512;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StoryPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StoryRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl StoryRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(self, point: StoryPoint) -> bool {
        point.x >= self.x
            && point.y >= self.y
            && point.x < self.x + self.width
            && point.y < self.y + self.height
    }
}

pub const STORY_VIEWPORT: StoryRect = StoryRect::new(0.0, 0.0, 640.0, 480.0);
const WHITE: [u8; 4] = [255, 255, 255, 255];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StoryPortrait {
    Stinky,
    Niko,
    Itchy,
    Prego,
    Zorf,
    Clyde,
    Vert,
    Rufus,
    Meryl,
    Wadsworth,
    Seymour,
    Shrapnel,
    Gumbo,
    Blip,
    Rhubarb,
    Nimbus,
    Amp,
    Gash,
    Angie,
    Presto,
    Brinkley,
    Nostradamus,
    Stanley,
    Walter,
    Sylvester,
    Balrog,
    Gus,
    Destructor,
    Ulysses,
    Psychosquid,
    Bilaterus,
    Minisylvester,
    Cyrax,
}

impl StoryPortrait {
    pub const ALL: [Self; 33] = [
        Self::Stinky,
        Self::Niko,
        Self::Itchy,
        Self::Prego,
        Self::Zorf,
        Self::Clyde,
        Self::Vert,
        Self::Rufus,
        Self::Meryl,
        Self::Wadsworth,
        Self::Seymour,
        Self::Shrapnel,
        Self::Gumbo,
        Self::Blip,
        Self::Rhubarb,
        Self::Nimbus,
        Self::Amp,
        Self::Gash,
        Self::Angie,
        Self::Presto,
        Self::Brinkley,
        Self::Nostradamus,
        Self::Stanley,
        Self::Walter,
        Self::Sylvester,
        Self::Balrog,
        Self::Gus,
        Self::Destructor,
        Self::Ulysses,
        Self::Psychosquid,
        Self::Bilaterus,
        Self::Minisylvester,
        Self::Cyrax,
    ];

    pub fn for_story(id: StoryId) -> Self {
        Self::ALL[usize::from(id.get())]
    }

    pub const fn resource_id(self) -> &'static str {
        match self {
            Self::Stinky => "IMAGE_STINKY",
            Self::Niko => "IMAGE_NIKO",
            Self::Itchy => "IMAGE_ITCHY",
            Self::Prego => "IMAGE_PREGO",
            Self::Zorf => "IMAGE_ZORF",
            Self::Clyde => "IMAGE_CLYDE",
            Self::Vert => "IMAGE_VERT",
            Self::Rufus => "IMAGE_RUFUS",
            Self::Meryl => "IMAGE_MERYL",
            Self::Wadsworth => "IMAGE_WADSWORTH",
            Self::Seymour => "IMAGE_SEYMOUR",
            Self::Shrapnel => "IMAGE_SHRAPNEL",
            Self::Gumbo => "IMAGE_GUMBO",
            Self::Blip => "IMAGE_BLIP",
            Self::Rhubarb => "IMAGE_RHUBARB",
            Self::Nimbus => "IMAGE_NIMBUS",
            Self::Amp => "IMAGE_AMP",
            Self::Gash => "IMAGE_GASH",
            Self::Angie => "IMAGE_ANGIE",
            Self::Presto => "IMAGE_PRESTO",
            Self::Brinkley => "IMAGE_BRINKLEY",
            Self::Nostradamus => "IMAGE_NOSTRADAMUS",
            Self::Stanley => "IMAGE_STANLEY",
            Self::Walter => "IMAGE_WALTER",
            Self::Sylvester => "IMAGE_SYLV",
            Self::Balrog => "IMAGE_BALROG",
            Self::Gus => "IMAGE_GUS",
            Self::Destructor => "IMAGE_DESTRUCTOR",
            Self::Ulysses => "IMAGE_ULYSSES",
            Self::Psychosquid => "IMAGE_PSYCHOSQUID",
            Self::Bilaterus => "IMAGE_BILATERUS",
            Self::Minisylvester => "IMAGE_MINISYLV",
            Self::Cyrax => "IMAGE_BOSS",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StoryAsset {
    Portrait(StoryPortrait),
    TalkShow,
}

impl StoryAsset {
    /// Talkshow is an install-relative image stem, not a manifest resource ID.
    pub const fn resource_key(self) -> &'static str {
        match self {
            Self::Portrait(portrait) => portrait.resource_id(),
            Self::TalkShow => "images/talkshow",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoryImageMetrics {
    pub cel_width: u32,
    pub cel_height: u32,
    pub columns: u16,
    pub rows: u16,
}

impl StoryImageMetrics {
    fn valid(self) -> bool {
        self.cel_width > 0 && self.cel_height > 0 && self.columns > 0 && self.rows > 0
    }
}

/// Read-only frontend adapter; no asset decoding or fabricated animation here.
pub trait StoryAssets {
    fn image_metrics(&self, asset: StoryAsset) -> Option<StoryImageMetrics>;

    /// Resolve the owned resource's GetAnimCel-equivalent at this explicit
    /// update count. A verified resource without animation metadata returns 0.
    /// Unavailable/unimplemented metadata returns None, not an invented loop.
    fn animation_cel(&self, portrait: StoryPortrait, updates: u64) -> Option<u16>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StoryFont {
    Liddie18,
    Liddie15,
    ContinuumBold14,
    JungleFever17Outline,
    JungleFever15Outline,
    ContinuumBold12Outline,
    ContinuumBold12,
    JungleFever10Outline,
}

impl StoryFont {
    pub const fn resource_id(self) -> &'static str {
        match self {
            Self::Liddie18 => "FONT_LIDDIE18",
            Self::Liddie15 => "FONT_LIDDIE15",
            Self::ContinuumBold14 => "FONT_CONTINUUMBOLD14",
            Self::JungleFever17Outline => "FONT_JUNGLEFEVER17OUTLINE",
            Self::JungleFever15Outline => "FONT_JUNGLEFEVER15OUTLINE",
            Self::ContinuumBold12Outline => "FONT_CONTINUUMBOLD12OUTLINE",
            Self::ContinuumBold12 => "FONT_CONTINUUMBOLD12",
            Self::JungleFever10Outline => "FONT_JUNGLEFEVER10OUTLINE",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StoryFontMetrics {
    pub height: f32,
    pub ascent: f32,
    pub line_advance: f32,
}

pub trait StoryTextMetrics {
    fn font_metrics(&self, font: StoryFont) -> Option<StoryFontMetrics>;
    fn text_width(&self, font: StoryFont, text: &str) -> Option<f32>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoryLayoutError {
    FontUnavailable(StoryFont),
    InvalidFontMetrics(StoryFont),
    BodyLineTooTall(StoryFont),
}

impl fmt::Display for StoryLayoutError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "Story layout unavailable: {self:?}")
    }
}

impl std::error::Error for StoryLayoutError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoryAssetIssue {
    Missing(StoryAsset),
    InvalidGrid(StoryAsset),
    AnimationUnavailable(StoryPortrait),
    InvalidFrame(StoryPortrait),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoryFrame {
    pub column: u16,
    pub row: u16,
    pub mirror_x: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StorySprite {
    pub asset: StoryAsset,
    pub frame: StoryFrame,
    /// Natural cel dimensions: the source does not scale the portrait.
    pub destination: StoryRect,
    pub clip: StoryRect,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StoryBubbleSpec {
    pub bounds: StoryRect,
    pub max_bubbles: u32,
    pub spawn_chance: u32,
    /// Initialization intention: run the owning BubbleMgr's UpdateALot once.
    /// Composing this descriptor must not repeat prewarming.
    pub prewarm_once: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StoryBackground {
    Ordinary {
        color: [u8; 4],
        bubbles: StoryBubbleSpec,
    },
    TalkShow(StorySprite),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoryLayer {
    Background,
    Heading,
    Title,
    Portrait,
    Body,
    Chrome,
    PageCounter,
    Controls,
}

pub const STORY_PAINT_ORDER: [StoryLayer; 8] = [
    StoryLayer::Background,
    StoryLayer::Heading,
    StoryLayer::Title,
    StoryLayer::Portrait,
    StoryLayer::Body,
    StoryLayer::Chrome,
    StoryLayer::PageCounter,
    StoryLayer::Controls,
];

#[derive(Clone, Debug, PartialEq)]
pub struct StoryLabel {
    pub text: String,
    pub font: StoryFont,
    pub color: [u8; 4],
    pub baseline: StoryPoint,
    pub clip: StoryRect,
    pub clipped: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StoryTextBlock {
    pub font: StoryFont,
    pub color: [u8; 4],
    pub clip: StoryRect,
    pub lines: Vec<StoryLabel>,
    pub first_line: usize,
    pub total_lines: usize,
    pub max_scroll: usize,
    pub text_truncated: bool,
    pub horizontal_overflow: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoryControlId {
    Previous,
    Next,
    Menu,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StoryControl {
    pub id: StoryControlId,
    pub label: &'static str,
    pub bounds: StoryRect,
    pub font: StoryFont,
    pub normal_color: [u8; 4],
    pub hover_color: [u8; 4],
    pub visible: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StoryFill {
    pub bounds: StoryRect,
    pub color: [u8; 4],
}

#[derive(Clone, Debug, PartialEq)]
pub struct StoryScreenView {
    pub story: StoryId,
    pub background: StoryBackground,
    pub heading: StoryLabel,
    pub title: Option<StoryLabel>,
    pub portraits: Vec<StorySprite>,
    pub body: Option<StoryTextBlock>,
    pub chrome: [StoryFill; 2],
    pub page_counter: StoryLabel,
    pub controls: [StoryControl; 3],
    pub asset_issues: Vec<StoryAssetIssue>,
    pub catalog_unavailable: bool,
}

impl StoryScreenView {
    pub fn hit_control(&self, point: StoryPoint) -> Option<StoryControlId> {
        self.controls
            .iter()
            .find(|control| control.visible && control.bounds.contains(point))
            .map(|control| control.id)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoryClockPolicy {
    FocusedOnly,
    WidgetUpdates,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StoryUpdate {
    pub portrait_updates: u32,
    pub bubble_updates: u32,
    pub redraw: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoryDirection {
    Previous,
    Next,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoryAction {
    Changed {
        from: StoryId,
        to: StoryId,
        direction: StoryDirection,
    },
    ReturnToCaller,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorySound {
    ButtonClick,
}

impl StorySound {
    pub const fn resource_id(self) -> &'static str {
        "SOUND_BUTTONCLICK"
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoryMusicIntent {
    StopStoryMusic,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StoryIntent {
    pub sound: Option<StorySound>,
    pub music: Option<StoryMusicIntent>,
    pub action: Option<StoryAction>,
    pub scroll_changed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StoryInput {
    PointerDown(StoryPoint),
    PointerUp(StoryPoint),
    CancelPress,
    /// Caller keyboard/accessibility activation. Press sound and release action
    /// are combined; physical pointer input should use Down/Up instead.
    Activate(StoryControlId),
    /// Positive lines scroll down. Wheel/PageUp/PageDown mapping is caller-owned.
    ScrollLines(i32),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoryScreenController {
    browser: StoryBrowser,
    updates: u64,
    scroll: usize,
    pressed: Option<StoryControlId>,
}

impl StoryScreenController {
    pub fn new(browser: StoryBrowser) -> Self {
        Self {
            browser,
            updates: 0,
            scroll: 0,
            pressed: None,
        }
    }

    pub fn current(&self) -> StoryId {
        self.browser.current
    }

    pub fn update_count(&self) -> u64 {
        self.updates
    }

    /// Caller owns Challenge/selector transitions. Sync a changed authoritative
    /// page without awarding it or restarting the source's portrait clock.
    pub fn sync_browser(&mut self, browser: StoryBrowser) {
        if self.browser != browser {
            self.browser = browser;
            self.scroll = 0;
            self.pressed = None;
        }
    }

    pub fn advance(
        &mut self,
        updates: u32,
        focused: bool,
        policy: StoryClockPolicy,
    ) -> StoryUpdate {
        if !focused {
            self.pressed = None;
        }
        let portrait_updates = if focused || policy == StoryClockPolicy::WidgetUpdates {
            updates
        } else {
            0
        };
        self.updates = self.updates.saturating_add(u64::from(portrait_updates));
        StoryUpdate {
            portrait_updates,
            bubble_updates: if focused { updates } else { 0 },
            redraw: focused && updates > 0,
        }
    }

    /// Pure projection. Catalog entry must correspond to `current()`; the caller
    /// obtains it from the existing identity-checked StoryCatalog.
    pub fn view(
        &self,
        earned: &StoryState,
        entry: Option<&StoryEntry>,
        assets: &impl StoryAssets,
        metrics: &impl StoryTextMetrics,
    ) -> Result<StoryScreenView, StoryLayoutError> {
        let id = self.current();
        let mut issues = Vec::new();
        let background = background(id, assets, &mut issues);
        let talkshow = matches!(background, StoryBackground::TalkShow(_));
        let (heading_font, title_font, body_font) = if talkshow {
            (
                StoryFont::JungleFever17Outline,
                StoryFont::JungleFever15Outline,
                StoryFont::ContinuumBold12Outline,
            )
        } else {
            (
                StoryFont::Liddie18,
                StoryFont::Liddie15,
                StoryFont::ContinuumBold14,
            )
        };
        let title_clip = StoryRect::new(0.0, 40.0, 640.0, 96.0);
        let heading = centered_label(
            "The Story Of",
            heading_font,
            if talkshow {
                WHITE
            } else {
                [215, 255, 255, 255]
            },
            80.0,
            title_clip,
            metrics,
        )?;
        let title = entry
            .map(|entry| {
                centered_label(
                    &entry.title,
                    title_font,
                    if talkshow {
                        WHITE
                    } else {
                        [150, 250, 150, 255]
                    },
                    110.0,
                    title_clip,
                    metrics,
                )
            })
            .transpose()?;
        let body_clip = if talkshow {
            StoryRect::new(50.0, 330.0, 540.0, 110.0)
        } else {
            StoryRect::new(50.0, 280.0, 540.0, 160.0)
        };
        let body = entry
            .map(|entry| body_block(&entry.text, body_font, body_clip, self.scroll, metrics))
            .transpose()?;
        let portraits = if talkshow {
            Vec::new()
        } else {
            portraits(id, self.updates, assets, &mut issues)
        };
        let page_counter = centered_label(
            &format!("{} of 33", u16::from(id.get()) + 1),
            StoryFont::ContinuumBold12,
            WHITE,
            28.0,
            StoryRect::new(220.0, 0.0, 200.0, 40.0),
            metrics,
        )?;
        let controls = controls(earned.unlocked_count() > 1, metrics)?;
        Ok(StoryScreenView {
            story: id,
            background,
            heading,
            title,
            portraits,
            body,
            chrome: [
                StoryFill {
                    bounds: StoryRect::new(0.0, 0.0, 640.0, 40.0),
                    color: [0, 0, 0, 255],
                },
                StoryFill {
                    bounds: StoryRect::new(0.0, 440.0, 640.0, 40.0),
                    color: [0, 0, 0, 255],
                },
            ],
            page_counter,
            controls,
            asset_issues: issues,
            catalog_unavailable: entry.is_none(),
        })
    }

    /// Feed only active-screen, focused input. A stale page view cannot activate
    /// its controls. All output is an intention; caller owns actual audio/exit.
    pub fn input(
        &mut self,
        input: StoryInput,
        earned: &StoryState,
        view: &StoryScreenView,
    ) -> StoryIntent {
        if view.story != self.current() {
            self.pressed = None;
            return StoryIntent::default();
        }
        match input {
            StoryInput::PointerDown(point) => {
                self.pressed = view
                    .hit_control(point)
                    .filter(|id| control_allowed(*id, earned));
                StoryIntent {
                    sound: self.pressed.map(|_| StorySound::ButtonClick),
                    ..StoryIntent::default()
                }
            }
            StoryInput::PointerUp(point) => {
                let pressed = self.pressed.take();
                if let Some(id) = pressed.filter(|id| view.hit_control(point) == Some(*id)) {
                    self.activate(id, earned)
                } else {
                    StoryIntent::default()
                }
            }
            StoryInput::CancelPress => {
                self.pressed = None;
                StoryIntent::default()
            }
            StoryInput::Activate(id) => {
                self.pressed = None;
                if !view
                    .controls
                    .iter()
                    .any(|control| control.id == id && control.visible)
                    || !control_allowed(id, earned)
                {
                    return StoryIntent::default();
                }
                let mut intent = self.activate(id, earned);
                intent.sound = Some(StorySound::ButtonClick);
                intent
            }
            StoryInput::ScrollLines(lines) => {
                let Some(body) = &view.body else {
                    return StoryIntent::default();
                };
                let current = self.scroll.min(body.max_scroll);
                let next = if lines >= 0 {
                    current.saturating_add(lines as usize)
                } else {
                    current.saturating_sub(lines.unsigned_abs() as usize)
                };
                self.scroll = next.min(body.max_scroll);
                StoryIntent {
                    scroll_changed: self.scroll != current,
                    ..StoryIntent::default()
                }
            }
        }
    }

    fn activate(&mut self, id: StoryControlId, earned: &StoryState) -> StoryIntent {
        if !control_allowed(id, earned) {
            return StoryIntent::default();
        }
        if id == StoryControlId::Menu {
            return StoryIntent {
                action: Some(StoryAction::ReturnToCaller),
                music: Some(StoryMusicIntent::StopStoryMusic),
                ..StoryIntent::default()
            };
        }
        let from = self.current();
        let direction = if id == StoryControlId::Previous {
            StoryDirection::Previous
        } else {
            StoryDirection::Next
        };
        let to = self.browser.step(
            earned,
            if direction == StoryDirection::Previous {
                -1
            } else {
                1
            },
        );
        if from == to {
            return StoryIntent::default();
        }
        self.scroll = 0;
        StoryIntent {
            action: Some(StoryAction::Changed {
                from,
                to,
                direction,
            }),
            ..StoryIntent::default()
        }
    }
}

fn control_allowed(id: StoryControlId, earned: &StoryState) -> bool {
    id == StoryControlId::Menu || earned.unlocked_count() > 1
}

fn image_metrics(
    asset: StoryAsset,
    assets: &impl StoryAssets,
    issues: &mut Vec<StoryAssetIssue>,
) -> Option<StoryImageMetrics> {
    match assets.image_metrics(asset) {
        None => {
            issues.push(StoryAssetIssue::Missing(asset));
            None
        }
        Some(image) if !image.valid() => {
            issues.push(StoryAssetIssue::InvalidGrid(asset));
            None
        }
        Some(image) => Some(image),
    }
}

fn sprite(
    asset: StoryAsset,
    image: StoryImageMetrics,
    x: f32,
    y: f32,
    column: u16,
    row: u16,
    mirror_x: bool,
) -> StorySprite {
    StorySprite {
        asset,
        frame: StoryFrame {
            column,
            row,
            mirror_x,
        },
        destination: StoryRect::new(x, y, image.cel_width as f32, image.cel_height as f32),
        clip: STORY_VIEWPORT,
    }
}

fn background(
    id: StoryId,
    assets: &impl StoryAssets,
    issues: &mut Vec<StoryAssetIssue>,
) -> StoryBackground {
    if id.get() == 32
        && let Some(image) = image_metrics(StoryAsset::TalkShow, assets, issues)
    {
        if image.columns == 1 && image.rows == 1 {
            return StoryBackground::TalkShow(sprite(
                StoryAsset::TalkShow,
                image,
                0.0,
                0.0,
                0,
                0,
                false,
            ));
        }
        issues.push(StoryAssetIssue::InvalidGrid(StoryAsset::TalkShow));
    }
    StoryBackground::Ordinary {
        color: [25, 105, 195, 255],
        bubbles: StoryBubbleSpec {
            bounds: StoryRect::new(0.0, -20.0, 640.0, 520.0),
            max_bubbles: 10,
            spawn_chance: 3,
            prewarm_once: true,
        },
    }
}

fn portraits(
    id: StoryId,
    updates: u64,
    assets: &impl StoryAssets,
    issues: &mut Vec<StoryAssetIssue>,
) -> Vec<StorySprite> {
    let portrait = StoryPortrait::for_story(id);
    let asset = StoryAsset::Portrait(portrait);
    let Some(image) = image_metrics(asset, assets, issues) else {
        return Vec::new();
    };
    if portrait == StoryPortrait::Bilaterus {
        if image.columns < 10 || image.rows < 8 {
            issues.push(StoryAssetIssue::InvalidGrid(asset));
            return Vec::new();
        }
        let timer = (updates % 20) as u16;
        let head = if timer > 9 { 19 - timer } else { timer };
        return [
            (395.0, 135.0, timer / 2, 5, true),
            (360.0, 145.0, 2, 7, false),
            (330.0, 140.0, 2, 6, false),
            (300.0, 145.0, 2, 6, false),
            (270.0, 155.0, 2, 7, false),
            (240.0, 160.0, 2, 6, false),
            (210.0, 155.0, 2, 7, false),
            (180.0, 150.0, head, 2, false),
        ]
        .map(|(x, y, column, row, mirror)| sprite(asset, image, x, y, column, row, mirror))
        .to_vec();
    }
    let Some(column) = assets.animation_cel(portrait, updates) else {
        issues.push(StoryAssetIssue::AnimationUnavailable(portrait));
        return Vec::new();
    };
    let row = u16::from(portrait == StoryPortrait::Rufus);
    if column >= image.columns || row >= image.rows {
        issues.push(StoryAssetIssue::InvalidFrame(portrait));
        return Vec::new();
    }
    // Match the source's integer half-cel centering, including odd dimensions.
    vec![sprite(
        asset,
        image,
        320.0 - (image.cel_width / 2) as f32,
        190.0 - (image.cel_height / 2) as f32,
        column,
        row,
        false,
    )]
}

fn font_metrics(
    font: StoryFont,
    metrics: &impl StoryTextMetrics,
) -> Result<StoryFontMetrics, StoryLayoutError> {
    let value = metrics
        .font_metrics(font)
        .ok_or(StoryLayoutError::FontUnavailable(font))?;
    if !value.height.is_finite()
        || value.height <= 0.0
        || !value.ascent.is_finite()
        || value.ascent < 0.0
        || value.ascent > value.height
        || !value.line_advance.is_finite()
        || value.line_advance <= 0.0
    {
        return Err(StoryLayoutError::InvalidFontMetrics(font));
    }
    Ok(value)
}

fn text_width(
    font: StoryFont,
    text: &str,
    metrics: &impl StoryTextMetrics,
) -> Result<f32, StoryLayoutError> {
    let value = metrics
        .text_width(font, text)
        .ok_or(StoryLayoutError::FontUnavailable(font))?;
    if !value.is_finite() || value < 0.0 {
        return Err(StoryLayoutError::InvalidFontMetrics(font));
    }
    Ok(value)
}

fn bounded_text(text: &str, limit: usize) -> (&str, bool) {
    let mut end = text.len().min(limit);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    (&text[..end], end < text.len())
}

fn centered_label(
    text: &str,
    font: StoryFont,
    color: [u8; 4],
    y: f32,
    clip: StoryRect,
    metrics: &impl StoryTextMetrics,
) -> Result<StoryLabel, StoryLayoutError> {
    let (text, truncated) = bounded_text(text, MAX_TITLE_BYTES);
    let width = text_width(font, text, metrics)?;
    Ok(StoryLabel {
        text: text.to_owned(),
        font,
        color,
        baseline: StoryPoint {
            x: 320.0 - width / 2.0,
            y,
        },
        clip,
        clipped: truncated || width > clip.width,
    })
}

fn controls(
    navigation_visible: bool,
    metrics: &impl StoryTextMetrics,
) -> Result<[StoryControl; 3], StoryLayoutError> {
    let font = StoryFont::JungleFever10Outline;
    let height = font_metrics(font, metrics)?.height;
    let mut controls = Vec::with_capacity(3);
    for (id, label) in [
        (StoryControlId::Previous, "Back"),
        (StoryControlId::Next, "Next"),
        (StoryControlId::Menu, "Back To Main Menu"),
    ] {
        let width = text_width(font, label, metrics)? + 20.0;
        let (x, y) = match id {
            StoryControlId::Previous => (220.0 - width, 15.0),
            StoryControlId::Next => (420.0, 15.0),
            StoryControlId::Menu => (630.0 - width, 460.0 - (height / 2.0).floor()),
        };
        controls.push(StoryControl {
            id,
            label,
            bounds: StoryRect::new(x, y, width, height),
            font,
            normal_color: [128, 128, 128, 255],
            hover_color: WHITE,
            visible: id == StoryControlId::Menu || navigation_visible,
        });
    }
    Ok([controls[0], controls[1], controls[2]])
}

fn body_block(
    text: &str,
    font: StoryFont,
    clip: StoryRect,
    scroll: usize,
    metrics: &impl StoryTextMetrics,
) -> Result<StoryTextBlock, StoryLayoutError> {
    let info = font_metrics(font, metrics)?;
    if info.height > clip.height {
        return Err(StoryLayoutError::BodyLineTooTall(font));
    }
    let visible = (((clip.height - info.height) / info.line_advance).floor() + 1.0)
        .min(MAX_BODY_LINES as f32) as usize;
    let (wrapped, truncated, horizontal_overflow) = wrap_text(text, font, clip.width, metrics)?;
    let total_lines = wrapped.len();
    let max_scroll = total_lines.saturating_sub(visible);
    let first_line = scroll.min(max_scroll);
    let lines = wrapped
        .into_iter()
        .skip(first_line)
        .take(visible)
        .enumerate()
        .map(|(index, text)| StoryLabel {
            text,
            font,
            color: WHITE,
            baseline: StoryPoint {
                x: clip.x,
                y: clip.y + info.ascent + index as f32 * info.line_advance,
            },
            clip,
            clipped: horizontal_overflow,
        })
        .collect();
    Ok(StoryTextBlock {
        font,
        color: WHITE,
        clip,
        lines,
        first_line,
        total_lines,
        max_scroll,
        text_truncated: truncated,
        horizontal_overflow,
    })
}

fn emit_line(lines: &mut Vec<String>, line: &mut String) -> bool {
    if lines.len() == MAX_BODY_LINES {
        return false;
    }
    lines.push(std::mem::take(line));
    true
}

fn wrap_text(
    text: &str,
    font: StoryFont,
    width: f32,
    metrics: &impl StoryTextMetrics,
) -> Result<(Vec<String>, bool, bool), StoryLayoutError> {
    let (text, mut truncated) = bounded_text(text, MAX_BODY_BYTES);
    let mut lines = Vec::new();
    let mut horizontal_overflow = false;
    'paragraphs: for paragraph in text.split('\n') {
        let mut line = String::new();
        for word in paragraph.split_whitespace() {
            let candidate = if line.is_empty() {
                word.to_owned()
            } else {
                format!("{line} {word}")
            };
            if text_width(font, &candidate, metrics)? <= width {
                line = candidate;
                continue;
            }
            if !line.is_empty() && !emit_line(&mut lines, &mut line) {
                truncated = true;
                break 'paragraphs;
            }
            if text_width(font, word, metrics)? <= width {
                line = word.to_owned();
                continue;
            }
            // Split oversized tokens at UTF-8 character boundaries. An oversized
            // individual glyph remains clipped and flagged, without a retry loop.
            for character in word.chars() {
                let mut candidate = line.clone();
                candidate.push(character);
                if !line.is_empty()
                    && text_width(font, &candidate, metrics)? > width
                    && !emit_line(&mut lines, &mut line)
                {
                    truncated = true;
                    break 'paragraphs;
                }
                line.push(character);
                horizontal_overflow |= text_width(font, &line, metrics)? > width;
            }
        }
        if !emit_line(&mut lines, &mut line) {
            truncated = true;
            break;
        }
    }
    Ok((lines, truncated, horizontal_overflow))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixtures {
        talkshow: bool,
        missing: Option<StoryPortrait>,
        animation: Option<u16>,
    }

    impl Default for Fixtures {
        fn default() -> Self {
            Self {
                talkshow: false,
                missing: None,
                animation: Some(3),
            }
        }
    }

    impl StoryAssets for Fixtures {
        fn image_metrics(&self, asset: StoryAsset) -> Option<StoryImageMetrics> {
            match asset {
                StoryAsset::TalkShow => self.talkshow.then_some(StoryImageMetrics {
                    cel_width: 640,
                    cel_height: 480,
                    columns: 1,
                    rows: 1,
                }),
                StoryAsset::Portrait(portrait) if self.missing == Some(portrait) => None,
                StoryAsset::Portrait(_) => Some(StoryImageMetrics {
                    cel_width: 61,
                    cel_height: 51,
                    columns: 10,
                    rows: 8,
                }),
            }
        }

        fn animation_cel(&self, _portrait: StoryPortrait, _updates: u64) -> Option<u16> {
            self.animation
        }
    }

    struct Metrics;

    impl StoryTextMetrics for Metrics {
        fn font_metrics(&self, _font: StoryFont) -> Option<StoryFontMetrics> {
            Some(StoryFontMetrics {
                height: 16.0,
                ascent: 12.0,
                line_advance: 20.0,
            })
        }

        fn text_width(&self, _font: StoryFont, text: &str) -> Option<f32> {
            Some(text.chars().count() as f32 * 8.0)
        }
    }

    fn controller(value: u8) -> StoryScreenController {
        StoryScreenController::new(StoryBrowser {
            current: StoryId::new(value).unwrap(),
        })
    }

    fn earned() -> StoryState {
        StoryState {
            seen_mask: u32::MAX,
            cyrax_unlocked: true,
        }
    }

    fn entry(text: &str) -> StoryEntry {
        StoryEntry {
            title: "Test title".to_owned(),
            text: text.to_owned(),
        }
    }

    #[test]
    fn complete_roster_uses_resource_frames_and_rufus_row() {
        let assets = Fixtures::default();
        let state = earned();
        let mut ids = std::collections::HashSet::new();
        for number in 0..33 {
            let view = controller(number)
                .view(&state, None, &assets, &Metrics)
                .unwrap();
            let portrait = StoryPortrait::for_story(view.story);
            assert!(ids.insert(portrait.resource_id()));
            assert_eq!(view.page_counter.text, format!("{} of 33", number + 1));
            assert!(view.catalog_unavailable);
            assert!(view.title.is_none() && view.body.is_none());
            if number != 30 {
                assert_eq!(view.portraits.len(), 1);
                let sprite = view.portraits[0];
                assert_eq!(sprite.asset, StoryAsset::Portrait(portrait));
                assert_eq!(
                    sprite.frame,
                    StoryFrame {
                        column: 3,
                        row: u16::from(number == 7),
                        mirror_x: false
                    }
                );
                assert_eq!(sprite.destination, StoryRect::new(290.0, 165.0, 61.0, 51.0));
            }
        }
        assert_eq!(ids.len(), 33);
        assert_eq!(
            StoryPortrait::for_story(StoryId::new(24).unwrap()),
            StoryPortrait::Sylvester
        );
        assert_eq!(
            StoryPortrait::for_story(StoryId::new(32).unwrap()).resource_id(),
            "IMAGE_BOSS"
        );
    }

    #[test]
    fn bilaterus_parts_preserve_distinct_head_and_tail_clocks() {
        let mut screen = controller(30);
        let assets = Fixtures {
            animation: None,
            ..Fixtures::default()
        };
        screen.advance(10, true, StoryClockPolicy::FocusedOnly);
        let view = screen.view(&earned(), None, &assets, &Metrics).unwrap();
        assert_eq!(view.portraits.len(), 8);
        assert!(view.asset_issues.is_empty());
        let parts: Vec<_> = view
            .portraits
            .iter()
            .map(|part| {
                (
                    part.destination.x as u32,
                    part.destination.y as u32,
                    part.frame.column,
                    part.frame.row,
                    part.frame.mirror_x,
                )
            })
            .collect();
        assert_eq!(
            parts,
            vec![
                (395, 135, 5, 5, true),
                (360, 145, 2, 7, false),
                (330, 140, 2, 6, false),
                (300, 145, 2, 6, false),
                (270, 155, 2, 7, false),
                (240, 160, 2, 6, false),
                (210, 155, 2, 7, false),
                (180, 150, 9, 2, false),
            ]
        );
        screen.advance(9, true, StoryClockPolicy::FocusedOnly);
        let view = screen.view(&earned(), None, &assets, &Metrics).unwrap();
        assert_eq!(view.portraits[0].frame.column, 9);
        assert_eq!(view.portraits[7].frame.column, 0);
        screen.advance(1, true, StoryClockPolicy::FocusedOnly);
        assert_eq!(
            screen
                .view(&earned(), None, &assets, &Metrics)
                .unwrap()
                .portraits[0]
                .frame
                .column,
            0
        );
    }

    #[test]
    fn cyrax_changes_scene_fonts_and_body_only_when_talkshow_exists() {
        let screen = controller(32);
        let text = entry("Caller supplied content");
        let assets = Fixtures {
            talkshow: true,
            missing: Some(StoryPortrait::Cyrax),
            ..Fixtures::default()
        };
        let view = screen
            .view(&earned(), Some(&text), &assets, &Metrics)
            .unwrap();
        assert!(matches!(view.background, StoryBackground::TalkShow(_)));
        assert!(view.portraits.is_empty() && view.asset_issues.is_empty());
        assert_eq!(view.heading.font, StoryFont::JungleFever17Outline);
        assert_eq!(view.title.unwrap().font, StoryFont::JungleFever15Outline);
        let body = view.body.unwrap();
        assert_eq!(body.font, StoryFont::ContinuumBold12Outline);
        assert_eq!(body.clip, StoryRect::new(50.0, 330.0, 540.0, 110.0));

        let assets = Fixtures::default();
        let view = screen
            .view(&earned(), Some(&text), &assets, &Metrics)
            .unwrap();
        assert!(matches!(
            view.background,
            StoryBackground::Ordinary {
                color: [25, 105, 195, 255],
                ..
            }
        ));
        assert_eq!(
            view.portraits[0].asset,
            StoryAsset::Portrait(StoryPortrait::Cyrax)
        );
        assert_eq!(view.heading.font, StoryFont::Liddie18);
        assert_eq!(
            view.body.unwrap().clip,
            StoryRect::new(50.0, 280.0, 540.0, 160.0)
        );
        assert_eq!(
            view.asset_issues,
            vec![StoryAssetIssue::Missing(StoryAsset::TalkShow)]
        );

        let assets = Fixtures {
            animation: None,
            ..Fixtures::default()
        };
        let view = controller(0)
            .view(&earned(), None, &assets, &Metrics)
            .unwrap();
        assert!(view.portraits.is_empty());
        assert_eq!(
            view.asset_issues,
            vec![StoryAssetIssue::AnimationUnavailable(StoryPortrait::Stinky)]
        );
        let assets = Fixtures {
            missing: Some(StoryPortrait::Stinky),
            ..Fixtures::default()
        };
        assert_eq!(
            controller(0)
                .view(&earned(), None, &assets, &Metrics)
                .unwrap()
                .asset_issues,
            vec![StoryAssetIssue::Missing(StoryAsset::Portrait(
                StoryPortrait::Stinky
            ))]
        );
    }

    #[test]
    fn composing_is_pure_and_focus_policy_is_explicit() {
        let mut screen = controller(7);
        let assets = Fixtures::default();
        let before = screen.clone();
        let first = screen.view(&earned(), None, &assets, &Metrics).unwrap();
        assert_eq!(
            first,
            screen.view(&earned(), None, &assets, &Metrics).unwrap()
        );
        assert_eq!(screen, before);
        assert_eq!(
            screen.advance(12, false, StoryClockPolicy::FocusedOnly),
            StoryUpdate::default()
        );
        assert_eq!(screen.update_count(), 0);
        assert_eq!(
            screen.advance(12, false, StoryClockPolicy::WidgetUpdates),
            StoryUpdate {
                portrait_updates: 12,
                bubble_updates: 0,
                redraw: false,
            }
        );
        assert_eq!(
            screen.advance(3, true, StoryClockPolicy::FocusedOnly),
            StoryUpdate {
                portrait_updates: 3,
                bubble_updates: 3,
                redraw: true,
            }
        );
        screen.sync_browser(StoryBrowser {
            current: StoryId::new(0).unwrap(),
        });
        assert_eq!(screen.update_count(), 15);
    }

    #[test]
    fn input_wraps_earned_pages_without_awards_and_menu_remains_caller_owned() {
        // Explicit project contract: ordinary bit plus separate Cyrax bit gives
        // two usable pages, despite source's ordinary-mask-only visibility quirk.
        let state = StoryState {
            seen_mask: 1,
            cyrax_unlocked: true,
        };
        let before = state.clone();
        let assets = Fixtures::default();
        let mut screen = controller(0);
        let view = screen.view(&state, None, &assets, &Metrics).unwrap();
        assert!(view.controls.iter().all(|control| control.visible));
        assert_eq!(
            view.controls[0].bounds,
            StoryRect::new(168.0, 15.0, 52.0, 16.0)
        );
        assert_eq!(
            view.controls[2].bounds,
            StoryRect::new(474.0, 452.0, 156.0, 16.0)
        );
        let next = StoryPoint { x: 430.0, y: 20.0 };
        let down = screen.input(StoryInput::PointerDown(next), &state, &view);
        assert_eq!(down.sound, Some(StorySound::ButtonClick));
        assert!(down.action.is_none());
        assert_eq!(
            screen.input(
                StoryInput::PointerUp(StoryPoint { x: 0.0, y: 0.0 }),
                &state,
                &view
            ),
            StoryIntent::default()
        );
        let intent = screen.input(
            StoryInput::Activate(StoryControlId::Previous),
            &state,
            &view,
        );
        assert_eq!(
            intent.action,
            Some(StoryAction::Changed {
                from: StoryId::new(0).unwrap(),
                to: StoryId::new(32).unwrap(),
                direction: StoryDirection::Previous,
            })
        );
        assert_eq!(
            screen.input(StoryInput::Activate(StoryControlId::Next), &state, &view),
            StoryIntent::default()
        );
        let view = screen.view(&state, None, &assets, &Metrics).unwrap();
        screen.input(StoryInput::Activate(StoryControlId::Next), &state, &view);
        assert_eq!(screen.current().get(), 0);
        let view = screen.view(&state, None, &assets, &Metrics).unwrap();
        let menu = StoryPoint { x: 500.0, y: 460.0 };
        screen.input(StoryInput::PointerDown(menu), &state, &view);
        let intent = screen.input(StoryInput::PointerUp(menu), &state, &view);
        assert_eq!(intent.action, Some(StoryAction::ReturnToCaller));
        assert_eq!(intent.music, Some(StoryMusicIntent::StopStoryMusic));
        assert!(intent.sound.is_none());
        assert_eq!(state, before);

        let single = StoryState {
            seen_mask: 0,
            cyrax_unlocked: true,
        };
        let mut screen = controller(32);
        let view = screen.view(&single, None, &assets, &Metrics).unwrap();
        assert!(!view.controls[0].visible && !view.controls[1].visible);
        assert_eq!(
            screen.input(StoryInput::Activate(StoryControlId::Next), &single, &view),
            StoryIntent::default()
        );
    }

    #[test]
    fn long_unicode_text_is_bounded_scrollable_and_clipped_above_chrome() {
        let state = earned();
        let assets = Fixtures::default();
        let mut screen = controller(0);
        let text = entry(&format!("First line\n\n{}", "水".repeat(3_000)));
        let view = screen.view(&state, Some(&text), &assets, &Metrics).unwrap();
        let body = view.body.as_ref().unwrap();
        assert_eq!(body.lines[0].text, "First line");
        assert_eq!(body.lines[1].text, "");
        assert!(body.text_truncated && body.max_scroll > 0);
        assert!(body.total_lines <= MAX_BODY_LINES);
        for line in &body.lines {
            assert!(Metrics.text_width(body.font, &line.text).unwrap() <= 540.0);
            assert!(line.baseline.y >= 280.0 && line.baseline.y < 440.0);
            assert_eq!(line.clip, body.clip);
        }
        // Multiple wheel events before the next composition accumulate.
        screen.input(StoryInput::ScrollLines(1), &state, &view);
        screen.input(StoryInput::ScrollLines(1), &state, &view);
        assert_eq!(
            screen
                .view(&state, Some(&text), &assets, &Metrics)
                .unwrap()
                .body
                .unwrap()
                .first_line,
            2
        );
        screen.input(StoryInput::ScrollLines(i32::MAX), &state, &view);
        assert_eq!(
            screen
                .view(&state, Some(&text), &assets, &Metrics)
                .unwrap()
                .body
                .unwrap()
                .first_line,
            body.max_scroll
        );
        screen.input(StoryInput::ScrollLines(i32::MIN), &state, &view);
        assert_eq!(
            screen
                .view(&state, Some(&text), &assets, &Metrics)
                .unwrap()
                .body
                .unwrap()
                .first_line,
            0
        );
        let text = entry(&"line\n".repeat(1_000));
        let body = screen
            .view(&state, Some(&text), &assets, &Metrics)
            .unwrap()
            .body
            .unwrap();
        assert_eq!(body.total_lines, MAX_BODY_LINES);
        assert!(body.text_truncated);
    }

    #[test]
    fn invalid_animation_and_font_metrics_are_reported() {
        let assets = Fixtures {
            animation: Some(10),
            ..Fixtures::default()
        };
        let view = controller(0)
            .view(&earned(), None, &assets, &Metrics)
            .unwrap();
        assert!(view.portraits.is_empty());
        assert_eq!(
            view.asset_issues,
            vec![StoryAssetIssue::InvalidFrame(StoryPortrait::Stinky)]
        );
        struct BadMetrics;
        impl StoryTextMetrics for BadMetrics {
            fn font_metrics(&self, font: StoryFont) -> Option<StoryFontMetrics> {
                Metrics.font_metrics(font)
            }
            fn text_width(&self, _font: StoryFont, _text: &str) -> Option<f32> {
                Some(f32::NAN)
            }
        }
        assert_eq!(
            controller(0).view(&earned(), None, &Fixtures::default(), &BadMetrics),
            Err(StoryLayoutError::InvalidFontMetrics(StoryFont::Liddie18))
        );
    }
}
