//! Ordinary Sylvester actor for Adventure 1-2 and 1-3.
//!
//! The board owns spawn timing, its random stream, ordered prey membership,
//! coin creation and removal. This actor owns only its live motion, contact,
//! shooting and animation state. Constructor and update rules are secondary
//! source-derived from WinFish f919b3c (`Alien.cpp`). The installed PB05
//! payload confirms the class and weak/strong HP and divisor (PB17), and
//! shot damage via Board weapon (PB13); update order remains source-derived.
//! No original-game run or retail parity measurement has been made.

use serde::{Deserialize, Serialize};

pub const WEAK_SYLVESTER_SIZE: i32 = 160;
const WEAK_SPEED_DIVISOR: f64 = 2.0;
const WEAK_STARTING_HEALTH: f64 = 50.0;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SylvesterKind {
    #[default]
    Weak,
    Strong,
    Balrog,
    Gus,
    Destructor,
    Psychosquid,
    Ulysses,
}

impl SylvesterKind {
    pub fn speed_divisor(self) -> f64 {
        match self {
            Self::Weak => WEAK_SPEED_DIVISOR,
            Self::Strong => 1.6,
            Self::Balrog => 1.2,
            Self::Gus => 1.6,
            Self::Destructor => 1.2,
            Self::Psychosquid => 0.5,
            Self::Ulysses => 3.5,
        }
    }

    pub fn starting_health(self) -> f64 {
        match self {
            Self::Weak => WEAK_STARTING_HEALTH,
            Self::Strong => 60.0,
            Self::Balrog => 130.0,
            Self::Gus => 100.0,
            Self::Destructor => 150.0,
            Self::Psychosquid => 260.0,
            Self::Ulysses => 220.0,
        }
    }
}

/// A board-ordered snapshot of a possible alien target. The board decides
/// whether a fish is eligible (including its protection and newborn delay).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PreyView {
    pub id: u64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub width: i32,
    pub height: i32,
    pub eligible: bool,
}

/// Current food-list order, captured after food and fish update but before
/// the alien's object update. `eligible` includes the food's eating delay.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AlienFoodView {
    pub id: u64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub quality: u8,
    pub eligible: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AlienUpdate {
    pub food_eaten: Option<(u64, u8)>,
    /// At most one prey, selected in the board-provided order.
    pub prey_eaten: Option<u64>,
    /// Registered zero-health aliens finish this active update before removal.
    pub defeated: bool,
    pub phase_changed: Option<bool>,
}

/// A Board callback executes the launch before the actor requests its reload
/// draw. `Launch` returns one on successful registration, zero otherwise.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlienRuntimeRequest {
    Random,
    /// Target search without construction at the prelaunch cue.
    ProbeTarget {
        center_x: i32,
        center_y: i32,
        excluded_prey: Option<u64>,
    },
    Launch {
        slot: u8,
        x: i32,
        y: i32,
        center_x: i32,
        center_y: i32,
        excluded_prey: Option<u64>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ShotResult {
    Miss,
    Hit {
        health: f64,
    },
    /// The board creates the single diamond and removes the live alien.
    Defeated {
        diamond_x: i32,
        diamond_y: i32,
    },
}

/// Integer widget coordinates are distinct from double motion coordinates:
/// animation can move the latter after the widget is positioned for a tick.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeakSylvester {
    pub id: u64,
    #[serde(default)] // Only pre-format-5 project saves omit the variant.
    pub kind: SylvesterKind,
    pub x: f64,
    pub y: f64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub vx: f64,
    pub vy: f64,
    pub target_vx: f64,
    pub target_vy: f64,
    pub previous_vx: f64,
    pub health: f64,
    pub spawn_ticks: u8,
    pub chase_ticks: u8,
    pub hit_ticks: u8,
    pub movement_state: u8,
    pub movement_change_ticks: u8,
    pub swim_ticks: u8,
    pub turn_ticks: i8,
    pub frame: u8,
    pub alive: bool,
    pub launch_ticks: u16,
    pub reload_ticks: u16,
    pub special_ticks: u8,
    pub phase_ticks: u16,
    pub phase_threshold: u16,
    pub healing: bool,
    pub ever_healed: bool,
    pub movement_divisor: f64,
}

impl WeakSylvester {
    /// `direction_draw` and `movement_draw` are two successive values from
    /// the board's existing RNG, supplied in that order. The actor owns no RNG.
    pub fn spawn(
        id: u64,
        widget_x: i32,
        widget_y: i32,
        direction_draw: u32,
        movement_draw: u32,
    ) -> Self {
        Self::spawn_kind(
            SylvesterKind::Weak,
            id,
            widget_x,
            widget_y,
            direction_draw,
            movement_draw,
        )
    }

    pub fn spawn_kind(
        kind: SylvesterKind,
        id: u64,
        widget_x: i32,
        widget_y: i32,
        direction_draw: u32,
        movement_draw: u32,
    ) -> Self {
        let left = direction_draw.is_multiple_of(2);
        // Ordinary Destructor and Ulysses fix their vertical lane after the
        // warning has chosen a location (W1 Alien constructor, 98-116).
        let widget_y = if matches!(kind, SylvesterKind::Destructor | SylvesterKind::Ulysses) {
            280
        } else {
            widget_y
        };
        Self {
            id,
            kind,
            x: f64::from(widget_x),
            y: f64::from(widget_y),
            widget_x,
            widget_y,
            vx: if left { -3.0 } else { 3.0 },
            vy: 0.0,
            target_vx: 0.0,
            target_vy: 0.0,
            previous_vx: if left { -1.0 } else { 1.0 },
            health: kind.starting_health(),
            spawn_ticks: 15,
            chase_ticks: 100,
            hit_ticks: 0,
            movement_state: (movement_draw % 10) as u8,
            movement_change_ticks: 20,
            swim_ticks: 0,
            turn_ticks: 0,
            frame: 1,
            alive: true,
            launch_ticks: 0,
            reload_ticks: if matches!(kind, SylvesterKind::Destructor | SylvesterKind::Ulysses) {
                75
            } else {
                0
            },
            special_ticks: 0,
            phase_ticks: if kind == SylvesterKind::Psychosquid {
                200
            } else {
                0
            },
            phase_threshold: if kind == SylvesterKind::Psychosquid {
                400
            } else {
                0
            },
            healing: false,
            ever_healed: false,
            movement_divisor: kind.speed_divisor(),
        }
    }

    /// Advances one unpaused actor update. The callback must yield the next
    /// board RNG value. It is called only when the idle movement timer rolls
    /// over, and a second time only if the first roll selects a new state.
    /// Caller-supplied prey order is the collision order; no target is saved.
    pub fn update(&mut self, prey: &[PreyView], next_random: impl FnMut() -> u32) -> AlienUpdate {
        self.update_with_food(prey, &[], next_random)
    }

    pub fn update_with_food(
        &mut self,
        prey: &[PreyView],
        food: &[AlienFoodView],
        mut next_random: impl FnMut() -> u32,
    ) -> AlienUpdate {
        self.update_with_runtime(prey, food, |request| match request {
            AlienRuntimeRequest::Random => next_random(),
            AlienRuntimeRequest::Launch { .. } | AlienRuntimeRequest::ProbeTarget { .. } => 0,
        })
    }

