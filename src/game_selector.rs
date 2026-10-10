//! Transient main-menu controller and scene, informed by pinned W1 GameSelector.
//!
//! Profiles, admission, Sandbox key matching, assets, routing and saves remain
//! caller-owned. No production registration is part of this independent chunk.
//! See `docs/main-menu.md` for evidence, integration and deliberate adaptations.

pub const CANVAS: PixelRect = PixelRect {
    x: 0,
    y: 0,
    width: 640,
    height: 480,
};
const PROFILE_LINK: &str = "Switch profile";
const TROPHY_ORIGIN: Point = Point { x: 170, y: 310 };
const TROPHY_HOVER: PixelRect = PixelRect {
    x: 220,
    y: 340,
    width: 50,
    height: 50,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PixelRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl PixelRect {
    pub fn contains(self, point: Point) -> bool {
        let x = i64::from(point.x) - i64::from(self.x);
        let y = i64::from(point.y) - i64::from(self.y);
        x >= 0 && y >= 0 && x < i64::from(self.width) && y < i64::from(self.height)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectorImage {
    Background,
    Blank,
    AdventureOver,
    AdventureDown,
    MiddleOver,
    MiddleDown,
    Options,
    Quit,
    HallOfFame,
    Help,
    SpeechBubble,
    WelcomeTo,
    WelcomeBack,
    SilverTrophy,
    GoldTrophy,
    MerylBlink,
    MerylTail,
    Sparkle,
}

impl SelectorImage {
    pub const ALL: [Self; 18] = [
        Self::Background,
        Self::Blank,
        Self::AdventureOver,
        Self::AdventureDown,
        Self::MiddleOver,
        Self::MiddleDown,
        Self::Options,
        Self::Quit,
        Self::HallOfFame,
        Self::Help,
        Self::SpeechBubble,
        Self::WelcomeTo,
        Self::WelcomeBack,
        Self::SilverTrophy,
        Self::GoldTrophy,
        Self::MerylBlink,
        Self::MerylTail,
        Self::Sparkle,
    ];

    pub const fn resource_id(self) -> &'static str {
        match self {
            Self::Background => "IMAGE_SELECTORSCREEN",
            Self::Blank => "IMAGE_BLANK",
            Self::AdventureOver => "IMAGE_BATTLETANKBUTTON",
            Self::AdventureDown => "IMAGE_BATTLETANKBUTTOND",
            Self::MiddleOver => "IMAGE_MIDDLEBUTTON",
            Self::MiddleDown => "IMAGE_MIDDLEBUTTOND",
            Self::Options => "IMAGE_LEFTBUTTON",
            Self::Quit => "IMAGE_RIGHTBUTTON",
            Self::HallOfFame => "IMAGE_MAINBUTTON",
            Self::Help => "IMAGE_CENTERBUTTON",
            Self::SpeechBubble => "IMAGE_SELECTORSPEECHBUBBLE",
            Self::WelcomeTo => "IMAGE_WELCOMETO",
            Self::WelcomeBack => "IMAGE_WELCOMEBACK",
            Self::SilverTrophy => "IMAGE_BONUSAWARD",
            Self::GoldTrophy => "IMAGE_BONUSAWARD2",
            Self::MerylBlink => "IMAGE_MERYLBLINK",
            Self::MerylTail => "IMAGE_MERYLTAIL",
            Self::Sparkle => "IMAGE_SPARKLE",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectorFont {
    ModeButton,
    SmallOutline,
    Hint,
    ProfileName,
    ProfileLink,
}

impl SelectorFont {
    pub const fn resource_id(self) -> &'static str {
        match self {
            Self::ModeButton => "FONT_JUNGLEFEVER15OUTLINE",
            Self::SmallOutline => "FONT_JUNGLEFEVER10OUTLINE",
            Self::Hint => "FONT_BLAMBOTPRO8",
            Self::ProfileName => "FONT_BLAMBOTPRO15",
            Self::ProfileLink => "FONT_TINY",
        }
    }
}

/// Actual decoded image/sheet metadata. Invalid metadata is never guessed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageMetadata {
    pub width: u32,
    pub height: u32,
    pub columns: u32,
    pub rows: u32,
}

impl ImageMetadata {
    fn valid(self) -> bool {
        self.width > 0
            && self.height > 0
            && self.width <= u32::from(u16::MAX)
            && self.height <= u32::from(u16::MAX)
            && self.columns > 0
            && self.rows > 0
            && self.width.is_multiple_of(self.columns)
            && self.height.is_multiple_of(self.rows)
    }

    fn whole(self) -> PixelRect {
        PixelRect {
            x: 0,
            y: 0,
            width: self.width,
            height: self.height,
        }
    }

    fn cel(self, cel: u32) -> Option<PixelRect> {
        if !self.valid() || cel >= self.columns.checked_mul(self.rows)? {
            return None;
        }
        let width = self.width / self.columns;
        let height = self.height / self.rows;
        Some(PixelRect {
            x: ((cel % self.columns) * width) as i32,
            y: ((cel / self.columns) * height) as i32,
            width,
            height,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FontMetadata {
    pub height: u32,
    pub ascent: u32,
    pub ascent_padding: u32,
}

impl FontMetadata {
    fn valid(self) -> bool {
        self.height > 0
            && self.height <= u32::from(u16::MAX)
            && self.ascent <= u32::from(u16::MAX)
            && self.ascent_padding <= self.ascent
    }
}

/// Read-only snapshot of loaded resources; methods must not advance animation.
pub trait SelectorResources {
    fn image(&self, image: SelectorImage) -> Option<ImageMetadata>;
    fn font(&self, font: SelectorFont) -> Option<FontMetadata>;
    fn text_width(&self, font: SelectorFont, text: &str) -> Option<u32>;
    /// Resolve the supplied countdown with the actual resource's AnimInfo.
    /// Return None when the mapping is unavailable or its cel is hidden.
    fn animation_cel(&self, image: SelectorImage, countdown: u16) -> Option<u32>;
    /// A present AnimInfo can deliberately select a hidden (-1) cel. Keep
    /// that distinct from an unavailable mapping; neither draws a frame.
    fn animation_is_hidden(&self, _image: SelectorImage, _countdown: u16) -> bool {
        false
    }
    fn hint_text(&self, _hint: HintKey) -> Option<&str> {
        None
    }
    fn profile_link_text(&self) -> &str {
        PROFILE_LINK
    }
}

/// W1 uses its seeded character stream and a separate Rand() effect stream.
/// Modulo bounds here preserve source-shaped draws, not retail PRNG parity.
pub trait SelectorRng {
    fn character_u32(&mut self) -> u32;
    fn effect_u32(&mut self) -> u32;
}

pub trait TrophyMask {
    fn available(&self, image: SelectorImage) -> bool;
    fn alpha(&self, image: SelectorImage, x: u32, y: u32) -> Option<u8>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trophy {
    None,
    Silver,
    Gold,
}

/// Presentation facts only; neither labels nor trophy appearance grant access.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProfileFacts<'a> {
    pub id: u64,
    pub name: &'a str,
    pub tank: u8,
    pub level: u8,
    pub adventure_completed: bool,
    pub returning_player: bool,
    pub cyrax_defeats: u32,
    pub shells: u32,
    pub trophy: Trophy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Adventure,
    TimeTrial,
    Challenge,
    VirtualTank,
    Sandbox,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockReason {
    NoProfile,
    CompleteAdventureTank,
    CompleteAdventure,
    CompleteBonusPurchases,
    CallerBlocked,
}

impl LockReason {
    /// Project-authored fallback explanations, not copied dialog prose.
    pub const fn guidance(self) -> &'static str {
        match self {
            Self::NoProfile => "Choose a profile first.",
            Self::CompleteAdventureTank => "Finish an Adventure tank to unlock Time Trial.",
            Self::CompleteAdventure => "Finish Adventure to unlock Challenge.",
            Self::CompleteBonusPurchases => "Complete the bonus purchases to enter Sandbox.",
            Self::CallerBlocked => "This mode is currently unavailable.",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Availability {
    Available,
    Locked(LockReason),
}

/// Supply these from production's existing admission rules on every call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModeAvailability {
    pub adventure: Availability,
    pub time_trial: Availability,
    pub challenge: Availability,
    pub virtual_tank: Availability,
    pub sandbox: Availability,
}

impl ModeAvailability {
    pub const fn get(self, mode: Mode) -> Availability {
        match mode {
            Mode::Adventure => self.adventure,
            Mode::TimeTrial => self.time_trial,
            Mode::Challenge => self.challenge,
            Mode::VirtualTank => self.virtual_tank,
            Mode::Sandbox => self.sandbox,
        }
    }
}

impl Default for ModeAvailability {
    fn default() -> Self {
        let unavailable = Availability::Locked(LockReason::NoProfile);
        Self {
            adventure: unavailable,
            time_trial: unavailable,
            challenge: unavailable,
            virtual_tank: unavailable,
            sandbox: unavailable,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SelectorFacts<'a> {
    pub profile: Option<ProfileFacts<'a>>,
    pub modes: ModeAvailability,
    pub modal_open: bool,
    pub debug_keys_enabled: bool,
    /// Increment when caller replaces profile/session/input context.
    pub context_epoch: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonId {
    Adventure,
    VirtualTank,
    Options,
    Quit,
    HallOfFame,
    Help,
    TimeTrial,
    Challenge,
    ProfileLink,
}

impl ButtonId {
    pub const ALL: [Self; 9] = [
        Self::Adventure,
        Self::TimeTrial,
        Self::Challenge,
        Self::VirtualTank,
        Self::Options,
        Self::Quit,
        Self::HallOfFame,
        Self::Help,
        Self::ProfileLink,
    ];

    pub const fn source_id(self) -> u16 {
        match self {
            Self::Adventure => 1,
            Self::VirtualTank => 2,
            Self::Options => 3,
            Self::Quit => 4,
            Self::HallOfFame => 6,
            Self::Help => 7,
            Self::TimeTrial => 8,
            Self::Challenge => 9,
            Self::ProfileLink => 9999,
        }
    }

    pub const fn mode(self) -> Option<Mode> {
        match self {
            Self::Adventure => Some(Mode::Adventure),
            Self::TimeTrial => Some(Mode::TimeTrial),
            Self::Challenge => Some(Mode::Challenge),
            Self::VirtualTank => Some(Mode::VirtualTank),
            _ => None,
        }
    }

    fn component(self) -> Option<SelectorImage> {
        match self {
            Self::Options => Some(SelectorImage::Options),
            Self::Quit => Some(SelectorImage::Quit),
            Self::HallOfFame => Some(SelectorImage::HallOfFame),
            Self::Help => Some(SelectorImage::Help),
            _ => None,
        }
    }

    fn label(self, facts: &SelectorFacts<'_>, resources: &impl SelectorResources) -> String {
        match self {
            Self::Adventure => adventure_label(facts.profile.as_ref()).to_owned(),
            Self::VirtualTank => "Virtual Tank".to_owned(),
            Self::Options => "Options".to_owned(),
            Self::Quit => "Quit".to_owned(),
            Self::HallOfFame => "Hall of Fame".to_owned(),
            Self::Help => "Help".to_owned(),
            Self::TimeTrial => "Time Trial".to_owned(),
            Self::Challenge => "Challenge".to_owned(),
            Self::ProfileLink => resources.profile_link_text().to_owned(),
        }
    }
}

pub fn adventure_label(profile: Option<&ProfileFacts<'_>>) -> &'static str {
    match profile {
        Some(profile) if profile.adventure_completed => "Bonus Adventure",
        Some(profile) if profile.tank == 1 && profile.level == 1 => "Start Adventure",
        _ => "Adventure",
    }
}

pub fn stage_subtitle(profile: &ProfileFacts<'_>) -> Option<String> {
    if profile.tank == 5 && profile.level == 1 {
        // u64 preserves the next-attempt label even at the caller's u32 limit.
        let attempt = u64::from(profile.cyrax_defeats) + 1;
        if attempt == 1 {
            return Some("FINAL BOSS".to_owned());
        }
        let suffix = if (attempt / 10) % 10 == 1 {
            "th"
        } else {
            match attempt % 10 {
                1 => "st",
                2 => "nd",
                3 => "rd",
                _ => "th",
            }
        };
        Some(format!("{attempt}{suffix} FINAL BOSS"))
    } else if profile.level == 6 {
        Some(format!("Bonus Level {}", profile.tank))
    } else if profile.tank != 1 || profile.level != 1 {
        Some(format!("Tank {}-{}", profile.tank, profile.level))
    } else {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonState {
    Normal,
    Hover,
    Pressed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectorInput {
    PointerMove(Point),
    PrimaryDown(Point),
    PrimaryUp(Point),
    PointerLeft,
    Cancel,
    FocusLost,
    Character(char),
    /// Forward noncharacter key events once; do not turn them into characters.
    NonCharacterKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerIntent {
    Cursor(Cursor),
    Capture(ButtonId),
    Release(ButtonId),
    CancelCapture(ButtonId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cursor {
    Arrow,
    Hand,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonIntent {
    Pressed(ButtonId),
    Activated(ButtonId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectorSound {
    ButtonClick,
}

impl SelectorSound {
    pub const fn resource_id(self) -> &'static str {
        match self {
            Self::ButtonClick => "SOUND_BUTTONCLICK",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Destination {
    Mode(Mode),
    HallOfFame,
    Help,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogIntent {
    Options,
    Quit,
    ChooseProfile,
    OpenGiveDialog,
    /// W1's mode-lock explanation uses dialog ID 14. Text is caller-owned.
    LockedMode {
        mode: Mode,
        reason: LockReason,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectorIntent {
    Pointer(PointerIntent),
    Button(ButtonIntent),
    Sound(SelectorSound),
    Navigate(Destination),
    Dialog(DialogIntent),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HintKey {
    Welcome,
    AdventureStart,
    AdventureContinue,
    TimeTrial,
    Challenge,
    VirtualTank,
    SilverTrophy,
    GoldTrophy,
}

impl HintKey {
    fn same_hover(self, other: Self) -> bool {
        self == other
            || matches!(
                (self, other),
                (Self::AdventureStart, Self::AdventureContinue)
                    | (Self::AdventureContinue, Self::AdventureStart)
            )
    }

    /// Concise project-authored guidance. The original prose catalog is deferred.
    pub const fn guidance(self) -> &'static str {
        match self {
            Self::Welcome => "",
            Self::AdventureStart => "Begin Adventure. Feed fish and defend your tank.",
            Self::AdventureContinue => "Continue your Adventure progress.",
            Self::TimeTrial => "Earn as much as possible before time expires.",
            Self::Challenge => "Defend your tank against harder alien waves.",
            Self::VirtualTank => "Care for your collection and set up its screensaver.",
            Self::SilverTrophy => "Your bonus-purchase trophy.",
            Self::GoldTrophy => "Your champion trophy.",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HintState {
    pub current: HintKey,
    pub previous: HintKey,
    pub divergent_updates: u8,
    pub fade_remaining: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MerylState {
    pub blink_countdown: u16,
    pub tail_countdown: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unavailable {
    Image(SelectorImage),
    InvalidImage(SelectorImage),
    Animation(SelectorImage),
    Font(SelectorFont),
    TextMetrics(SelectorFont),
    Mask(SelectorImage),
    OpaqueTrophyPoint,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SparkleState {
    /// 0 is idle; 1..59 is the source lifecycle, including nondrawing clocks.
    pub clock: u8,
    pub position: Option<Point>,
    pub unavailable: Option<Unavailable>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Layer {
    Background,
    Trophy,
    Meryl,
    Speech,
    Buttons,
    Overlay,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blend {
    Alpha,
    Additive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Paint {
    pub layer: Layer,
    pub alpha: u8,
    pub blend: Blend,
    pub clip: PixelRect,
}

impl Paint {
    fn new(layer: Layer) -> Self {
        Self {
            layer,
            alpha: 255,
            blend: Blend::Alpha,
            clip: CANVAS,
        }
    }

    fn with_alpha(mut self, alpha: u8) -> Self {
        self.alpha = alpha;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageFrame {
    Whole,
    Cel(u32),
    /// Three horizontal state regions; caller renders the selected image box.
    Component {
        state: ButtonState,
        bounds: PixelRect,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageSizing {
    Natural,
    ImageBox,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageDraw {
    pub image: SelectorImage,
    pub frame: ImageFrame,
    pub source: PixelRect,
    pub destination: PixelRect,
    pub sizing: ImageSizing,
    pub paint: Paint,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HorizontalAlign {
    Left,
    Center,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextLayout {
    Baseline {
        anchor: Point,
        align: HorizontalAlign,
    },
    /// Use actual glyph metrics to wrap, then center the wrapped height.
    WrappedCenter {
        bounds: PixelRect,
        line_spacing: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextDraw {
    pub font: SelectorFont,
    pub text: String,
    pub hint: Option<HintKey>,
    pub layout: TextLayout,
    pub rgb: [u8; 3],
    pub underline: bool,
    pub paint: Paint,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DrawCommand {
    Image(ImageDraw),
    Text(TextDraw),
}

impl DrawCommand {
    pub fn layer(&self) -> Layer {
        match self {
            Self::Image(draw) => draw.paint.layer,
            Self::Text(draw) => draw.paint.layer,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ButtonView {
    pub id: ButtonId,
    pub bounds: Option<PixelRect>,
    pub state: ButtonState,
    pub availability: Availability,
    pub interactive: bool,
    pub captured: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectorScene {
    pub commands: Vec<DrawCommand>,
    pub buttons: Vec<ButtonView>,
    pub unavailable: Vec<Unavailable>,
    pub updates: u64,
    pub hint: HintState,
    pub meryl: MerylState,
    pub sparkle: SparkleState,
}

impl SelectorScene {
    fn missing(&mut self, unavailable: Unavailable) {
        if !self.unavailable.contains(&unavailable) {
            self.unavailable.push(unavailable);
        }
    }

    fn push_image(
        &mut self,
        resources: &impl SelectorResources,
        image: SelectorImage,
        origin: Point,
        frame: ImageFrame,
        paint: Paint,
    ) {
        let metadata = match checked_image(resources, image) {
            Ok(metadata) => metadata,
            Err(issue) => {
                self.missing(issue);
                return;
            }
        };
        let (source, destination, sizing) = match frame {
            ImageFrame::Whole => {
                let source = metadata.whole();
                (
                    source,
                    PixelRect {
                        x: origin.x,
                        y: origin.y,
                        ..source
                    },
                    ImageSizing::Natural,
                )
            }
            ImageFrame::Cel(cel) => {
                let Some(source) = metadata.cel(cel) else {
                    self.missing(Unavailable::Animation(image));
                    return;
                };
                (
                    source,
                    PixelRect {
                        x: origin.x,
                        y: origin.y,
                        ..source
                    },
                    ImageSizing::Natural,
                )
            }
            ImageFrame::Component { state, bounds } => {
                if !metadata.width.is_multiple_of(3) {
                    self.missing(Unavailable::InvalidImage(image));
                    return;
                }
                let index = match state {
                    ButtonState::Normal => 0,
                    ButtonState::Hover => 1,
                    ButtonState::Pressed => 2,
                };
                let width = metadata.width / 3;
                let source = PixelRect {
                    x: (index * width) as i32,
                    y: 0,
                    width,
                    height: metadata.height,
                };
                (source, bounds, ImageSizing::ImageBox)
            }
        };
        self.commands.push(DrawCommand::Image(ImageDraw {
            image,
            frame,
            source,
            destination,
            sizing,
            paint,
        }));
    }

    fn push_animation(
        &mut self,
        resources: &impl SelectorResources,
        image: SelectorImage,
        origin: Point,
        countdown: u16,
    ) {
        // Report missing image separately instead of inventing an animation.
        if let Err(issue) = checked_image(resources, image) {
            self.missing(issue);
            return;
        }
        if resources.animation_is_hidden(image, countdown) {
            return;
        }
        if let Some(cel) = resources.animation_cel(image, countdown) {
            self.push_image(
                resources,
                image,
                origin,
                ImageFrame::Cel(cel),
                Paint::new(Layer::Meryl),
            );
        } else {
            self.missing(Unavailable::Animation(image));
        }
    }

    fn push_text(&mut self, resources: &impl SelectorResources, draw: TextDraw) {
        if checked_font(resources, draw.font).is_none() {
            self.missing(Unavailable::Font(draw.font));
            return;
        }
        if resources.text_width(draw.font, &draw.text).is_none() {
            self.missing(Unavailable::TextMetrics(draw.font));
            return;
        }
        self.commands.push(DrawCommand::Text(draw));
    }
}

fn checked_image(
    resources: &impl SelectorResources,
    image: SelectorImage,
) -> Result<ImageMetadata, Unavailable> {
    match resources.image(image) {
        Some(metadata) if metadata.valid() => Ok(metadata),
        Some(_) => Err(Unavailable::InvalidImage(image)),
        None => Err(Unavailable::Image(image)),
    }
}

fn checked_font(resources: &impl SelectorResources, font: SelectorFont) -> Option<FontMetadata> {
    resources.font(font).filter(|metadata| metadata.valid())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Capture {
    button: ButtonId,
    epoch: u64,
    profile: Option<u64>,
}

/// Only transient scene state. Construct a new selector after leaving this scene.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GameSelector {
    updates: u64,
    pointer: Option<Point>,
    capture: Option<Capture>,
    hint: HintState,
    meryl: MerylState,
    sparkle: SparkleState,
    give_matched: u8,
    give_context: Option<(u64, Option<u64>)>,
}

impl GameSelector {
    pub fn new(random: &mut impl SelectorRng) -> Self {
        Self {
            updates: 0,
            pointer: None,
            capture: None,
            hint: HintState {
                current: HintKey::Welcome,
                previous: HintKey::Welcome,
                divergent_updates: 0,
                fade_remaining: 0,
            },
            meryl: MerylState {
                blink_countdown: (random.character_u32() % 150 + 100) as u16,
                tail_countdown: (random.character_u32() % 500 + 200) as u16,
            },
            sparkle: SparkleState {
                clock: 0,
                position: None,
                unavailable: None,
            },
            give_matched: 0,
            give_context: None,
        }
    }

    fn capture_current(&self, facts: &SelectorFacts<'_>) -> Option<ButtonId> {
        self.capture
            .filter(|capture| {
                !facts.modal_open
                    && capture.epoch == facts.context_epoch
                    && capture.profile == facts.profile.map(|profile| profile.id)
            })
            .map(|capture| capture.button)
    }

    fn cancel_capture(&mut self, intentions: &mut Vec<SelectorIntent>) {
        if let Some(capture) = self.capture.take() {
            intentions.push(SelectorIntent::Pointer(PointerIntent::CancelCapture(
                capture.button,
            )));
        }
    }

    fn reconcile(
        &mut self,
        facts: &SelectorFacts<'_>,
        resources: &impl SelectorResources,
        intentions: &mut Vec<SelectorIntent>,
    ) {
        let context = (facts.context_epoch, facts.profile.map(|profile| profile.id));
        if facts.modal_open || facts.debug_keys_enabled || self.give_context != Some(context) {
            self.give_matched = 0;
        }
        self.give_context = Some(context);
        if self.capture.is_some()
            && self
                .capture_current(facts)
                .is_none_or(|button| self.button_bounds(button, facts, resources).is_none())
        {
            self.cancel_capture(intentions);
        }
    }

    pub fn button_bounds(
        &self,
        button: ButtonId,
        facts: &SelectorFacts<'_>,
        resources: &impl SelectorResources,
    ) -> Option<PixelRect> {
        let (x, y, width, height) = match button {
            ButtonId::Adventure => (357, 48, 217, 66),
            ButtonId::VirtualTank => (357, 287, 217, 66),
            ButtonId::TimeTrial => (359, 142, 213, 48),
            ButtonId::Challenge => (359, 212, 213, 48),
            ButtonId::Options | ButtonId::Quit | ButtonId::HallOfFame | ButtonId::Help => {
                let metadata = checked_image(resources, button.component()?).ok()?;
                if !metadata.width.is_multiple_of(3) {
                    return None;
                }
                match button {
                    ButtonId::Options => (325, 412, 92, metadata.height),
                    ButtonId::Quit => (514, 412, 89, metadata.height),
                    ButtonId::HallOfFame => (401, 380, 127, metadata.height),
                    ButtonId::Help => (419, 412, 93, metadata.height),
                    _ => unreachable!(),
                }
            }
            ButtonId::ProfileLink => {
                if self.hint.current != HintKey::Welcome
                    || facts.profile.is_none_or(|profile| profile.name.is_empty())
                {
                    return None;
                }
                let font = checked_font(resources, SelectorFont::ProfileLink)?;
                let width = resources
                    .text_width(SelectorFont::ProfileLink, resources.profile_link_text())?;
                if width == 0 || width > u32::from(u16::MAX) {
                    return None;
                }
                (97, 90, width, font.height)
            }
        };
        Some(PixelRect {
            x,
            y,
            width,
            height,
        })
    }

    fn hit_button(
        &self,
        point: Point,
        facts: &SelectorFacts<'_>,
        resources: &impl SelectorResources,
    ) -> Option<ButtonId> {
        if facts.modal_open || !CANVAS.contains(point) {
            return None;
        }
        ButtonId::ALL.into_iter().find(|&button| {
            self.button_bounds(button, facts, resources)
                .is_some_and(|bounds| bounds.contains(point))
        })
    }

    fn cursor(&self, facts: &SelectorFacts<'_>, resources: &impl SelectorResources) -> Cursor {
        if self
            .pointer
            .and_then(|point| self.hit_button(point, facts, resources))
            .is_some()
        {
            Cursor::Hand
        } else {
            Cursor::Arrow
        }
    }

    fn availability(&self, mode: Mode, facts: &SelectorFacts<'_>) -> Availability {
        if facts.profile.is_none() {
            Availability::Locked(LockReason::NoProfile)
        } else {
            facts.modes.get(mode)
        }
    }

    fn route_mode(&self, mode: Mode, facts: &SelectorFacts<'_>) -> SelectorIntent {
        match self.availability(mode, facts) {
            Availability::Available => SelectorIntent::Navigate(Destination::Mode(mode)),
            Availability::Locked(reason) => {
                SelectorIntent::Dialog(DialogIntent::LockedMode { mode, reason })
            }
        }
    }

    fn activate(&self, button: ButtonId, facts: &SelectorFacts<'_>) -> SelectorIntent {
        match button {
            ButtonId::Adventure => self.route_mode(Mode::Adventure, facts),
            ButtonId::VirtualTank => self.route_mode(Mode::VirtualTank, facts),
            ButtonId::TimeTrial => self.route_mode(Mode::TimeTrial, facts),
            ButtonId::Challenge => self.route_mode(Mode::Challenge, facts),
            ButtonId::Options => SelectorIntent::Dialog(DialogIntent::Options),
            ButtonId::Quit => SelectorIntent::Dialog(DialogIntent::Quit),
            ButtonId::HallOfFame => SelectorIntent::Navigate(Destination::HallOfFame),
            ButtonId::Help => SelectorIntent::Navigate(Destination::Help),
            ButtonId::ProfileLink => SelectorIntent::Dialog(DialogIntent::ChooseProfile),
        }
    }

    /// Emit intentions in order. Only a matched down/up pair activates a button.
    pub fn input(
        &mut self,
        input: SelectorInput,
        facts: &SelectorFacts<'_>,
        resources: &impl SelectorResources,
    ) -> Vec<SelectorIntent> {
        let mut intentions = Vec::new();
        self.reconcile(facts, resources, &mut intentions);
        match input {
            SelectorInput::PointerMove(point) => self.pointer = Some(point),
            SelectorInput::PrimaryDown(point) => {
                self.pointer = Some(point);
                // Duplicate OS down events cannot replace an active capture.
                if self.capture.is_none() && !facts.modal_open {
                    if let Some(button) = self.hit_button(point, facts, resources) {
                        self.capture = Some(Capture {
                            button,
                            epoch: facts.context_epoch,
                            profile: facts.profile.map(|profile| profile.id),
                        });
                        intentions.push(SelectorIntent::Pointer(PointerIntent::Capture(button)));
                        intentions.push(SelectorIntent::Button(ButtonIntent::Pressed(button)));
                        intentions.push(SelectorIntent::Sound(SelectorSound::ButtonClick));
                    } else if point.x > 19
                        && point.x < 310
                        && point.y > 149
                        && point.y < 510
                        && self.meryl.tail_countdown > 15
                    {
                        self.meryl.tail_countdown = 15;
                    }
                }
            }
            SelectorInput::PrimaryUp(point) => {
                self.pointer = Some(point);
                if let Some(capture) = self.capture.take() {
                    intentions.push(SelectorIntent::Pointer(PointerIntent::Release(
                        capture.button,
                    )));
                    if self.hit_button(point, facts, resources) == Some(capture.button) {
                        intentions.push(SelectorIntent::Button(ButtonIntent::Activated(
                            capture.button,
                        )));
                        intentions.push(self.activate(capture.button, facts));
                    }
                }
            }
            SelectorInput::PointerLeft | SelectorInput::Cancel | SelectorInput::FocusLost => {
                self.pointer = None;
                self.cancel_capture(&mut intentions);
                self.give_matched = 0;
            }
            SelectorInput::Character(character) => {
                if !facts.modal_open && !facts.debug_keys_enabled {
                    const CODE: [char; 4] = ['g', 'i', 'v', 'e'];
                    if character == CODE[usize::from(self.give_matched)] {
                        self.give_matched += 1;
                        if self.give_matched == 4 {
                            self.give_matched = 0;
                            intentions.push(SelectorIntent::Dialog(DialogIntent::OpenGiveDialog));
                        }
                    } else {
                        self.give_matched = 0;
                    }
                }
            }
            SelectorInput::NonCharacterKey => {}
        }
        intentions.push(SelectorIntent::Pointer(PointerIntent::Cursor(
            self.cursor(facts, resources),
        )));
        intentions
    }

    /// Call only after the existing sandbox::EntryCode fires. Admission is the
    /// caller's sandbox::admitted result, projected into facts.modes.sandbox.
    /// There is no matcher, capacity mutation, Give transaction or Relax entry.
    pub fn sandbox_activated(&self, facts: &SelectorFacts<'_>) -> Option<SelectorIntent> {
        if facts.modal_open || facts.debug_keys_enabled {
            None
        } else {
            Some(self.route_mode(Mode::Sandbox, facts))
        }
    }

    fn desired_hint(
        &self,
        facts: &SelectorFacts<'_>,
        resources: &impl SelectorResources,
    ) -> HintKey {
        if facts.modal_open {
            return HintKey::Welcome;
        }
        let button = self.capture_current(facts).or_else(|| {
            self.pointer
                .and_then(|point| self.hit_button(point, facts, resources))
        });
        match button {
            Some(ButtonId::Adventure) => {
                if facts.profile.is_none_or(|profile| {
                    profile.tank == 1 && profile.level == 1 && !profile.adventure_completed
                }) {
                    HintKey::AdventureStart
                } else {
                    HintKey::AdventureContinue
                }
            }
            Some(ButtonId::TimeTrial) => HintKey::TimeTrial,
            Some(ButtonId::Challenge) => HintKey::Challenge,
            Some(ButtonId::VirtualTank) => HintKey::VirtualTank,
            _ if self
                .pointer
                .is_some_and(|point| TROPHY_HOVER.contains(point)) =>
            {
                match facts
                    .profile
                    .map(|profile| profile.trophy)
                    .unwrap_or(Trophy::None)
                {
                    Trophy::Silver => HintKey::SilverTrophy,
                    Trophy::Gold => HintKey::GoldTrophy,
                    Trophy::None => HintKey::Welcome,
                }
            }
            _ => HintKey::Welcome,
        }
    }

    /// One explicit caller-owned UI tick. Rendering/input never advances clocks.
    pub fn update(
        &mut self,
        facts: &SelectorFacts<'_>,
        resources: &impl SelectorResources,
        random: &mut impl SelectorRng,
        mask: &impl TrophyMask,
    ) -> Vec<SelectorIntent> {
        let mut intentions = Vec::new();
        self.reconcile(facts, resources, &mut intentions);
        self.updates = self.updates.saturating_add(1);
        self.meryl.blink_countdown -= 1;
        if self.meryl.blink_countdown == 0 {
            self.meryl.blink_countdown = if random.character_u32().is_multiple_of(7) {
                (random.character_u32() % 15 + 15) as u16
            } else {
                (random.character_u32() % 150 + 100) as u16
            };
        }
        self.meryl.tail_countdown -= 1;
        if self.meryl.tail_countdown == 0 {
            self.meryl.tail_countdown = (random.character_u32() % 500 + 200) as u16;
        }
        self.update_sparkle(facts, resources, random, mask);
        let desired = self.desired_hint(facts, resources);
        if !self.hint.current.same_hover(desired) {
            self.hint.divergent_updates += 1;
            if self.hint.divergent_updates > 5 {
                self.hint.previous = self.hint.current;
                self.hint.current = desired;
                self.hint.divergent_updates = 0;
                self.hint.fade_remaining = 5;
            }
        } else {
            self.hint.current = desired;
        }
        self.hint.fade_remaining = self.hint.fade_remaining.saturating_sub(1);
        self.reconcile(facts, resources, &mut intentions);
        intentions.push(SelectorIntent::Pointer(PointerIntent::Cursor(
            self.cursor(facts, resources),
        )));
        intentions
    }

    fn update_sparkle(
        &mut self,
        facts: &SelectorFacts<'_>,
        resources: &impl SelectorResources,
        random: &mut impl SelectorRng,
        mask: &impl TrophyMask,
    ) {
        if !facts
            .profile
            .is_some_and(|profile| profile.trophy == Trophy::Gold)
        {
            self.sparkle = SparkleState {
                clock: 0,
                position: None,
                unavailable: None,
            };
            return;
        }
        if self.sparkle.clock > 0 {
            self.sparkle.clock += 1;
            if self.sparkle.clock > 59 {
                self.sparkle = SparkleState {
                    clock: 0,
                    position: None,
                    unavailable: None,
                };
            }
            return;
        }
        if self.updates <= 40 {
            return;
        }
        self.sparkle.clock = 1;
        match Self::sample_sparkle(resources, random, mask) {
            Ok(position) => {
                self.sparkle.position = Some(position);
                self.sparkle.unavailable = None;
            }
            Err(issue) => {
                self.sparkle.position = None;
                self.sparkle.unavailable = Some(issue);
            }
        }
    }

    fn sample_sparkle(
        resources: &impl SelectorResources,
        random: &mut impl SelectorRng,
        mask: &impl TrophyMask,
    ) -> Result<Point, Unavailable> {
        checked_image(resources, SelectorImage::GoldTrophy)?;
        // W1 deliberately samples the silver image even while drawing gold.
        let trophy = checked_image(resources, SelectorImage::SilverTrophy)?;
        let sparkle = checked_image(resources, SelectorImage::Sparkle)?;
        if !mask.available(SelectorImage::SilverTrophy) {
            return Err(Unavailable::Mask(SelectorImage::SilverTrophy));
        }
        for _ in 0..100 {
            let x = random.effect_u32() % trophy.width;
            let y = random.effect_u32() % trophy.height;
            match mask.alpha(SelectorImage::SilverTrophy, x, y) {
                Some(255) => {
                    return Ok(Point {
                        x: TROPHY_ORIGIN.x + x as i32
                            - (sparkle.width / sparkle.columns / 2) as i32,
                        y: TROPHY_ORIGIN.y + y as i32 - (sparkle.height / sparkle.rows / 2) as i32,
                    });
                }
                Some(_) => {}
                None => return Err(Unavailable::Mask(SelectorImage::SilverTrophy)),
            }
        }
        Err(Unavailable::OpaqueTrophyPoint)
    }

    fn button_state(
        &self,
        button: ButtonId,
        facts: &SelectorFacts<'_>,
        resources: &impl SelectorResources,
    ) -> ButtonState {
        let over = self
            .pointer
            .is_some_and(|point| self.hit_button(point, facts, resources) == Some(button));
        if over && self.capture_current(facts) == Some(button) {
            ButtonState::Pressed
        } else if over {
            ButtonState::Hover
        } else {
            ButtonState::Normal
        }
    }

    /// Pure scene generation. Repeated calls do not consume clocks, RNG or masks.
    pub fn view(
        &self,
        facts: &SelectorFacts<'_>,
        resources: &impl SelectorResources,
    ) -> SelectorScene {
        let mut scene = SelectorScene {
            commands: Vec::new(),
            buttons: Vec::new(),
            unavailable: Vec::new(),
            updates: self.updates,
            hint: self.hint,
            meryl: self.meryl,
            sparkle: self.sparkle,
        };
        scene.push_image(
            resources,
            SelectorImage::Background,
            Point { x: 0, y: 0 },
            ImageFrame::Whole,
            Paint::new(Layer::Background),
        );
        if let Some(profile) = facts.profile {
            let image = match profile.trophy {
                Trophy::None => None,
                Trophy::Silver => Some(SelectorImage::SilverTrophy),
                Trophy::Gold => Some(SelectorImage::GoldTrophy),
            };
            if let Some(image) = image {
                scene.push_image(
                    resources,
                    image,
                    TROPHY_ORIGIN,
                    ImageFrame::Whole,
                    Paint::new(Layer::Trophy),
                );
            }
            if profile.trophy == Trophy::Gold {
                if let Some(issue) = self.sparkle.unavailable {
                    scene.missing(issue);
                }
                if let Some(position) = self.sparkle.position {
                    match checked_image(resources, SelectorImage::Sparkle) {
                        Ok(metadata)
                            if self.sparkle.clock > 0
                                && u32::from(self.sparkle.clock) <= metadata.columns =>
                        {
                            let mut paint = Paint::new(Layer::Trophy);
                            paint.blend = Blend::Additive;
                            scene.push_image(
                                resources,
                                SelectorImage::Sparkle,
                                position,
                                ImageFrame::Cel(u32::from(self.sparkle.clock - 1)),
                                paint,
                            );
                        }
                        Err(issue) => scene.missing(issue),
                        _ => {}
                    }
                }
            }
        }
        scene.push_animation(
            resources,
            SelectorImage::MerylBlink,
            Point { x: 139, y: 196 },
            self.meryl.blink_countdown,
        );
        scene.push_animation(
            resources,
            SelectorImage::MerylTail,
            Point { x: 39, y: 301 },
            self.meryl.tail_countdown,
        );
        let old_alpha = u16::from(self.hint.fade_remaining) * 255 / 5;
        self.draw_speech(
            &mut scene,
            facts,
            resources,
            self.hint.current,
            (255 - old_alpha) as u8,
            true,
        );
        if old_alpha > 0 {
            self.draw_speech(
                &mut scene,
                facts,
                resources,
                self.hint.previous,
                old_alpha as u8,
                false,
            );
        }
        for button in ButtonId::ALL {
            let bounds = self.button_bounds(button, facts, resources);
            let state = self.button_state(button, facts, resources);
            scene.buttons.push(ButtonView {
                id: button,
                bounds,
                state,
                availability: button
                    .mode()
                    .map(|mode| self.availability(mode, facts))
                    .unwrap_or(Availability::Available),
                interactive: bounds.is_some() && !facts.modal_open,
                captured: self.capture_current(facts) == Some(button),
            });
            if button == ButtonId::ProfileLink
                && bounds.is_none()
                && self.hint.current == HintKey::Welcome
                && facts
                    .profile
                    .is_some_and(|profile| !profile.name.is_empty())
            {
                scene.missing(
                    if checked_font(resources, SelectorFont::ProfileLink).is_none() {
                        Unavailable::Font(SelectorFont::ProfileLink)
                    } else {
                        Unavailable::TextMetrics(SelectorFont::ProfileLink)
                    },
                );
            }
            if button != ButtonId::ProfileLink {
                self.draw_button(&mut scene, facts, resources, button, bounds, state);
            }
        }
        self.draw_overlay(&mut scene, facts, resources);
        scene
    }

    fn draw_button(
        &self,
        scene: &mut SelectorScene,
        facts: &SelectorFacts<'_>,
        resources: &impl SelectorResources,
        button: ButtonId,
        bounds: Option<PixelRect>,
        state: ButtonState,
    ) {
        let Some(bounds) = bounds else {
            if let Some(image) = button.component() {
                let issue = checked_image(resources, image)
                    .err()
                    .unwrap_or(Unavailable::InvalidImage(image));
                scene.missing(issue);
            }
            return;
        };
        let mut paint = Paint::new(Layer::Buttons);
        paint.clip = bounds;
        let origin = Point {
            x: bounds.x,
            y: bounds.y,
        };
        let component = button.component();
        if let Some(image) = component {
            scene.push_image(
                resources,
                image,
                origin,
                ImageFrame::Component { state, bounds },
                paint,
            );
        } else {
            let image = match (button, state) {
                (_, ButtonState::Normal) => SelectorImage::Blank,
                (ButtonId::Adventure | ButtonId::VirtualTank, ButtonState::Hover) => {
                    SelectorImage::AdventureOver
                }
                (ButtonId::Adventure | ButtonId::VirtualTank, ButtonState::Pressed) => {
                    SelectorImage::AdventureDown
                }
                (_, ButtonState::Hover) => SelectorImage::MiddleOver,
                (_, ButtonState::Pressed) => SelectorImage::MiddleDown,
            };
            scene.push_image(resources, image, origin, ImageFrame::Whole, paint);
        }
        let font = if component.is_some() {
            SelectorFont::SmallOutline
        } else {
            SelectorFont::ModeButton
        };
        let Some(metrics) = checked_font(resources, font) else {
            scene.missing(Unavailable::Font(font));
            return;
        };
        let pressed = i32::from(state == ButtonState::Pressed);
        let padding = if component.is_some() {
            metrics.ascent_padding
        } else {
            0
        };
        let baseline = (bounds.height as i32 + metrics.ascent as i32
            - padding as i32
            - (metrics.ascent / 6) as i32
            - 1)
            / 2;
        let rgb = match (component.is_some(), state) {
            (_, ButtonState::Normal) => [255, 240, 0],
            (true, _) => [255, 255, 255],
            (false, _) => [200, 200, 255],
        };
        scene.push_text(
            resources,
            TextDraw {
                font,
                text: button.label(facts, resources),
                hint: None,
                layout: TextLayout::Baseline {
                    anchor: Point {
                        x: bounds.x + (bounds.width / 2) as i32 + pressed
                            - if component.is_some() { 2 } else { 0 },
                        y: bounds.y + baseline + pressed,
                    },
                    align: HorizontalAlign::Center,
                },
                rgb,
                underline: false,
                paint,
            },
        );
    }

    fn draw_speech(
        &self,
        scene: &mut SelectorScene,
        facts: &SelectorFacts<'_>,
        resources: &impl SelectorResources,
        hint: HintKey,
        alpha: u8,
        bubble: bool,
    ) {
        let profile = facts.profile.filter(|profile| !profile.name.is_empty());
        if hint == HintKey::Welcome && profile.is_none() {
            return;
        }
        let paint = Paint::new(Layer::Speech).with_alpha(alpha);
        if bubble {
            scene.push_image(
                resources,
                SelectorImage::SpeechBubble,
                Point { x: 45, y: 20 },
                ImageFrame::Whole,
                Paint::new(Layer::Speech),
            );
        }
        if hint != HintKey::Welcome {
            let bounds = PixelRect {
                x: 64,
                y: 40,
                width: 214,
                height: 60,
            };
            scene.push_text(
                resources,
                TextDraw {
                    font: SelectorFont::Hint,
                    text: resources
                        .hint_text(hint)
                        .unwrap_or(hint.guidance())
                        .to_owned(),
                    hint: Some(hint),
                    layout: TextLayout::WrappedCenter {
                        bounds,
                        line_spacing: 20,
                    },
                    rgb: [0, 0, 100],
                    underline: false,
                    paint: Paint {
                        clip: bounds,
                        ..paint
                    },
                },
            );
            return;
        }
        let Some(profile) = profile else {
            return;
        };
        let welcome = if profile.returning_player {
            SelectorImage::WelcomeBack
        } else {
            SelectorImage::WelcomeTo
        };
        match checked_image(resources, welcome) {
            Ok(metadata) => scene.push_image(
                resources,
                welcome,
                Point {
                    x: (250 - metadata.width as i32) / 2 + 45,
                    y: 33,
                },
                ImageFrame::Whole,
                paint,
            ),
            Err(issue) => scene.missing(issue),
        }
        scene.push_text(
            resources,
            TextDraw {
                font: SelectorFont::ProfileName,
                text: format!("{}!", profile.name),
                hint: Some(HintKey::Welcome),
                layout: TextLayout::Baseline {
                    anchor: Point { x: 170, y: 77 },
                    align: HorizontalAlign::Center,
                },
                rgb: [0, 0, 100],
                underline: false,
                paint,
            },
        );
        // During the outgoing welcome fade, draw link text but do not restore
        // its hit area: only the current welcome state may capture that link.
        if let Some(font) = checked_font(resources, SelectorFont::ProfileLink) {
            let over =
                self.button_state(ButtonId::ProfileLink, facts, resources) != ButtonState::Normal;
            scene.push_text(
                resources,
                TextDraw {
                    font: SelectorFont::ProfileLink,
                    text: resources.profile_link_text().to_owned(),
                    hint: None,
                    layout: TextLayout::Baseline {
                        anchor: Point {
                            x: 97,
                            y: 90 + ((font.ascent + font.height) / 2) as i32 - 1,
                        },
                        align: HorizontalAlign::Left,
                    },
                    rgb: if over { [100, 100, 220] } else { [0, 0, 120] },
                    underline: true,
                    paint,
                },
            );
        } else {
            scene.missing(Unavailable::Font(SelectorFont::ProfileLink));
        }
    }

    fn draw_overlay(
        &self,
        scene: &mut SelectorScene,
        facts: &SelectorFacts<'_>,
        resources: &impl SelectorResources,
    ) {
        let Some(profile) = facts.profile else {
            return;
        };
        for (text, y) in [
            (stage_subtitle(&profile), 107),
            (
                (profile.shells > 0).then(|| format!("{} Shells", profile.shells)),
                346,
            ),
        ] {
            if let Some(text) = text {
                scene.push_text(
                    resources,
                    TextDraw {
                        font: SelectorFont::SmallOutline,
                        text,
                        hint: None,
                        layout: TextLayout::Baseline {
                            anchor: Point { x: 465, y },
                            align: HorizontalAlign::Center,
                        },
                        rgb: [225, 250, 250],
                        underline: false,
                        paint: Paint::new(Layer::Overlay),
                    },
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, collections::VecDeque};

    // Declared fixture dimensions are not claims about installed assets.
    struct Resources {
        missing_image: Option<SelectorImage>,
        animations: bool,
    }

    impl Default for Resources {
        fn default() -> Self {
            Self {
                missing_image: None,
                animations: true,
            }
        }
    }

    impl SelectorResources for Resources {
        fn image(&self, image: SelectorImage) -> Option<ImageMetadata> {
            if self.missing_image == Some(image) {
                return None;
            }
            let (width, height, columns) = match image {
                SelectorImage::Options
                | SelectorImage::Quit
                | SelectorImage::Help
                | SelectorImage::HallOfFame => (90, 24, 1),
                SelectorImage::Sparkle => (160, 16, 10),
                SelectorImage::SilverTrophy | SelectorImage::GoldTrophy => (80, 50, 1),
                SelectorImage::MerylBlink | SelectorImage::MerylTail => (30, 10, 3),
                SelectorImage::Background => (640, 480, 1),
                SelectorImage::Blank => (1, 1, 1),
                _ => (60, 20, 1),
            };
            Some(ImageMetadata {
                width,
                height,
                columns,
                rows: 1,
            })
        }

        fn font(&self, _font: SelectorFont) -> Option<FontMetadata> {
            Some(FontMetadata {
                height: 16,
                ascent: 12,
                ascent_padding: 2,
            })
        }

        fn text_width(&self, _font: SelectorFont, text: &str) -> Option<u32> {
            Some(text.chars().count() as u32 * 6)
        }

        fn animation_cel(&self, _image: SelectorImage, countdown: u16) -> Option<u32> {
            self.animations.then_some(u32::from(countdown % 3))
        }
    }

    #[derive(Default)]
    struct Random {
        characters: VecDeque<u32>,
        effects: u32,
    }

    impl SelectorRng for Random {
        fn character_u32(&mut self) -> u32 {
            self.characters.pop_front().unwrap_or(0)
        }
        fn effect_u32(&mut self) -> u32 {
            self.effects += 1;
            0
        }
    }

    struct Mask {
        available: bool,
        opaque: bool,
        reads: Cell<u32>,
    }

    impl Default for Mask {
        fn default() -> Self {
            Self {
                available: true,
                opaque: true,
                reads: Cell::new(0),
            }
        }
    }

    impl TrophyMask for Mask {
        fn available(&self, image: SelectorImage) -> bool {
            assert_eq!(image, SelectorImage::SilverTrophy);
            self.available
        }
        fn alpha(&self, image: SelectorImage, x: u32, y: u32) -> Option<u8> {
            assert_eq!(image, SelectorImage::SilverTrophy);
            assert!(x < 80 && y < 50);
            self.reads.set(self.reads.get() + 1);
            Some(if self.opaque { 255 } else { 0 })
        }
    }

    fn facts() -> SelectorFacts<'static> {
        SelectorFacts {
            profile: Some(ProfileFacts {
                id: 7,
                name: "Ada",
                tank: 1,
                level: 1,
                adventure_completed: false,
                returning_player: false,
                cyrax_defeats: 0,
                shells: 0,
                trophy: Trophy::None,
            }),
            modes: ModeAvailability {
                adventure: Availability::Available,
                time_trial: Availability::Locked(LockReason::CompleteAdventureTank),
                challenge: Availability::Locked(LockReason::CompleteAdventure),
                virtual_tank: Availability::Available,
                sandbox: Availability::Locked(LockReason::CompleteBonusPurchases),
            },
            modal_open: false,
            debug_keys_enabled: false,
            context_epoch: 1,
        }
    }

    fn ticks(
        selector: &mut GameSelector,
        facts: &SelectorFacts<'_>,
        resources: &Resources,
        random: &mut Random,
        mask: &Mask,
        count: u32,
    ) {
        for _ in 0..count {
            selector.update(facts, resources, random, mask);
        }
    }

    fn has_navigation(intentions: &[SelectorIntent]) -> bool {
        intentions
            .iter()
            .any(|intent| matches!(intent, SelectorIntent::Navigate(_)))
    }

    #[test]
    fn capture_release_revalidates_lock_profile_and_modal_context() {
        let resources = Resources::default();
        let mut facts = facts();
        let mut selector = GameSelector::new(&mut Random::default());
        let adventure = Point { x: 400, y: 80 };
        let trial = Point { x: 400, y: 160 };
        let pressed = selector.input(SelectorInput::PrimaryDown(adventure), &facts, &resources);
        assert_eq!(
            &pressed[..3],
            &[
                SelectorIntent::Pointer(PointerIntent::Capture(ButtonId::Adventure)),
                SelectorIntent::Button(ButtonIntent::Pressed(ButtonId::Adventure)),
                SelectorIntent::Sound(SelectorSound::ButtonClick),
            ]
        );
        selector.input(SelectorInput::PointerMove(trial), &facts, &resources);
        assert_eq!(
            selector.view(&facts, &resources).buttons[0].state,
            ButtonState::Normal
        );
        assert!(!has_navigation(&selector.input(
            SelectorInput::PrimaryUp(trial),
            &facts,
            &resources
        )));
        selector.input(SelectorInput::PrimaryDown(trial), &facts, &resources);
        let locked = selector.input(SelectorInput::PrimaryUp(trial), &facts, &resources);
        assert!(
            locked.contains(&SelectorIntent::Dialog(DialogIntent::LockedMode {
                mode: Mode::TimeTrial,
                reason: LockReason::CompleteAdventureTank
            }))
        );
        assert!(!has_navigation(&locked));
        assert!(
            selector
                .view(&facts, &resources)
                .buttons
                .iter()
                .find(|button| button.id == ButtonId::TimeTrial)
                .unwrap()
                .interactive
        );

        selector.input(SelectorInput::PrimaryDown(adventure), &facts, &resources);
        facts.modal_open = true;
        let canceled = selector.input(SelectorInput::PrimaryUp(adventure), &facts, &resources);
        assert!(
            canceled.contains(&SelectorIntent::Pointer(PointerIntent::CancelCapture(
                ButtonId::Adventure
            )))
        );
        assert!(!has_navigation(&canceled));
        facts.modal_open = false;
        selector.input(SelectorInput::PrimaryDown(adventure), &facts, &resources);
        facts.profile.as_mut().unwrap().id = 8;
        assert!(!has_navigation(&selector.input(
            SelectorInput::PrimaryUp(adventure),
            &facts,
            &resources
        )));
        selector.input(SelectorInput::PrimaryDown(adventure), &facts, &resources);
        facts.context_epoch += 1;
        assert!(!has_navigation(&selector.input(
            SelectorInput::PrimaryUp(adventure),
            &facts,
            &resources
        )));
        selector.input(SelectorInput::PrimaryDown(adventure), &facts, &resources);
        selector.input(SelectorInput::FocusLost, &facts, &resources);
        assert!(!has_navigation(&selector.input(
            SelectorInput::PrimaryUp(adventure),
            &facts,
            &resources
        )));

        selector.input(SelectorInput::PrimaryDown(adventure), &facts, &resources);
        facts.modes.adventure = Availability::Locked(LockReason::CallerBlocked);
        assert!(!has_navigation(&selector.input(
            SelectorInput::PrimaryUp(adventure),
            &facts,
            &resources
        )));
        facts.modes.adventure = Availability::Available;
        selector.input(SelectorInput::PrimaryDown(adventure), &facts, &resources);
        let allowed = selector.input(SelectorInput::PrimaryUp(adventure), &facts, &resources);
        assert!(
            allowed.contains(&SelectorIntent::Navigate(Destination::Mode(
                Mode::Adventure
            )))
        );
        assert!(!has_navigation(&selector.input(
            SelectorInput::PrimaryUp(adventure),
            &facts,
            &resources
        )));
    }

    #[test]
    fn source_hover_accumulates_six_divergent_updates_and_view_is_read_only() {
        let facts = facts();
        let resources = Resources::default();
        let mut random = Random::default();
        let mask = Mask::default();
        let mut selector = GameSelector::new(&mut random);
        selector.input(
            SelectorInput::PointerMove(Point { x: 400, y: 80 }),
            &facts,
            &resources,
        );
        ticks(&mut selector, &facts, &resources, &mut random, &mask, 5);
        assert_eq!(
            selector.view(&facts, &resources).hint.current,
            HintKey::Welcome
        );
        selector.input(
            SelectorInput::PointerMove(Point { x: 600, y: 400 }),
            &facts,
            &resources,
        );
        ticks(&mut selector, &facts, &resources, &mut random, &mask, 1);
        assert_eq!(selector.view(&facts, &resources).hint.divergent_updates, 5);
        selector.input(
            SelectorInput::PointerMove(Point { x: 400, y: 160 }),
            &facts,
            &resources,
        );
        ticks(&mut selector, &facts, &resources, &mut random, &mask, 1);
        let before = selector.clone();
        let scene = selector.view(&facts, &resources);
        assert_eq!(scene.hint.current, HintKey::TimeTrial);
        assert_eq!(scene.hint.fade_remaining, 4);
        assert!(scene.commands.iter().any(|draw| matches!(draw, DrawCommand::Text(draw) if draw.hint == Some(HintKey::TimeTrial) && draw.paint.alpha == 51)));
        assert!(scene.commands.iter().any(|draw| matches!(draw, DrawCommand::Text(draw) if draw.hint == Some(HintKey::Welcome) && draw.paint.alpha == 204)));
        assert!(
            scene
                .buttons
                .iter()
                .find(|button| button.id == ButtonId::ProfileLink)
                .unwrap()
                .bounds
                .is_none()
        );
        assert_eq!(scene, selector.view(&facts, &resources));
        assert_eq!(selector, before);
        assert_eq!(random.effects, 0);
        assert_eq!(mask.reads.get(), 0);
        assert!(
            scene
                .commands
                .windows(2)
                .all(|pair| pair[0].layer() <= pair[1].layer())
        );
        ticks(&mut selector, &facts, &resources, &mut random, &mask, 4);
        assert_eq!(selector.view(&facts, &resources).hint.fade_remaining, 0);
    }

    #[test]
    fn meryl_strict_click_bounds_and_source_countdown_reset_are_explicit() {
        let facts = facts();
        let mut resources = Resources::default();
        let mut random = Random::default();
        let mask = Mask::default();
        let mut selector = GameSelector::new(&mut random);
        assert_eq!(
            selector.view(&facts, &resources).meryl,
            MerylState {
                blink_countdown: 100,
                tail_countdown: 200
            }
        );
        selector.input(
            SelectorInput::PrimaryDown(Point { x: 19, y: 150 }),
            &facts,
            &resources,
        );
        assert_eq!(selector.view(&facts, &resources).meryl.tail_countdown, 200);
        selector.input(
            SelectorInput::PrimaryDown(Point { x: 20, y: 150 }),
            &facts,
            &resources,
        );
        assert_eq!(selector.view(&facts, &resources).meryl.tail_countdown, 15);
        ticks(&mut selector, &facts, &resources, &mut random, &mask, 14);
        assert_eq!(selector.view(&facts, &resources).meryl.tail_countdown, 1);
        ticks(&mut selector, &facts, &resources, &mut random, &mask, 1);
        assert_eq!(selector.view(&facts, &resources).meryl.tail_countdown, 200);
        random.characters.extend([0, 14]);
        ticks(&mut selector, &facts, &resources, &mut random, &mask, 85);
        assert_eq!(selector.view(&facts, &resources).meryl.blink_countdown, 29);
        resources.animations = false;
        resources.missing_image = Some(SelectorImage::Options);
        let scene = selector.view(&facts, &resources);
        assert!(
            scene
                .unavailable
                .contains(&Unavailable::Animation(SelectorImage::MerylBlink))
        );
        assert!(
            scene
                .unavailable
                .contains(&Unavailable::Animation(SelectorImage::MerylTail))
        );
        assert!(
            scene
                .unavailable
                .contains(&Unavailable::Image(SelectorImage::Options))
        );
        assert!(
            selector
                .button_bounds(ButtonId::Options, &facts, &resources)
                .is_none()
        );
        assert!(!scene.commands.iter().any(|draw| matches!(draw, DrawCommand::Image(draw) if draw.image == SelectorImage::MerylBlink || draw.image == SelectorImage::MerylTail)));
    }

    #[test]
    fn gold_sparkle_uses_silver_mask_and_bounded_1_to_59_lifecycle() {
        let mut facts = facts();
        facts.profile.as_mut().unwrap().trophy = Trophy::Gold;
        let resources = Resources::default();
        let mut random = Random::default();
        let mask = Mask::default();
        let mut selector = GameSelector::new(&mut random);
        ticks(&mut selector, &facts, &resources, &mut random, &mask, 40);
        assert_eq!(mask.reads.get(), 0);
        ticks(&mut selector, &facts, &resources, &mut random, &mask, 1);
        let scene = selector.view(&facts, &resources);
        assert_eq!(scene.sparkle.clock, 1);
        assert_eq!(scene.sparkle.position, Some(Point { x: 162, y: 302 }));
        assert_eq!(mask.reads.get(), 1);
        assert_eq!(random.effects, 2);
        assert!(scene.commands.iter().any(|draw| matches!(draw, DrawCommand::Image(draw) if draw.image == SelectorImage::Sparkle && draw.frame == ImageFrame::Cel(0) && draw.paint.blend == Blend::Additive)));
        ticks(&mut selector, &facts, &resources, &mut random, &mask, 10);
        let scene = selector.view(&facts, &resources);
        assert_eq!(scene.sparkle.clock, 11);
        assert!(!scene.commands.iter().any(
            |draw| matches!(draw, DrawCommand::Image(draw) if draw.image == SelectorImage::Sparkle)
        ));
        ticks(&mut selector, &facts, &resources, &mut random, &mask, 48);
        assert_eq!(selector.view(&facts, &resources).sparkle.clock, 59);
        ticks(&mut selector, &facts, &resources, &mut random, &mask, 1);
        assert_eq!(selector.view(&facts, &resources).sparkle.clock, 0);

        let transparent = Mask {
            opaque: false,
            ..Mask::default()
        };
        ticks(
            &mut selector,
            &facts,
            &resources,
            &mut random,
            &transparent,
            1,
        );
        let scene = selector.view(&facts, &resources);
        assert_eq!(transparent.reads.get(), 100);
        assert_eq!(random.effects, 202);
        assert_eq!(scene.sparkle.position, None);
        assert!(scene.unavailable.contains(&Unavailable::OpaqueTrophyPoint));
        // An unavailable mask retires a nondrawing cycle without random draws.
        let missing = Mask {
            available: false,
            ..Mask::default()
        };
        let mut selector = GameSelector::new(&mut random);
        ticks(&mut selector, &facts, &resources, &mut random, &missing, 41);
        assert_eq!(random.effects, 202);
        assert_eq!(missing.reads.get(), 0);
        assert!(
            selector
                .view(&facts, &resources)
                .unavailable
                .contains(&Unavailable::Mask(SelectorImage::SilverTrophy))
        );
    }

    #[test]
    fn labels_stage_overlay_profile_link_and_component_states_match_contracts() {
        let mut facts = facts();
        let resources = Resources::default();
        let mut selector = GameSelector::new(&mut Random::default());
        assert_eq!(adventure_label(facts.profile.as_ref()), "Start Adventure");
        assert_eq!(stage_subtitle(&facts.profile.unwrap()), None);
        let profile = facts.profile.as_mut().unwrap();
        profile.tank = 2;
        profile.level = 6;
        profile.shells = 125;
        assert_eq!(adventure_label(Some(profile)), "Adventure");
        assert_eq!(stage_subtitle(profile).as_deref(), Some("Bonus Level 2"));
        profile.adventure_completed = true;
        assert_eq!(adventure_label(Some(profile)), "Bonus Adventure");
        profile.tank = 5;
        profile.level = 1;
        assert_eq!(stage_subtitle(profile).as_deref(), Some("FINAL BOSS"));
        profile.cyrax_defeats = 10;
        assert_eq!(stage_subtitle(profile).as_deref(), Some("11th FINAL BOSS"));
        profile.cyrax_defeats = 20;
        assert_eq!(stage_subtitle(profile).as_deref(), Some("21st FINAL BOSS"));
        let scene = selector.view(&facts, &resources);
        assert!(scene.commands.iter().any(|draw| matches!(draw, DrawCommand::Text(draw) if draw.text == "125 Shells" && draw.layout == TextLayout::Baseline { anchor: Point { x: 465, y: 346 }, align: HorizontalAlign::Center })));
        assert_eq!(
            selector.button_bounds(ButtonId::Options, &facts, &resources),
            Some(PixelRect {
                x: 325,
                y: 412,
                width: 92,
                height: 24
            })
        );
        assert_eq!(
            selector.button_bounds(ButtonId::Challenge, &facts, &resources),
            Some(PixelRect {
                x: 359,
                y: 212,
                width: 213,
                height: 48
            })
        );
        let options = Point { x: 340, y: 420 };
        selector.input(SelectorInput::PointerMove(options), &facts, &resources);
        assert!(selector.view(&facts, &resources).commands.iter().any(|draw| matches!(draw, DrawCommand::Image(draw) if draw.image == SelectorImage::Options && draw.source.x == 30 && draw.source.width == 30 && draw.sizing == ImageSizing::ImageBox)));
        selector.input(SelectorInput::PrimaryDown(options), &facts, &resources);
        assert!(selector.view(&facts, &resources).commands.iter().any(|draw| matches!(draw, DrawCommand::Image(draw) if draw.image == SelectorImage::Options && draw.source.x == 60)));
        assert!(
            selector
                .input(SelectorInput::PrimaryUp(options), &facts, &resources)
                .contains(&SelectorIntent::Dialog(DialogIntent::Options))
        );
        let link = Point { x: 100, y: 95 };
        selector.input(SelectorInput::PrimaryDown(link), &facts, &resources);
        assert!(
            selector
                .input(SelectorInput::PrimaryUp(link), &facts, &resources)
                .contains(&SelectorIntent::Dialog(DialogIntent::ChooseProfile))
        );
        facts.profile = None;
        assert!(
            selector
                .button_bounds(ButtonId::ProfileLink, &facts, &resources)
                .is_none()
        );
    }

    fn characters(
        selector: &mut GameSelector,
        facts: &SelectorFacts<'_>,
        resources: &Resources,
        text: &str,
    ) -> Vec<SelectorIntent> {
        text.chars()
            .flat_map(|character| {
                selector.input(SelectorInput::Character(character), facts, resources)
            })
            .collect()
    }

    #[test]
    fn give_is_lowercase_gated_and_sandbox_admission_stays_caller_owned() {
        let mut facts = facts();
        let resources = Resources::default();
        let mut selector = GameSelector::new(&mut Random::default());
        let give = SelectorIntent::Dialog(DialogIntent::OpenGiveDialog);
        assert!(!characters(&mut selector, &facts, &resources, "GIVE").contains(&give));
        assert!(!characters(&mut selector, &facts, &resources, "ggive").contains(&give));
        characters(&mut selector, &facts, &resources, "gi");
        selector.input(SelectorInput::NonCharacterKey, &facts, &resources);
        assert!(characters(&mut selector, &facts, &resources, "ve").contains(&give));
        characters(&mut selector, &facts, &resources, "gi");
        facts.debug_keys_enabled = true;
        assert!(!characters(&mut selector, &facts, &resources, "ve").contains(&give));
        assert_eq!(selector.sandbox_activated(&facts), None);
        facts.debug_keys_enabled = false;
        assert!(!characters(&mut selector, &facts, &resources, "ve").contains(&give));
        characters(&mut selector, &facts, &resources, "gi");
        facts.modal_open = true;
        assert!(!characters(&mut selector, &facts, &resources, "ve").contains(&give));
        facts.modal_open = false;
        assert!(!characters(&mut selector, &facts, &resources, "ve").contains(&give));
        characters(&mut selector, &facts, &resources, "gi");
        selector.input(SelectorInput::FocusLost, &facts, &resources);
        assert!(!characters(&mut selector, &facts, &resources, "ve").contains(&give));
        characters(&mut selector, &facts, &resources, "gi");
        facts.context_epoch += 1;
        assert!(!characters(&mut selector, &facts, &resources, "ve").contains(&give));
        let before = facts;
        let intentions = characters(&mut selector, &facts, &resources, "give");
        assert_eq!(
            intentions.iter().filter(|&&intent| intent == give).count(),
            1
        );
        assert!(!has_navigation(&intentions));
        assert_eq!(facts, before);
        assert_eq!(
            selector.sandbox_activated(&facts),
            Some(SelectorIntent::Dialog(DialogIntent::LockedMode {
                mode: Mode::Sandbox,
                reason: LockReason::CompleteBonusPurchases
            }))
        );
        facts.modes.sandbox = Availability::Available;
        assert_eq!(
            selector.sandbox_activated(&facts),
            Some(SelectorIntent::Navigate(Destination::Mode(Mode::Sandbox)))
        );
        facts.profile = None;
        assert_eq!(
            selector.sandbox_activated(&facts),
            Some(SelectorIntent::Dialog(DialogIntent::LockedMode {
                mode: Mode::Sandbox,
                reason: LockReason::NoProfile
            }))
        );
    }
}
