//! Adventure shell bonuses. W1 f919b3c supplies the rules; PB25/PB26 confirm
//! flight arithmetic and Board timing in the installed payload. The controlled
//! project PRNG does not reproduce the original's separate random streams.

use crate::sim::{AdventureState, TICK_MS};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const MAX_SHELL_BALANCE: u32 = 9_999_999;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShellKind {
    Silver,
    Gold,
    Diamond,
    Pearl,
    Treasure,
}

impl ShellKind {
    pub fn base_value(self) -> u32 {
        match self {
            Self::Silver => 1,
            Self::Gold => 2,
            Self::Diamond => 5,
            Self::Pearl => 10,
            Self::Treasure => 20,
        }
    }

    fn from_roll(roll: u64) -> Self {
        match roll {
            0..=19 => Self::Silver,
            20..=29 => Self::Treasure,
            30..=49 => Self::Pearl,
            50..=79 => Self::Diamond,
            _ => Self::Gold,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShellPhase {
    Falling,
    Collecting {
        origin_x: i32,
        origin_y: i32,
        updates: u8,
        multiplier: u8,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShellState {
    pub id: u64,
    pub kind: ShellKind,
    pub x: f64,
    pub y: f64,
    pub vy: f64,
    pub animation_tick: u8,
    pub fade_ticks: u8,
    pub phase: ShellPhase,
}

impl ShellState {
    pub fn sprite_frame(&self) -> u8 {
        (self.animation_tick / 2) % 20
    }

    pub fn alpha(&self) -> f32 {
        if self.fade_ticks == 0 {
            1.0
        } else {
            f32::from(self.fade_ticks) / 5.0
        }
    }

    fn contains(&self, x: f32, y: f32) -> bool {
        let left = self.x.trunc() as f32;
        let top = self.y.trunc() as f32;
        self.phase == ShellPhase::Falling
            && x >= left
            && x < left + 72.0
            && y >= top
            && y < top + 72.0
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BonusEvent {
    Started {
        manual: bool,
    },
    Dropped {
        id: u64,
        kind: ShellKind,
        x: i32,
        y: i32,
    },
    Claimed {
        id: u64,
        kind: ShellKind,
        multiplier: u8,
    },
    Credited {
        id: u64,
        value: u32,
        total: u32,
    },
    Expired {
        id: u64,
    },
    TimedOut,
}

pub struct BonusUpdate {
    pub events: Vec<BonusEvent>,
    pub completed: bool,
}

/// The profile award is committed by the Adventure session on results entry.
/// This record only drives the count-up shown after that commit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PurchaseReceipt {
    pub offered_cursor: u8,
    pub confirming: bool,
    pub purchased: bool,
}

impl PurchaseReceipt {
    pub fn new(offered_cursor: u8) -> Self {
        Self {
            offered_cursor,
            confirming: false,
            purchased: false,
        }
    }

    pub fn validate(&self, updates: u32) -> Result<(), String> {
        if self.offered_cursor > 5
            || (self.confirming && (self.purchased || self.offered_cursor >= 5 || updates < 30))
            || (self.purchased && (self.offered_cursor >= 5 || updates < 30))
        {
            return Err("invalid results purchase receipt".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BonusResult {
    /// Completed bonus identity, retained after the profile advances.
    pub origin_tank: u8,
    pub origin_level: u8,
    pub earned: u32,
    pub previous_balance: u32,
    pub updates: u32,
    pub purchase: PurchaseReceipt,
}

impl BonusResult {
    pub fn presented_balance(&self) -> u32 {
        let mut step = 50;
        let initial_steps = self.earned / step;
        if initial_steps < 15 {
            step = self.earned / 15;
        } else if initial_steps > 50 {
            step = self.earned / 50;
        }
        step = step.max(50);
        let counted = self
            .updates
            .saturating_sub(30)
            .saturating_mul(step)
            .min(self.earned);
        self.previous_balance
            .saturating_add(counted)
            .min(MAX_SHELL_BALANCE)
    }

    pub fn validate(&self) -> Result<(), String> {
        if !matches!(
            (self.origin_tank, self.origin_level),
            (1..=3, 1..=6) | (4, 1..=5) | (5, 1)
        ) || self.previous_balance > MAX_SHELL_BALANCE
            || self.earned > MAX_SHELL_BALANCE
        {
            return Err("invalid bonus result balance or award".into());
        }
        self.purchase.validate(self.updates)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BonusState {
    /// Immutable scenario identity; duration/background derive from this field.
    pub origin_tank: u8,
    pub origin_level: u8,
    pub tick: u64,
    pub initial_count: u64,
    pub started_at: Option<u64>,
    pub timed_out: bool,
    pub empty_updates: u16,
    pub shells: Vec<ShellState>,
    /// Forward widget draw/update order from the last Board sort. Inputs use
    /// its reverse, even if an earlier click in the batch changes last_type.
    pub draw_order: Vec<u64>,
    pub sorted_type: Option<ShellKind>,
    pub last_type: Option<ShellKind>,
    pub combo_count: u8,
    pub shells_earned: u32,
    next_id: u64,
    rng_state: u64,
}

impl BonusState {
    pub fn new(seed: u64) -> Self {
        Self::new_for_tank(seed, 1).expect("tank-one bonus is supported")
    }

    pub fn new_for_tank(seed: u64, origin_tank: u8) -> Result<Self, String> {
        Self::new_for_stage(AdventureState::initial_rng(seed), 1, origin_tank, 6)
    }

    pub fn new_for_stage(
        seed: u64,
        next_id: u64,
        origin_tank: u8,
        origin_level: u8,
    ) -> Result<Self, String> {
        if !matches!((origin_tank, origin_level), (1..=3, 1..=6) | (4, 1..=5))
            || next_id == 0
            || next_id == u64::MAX
        {
            return Err("unsupported bonus origin tank".into());
        }
        Ok(Self {
            origin_tank,
            origin_level,
            tick: 0,
            initial_count: 0,
            started_at: None,
            timed_out: false,
            empty_updates: 0,
            shells: Vec::new(),
            draw_order: Vec::new(),
            sorted_type: None,
            last_type: None,
            combo_count: 0,
            shells_earned: 0,
            next_id,
            rng_state: seed,
        })
    }

    pub fn duration_seconds(&self) -> u32 {
        // PB05 00537e20: tank base10/15/20/25 plus clamped level minus one.
        10 + u32::from(self.origin_tank - 1) * 5 + u32::from(self.origin_level - 1)
    }

    pub fn transition_seed(&self) -> u64 {
        self.rng_state
    }

    pub fn next_entity_id(&self) -> u64 {
        self.next_id
    }

    pub fn remaining_seconds(&self) -> u32 {
        let elapsed = self.started_at.map_or(0, |start| {
            self.tick
                .saturating_sub(start)
                .saturating_mul(u64::from(TICK_MS))
                / 1000
        });
        self.duration_seconds()
            .saturating_sub(elapsed.min(u64::from(u32::MAX)) as u32)
    }

    pub fn click(&mut self, x: f32, y: f32) -> Vec<BonusEvent> {
        if !x.is_finite()
            || !y.is_finite()
            || !(0.0..640.0).contains(&x)
            || !(0.0..480.0).contains(&y)
        {
            return Vec::new();
        }
        if self.started_at.is_none() {
            self.started_at = Some(self.tick);
            return vec![BonusEvent::Started { manual: true }];
        }
        let target = self.draw_order.iter().rev().find_map(|id| {
            self.shells
                .iter()
                .position(|shell| shell.id == *id && shell.contains(x, y))
        });
        let Some(index) = target else {
            return Vec::new();
        };
        let shell = &mut self.shells[index];
        if self.last_type != Some(shell.kind) {
            self.last_type = Some(shell.kind);
            self.combo_count = 0;
        }
        self.combo_count += 1;
        let multiplier = if self.combo_count == 10 {
            self.combo_count = 1;
            self.last_type = None;
            25
        } else {
            self.combo_count
        };
        shell.phase = ShellPhase::Collecting {
            origin_x: shell.x.trunc() as i32,
            origin_y: shell.y.trunc() as i32,
            updates: 1,
            multiplier,
        };
        vec![BonusEvent::Claimed {
            id: shell.id,
            kind: shell.kind,
            multiplier,
        }]
    }

    fn sorted_ids(&self, kind: Option<ShellKind>) -> Vec<u64> {
        self.shells
            .iter()
            .filter(|shell| Some(shell.kind) != kind)
            .chain(self.shells.iter().filter(|shell| Some(shell.kind) == kind))
            .map(|shell| shell.id)
            .collect()
    }

    fn rand_range(&mut self, upper: u64) -> u64 {
        AdventureState::advance_rng(&mut self.rng_state) % upper
    }

    fn drop_shells(&mut self, events: &mut Vec<BonusEvent>) {
        let count = self.rand_range(2) + 1;
        for _ in 0..count {
            let y = self.rand_range(10) as i32 + 50;
            let x = self.rand_range(520) as i32 + 20;
            let kind = ShellKind::from_roll(self.rand_range(100));
            let vy = self.rand_range(10) as f64 / 10.0 * 3.0 + 1.0;
            let id = self.next_id;
            self.next_id += 1;
            self.shells.push(ShellState {
                id,
                kind,
                x: f64::from(x),
                y: f64::from(y),
                vy,
                animation_tick: 0,
                fade_ticks: 0,
                phase: ShellPhase::Falling,
            });
            self.draw_order.push(id);
            events.push(BonusEvent::Dropped { id, kind, x, y });
        }
    }

    /// One 28 ms Board update, followed by Coin widgets in the sorted forward order.
    pub fn update(&mut self) -> BonusUpdate {
        let mut events = Vec::new();
        let mut completed = false;
        if let Some(start) = self.started_at {
            let elapsed_ms = self
                .tick
                .saturating_sub(start)
                .saturating_mul(u64::from(TICK_MS));
            if elapsed_ms / 1000 > u64::from(self.duration_seconds()) {
                if !self.timed_out {
                    self.timed_out = true;
                    events.push(BonusEvent::TimedOut);
                }
                if self.shells.is_empty() {
                    if self.empty_updates > 100 {
                        completed = true;
                    }
                    self.empty_updates = self.empty_updates.saturating_add(1).min(102);
                }
            } else if self.tick.is_multiple_of(10) {
                self.drop_shells(&mut events);
            }
        } else if self.tick.saturating_sub(self.initial_count) > 159 {
            self.started_at = Some(self.tick);
            events.push(BonusEvent::Started { manual: false });
        }
        if completed {
            return BonusUpdate { events, completed };
        }

        self.draw_order = self.sorted_ids(self.last_type);
        self.sorted_type = self.last_type;
        self.tick += 1;

        let mut removed = Vec::new();
        for id in &self.draw_order {
            let Some(shell) = self.shells.iter_mut().find(|shell| shell.id == *id) else {
                continue;
            };
            shell.animation_tick = (shell.animation_tick + 1) % 80;
            if shell.fade_ticks > 0 {
                if shell.phase == ShellPhase::Falling {
                    shell.fade_ticks -= 1;
                    if shell.fade_ticks == 0 {
                        removed.push(shell.id);
                        events.push(BonusEvent::Expired { id: shell.id });
                    }
                    continue;
                }
                shell.fade_ticks = 0;
            }
            match &mut shell.phase {
                ShellPhase::Falling => {
                    shell.y += shell.vy;
                    if shell.y > 370.0 {
                        shell.y = 370.0;
                        shell.fade_ticks = 5;
                    }
                }
                ShellPhase::Collecting {
                    origin_x,
                    origin_y,
                    updates,
                    multiplier,
                } => {
                    *updates += 1;
                    let dx = i64::from(*origin_x) - 290;
                    let dy = i64::from(*origin_y) - 315;
                    let threshold = if dx * dx + dy * dy < 22_501 {
                        5_i64
                    } else {
                        15_i64
                    };
                    let progress = i64::from(*updates);
                    if progress >= threshold {
                        let value = shell.kind.base_value() * u32::from(*multiplier);
                        self.shells_earned = self.shells_earned.saturating_add(value);
                        removed.push(shell.id);
                        events.push(BonusEvent::Credited {
                            id: shell.id,
                            value,
                            total: self.shells_earned,
                        });
                    } else {
                        shell.x = (((threshold - progress) * i64::from(*origin_x) + 290 * progress)
                            / threshold) as f64;
                        shell.y = (((threshold - progress) * i64::from(*origin_y) + 315 * progress)
                            / threshold) as f64;
                    }
                }
            }
        }
        if !removed.is_empty() {
            self.shells.retain(|shell| !removed.contains(&shell.id));
            self.draw_order.retain(|id| !removed.contains(id));
        }
        BonusUpdate { events, completed }
    }

    pub fn validate(&self) -> Result<(), String> {
        if !matches!(
            (self.origin_tank, self.origin_level),
            (1..=3, 1..=6) | (4, 1..=5)
        ) || self.tick < self.initial_count
            || self.tick == u64::MAX
            || self
                .started_at
                .is_some_and(|start| start < self.initial_count || start > self.tick)
            || (self.started_at.is_none() && (self.timed_out || self.empty_updates != 0))
            || (self.timed_out && self.started_at.is_none())
            || self.empty_updates > 102
            || (self.empty_updates > 0 && !self.timed_out)
            || (self.empty_updates > 0 && !self.shells.is_empty())
            || self.combo_count > 9
            || (self.last_type.is_none() && self.combo_count > 1)
            || (self.last_type.is_some() && self.combo_count == 0)
            || self.shells_earned > MAX_SHELL_BALANCE
            || self.next_id == 0
            || self.next_id == u64::MAX
            || self.rng_state == 0
        {
            return Err("invalid bonus clock, combo, award, or RNG".into());
        }
        if let Some(start) = self.started_at {
            let elapsed = self
                .tick
                .saturating_sub(1)
                .saturating_sub(start)
                .saturating_mul(u64::from(TICK_MS));
            if self.timed_out != (elapsed / 1000 > u64::from(self.duration_seconds())) {
                return Err("bonus timeout disagrees with elapsed clock".into());
            }
        }
        let mut ids = HashSet::new();
        for shell in &self.shells {
            if shell.id == 0
                || shell.id >= self.next_id
                || !ids.insert(shell.id)
                || !shell.x.is_finite()
                || !shell.y.is_finite()
                || !shell.vy.is_finite()
                || !(1.0..=3.7).contains(&shell.vy)
                || shell.animation_tick >= 80
                || shell.fade_ticks > 5
            {
                return Err("invalid bonus shell identity, position, or counters".into());
            }
            match shell.phase {
                ShellPhase::Falling => {
                    if !(20.0..=539.0).contains(&shell.x)
                        || !(50.0..=370.0).contains(&shell.y)
                        || (shell.fade_ticks > 0 && shell.y != 370.0)
                    {
                        return Err("invalid falling shell position or fade".into());
                    }
                }
                ShellPhase::Collecting {
                    origin_x,
                    origin_y,
                    updates,
                    multiplier,
                } => {
                    if !(20..=539).contains(&origin_x) || !(50..=370).contains(&origin_y) {
                        return Err("invalid collecting shell origin".into());
                    }
                    let dx = i64::from(origin_x) - 290;
                    let dy = i64::from(origin_y) - 315;
                    let threshold = if dx * dx + dy * dy < 22_501 { 5 } else { 15 };
                    if !(1..threshold).contains(&updates)
                        || !(1..=9).contains(&multiplier) && multiplier != 25
                        || shell.x < f64::from(origin_x.min(290))
                        || shell.x > f64::from(origin_x.max(290))
                        || shell.y < f64::from(origin_y.min(315))
                        || shell.y > f64::from(origin_y.max(315))
                    {
                        return Err("invalid collecting shell flight".into());
                    }
                }
            }
        }
        if self.draw_order.len() != self.shells.len()
            || self.draw_order.iter().any(|id| !ids.remove(id))
            || !ids.is_empty()
            || self.draw_order != self.sorted_ids(self.sorted_type)
        {
            return Err("bonus draw order disagrees with live shells".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seeded_shell(state: &mut BonusState, kind: ShellKind, x: f64, y: f64) -> u64 {
        let id = state.next_id;
        state.next_id += 1;
        state.shells.push(ShellState {
            id,
            kind,
            x,
            y,
            vy: 1.0,
            animation_tick: 0,
            fade_ticks: 0,
            phase: ShellPhase::Falling,
        });
        state.draw_order.push(id);
        id
    }

    fn started() -> BonusState {
        let mut state = BonusState::new(17);
        assert_eq!(
            state.click(0.0, 0.0),
            vec![BonusEvent::Started { manual: true }]
        );
        state
    }

    #[test]
    fn replay_bonus_uses_stage_duration_and_primary_y_x_type_speed_draw_order() {
        let seed = AdventureState::initial_rng(0x375b);
        let mut expected_rng = seed;
        let count = AdventureState::advance_rng(&mut expected_rng) % 2 + 1;
        let y = (AdventureState::advance_rng(&mut expected_rng) % 10 + 50) as i32;
        let x = (AdventureState::advance_rng(&mut expected_rng) % 520 + 20) as i32;
        let kind = ShellKind::from_roll(AdventureState::advance_rng(&mut expected_rng) % 100);
        let vy = (AdventureState::advance_rng(&mut expected_rng) % 10) as f64 / 10.0 * 3.0 + 1.0;
        let mut bonus = BonusState::new_for_stage(seed, 100, 4, 5).unwrap();
        assert_eq!(bonus.duration_seconds(), 29);
        bonus.click(0.0, 0.0);
        bonus.update();
        assert_eq!(bonus.shells.len(), count as usize);
        assert_eq!((bonus.shells[0].id, bonus.shells[0].kind), (100, kind));
        assert_eq!(bonus.shells[0].x as i32, x);
        assert_eq!(bonus.shells[0].y.to_bits(), (f64::from(y) + vy).to_bits());
        assert_eq!(bonus.shells[0].vy.to_bits(), vy.to_bits());

        let mut early = BonusState::new_for_stage(seed, 100, 1, 1).unwrap();
        assert_eq!(early.duration_seconds(), 10);
        early.click(0.0, 0.0);
        early.tick = 392;
        assert!(!early.update().events.contains(&BonusEvent::TimedOut));
        early.tick = 393;
        assert!(early.update().events.contains(&BonusEvent::TimedOut));
    }

    #[test]
    fn manual_and_automatic_start_preserve_absolute_drop_phase() {
        let mut manual = started();
        for old_count in 0..=10 {
            let update = manual.update();
            assert_eq!(
                update
                    .events
                    .iter()
                    .any(|event| matches!(event, BonusEvent::Dropped { .. })),
                old_count == 0 || old_count == 10
            );
        }
        let mut before_divisible = BonusState::new(17);
        for _ in 0..10 {
            before_divisible.update();
        }
        assert_eq!(
            before_divisible.click(2.0, 2.0),
            vec![BonusEvent::Started { manual: true }]
        );
        assert!(
            before_divisible
                .update()
                .events
                .iter()
                .any(|event| matches!(event, BonusEvent::Dropped { .. }))
        );

        let mut automatic = BonusState::new(17);
        for _ in 0..160 {
            automatic.update();
        }
        assert_eq!(automatic.started_at, None);
        let update = automatic.update();
        assert_eq!(automatic.started_at, Some(160));
        assert_eq!(update.events, vec![BonusEvent::Started { manual: false }]);
        for _ in 0..9 {
            automatic.update();
        }
        assert!(
            automatic
                .update()
                .events
                .iter()
                .any(|event| matches!(event, BonusEvent::Dropped { .. }))
        );
    }

    #[test]
    fn shell_roll_boundaries_match_source_buckets() {
        for (roll, kind) in [
            (0, ShellKind::Silver),
            (19, ShellKind::Silver),
            (20, ShellKind::Treasure),
            (29, ShellKind::Treasure),
            (30, ShellKind::Pearl),
            (49, ShellKind::Pearl),
            (50, ShellKind::Diamond),
            (79, ShellKind::Diamond),
            (80, ShellKind::Gold),
            (99, ShellKind::Gold),
        ] {
            assert_eq!(ShellKind::from_roll(roll), kind);
        }
    }

    #[test]
    fn old_count_572_times_out_and_completion_needs_102_empty_checks() {
        let mut state = started();
        for _ in 0..572 {
            state.update();
        }
        assert!(!state.timed_out);
        assert_eq!(state.remaining_seconds(), 0);
        let update = state.update();
        assert!(state.timed_out);
        assert_eq!(
            state.empty_updates,
            if state.shells.is_empty() { 1 } else { 0 }
        );
        assert!(update.events.contains(&BonusEvent::TimedOut));
        assert!(!update.completed);

        // No shell generation after timeout, and Board checks emptiness before Coin updates.
        state.shells.clear();
        state.draw_order.clear();
        state.empty_updates = 0;
        for _ in 0..101 {
            assert!(!state.update().completed);
        }
        assert_eq!(state.empty_updates, 101);
        assert!(state.update().completed);
        assert_eq!(state.empty_updates, 102);
    }

    #[test]
    fn short_and_boundary_flights_credit_once_after_integer_interpolation() {
        let mut short = started();
        let id = seeded_shell(&mut short, ShellKind::Silver, 200.9, 250.8);
        assert_eq!(
            short.click(201.0, 251.0),
            vec![BonusEvent::Claimed {
                id,
                kind: ShellKind::Silver,
                multiplier: 1
            }]
        );
        short.update();
        assert_eq!((short.shells[0].x, short.shells[0].y), (236.0, 276.0));
        for _ in 0..2 {
            short.update();
        }
        assert_eq!(short.shells_earned, 0);
        let arrival = short.update();
        assert_eq!(
            arrival
                .events
                .iter()
                .filter(|event| matches!(event, BonusEvent::Credited { .. }))
                .count(),
            1
        );
        assert_eq!(short.shells_earned, 1);
        assert!(short.shells.iter().all(|shell| shell.id != id));
        assert!(
            short
                .update()
                .events
                .iter()
                .all(|event| !matches!(event, BonusEvent::Credited { .. }))
        );

        let mut boundary = started();
        let id = seeded_shell(&mut boundary, ShellKind::Treasure, 140.0, 316.0);
        boundary.click(141.0, 317.0);
        for _ in 0..13 {
            boundary.update();
        }
        assert_eq!(boundary.shells_earned, 0); // distance squared is exactly 22,501
        assert_eq!(boundary.shells[0].id, id);
        boundary.update();
        assert_eq!(boundary.shells_earned, 20);
    }

    #[test]
    fn tenth_combo_resets_and_type_change_restarts_at_one() {
        let mut state = started();
        for expected in 1..=10 {
            let id = seeded_shell(&mut state, ShellKind::Gold, 200.0, 200.0);
            let expected_multiplier = if expected == 10 { 25 } else { expected };
            assert_eq!(
                state.click(201.0, 201.0),
                vec![BonusEvent::Claimed {
                    id,
                    kind: ShellKind::Gold,
                    multiplier: expected_multiplier
                }]
            );
        }
        assert_eq!(state.last_type, None);
        assert_eq!(state.combo_count, 1);
        let id = seeded_shell(&mut state, ShellKind::Silver, 300.0, 200.0);
        assert_eq!(
            state.click(301.0, 201.0),
            vec![BonusEvent::Claimed {
                id,
                kind: ShellKind::Silver,
                multiplier: 1
            }]
        );
    }

    #[test]
    fn overlap_uses_previous_sort_until_next_board_update() {
        let mut state = started();
        let first = seeded_shell(&mut state, ShellKind::Silver, 200.0, 200.0);
        let middle = seeded_shell(&mut state, ShellKind::Gold, 200.0, 200.0);
        let last = seeded_shell(&mut state, ShellKind::Silver, 200.0, 200.0);
        assert_eq!(
            state.click(201.0, 201.0),
            vec![BonusEvent::Claimed {
                id: last,
                kind: ShellKind::Silver,
                multiplier: 1
            }]
        );
        assert_eq!(
            state.click(201.0, 201.0),
            vec![BonusEvent::Claimed {
                id: middle,
                kind: ShellKind::Gold,
                multiplier: 1
            }]
        );
        state.update();
        let position = |id| {
            state
                .draw_order
                .iter()
                .position(|candidate| *candidate == id)
                .unwrap()
        };
        assert!(position(first) < position(last) && position(last) < position(middle));
        assert_eq!(
            state.click(201.0, 202.0),
            vec![BonusEvent::Claimed {
                id: first,
                kind: ShellKind::Silver,
                multiplier: 1
            }]
        );
    }

    #[test]
    fn simultaneous_arrivals_credit_in_forward_sorted_widget_order() {
        let mut state = started();
        state.tick = 1;
        let silver = seeded_shell(&mut state, ShellKind::Silver, 290.0, 315.0);
        let gold = seeded_shell(&mut state, ShellKind::Gold, 290.0, 315.0);
        for shell in &mut state.shells {
            shell.phase = ShellPhase::Collecting {
                origin_x: 290,
                origin_y: 315,
                updates: 4,
                multiplier: 1,
            };
        }
        state.last_type = Some(ShellKind::Silver);
        state.combo_count = 1;
        let credits: Vec<_> = state
            .update()
            .events
            .into_iter()
            .filter(|event| matches!(event, BonusEvent::Credited { .. }))
            .collect();
        assert_eq!(
            credits,
            vec![
                BonusEvent::Credited {
                    id: gold,
                    value: 2,
                    total: 2
                },
                BonusEvent::Credited {
                    id: silver,
                    value: 1,
                    total: 3
                },
            ]
        );
    }

    #[test]
    fn bottom_contact_fades_for_five_subsequent_coin_updates() {
        let mut state = started();
        let id = seeded_shell(&mut state, ShellKind::Pearl, 200.0, 369.0);
        state.shells[0].vy = 2.0;
        state.update();
        assert_eq!((state.shells[0].y, state.shells[0].fade_ticks), (370.0, 5));
        for remaining in (1..=4).rev() {
            state.update();
            assert_eq!(state.shells[0].fade_ticks, remaining);
        }
        assert_eq!(state.update().events, vec![BonusEvent::Expired { id }]);
        assert!(state.shells.iter().all(|shell| shell.id != id));
    }

    #[test]
    fn fading_shell_claim_survives_save_and_clears_fade_before_flight() {
        let mut state = started();
        let id = seeded_shell(&mut state, ShellKind::Silver, 290.0, 369.0);
        state.shells[0].vy = 2.0;
        state.update();
        for _ in 0..4 {
            state.update();
        }
        assert_eq!(state.shells[0].fade_ticks, 1);
        assert_eq!(
            state.click(291.0, 371.0),
            vec![BonusEvent::Claimed {
                id,
                kind: ShellKind::Silver,
                multiplier: 1
            }]
        );
        assert!(state.validate().is_ok());
        let mut restored: BonusState =
            serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
        assert!(restored.validate().is_ok());
        let first_flight = restored.update();
        assert!(!first_flight.events.contains(&BonusEvent::Expired { id }));
        assert_eq!(restored.shells[0].fade_ticks, 0);
        assert!(matches!(
            restored.shells[0].phase,
            ShellPhase::Collecting { updates: 2, .. }
        ));
        for _ in 0..3 {
            restored.update();
        }
        assert_eq!(restored.shells_earned, 1);
        assert!(restored.shells.iter().all(|shell| shell.id != id));
    }

    #[test]
    fn final_arrival_begins_empty_wait_on_following_board_update() {
        let mut state = started();
        let id = seeded_shell(&mut state, ShellKind::Silver, 290.0, 315.0);
        state.click(291.0, 316.0);
        state.tick = 572;
        let timeout = state.update();
        assert_eq!(timeout.events, vec![BonusEvent::TimedOut]);
        assert_eq!(state.empty_updates, 0);
        for _ in 0..2 {
            state.update();
        }
        let last_arrival = state.update();
        assert_eq!(
            last_arrival.events,
            vec![BonusEvent::Credited {
                id,
                value: 1,
                total: 1
            }]
        );
        assert_eq!(state.empty_updates, 0);
        assert!(state.shells.is_empty());
        state.update();
        assert_eq!(state.empty_updates, 1);
    }

    #[test]
    fn validation_rejects_empty_wait_with_live_shells_after_timeout() {
        let mut state = started();
        while !state.timed_out {
            state.update();
        }
        assert!(!state.shells.is_empty());
        assert_eq!(state.empty_updates, 0);
        state.validate().unwrap();

        state.empty_updates = 1;
        assert!(state.validate().is_err());
    }

    #[test]
    fn validation_rejects_nonfinite_duplicate_and_disconnected_order() {
        let mut state = started();
        seeded_shell(&mut state, ShellKind::Silver, 200.0, 200.0);
        assert!(state.validate().is_ok());
        state.shells[0].x = f64::NAN;
        assert!(state.validate().is_err());
        state.shells[0].x = 200.0;
        state.draw_order[0] = 999;
        assert!(state.validate().is_err());
        state.draw_order[0] = state.shells[0].id;
        state.shells.push(state.shells[0].clone());
        state.draw_order.push(state.shells[0].id);
        assert!(state.validate().is_err());

        state.shells.pop();
        state.draw_order.pop();
        state.shells[0].phase = ShellPhase::Collecting {
            origin_x: i32::MIN,
            origin_y: i32::MIN,
            updates: 1,
            multiplier: 1,
        };
        assert!(state.validate().is_err());
    }

    #[test]
    fn validation_keeps_last_sort_across_click_but_rejects_wrong_permutation() {
        let mut state = started();
        seeded_shell(&mut state, ShellKind::Silver, 200.0, 200.0);
        seeded_shell(&mut state, ShellKind::Gold, 300.0, 200.0);
        state.click(201.0, 201.0);
        assert_eq!(state.last_type, Some(ShellKind::Silver));
        assert_eq!(state.sorted_type, None);
        assert!(state.validate().is_ok());
        state.draw_order.reverse();
        assert!(state.validate().is_err());
    }

    #[test]
    fn midflight_and_between_clicks_roundtrip_without_changing_hit_order() {
        let mut state = started();
        seeded_shell(&mut state, ShellKind::Silver, 200.0, 200.0);
        seeded_shell(&mut state, ShellKind::Gold, 200.0, 200.0);
        state.click(201.0, 201.0);
        let mut restored: BonusState =
            serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
        assert_eq!(restored.click(201.0, 201.0), state.click(201.0, 201.0));
        assert_eq!(restored.update().events, state.update().events);
        assert_eq!(restored, state);
        assert!(restored.validate().is_ok());
    }

    #[test]
    fn results_count_up_only_after_thirty_updates_and_caps_presentation() {
        let mut result = BonusResult {
            origin_tank: 1,
            origin_level: 6,
            earned: 125,
            previous_balance: 9_999_900,
            updates: 30,
            purchase: PurchaseReceipt::new(0),
        };
        assert_eq!(result.presented_balance(), 9_999_900);
        result.updates = 31;
        assert_eq!(result.presented_balance(), 9_999_950);
        result.updates = 35;
        assert_eq!(result.presented_balance(), MAX_SHELL_BALANCE);
        assert!(result.validate().is_ok());
        result.previous_balance = MAX_SHELL_BALANCE + 1;
        assert!(result.validate().is_err());
    }

    #[test]
    fn tank_two_bonus_uses_integer_seconds_and_waits_for_old_empty_counter() {
        // A25-08: budget20, strict integer elapsed>20. Relative old tick749
        // is20seconds; tick750 is21seconds. This is a controlled boundary
        // fixture, independent of the native normal-speed earning scenario.
        let mut state = BonusState::new_for_tank(42, 2).unwrap();
        assert_eq!(state.duration_seconds(), 20);
        state.click(0.0, 0.0);
        state.tick = 749;
        assert_eq!(state.remaining_seconds(), 0);
        let before = state.update();
        assert!(!before.events.contains(&BonusEvent::TimedOut));
        assert!(!state.timed_out);
        let expired = state.update();
        assert_eq!(expired.events, vec![BonusEvent::TimedOut]);
        assert!(!expired.completed);
        assert_eq!(state.empty_updates, 1);
        state.validate().unwrap();
        for _ in 0..100 {
            let update = state.update();
            assert!(!update.completed);
            assert!(!update.events.contains(&BonusEvent::TimedOut));
        }
        assert_eq!(state.empty_updates, 101);
        let final_tick = state.tick;
        assert!(state.update().completed);
        assert_eq!(state.empty_updates, 102);
        assert_eq!(state.tick, final_tick);
        assert!(BonusState::new_for_tank(42, 4).is_err());
        state.origin_tank = 0;
        assert!(state.validate().is_err());
    }

    #[test]
    fn tank_three_bonus_keeps_origin_and_strict_integer_timeout_boundary() {
        // PB52 base20 + level6 - 1. At old relative tick928 the integer
        // elapsed clock is25; at929 it is26 and exceeds the budget.
        let mut state = BonusState::new_for_tank(42, 3).unwrap();
        assert_eq!(state.duration_seconds(), 25);
        state.click(0.0, 0.0);
        state.tick = 928;
        assert!(!state.update().events.contains(&BonusEvent::TimedOut));
        assert_eq!(state.origin_tank, 3);
        let expired = state.update();
        assert_eq!(expired.events, vec![BonusEvent::TimedOut]);
        assert!(!expired.completed);
        assert_eq!(state.empty_updates, 1);
        let mut restored: BonusState =
            serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
        assert_eq!(restored.duration_seconds(), 25);
        for _ in 0..101 {
            let actual = restored.update();
            let expected = state.update();
            assert_eq!(actual.events, expected.events);
            assert_eq!(actual.completed, expected.completed);
            assert_eq!(restored, state);
        }
        state.validate().unwrap();
        let result = BonusResult {
            origin_tank: 3,
            origin_level: 6,
            earned: 808,
            previous_balance: 1347,
            updates: 30,
            purchase: PurchaseReceipt::new(0),
        };
        result.validate().unwrap();
        assert_eq!(result.origin_tank, 3);
        assert!(BonusState::new_for_tank(42, 255).is_err());
        restored.origin_tank = 255;
        assert!(restored.validate().is_err());
    }
}