    pub fn update_with_runtime(
        &mut self,
        prey: &[PreyView],
        food: &[AlienFoodView],
        mut runtime: impl FnMut(AlienRuntimeRequest) -> u32,
    ) -> AlienUpdate {
        if !self.alive {
            return AlienUpdate::default();
        }

        let mut emergence_dx = 0.0;
        if self.spawn_ticks != 0 {
            if self.spawn_ticks <= 10 {
                emergence_dx = if self.vx >= 0.0 { 3.0 } else { -3.0 };
            }
            self.spawn_ticks -= 1;
            // The count starts at 15: updates 1–6 return. On update 7,
            // old count 9 selects ±3, then count 8 permits motion.
            if self.spawn_ticks > 8 {
                return AlienUpdate::default();
            }
        }

        let mut result = AlienUpdate::default();
        if self.kind == SylvesterKind::Gus {
            if let Some((target_x, target_y, food_found)) = self.gus_target(food, prey) {
                self.chase_gus(target_x, target_y, food_found);
                result.food_eaten = self.gus_food_contact(food);
                if self.chase_ticks == 0 {
                    result.prey_eaten = self.first_contact(prey);
                    if result.prey_eaten.is_some() {
                        self.health -= 15.0;
                        self.hit_ticks = 10;
                    }
                }
            } else {
                self.wander(&mut || runtime(AlienRuntimeRequest::Random));
            }
        } else if matches!(
            self.kind,
            SylvesterKind::Destructor | SylvesterKind::Ulysses
        ) || self.healing
        {
            // AlienUnk01 never chases for either projectile species.
            self.wander(&mut || runtime(AlienRuntimeRequest::Random));
        } else if let Some(target) = self.nearest_eligible(prey) {
            self.chase(target);
            if self.chase_ticks == 0 {
                result.prey_eaten = self.first_contact(prey);
            }
        } else {
            self.wander(&mut || runtime(AlienRuntimeRequest::Random));
        }

        if self.kind == SylvesterKind::Psychosquid {
            self.phase_ticks += 1;
            if self.phase_ticks >= self.phase_threshold {
                self.healing = !self.healing;
                self.ever_healed |= self.healing;
                self.special_ticks = 10;
                self.phase_ticks = (runtime(AlienRuntimeRequest::Random) % 100) as u16;
                self.movement_divisor = if self.healing { 2.0 } else { 0.5 };
                result.phase_changed = Some(self.healing);
            }
        }
        if self.kind == SylvesterKind::Ulysses {
            // PB05 raw6 pulse reads the *previous* animation frame and
            // special timer before integrating position or animating.
            if self.special_ticks == 0 {
                match self.frame % 5 {
                    0 | 4 if self.vx < -0.9 => self.vx += 0.8,
                    0 | 4 if self.vx > 0.9 => self.vx -= 0.8,
                    1 | 2 if self.vx < 0.0 => self.vx -= 0.55,
                    1 | 2 if self.vx > 0.0 => self.vx += 0.55,
                    _ => {}
                }
            } else {
                self.vx = self.vx.clamp(-0.1, 0.1);
            }
        }
        self.x = self.x.clamp(-10.0, 490.0) + self.vx / self.movement_divisor + emergence_dx;
        self.y = self.y.clamp(85.0, 290.0) + self.vy / self.movement_divisor;
        self.hit_ticks = self.hit_ticks.saturating_sub(1);
        self.chase_ticks = self.chase_ticks.saturating_sub(1);

        if matches!(
            self.kind,
            SylvesterKind::Destructor | SylvesterKind::Ulysses
        ) && !prey.is_empty()
        {
            self.launch_ticks += 1;
            if self.kind == SylvesterKind::Ulysses
                && self.launch_ticks == self.reload_ticks.saturating_sub(15)
            {
                if self.turn_ticks == 0 {
                    if runtime(AlienRuntimeRequest::ProbeTarget {
                        center_x: self.widget_x + 80,
                        center_y: self.widget_y + 80,
                        excluded_prey: result.prey_eaten,
                    }) == 0
                    {
                        self.launch_ticks = 0;
                    } else {
                        self.special_ticks = 40;
                    }
                } else {
                    self.launch_ticks = self.launch_ticks.saturating_sub(2);
                }
            }
            if self.launch_ticks > self.reload_ticks {
                let right = self.vx >= 0.0;
                let mut launched = false;
                let count = if self.kind == SylvesterKind::Ulysses {
                    2
                } else {
                    3
                };
                for slot in 0..count {
                    let (x, y) = if self.kind == SylvesterKind::Ulysses {
                        (
                            self.widget_x
                                + if right {
                                    70 + i32::from(slot) * 8
                                } else {
                                    10 - i32::from(slot) * 8
                                },
                            self.widget_y - 24 + i32::from(slot) * 8,
                        )
                    } else {
                        (
                            self.widget_x
                                + if right {
                                    90 + i32::from(slot) * 15
                                } else {
                                    -10 - i32::from(slot) * 15
                                },
                            self.widget_y - 35 + i32::from(slot) * 5,
                        )
                    };
                    if runtime(AlienRuntimeRequest::Launch {
                        slot,
                        x,
                        y,
                        center_x: self.widget_x + 80,
                        center_y: self.widget_y + 80,
                        excluded_prey: result.prey_eaten,
                    }) == 0
                    {
                        break;
                    }
                    launched = true;
                }
                self.launch_ticks = 0;
                self.reload_ticks = 150 + (runtime(AlienRuntimeRequest::Random) % 50) as u16;
                if launched && self.kind == SylvesterKind::Destructor {
                    self.special_ticks = 10;
                }
            }
        }

        // C++ Widget::Move(int,int) truncates the double arguments. Keep
        // this snapshot before animation's further double-position shift.
        self.widget_x = self.x as i32;
        self.widget_y = self.y as i32;
        self.animate();
        if self.health <= 0.0 {
            self.alive = false;
            result.defeated = true;
        }
        result
    }

    /// A single player laser attempt. The board owns shot effects and the
    /// exactly-once removal/diamond transaction returned on defeat.
    pub fn shot(&mut self, shot_x: i32, shot_y: i32) -> ShotResult {
        self.shot_with_weapon(shot_x, shot_y, 2)
    }

    pub fn shot_with_weapon(&mut self, shot_x: i32, shot_y: i32, weapon: u8) -> ShotResult {
        self.shot_with_weapon_and_random(shot_x, shot_y, weapon, || 0)
    }

    pub fn shot_with_weapon_and_random(
        &mut self,
        shot_x: i32,
        shot_y: i32,
        weapon: u8,
        mut next_random: impl FnMut() -> u32,
    ) -> ShotResult {
        let sx = f64::from(shot_x);
        let sy = f64::from(shot_y);
        if !self.alive
            || sx <= self.x
            || sx >= self.x + f64::from(WEAK_SYLVESTER_SIZE)
            || sy <= self.y
            || sy >= self.y + f64::from(WEAK_SYLVESTER_SIZE)
            || self.hit_ticks != 0
        {
            return ShotResult::Miss;
        }

        if self.kind == SylvesterKind::Psychosquid && self.healing {
            self.health += f64::from(weapon) * 3.0;
        } else {
            self.health -= if matches!(
                self.kind,
                SylvesterKind::Destructor | SylvesterKind::Ulysses
            ) {
                f64::from(weapon) * 2.0 + 2.0
            } else {
                f64::from(weapon) * 3.0
            };
            if self.kind == SylvesterKind::Psychosquid && self.health < 100.0 && !self.ever_healed {
                self.healing = true;
                self.ever_healed = true;
                self.special_ticks = 10;
                self.phase_ticks = (next_random() % 100) as u16;
                // The forced Shot path does not write movement_divisor.
            }
        }
        self.apply_shot_push(sx, sy);
        if self.health <= 0.0 {
            self.alive = false;
            ShotResult::Defeated {
                diamond_x: self.widget_x + 25,
                diamond_y: self.widget_y + 25,
            }
        } else {
            self.hit_ticks = 10;
            ShotResult::Hit {
                health: self.health,
            }
        }
    }

