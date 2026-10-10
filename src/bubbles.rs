//! Transient bubble particles and decorative fish, with no save or RNG authority.
//!
//! Rules are secondary-source-derived from pinned W1 `BubbleMgr.{cpp,h}`
//! (`f919b3c241cfd611c547f1653fb3514f500f761b`); see `docs/bubbles.md`.
//! Each RNG call must return a value in `0..upper` and consume one caller draw.
//! Call `update` once per caller-owned tick; pause and scene lifetime stay there.

const BUBBLE_SPEEDS: [f32; 6] = [2.0, 2.5, 3.0, 3.2, 2.3, 2.8];
const FISH_SPEEDS: [f32; 6] = [1.3, 1.7, 2.0, 2.5, 3.0, 3.5];
// Preserve the source's approximation rather than substituting Rust's PI.
const BOB_PI: f64 = f64::from_bits(0x4009_21fa_0000_0000);

// Paired RGB descriptors after the source's +5, saturated-at-255 operation.
// These numeric appearance facts are not upstream source/asset data imports.
const FISH_PALETTES: [[[u8; 3]; 2]; 30] = [
    [[130, 215, 255], [55, 115, 215]],
    [[255, 245, 65], [50, 100, 200]],
    [[140, 255, 250], [255, 5, 130]],
    [[180, 240, 140], [55, 120, 215]],
    [[200, 145, 95], [70, 125, 70]],
    [[255, 255, 90], [255, 160, 5]],
    [[255, 255, 255], [235, 135, 255]],
    [[65, 65, 65], [5, 5, 5]],
    [[255, 70, 70], [80, 80, 80]],
    [[195, 65, 255], [255, 5, 130]],
    [[195, 195, 195], [255, 255, 255]],
    [[150, 150, 180], [35, 85, 230]],
    [[245, 150, 240], [115, 15, 230]],
    [[255, 255, 45], [215, 15, 5]],
    [[215, 220, 180], [115, 100, 215]],
    [[180, 245, 180], [35, 85, 130]],
    [[255, 185, 115], [215, 15, 5]],
    [[70, 75, 165], [245, 245, 245]],
    [[245, 245, 245], [5, 10, 135]],
    [[245, 230, 130], [245, 10, 245]],
    [[255, 235, 5], [5, 130, 5]],
    [[255, 45, 5], [255, 230, 5]],
    [[180, 220, 115], [20, 35, 20]],
    [[230, 180, 100], [70, 60, 195]],
    [[135, 170, 150], [130, 20, 20]],
    [[165, 235, 55], [150, 5, 215]],
    [[210, 55, 185], [255, 150, 225]],
    [[195, 145, 255], [255, 250, 180]],
    [[245, 255, 245], [85, 255, 150]],
    [[215, 255, 55], [115, 215, 5]],
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BubbleRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl BubbleRect {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BubbleConfig {
    pub bubble_bounds: BubbleRect,
    pub fish_bounds: BubbleRect,
    /// Automatic-spawn limits. Explicit spawns have their own admission policy.
    pub max_bubbles: usize,
    pub bubble_spawn_chance: i32,
    pub max_fish: usize,
    pub fish_spawn_chance: i32,
    pub default_fish_vy: f32,
    pub clip_fish: bool,
}

impl Default for BubbleConfig {
    fn default() -> Self {
        Self {
            bubble_bounds: BubbleRect::new(0, 0, 640, 480),
            fish_bounds: BubbleRect::new(0, 0, 640, 480),
            max_bubbles: 10,
            bubble_spawn_chance: 10,
            max_fish: 0,
            fish_spawn_chance: 10,
            default_fish_vy: 0.0,
            clip_fish: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BubbleTint {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl BubbleTint {
    const fn from_rgb([red, green, blue]: [u8; 3]) -> Self {
        Self { red, green, blue }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BubbleSheet {
    Bubbles,
    SwimBase,
    SwimColor2,
    SwimColor1,
    TurnBase,
    TurnColor2,
    TurnColor1,
}

impl BubbleSheet {
    pub const fn resource_key(self) -> &'static str {
        match self {
            Self::Bubbles => "IMAGE_BUBBLES",
            Self::SwimBase => "IMAGE_SIMSWIM1",
            Self::SwimColor2 => "IMAGE_SIMSWIM2",
            Self::SwimColor1 => "IMAGE_SIMSWIM3",
            Self::TurnBase => "IMAGE_SIMTURN1",
            Self::TurnColor2 => "IMAGE_SIMTURN2",
            Self::TurnColor1 => "IMAGE_SIMTURN3",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BubbleBlend {
    Normal,
    Additive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BubbleSpriteSource {
    /// Resolve through the owned image's cel metadata (four bubble columns).
    Cel(u8),
    Rect(BubbleRect),
}

/// Operations are yielded in paint order: all bubbles, then each fish's
/// base/color2/color1 layers. Scene depth remains the caller's decision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BubbleRenderOp {
    pub sheet: BubbleSheet,
    pub source: BubbleSpriteSource,
    pub x: f32,
    pub y: f32,
    pub mirror_x: bool,
    /// `None` means the uncolored source image, not a retained graphics tint.
    pub tint: Option<BubbleTint>,
    pub blend: BubbleBlend,
    /// Intersect with caller clipping; `None` preserves caller clipping alone.
    pub clip: Option<BubbleRect>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bubble {
    pub kind: u8,
    pub shake_offset: u8,
    pub x: f32,
    pub y: f32,
    pub vy: f32,
}

impl Bubble {
    pub fn new(rand_range: &mut impl FnMut(u64) -> u64) -> Self {
        let mut kind = rand_range(4) as u8;
        if kind == 3 {
            kind = rand_range(4) as u8;
        }
        let y = 400.0 + rand_range(6) as f32;
        let x = 150.0 + rand_range(22) as f32;
        let vy = BUBBLE_SPEEDS[rand_range(6) as usize];
        Self {
            kind,
            shake_offset: 0,
            x,
            y,
            vy,
        }
    }

    pub fn update(&mut self, rand_range: &mut impl FnMut(u64) -> u64) {
        self.shake_offset = rand_range(2) as u8;
        self.y -= self.vy;
    }

    pub fn render_op(&self) -> BubbleRenderOp {
        BubbleRenderOp {
            sheet: BubbleSheet::Bubbles,
            source: BubbleSpriteSource::Cel(self.kind),
            x: self.x + f32::from(self.shake_offset),
            y: self.y,
            mirror_x: false,
            tint: None,
            blend: BubbleBlend::Additive,
            // W1 clips a graphics copy, but passes the original to Bubble::Draw.
            clip: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BubbleFish {
    pub size: u8,
    pub update_count: u32,
    pub turn_timer: u32,
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub color1: BubbleTint,
    pub color2: BubbleTint,
}

impl BubbleFish {
    pub fn new(rand_range: &mut impl FnMut(u64) -> u64) -> Self {
        let vx = FISH_SPEEDS[rand_range(6) as usize];
        let size_roll = rand_range(10);
        let size = match size_roll {
            0..=4 => 0,
            5..=7 => 1,
            _ => 2,
        };
        let [color1, color2] = FISH_PALETTES[rand_range(30) as usize];
        Self {
            size,
            update_count: 0,
            turn_timer: 0,
            x: 0.0,
            y: 0.0,
            vx,
            vy: 0.0,
            color1: BubbleTint::from_rgb(color1),
            color2: BubbleTint::from_rgb(color2),
        }
    }

    pub fn update(&mut self) {
        self.update_count = self.update_count.wrapping_add(1);
        self.x += self.vx;
        self.y += self.vy;
        self.turn_timer = self.turn_timer.saturating_sub(1);
    }

    pub fn render_ops(&self, clip: Option<BubbleRect>) -> [BubbleRenderOp; 3] {
        let turning = (1..=5).contains(&self.turn_timer);
        let column = if turning {
            ((5 - self.turn_timer) * 160) as i32
        } else {
            ((self.update_count / 2 % 10) * 80) as i32
        };
        let source =
            BubbleSpriteSource::Rect(BubbleRect::new(column, i32::from(self.size) * 80, 80, 80));
        let angle = (f64::from(self.update_count) * 5.0 * BOB_PI / 180.0) as f32;
        let y = self.y + angle.sin() * 3.0;
        let mirror_x = (self.vx > 0.0) ^ turning;
        let sheets = if turning {
            [
                BubbleSheet::TurnBase,
                BubbleSheet::TurnColor2,
                BubbleSheet::TurnColor1,
            ]
        } else {
            [
                BubbleSheet::SwimBase,
                BubbleSheet::SwimColor2,
                BubbleSheet::SwimColor1,
            ]
        };
        let base = BubbleRenderOp {
            sheet: sheets[0],
            source,
            x: self.x,
            y,
            mirror_x,
            tint: None,
            blend: BubbleBlend::Normal,
            clip,
        };
        [
            base,
            BubbleRenderOp {
                sheet: sheets[1],
                tint: Some(self.color2),
                blend: BubbleBlend::Additive,
                ..base
            },
            BubbleRenderOp {
                sheet: sheets[2],
                tint: Some(self.color1),
                blend: BubbleBlend::Additive,
                ..base
            },
        ]
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BubbleManager {
    config: BubbleConfig,
    bubbles: Vec<Bubble>,
    fish: Vec<BubbleFish>,
}

impl BubbleManager {
    pub fn new(config: BubbleConfig) -> Self {
        Self {
            config,
            bubbles: Vec::new(),
            fish: Vec::new(),
        }
    }

    pub fn config(&self) -> &BubbleConfig {
        &self.config
    }

    pub fn bubbles(&self) -> &[Bubble] {
        &self.bubbles
    }

    pub fn fish(&self) -> &[BubbleFish] {
        &self.fish
    }

    /// Caller-controlled turn/motion changes use the same decorative fish state.
    pub fn fish_mut(&mut self) -> &mut [BubbleFish] {
        &mut self.fish
    }

    pub fn set_bubble_bounds(&mut self, bounds: BubbleRect) {
        self.config.bubble_bounds = bounds;
    }

    pub fn set_fish_bounds(&mut self, bounds: BubbleRect) {
        self.config.fish_bounds = bounds;
    }

    /// Chance is compared directly with a roll in 0..100: <=0 never spawns,
    /// >=100 always attempts. Available capacity still consumes the chance roll.
    pub fn set_bubble_config(&mut self, max_bubbles: usize, spawn_chance: i32) {
        self.config.max_bubbles = max_bubbles;
        self.config.bubble_spawn_chance = spawn_chance;
    }

    pub fn set_fish_config(&mut self, max_fish: usize, spawn_chance: i32) {
        self.config.max_fish = max_fish;
        self.config.fish_spawn_chance = spawn_chance;
    }

    pub fn set_clip_fish(&mut self, clip: bool) {
        self.config.clip_fish = clip;
    }

    pub fn set_default_fish_vy(&mut self, vy: f32) {
        if vy == self.config.default_fish_vy {
            return;
        }
        self.config.default_fish_vy = vy;
        for fish in &mut self.fish {
            fish.vy = vy;
        }
    }

    /// Manager explicit spawn is deliberately independent of its automatic cap.
    pub fn spawn_bubble(&mut self, x: i32, y: i32, rand_range: &mut impl FnMut(u64) -> u64) {
        let mut bubble = Bubble::new(rand_range);
        // Preserve constructor draws even though the caller overrides position.
        bubble.x = x as f32;
        bubble.y = y as f32;
        self.bubbles.push(bubble);
    }

    /// Board::SpawnBubble uses cap30; Board::SpawnRandomBubble uses cap50.
    /// Rejection happens before particle-constructor RNG consumption.
    pub fn spawn_bubble_capped(
        &mut self,
        x: i32,
        y: i32,
        cap: usize,
        rand_range: &mut impl FnMut(u64) -> u64,
    ) -> bool {
        if self.bubbles.len() >= cap {
            return false;
        }
        self.spawn_bubble(x, y, rand_range);
        true
    }

    /// Direct random spawning, like W1, does not apply the automatic cap.
    /// Returns false without consuming placement/construction RNG for width<=20.
    pub fn spawn_random_bubble(&mut self, rand_range: &mut impl FnMut(u64) -> u64) -> bool {
        let bounds = self.config.bubble_bounds;
        if bounds.width <= 20 {
            return false;
        }
        let x_offset = rand_range((bounds.width - 20) as u64);
        let mut bubble = Bubble::new(rand_range);
        bubble.x = bounds.x as f32 + x_offset as f32;
        bubble.y = bounds.y as f32 + bounds.height as f32;
        self.bubbles.push(bubble);
        true
    }

    /// Returns false without construction RNG for height<=80. Side selection
    /// follows constructor, Y placement and default vertical-speed assignment.
    pub fn spawn_random_bubble_fish(&mut self, rand_range: &mut impl FnMut(u64) -> u64) -> bool {
        let bounds = self.config.fish_bounds;
        if bounds.height <= 80 {
            return false;
        }
        let mut fish = BubbleFish::new(rand_range);
        fish.y = bounds.y as f32 + rand_range((bounds.height - 80) as u64) as f32;
        fish.vy = self.config.default_fish_vy;
        if rand_range(2) == 0 {
            fish.x = bounds.x as f32 - 80.0;
        } else {
            fish.x = bounds.x as f32 + bounds.width as f32;
            fish.vx = -fish.vx;
        }
        self.fish.push(fish);
        true
    }

    pub fn update(&mut self, rand_range: &mut impl FnMut(u64) -> u64) {
        let top = self.config.bubble_bounds.y as f32;
        self.bubbles.retain_mut(|bubble| {
            bubble.update(rand_range);
            // Preserve the strict source predicate, including equality at top.
            bubble.y.partial_cmp(&top) != Some(std::cmp::Ordering::Less)
        });
        if self.bubbles.len() < self.config.max_bubbles
            && (rand_range(100) as i32) < self.config.bubble_spawn_chance
        {
            self.spawn_random_bubble(rand_range);
        }

        let left = self.config.fish_bounds.x as f32 - 80.0;
        let right = self.config.fish_bounds.x as f32 + self.config.fish_bounds.width as f32;
        self.fish.retain_mut(|fish| {
            fish.update();
            !(fish.vx == 0.0 || fish.x < left || fish.x > right)
        });
        if self.fish.len() < self.config.max_fish
            && (rand_range(100) as i32) < self.config.fish_spawn_chance
        {
            self.spawn_random_bubble_fish(rand_range);
        }
    }

    /// Warm initial population with exactly500 regular ticks, including RNG,
    /// admission, movement and removal; this does not advance caller game time.
    pub fn update_a_lot(&mut self, rand_range: &mut impl FnMut(u64) -> u64) {
        for _ in 0..500 {
            self.update(rand_range);
        }
    }

    pub fn clear(&mut self) {
        self.bubbles.clear();
        self.fish.clear();
    }

    /// Callers apply each operation's blend/tint/clip and restore their graphics
    /// state afterward. Iteration and drawing consume no random values.
    pub fn render_ops(&self) -> impl Iterator<Item = BubbleRenderOp> + '_ {
        let clip = self.config.clip_fish.then_some(self.config.fish_bounds);
        self.bubbles
            .iter()
            .map(Bubble::render_op)
            .chain(self.fish.iter().flat_map(move |fish| fish.render_ops(clip)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DrawScript<'a> {
        draws: &'a [(u64, u64)],
        cursor: usize,
    }

    impl DrawScript<'_> {
        fn next(&mut self, upper: u64) -> u64 {
            let (expected_upper, result) = self.draws[self.cursor];
            assert_eq!(upper, expected_upper, "draw {}", self.cursor);
            assert!(result < upper);
            self.cursor += 1;
            result
        }
    }

    impl Drop for DrawScript<'_> {
        fn drop(&mut self) {
            if !std::thread::panicking() {
                assert_eq!(self.cursor, self.draws.len(), "unused expected RNG draws");
            }
        }
    }

    fn scripted_rng(draws: &[(u64, u64)]) -> impl FnMut(u64) -> u64 + '_ {
        let mut script = DrawScript { draws, cursor: 0 };
        move |upper| script.next(upper)
    }

    fn no_rng(_: u64) -> u64 {
        panic!("this path must not draw RNG")
    }

    fn still_fish() -> BubbleFish {
        BubbleFish::new(&mut scripted_rng(&[(6, 0), (10, 0), (30, 0)]))
    }

    fn disabled_manager() -> BubbleManager {
        BubbleManager::new(BubbleConfig {
            max_bubbles: 0,
            max_fish: 0,
            ..BubbleConfig::default()
        })
    }

    // B02/B03: first type3 triggers exactly one reroll; overwritten initial
    // positions still consume their independent constructor draws.
    #[test]
    fn explicit_spawn_keeps_conditional_rng_and_separate_cap() {
        let draws = [(4, 3), (4, 3), (6, 5), (22, 21), (6, 3)];
        let initial = Bubble::new(&mut scripted_rng(&draws));
        assert_eq!((initial.x, initial.y), (171.0, 405.0));
        assert_eq!((initial.kind, initial.vy), (3, 3.2));

        let mut manager = disabled_manager();
        assert!(manager.spawn_bubble_capped(50, 300, 1, &mut scripted_rng(&draws)));
        let bubble = &manager.bubbles()[0];
        assert_eq!(
            (bubble.x, bubble.y, bubble.kind, bubble.vy),
            (50.0, 300.0, 3, 3.2)
        );
        assert!(!manager.spawn_bubble_capped(0, 0, 1, &mut no_rng));
        assert_eq!(manager.bubbles().len(), 1);
    }

    // B02: width21 gives one possible X; <=20 and fish height<=80 reject
    // before construction. Direct random calls ignore automatic admission caps.
    #[test]
    fn random_spawn_respects_geometry_and_placement_rng_order() {
        let mut manager = disabled_manager();
        manager.set_bubble_bounds(BubbleRect::new(100, 200, 21, 300));
        assert!(manager.spawn_random_bubble(&mut scripted_rng(&[
            (1, 0),
            (4, 2),
            (6, 0),
            (22, 0),
            (6, 5),
        ])));
        let bubble = &manager.bubbles()[0];
        assert_eq!((bubble.x, bubble.y, bubble.vy), (100.0, 500.0, 2.8));
        manager.set_bubble_bounds(BubbleRect::new(0, 0, 20, 300));
        assert!(!manager.spawn_random_bubble(&mut no_rng));
        manager.set_fish_bounds(BubbleRect::new(0, 0, 640, 80));
        assert!(!manager.spawn_random_bubble_fish(&mut no_rng));
    }

    // B01/B02/B04: old effects retire before their respective cap check; a
    // spawned bubble precedes fish RNG and neither new object moves this tick.
    #[test]
    fn ordered_update_removes_then_spawns_without_advancing_new_effects() {
        let mut manager = BubbleManager::new(BubbleConfig {
            bubble_bounds: BubbleRect::new(0, 10, 21, 100),
            fish_bounds: BubbleRect::new(0, 0, 100, 81),
            max_bubbles: 1,
            bubble_spawn_chance: 100,
            max_fish: 1,
            fish_spawn_chance: 100,
            default_fish_vy: -1.0,
            ..BubbleConfig::default()
        });
        manager.bubbles.push(Bubble {
            kind: 0,
            shake_offset: 0,
            x: 3.0,
            y: 10.0,
            vy: 2.0,
        });
        let mut old_fish = still_fish();
        old_fish.x = 100.0;
        manager.fish.push(old_fish);
        manager.update(&mut scripted_rng(&[
            (2, 1),
            (100, 0),
            (1, 0),
            (4, 0),
            (6, 0),
            (22, 0),
            (6, 1),
            (100, 0),
            (6, 0),
            (10, 8),
            (30, 0),
            (1, 0),
            (2, 1),
        ]));
        assert_eq!(manager.bubbles().len(), 1);
        assert_eq!(
            (manager.bubbles()[0].y, manager.bubbles()[0].shake_offset),
            (110.0, 0)
        );
        let fish = &manager.fish()[0];
        assert_eq!((fish.x, fish.y, fish.vx, fish.vy), (100.0, 0.0, -1.3, -1.0));
        assert_eq!((fish.size, fish.update_count), (2, 0));
        assert_eq!(fish.color1, BubbleTint::from_rgb([130, 215, 255]));
        assert_eq!(fish.color2, BubbleTint::from_rgb([55, 115, 215]));
    }

    // B01: disabled caps skip chance RNG; available capacity consumes rolls
    // even for a zero/negative chance. Narrow bounds still consume chance RNG.
    #[test]
    fn conditional_chance_rng_survives_disabled_and_narrow_configs() {
        let mut manager = disabled_manager();
        manager.update(&mut no_rng);
        manager.set_bubble_config(1, 0);
        manager.set_fish_config(1, -1);
        manager.update(&mut scripted_rng(&[(100, 0), (100, 0)]));
        assert!(manager.bubbles().is_empty() && manager.fish().is_empty());
        manager.set_bubble_bounds(BubbleRect::new(0, 0, 20, 480));
        manager.set_fish_bounds(BubbleRect::new(0, 0, 640, 80));
        manager.set_bubble_config(1, 101);
        manager.set_fish_config(1, 100);
        manager.update(&mut scripted_rng(&[(100, 99), (100, 99)]));
        assert!(manager.bubbles().is_empty() && manager.fish().is_empty());
        manager.spawn_bubble(
            0,
            200,
            &mut scripted_rng(&[(4, 0), (6, 0), (22, 0), (6, 0)]),
        );
        // Full bubble capacity skips its chance roll; fish retain theirs.
        manager.update(&mut scripted_rng(&[(2, 0), (100, 99)]));
        assert_eq!(manager.bubbles().len(), 1);
    }

    // B03/B04: equality survives, one pixel beyond does not; fish Y never
    // participates in retirement, and zero horizontal velocity always retires.
    #[test]
    fn lifetime_uses_strict_edges_and_no_fish_vertical_culling() {
        let mut manager = disabled_manager();
        manager.set_fish_bounds(BubbleRect::new(0, 0, 100, 480));
        for (x, vx, y) in [
            (-79.0, -1.0, -500.0),
            (101.0, -1.0, 900.0),
            (-80.0, -1.0, 0.0),
            (100.0, 1.0, 0.0),
            (50.0, 0.0, 0.0),
        ] {
            let mut fish = still_fish();
            fish.x = x;
            fish.y = y;
            fish.vx = vx;
            manager.fish.push(fish);
        }
        manager.bubbles.push(Bubble {
            kind: 0,
            shake_offset: 0,
            x: 1.0,
            y: 2.0,
            vy: 2.0,
        });
        manager.update(&mut scripted_rng(&[(2, 1)]));
        assert_eq!(manager.bubbles()[0].y, 0.0);
        assert_eq!(
            manager
                .fish()
                .iter()
                .map(|fish| (fish.x, fish.y))
                .collect::<Vec<_>>(),
            [(-80.0, -500.0), (100.0, 900.0)]
        );
        manager.update(&mut scripted_rng(&[(2, 0)]));
        assert!(manager.bubbles().is_empty());
    }

    // B05: swim18 is cel9/size2, bob near+3; turn1 is sourceX640 with
    // width80. Source paints base then color2/color1 and only clips fish.
    #[test]
    fn render_descriptors_preserve_layer_turn_tint_and_clip_contract() {
        let mut manager = disabled_manager();
        manager.bubbles.push(Bubble {
            kind: 2,
            shake_offset: 1,
            x: 10.0,
            y: 20.0,
            vy: 2.0,
        });
        let mut fish = BubbleFish::new(&mut scripted_rng(&[(6, 5), (10, 7), (30, 29)]));
        assert_eq!(fish.size, 1);
        assert_eq!(fish.vx, 3.5);
        assert_eq!(fish.color1, BubbleTint::from_rgb([215, 255, 55]));
        assert_eq!(fish.color2, BubbleTint::from_rgb([115, 215, 5]));
        fish.size = 2;
        fish.update_count = 18;
        fish.y = 20.0;
        manager.fish.push(fish);
        let ops: Vec<_> = manager.render_ops().collect();
        assert_eq!(ops.len(), 4);
        assert_eq!(
            (ops[0].sheet, ops[0].x, ops[0].clip),
            (BubbleSheet::Bubbles, 11.0, None)
        );
        assert_eq!(ops[0].source, BubbleSpriteSource::Cel(2));
        assert_eq!(
            ops[1].source,
            BubbleSpriteSource::Rect(BubbleRect::new(720, 160, 80, 80))
        );
        assert!((ops[1].y - 23.0).abs() < 0.00001);
        assert!(ops[1].mirror_x);
        assert_eq!(ops[1].blend, BubbleBlend::Normal);
        assert_eq!(ops[1].clip, Some(manager.config().fish_bounds));
        assert_eq!(ops[2].sheet.resource_key(), "IMAGE_SIMSWIM2");
        assert_eq!(ops[2].tint, Some(manager.fish()[0].color2));
        assert_eq!(ops[3].tint, Some(manager.fish()[0].color1));
        assert_eq!(ops[2].blend, BubbleBlend::Additive);
        manager.set_clip_fish(false);
        manager.fish_mut()[0].turn_timer = 1;
        let turn = manager.render_ops().nth(1).unwrap();
        assert_eq!(turn.sheet, BubbleSheet::TurnBase);
        assert_eq!(
            turn.source,
            BubbleSpriteSource::Rect(BubbleRect::new(640, 160, 80, 80))
        );
        assert!(!turn.mirror_x);
        assert_eq!(turn.clip, None);
        manager.fish_mut()[0].turn_timer = 6;
        assert_eq!(
            manager.render_ops().nth(1).unwrap().sheet,
            BubbleSheet::SwimBase
        );
    }

    // B06: equal-default setter leaves individually changed speed alone;
    // changed-default setter retimes existing fish without resetting clocks.
    #[test]
    fn vertical_speed_setter_retimes_existing_fish_only_when_changed() {
        let mut manager = disabled_manager();
        let mut fish = still_fish();
        fish.vy = 7.0;
        fish.update_count = 9;
        manager.fish.push(fish);
        manager.set_default_fish_vy(0.0);
        assert_eq!(manager.fish()[0].vy, 7.0);
        manager.set_default_fish_vy(-1.0);
        assert_eq!(
            (manager.fish()[0].vy, manager.fish()[0].update_count),
            (-1.0, 9)
        );
        manager.set_bubble_config(0, 0);
        manager.set_fish_config(0, 0);
        assert_eq!(manager.fish().len(), 1);
    }

    // B06: an existing bubble atY100/speed2 survives50 updates at top0 and
    // retires on51. Existing fish get all500 updates and remain despite Y<0.
    #[test]
    fn initial_population_helper_runs_500_real_updates() {
        let mut manager = disabled_manager();
        manager.bubbles.push(Bubble {
            kind: 0,
            shake_offset: 0,
            x: 0.0,
            y: 100.0,
            vy: 2.0,
        });
        let mut fish = still_fish();
        fish.x = -80.0;
        fish.y = 15.0;
        fish.vx = 1.0;
        fish.vy = -1.0;
        fish.turn_timer = 7;
        manager.fish.push(fish);
        let mut draw_count = 0;
        manager.update_a_lot(&mut |upper| {
            assert_eq!(upper, 2);
            draw_count += 1;
            0
        });
        assert_eq!(draw_count, 51);
        assert!(manager.bubbles().is_empty());
        let fish = &manager.fish()[0];
        assert_eq!(
            (fish.x, fish.y, fish.update_count, fish.turn_timer),
            (420.0, -485.0, 500, 0)
        );
        manager.clear();
        assert!(manager.bubbles().is_empty() && manager.fish().is_empty());
    }
}
