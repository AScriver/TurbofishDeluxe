//! Four-card Time Trial/Challenge chooser, informed by pinned W1 TankScreen.
//! Eligibility and earned stories are immutable caller facts. This controller
//! owns only transient presentation/input, never Profile, gameplay or saves.

use crate::assets::{DecodedImage, GameAssets};
use crate::font::BitmapFont;
use std::collections::HashMap;
use std::fmt;
use std::path::Path;

pub const SCREEN: Rect = Rect::new(0, 0, 640, 480);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectionMode {
    TimeTrial,
    Challenge,
}

impl SelectionMode {
    pub const fn title(self) -> &'static str {
        match self {
            Self::TimeTrial => "Time Trial",
            Self::Challenge => "Challenge",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::TimeTrial => "Earn money against the clock.",
            Self::Challenge => "Survive alien waves and finish the tank.",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tank {
    One,
    Two,
    Three,
    Four,
}

impl Tank {
    pub const ALL: [Self; 4] = [Self::One, Self::Two, Self::Three, Self::Four];

    pub const fn index(self) -> usize {
        match self {
            Self::One => 0,
            Self::Two => 1,
            Self::Three => 2,
            Self::Four => 3,
        }
    }

    pub const fn number(self) -> u8 {
        self.index() as u8 + 1
    }

    /// W1 skips the third thumbnail resource, not the third displayed card.
    pub const fn backdrop(self) -> TankImage {
        match self {
            Self::One => TankImage::Backdrop1,
            Self::Two => TankImage::Backdrop2,
            Self::Three => TankImage::Backdrop4,
            Self::Four => TankImage::Backdrop5,
        }
    }
}

/// Values come from the production session's eligibility authority. No unlock
/// rule is recomputed here. Cyrax alone is not an ordinary earned story.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TankFacts {
    pub available: [bool; 4],
    pub has_ordinary_stories: bool,
}

impl TankFacts {
    pub const fn stories_visible(self, mode: SelectionMode) -> bool {
        matches!(mode, SelectionMode::Challenge) && self.has_ordinary_stories
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
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
            && point.x >= self.x as f32
            && point.x < self.x as f32 + self.width as f32
            && point.y >= self.y as f32
            && point.y < self.y as f32 + self.height as f32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TankImage {
    ScreenBack,
    HorizontalCrease,
    VerticalCrease,
    Bolt,
    ScreenTitle,
    ScreenTitleHole,
    TankChooser,
    Backdrop1,
    Backdrop2,
    Backdrop4,
    Backdrop5,
    MainButton,
    DialogButton,
}

impl TankImage {
    pub const ALL: [Self; 13] = [
        Self::ScreenBack,
        Self::HorizontalCrease,
        Self::VerticalCrease,
        Self::Bolt,
        Self::ScreenTitle,
        Self::ScreenTitleHole,
        Self::TankChooser,
        Self::Backdrop1,
        Self::Backdrop2,
        Self::Backdrop4,
        Self::Backdrop5,
        Self::MainButton,
        Self::DialogButton,
    ];

    pub const fn resource_key(self) -> &'static str {
        match self {
            Self::ScreenBack => "IMAGE_SCREENBACK",
            Self::HorizontalCrease => "IMAGE_HORZCREASE",
            Self::VerticalCrease => "IMAGE_VERTCREASE",
            Self::Bolt => "IMAGE_BOLT",
            Self::ScreenTitle => "IMAGE_SCREENTITLE",
            Self::ScreenTitleHole => "IMAGE_SCREENTITLEHOLE",
            Self::TankChooser => "IMAGE_TANKCHOOSER",
            Self::Backdrop1 => "IMAGE_TANKBACKDROP1",
            Self::Backdrop2 => "IMAGE_TANKBACKDROP2",
            Self::Backdrop4 => "IMAGE_TANKBACKDROP4",
            Self::Backdrop5 => "IMAGE_TANKBACKDROP5",
            Self::MainButton => "IMAGE_MAINBUTTON",
            Self::DialogButton => "IMAGE_DIALOGBUTTON",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TankFont {
    Title,
    Small,
    Button,
}

impl TankFont {
    pub const ALL: [Self; 3] = [Self::Title, Self::Small, Self::Button];

    pub const fn resource_key(self) -> &'static str {
        match self {
            Self::Title => "FONT_JUNGLEFEVER17OUTLINE",
            Self::Small => "FONT_JUNGLEFEVER10OUTLINE",
            Self::Button => "FONT_JUNGLEFEVER12OUTLINE",
        }
    }

    pub const fn script_stem(self) -> &'static str {
        match self {
            Self::Title => "JungleFever17outline",
            Self::Small => "JungleFever10outline",
            Self::Button => "JungleFever12outline",
        }
    }
}

/// Complete decoded sheet dimensions and its manifest grid, not guessed sizes.
/// DialogButton has three painted states despite its manifest's default 1x1.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
    pub cols: u32,
    pub rows: u32,
}

impl From<&DecodedImage> for ImageMetadata {
    fn from(image: &DecodedImage) -> Self {
        Self {
            width: image.width,
            height: image.height,
            cols: image.cols,
            rows: image.rows,
        }
    }
}

/// Implement on the actual app presentation to reuse decoded texture metadata
/// and its production BitmapFonts. None is an explicit unavailable resource.
pub trait TankResources {
    fn image(&self, image: TankImage) -> Option<ImageMetadata>;
    fn font(&self, font: TankFont) -> Option<&BitmapFont>;
}

/// Read-only convenience adapter for tools/tests. The app can instead implement
/// TankResources on its existing asset cache, avoiding duplicate decoding.
pub struct OwnedTankResources {
    images: HashMap<TankImage, ImageMetadata>,
    fonts: HashMap<TankFont, BitmapFont>,
}

impl OwnedTankResources {
    pub fn load(assets: &GameAssets, game_root: &Path) -> Result<Self, SceneUnavailable> {
        let mut images = HashMap::new();
        for image in TankImage::ALL {
            let decoded = assets.load_image(image.resource_key()).map_err(|error| {
                SceneUnavailable::Load {
                    resource: image.resource_key(),
                    reason: error.to_string(),
                }
            })?;
            images.insert(image, ImageMetadata::from(&decoded));
        }
        let mut fonts = HashMap::new();
        for font in TankFont::ALL {
            let bitmap = BitmapFont::load(game_root, font.script_stem()).map_err(|error| {
                SceneUnavailable::Load {
                    resource: font.resource_key(),
                    reason: error.to_string(),
                }
            })?;
            for layer in &bitmap.layers {
                let atlas = assets
                    .load_image(&layer.image_path.to_string_lossy())
                    .map_err(|error| SceneUnavailable::Load {
                        resource: font.resource_key(),
                        reason: error.to_string(),
                    })?;
                layer
                    .validate_atlas(atlas.width, atlas.height)
                    .map_err(|error| SceneUnavailable::InvalidFont(font, error.to_string()))?;
            }
            fonts.insert(font, bitmap);
        }
        Ok(Self { images, fonts })
    }
}

impl TankResources for OwnedTankResources {
    fn image(&self, image: TankImage) -> Option<ImageMetadata> {
        self.images.get(&image).copied()
    }