    /// FishTypePet Itchy contact only subtracts HP. It does not apply shot
    /// immunity, flash, push or immediate removal; the next active alien
    /// update owns the eventual death transaction.
    pub fn itchy_hit(&mut self) -> Option<f64> {
        if !self.alive {
            return None;
        }
        self.health -= if matches!(self.kind, SylvesterKind::Gus | SylvesterKind::Destructor) {
            0.25
        } else {
            1.0
        };
        Some(self.health)
    }

    pub fn rufus_hit(&mut self) -> Option<f64> {
        if !self.alive {
            return None;
        }
        self.health -= match self.kind {
            SylvesterKind::Destructor => 0.25,
            SylvesterKind::Gus => 0.5,
            _ => 2.0,
        };
        Some(self.health)
    }

    /// PB71: Gash bypasses shot immunity and halves only raw4/5 damage.
    /// The encounter owner commits lethal removal synchronously.
    pub fn gash_hit(&mut self) -> Option<f64> {
        if !self.alive || self.healing {
            return None;
        }
        self.health -= if matches!(self.kind, SylvesterKind::Gus | SylvesterKind::Destructor) {
            0.5
        } else {
            3.0
        };
        Some(self.health)
    }

    pub fn validate(&self) -> Result<(), String> {
        if ![
            self.x,
            self.y,
            self.vx,
            self.vy,
            self.target_vx,
            self.target_vy,
            self.previous_vx,
        ]
        .into_iter()
        .all(f64::is_finite)
            || !(-32.0..=512.0).contains(&self.x)
            || !(64.0..=312.0).contains(&self.y)
            || self.vx.abs()
                > (if self.kind == SylvesterKind::Ulysses {
                    12.0
                } else {
                    7.0
                })
            || self.vy.abs() > 7.0
            || self.target_vx.abs() > 2.0
            || self.target_vy.abs() > 2.0
        {
            return Err("invalid weak Sylvester motion".into());
        }
        if self.widget_x < -32
            || self.widget_x > 512
            || self.widget_y < 64
            || self.widget_y > 312
            || !self.health.is_finite()
            || (self.kind != SylvesterKind::Psychosquid && self.health > self.kind.starting_health())
            || (self.health * 4.0).fract() != 0.0
            // Project-save guard for one ordinary Itchy contact per elapsed
            // spawn update. This bounds malformed pending-death state without
            // rejecting the original's deferred removal during emergence.
            || self.alive
                && self.health < -f64::from(15_u8.saturating_sub(self.spawn_ticks))
                    * (match self.kind {
                        SylvesterKind::Destructor => 0.5,
                        SylvesterKind::Gus => 0.75,
                        _ => 3.0,
                    })
            || !self.alive && self.health > 0.0
            || self.spawn_ticks > 15
            || self.chase_ticks > 100
            || self.hit_ticks > 10
            || self.movement_state > 9
            || self.movement_change_ticks > 20
            || self.swim_ticks > (if self.kind == SylvesterKind::Ulysses { 79 } else { 50 })
            || !(if self.kind == SylvesterKind::Ulysses { -19..=19 } else { -9..=9 }).contains(&self.turn_ticks)
            || self.frame > 9
            || (matches!(self.kind, SylvesterKind::Destructor | SylvesterKind::Ulysses)
                && (self.reload_ticks < 75 || self.reload_ticks > 199
                    || (self.kind == SylvesterKind::Ulysses
                        && self.reload_ticks != 75
                        && !(150..=199).contains(&self.reload_ticks))
                    || self.launch_ticks > self.reload_ticks || self.special_ticks > (if self.kind == SylvesterKind::Ulysses { 40 } else { 10 })))
            || (!matches!(self.kind, SylvesterKind::Destructor | SylvesterKind::Ulysses)
                && (self.launch_ticks != 0 || self.reload_ticks != 0
                    || (self.kind != SylvesterKind::Psychosquid && self.special_ticks != 0)))
            || (self.kind == SylvesterKind::Psychosquid && (self.phase_threshold != 400
                || self.phase_ticks >= 400 || self.special_ticks > 10
                || (self.movement_divisor != 0.5 && self.movement_divisor != 2.0)
                || (!self.ever_healed && self.healing)))
            || (self.kind != SylvesterKind::Psychosquid && (self.phase_ticks != 0
                || self.phase_threshold != 0 || self.healing || self.ever_healed
                || self.movement_divisor != self.kind.speed_divisor()))
        {
            return Err("invalid weak Sylvester counters".into());
        }
        Ok(())
    }

    pub fn sprite_row(&self) -> u8 {
        if self.kind == SylvesterKind::Ulysses && self.special_ticks > 0 {
            2
        } else if self.kind == SylvesterKind::Psychosquid && self.special_ticks > 0 {
            4
        } else if self.kind == SylvesterKind::Gus && self.hit_ticks > 0 {
            2
        } else if self.kind == SylvesterKind::Psychosquid && self.healing {
            2 + u8::from(self.turn_ticks != 0)
        } else {
            u8::from(self.turn_ticks != 0)
        }
    }

    pub fn sprite_frame(&self) -> u8 {
        if self.kind == SylvesterKind::Gus && self.hit_ticks > 0 {
            self.hit_ticks.min(9)
        } else {
            self.frame
        }
    }

    pub fn facing_right(&self) -> bool {
        if self.kind == SylvesterKind::Gus && self.hit_ticks > 0 {
            return self.vx >= 0.0;
        }
        if self.turn_ticks != 0 {
            self.turn_ticks > 0
        } else {
            self.vx >= 0.0
        }
    }

    pub fn hit_flash(&self) -> bool {
        self.hit_ticks > 0
    }

