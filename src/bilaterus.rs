//! The two physical heads and six ordered bones of one Bilaterus encounter.
//!
//! PB61 confirms the installed constructor, emergence, health, head swap and
//! player-hit route. Motion, prey contact and targetless fragments below are
//! functional rules from pinned WinFish f919b3c; retail timing is unmeasured.
//! The Board supplies its ordered, freshly committed prey and shared RNG.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BilaterusPreyView {
    pub id: u64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub width: i32,
    pub height: i32,
    /// Includes live membership, eating delay, virtual status and (only for
    /// guppies) Wadsworth protection. Breeders are not ward-protected.
    pub eligible: bool,
    pub ultra: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FragmentKind {
    HeadFront,
    HeadBack,
    Bone,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FragmentSpawn {
    pub kind: FragmentKind,
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum BilaterusTransition {
    FirstHeadLost {
        fragment: FragmentSpawn,
    },
    Defeated {
        /// The group has one combat identity and can award only one diamond.
        diamond_at: (i32, i32),
        fragments: [FragmentSpawn; 7],
    },
}

#[derive(Clone, Debug, PartialEq)]
pub enum BilaterusShot {
    Miss,
    Hit {
        health: f64,
        transition: Option<BilaterusTransition>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BilaterusHead {
    pub x: f64,
    pub y: f64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub vx: f64,
    pub vy: f64,
    pub health: f64,
    pub hit_ticks: u8,
    pub bite_cooldown: u8,
    pub frame: u8,
    pub swim_ticks: u8,
    pub turn_ticks: i8,
    pub movement_state: u8,
    pub movement_ticks: u8,
    pub movement_vx: f64,
    pub movement_vy: f64,
    pub facing_velocity: f64,
    pub follow_vx: f64,
    pub follow_vy: f64,
    /// Tail position captured by BHUnk01 and refreshed during passive follow.
    pub follow_x: f64,
    pub follow_y: f64,
    pub follow_ticks: u8,
    /// DrawHead updates this only when the passive connector pose is visible.
    pub connector_left: bool,
    /// BHUnk01 suppresses the immediate direction-change animation for one move.
    pub turn_suppressed: bool,
    pub back: bool,
}

impl BilaterusHead {
    fn spawn(x: i32, y: i32, active: bool, random: &mut impl FnMut() -> u32) -> Self {
        let left = random().is_multiple_of(2);
        let movement_state = (random() % 10) as u8;
        Self {
            x: f64::from(x),
            y: f64::from(y),
            widget_x: x,
            widget_y: y,
            vx: if left { -0.1 } else { 0.0 },
            vy: -0.5,
            health: 100.0,
            hit_ticks: 0,
            bite_cooldown: 100,
            frame: 0,
            swim_ticks: 0,
            turn_ticks: 0,
            movement_state,
            movement_ticks: 0,
            movement_vx: 0.0,
            movement_vy: 0.0,
            facing_velocity: if left { -1.0 } else { 1.0 },
            follow_vx: 0.0,
            follow_vy: 0.0,
            follow_x: 0.0,
            follow_y: 0.0,
            follow_ticks: 12,
            connector_left: false,
            turn_suppressed: false,
            back: !active,
        }
    }

    pub fn facing_right(&self) -> bool {
        if self.turn_ticks != 0 {
            self.turn_ticks < 0
        } else if self.vx != 0.0 {
            self.vx > 0.0
        } else {
            self.facing_velocity > 0.0
        }
    }

    pub fn sprite_row(&self) -> u8 {
        (if self.back { 0 } else { 3 }) + if self.turn_ticks != 0 { 1 } else { 2 }
    }

    pub fn sprite_frame(&self) -> u8 {
        self.frame
    }

    fn animate(&mut self) {
        if self.turn_ticks == 0
            && self.vx != 0.0
            && self.facing_velocity.signum() != self.vx.signum()
            && !self.turn_suppressed
        {
            self.turn_ticks = if self.vx > 0.0 { -20 } else { 20 };
        }
        if self.turn_ticks > 0 {
            self.turn_ticks -= 1;
        } else if self.turn_ticks < 0 {
            self.turn_ticks += 1;
        }
        if self.turn_ticks == 0 {
            self.swim_ticks = if self.swim_ticks >= 19 {
                0
            } else {
                self.swim_ticks + 1
            };
            self.frame = if self.back {
                if self.swim_ticks < 10 {
                    9 - self.swim_ticks
                } else {
                    self.swim_ticks - 10
                }
            } else {
                self.swim_ticks / 2
            };
        } else {
            self.frame = if self.turn_ticks > 0 {
                9 - (self.turn_ticks as u8 / 2)
            } else {
                (self.turn_ticks / 2 + 9) as u8
            };
        }
        if self.vx != 0.0 {
            self.facing_velocity = self.vx;
        }
    }

    fn move_tail(&mut self) {
        self.animate();
        if self.x < 10.0 {
            self.x = 10.0;
            self.movement_vx = 1.5;
            self.vx = 0.0;
        }
        if self.x > 540.0 {
            self.x = 540.0;
            self.movement_vx = -1.5;
            self.vx = 0.0;
        }
        if self.y < 95.0 {
            self.y = 95.0;
            self.vy = 0.0;
        }
        if self.y > 370.0 {
            self.y = 370.0;
            self.vy = 0.0;
        }
        self.x += self.vx / 0.8;
        self.y += self.vy / 0.8;
        self.hit_ticks = self.hit_ticks.saturating_sub(1);
        self.bite_cooldown = self.bite_cooldown.saturating_sub(1);
        self.widget_x = self.x as i32;
        self.widget_y = self.y as i32;
        self.turn_suppressed = false;
    }

    fn contact(&self, prey: &[BilaterusPreyView]) -> Option<u64> {
        let (cx, cy) = (self.widget_x + 40, self.widget_y + 40);
        prey.iter()
            .find(|p| {
                let radius = if p.ultra { 70 } else { 30 };
                let dx = cx - (p.widget_x + p.width / 2);
                let dy = cy - (p.widget_y + p.height / 2);
                p.eligible && dx > -radius && dx < radius && dy > -radius && dy < radius
            })
            .map(|p| p.id)
    }

    fn validate(&self) -> Result<(), String> {
        let finite = [
            self.x,
            self.y,
            self.vx,
            self.vy,
            self.health,
            self.movement_vx,
            self.movement_vy,
            self.facing_velocity,
            self.follow_vx,
            self.follow_vy,
            self.follow_x,
            self.follow_y,
        ]
        .iter()
        .all(|v| v.is_finite());
        // A pet can reduce a registered head below zero before the next group
        // update, including during its 15 emergence ticks. The Board validates
        // the live pet source separately; do not reject this pending state.
        if !finite
            || self.health < -30.0
            || self.health > 100.0
            || self.hit_ticks > 10
            || self.bite_cooldown > 100
            || self.frame > 9
            || self.swim_ticks > 19
            || self.turn_ticks.unsigned_abs() > 19
            || self.movement_state > 9
            || self.movement_ticks > 20
            || self.follow_ticks > 12
        {
            return Err("invalid Bilaterus head state".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BilaterusBone {
    pub x: f64,
    pub y: f64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub vx: f64,
    pub vy: f64,
    pub facing_velocity: f64,
    pub bite_cooldown: u8,
    pub follow_ticks: u8,
    pub follow_vx: f64,
    pub follow_vy: f64,
}

impl BilaterusBone {
    fn spawn(x: i32, y: i32, random: &mut impl FnMut() -> u32) -> Self {
        let left = random().is_multiple_of(2);
        Self {
            x: f64::from(x),
            y: f64::from(y),
            widget_x: x,
            widget_y: y,
            vx: if left { -0.1 } else { 0.0 },
            vy: -0.5,
            facing_velocity: if left { -1.0 } else { 1.0 },
            bite_cooldown: 100,
            follow_ticks: 12,
            follow_vx: 0.0,
            follow_vy: 0.0,
        }
    }

    pub fn facing_right(&self) -> bool {
        self.vx > 0.0 || self.vx == 0.0 && self.facing_velocity > 0.0
    }
    pub fn sprite_row(index: usize) -> u8 {
        6 + (index % 2) as u8
    }
    pub fn sprite_frame(&self, predecessor_x: f64) -> u8 {
        let dx = self.x - predecessor_x;
        if dx > -30.0 && dx < 36.0 {
            (5 - (dx / 6.0) as i32).clamp(0, 9) as u8
        } else {
            0
        }
    }

    fn update(
        &mut self,
        index: usize,
        previous: (f64, f64, f64, f64),
        prey: &[BilaterusPreyView],
    ) -> Option<u64> {
        let (px, py, pvx, pvy) = previous;
        self.follow_ticks += 1;
        if self.follow_ticks >= 12 {
            self.follow_ticks = 0;
            self.follow_vx = pvx;
            self.follow_vy = pvy;
        }
        if self.vx < self.follow_vx {
            self.vx += 0.02;
        }
        if self.vx > self.follow_vx {
            self.vx -= 0.02;
        }
        if self.vy < self.follow_vy {
            self.vy += 0.02;
        }
        if self.vy > self.follow_vy {
            self.vy -= 0.02;
        }
        if self.vx != 0.0 {
            self.facing_velocity = self.vx;
        }
        let max_dx = if index == 0 { 40.0 } else { 30.0 };
        self.x = self.x.clamp(px - max_dx, px + max_dx).clamp(10.0, 540.0);
        self.y = self.y.clamp(py - 20.0, py + 20.0).clamp(95.0, 370.0);
        self.x += self.vx / 0.8;
        self.y += self.vy / 0.8;
        self.bite_cooldown = self.bite_cooldown.saturating_sub(1);
        self.widget_x = self.x as i32;
        self.widget_y = self.y as i32;
        if self.bite_cooldown > 0 {
            return None;
        }
        let (cx, cy) = (self.widget_x + 40, self.widget_y + 40);
        prey.iter()
            .find(|p| {
                let radius = if p.ultra { 70 } else { 30 };
                let dx = cx - (p.widget_x + p.width / 2);
                let dy = cy - (p.widget_y + p.height / 2);
                p.eligible && dx > -radius && dx < radius && dy > -radius && dy < radius
            })
            .map(|p| p.id)
    }

    fn validate(&self) -> Result<(), String> {
        if ![
            self.x,
            self.y,
            self.vx,
            self.vy,
            self.facing_velocity,
            self.follow_vx,
            self.follow_vy,
        ]
        .iter()
        .all(|v| v.is_finite())
            || self.bite_cooldown > 100
            || self.follow_ticks > 12
        {
            return Err("invalid Bilaterus bone state".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BilaterusState {
    pub id: u64,
    pub heads: [Option<BilaterusHead>; 2],
    pub active_head: usize,
    /// This order is physical head-to-tail and reverses at each head swap.
    pub bones: [BilaterusBone; 6],
    pub emergence_ticks: u8,
    pub swap_ticks: i32,
    pub first_head_lost: bool,
    pub widget_x: i32,
    pub widget_y: i32,
}

impl BilaterusState {
    pub fn spawn(id: u64, x: i32, y: i32, random: &mut impl FnMut() -> u32) -> Self {
        let heads = [
            Some(BilaterusHead::spawn(x, y, true, random)),
            Some(BilaterusHead::spawn(x, y, false, random)),
        ];
        let bones = std::array::from_fn(|_| BilaterusBone::spawn(x, y, random));
        let mut state = Self {
            id,
            heads,
            active_head: 0,
            bones,
            emergence_ticks: 15,
            swap_ticks: 0,
            first_head_lost: false,
            widget_x: x,
            widget_y: y,
        };
        if random().is_multiple_of(2) {
            state.swap_heads();
        }
        state
    }

    pub fn active(&self) -> &BilaterusHead {
        self.heads[self.active_head].as_ref().expect("active head")
    }
    pub fn active_mut(&mut self) -> &mut BilaterusHead {
        self.heads[self.active_head].as_mut().expect("active head")
    }
    pub fn passive(&self) -> Option<&BilaterusHead> {
        self.heads[1 - self.active_head].as_ref()
    }
    pub fn active_head_contains_widget(&self, x: i32, y: i32) -> bool {
        let head = self.active();
        x >= head.widget_x && x < head.widget_x + 80 && y >= head.widget_y && y < head.widget_y + 80
    }
    pub fn active_head_position(&self) -> (i32, i32) {
        let head = self.active();
        (head.widget_x, head.widget_y)
    }

    /// Source draws no head before the final six emergence counts. The
    /// active head is compressed more strongly during the remaining stretch.
    pub fn head_emergence_scale(&self, physical_index: usize) -> Option<f32> {
        if self.emergence_ticks >= 7 {
            return None;
        }
        if self.emergence_ticks == 0 {
            return Some(1.0);
        }
        let ratio = f32::from(self.emergence_ticks) / 7.0;
        let shrink = if physical_index == self.active_head {
            (ratio * 0.8).powi(2)
        } else {
            ratio
        };
        Some(1.0 - shrink)
    }

    /// Passive head uses the first bone as a connector pose while close
    /// enough; its own swim/turn frame applies outside that interval.
    pub fn head_sprite_pose(&self, physical_index: usize) -> Option<(u8, u8, bool)> {
        let head = self.heads.get(physical_index)?.as_ref()?;
        if physical_index != self.active_head {
            let dx = head.x - self.bones[0].x;
            if dx > -30.0 && dx < 36.0 {
                return Some((
                    if head.back { 0 } else { 3 },
                    (5 - (dx / 6.0) as i32).clamp(0, 9) as u8,
                    false,
                ));
            }
        }
        Some((
            head.sprite_row(),
            head.sprite_frame(),
            if physical_index == self.active_head {
                head.facing_right()
            } else {
                !head.connector_left
            },
        ))
    }

    /// The original connector direction is updated by passive-head drawing,
    /// not by Update. Call only when that rendered pose was observed; saving
    /// the resulting flag preserves the next swap's velocity sign.
    pub fn observe_rendered_connectors(&mut self) -> bool {
        let passive_index = 1 - self.active_head;
        let Some(head) = self.heads[passive_index].as_mut() else {
            return false;
        };
        let dx = head.x - self.bones[0].x;
        if dx <= -30.0 || dx >= 36.0 {
            return false;
        }
        let connector_left = dx < 0.0;
        let changed = head.connector_left != connector_left;
        head.connector_left = connector_left;
        changed
    }

    fn swap_heads(&mut self) {
        if self.heads.iter().any(Option::is_none) {
            return;
        }
        self.active_head = 1 - self.active_head;
        // Source m0x160 is a physical sheet/fragment identity, independent of
        // which physical head currently leads the group.
        for head in self.heads.iter_mut().flatten() {
            head.turn_ticks = -head.turn_ticks;
            head.vx = 0.0;
        }
        self.bones.reverse();
        self.reorient_head(self.active_head);
        self.refresh_bone_follow();
        self.reorient_head(1 - self.active_head);
    }

    fn reorient_head(&mut self, physical_index: usize) {
        let tail = &self.bones[5];
        let (tail_x, tail_y, tail_vx, tail_vy) = (tail.x, tail.y, tail.vx, tail.vy);
        let head = self.heads[physical_index]
            .as_mut()
            .expect("swap retains both heads");
        // BHUnk01 reads the previously rendered connector flag before it
        // refreshes the tail relation. The refreshed flag governs later draws.
        head.vx = if head.connector_left { -1.5 } else { 1.5 };
        head.follow_x = tail_x;
        head.follow_y = tail_y;
        head.follow_vx = tail_vx;
        head.follow_vy = tail_vy;
        head.connector_left = head.x < tail_x;
        head.turn_suppressed = true;
    }

    fn refresh_bone_follow(&mut self) {
        for index in 0..5 {
            let (vx, vy) = if index == 0 {
                let head = self.active();
                (head.vx, head.vy)
            } else {
                (self.bones[index - 1].vx, self.bones[index - 1].vy)
            };
            let bone = &mut self.bones[index];
            bone.follow_vx = vx;
            bone.follow_vy = vy;
            bone.vx = 0.0;
        }
    }

    /// Called once before children. A pet-lethal head remains registered until
    /// `finish_update`, even while emergence is counting down.
    pub fn begin_update(&mut self) -> bool {
        self.begin_update_with_swap().0
    }

    /// The swap happens before the active head's update, and its sound/event
    /// therefore precedes any child prey transaction on this board tick.
    pub fn begin_update_with_swap(&mut self) -> (bool, bool) {
        if self.emergence_ticks > 0 {
            self.emergence_ticks -= 1;
            return (false, false);
        }
        // W1 Bilaterus.h declares this counter as int. It continues advancing
        // after the first head is lost, when periodic swapping is disabled.
        self.swap_ticks = self.swap_ticks.wrapping_add(1);
        let mut swapped = false;
        if !self.first_head_lost && self.swap_ticks > 999 {
            self.swap_heads();
            self.swap_ticks = 0;
            swapped = true;
        }
        (true, swapped)
    }

    pub fn update_active(
        &mut self,
        prey: &[BilaterusPreyView],
        random: &mut impl FnMut() -> u32,
    ) -> Option<u64> {
        let (eaten, widget_x, widget_y) = {
            let head = self.active_mut();
            let nearest = prey
                .iter()
                .filter(|p| p.eligible)
                .min_by_key(|p| {
                    let dx = head.widget_x + 40 - p.width / 2 - p.widget_x;
                    let dy = head.widget_y + 40 - p.height / 2 - p.widget_y;
                    i64::from(dx) * i64::from(dx) + i64::from(dy) * i64::from(dy)
                })
                .copied();
            let eaten = if let Some(target) = nearest {
                if head.x + 40.0 < f64::from(target.widget_x + 40) && head.vx < 1.8 {
                    head.vx += 0.1;
                } else if head.x + 40.0 > f64::from(target.widget_x + 40) && head.vx > -1.8 {
                    head.vx -= 0.1;
                }
                if head.y + 40.0 < f64::from(target.widget_y + 40) && head.vy < 1.8 {
                    head.vy += 0.1;
                } else if head.y + 40.0 > f64::from(target.widget_y + 40) && head.vy > -1.8 {
                    head.vy -= 0.1;
                }
                if head.bite_cooldown == 0 {
                    head.contact(prey)
                } else {
                    None
                }
            } else {
                match head.movement_state {
                    0 => head.movement_vx = -1.5,
                    1 => head.movement_vx = 1.5,
                    2 => head.movement_vy = -1.5,
                    3 => head.movement_vy = 1.5,
                    _ => {}
                }
                if head.vx > head.movement_vx {
                    head.vx -= 0.1;
                } else if head.vx < head.movement_vx {
                    head.vx += 0.1;
                }
                if head.vy > head.movement_vy {
                    head.vy -= 0.1;
                } else if head.vy < head.movement_vy {
                    head.vy += 0.1;
                }
                head.movement_ticks += 1;
                if head.movement_ticks > 20 {
                    head.movement_ticks = 0;
                    if random().is_multiple_of(5) {
                        head.movement_state = (random() % 4) as u8;
                    }
                }
                None
            };
            head.move_tail();
            (eaten, head.widget_x, head.widget_y)
        };
        self.widget_x = widget_x;
        self.widget_y = widget_y;
        eaten
    }

    pub fn update_bone(&mut self, index: usize, prey: &[BilaterusPreyView]) -> Option<u64> {
        if index >= 6 {
            return None;
        }
        let prev = if index == 0 {
            let h = self.active();
            (h.x, h.y, h.vx, h.vy)
        } else {
            let b = &self.bones[index - 1];
            (b.x, b.y, b.vx, b.vy)
        };
        self.bones[index].update(index, prev, prey)
    }

    pub fn update_passive(&mut self) {
        let tail = &self.bones[5];
        let (tx, ty, tvx, tvy) = (tail.x, tail.y, tail.vx, tail.vy);
        if let Some(head) = self.heads[1 - self.active_head].as_mut() {
            head.follow_ticks += 1;
            if head.follow_ticks >= 12 {
                head.follow_ticks = 0;
                head.follow_vx = tvx;
                head.follow_vy = tvy;
            }
            if head.vx < head.follow_vx {
                head.vx += 0.02;
            }
            if head.vx > head.follow_vx {
                head.vx -= 0.02;
            }
            if head.vy < head.follow_vy {
                head.vy += 0.02;
            }
            if head.vy > head.follow_vy {
                head.vy -= 0.02;
            }
            head.follow_x = tx;
            head.follow_y = ty;
            head.x = head.x.clamp(head.follow_x - 40.0, head.follow_x + 40.0);
            head.y = head.y.clamp(head.follow_y - 20.0, head.follow_y + 20.0);
            head.move_tail();
        }
    }

    pub fn pet_damage(&mut self, amount: f64) -> f64 {
        let head = self.active_mut();
        head.health -= amount;
        head.health
    }

    /// PB71: Gash calls the head-death route immediately, including emergence.
    /// Itchy/Rufus retain their existing deferred update path.
    pub fn gash_hit(&mut self) -> (f64, Option<BilaterusTransition>) {
        let health = self.pet_damage(3.0);
        let transition = (health <= 0.0).then(|| self.transition());
        (health, transition)
    }

    pub fn shoot(&mut self, x: i32, y: i32, weapon: u8) -> BilaterusShot {
        let head = self.active_mut();
        if head.hit_ticks > 0
            || !(f64::from(x) > head.x
                && f64::from(x) < head.x + 80.0
                && f64::from(y) > head.y
                && f64::from(y) < head.y + 80.0)
        {
            return BilaterusShot::Miss;
        }
        let dx = f64::from(x) - head.x;
        let dy = f64::from(y) - head.y;
        if dx < 30.0 && dy < 30.0 {
            head.vx = 2.8;
            head.vy = 2.8;
        } else if dx < 30.0 && dy < 50.0 {
            head.vx = 3.2;
        } else if dx < 30.0 {
            head.vx = 2.8;
            head.vy = -2.8;
        } else if dx < 50.0 && dy < 30.0 {
            head.vy = 3.2;
        } else if dx > 50.0 && dy > 50.0 {
            head.vx = -2.8;
            head.vy = -2.8;
        } else if dx > 50.0 && dy < 30.0 {
            head.vx = -2.8;
            head.vy = 2.8;
        } else if dx > 50.0 {
            head.vx = -3.2;
        } else if dy > 50.0 {
            head.vy = -3.2;
        }
        head.health -= f64::from(weapon) * 2.0 + 2.0;
        head.hit_ticks = 10;
        let health = head.health;
        let transition = if health <= 0.0 {
            Some(self.transition())
        } else {
            None
        };
        BilaterusShot::Hit { health, transition }
    }

    pub fn finish_update(&mut self) -> Option<BilaterusTransition> {
        (self.emergence_ticks == 0 && self.active().health <= 0.0).then(|| self.transition())
    }

    fn transition(&mut self) -> BilaterusTransition {
        let head = self.active();
        let fragment = FragmentSpawn {
            kind: if head.back {
                FragmentKind::HeadBack
            } else {
                FragmentKind::HeadFront
            },
            x: head.widget_x,
            y: head.widget_y,
        };
        if !self.first_head_lost {
            self.swap_heads();
            self.first_head_lost = true;
            self.heads[1 - self.active_head] = None;
            BilaterusTransition::FirstHeadLost { fragment }
        } else {
            let mut fragments = [fragment; 7];
            for (slot, bone) in fragments[..6].iter_mut().zip(&self.bones) {
                *slot = FragmentSpawn {
                    kind: FragmentKind::Bone,
                    x: bone.widget_x,
                    y: bone.widget_y,
                };
            }
            fragments[6] = fragment;
            BilaterusTransition::Defeated {
                diamond_at: (self.widget_x + 25, self.widget_y + 25),
                fragments,
            }
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        let count = self.heads.iter().filter(|h| h.is_some()).count();
        if self.id == 0
            || self.active_head > 1
            || self.heads[self.active_head].is_none()
            || self.first_head_lost != (count == 1)
            || self.emergence_ticks > 15
            || (!self.first_head_lost && !(0..=999).contains(&self.swap_ticks))
            || count == 0
            || self.heads[0].as_ref().is_some_and(|h| h.back)
            || self.heads[1].as_ref().is_some_and(|h| !h.back)
        {
            return Err("invalid Bilaterus group state".into());
        }
        for head in self.heads.iter().flatten() {
            head.validate()?;
        }
        for bone in &self.bones {
            bone.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BilaterusFragment {
    pub id: u64,
    pub kind: FragmentKind,
    pub x: f64,
    pub y: f64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub vx: f64,
    pub vy: f64,
    pub frame: u8,
    pub loop_count: u8,
}

impl BilaterusFragment {
    pub fn spawn(id: u64, spec: FragmentSpawn, random: &mut impl FnMut() -> u32) -> Self {
        // W1 Missle constructor uses C Rand()%4 here, separate from the game
        // mSeed stream. Its value has no known targetless visual consumer.
        // This project-owned fallback is isolated from gameplay RNG; callers
        // may supply a distinct visual source via `spawn_with_visual_seed`.
        Self::spawn_with_visual_seed(id, spec, id as u32, random)
    }
    pub fn spawn_with_visual_seed(
        id: u64,
        spec: FragmentSpawn,
        _visual_seed: u32,
        random: &mut impl FnMut() -> u32,
    ) -> Self {
        const X_SPEED: [f64; 6] = [-6.0, -5.0, -4.0, 4.0, 5.0, 6.0];
        const Y_SPEED: [f64; 6] = [-6.0, -5.0, -4.0, 4.0, 4.0, 4.0];
        let vx = X_SPEED[(random() % 6) as usize];
        let vy = Y_SPEED[(random() % 6) as usize];
        Self {
            id,
            kind: spec.kind,
            x: f64::from(spec.x),
            y: f64::from(spec.y),
            widget_x: spec.x,
            widget_y: spec.y,
            vx,
            vy,
            frame: 1,
            loop_count: 1,
        }
    }
    pub fn sprite_frame(&self) -> u8 {
        self.frame
    }
    pub fn sprite_row(&self) -> u8 {
        match self.kind {
            FragmentKind::Bone => 6,
            FragmentKind::HeadFront => 4,
            FragmentKind::HeadBack => 1,
        }
    }
    pub fn facing_right(&self) -> bool {
        self.loop_count.is_multiple_of(2)
    }
    /// Returns false once the pre-movement position is outside source bounds.
    pub fn tick(&mut self) -> bool {
        if self.x > 580.0 || self.x < -20.0 || self.y > 380.0 || self.y < 45.0 {
            return false;
        }
        self.x += self.vx / 0.8;
        self.y += self.vy / 0.8;
        self.widget_x = self.x as i32;
        self.widget_y = self.y as i32;
        self.frame = (self.frame + 1) % 10;
        if self.frame == 0 {
            self.loop_count = self.loop_count.wrapping_add(1);
        }
        true
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0
            || self.frame > 9
            || ![self.x, self.y, self.vx, self.vy]
                .iter()
                .all(|v| v.is_finite())
        {
            Err("invalid Bilaterus targetless fragment".into())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn group() -> BilaterusState {
        // Odd final bit retains the original physical leading head.
        BilaterusState::spawn(41, 100, 120, &mut || 1)
    }

    #[test]
    fn constructor_keeps_two_physical_heads_and_consumes_bone_and_swap_draws() {
        let mut draws = 0;
        let group = BilaterusState::spawn(41, 100, 120, &mut || {
            draws += 1;
            1
        });
        assert_eq!(draws, 11); // two head pairs, six bones, one lead bit
        assert_eq!(group.active_head, 0);
        assert_eq!(group.heads.iter().filter(|head| head.is_some()).count(), 2);
        assert!(!group.heads[0].as_ref().unwrap().back);
        assert!(group.heads[1].as_ref().unwrap().back);
        assert_eq!(group.emergence_ticks, 15);
        group.validate().unwrap();
    }

    #[test]
    fn strict_active_geometry_immunity_and_two_deaths_award_only_final_group() {
        let mut group = group();
        assert_eq!(group.shoot(100, 150, 12), BilaterusShot::Miss);
        assert_eq!(group.shoot(180, 150, 12), BilaterusShot::Miss);
        assert_eq!(
            group.shoot(120, 140, 12),
            BilaterusShot::Hit {
                health: 74.0,
                transition: None
            }
        );
        assert_eq!(group.shoot(120, 140, 12), BilaterusShot::Miss);
        group.active_mut().hit_ticks = 0;
        group.active_mut().health = 26.0;
        let BilaterusShot::Hit {
            transition: Some(BilaterusTransition::FirstHeadLost { fragment }),
            ..
        } = group.shoot(120, 140, 12)
        else {
            panic!("first head must be removed immediately")
        };
        assert_eq!(fragment.kind, FragmentKind::HeadFront);
        assert_eq!(group.heads.iter().filter(|head| head.is_some()).count(), 1);
        assert_eq!(group.active_head, 1);
        group.active_mut().health = 26.0;
        let BilaterusShot::Hit {
            transition:
                Some(BilaterusTransition::Defeated {
                    diamond_at,
                    fragments,
                }),
            ..
        } = group.shoot(120, 140, 12)
        else {
            panic!("second click targets promoted head")
        };
        assert_eq!(diamond_at, (125, 145));
        assert_eq!(fragments.len(), 7);
        assert_eq!(fragments[6].kind, FragmentKind::HeadBack);
    }

    #[test]
    fn pet_lethal_head_still_runs_active_then_each_bone_before_group_check() {
        let mut group = group();
        group.emergence_ticks = 0;
        group.active_mut().bite_cooldown = 0;
        group.bones[0].bite_cooldown = 0;
        assert_eq!(group.pet_damage(101.0), -1.0);
        assert!(group.begin_update());
        let first = [BilaterusPreyView {
            id: 10,
            widget_x: 100,
            widget_y: 120,
            width: 80,
            height: 80,
            eligible: true,
            ultra: false,
        }];
        assert_eq!(group.update_active(&first, &mut || 1), Some(10));
        // Board has committed prey 10; this second view contains only prey 11.
        let next = [BilaterusPreyView { id: 11, ..first[0] }];
        assert_eq!(group.update_bone(0, &next), Some(11));
        for index in 1..6 {
            group.update_bone(index, &[]);
        }
        group.update_passive();
        assert!(matches!(
            group.finish_update(),
            Some(BilaterusTransition::FirstHeadLost { .. })
        ));
        assert!(group.first_head_lost);
    }

    #[test]
    fn surviving_one_head_counter_continues_past_swap_and_u16_boundaries() {
        let mut group = group();
        group.emergence_ticks = 0;
        group.active_mut().health = 26.0;
        assert!(matches!(
            group.shoot(120, 140, 12),
            BilaterusShot::Hit {
                transition: Some(BilaterusTransition::FirstHeadLost { .. }),
                ..
            }
        ));
        group.swap_ticks = 999;
        assert_eq!(group.begin_update_with_swap(), (true, false));
        assert_eq!(group.swap_ticks, 1000);
        group.validate().unwrap();
        group.swap_ticks = 65535;
        assert_eq!(group.begin_update_with_swap(), (true, false));
        assert_eq!(group.swap_ticks, 65536);
        let restored: BilaterusState =
            serde_json::from_value(serde_json::to_value(&group).unwrap()).unwrap();
        assert_eq!(restored.swap_ticks, 65536);
        restored.validate().unwrap();
    }

    #[test]
    fn periodic_swap_can_leave_pet_lethal_head_passive_until_later_turn() {
        // A pet subtracts HP without Die; the group swaps before checking
        // active-head death. The lethal former leader becomes passive.
        let mut group = group();
        group.emergence_ticks = 0;
        group.swap_ticks = 999;
        assert_eq!(group.pet_damage(100.25), -0.25);
        assert_eq!(group.begin_update_with_swap(), (true, true));
        assert_eq!(group.passive().unwrap().health, -0.25);
        group.validate().unwrap();
        let restored: BilaterusState =
            serde_json::from_value(serde_json::to_value(&group).unwrap()).unwrap();
        restored.validate().unwrap();
        assert!(restored.active().health > 0.0);
    }

    #[test]
    fn observed_passive_connector_changes_only_inside_strict_draw_band() {
        let mut group = group();
        group.bones[0].x = 100.0;
        group.heads[1].as_mut().unwrap().x = 70.0; // exact -30 boundary
        assert!(!group.observe_rendered_connectors());
        assert!(!group.passive().unwrap().connector_left);
        group.heads[1].as_mut().unwrap().x = 70.001;
        assert!(group.observe_rendered_connectors());
        assert!(group.passive().unwrap().connector_left);
        assert!(!group.observe_rendered_connectors());
        group.heads[1].as_mut().unwrap().x = 136.0; // exact +36 boundary
        assert!(!group.observe_rendered_connectors());
        assert!(group.passive().unwrap().connector_left);
        group.heads[1].as_mut().unwrap().x = 135.999;
        assert!(group.observe_rendered_connectors());
        assert!(!group.passive().unwrap().connector_left);
        let restored: BilaterusState =
            serde_json::from_value(serde_json::to_value(&group).unwrap()).unwrap();
        assert!(!restored.passive().unwrap().connector_left);
    }

    #[test]
    fn swap_and_first_death_use_prior_connector_sign_then_snapshot_tail() {
        let mut group = group();
        group.heads[0].as_mut().unwrap().connector_left = true;
        group.heads[1].as_mut().unwrap().connector_left = false;
        group.heads[1].as_mut().unwrap().facing_velocity = -1.0;
        group.bones[0].x = 128.0;
        group.bones[0].y = 151.0;
        group.bones[0].vx = 0.4;
        group.bones[0].vy = -0.3;
        group.swap_heads();
        assert_eq!(group.active().vx, 1.5); // old false, before new tail comparison
        assert_eq!(group.passive().unwrap().vx, -1.5); // old true
        assert_eq!(
            (group.active().follow_x, group.active().follow_y),
            (128.0, 151.0)
        );
        assert!(group.active().turn_suppressed);
        let mut without_suppression = group.active().clone();
        without_suppression.turn_suppressed = false;
        without_suppression.move_tail();
        assert_eq!(without_suppression.turn_ticks, -19);
        group.active_mut().move_tail();
        assert_eq!(group.active().turn_ticks, 0);
        assert!(!group.active().turn_suppressed);
        group.validate().unwrap();

        let mut lethal = self::group();
        lethal.heads[0].as_mut().unwrap().connector_left = true;
        lethal.heads[1].as_mut().unwrap().connector_left = false;
        lethal.active_mut().health = 26.0;
        assert!(matches!(
            lethal.shoot(120, 140, 12),
            BilaterusShot::Hit {
                transition: Some(BilaterusTransition::FirstHeadLost { .. }),
                ..
            }
        ));
        assert_eq!(lethal.active().vx, 1.5);
        assert!(lethal.active().turn_suppressed);
        lethal.validate().unwrap();
    }

    #[test]
    fn fragment_is_visual_only_and_uses_pre_move_offscreen_removal() {
        let mut draws = [3, 5].into_iter();
        let mut fragment = BilaterusFragment::spawn(
            9,
            FragmentSpawn {
                kind: FragmentKind::Bone,
                x: 579,
                y: 100,
            },
            &mut || draws.next().unwrap(),
        );
        assert_eq!((fragment.vx, fragment.vy), (4.0, 4.0));
        assert!(fragment.tick());
        assert!(fragment.x > 580.0);
        assert!(!fragment.tick());
        assert_eq!(draws.next(), None);
    }
}
