//! Transient Help examples and composition, informed by pinned WinFish HelpScreen.
//!
//! Source ranges, qualifications and frontend integration belong to docs/help.md.
//! This module samples its own clock; it never creates simulation actors or consumes RNG.

use super::{HelpReturn, HelpView, LAST_PAGE};
use std::time::Duration;

pub const CANVAS: HelpRect = HelpRect::new(0, 0, 640, 480);
const UPDATE_NANOS: u128 = crate::sim::TICK_MS as u128 * 1_000_000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DemoClock {
    updates: u64,
    remainder_nanos: u32,
}

impl DemoClock {
    pub fn updates(&self) -> u64 {
        self.updates
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// O(1), including long elapsed durations. No per-update actor work is hidden here.
    pub fn advance(&mut self, elapsed: Duration) -> u64 {
        let nanos = elapsed.as_nanos() + u128::from(self.remainder_nanos);
        let updates = (nanos / UPDATE_NANOS).min(u128::from(u64::MAX)) as u64;
        self.remainder_nanos = (nanos % UPDATE_NANOS) as u32;
        self.advance_ticks(updates);
        updates
    }

    pub fn advance_ticks(&mut self, updates: u64) {
        self.updates = self.updates.wrapping_add(updates);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HelpPoint {
    pub x: i32,
    pub y: i32,
}

impl HelpPoint {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HelpRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl HelpRect {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(self, point: HelpPoint) -> bool {
        point.x >= self.x
            && point.y >= self.y
            && i64::from(point.x) < i64::from(self.x) + i64::from(self.width)
            && i64::from(point.y) < i64::from(self.y) + i64::from(self.height)
    }

    fn canvas_clip(self) -> Self {
        let left = i64::from(self.x).clamp(0, 640);
        let top = i64::from(self.y).clamp(0, 480);
        let right = (i64::from(self.x) + i64::from(self.width)).clamp(left, 640);
        let bottom = (i64::from(self.y) + i64::from(self.height)).clamp(top, 480);
        Self::new(
            left as i32,
            top as i32,
            (right - left) as i32,
            (bottom - top) as i32,
        )
    }

    fn center(self) -> HelpPoint {
        HelpPoint::new(self.x + self.width / 2, self.y + self.height / 2)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HelpLayer {
    Background,
    Ambient,
    Panels,
    Examples,
    Text,
    Chrome,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HelpBlend {
    Normal,
    Additive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HelpTint {
    Rgba(u8, u8, u8, u8),
    /// PopCap's 0..255 HSL coordinates. Conversion is a renderer responsibility.
    Hsl {
        hue: u8,
        saturation: u8,
        luminance: u8,
    },
}

const WHITE: HelpTint = HelpTint::Rgba(255, 255, 255, 255);
const YELLOW: HelpTint = HelpTint::Rgba(255, 255, 0, 255);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HelpSprite {
    ScreenBack,
    ScreenTitle,
    ScreenTitleHole,
    FishBox,
    Bolt,
    HorizontalCrease,
    VerticalCrease,
    Wave,
    SmallSwim,
    SmallTurn,
    SmallEat,
    HungrySwim,
    SmallDie,
    Food,
    Money,
    Sylvester,
    HelpAlien,
    EggPieces,
    LaserUpgrades,
    Itchy,
    Stinky,
    PetSelection,
    TrophyButtons,
    Shell,
    SimSwimBase,
    SimSwimColor,
    SimSwimAccent,
    SimEatBase,
    SimEatColor,
    SimEatAccent,
    Ball,
    Screensaver,
    Cursor,
    MainButton,
    DialogButton,
}

impl HelpSprite {
    /// Preload these through the existing read-only asset owner before composing.
    pub const ALL: [Self; 35] = [
        Self::ScreenBack,
        Self::ScreenTitle,
        Self::ScreenTitleHole,
        Self::FishBox,
        Self::Bolt,
        Self::HorizontalCrease,
        Self::VerticalCrease,
        Self::Wave,
        Self::SmallSwim,
        Self::SmallTurn,
        Self::SmallEat,
        Self::HungrySwim,
        Self::SmallDie,
        Self::Food,
        Self::Money,
        Self::Sylvester,
        Self::HelpAlien,
        Self::EggPieces,
        Self::LaserUpgrades,
        Self::Itchy,
        Self::Stinky,
        Self::PetSelection,
        Self::TrophyButtons,
        Self::Shell,
        Self::SimSwimBase,
        Self::SimSwimColor,
        Self::SimSwimAccent,
        Self::SimEatBase,
        Self::SimEatColor,
        Self::SimEatAccent,
        Self::Ball,
        Self::Screensaver,
        Self::Cursor,
        Self::MainButton,
        Self::DialogButton,
    ];

    pub const fn asset_id(self) -> &'static str {
        match self {
            Self::ScreenBack => "IMAGE_SCREENBACK",
            Self::ScreenTitle => "IMAGE_SCREENTITLE",
            Self::ScreenTitleHole => "IMAGE_SCREENTITLEHOLE",
            Self::FishBox => "IMAGE_FISHBOX",
            Self::Bolt => "IMAGE_BOLT",
            Self::HorizontalCrease => "IMAGE_HORZCREASE",
            Self::VerticalCrease => "IMAGE_VERTCREASE",
            Self::Wave => "IMAGE_WAVE",
            Self::SmallSwim => "IMAGE_SMALLSWIM",
            Self::SmallTurn => "IMAGE_SMALLTURN",
            Self::SmallEat => "IMAGE_SMALLEAT",
            Self::HungrySwim => "IMAGE_HUNGRYSWIM",
            Self::SmallDie => "IMAGE_SMALLDIE",
            Self::Food => "IMAGE_FOOD",
            Self::Money => "IMAGE_MONEY",
            Self::Sylvester => "IMAGE_SYLV",
            Self::HelpAlien => "IMAGE_HELPALIEN",
            Self::EggPieces => "IMAGE_EGGPIECES",
            Self::LaserUpgrades => "IMAGE_LASERUPGRADES",
            Self::Itchy => "IMAGE_SCL_ITCHY",
            Self::Stinky => "IMAGE_SCL_STINKY",
            Self::PetSelection => "IMAGE_HELPPETCHOOSE",
            Self::TrophyButtons => "IMAGE_TROPHYBUTTONS",
            Self::Shell => "IMAGE_HELPSHELL",
            Self::SimSwimBase => "IMAGE_SIMSWIM1",
            Self::SimSwimColor => "IMAGE_SIMSWIM2",
            Self::SimSwimAccent => "IMAGE_SIMSWIM3",
            Self::SimEatBase => "IMAGE_SIMEAT1",
            Self::SimEatColor => "IMAGE_SIMEAT2",
            Self::SimEatAccent => "IMAGE_SIMEAT3",
            Self::Ball => "IMAGE_BALLS",
            Self::Screensaver => "IMAGE_HELPSCREENSAVER",
            Self::Cursor => "IMAGE_CURSOR_POINTER",
            Self::MainButton => "IMAGE_MAINBUTTON",
            Self::DialogButton => "IMAGE_DIALOGBUTTON",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HelpFont {
    ContinuumBold14,
    ContinuumBold14Outline,
    ContinuumBold12,
    JungleFever10Outline,
    JungleFever17Outline,
}

impl HelpFont {
    pub const ALL: [Self; 5] = [
        Self::ContinuumBold14,
        Self::ContinuumBold14Outline,
        Self::ContinuumBold12,
        Self::JungleFever10Outline,
        Self::JungleFever17Outline,
    ];

    pub const fn asset_id(self) -> &'static str {
        match self {
            Self::ContinuumBold14 => "FONT_CONTINUUMBOLD14",
            Self::ContinuumBold14Outline => "FONT_CONTINUUMBOLD14OUTLINE",
            Self::ContinuumBold12 => "FONT_CONTINUUMBOLD12",
            Self::JungleFever10Outline => "FONT_JUNGLEFEVER10OUTLINE",
            Self::JungleFever17Outline => "FONT_JUNGLEFEVER17OUTLINE",
        }
    }
}

/// Validated decoded dimensions and grid, supplied by the existing asset owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageMetrics {
    width: u16,
    height: u16,
    columns: u16,
    rows: u16,
}

impl ImageMetrics {
    pub fn new(width: u32, height: u32, columns: u32, rows: u32) -> Option<Self> {
        if width == 0
            || height == 0
            || columns == 0
            || rows == 0
            || width > u32::from(u16::MAX)
            || height > u32::from(u16::MAX)
            || !width.is_multiple_of(columns)
            || !height.is_multiple_of(rows)
        {
            return None;
        }
        Some(Self {
            width: width as u16,
            height: height as u16,
            columns: columns as u16,
            rows: rows as u16,
        })
    }

    pub fn full_rect(self) -> HelpRect {
        HelpRect::new(0, 0, i32::from(self.width), i32::from(self.height))
    }

    pub fn cell_rect(self, column: u16, row: u16) -> Option<HelpRect> {
        if column >= self.columns || row >= self.rows {
            return None;
        }
        let width = i32::from(self.width / self.columns);
        let height = i32::from(self.height / self.rows);
        Some(HelpRect::new(
            i32::from(column) * width,
            i32::from(row) * height,
            width,
            height,
        ))
    }

    fn includes(self, rect: HelpRect) -> bool {
        rect.x >= 0
            && rect.y >= 0
            && rect.width > 0
            && rect.height > 0
            && i64::from(rect.x) + i64::from(rect.width) <= i64::from(self.width)
            && i64::from(rect.y) + i64::from(rect.height) <= i64::from(self.height)
    }
}

pub trait HelpAssets {
    /// Return None for an unavailable/invalid resource; do not invent dimensions.
    fn image_metrics(&self, sprite: HelpSprite) -> Option<ImageMetrics>;

    /// The WAVE asset's GetAnimCel metadata is not guessed from its sheet size.
    fn wave_cell(&self, _updates: u64) -> Option<u16> {
        None
    }

    /// The secondary source bobs credits only on its accelerated graphics path.
    fn animate_credits(&self) -> bool {
        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HelpSource {
    Whole,
    Cell { column: u16, row: u16 },
    Pixels(HelpRect),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpriteDraw {
    pub sprite: HelpSprite,
    /// Resolved, checked crop; the renderer need not infer grid layout.
    pub source: HelpRect,
    pub destination: HelpRect,
    pub mirrored: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextDraw {
    pub text: String,
    pub font: HelpFont,
    pub outline: Option<HelpFont>,
    pub bounds: HelpRect,
    pub baseline: i32,
    pub line_height: u8,
    pub align: TextAlign,
    pub wrap: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HelpPrimitive {
    Fill(HelpRect),
    Outline(HelpRect),
    Line {
        start: HelpPoint,
        end: HelpPoint,
    },
    Sprite(SpriteDraw),
    /// Existing DrawImageBox/nine-slice treatment; dimensions are decoded, not bundled.
    ImageBox {
        sprite: HelpSprite,
        destination: HelpRect,
    },
    /// Repeat unscaled pixels and crop the last tile at the supplied destination.
    Tile {
        sprite: HelpSprite,
        destination: HelpRect,
        axis: TileAxis,
    },
    Text(TextDraw),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HelpDraw {
    pub layer: HelpLayer,
    pub clip: HelpRect,
    pub blend: HelpBlend,
    pub tint: HelpTint,
    pub primitive: HelpPrimitive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HelpAction {
    Continue,
    Return,
    Previous,
    Next,
    Page(u8),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HelpControl {
    pub action: HelpAction,
    pub bounds: HelpRect,
    pub label: String,
    pub selected: bool,
    pub font: HelpFont,
    pub sprite: HelpSprite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetIssueKind {
    MissingMetrics,
    CropOutsideImage,
    MissingAnimation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssetIssue {
    pub sprite: HelpSprite,
    pub kind: AssetIssueKind,
}

/// Configuration for the caller's ambient BubbleMgr, not an owned particle system.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HelpAmbient {
    pub draw_bubbles: bool,
    pub bubble_bounds: HelpRect,
    pub fish_bounds: Option<HelpRect>,
    /// Arguments to the pinned source SetBubbleConfig/SetBubbleFishConfig helpers.
    pub bubble_config: [u16; 2],
    pub fish_config: [u16; 2],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HelpRenderPlan {
    pub page: u8,
    pub updates: u64,
    pub ambient: HelpAmbient,
    /// Stable layer order, then original command order within a layer.
    pub draws: Vec<HelpDraw>,
    /// Chrome drawn/hit-tested by the frontend after draws. Coordinates match the plan.
    pub controls: Vec<HelpControl>,
    pub asset_issues: Vec<AssetIssue>,
}

impl HelpRenderPlan {
    pub fn control_at(&self, point: HelpPoint) -> Option<HelpAction> {
        self.controls
            .iter()
            .rev()
            .find(|control| control.bounds.contains(point))
            .map(|control| control.action)
    }
}

struct Composer<'a, A> {
    assets: &'a A,
    tick: u64,
    plan: HelpRenderPlan,
}

impl<A: HelpAssets> Composer<'_, A> {
    fn issue(&mut self, sprite: HelpSprite, kind: AssetIssueKind) {
        let issue = AssetIssue { sprite, kind };
        if !self.plan.asset_issues.contains(&issue) {
            self.plan.asset_issues.push(issue);
        }
    }

    fn crop(&mut self, sprite: HelpSprite, source: HelpSource) -> Option<HelpRect> {
        let Some(metrics) = self.assets.image_metrics(sprite) else {
            self.issue(sprite, AssetIssueKind::MissingMetrics);
            return None;
        };
        let crop = match source {
            HelpSource::Whole => Some(metrics.full_rect()),
            HelpSource::Cell { column, row } => metrics.cell_rect(column, row),
            HelpSource::Pixels(rect) => metrics.includes(rect).then_some(rect),
        };
        if crop.is_none() {
            self.issue(sprite, AssetIssueKind::CropOutsideImage);
        }
        crop
    }

    fn push(&mut self, layer: HelpLayer, clip: HelpRect, tint: HelpTint, primitive: HelpPrimitive) {
        self.plan.draws.push(HelpDraw {
            layer,
            clip: clip.canvas_clip(),
            blend: HelpBlend::Normal,
            tint,
            primitive,
        });
    }

    fn sprite(
        &mut self,
        sprite: HelpSprite,
        source: HelpSource,
        origin: HelpPoint,
        mirrored: bool,
        clip: HelpRect,
    ) {
        if let Some(crop) = self.crop(sprite, source) {
            self.push(
                HelpLayer::Examples,
                clip,
                WHITE,
                HelpPrimitive::Sprite(SpriteDraw {
                    sprite,
                    source: crop,
                    destination: HelpRect::new(origin.x, origin.y, crop.width, crop.height),
                    mirrored,
                }),
            );
        }
    }

    fn centered(
        &mut self,
        sprite: HelpSprite,
        source: HelpSource,
        area: HelpRect,
        offset: HelpPoint,
    ) {
        if let Some(crop) = self.crop(sprite, source) {
            let origin = HelpPoint::new(
                area.x + (area.width - crop.width) / 2 + offset.x,
                area.y + (area.height - crop.height) / 2 + offset.y,
            );
            self.sprite(sprite, HelpSource::Pixels(crop), origin, false, CANVAS);
        }
    }

    fn cell(&mut self, sprite: HelpSprite, column: u16, row: u16, origin: HelpPoint) {
        self.sprite(
            sprite,
            HelpSource::Cell { column, row },
            origin,
            false,
            CANVAS,
        );
    }

    fn image_box(&mut self, sprite: HelpSprite, area: HelpRect, layer: HelpLayer) {
        if self.crop(sprite, HelpSource::Whole).is_some() {
            self.push(
                layer,
                CANVAS,
                WHITE,
                HelpPrimitive::ImageBox {
                    sprite,
                    destination: area,
                },
            );
        }
    }

    fn text(
        &mut self,
        text: impl Into<String>,
        area: HelpRect,
        baseline: i32,
        align: TextAlign,
        font: HelpFont,
        tint: HelpTint,
    ) {
        self.push(
            HelpLayer::Text,
            area,
            tint,
            HelpPrimitive::Text(TextDraw {
                text: text.into(),
                font,
                outline: (font == HelpFont::ContinuumBold14)
                    .then_some(HelpFont::ContinuumBold14Outline),
                bounds: area,
                baseline,
                line_height: 15,
                align,
                wrap: true,
            }),
        );
    }

    fn click_burst(&mut self, center: HelpPoint, clip: HelpRect) {
        for (x, y) in [
            (-1, -1),
            (1, 1),
            (-1, 1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (0, 1),
            (0, -1),
        ] {
            self.push(
                HelpLayer::Examples,
                clip,
                WHITE,
                HelpPrimitive::Line {
                    start: HelpPoint::new(center.x + x * 3, center.y + y * 3),
                    end: HelpPoint::new(center.x + x * 8, center.y + y * 8),
                },
            );
        }
    }

    fn cursor(&mut self, center: HelpPoint, click: bool, clip: HelpRect) {
        if click {
            self.click_burst(center, clip);
        }
        if let Some(crop) = self.crop(HelpSprite::Cursor, HelpSource::Whole) {
            self.sprite(
                HelpSprite::Cursor,
                HelpSource::Pixels(crop),
                HelpPoint::new(center.x - crop.width / 2, center.y - crop.height / 2),
                false,
                clip,
            );
        }
    }

    fn fish_layers(
        &mut self,
        area: HelpRect,
        frame: u16,
        row: u16,
        eating: bool,
        colors: [HelpTint; 2],
    ) {
        let sprites = if eating {
            [
                HelpSprite::SimEatBase,
                HelpSprite::SimEatColor,
                HelpSprite::SimEatAccent,
            ]
        } else {
            [
                HelpSprite::SimSwimBase,
                HelpSprite::SimSwimColor,
                HelpSprite::SimSwimAccent,
            ]
        };
        let source = HelpSource::Pixels(HelpRect::new(
            i32::from(frame) * 80,
            i32::from(row) * 80,
            80,
            80,
        ));
        let origin = HelpPoint::new(
            area.x + (area.width - 80) / 2,
            area.y + (area.height - 80) / 2,
        );
        for (index, sprite) in sprites.into_iter().enumerate() {
            if let Some(crop) = self.crop(sprite, source) {
                self.plan.draws.push(HelpDraw {
                    layer: HelpLayer::Examples,
                    clip: CANVAS,
                    blend: if index == 0 {
                        HelpBlend::Normal
                    } else {
                        HelpBlend::Additive
                    },
                    tint: if index == 0 { WHITE } else { colors[index - 1] },
                    primitive: HelpPrimitive::Sprite(SpriteDraw {
                        sprite,
                        source: crop,
                        destination: HelpRect::new(origin.x, origin.y, 80, 80),
                        mirrored: false,
                    }),
                });
            }
        }
    }

    fn control(
        &mut self,
        action: HelpAction,
        area: HelpRect,
        label: impl Into<String>,
        selected: bool,
        sprite: HelpSprite,
    ) {
        self.plan.controls.push(HelpControl {
            action,
            bounds: area.canvas_clip(),
            label: label.into(),
            selected,
            font: HelpFont::JungleFever10Outline,
            sprite,
        });
    }
}

/// Directly constructed invalid HelpView pages are rejected without panicking.
pub fn compose(
    view: HelpView,
    clock: &DemoClock,
    assets: &impl HelpAssets,
) -> Option<HelpRenderPlan> {
    if view.page > LAST_PAGE {
        return None;
    }
    let mut composer = Composer {
        assets,
        tick: clock.updates(),
        plan: HelpRenderPlan {
            page: view.page,
            updates: clock.updates(),
            ambient: HelpAmbient {
                draw_bubbles: view.page != 0,
                bubble_bounds: if view.page == 0 {
                    HelpRect::new(0, 60, 640, 420)
                } else {
                    HelpRect::new(0, 80, 640, 400)
                },
                fish_bounds: (view.page != 0).then_some(HelpRect::new(0, 80, 640, 350)),
                bubble_config: [20, 3],
                fish_config: if view.page == 8 { [3, 3] } else { [0, 0] },
            },
            draws: Vec::new(),
            controls: Vec::new(),
            asset_issues: Vec::new(),
        },
    };
    if view.page == 0 {
        composer.image_box(
            HelpSprite::ScreenBack,
            HelpRect::new(-5, -5, 650, 490),
            HelpLayer::Background,
        );
        instructions(&mut composer, view);
    } else {
        ordinary_background(&mut composer);
        if view.page == 8 {
            credits(&mut composer, view);
        } else {
            ordinary_rows(&mut composer, view);
            ordinary_examples(&mut composer, view.page);
        }
    }
    chrome(&mut composer, view);
    composer.plan.draws.sort_by_key(|draw| draw.layer);
    Some(composer.plan)
}

fn row(index: u8) -> HelpRect {
    HelpRect::new(10, 160 + i32::from(index) * 65, 620, 50)
}

fn thumbnail(index: u8, inset: i32) -> HelpRect {
    HelpRect::new(22, row(index).y + inset, 75, 50 - inset * 2)
}

fn frame(tick: u64, interval: u64, count: u16) -> u16 {
    ((tick / interval) % u64::from(count)) as u16
}

fn radians(tick: u64, degrees_per_tick: u16) -> f64 {
    // Reducing angles keeps geometry bounded over long UI lifetimes. The secondary
    // source's approximate PI drift is not a claimed pixel-fidelity requirement.
    ((tick % 360) * u64::from(degrees_per_tick) % 360) as f64 * std::f64::consts::PI / 180.0
}

fn pulse(tick: u64, baseline: i32, hue: u8, saturation: u8) -> HelpTint {
    let phase = ((tick % 200) as i32 + baseline).rem_euclid(200);
    let triangle = phase.min(200 - phase);
    HelpTint::Hsl {
        hue,
        saturation,
        luminance: (triangle * 70 / 100 + 180) as u8,
    }
}

fn interpolate(start: i32, end: i32, progress: i32, duration: i32, swap: bool) -> i32 {
    let (start, end) = if swap { (end, start) } else { (start, end) };
    let progress = progress.clamp(0, duration);
    (start * (duration - progress) + end * progress) / duration
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FeedingSample {
    sprite: HelpSprite,
    frame: u16,
    mirrored: bool,
    fish: HelpPoint,
    food: Option<HelpPoint>,
    cursor: HelpPoint,
    click: bool,
}

fn feeding_sample(tick: u64, card: HelpRect) -> FeedingSample {
    let phase = (tick % 115) as i32;
    let swap = (tick / 115) % 2 == 1;
    let fish_y = (f64::from(card.y + 120) + f64::from(radians(tick, 5).sin() as f32 * 3.0)) as i32;
    let food_x = interpolate(card.x + 50, card.x + card.width - 50, phase, 30, swap);
    let fish_x = interpolate(card.x + 65, card.x + card.width - 60, phase - 45, 40, swap);
    let (sprite, column, mirrored) = match phase {
        45..=64 => (HelpSprite::SmallTurn, ((phase - 45) / 2) as u16, swap),
        75..=94 => (HelpSprite::SmallEat, ((phase - 75) / 2) as u16, !swap),
        65..=114 => (HelpSprite::SmallSwim, frame(tick, 2, 10), !swap),
        _ => (HelpSprite::SmallSwim, frame(tick, 2, 10), swap),
    };
    FeedingSample {
        sprite,
        frame: column,
        mirrored,
        fish: HelpPoint::new(fish_x - 37, fish_y - 40),
        food: (40..=84).contains(&phase).then(|| {
            HelpPoint::new(
                food_x - 10,
                interpolate(card.y + 30, fish_y, phase - 40, 45, false) - 20,
            )
        }),
        cursor: HelpPoint::new(food_x, card.y + 30),
        click: (40..=59).contains(&phase),
    }
}

fn cursor_orbit(tick: u64, center: HelpPoint) -> HelpPoint {
    HelpPoint::new(
        (f64::from(center.x) + f64::from(radians(tick, 2).cos() as f32 * 10.0)) as i32,
        (f64::from(center.y) + f64::from(radians(tick, 5).sin() as f32 * 10.0)) as i32,
    )
}

fn instructions<A: HelpAssets>(composer: &mut Composer<'_, A>, view: HelpView) {
    let tick = composer.tick;
    let cards = [
        HelpRect::new(30, 70, 185, 320),
        HelpRect::new(227, 70, 185, 320),
        HelpRect::new(424, 70, 185, 320),
    ];
    for card in cards {
        composer.image_box(HelpSprite::FishBox, card, HelpLayer::Panels);
    }
    let feeding = feeding_sample(tick, cards[0]);
    let left_clip = HelpRect::new(cards[0].x, cards[0].y, cards[0].width, 210);
    if let Some(food) = feeding.food {
        composer.sprite(
            HelpSprite::Food,
            HelpSource::Cell {
                column: frame(tick, 4, 10),
                row: 0,
            },
            food,
            false,
            left_clip,
        );
    }
    composer.sprite(
        feeding.sprite,
        HelpSource::Pixels(HelpRect::new(i32::from(feeding.frame) * 80, 0, 80, 80)),
        feeding.fish,
        feeding.mirrored,
        left_clip,
    );
    composer.cursor(feeding.cursor, feeding.click, left_clip);

    let card = cards[1];
    let combat_clip = HelpRect::new(card.x, card.y, card.width, 210);
    let center = HelpPoint::new(card.x + card.width / 2, card.y + 80);
    let origin = HelpPoint::new(
        center.x - 80,
        (f64::from(card.y + 20) + f64::from(radians(tick, 3).sin() as f32 * 5.0)) as i32,
    );
    let source = HelpSource::Pixels(HelpRect::new(
        i32::from(frame(tick, 3, 10)) * 160,
        0,
        160,
        160,
    ));
    composer.sprite(HelpSprite::Sylvester, source, origin, false, combat_clip);
    let phase = (tick % 40) as i32 - 20;
    if phase >= 0 {
        if phase < 10
            && let Some(crop) = composer.crop(HelpSprite::Sylvester, source)
        {
            composer.plan.draws.push(HelpDraw {
                layer: HelpLayer::Examples,
                clip: combat_clip,
                blend: HelpBlend::Additive,
                tint: HelpTint::Rgba(255, 255, 255, ((10 - phase) * 10) as u8),
                primitive: HelpPrimitive::Sprite(SpriteDraw {
                    sprite: HelpSprite::Sylvester,
                    source: crop,
                    destination: HelpRect::new(origin.x, origin.y, 160, 160),
                    mirrored: false,
                }),
            });
        }
        composer.click_burst(cursor_orbit(tick - phase as u64, center), combat_clip);
    }
    composer.cursor(cursor_orbit(tick, center), false, combat_clip);
    if let Some(egg) = composer.crop(HelpSprite::EggPieces, HelpSource::Whole) {
        composer.sprite(
            HelpSprite::EggPieces,
            HelpSource::Whole,
            HelpPoint::new(
                cards[2].x + (cards[2].width - egg.width) / 2,
                cards[2].y + 80,
            ),
            false,
            cards[2],
        );
    }

    // Existing authored instruction lines are retained, with an explicit egg caption.
    let captions = ["Feeding", "Defence", "Egg pieces"];
    let bodies = [
        format!(
            "{} {}",
            view.lines.first().copied().unwrap_or(""),
            view.lines.get(1).copied().unwrap_or("")
        ),
        view.lines.get(2).copied().unwrap_or("").to_owned(),
        "Buy three egg pieces to finish an Adventure level.".to_owned(),
    ];
    for (index, card) in cards.into_iter().enumerate() {
        composer.text(
            captions[index],
            HelpRect::new(card.x, card.y + 200, card.width, 32),
            card.y + 220,
            TextAlign::Center,
            HelpFont::ContinuumBold14,
            YELLOW,
        );
        composer.text(
            bodies[index].as_str(),
            HelpRect::new(card.x + 8, card.y + 237, card.width - 16, 72),
            card.y + 255,
            TextAlign::Center,
            HelpFont::JungleFever10Outline,
            WHITE,
        );
    }
    // Keep the existing Continue guidance above its button.
    composer.text(
        view.lines.get(3).copied().unwrap_or(""),
        HelpRect::new(30, 390, 580, 22),
        405,
        TextAlign::Center,
        HelpFont::JungleFever10Outline,
        WHITE,
    );

    for (x, width) in [(13, 23), (209, 24), (406, 24), (603, 24)] {
        tile(
            composer,
            HelpSprite::HorizontalCrease,
            HelpRect::new(x, 230, width, 0),
            TileAxis::Horizontal,
        );
    }
    for (x, y, height) in [(120, 33, 40), (515, 33, 40), (120, 386, 83), (515, 386, 83)] {
        tile(
            composer,
            HelpSprite::VerticalCrease,
            HelpRect::new(x, y, 0, height),
            TileAxis::Vertical,
        );
    }
    for y in [53, 406] {
        for x in [105, 135, 500, 530] {
            composer.sprite(
                HelpSprite::Bolt,
                HelpSource::Whole,
                HelpPoint::new(x, y),
                false,
                CANVAS,
            );
            if let Some(draw) = composer.plan.draws.last_mut()
                && matches!(draw.primitive, HelpPrimitive::Sprite(ref sprite) if sprite.sprite == HelpSprite::Bolt)
            {
                draw.layer = HelpLayer::Panels;
            }
        }
    }
}

fn tile<A: HelpAssets>(
    composer: &mut Composer<'_, A>,
    sprite: HelpSprite,
    mut area: HelpRect,
    axis: TileAxis,
) {
    if let Some(crop) = composer.crop(sprite, HelpSource::Whole) {
        match axis {
            TileAxis::Horizontal => area.height = crop.height,
            TileAxis::Vertical => area.width = crop.width,
        }
        composer.push(
            HelpLayer::Panels,
            CANVAS,
            WHITE,
            HelpPrimitive::Tile {
                sprite,
                destination: area,
                axis,
            },
        );
    }
}

fn ordinary_background<A: HelpAssets>(composer: &mut Composer<'_, A>) {
    for strip in 0..120 {
        composer.push(
            HelpLayer::Background,
            CANVAS,
            HelpTint::Hsl {
                hue: 151,
                saturation: 138,
                luminance: (107 - strip).max(50) as u8,
            },
            HelpPrimitive::Fill(HelpRect::new(0, strip * 4, 640, 4)),
        );
    }
    let Some(column) = composer.assets.wave_cell(composer.tick) else {
        composer.issue(HelpSprite::Wave, AssetIssueKind::MissingAnimation);
        return;
    };
    if let Some(source) = composer.crop(HelpSprite::Wave, HelpSource::Cell { column, row: 0 }) {
        for x in [0, 160, 320, 480] {
            composer.plan.draws.push(HelpDraw {
                layer: HelpLayer::Ambient,
                clip: CANVAS,
                blend: HelpBlend::Additive,
                tint: WHITE,
                primitive: HelpPrimitive::Sprite(SpriteDraw {
                    sprite: HelpSprite::Wave,
                    source,
                    destination: HelpRect::new(x, 80, source.width, source.height),
                    mirrored: false,
                }),
            });
        }
    }
}

fn row_guidance(view: HelpView, index: usize) -> String {
    let line = |idx| view.lines.get(idx).copied().unwrap_or("");
    match (view.page, index) {
        (2, 2) => "Food quantity raises the number of pellets you can drop at once.".to_owned(),
        (2, 3) => format!("{} {}", line(2), line(3)),
        (3, 1) => format!("Itchy attacks aliens. {}", line(1)),
        (3, 2) => "Stinky collects coins along the tank floor.".to_owned(),
        (3, 3) => line(2).to_owned(),
        (5, 1) => "Shells earned in game modes pay for Virtual Tank purchases.".to_owned(),
        (5, 2) => line(1).to_owned(),
        (5, 3) => line(2).to_owned(),
        (6, 1) => "Cared-for Virtual Tank fish grow over time.".to_owned(),
        (6, 2) => line(1).to_owned(),
        (6, 3) => format!("Your purchased fish remain protected. {}", line(2)),
        (7, 3) => "Saver preferences configure the disposable Virtual Tank display.".to_owned(),
        _ => line(index).to_owned(),
    }
}

fn ordinary_rows<A: HelpAssets>(composer: &mut Composer<'_, A>, view: HelpView) {
    let panel = HelpRect::new(10, 153, 620, 259);
    composer.push(
        HelpLayer::Panels,
        CANVAS,
        HelpTint::Rgba(0, 0, 0, 30),
        HelpPrimitive::Fill(panel),
    );
    composer.push(
        HelpLayer::Panels,
        CANVAS,
        HelpTint::Rgba(136, 170, 170, 255),
        HelpPrimitive::Outline(panel),
    );
    for index in 0..4 {
        let inset = if (view.page == 6 && index == 1) || (view.page == 7 && index == 3) {
            0
        } else {
            4
        };
        let icon = thumbnail(index, inset);
        composer.push(
            HelpLayer::Panels,
            CANVAS,
            HelpTint::Rgba(0, 0, 0, 96),
            HelpPrimitive::Fill(icon),
        );
        composer.push(
            HelpLayer::Panels,
            CANVAS,
            HelpTint::Rgba(49, 101, 132, 255),
            HelpPrimitive::Outline(icon),
        );
        if index < 3 {
            composer.push(
                HelpLayer::Panels,
                CANVAS,
                HelpTint::Rgba(136, 170, 170, 255),
                HelpPrimitive::Fill(HelpRect::new(10, row(index).y + 57, 620, 1)),
            );
        }
        let baseline = row(index).y + 20;
        composer.text(
            row_guidance(view, usize::from(index)),
            HelpRect::new(120, row(index).y + 3, 500, 47),
            baseline,
            TextAlign::Left,
            HelpFont::JungleFever10Outline,
            pulse(composer.tick, baseline, 159, 119),
        );
    }
    composer.text(
        "Learn by watching these examples.",
        HelpRect::new(10, 114, 620, 35),
        135,
        TextAlign::Center,
        HelpFont::JungleFever10Outline,
        HelpTint::Rgba(255, 255, 170, 255),
    );
}

fn ordinary_examples<A: HelpAssets>(composer: &mut Composer<'_, A>, page: u8) {
    let tick = composer.tick;
    match page {
        1 => {
            let center = thumbnail(0, 4).center();
            composer.cell(
                HelpSprite::HungrySwim,
                frame(tick, 3, 10),
                0,
                HelpPoint::new(center.x - 40, center.y - 40),
            );
            let center = thumbnail(1, 4).center();
            composer.cell(
                HelpSprite::Money,
                frame(tick, 2, 10),
                1,
                HelpPoint::new(center.x - 36, center.y - 36),
            );
            composer.centered(
                HelpSprite::HelpAlien,
                HelpSource::Whole,
                thumbnail(2, 4),
                HelpPoint::new(
                    (radians(tick, 2).cos() as f32 * 2.0 + 2.0) as i32,
                    (radians(tick, 4).sin() as f32 * 3.0 + 3.0) as i32,
                ),
            );
            let center = thumbnail(3, 4).center();
            composer.cell(
                HelpSprite::EggPieces,
                frame(tick, 36, 3),
                0,
                HelpPoint::new(center.x - 23, center.y - 18),
            );
        }
        2 => upgrades(composer),
        3 => {
            let center = thumbnail(0, 4).center();
            composer.cell(
                HelpSprite::EggPieces,
                2,
                0,
                HelpPoint::new(center.x - 23, center.y - 18),
            );
            for (index, sprite) in [(1, HelpSprite::Itchy), (2, HelpSprite::Stinky)] {
                let center = thumbnail(index, 4).center();
                composer.cell(
                    sprite,
                    frame(tick, 4, 10),
                    0,
                    HelpPoint::new(center.x - 30, center.y - 30),
                );
            }
            composer.centered(
                HelpSprite::PetSelection,
                HelpSource::Whole,
                thumbnail(3, 4),
                HelpPoint::new(0, 2),
            );
        }
        4 => {
            for (index, label) in [
                (0, "Adventure"),
                (1, "Time Trial"),
                (2, "Challenge"),
                (3, "Virtual\nTank"),
            ] {
                let area = thumbnail(index, 4);
                let baseline = area.y + if index == 3 { 18 } else { 30 };
                composer.text(
                    label,
                    area,
                    baseline,
                    TextAlign::Center,
                    HelpFont::ContinuumBold12,
                    pulse(tick, baseline, 40, 200),
                );
            }
        }
        5 => {
            composer.centered(
                HelpSprite::TrophyButtons,
                HelpSource::Cell { column: 2, row: 0 },
                thumbnail(0, 4),
                HelpPoint::new(0, 0),
            );
            composer.centered(
                HelpSprite::Shell,
                HelpSource::Whole,
                thumbnail(1, 4),
                HelpPoint::new(0, 0),
            );
            composer.fish_layers(
                thumbnail(2, 4),
                frame(tick, 2, 10),
                0,
                false,
                [
                    HelpTint::Rgba(50, 100, 200, 255),
                    HelpTint::Rgba(255, 245, 65, 255),
                ],
            );
            composer.centered(
                HelpSprite::Ball,
                HelpSource::Pixels(HelpRect::new(i32::from(frame(tick, 4, 10)) * 50, 0, 50, 50)),
                thumbnail(3, 4),
                HelpPoint::new(0, 0),
            );
        }
        6 => {
            let animation = frame(tick, 2, 27);
            let eating = animation < 10;
            let column = if eating {
                animation
            } else {
                (animation + 3) % 10
            };
            let colors = [
                HelpTint::Rgba(35, 85, 130, 255),
                HelpTint::Rgba(180, 245, 180, 255),
            ];
            composer.fish_layers(thumbnail(0, 4), column, 0, eating, colors);
            // Source intentionally centers row1's larger fish with the standard 4px inset,
            // despite drawing that row's background with inset0.
            composer.fish_layers(thumbnail(1, 4), frame(tick, 2, 10), 1, false, colors);
            composer.centered(
                HelpSprite::TrophyButtons,
                HelpSource::Cell { column: 1, row: 0 },
                thumbnail(2, 4),
                HelpPoint::new(0, 2),
            );
            composer.centered(
                HelpSprite::SmallDie,
                HelpSource::Pixels(HelpRect::new(720, 0, 80, 80)),
                thumbnail(3, 4),
                HelpPoint::new(0, 0),
            );
            let area = thumbnail(3, 4);
            composer.text(
                "?",
                area,
                area.y + 25,
                TextAlign::Center,
                HelpFont::ContinuumBold14,
                YELLOW,
            );
        }
        7 => {
            for (index, column, y_offset) in [(0, 3, 3), (1, 4, 0), (2, 5, 0)] {
                composer.centered(
                    HelpSprite::TrophyButtons,
                    HelpSource::Cell { column, row: 0 },
                    thumbnail(index, 4),
                    HelpPoint::new(0, y_offset),
                );
            }
            composer.centered(
                HelpSprite::Screensaver,
                HelpSource::Whole,
                thumbnail(3, 4),
                HelpPoint::new(0, 2),
            );
        }
        _ => {}
    }
}

fn upgrades<A: HelpAssets>(composer: &mut Composer<'_, A>) {
    let tick = composer.tick;
    let center = thumbnail(0, 4).center();
    composer.cell(
        HelpSprite::SmallSwim,
        frame(tick, 3, 10),
        0,
        HelpPoint::new(center.x - 40, center.y - 40),
    );
    let center = thumbnail(1, 4).center();
    let angle = radians(tick, 5);
    for (column, row, x, offset) in [
        (frame(tick, 3, 10), 0, -40, 0.0),
        (frame(tick, 4, 10), 1, -20, 30.017454),
        (frame(tick, 3, 10), 2, 0, 60.034908),
    ] {
        composer.cell(
            HelpSprite::Food,
            column,
            row,
            HelpPoint::new(
                center.x + x,
                (f64::from(center.y - 20) + f64::from((angle + offset).sin() as f32 * 3.0)) as i32,
            ),
        );
    }
    let center = thumbnail(2, 4).center();
    // Preserve unchecked W1 quirks: first quantity pellet has sin(0), subsequent
    // offsets are radians (including 60), and the last uses twice the base angle.
    let angle = angle + 60.0;
    for (column, x, offset) in [
        (frame(tick, 3, 10), -40, 0.0),
        (
            ((tick / 4 % 10 + 3) % 10) as u16,
            -20,
            (angle + 0.08726639).sin(),
        ),
        (
            ((tick / 3 % 10 + 6) % 10) as u16,
            0,
            (angle * 2.0 + 0.08726639).sin(),
        ),
    ] {
        composer.cell(
            HelpSprite::Food,
            column,
            0,
            HelpPoint::new(
                center.x + x,
                (f64::from(center.y - 20) + f64::from(offset as f32 * 3.0)) as i32,
            ),
        );
    }
    let center = thumbnail(3, 4).center();
    composer.cell(
        HelpSprite::LaserUpgrades,
        frame(tick, 36, 3),
        0,
        HelpPoint::new(center.x - 23, center.y - 18),
    );
}

fn credits<A: HelpAssets>(composer: &mut Composer<'_, A>, view: HelpView) {
    // Retain project attribution. Retail author matrix belongs to the owned game,
    // not to this module or the separately owned finale catalog.
    for (index, line) in view.lines.iter().enumerate() {
        let baseline = 150 + index as i32 * 25;
        let offset = if composer.assets.animate_credits() {
            let angle =
                f64::from(baseline * 2) * std::f64::consts::PI / 180.0 + radians(composer.tick, 3);
            (angle.sin() as f32 * 3.0) as i32
        } else {
            0
        };
        let label = ["Original game", "Rust runtime", "Owned assets"]
            .get(index)
            .copied()
            .unwrap_or("");
        composer.text(
            label,
            HelpRect::new(30, baseline - 20 + offset, 220, 25),
            baseline + offset,
            TextAlign::Left,
            HelpFont::ContinuumBold14,
            pulse(composer.tick, baseline, 40, 200),
        );
        composer.text(
            *line,
            HelpRect::new(270, baseline - 20 + offset, 350, 25),
            baseline + offset,
            TextAlign::Left,
            HelpFont::JungleFever10Outline,
            pulse(composer.tick, baseline, 159, 119),
        );
    }
}

fn chrome<A: HelpAssets>(composer: &mut Composer<'_, A>, view: HelpView) {
    // Asset-dependent heights come from decoded button/title resources. Missing
    // chrome art still permits project buttons with a documented 30px UI fallback.
    let button_height = composer
        .assets
        .image_metrics(HelpSprite::MainButton)
        .map_or(30, |metrics| i32::from(metrics.height).min(46));
    let return_area = if view.return_to == HelpReturn::PausedTank {
        HelpRect::new(480, 4, 125, button_height)
    } else {
        HelpRect::new(525, 4, 80, button_height)
    };
    let title_height = composer
        .assets
        .image_metrics(HelpSprite::ScreenTitle)
        .map_or(32, |metrics| i32::from(metrics.height).min(48));
    composer.image_box(
        HelpSprite::ScreenTitle,
        HelpRect::new(20, 0, 600, title_height),
        HelpLayer::Chrome,
    );
    let hole_height = composer
        .assets
        .image_metrics(HelpSprite::ScreenTitleHole)
        .map_or(button_height, |metrics| i32::from(metrics.height).min(48));
    composer.image_box(
        HelpSprite::ScreenTitleHole,
        HelpRect::new(
            return_area.x - 1,
            return_area.y - 1,
            return_area.width + 2,
            hole_height,
        ),
        HelpLayer::Chrome,
    );
    let title = if view.is_instructions() {
        "Instructions"
    } else {
        "Turbofish Help"
    };
    composer.text(
        title,
        HelpRect::new(30, 0, 440, 35),
        25,
        TextAlign::Center,
        HelpFont::JungleFever17Outline,
        HelpTint::Rgba(255, 200, 0, 255),
    );
    if let Some(draw) = composer.plan.draws.last_mut() {
        draw.layer = HelpLayer::Chrome;
    }
    composer.control(
        HelpAction::Return,
        return_area,
        if view.return_to == HelpReturn::PausedTank {
            "Back to Game"
        } else {
            "Menu"
        },
        false,
        HelpSprite::MainButton,
    );
    if view.is_instructions() {
        let height = composer
            .assets
            .image_metrics(HelpSprite::DialogButton)
            .map_or(30, |metrics| i32::from(metrics.height).min(54));
        composer.control(
            HelpAction::Continue,
            HelpRect::new(210, 415, 220, height),
            "Continue",
            false,
            HelpSprite::DialogButton,
        );
    } else {
        composer.text(
            view.title,
            HelpRect::new(10, 40, 620, 25),
            60,
            TextAlign::Center,
            HelpFont::ContinuumBold14,
            WHITE,
        );
        composer.text(
            format!("Page {} of 8", view.page),
            HelpRect::new(10, 65, 620, 20),
            80,
            TextAlign::Center,
            HelpFont::ContinuumBold14,
            WHITE,
        );
        composer.control(
            HelpAction::Previous,
            HelpRect::new(190, 430, 120, button_height),
            "Previous",
            false,
            HelpSprite::MainButton,
        );
        composer.control(
            HelpAction::Next,
            HelpRect::new(330, 430, 120, button_height),
            "Next",
            false,
            HelpSprite::MainButton,
        );
        // Existing direct tabs are a project extension to source Previous/Next.
        for page in 1..=LAST_PAGE {
            composer.control(
                HelpAction::Page(page),
                HelpRect::new(15 + i32::from(page - 1) * 77, 90, 70, 18),
                page.to_string(),
                page == view.page,
                HelpSprite::MainButton,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Synthetic decoded metadata, not bundled assets or evidence of retail pixels.
    struct TestAssets;

    impl HelpAssets for TestAssets {
        fn image_metrics(&self, sprite: HelpSprite) -> Option<ImageMetrics> {
            let (width, height, columns, rows) = match sprite {
                HelpSprite::SmallSwim
                | HelpSprite::SmallTurn
                | HelpSprite::SmallEat
                | HelpSprite::HungrySwim
                | HelpSprite::SmallDie
                | HelpSprite::SimSwimBase
                | HelpSprite::SimSwimColor
                | HelpSprite::SimSwimAccent
                | HelpSprite::SimEatBase
                | HelpSprite::SimEatColor
                | HelpSprite::SimEatAccent => (800, 400, 10, 5),
                HelpSprite::Sylvester => (1600, 160, 10, 1),
                HelpSprite::Food => (400, 120, 10, 3),
                HelpSprite::Money => (720, 720, 10, 10),
                HelpSprite::EggPieces | HelpSprite::LaserUpgrades => (138, 37, 3, 1),
                HelpSprite::Itchy | HelpSprite::Stinky => (600, 60, 10, 1),
                HelpSprite::TrophyButtons => (360, 32, 6, 1),
                HelpSprite::Ball => (500, 50, 10, 1),
                HelpSprite::Wave => (640, 40, 4, 1),
                _ => (30, 30, 1, 1),
            };
            ImageMetrics::new(width, height, columns, rows)
        }

        fn wave_cell(&self, _updates: u64) -> Option<u16> {
            Some(0)
        }
    }

    fn help_view(page: u8) -> HelpView {
        let (title, lines) = super::super::page_content(page);
        HelpView {
            page,
            title,
            lines,
            return_to: HelpReturn::Selector,
        }
    }

    fn plan(page: u8, updates: u64) -> HelpRenderPlan {
        let mut clock = DemoClock::default();
        clock.advance_ticks(updates);
        compose(help_view(page), &clock, &TestAssets).unwrap()
    }

    fn sprite_draws(plan: &HelpRenderPlan, sprite: HelpSprite) -> Vec<(&HelpDraw, &SpriteDraw)> {
        plan.draws
            .iter()
            .filter_map(|draw| match &draw.primitive {
                HelpPrimitive::Sprite(image) if image.sprite == sprite => Some((draw, image)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn feeding_uses_source_windows_interpolation_and_alternating_turns() {
        // W1 HelpScreen333-378: absolute thresholds, not expectations read from code.
        let card = HelpRect::new(30, 70, 185, 320);
        for (tick, sprite, column, mirrored, food, click) in [
            (39, HelpSprite::SmallSwim, 9, false, false, false),
            (40, HelpSprite::SmallSwim, 0, false, true, true),
            (45, HelpSprite::SmallTurn, 0, false, true, true),
            (59, HelpSprite::SmallTurn, 7, false, true, true),
            (60, HelpSprite::SmallTurn, 7, false, true, false),
            (64, HelpSprite::SmallTurn, 9, false, true, false),
            (65, HelpSprite::SmallSwim, 2, true, true, false),
            (75, HelpSprite::SmallEat, 0, true, true, false),
            (84, HelpSprite::SmallEat, 4, true, true, false),
            (85, HelpSprite::SmallEat, 5, true, false, false),
            (94, HelpSprite::SmallEat, 9, true, false, false),
            (95, HelpSprite::SmallSwim, 7, true, false, false),
            (115, HelpSprite::SmallSwim, 7, true, false, false),
            (180, HelpSprite::SmallSwim, 0, false, true, false),
        ] {
            let sample = feeding_sample(tick, card);
            assert_eq!(
                (
                    sample.sprite,
                    sample.frame,
                    sample.mirrored,
                    sample.food.is_some(),
                    sample.click
                ),
                (sprite, column, mirrored, food, click),
                "tick {tick}"
            );
        }
        assert_eq!(feeding_sample(0, card).fish.x, 58);
        assert_eq!(feeding_sample(45, card).fish.x, 58);
        assert_eq!(feeding_sample(65, card).fish.x, 88);
        assert_eq!(feeding_sample(85, card).fish.x, 118);
        assert_eq!(feeding_sample(115, card).fish.x, 118);
        assert_eq!(feeding_sample(200, card).fish.x, 58);
        assert_eq!(feeding_sample(0, card).cursor.x, 80);
        assert_eq!(feeding_sample(30, card).cursor.x, 165);
        assert_eq!(feeding_sample(115, card).cursor.x, 165);
    }

    #[test]
    fn combat_flash_ends_before_burst_and_burst_stays_at_original_click() {
        // W1 HelpScreen388-430: phase20 starts flash100 and eight rays;
        // phase30 ends flash; phase40 removes rays. Source anchors rays at tick20.
        for (tick, flash_alpha, rays) in [
            (19, None, 0),
            (20, Some(100), 8),
            (29, Some(10), 8),
            (30, None, 8),
            (39, None, 8),
            (40, None, 0),
        ] {
            let plan = plan(0, tick);
            let flashes: Vec<_> = sprite_draws(&plan, HelpSprite::Sylvester)
                .into_iter()
                .filter(|(draw, _)| draw.blend == HelpBlend::Additive)
                .collect();
            assert_eq!(
                flashes.first().map(|(draw, _)| draw.tint),
                flash_alpha.map(|alpha| HelpTint::Rgba(255, 255, 255, alpha))
            );
            let combat_rays: Vec<_> = plan
                .draws
                .iter()
                .filter(|draw| {
                    draw.clip.x == 227 && matches!(draw.primitive, HelpPrimitive::Line { .. })
                })
                .collect();
            assert_eq!(combat_rays.len(), rays, "tick {tick}");
        }
        let start = plan(0, 20);
        let end = plan(0, 39);
        let rays = |plan: &HelpRenderPlan| {
            plan.draws
                .iter()
                .filter(|draw| {
                    draw.clip.x == 227 && matches!(draw.primitive, HelpPrimitive::Line { .. })
                })
                .cloned()
                .collect::<Vec<_>>()
        };
        assert_eq!(rays(&start), rays(&end));
        // The independent cursor continues moving after the anchored click.
        let cursor = |plan: &HelpRenderPlan| {
            sprite_draws(plan, HelpSprite::Cursor)
                .into_iter()
                .find(|(draw, _)| draw.clip.x == 227)
                .unwrap()
                .1
                .destination
        };
        assert_ne!(cursor(&start), cursor(&end));
    }

    #[test]
    fn virtual_fish_switches_eat_swim_sheets_and_scopes_color_layers() {
        // W1 HelpScreen697-743: 27 animation slots; first10 eat, remaining17
        // swim with+3; next cycle starts on eat0. The second example uses row1.
        for (tick, sprite, column) in [
            (0, HelpSprite::SimEatBase, 0),
            (18, HelpSprite::SimEatBase, 9),
            (20, HelpSprite::SimSwimBase, 3),
            (52, HelpSprite::SimSwimBase, 9),
            (54, HelpSprite::SimEatBase, 0),
        ] {
            let plan = plan(6, tick);
            let first = sprite_draws(&plan, sprite)
                .into_iter()
                .find(|(_, image)| image.source.y == 0)
                .unwrap();
            assert_eq!(first.1.source, HelpRect::new(column * 80, 0, 80, 80));
            assert_eq!(first.0.blend, HelpBlend::Normal);
            let larger = sprite_draws(&plan, HelpSprite::SimSwimBase)
                .into_iter()
                .find(|(_, image)| image.source.y == 80)
                .unwrap();
            assert_eq!(larger.0.blend, HelpBlend::Normal);
            let overlays = plan
                .draws
                .iter()
                .filter(|draw| {
                    draw.layer == HelpLayer::Examples && draw.blend == HelpBlend::Additive
                })
                .collect::<Vec<_>>();
            assert_eq!(overlays.len(), 4);
            assert_eq!(overlays[0].tint, HelpTint::Rgba(35, 85, 130, 255));
            assert_eq!(overlays[1].tint, HelpTint::Rgba(180, 245, 180, 255));
            assert!(
                sprite_draws(&plan, HelpSprite::TrophyButtons)
                    .iter()
                    .all(|(draw, _)| draw.blend == HelpBlend::Normal)
            );
        }
    }

    #[test]
    fn source_page_examples_and_authored_guidance_have_bounded_composition() {
        // W1 page-specific resource choices (488-786), plus explicit project
        // contract: every existing authored guidance line survives composition.
        let expected = [
            vec![
                HelpSprite::SmallSwim,
                HelpSprite::Sylvester,
                HelpSprite::EggPieces,
                HelpSprite::Cursor,
            ],
            vec![
                HelpSprite::HungrySwim,
                HelpSprite::Money,
                HelpSprite::HelpAlien,
                HelpSprite::EggPieces,
            ],
            vec![
                HelpSprite::SmallSwim,
                HelpSprite::Food,
                HelpSprite::LaserUpgrades,
            ],
            vec![
                HelpSprite::EggPieces,
                HelpSprite::Itchy,
                HelpSprite::Stinky,
                HelpSprite::PetSelection,
            ],
            vec![],
            vec![
                HelpSprite::TrophyButtons,
                HelpSprite::Shell,
                HelpSprite::SimSwimBase,
                HelpSprite::Ball,
            ],
            vec![
                HelpSprite::SimEatBase,
                HelpSprite::SimSwimBase,
                HelpSprite::TrophyButtons,
                HelpSprite::SmallDie,
            ],
            vec![HelpSprite::TrophyButtons, HelpSprite::Screensaver],
            vec![],
        ];
        for page in 0..=LAST_PAGE {
            let plan = plan(page, 0);
            assert!(
                plan.asset_issues.is_empty(),
                "page {page}: {:?}",
                plan.asset_issues
            );
            for sprite in &expected[usize::from(page)] {
                assert!(
                    !sprite_draws(&plan, *sprite).is_empty(),
                    "page {page}: {sprite:?}"
                );
            }
            let text: String = plan
                .draws
                .iter()
                .filter_map(|draw| match &draw.primitive {
                    HelpPrimitive::Text(text) => Some(text.text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");
            for line in help_view(page).lines {
                assert!(text.contains(line), "page {page} lost {line}");
            }
            for draw in &plan.draws {
                assert!(draw.clip.x >= 0 && draw.clip.y >= 0);
                assert!(
                    draw.clip.x + draw.clip.width <= 640 && draw.clip.y + draw.clip.height <= 480
                );
                if let HelpPrimitive::Sprite(sprite) = &draw.primitive {
                    assert!(
                        TestAssets
                            .image_metrics(sprite.sprite)
                            .unwrap()
                            .includes(sprite.source)
                    );
                }
            }
            assert!(
                plan.draws
                    .windows(2)
                    .all(|pair| pair[0].layer <= pair[1].layer)
            );
            assert_eq!(
                plan.ambient.fish_config,
                if page == 8 { [3, 3] } else { [0, 0] }
            );
            assert_eq!(
                plan.controls
                    .iter()
                    .filter(|control| matches!(control.action, HelpAction::Page(_)))
                    .count(),
                if page == 0 { 0 } else { 8 }
            );
            let next = plan
                .controls
                .iter()
                .find(|control| control.action == HelpAction::Next);
            assert_eq!(next.is_some(), page != 0);
            if let Some(next) = next {
                assert_eq!(
                    plan.control_at(next.bounds.center()),
                    Some(HelpAction::Next)
                );
            }
        }
    }

    #[test]
    fn unavailable_or_invalid_metrics_omit_sprites_and_preserve_controls() {
        struct MissingAssets;
        impl HelpAssets for MissingAssets {
            fn image_metrics(&self, sprite: HelpSprite) -> Option<ImageMetrics> {
                (sprite == HelpSprite::HungrySwim).then(|| ImageMetrics::new(80, 80, 1, 1).unwrap())
            }
        }
        assert!(ImageMetrics::new(0, 80, 1, 1).is_none());
        assert!(ImageMetrics::new(80, 80, 0, 1).is_none());
        assert!(ImageMetrics::new(80, 80, 3, 1).is_none());
        assert!(ImageMetrics::new(u32::MAX, 80, 1, 1).is_none());
        let mut clock = DemoClock::default();
        clock.advance_ticks(3);
        let view = help_view(1);
        let plan = compose(view, &clock, &MissingAssets).unwrap();
        assert!(plan.asset_issues.contains(&AssetIssue {
            sprite: HelpSprite::HungrySwim,
            kind: AssetIssueKind::CropOutsideImage
        }));
        assert!(plan.asset_issues.contains(&AssetIssue {
            sprite: HelpSprite::Money,
            kind: AssetIssueKind::MissingMetrics
        }));
        assert!(plan.asset_issues.contains(&AssetIssue {
            sprite: HelpSprite::Wave,
            kind: AssetIssueKind::MissingAnimation
        }));
        assert!(sprite_draws(&plan, HelpSprite::HungrySwim).is_empty());
        assert!(
            plan.controls
                .iter()
                .any(|control| control.action == HelpAction::Return)
        );
        assert_eq!(clock.updates(), 3);
        assert_eq!(compose(view, &clock, &MissingAssets).unwrap(), plan);
        assert!(compose(HelpView { page: 255, ..view }, &clock, &MissingAssets).is_none());
    }

    #[test]
    fn controller_preserves_page_clock_return_targets_and_instruction_gate() {
        // W1 HelpScreen Update does not reset mUpdateCnt on page changes.
        // Existing Help navigation contract remains an explicit project invariant.
        let mut help = super::super::HelpController::new();
        assert_eq!(help.advance_demo(Duration::from_secs(1)), 0);
        help.open_pages(HelpReturn::PausedTank);
        assert_eq!(help.advance_demo(Duration::from_millis(27)), 0);
        assert_eq!(help.advance_demo(Duration::from_millis(1)), 1);
        help.advance_demo_ticks(19);
        assert!(help.select_page(8));
        assert_eq!(help.demo_clock().updates(), 20);
        assert!(help.next_page());
        assert_eq!(help.view().unwrap().page, 1);
        assert!(help.previous());
        assert_eq!(help.view().unwrap().page, 8);
        assert!(help.select_page(7));
        help.open_instructions(HelpReturn::Selector);
        assert_eq!(help.remembered_page(), 7);
        assert_eq!(help.demo_clock().updates(), 0);
        assert!(!help.select_page(2) && !help.next_page() && !help.previous());
        assert_eq!(help.advance_demo(Duration::from_millis(56)), 2);
        assert_eq!(
            help.continue_intro(),
            Some(super::super::HelpExit::Continue)
        );
        assert_eq!(help.advance_demo(Duration::from_secs(1)), 0);
        assert!(help.render_plan(&TestAssets).is_none());
        help.open_pages(HelpReturn::PausedTank);
        assert_eq!(help.view().unwrap().page, 7);
        assert_eq!(help.demo_clock().updates(), 0);
        assert_eq!(help.continue_intro(), None);
        assert_eq!(
            help.close(),
            Some(super::super::HelpExit::ReturnTo(HelpReturn::PausedTank))
        );
    }
}