    fn font(&self, font: TankFont) -> Option<&BitmapFont> {
        self.fonts.get(&font)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SceneUnavailable {
    MissingImage(TankImage),
    InvalidImage(TankImage),
    MissingFont(TankFont),
    InvalidFont(TankFont, String),
    Load {
        resource: &'static str,
        reason: String,
    },
}

impl fmt::Display for SceneUnavailable {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingImage(image) => write!(formatter, "missing {}", image.resource_key()),
            Self::InvalidImage(image) => {
                write!(formatter, "invalid metadata for {}", image.resource_key())
            }
            Self::MissingFont(font) => write!(formatter, "missing {}", font.resource_key()),
            Self::InvalidFont(font, reason) => {
                write!(formatter, "invalid {}: {reason}", font.resource_key())
            }
            Self::Load { resource, reason } => {
                write!(formatter, "cannot load {resource}: {reason}")
            }
        }
    }
}

impl std::error::Error for SceneUnavailable {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    Background,
    Creases,
    Bolts,
    Header,
    Cards,
    Controls,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Hovered,
    Pressed,
    Locked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageFrame {
    Static,
    Button(ButtonState),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageProjection {
    /// Copy pixels without scaling.
    Natural,
    /// Use the existing image-box tiler with borders derived from this crop.
    ImageBox,
    /// Repeat unscaled, clipping the final tile to the destination.
    HorizontalTile,
    VerticalTile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageDraw {
    pub resource: TankImage,
    pub frame: ImageFrame,
    pub crop: Rect,
    pub destination: Rect,
    pub projection: ImageProjection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextPlacement {
    Baseline {
        x: i32,
        y: i32,
    },
    /// Source lock rectangle. Center/wrap with the actual font's line metrics,
    /// clipping to bounds; no substitute bitmap font or guessed line height.
    WrappedCentered {
        bounds: Rect,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextDraw {
    pub font: TankFont,
    pub text: String,
    pub color: [u8; 4],
    pub placement: TextPlacement,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DrawKind {
    Image(ImageDraw),
    Text(TextDraw),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DrawCommand {
    pub layer: Layer,
    pub kind: DrawKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Control {
    Tank(Tank),
    Menu,
    Stories,
}

impl Control {
    pub const fn source_button_id(self) -> u8 {
        match self {
            Self::Tank(tank) => tank.index() as u8,
            Self::Menu => 99,
            Self::Stories => 4,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlView {
    pub control: Control,
    pub bounds: Rect,
    pub enabled: bool,
    pub state: ButtonState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardView {
    pub tank: Tank,
    pub frame_bounds: Rect,
    pub backdrop: Option<TankImage>,
    pub lock_explanation: Option<String>,
    pub button: ControlView,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TankScene {
    pub viewport: Rect,
    pub mode: SelectionMode,
    pub cards: [CardView; 4],
    pub menu: ControlView,
    pub stories: Option<ControlView>,
    /// Execute in order. Layer names describe ownership, not a sorting key.
    pub commands: Vec<DrawCommand>,
}

impl TankScene {
    pub fn hit_test(&self, point: Point) -> Option<ControlView> {
        if !self.viewport.contains(point) {
            return None;
        }
        std::iter::once(self.menu)
            .chain(self.stories)
            .chain(self.cards.iter().map(|card| card.button))
            .find(|button| button.enabled && button.bounds.contains(point))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonSound {
    Click,
}

impl ButtonSound {
    pub const fn resource_key(self) -> &'static str {
        "SOUND_BUTTONCLICK"
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TankIntent {
    SelectTank { mode: SelectionMode, tank: Tank },
    Menu,
    Stories,
    PlayButtonSound(ButtonSound),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TankInput {
    PointerMove(Point),
    PrimaryDown(Point),
    PrimaryUp(Point),
    Focus(bool),
    Modal(bool),
    Cancel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Capture {
    control: Control,
    bounds: Rect,
}

pub struct TankSelection {
    mode: SelectionMode,
    facts: TankFacts,
    focused: bool,
    modal: bool,
    hover: Option<Control>,
    capture: Option<Capture>,
    dirty: bool,
}

impl TankSelection {
    pub fn new(mode: SelectionMode, facts: TankFacts) -> Self {
        Self {
            mode,
            facts,
            focused: true,
            modal: false,
            hover: None,
            capture: None,
            dirty: true,
        }
    }

    pub const fn mode(&self) -> SelectionMode {
        self.mode
    }

    /// Replace snapshots on profile/progression/story changes, before input.
    /// Even a different card's availability change cancels an old capture.
    pub fn set_facts(&mut self, facts: TankFacts) {
        if self.facts != facts {
            self.facts = facts;
            self.cancel();
        }
    }

    /// W1 Update marks dirty; no clock, animation or simulation step belongs here.
    pub fn update(&mut self) {
        self.dirty = true;
    }

    pub fn take_dirty(&mut self) -> bool {
        std::mem::take(&mut self.dirty)
    }

    pub fn cancel(&mut self) {
        self.capture = None;
        self.hover = None;
        self.dirty = true;
    }

    pub fn input(
        &mut self,
        input: TankInput,
        resources: &impl TankResources,
    ) -> Result<Vec<TankIntent>, SceneUnavailable> {
        match input {
            TankInput::Focus(focused) => {
                if self.focused != focused {
                    self.focused = focused;
                    self.cancel();
                }
                return Ok(Vec::new());
            }
            TankInput::Modal(modal) => {
                if self.modal != modal {
                    self.modal = modal;
                    self.cancel();
                }
                return Ok(Vec::new());
            }
            TankInput::Cancel => {
                self.cancel();
                return Ok(Vec::new());
            }
            _ => {}
        }
        if !self.focused || self.modal {
            self.cancel();
            return Ok(Vec::new());
        }
        let scene = match self.scene(resources) {
            Ok(scene) => scene,
            Err(error) => {
                self.cancel();
                return Err(error);
            }
        };
        let point = match input {
            TankInput::PointerMove(point)
            | TankInput::PrimaryDown(point)
            | TankInput::PrimaryUp(point) => point,
            _ => unreachable!("non-pointer inputs handled above"),
        };
        let hit = scene.hit_test(point);
        self.hover = hit.map(|button| button.control);
        self.dirty = true;
        let mut intentions = Vec::new();
        match input {
            TankInput::PrimaryDown(_) if self.capture.is_none() => {
                if let Some(button) = hit {
                    self.capture = Some(Capture {
                        control: button.control,
                        bounds: button.bounds,
                    });
                    intentions.push(TankIntent::PlayButtonSound(ButtonSound::Click));
                }
            }
            TankInput::PrimaryUp(_) => {
                if let Some(capture) = self.capture.take()
                    && hit.is_some_and(|button| {
                        button.control == capture.control && button.bounds == capture.bounds
                    })
                {
                    intentions.push(match capture.control {
                        Control::Tank(tank) => TankIntent::SelectTank {
                            mode: self.mode,
                            tank,
                        },
                        Control::Menu => TankIntent::Menu,
                        Control::Stories => TankIntent::Stories,
                    });
                }
            }
            _ => {}
        }
        Ok(intentions)
    }

    pub fn scene(&self, resources: &impl TankResources) -> Result<TankScene, SceneUnavailable> {
        let stories_visible = self.facts.stories_visible(self.mode);
        let main = checked_image(resources, TankImage::MainButton)?;
        let frame = checked_image(resources, TankImage::TankChooser)?;
        let button = |control, bounds, available| {
            let enabled = available && self.focused && !self.modal;
            let state = if !available {
                ButtonState::Locked
            } else if enabled && self.hover == Some(control) {
                if self
                    .capture
                    .is_some_and(|capture| capture.control == control && capture.bounds == bounds)
                {
                    ButtonState::Pressed
                } else {
                    ButtonState::Hovered
                }
            } else {
                ButtonState::Normal
            };
            ControlView {
                control,
                bounds,
                enabled,
                state,
            }
        };
        let cards = Tank::ALL.map(|tank| {
            let x = if tank.index().is_multiple_of(2) {
                70
            } else {
                350
            };
            let y = if tank.index() < 2 {
                71
            } else if stories_visible {
                284
            } else {
                264
            };
            let available = self.facts.available[tank.index()];
            CardView {
                tank,
                frame_bounds: Rect::new(x, y, frame.width, frame.height),
                backdrop: available.then_some(tank.backdrop()),
                lock_explanation: (!available).then(|| lock_explanation(self.mode, tank)),
                button: button(
                    Control::Tank(tank),
                    Rect::new(x + 14, y + 134, 193, main.height),
                    available,
                ),
            }
        });
        let menu = button(Control::Menu, Rect::new(525, 4, 80, main.height), true);
        let stories = if stories_visible {
            let skin = checked_image(resources, TankImage::DialogButton)?;
            Some(button(
                Control::Stories,
                Rect::new(245, 247, 150, skin.height),
                true,
            ))
        } else {
            None
        };
        let mut commands = Vec::new();
        image_command(
            &mut commands,
            resources,
            Layer::Background,
            TankImage::ScreenBack,
            Rect::new(-5, -5, 650, 490),
            ImageProjection::ImageBox,
        )?;
        for y in [130, if stories_visible { 350 } else { 320 }] {
            let crease = checked_image(resources, TankImage::HorizontalCrease)?;
            image_command(
                &mut commands,
                resources,
                Layer::Creases,
                TankImage::HorizontalCrease,
                Rect::new(13, y, 614, crease.height),
                ImageProjection::HorizontalTile,
            )?;
        }
        for x in [180, 460] {
            let crease = checked_image(resources, TankImage::VerticalCrease)?;
            image_command(
                &mut commands,
                resources,
                Layer::Creases,
                TankImage::VerticalCrease,
                Rect::new(x, 100, crease.width, 369),
                ImageProjection::VerticalTile,
            )?;
        }
        let bolt = checked_image(resources, TankImage::Bolt)?;
        for (x, y) in [
            (50, 110),
            (590, 110),
            (50, 370),
            (590, 370),
            (30, 150),
            (610, 150),
            (30, 330),
            (610, 330),
            (320, 100),
            (320, 380),
            (320, 158),
            (320, 322),
        ] {
            image_command(
                &mut commands,
                resources,
                Layer::Bolts,
                TankImage::Bolt,
                Rect::new(x, y, bolt.width, bolt.height),
                ImageProjection::Natural,
            )?;
        }
        let title = checked_image(resources, TankImage::ScreenTitle)?;
        image_command(
            &mut commands,
            resources,
            Layer::Header,
            TankImage::ScreenTitle,
            Rect::new(20, 0, 600, title.height),
            ImageProjection::ImageBox,
        )?;
        let hole = checked_image(resources, TankImage::ScreenTitleHole)?;
        image_command(
            &mut commands,
            resources,
            Layer::Header,
            TankImage::ScreenTitleHole,
            Rect::new(524, 3, 82, hole.height),
            ImageProjection::ImageBox,
        )?;
        centered_text(
            &mut commands,
            resources,
            TankFont::Title,
            self.mode.title(),
            25,
            [255, 200, 0, 255],
        )?;
        centered_text(
            &mut commands,
            resources,
            TankFont::Small,
            self.mode.description(),
            60,
            [255; 4],
        )?;
        for card in &cards {
            image_command(
                &mut commands,
                resources,
                Layer::Cards,
                TankImage::TankChooser,
                card.frame_bounds,
                ImageProjection::Natural,
            )?;
            if let Some(backdrop) = card.backdrop {
                let metadata = checked_image(resources, backdrop)?;
                image_command(
                    &mut commands,
                    resources,
                    Layer::Cards,
                    backdrop,
                    Rect::new(
                        card.frame_bounds.x + 16,
                        card.frame_bounds.y + 10,
                        metadata.width,
                        metadata.height,
                    ),
                    ImageProjection::Natural,
                )?;
            } else if let Some(text) = &card.lock_explanation {
                checked_text(resources, TankFont::Small, text)?;
                commands.push(DrawCommand {
                    layer: Layer::Cards,
                    kind: DrawKind::Text(TextDraw {
                        font: TankFont::Small,
                        text: text.clone(),
                        color: [255; 4],
                        placement: TextPlacement::WrappedCentered {
                            bounds: Rect::new(
                                card.frame_bounds.x + 25,
                                card.frame_bounds.y + 43,
                                170,
                                200,
                            ),
                        },
                    }),
                });
            }
        }
        draw_button(&mut commands, resources, menu, "Menu", TankFont::Small)?;
        for card in &cards {
            draw_button(
                &mut commands,
                resources,
                card.button,
                &format!("Tank {}", card.tank.number()),
                TankFont::Button,
            )?;
        }
        if let Some(stories) = stories {
            draw_button(
                &mut commands,
                resources,
                stories,
                "Stories",
                TankFont::Button,
            )?;
        }
        Ok(TankScene {
            viewport: SCREEN,
            mode: self.mode,
            cards,
            menu,
            stories,
            commands,
        })
    }
}

fn lock_explanation(mode: SelectionMode, tank: Tank) -> String {
    match mode {
        SelectionMode::TimeTrial => format!("Finish Adventure\nTank {} to unlock.", tank.number()),
        SelectionMode::Challenge if tank == Tank::One => {
            "Finish Adventure\nto open Challenge.".into()
        }
        SelectionMode::Challenge => {
            format!("Finish Challenge\nTank {} to unlock.", tank.number() - 1)
        }
    }
}

fn checked_image(
    resources: &impl TankResources,
    image: TankImage,
) -> Result<ImageMetadata, SceneUnavailable> {
    let metadata = resources
        .image(image)
        .ok_or(SceneUnavailable::MissingImage(image))?;
    let button = matches!(image, TankImage::MainButton | TankImage::DialogButton);
    if metadata.width == 0
        || metadata.height == 0
        || metadata.width > u16::MAX.into()
        || metadata.height > u16::MAX.into()
        || metadata.rows != 1
        || !matches!(metadata.cols, 1 | 3)
        || (button
            && (metadata.width < 9 || !metadata.width.is_multiple_of(3) || metadata.height < 3))
        || (!button && metadata.cols != 1)
        || (matches!(
            image,
            TankImage::ScreenBack | TankImage::ScreenTitle | TankImage::ScreenTitleHole
        ) && (metadata.width < 3 || metadata.height < 3))
    {
        return Err(SceneUnavailable::InvalidImage(image));
    }
    Ok(metadata)
}

fn checked_text<'a>(
    resources: &'a impl TankResources,
    font: TankFont,
    text: &str,
) -> Result<&'a BitmapFont, SceneUnavailable> {
    let bitmap = resources
        .font(font)
        .ok_or(SceneUnavailable::MissingFont(font))?;
    let layer = bitmap
        .layers
        .first()
        .ok_or_else(|| SceneUnavailable::InvalidFont(font, "no font layer".into()))?;
    if layer.ascent <= 0
        || layer.ascent > i32::from(u16::MAX)
        || layer.ascent_padding < 0
        || layer.ascent_padding > layer.ascent
    {
        return Err(SceneUnavailable::InvalidFont(
            font,
            "invalid ascent or padding".into(),
        ));
    }
    for line in text.lines() {
        let width = bitmap
            .measure_width(line)
            .map_err(|error| SceneUnavailable::InvalidFont(font, error.to_string()))?;
        if width < 0 {
            return Err(SceneUnavailable::InvalidFont(
                font,
                "negative text width".into(),
            ));
        }
    }
    Ok(bitmap)
}

fn image_command(
    commands: &mut Vec<DrawCommand>,
    resources: &impl TankResources,
    layer: Layer,
    image: TankImage,
    destination: Rect,
    projection: ImageProjection,
) -> Result<(), SceneUnavailable> {
    let metadata = checked_image(resources, image)?;
    commands.push(DrawCommand {
        layer,
        kind: DrawKind::Image(ImageDraw {
            resource: image,
            frame: ImageFrame::Static,
            crop: Rect::new(0, 0, metadata.width, metadata.height),
            destination,
            projection,
        }),
    });
    Ok(())
}

fn centered_text(
    commands: &mut Vec<DrawCommand>,
    resources: &impl TankResources,
    font: TankFont,
    text: &str,
    baseline: i32,
    color: [u8; 4],
) -> Result<(), SceneUnavailable> {
    let bitmap = checked_text(resources, font, text)?;
    let width = bitmap
        .measure_width(text)
        .map_err(|error| SceneUnavailable::InvalidFont(font, error.to_string()))?;
    commands.push(DrawCommand {
        layer: Layer::Header,
        kind: DrawKind::Text(TextDraw {
            font,
            text: text.into(),
            color,
            placement: TextPlacement::Baseline {
                x: (640 - width) / 2,
                y: baseline,
            },
        }),
    });
    Ok(())
}

fn draw_button(
    commands: &mut Vec<DrawCommand>,
    resources: &impl TankResources,
    button: ControlView,
    label: &str,
    font: TankFont,
) -> Result<(), SceneUnavailable> {
    let bitmap = checked_text(resources, font, label)?;
    let layer = &bitmap.layers[0];
    let width = bitmap
        .measure_width(label)
        .map_err(|error| SceneUnavailable::InvalidFont(font, error.to_string()))?;
    let pressed = i32::from(button.state == ButtonState::Pressed);
    if button.state != ButtonState::Locked {
        let image = if button.control == Control::Stories {
            TankImage::DialogButton
        } else {
            TankImage::MainButton
        };
        let metadata = checked_image(resources, image)?;
        let frame_width = metadata.width / 3;
        let frame_index = match button.state {
            ButtonState::Hovered => 1,
            ButtonState::Pressed => 2,
            _ => 0,
        };
        commands.push(DrawCommand {
            layer: Layer::Controls,
            kind: DrawKind::Image(ImageDraw {
                resource: image,
                frame: ImageFrame::Button(button.state),
                crop: Rect::new(
                    (frame_index * frame_width) as i32,
                    0,
                    frame_width,
                    metadata.height,
                ),
                destination: button.bounds,
                projection: ImageProjection::ImageBox,
            }),
        });
    }
    let color = if matches!(
        button.state,
        ButtonState::Hovered | ButtonState::Pressed | ButtonState::Locked
    ) {
        [255; 4]
    } else {
        [255, 240, 0, 255]
    };
    commands.push(DrawCommand {
        layer: Layer::Controls,
        kind: DrawKind::Text(TextDraw {
            font,
            text: label.into(),
            color,
            placement: TextPlacement::Baseline {
                x: button.bounds.x + (button.bounds.width as i32 - width) / 2 - 2 + pressed,
                y: button.bounds.y
                    + (button.bounds.height as i32 + layer.ascent
                        - layer.ascent_padding
                        - layer.ascent / 6
                        - 1)
                        / 2
                    + pressed,
            },
        }),
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{FontLayer, Glyph};
    use std::path::PathBuf;

    /// Project-authored metadata fixture, using the real production font type.
    /// Dimensions deliberately differ from retail so hardcoded sizes fail.
    struct Resources {
        images: HashMap<TankImage, ImageMetadata>,
        font: BitmapFont,
        missing_font: Option<TankFont>,
    }

    impl Resources {
        fn new() -> Self {
            let mut images: HashMap<_, _> = TankImage::ALL
                .into_iter()
                .map(|image| {
                    (
                        image,
                        ImageMetadata {
                            width: 30,
                            height: 12,
                            cols: 1,
                            rows: 1,
                        },
                    )
                })
                .collect();
            images.insert(
                TankImage::MainButton,
                ImageMetadata {
                    width: 243,
                    height: 31,
                    cols: 3,
                    rows: 1,
                },
            );
            images.insert(
                TankImage::DialogButton,
                ImageMetadata {
                    width: 270,
                    height: 27,
                    cols: 1,
                    rows: 1,
                },
            );
            images.insert(
                TankImage::TankChooser,
                ImageMetadata {
                    width: 222,
                    height: 169,
                    cols: 1,
                    rows: 1,
                },
            );
            let glyphs = (' '..='~')
                .map(|character| {
                    (
                        character,
                        Glyph {
                            advance: 6,
                            ..Glyph::default()
                        },
                    )
                })
                .collect();
            Self {
                images,
                font: BitmapFont {
                    default_point_size: 10,
                    layers: vec![FontLayer {
                        name: "Main".into(),
                        image_path: PathBuf::from("fixture"),
                        ascent: 13,
                        ascent_padding: 2,
                        line_spacing_offset: 0,
                        point_size: 10,
                        spacing: 0,
                        glyphs,
                        kerning: HashMap::new(),
                    }],
                },
                missing_font: None,
            }
        }
    }

    impl TankResources for Resources {
        fn image(&self, image: TankImage) -> Option<ImageMetadata> {
            self.images.get(&image).copied()
        }
        fn font(&self, font: TankFont) -> Option<&BitmapFont> {
            (self.missing_font != Some(font)).then_some(&self.font)
        }
    }

    fn facts(available: [bool; 4], stories: bool) -> TankFacts {
        TankFacts {
            available,
            has_ordinary_stories: stories,
        }
    }

    fn point(rect: Rect) -> Point {
        Point {
            x: rect.x as f32 + 2.0,
            y: rect.y as f32 + 2.0,
        }
    }

    #[test]
    fn time_trial_keeps_four_source_cards_and_skips_backdrop_three() {
        let resources = Resources::new();
        let chooser = TankSelection::new(
            SelectionMode::TimeTrial,
            facts([true, true, false, true], true),
        );
        let scene = chooser.scene(&resources).unwrap();
        assert!(scene.stories.is_none());
        assert_eq!(
            scene
                .cards
                .map(|card| (card.frame_bounds.x, card.frame_bounds.y)),
            [(70, 71), (350, 71), (70, 264), (350, 264)]
        );
        let scene = chooser.scene(&resources).unwrap();
        assert_eq!(
            scene.cards[2].lock_explanation.as_deref(),
            Some("Finish Adventure\nTank 3 to unlock.")
        );
        assert_eq!(scene.cards[2].backdrop, None);
        assert_eq!(scene.cards[3].backdrop, Some(TankImage::Backdrop5));
        assert_eq!(
            Tank::ALL.map(Tank::backdrop),
            [
                TankImage::Backdrop1,
                TankImage::Backdrop2,
                TankImage::Backdrop4,
                TankImage::Backdrop5
            ]
        );
        assert_eq!(scene.cards[0].button.bounds, Rect::new(84, 205, 193, 31));
        assert!(scene.commands.iter().any(|command| matches!(
            command.kind,
            DrawKind::Image(ImageDraw {
                resource: TankImage::HorizontalCrease,
                destination: Rect { y: 320, .. },
                ..
            })
        )));
    }

    #[test]
    fn challenge_stories_gate_moves_only_lower_cards_and_second_crease() {
        let resources = Resources::new();
        let mut chooser = TankSelection::new(
            SelectionMode::Challenge,
            facts([true, false, false, false], false),
        );
        let closed = chooser.scene(&resources).unwrap();
        assert!(closed.stories.is_none());
        assert_eq!(closed.cards[2].frame_bounds.y, 264);
        assert_eq!(
            closed.cards[1].lock_explanation.as_deref(),
            Some("Finish Challenge\nTank 1 to unlock.")
        );
        chooser.set_facts(facts([true, false, false, false], true));
        let open = chooser.scene(&resources).unwrap();
        assert_eq!(open.stories.unwrap().bounds, Rect::new(245, 247, 150, 27));
        assert_eq!(
            open.cards
                .map(|card| (card.frame_bounds.x, card.frame_bounds.y)),
            [(70, 71), (350, 71), (70, 284), (350, 284)]
        );
        assert!(open.commands.iter().any(|command| matches!(
            command.kind,
            DrawKind::Image(ImageDraw {
                resource: TankImage::HorizontalCrease,
                destination: Rect { y: 350, .. },
                ..
            })
        )));
    }

    #[test]
    fn locked_buttons_and_release_without_matching_capture_cannot_select() {
        let resources = Resources::new();
        let mut chooser = TankSelection::new(
            SelectionMode::TimeTrial,
            facts([true, false, true, true], false),
        );
        let scene = chooser.scene(&resources).unwrap();
        let locked = point(scene.cards[1].button.bounds);
        assert!(
            chooser
                .input(TankInput::PrimaryDown(locked), &resources)
                .unwrap()
                .is_empty()
        );
        assert!(
            chooser
                .input(TankInput::PrimaryUp(locked), &resources)
                .unwrap()
                .is_empty()
        );
        let available = point(scene.cards[0].button.bounds);
        assert!(
            chooser
                .input(TankInput::PrimaryUp(available), &resources)
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            chooser
                .input(TankInput::PrimaryDown(available), &resources)
                .unwrap(),
            [TankIntent::PlayButtonSound(ButtonSound::Click)]
        );
        assert!(
            chooser
                .input(TankInput::PrimaryDown(available), &resources)
                .unwrap()
                .is_empty()
        );
        assert!(
            chooser
                .input(
                    TankInput::PrimaryUp(point(scene.cards[2].button.bounds)),
                    &resources
                )
                .unwrap()
                .is_empty()
        );
        chooser
            .input(TankInput::PrimaryDown(available), &resources)
            .unwrap();
        assert_eq!(
            chooser
                .input(TankInput::PrimaryUp(available), &resources)
                .unwrap(),
            [TankIntent::SelectTank {
                mode: SelectionMode::TimeTrial,
                tank: Tank::One
            }]
        );
        assert!(
            chooser
                .input(
                    TankInput::PrimaryDown(Point {
                        x: f32::NAN,
                        y: 205.0
                    }),
                    &resources
                )
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn facts_focus_modal_and_cancel_invalidate_old_captures() {
        let resources = Resources::new();
        let mut chooser = TankSelection::new(SelectionMode::Challenge, facts([true; 4], true));
        let location = point(chooser.scene(&resources).unwrap().cards[0].button.bounds);
        chooser
            .input(TankInput::PrimaryDown(location), &resources)
            .unwrap();
        chooser.set_facts(facts([false, true, true, true], true));
        assert!(
            chooser
                .input(TankInput::PrimaryUp(location), &resources)
                .unwrap()
                .is_empty()
        );
        chooser.set_facts(facts([true; 4], true));
        for (block, restore) in [
            (TankInput::Focus(false), TankInput::Focus(true)),
            (TankInput::Modal(true), TankInput::Modal(false)),
            (TankInput::Cancel, TankInput::Cancel),
        ] {
            chooser
                .input(TankInput::PrimaryDown(location), &resources)
                .unwrap();
            chooser.input(block, &resources).unwrap();
            assert!(
                chooser
                    .input(TankInput::PrimaryDown(location), &resources)
                    .unwrap()
                    .is_empty()
                    || block == TankInput::Cancel
            );
            chooser.input(restore, &resources).unwrap();
            assert!(
                chooser
                    .input(TankInput::PrimaryUp(location), &resources)
                    .unwrap()
                    .is_empty()
            );
        }
        let stories = point(chooser.scene(&resources).unwrap().stories.unwrap().bounds);
        chooser
            .input(TankInput::PrimaryDown(stories), &resources)
            .unwrap();
        chooser.set_facts(facts([true; 4], false));
        assert!(
            chooser
                .input(TankInput::PrimaryUp(stories), &resources)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn menu_stories_and_pressed_resource_crops_emit_only_typed_intentions() {
        let resources = Resources::new();
        let mut chooser = TankSelection::new(SelectionMode::Challenge, facts([true; 4], true));
        let menu = point(chooser.scene(&resources).unwrap().menu.bounds);
        chooser
            .input(TankInput::PrimaryDown(menu), &resources)
            .unwrap();
        let scene = chooser.scene(&resources).unwrap();
        assert!(scene.commands.iter().any(|command| matches!(
            command.kind,
            DrawKind::Image(ImageDraw {
                resource: TankImage::MainButton,
                frame: ImageFrame::Button(ButtonState::Pressed),
                crop: Rect {
                    x: 162,
                    width: 81,
                    height: 31,
                    ..
                },
                ..
            })
        )));
        assert_eq!(
            chooser
                .input(TankInput::PrimaryUp(menu), &resources)
                .unwrap(),
            [TankIntent::Menu]
        );
        let stories = point(chooser.scene(&resources).unwrap().stories.unwrap().bounds);
        chooser
            .input(TankInput::PrimaryDown(stories), &resources)
            .unwrap();
        assert_eq!(
            chooser
                .input(TankInput::PrimaryUp(stories), &resources)
                .unwrap(),
            [TankIntent::Stories]
        );
        chooser.update();
        assert!(chooser.take_dirty());
        assert!(!chooser.take_dirty());
    }

    #[test]
    fn missing_or_invalid_metadata_is_unavailable_and_cancels_input() {
        let mut resources = Resources::new();
        let mut chooser = TankSelection::new(SelectionMode::Challenge, facts([true; 4], true));
        let location = point(chooser.scene(&resources).unwrap().cards[0].button.bounds);
        chooser
            .input(TankInput::PrimaryDown(location), &resources)
            .unwrap();
        resources.missing_font = Some(TankFont::Small);
        assert_eq!(
            chooser.input(TankInput::PrimaryUp(location), &resources),
            Err(SceneUnavailable::MissingFont(TankFont::Small))
        );
        resources.missing_font = None;
        assert!(
            chooser
                .input(TankInput::PrimaryUp(location), &resources)
                .unwrap()
                .is_empty()
        );
        resources
            .images
            .get_mut(&TankImage::MainButton)
            .unwrap()
            .width = 242;
        assert_eq!(
            chooser.scene(&resources),
            Err(SceneUnavailable::InvalidImage(TankImage::MainButton))
        );
        resources.images.remove(&TankImage::MainButton);
        assert_eq!(
            chooser.scene(&resources),
            Err(SceneUnavailable::MissingImage(TankImage::MainButton))
        );
        let mut resources = Resources::new();
        resources.font.layers[0].ascent = i32::MAX;
        assert!(matches!(
            chooser.scene(&resources),
            Err(SceneUnavailable::InvalidFont(TankFont::Title, _))
        ));
    }
}