    /// Source target selection uses old integer widget centers; equal
    /// distances retain the first item in the board's ordered view.
    fn nearest_eligible<'a>(&self, prey: &'a [PreyView]) -> Option<&'a PreyView> {
        let center_x = i64::from(self.widget_x) + 80;
        let center_y = i64::from(self.widget_y) + 80;
        let mut nearest = None;
        let mut nearest_distance = 100_000_000_i64;
        for candidate in prey.iter().filter(|view| view.eligible) {
            let dx = center_x - i64::from(candidate.widget_x + candidate.width / 2);
            let dy = center_y - i64::from(candidate.widget_y + candidate.height / 2);
            let distance = dx * dx + dy * dy;
            if distance < nearest_distance {
                nearest = Some(candidate);
                nearest_distance = distance;
            }
        }
        nearest
    }

    /// Contact scans all eligible prey in board order, independently of the
    /// nearest target selected for steering. It uses the previous widget
    /// position because the alien moves later in the update.
    fn first_contact(&self, prey: &[PreyView]) -> Option<u64> {
        let center_x = i64::from(self.widget_x) + 80;
        let center_y = i64::from(self.widget_y) + 80;
        prey.iter()
            .filter(|view| view.eligible)
            .find_map(|candidate| {
                let dx = center_x - i64::from(candidate.widget_x + candidate.width / 2);
                let dy = center_y - i64::from(candidate.widget_y + candidate.height / 2);
                (dx.abs() < 45 && dy.abs() < 65).then_some(candidate.id)
            })
    }

    /// The source biases each accepted food distance by 2500 before the next
    /// comparison, then may replace that target with fish without clearing
    /// its food-steering flag (PB31).
    fn gus_target(&self, food: &[AlienFoodView], prey: &[PreyView]) -> Option<(i32, i32, bool)> {
        let cx = i64::from(self.widget_x + 80);
        let cy = i64::from(self.widget_y + 80);
        let mut best = 100_000_000_i64;
        let mut target = None;
        let mut food_found = false;
        for item in food.iter().filter(|item| item.eligible) {
            let dx = cx - i64::from(item.widget_x + 20);
            let dy = cy - i64::from(item.widget_y + 20);
            let distance = dx * dx + dy * dy;
            if distance < best {
                best = distance - 2500;
                food_found = true;
                target = Some((item.widget_x, item.widget_y));
            }
        }
        if self.chase_ticks == 0 {
            for item in prey.iter().filter(|item| item.eligible) {
                let dx = cx - i64::from(item.widget_x + item.width / 2);
                let dy = cy - i64::from(item.widget_y + item.height / 2);
                let distance = dx * dx + dy * dy;
                if distance < best {
                    best = distance;
                    target = Some((item.widget_x, item.widget_y));
                }
            }
        }
        target.map(|(x, y)| (x, y, food_found))
    }

    fn chase_gus(&mut self, x: i32, y: i32, food_found: bool) {
        let offset = if food_found { 20.0 } else { 40.0 };
        let tx = f64::from(x) + offset;
        let ty = f64::from(y) + offset;
        if self.x + 80.0 < tx && self.vx < 1.8 {
            self.vx += 0.1;
        } else if self.x + 80.0 > tx && self.vx > -1.8 {
            self.vx -= 0.1;
        }
        if self.y + 80.0 < ty && self.vy < (if food_found { 3.0 } else { 1.8 }) {
            self.vy += if food_found { 0.3 } else { 0.1 };
        } else if self.y + 80.0 > ty && self.vy > -1.8 {
            self.vy -= 0.1;
        }
    }

    fn gus_food_contact(&mut self, food: &[AlienFoodView]) -> Option<(u64, u8)> {
        if self.hit_ticks >= 6 {
            return None;
        }
        let cx = self.x + 80.0;
        let cy = self.y + 80.0;
        let item = food.iter().find(|item| {
            item.eligible
                && cx > f64::from(item.widget_x - 25)
                && cx < f64::from(item.widget_x + 65)
                && cy > f64::from(item.widget_y - 25)
                && cy < f64::from(item.widget_y + 65)
        })?;
        let damage = if item.quality == 3 {
            20
        } else {
            item.quality * 2 + 4
        };
        self.health -= f64::from(damage);
        self.hit_ticks = 10;
        Some((item.id, damage))
    }

    fn chase(&mut self, target: &PreyView) {
        let center_x = self.x + 80.0;
        let center_y = self.y + 80.0;
        let target_x = f64::from(target.widget_x + 40);
        let target_y = f64::from(target.widget_y + 40);
        if center_x < target_x && self.vx < 1.8 {
            self.vx += 0.1;
        } else if center_x > target_x && self.vx > -1.8 {
            self.vx -= 0.1;
        }
        if center_y < target_y && self.vy < 1.8 {
            self.vy += 0.1;
        } else if center_y > target_y && self.vy > -1.8 {
            self.vy -= 0.1;
        }
    }

    fn wander(&mut self, next_random: &mut impl FnMut() -> u32) {
        if matches!(
            self.kind,
            SylvesterKind::Destructor | SylvesterKind::Ulysses
        ) {
            match self.movement_state {
                0 | 2 => self.target_vx = -1.0,
                1 | 3 => self.target_vx = 1.0,
                _ => {}
            }
        } else {
            match self.movement_state {
                0 => self.target_vx = -1.5,
                1 => self.target_vx = 1.5,
                2 => self.target_vy = -1.5,
                3 => self.target_vy = 1.5,
                _ => {}
            }
            // W1 uses independent comparisons, retaining small overshoot.
            if self.vy > self.target_vy {
                self.vy -= 0.1;
            }
            if self.vy < self.target_vy {
                self.vy += 0.1;
            }
        }
        if self.vx > self.target_vx {
            self.vx -= 0.1;
        }
        if self.vx < self.target_vx {
            self.vx += 0.1;
        }
        if self.special_ticks == 0 {
            self.movement_change_ticks += 1;
            if self.movement_change_ticks > 20 {
                self.movement_change_ticks = 0;
                if next_random().is_multiple_of(10) {
                    self.movement_state = (next_random() % 4) as u8;
                }
            }
        }
    }

    fn animate(&mut self) {
        let speed_divisor = self.movement_divisor;
        if self.previous_vx < 0.0 && self.vx > 0.0 {
            self.turn_ticks = if self.kind == SylvesterKind::Ulysses {
                -20
            } else {
                -10
            };
        } else if self.previous_vx > 0.0 && self.vx < 0.0 {
            self.turn_ticks = if self.kind == SylvesterKind::Ulysses {
                20
            } else {
                10
            };
        }
        self.turn_ticks -= self.turn_ticks.signum();

        if self.special_ticks > 0 {
            self.special_ticks -= 1;
            if self.kind == SylvesterKind::Psychosquid {
                if self.special_ticks > 0 {
                    if self.healing {
                        self.frame = 10 - self.special_ticks;
                    }
                    self.previous_vx = self.vx;
                    return;
                }
                self.swim_ticks = 0;
            }
            if self.kind == SylvesterKind::Destructor {
                if self.special_ticks > 0 {
                    self.frame = self.special_ticks;
                    if self.vx != self.previous_vx && self.vx != 0.0 && self.previous_vx != 0.0 {
                        self.previous_vx = self.vx;
                    }
                    return;
                }
                self.swim_ticks = 0;
            }
            if self.kind == SylvesterKind::Ulysses {
                self.frame = if self.special_ticks > 20 {
                    9 - (self.special_ticks - 20) / 2
                } else {
                    self.special_ticks.saturating_sub(1) / 2
                };
                self.previous_vx = self.vx;
                return;
            }
        }

        if self.turn_ticks > 0 {
            self.frame = if self.kind == SylvesterKind::Ulysses {
                (9 - self.turn_ticks / 2) as u8
            } else {
                (9 - self.turn_ticks) as u8
            };
        } else if self.turn_ticks < 0 {
            self.frame = if self.kind == SylvesterKind::Ulysses {
                (9 + self.turn_ticks / 2) as u8
            } else {
                (self.turn_ticks + 10) as u8
            };
        } else if self.kind == SylvesterKind::Ulysses {
            self.swim_ticks = (self.swim_ticks + if self.vx.abs() > 8.0 { 2 } else { 1 }) % 80;
            self.frame = self.swim_ticks / 8;
        } else if matches!(
            self.kind,
            SylvesterKind::Destructor | SylvesterKind::Psychosquid
        ) || self.vx.abs() <= 1.6
        {
            self.swim_ticks += 1;
            if self.swim_ticks > 19 {
                self.swim_ticks = 0;
            }
            self.frame = self.swim_ticks / 2;
        } else {
            self.swim_ticks += 1;
            match self.swim_ticks {
                1..=6 => {
                    self.x -= self.vx / speed_divisor * 0.25;
                    self.y -= self.vy / speed_divisor * 0.25;
                    self.frame = self.swim_ticks / 2;
                }
                7..=10 => {
                    self.x += self.vx / speed_divisor * 0.75;
                    self.y += self.vy / speed_divisor * 0.75;
                    self.frame = self.swim_ticks / 2;
                }
                11..=39 => {
                    self.x += self.vx / speed_divisor * 0.5;
                    self.y += self.vy / speed_divisor * 0.5;
                    self.frame = 6;
                }
                40..=50 => {
                    self.x += self.vx / speed_divisor * 0.5;
                    self.y += self.vy / speed_divisor * 0.5;
                    self.frame = (50 - self.swim_ticks) / 2;
                }
                _ => {
                    self.swim_ticks = 0;
                    self.frame = 0;
                }
            }
        }

        if self.vx != self.previous_vx && self.vx != 0.0 && self.previous_vx != 0.0 {
            self.previous_vx = self.vx;
        }
    }

    fn apply_shot_push(&mut self, sx: f64, sy: f64) {
        if matches!(
            self.kind,
            SylvesterKind::Destructor | SylvesterKind::Ulysses
        ) {
            if self.special_ticks == 0 {
                if sx < self.x + 60.0 {
                    self.vx = self.movement_divisor * 3.0;
                } else if sx > self.x + 100.0 {
                    self.vx = -self.movement_divisor * 3.0;
                }
            }
            return;
        }
        let diagonal = self.movement_divisor
            * if self.kind == SylvesterKind::Psychosquid {
                3.5
            } else {
                2.5
            };
        let cardinal = self.movement_divisor
            * if self.kind == SylvesterKind::Psychosquid {
                4.0
            } else {
                3.0
            };
        let left = sx < self.x + 60.0;
        let upper = sy < self.y + 60.0;
        let right = sx > self.x + 100.0;
        let lower = sy > self.y + 100.0;
        if left && upper {
            (self.vx, self.vy) = (diagonal, diagonal);
        } else if left && sy < self.y + 100.0 {
            self.vx = cardinal;
        } else if left {
            (self.vx, self.vy) = (diagonal, -diagonal);
        } else if sx < self.x + 100.0 && upper {
            self.vy = cardinal;
        } else if right && lower {
            (self.vx, self.vy) = (-diagonal, -diagonal);
        } else if right && upper {
            (self.vx, self.vy) = (-diagonal, diagonal);
        } else if right {
            self.vx = -cardinal;
        } else if lower {
            self.vy = -cardinal;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alien() -> WeakSylvester {
        WeakSylvester::spawn(7, 100, 120, 1, 1)
    }

    #[test]
    fn psychosquid_timed_and_forced_healing_preserve_distinct_speed_writes() {
        let mut timed = WeakSylvester::spawn_kind(SylvesterKind::Psychosquid, 90, 100, 120, 1, 1);
        timed.spawn_ticks = 0;
        timed.phase_ticks = 399;
        timed.movement_change_ticks = 0;
        let mut requests = Vec::new();
        let changed = timed.update_with_runtime(&[], &[], |request| {
            requests.push(request);
            37
        });
        assert_eq!(changed.phase_changed, Some(true));
        assert_eq!(
            (
                timed.phase_ticks,
                timed.movement_divisor,
                timed.special_ticks
            ),
            (37, 2.0, 9)
        );
        assert_eq!(
            requests
                .iter()
                .filter(|request| matches!(request, AlienRuntimeRequest::Random))
                .count(),
            1
        );
        assert_eq!(timed.sprite_row(), 4);

        let mut forced = WeakSylvester::spawn_kind(SylvesterKind::Psychosquid, 91, 100, 120, 1, 1);
        forced.spawn_ticks = 0;
        forced.health = 104.0;
        assert!(matches!(
            forced.shot_with_weapon_and_random(110, 130, 2, || 81),
            ShotResult::Hit { health: 98.0 }
        ));
        assert!(forced.healing && forced.ever_healed);
        assert_eq!(
            (
                forced.phase_ticks,
                forced.movement_divisor,
                forced.special_ticks
            ),
            (81, 0.5, 10)
        );
        forced.validate().unwrap();
    }

    #[test]
    fn psychosquid_healing_has_no_starting_health_cap_and_uses_hit_immunity() {
        let mut actor = WeakSylvester::spawn_kind(SylvesterKind::Psychosquid, 92, 100, 120, 1, 1);
        actor.spawn_ticks = 0;
        actor.healing = true;
        actor.ever_healed = true;
        actor.movement_divisor = 2.0;
        actor.health = 259.0;
        assert!(matches!(
            actor.shot_with_weapon(110, 130, 12),
            ShotResult::Hit { health: 295.0 }
        ));
        assert_eq!(actor.shot_with_weapon(110, 130, 12), ShotResult::Miss);
        actor.validate().unwrap();
    }

    #[test]
    fn psychosquid_transition_freezes_wander_roll_but_keeps_motion() {
        let mut actor = WeakSylvester::spawn_kind(SylvesterKind::Psychosquid, 93, 100, 120, 1, 1);
        actor.spawn_ticks = 0;
        actor.healing = true;
        actor.ever_healed = true;
        actor.special_ticks = 10;
        actor.movement_change_ticks = 20;
        let old_x = actor.x;
        actor.update(&[], || panic!("transition must not consume movement RNG"));
        assert_eq!(actor.movement_change_ticks, 20);
        assert_eq!(actor.special_ticks, 9);
        assert_ne!(actor.x, old_x);
    }

    #[test]
    fn spawn_early_returns_then_widget_precedes_animation_and_emergence() {
        let mut actor = alien();
        let mut draws = 0;
        for _ in 0..6 {
            assert_eq!(
                actor.update(&[], || {
                    draws += 1;
                    1
                }),
                AlienUpdate::default()
            );
            assert_eq!((actor.widget_x, actor.widget_y), (100, 120));
            assert_eq!(actor.chase_ticks, 100);
        }
        assert_eq!(draws, 0);
        actor.update(&[], || {
            draws += 1;
            1
        });
        assert_eq!(actor.spawn_ticks, 8);
        assert_eq!(actor.widget_x, 104); // First applied ±3 offset.
        assert!(actor.x > f64::from(actor.widget_x));
        assert_eq!(draws, 1); // First active idle rollover.
        actor.update(&[], || {
            draws += 1;
            1
        });
        assert_eq!(actor.spawn_ticks, 7);
        assert_eq!(actor.widget_x, 108);
    }

    #[test]
    fn first_overlap_is_eaten_even_when_another_fish_is_nearest() {
        let mut actor = alien();
        actor.spawn_ticks = 0;
        actor.chase_ticks = 0;
        actor.x = 200.0;
        actor.y = 200.0;
        actor.widget_x = 200;
        actor.widget_y = 200;
        let prey = [
            PreyView {
                id: 1,
                widget_x: 285,
                widget_y: 240,
                width: 80,
                height: 80,
                eligible: true,
            },
            PreyView {
                id: 2,
                widget_x: 284,
                widget_y: 240,
                width: 80,
                height: 80,
                eligible: true,
            },
            PreyView {
                id: 3,
                widget_x: 240,
                widget_y: 240,
                width: 80,
                height: 80,
                eligible: true,
            },
        ];
        assert_eq!(
            actor
                .update(&prey, || panic!("chase should consume no RNG"))
                .prey_eaten,
            Some(2)
        );
    }

    #[test]
    fn shooting_uses_strict_double_bounds_and_one_death_reward() {
        let mut actor = alien();
        actor.spawn_ticks = 0;
        assert_eq!(actor.shot(100, 200), ShotResult::Miss);
        assert_eq!(actor.shot(260, 200), ShotResult::Miss);
        assert_eq!(actor.shot(101, 121), ShotResult::Hit { health: 44.0 });
        assert_eq!(actor.shot(101, 121), ShotResult::Miss);
        for accepted_shot in 2..=9 {
            for _ in 0..10 {
                actor.update(&[], || 1);
            }
            let shot_x = (actor.x + 80.0) as i32;
            let shot_y = (actor.y + 80.0) as i32;
            let expected_origin = (actor.widget_x + 25, actor.widget_y + 25);
            let result = actor.shot(shot_x, shot_y);
            if accepted_shot == 9 {
                assert_eq!(
                    result,
                    ShotResult::Defeated {
                        diamond_x: expected_origin.0,
                        diamond_y: expected_origin.1,
                    }
                );
            } else {
                assert_eq!(
                    result,
                    ShotResult::Hit {
                        health: f64::from(50 - 6 * accepted_shot)
                    }
                );
            }
        }
        assert_eq!(
            actor.shot(actor.widget_x + 80, actor.widget_y + 80),
            ShotResult::Miss
        );
    }

    #[test]
    fn strong_constructor_and_board_weapon_change_lethal_hit_count() {
        // PB17 confirms strong HP60/divisor1.6; PB13 confirms Board weapon×3.
        let mut strong = WeakSylvester::spawn_kind(SylvesterKind::Strong, 8, 100, 120, 1, 1);
        assert_eq!(strong.health, 60.0);
        assert_eq!(strong.kind.speed_divisor(), 1.6);
        for hit in 1..=9 {
            strong.hit_ticks = 0;
            assert_eq!(
                strong.shot_with_weapon(120, 140, 2),
                ShotResult::Hit {
                    health: f64::from(60 - hit * 6)
                }
            );
        }
        strong.hit_ticks = 0;
        assert!(matches!(
            strong.shot_with_weapon(120, 140, 2),
            ShotResult::Defeated { .. }
        ));

        let mut upgraded = WeakSylvester::spawn_kind(SylvesterKind::Strong, 9, 100, 120, 1, 1);
        for hit in 1..=6 {
            upgraded.hit_ticks = 0;
            assert_eq!(
                upgraded.shot_with_weapon(120, 140, 3),
                ShotResult::Hit {
                    health: f64::from(60 - hit * 9)
                }
            );
        }
        upgraded.hit_ticks = 0;
        assert!(matches!(
            upgraded.shot_with_weapon(120, 140, 3),
            ShotResult::Defeated { .. }
        ));
    }

    #[test]
    fn strong_shot_push_scales_with_confirmed_speed_divisor() {
        let mut actor = WeakSylvester::spawn_kind(SylvesterKind::Strong, 8, 100, 120, 1, 1);
        assert_eq!(
            actor.shot_with_weapon(101, 121, 2),
            ShotResult::Hit { health: 54.0 }
        );
        assert_eq!((actor.vx, actor.vy), (4.0, 4.0));
        let mut side = WeakSylvester::spawn_kind(SylvesterKind::Strong, 9, 100, 120, 1, 1);
        side.shot_with_weapon(101, 180, 2);
        assert!((side.vx - 4.8).abs() < 1e-9);
    }

    #[test]
    fn itchy_contact_leaves_zero_health_actor_registered_without_shot_state() {
        let mut actor = WeakSylvester::spawn_kind(SylvesterKind::Balrog, 48, 100, 120, 1, 1);
        actor.health = 1.0;
        let before_motion = (actor.vx, actor.vy);
        assert_eq!(actor.itchy_hit(), Some(0.0));
        assert!(actor.alive);
        assert_eq!(actor.hit_ticks, 0);
        assert_eq!((actor.vx, actor.vy), before_motion);
        actor.validate().unwrap();
        assert_eq!(actor.itchy_hit(), Some(-1.0));
        assert!(actor.alive);
    }

    #[test]
    fn pending_health_save_bound_tracks_emergence_and_malformed_math_cannot_overflow() {
        let mut actor = WeakSylvester::spawn_kind(SylvesterKind::Balrog, 49, 100, 120, 1, 1);
        actor.health = -1.0;
        assert!(
            actor.validate().is_err(),
            "negative HP before first update is impossible"
        );
        actor.health = 1.0;
        for _ in 0..6 {
            actor.update(&[], || 1);
            actor.itchy_hit();
            actor.validate().unwrap();
        }
        assert_eq!((actor.spawn_ticks, actor.health), (9, -5.0));
        assert!(actor.update(&[], || 1).defeated);

        let mut malformed = WeakSylvester::spawn_kind(SylvesterKind::Balrog, 50, 100, 120, 1, 1);
        malformed.health = f64::from(i16::MIN);
        assert!(malformed.validate().is_err());
        assert_eq!(malformed.itchy_hit(), Some(f64::from(i16::MIN) - 1.0));
        assert!(matches!(
            malformed.shot_with_weapon(180, 200, 12),
            ShotResult::Defeated { .. }
        ));
    }

    #[test]
    fn hidden_spawn_does_not_advance_nonlethal_hit_immunity() {
        let mut actor = alien();
        assert_eq!(actor.shot(180, 200), ShotResult::Hit { health: 44.0 });
        for _ in 0..6 {
            actor.update(&[], || panic!("hidden return should consume no RNG"));
        }
        assert_eq!(actor.hit_ticks, 10);
        actor.update(&[], || 1);
        assert_eq!(actor.hit_ticks, 9);
    }

    #[test]
    fn shot_push_bands_use_ordered_strict_edges() {
        let mut middle = alien();
        assert_eq!(middle.shot(160, 180), ShotResult::Hit { health: 44.0 });
        assert_eq!((middle.vx, middle.vy), (3.0, 0.0));

        let mut left_band = alien();
        left_band.shot(159, 180);
        assert_eq!((left_band.vx, left_band.vy), (6.0, 0.0));

        let mut right_upper = alien();
        right_upper.shot(201, 179);
        assert_eq!((right_upper.vx, right_upper.vy), (-5.0, 5.0));
    }

    #[test]
    fn idle_rng_requests_only_the_conditional_state_draw() {
        let mut actor = alien();
        actor.spawn_ticks = 0;
        let mut rolls = [0_u32, 3].into_iter();
        actor.update(&[], || rolls.next().unwrap());
        assert_eq!(actor.movement_state, 3);
        assert_eq!(rolls.next(), None);
        actor.movement_change_ticks = 20;
        let mut calls = 0;
        actor.update(&[], || {
            calls += 1;
            1
        });
        assert_eq!(calls, 1);
        assert_eq!(actor.movement_state, 3);
    }

    #[test]
    fn save_round_trip_keeps_double_and_widget_positions_distinct() {
        let mut actor = alien();
        actor.spawn_ticks = 0;
        actor.update(&[], || 1);
        assert_ne!(actor.x, f64::from(actor.widget_x));
        let encoded = serde_json::to_string(&actor).unwrap();
        let resumed: WeakSylvester = serde_json::from_str(&encoded).unwrap();
        assert_eq!(resumed.widget_x, actor.widget_x);
        assert_eq!(resumed.x, actor.x);
        resumed.validate().unwrap();
    }

    #[test]
    fn slowing_from_fast_swim_resets_the_shared_counter_above_nineteen() {
        let mut actor = alien();
        actor.vx = 1.6;
        actor.previous_vx = 1.6;
        actor.swim_ticks = 30;
        actor.animate();
        assert_eq!(actor.swim_ticks, 0);
        assert_eq!(actor.frame, 0);
    }

    #[test]
    fn gus_food_distance_subtraction_is_ordered_and_fish_replacement_keeps_food_flag() {
        let mut gus = WeakSylvester::spawn_kind(SylvesterKind::Gus, 70, 0, 0, 1, 1);
        gus.chase_ticks = 0;
        let food = [
            AlienFoodView {
                id: 1,
                widget_x: 60,
                widget_y: 70,
                quality: 0,
                eligible: true,
            },
            AlienFoodView {
                id: 2,
                widget_x: 70,
                widget_y: 70,
                quality: 0,
                eligible: true,
            },
        ];
        assert_eq!(gus.gus_target(&food, &[]), Some((60, 70, true)));
        let distant_food = [AlienFoodView {
            id: 3,
            widget_x: 300,
            widget_y: 300,
            quality: 0,
            eligible: true,
        }];
        let fish = [PreyView {
            id: 4,
            widget_x: 50,
            widget_y: 40,
            width: 80,
            height: 80,
            eligible: true,
        }];
        assert_eq!(gus.gus_target(&distant_food, &fish), Some((50, 40, true)));
        gus.vx = 0.0;
        gus.chase_gus(50, 40, true);
        assert_eq!(gus.vx, -0.1, "sticky food flag steers to fish X+20");
    }

    #[test]
    fn gus_food_target_uses_forty_pixel_food_centers_before_distance_bias() {
        let gus = WeakSylvester::spawn_kind(SylvesterKind::Gus, 74, 100, 100, 1, 1);
        let foods = [
            AlienFoodView {
                id: 1,
                widget_x: 260,
                widget_y: 160,
                quality: 0,
                eligible: true,
            },
            AlienFoodView {
                id: 2,
                widget_x: 160,
                widget_y: 245,
                quality: 0,
                eligible: true,
            },
        ];
        assert_eq!(gus.gus_target(&foods, &[]), Some((160, 245, true)));
    }

    #[test]
    fn gus_food_edges_hitflash_and_quality_damage_use_old_double_center() {
        let mut gus = WeakSylvester::spawn_kind(SylvesterKind::Gus, 71, 100, 100, 1, 1);
        gus.hit_ticks = 5;
        let edge = AlienFoodView {
            id: 1,
            widget_x: 115,
            widget_y: 115,
            quality: 2,
            eligible: true,
        };
        assert_eq!(gus.gus_food_contact(&[edge]), None);
        let inside = AlienFoodView {
            widget_x: 116,
            widget_y: 116,
            ..edge
        };
        assert_eq!(gus.gus_food_contact(&[inside]), Some((1, 8)));
        assert_eq!((gus.health, gus.hit_ticks), (92.0, 10));
        assert_eq!(gus.gus_food_contact(&[inside]), None);
        gus.hit_ticks = 5;
        let potion = AlienFoodView {
            quality: 3,
            ..inside
        };
        assert_eq!(gus.gus_food_contact(&[potion]), Some((1, 20)));
        assert_eq!(gus.health, 72.0);
    }

    #[test]
    fn lethal_gus_food_still_allows_same_update_prey_and_no_early_reward() {
        let mut gus = WeakSylvester::spawn_kind(SylvesterKind::Gus, 72, 100, 100, 1, 1);
        gus.spawn_ticks = 0;
        gus.chase_ticks = 0;
        gus.health = 4.0;
        let food = [AlienFoodView {
            id: 5,
            widget_x: 120,
            widget_y: 120,
            quality: 0,
            eligible: true,
        }];
        let prey = [PreyView {
            id: 6,
            widget_x: 140,
            widget_y: 140,
            width: 80,
            height: 80,
            eligible: true,
        }];
        let update = gus.update_with_food(&prey, &food, || 1);
        assert_eq!(update.food_eaten, Some((5, 4)));
        assert_eq!(update.prey_eaten, Some(6));
        assert!(update.defeated);
        assert_eq!(gus.health, -15.0);
        assert_eq!(gus.hit_ticks, 9);
    }

    #[test]
    fn gus_itchy_hit_is_quarter_health_and_survives_round_trip_until_active_update() {
        let mut gus = WeakSylvester::spawn_kind(SylvesterKind::Gus, 73, 100, 100, 1, 1);
        gus.health = 0.25;
        assert_eq!(gus.itchy_hit(), Some(0.0));
        assert!(gus.alive);
        let saved = serde_json::to_string(&gus).unwrap();
        let mut resumed: WeakSylvester = serde_json::from_str(&saved).unwrap();
        resumed.validate().unwrap();
        resumed.spawn_ticks = 0;
        assert!(resumed.update_with_food(&[], &[], || 1).defeated);
    }

    #[test]
    fn gus_eat_pose_keeps_velocity_facing_and_rejects_nonquarter_save_health() {
        let mut gus = WeakSylvester::spawn_kind(SylvesterKind::Gus, 75, 100, 100, 1, 1);
        gus.vx = -1.0;
        gus.turn_ticks = 4;
        gus.hit_ticks = 9;
        assert!(!gus.facing_right());
        assert_eq!(gus.sprite_row(), 2);
        gus.health = 99.9;
        assert!(gus.validate().is_err());
        gus.health = 99.75;
        gus.validate().unwrap();
    }

    #[test]
    fn destructor_first_launch_on_76_registers_before_reload_draw_and_sets_animation() {
        let mut actor = WeakSylvester::spawn_kind(SylvesterKind::Destructor, 76, 100, 120, 1, 1);
        assert_eq!((actor.widget_x, actor.widget_y, actor.y), (100, 280, 280.0));
        actor.spawn_ticks = 0;
        actor.launch_ticks = 75;
        let prey = [PreyView {
            id: 5,
            widget_x: 400,
            widget_y: 300,
            width: 80,
            height: 80,
            eligible: true,
        }];
        let mut requests = Vec::new();
        actor.update_with_runtime(&prey, &[], |request| {
            requests.push(request);
            match request {
                AlienRuntimeRequest::Launch { slot: 0, .. } => 1,
                AlienRuntimeRequest::Launch { .. } => 0,
                AlienRuntimeRequest::Random => 49,
                AlienRuntimeRequest::ProbeTarget { .. } => 0,
            }
        });
        assert!(matches!(
            requests.as_slice(),
            [
                AlienRuntimeRequest::Random, // due wander roll precedes launch
                AlienRuntimeRequest::Launch { slot: 0, .. },
                AlienRuntimeRequest::Launch { slot: 1, .. },
                AlienRuntimeRequest::Random,
            ]
        ));
        assert_eq!(
            (
                actor.launch_ticks,
                actor.reload_ticks,
                actor.special_ticks,
                actor.frame
            ),
            (0, 199, 9, 9)
        );
    }

    #[test]
    fn pending_rufus_quarter_kill_can_launch_before_destructor_final_removal() {
        let mut actor = WeakSylvester::spawn_kind(SylvesterKind::Destructor, 77, 100, 120, 1, 1);
        actor.spawn_ticks = 0;
        actor.health = 0.25;
        actor.launch_ticks = 75;
        assert_eq!(actor.rufus_hit(), Some(0.0));
        let prey = [PreyView {
            id: 5,
            widget_x: 400,
            widget_y: 300,
            width: 80,
            height: 80,
            eligible: true,
        }];
        let mut launched = false;
        let update = actor.update_with_runtime(&prey, &[], |request| match request {
            AlienRuntimeRequest::Launch { slot: 0, .. } => {
                launched = true;
                1
            }
            AlienRuntimeRequest::Launch { .. } => 0,
            AlienRuntimeRequest::Random => 0,
            AlienRuntimeRequest::ProbeTarget { .. } => 0,
        });
        assert!(launched && update.defeated);
    }

    #[test]
    fn destructor_never_chases_or_bites_and_special_freezes_wander_rng() {
        let mut actor = WeakSylvester::spawn_kind(SylvesterKind::Destructor, 78, 100, 120, 1, 1);
        actor.spawn_ticks = 0;
        actor.chase_ticks = 0;
        actor.special_ticks = 5;
        actor.movement_change_ticks = 20;
        let prey = [PreyView {
            id: 5,
            widget_x: 120,
            widget_y: 300,
            width: 80,
            height: 80,
            eligible: true,
        }];
        let update = actor.update_with_runtime(&prey, &[], |_| panic!("no random draw or launch"));
        assert_eq!(update.prey_eaten, None);
        assert_eq!(actor.movement_change_ticks, 20);
        assert_eq!(actor.target_vx, 1.0);
        assert_eq!((actor.y, actor.vy), (280.0, 0.0));
    }

    #[test]
    fn destructor_special_end_uses_twenty_clock_and_shot_push_is_horizontal_only() {
        let mut actor = WeakSylvester::spawn_kind(SylvesterKind::Destructor, 79, 100, 120, 1, 1);
        actor.spawn_ticks = 0;
        actor.vx = 2.0;
        actor.swim_ticks = 30;
        actor.special_ticks = 1;
        let old_x = actor.x;
        actor.animate();
        assert_eq!(
            (actor.special_ticks, actor.swim_ticks, actor.frame, actor.x),
            (0, 1, 0, old_x)
        );
        assert!(matches!(
            actor.shot_with_weapon(110, 290, 2),
            ShotResult::Hit { health: 144.0 }
        ));
        assert!((actor.vx - 3.6).abs() < 1e-9 && actor.vy == 0.0);
        actor.hit_ticks = 0;
        actor.special_ticks = 4;
        assert!(matches!(
            actor.shot_with_weapon(110, 290, 2),
            ShotResult::Hit { .. }
        ));
        assert!((actor.vx - 3.6).abs() < 1e-9 && actor.vy == 0.0);
        actor.health = 0.25;
        assert_eq!(actor.itchy_hit(), Some(0.0));
    }

    #[test]
    fn ulysses_prelaunch_probe_and_two_launches_precede_reload_rng() {
        let mut actor = WeakSylvester::spawn_kind(SylvesterKind::Ulysses, 7, 180, 200, 1, 1);
        assert_eq!(
            (actor.health, actor.movement_divisor, actor.reload_ticks),
            (220.0, 3.5, 75)
        );
        actor.spawn_ticks = 0;
        actor.launch_ticks = 59;
        let prey = [PreyView {
            id: 3,
            widget_x: 400,
            widget_y: 250,
            width: 80,
            height: 80,
            eligible: true,
        }];
        let mut calls = Vec::new();
        actor.update_with_runtime(&prey, &[], |request| {
            calls.push(request);
            match request {
                AlienRuntimeRequest::ProbeTarget { .. } => 1,
                _ => 0,
            }
        });
        assert!(
            calls
                .iter()
                .any(|call| matches!(call, AlienRuntimeRequest::ProbeTarget { .. }))
        );
        assert_eq!(actor.launch_ticks, 60);
        assert_eq!(actor.special_ticks, 39); // Animate follows prelaunch cue.
        actor.launch_ticks = 75;
        actor.special_ticks = 0;
        calls.clear();
        actor.update_with_runtime(&prey, &[], |request| {
            calls.push(request);
            match request {
                AlienRuntimeRequest::Launch { .. } => 1,
                _ => 0,
            }
        });
        let launch_slots: Vec<_> = calls
            .iter()
            .filter_map(|request| match request {
                AlienRuntimeRequest::Launch { slot, .. } => Some(*slot),
                _ => None,
            })
            .collect();
        assert_eq!(launch_slots, [0, 1]);
        assert!(matches!(calls.last(), Some(AlienRuntimeRequest::Random)));
        assert_eq!((actor.launch_ticks, actor.reload_ticks), (0, 150));
    }

    #[test]
    fn ordinary_ulysses_lane_wander_and_weapon_damage_do_not_use_prey_chase() {
        let mut actor = WeakSylvester::spawn_kind(SylvesterKind::Ulysses, 7, 100, 120, 1, 0);
        assert_eq!((actor.widget_y, actor.y), (280, 280.0));
        actor.spawn_ticks = 0;
        actor.chase_ticks = 0;
        let prey = [PreyView {
            id: 2,
            widget_x: 100,
            widget_y: 280,
            width: 80,
            height: 80,
            eligible: true,
        }];
        let update = actor.update(&prey, || 0);
        assert_eq!(update.prey_eaten, None);
        assert_eq!(
            (actor.target_vx, actor.target_vy, actor.vy),
            (-1.0, 0.0, 0.0)
        );
        actor.hit_ticks = 0;
        assert_eq!(
            actor.shot_with_weapon(110, 290, 1),
            ShotResult::Hit { health: 216.0 }
        );
        assert_eq!((actor.vx, actor.vy), (10.5, 0.0));
        actor.validate().unwrap();
        let mut protected = WeakSylvester::spawn_kind(SylvesterKind::Ulysses, 8, 100, 200, 1, 0);
        protected.special_ticks = 40;
        assert_eq!(
            protected.shot_with_weapon(110, 290, 12),
            ShotResult::Hit { health: 194.0 }
        );
        assert_eq!((protected.vx, protected.vy), (3.0, 0.0));
    }

    #[test]
    fn ulysses_pulse_uses_old_frame_and_special_timer_before_animation() {
        let pulse = |frame: u8, vx: f64, special_ticks: u8| {
            let mut actor = WeakSylvester::spawn_kind(SylvesterKind::Ulysses, 7, 100, 200, 1, 4);
            actor.spawn_ticks = 0;
            actor.movement_change_ticks = 0;
            actor.frame = frame;
            actor.vx = vx;
            actor.target_vx = vx;
            actor.special_ticks = special_ticks;
            actor.update(&[], || 0);
            (actor.vx, actor.special_ticks)
        };
        assert!((pulse(0, -1.0, 0).0 + 0.2).abs() < 1e-9);
        assert!((pulse(1, -1.0, 0).0 + 1.55).abs() < 1e-9);
        assert_eq!(pulse(3, -1.0, 0).0, -1.0);
        assert_eq!(pulse(1, 0.0, 0).0, 0.0);
        assert_eq!(pulse(1, 5.0, 1), (0.1, 0));
    }
}
