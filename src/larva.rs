//! Ordinary Tank 3 larva collectible. Installed-binary behavior is recorded in
//! PB38; animation and widget ordering are also described by pinned W1
//! `Larva.cpp` (f919b3c). The board owns IDs, removal, and cash transactions.

use serde::{Deserialize, Serialize};

pub const LARVA_VALUE: i32 = 150;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LarvaUpdate {
    Alive,
    Expired,
    Credited,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LarvaState {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub widget_x: i32,
    pub widget_y: i32,
    pub frame: u8,
    /// The source exposes the cursor only after a subsequent update begins
    /// below Y=320. This remains true during the credit flight.
    pub mouse_visible: bool,
    pub picked_up: bool,
    /// Source byte +0x175 suppresses cursor activation independently of pickup.
    pub cursor_suppressed: bool,
}

impl LarvaState {
    /// Constructor consumes one full-width draw even though it does not use it.
    pub fn spawn(id: u64, x: i32, y: i32, next_random: &mut impl FnMut() -> u64) -> Self {
        let _unused = next_random();
        Self {
            id,
            x: f64::from(x),
            y: f64::from(y),
            widget_x: x,
            widget_y: y,
            frame: 0,
            mouse_visible: false,
            picked_up: false,
            cursor_suppressed: false,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.id == 0
            || !self.x.is_finite()
            || !self.y.is_finite()
            || !(-100.0..=640.0).contains(&self.x)
            || !(0.0..=500.0).contains(&self.y)
            || self.widget_x != self.x as i32
            || self.widget_y != self.y as i32
            || self.frame >= 10
            || (self.picked_up && !self.mouse_visible)
        {
            return Err("invalid ordinary larva state".into());
        }
        Ok(())
    }

    /// World-coordinate hit area; Board decides whether this widget receives
    /// the click ahead of other overlapping objects.
    pub fn contains_world_point(&self, x: i32, y: i32) -> bool {
        self.mouse_visible
            && x >= self.widget_x
            && x < self.widget_x + 72
            && y >= self.widget_y
            && y < self.widget_y + 72
    }

    /// Collection begins only on a visible, unclaimed larva. Credit remains
    /// deferred until a later `tick` returns `Credited`.
    pub fn try_pick_up(&mut self) -> bool {
        if !self.mouse_visible || self.picked_up {
            return false;
        }
        self.picked_up = true;
        true
    }

    pub fn tick(&mut self) -> LarvaUpdate {
        if !self.mouse_visible && self.y < 320.0 && !self.cursor_suppressed {
            self.mouse_visible = true;
        }

        if self.picked_up {
            self.x += (550.0 - self.x) / 7.0;
            self.y += (30.0 - self.y) / 7.0;
            // The original checks the previous integer widget location, not
            // the new double location or the horizontal HUD distance.
            if self.widget_y < 40 {
                return LarvaUpdate::Credited;
            }
        } else {
            self.y -= 1.0;
            if self.y < 64.0 {
                return LarvaUpdate::Expired;
            }
            self.x = self.x.min(530.0);
        }

        self.widget_x = self.x as i32;
        self.widget_y = self.y as i32;
        self.frame = (self.frame + 1) % 10;
        LarvaUpdate::Alive
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn larva(x: i32, y: i32) -> LarvaState {
        let mut draws = 0;
        let result = LarvaState::spawn(1, x, y, &mut || {
            draws += 1;
            47
        });
        assert_eq!(draws, 1);
        result
    }

    #[test]
    fn visibility_uses_old_y_and_cursor_suppression_is_separate() {
        let mut l = larva(100, 320);
        assert!(!l.try_pick_up());
        assert_eq!(l.tick(), LarvaUpdate::Alive);
        assert_eq!(l.widget_y, 319);
        assert!(!l.mouse_visible);
        assert_eq!(l.tick(), LarvaUpdate::Alive);
        assert!(l.mouse_visible);
        assert!(l.try_pick_up());
        let mut hidden = larva(100, 319);
        hidden.cursor_suppressed = true;
        hidden.tick();
        assert!(!hidden.mouse_visible);
        assert!(!hidden.picked_up);
    }

    #[test]
    fn expiry_is_strict_after_rise_and_arrival_uses_old_widget() {
        let mut l = larva(200, 65);
        assert_eq!(l.tick(), LarvaUpdate::Alive);
        assert_eq!(l.widget_y, 64);
        assert_eq!(l.tick(), LarvaUpdate::Expired);

        let mut claimed = larva(100, 40);
        claimed.mouse_visible = true;
        assert!(claimed.try_pick_up());
        assert_eq!(claimed.tick(), LarvaUpdate::Alive);
        assert!(claimed.widget_y < 40);
        assert_eq!(claimed.tick(), LarvaUpdate::Credited);
    }

    #[test]
    fn serialized_flight_keeps_fractional_position_and_single_arrival_boundary() {
        let mut l = larva(101, 80);
        l.mouse_visible = true;
        assert!(l.try_pick_up());
        assert_eq!(l.tick(), LarvaUpdate::Alive);
        let encoded = serde_json::to_string(&l).unwrap();
        let mut restored: LarvaState = serde_json::from_str(&encoded).unwrap();
        restored.validate().unwrap();
        assert_eq!(restored.tick(), l.tick());
        assert_eq!(restored.widget_x, l.widget_x);
        assert_eq!(restored.widget_y, l.widget_y);
    }

    #[test]
    fn claimed_larva_requires_prior_visibility_but_cursor_flag_is_independent() {
        let mut l = larva(100, 319);
        l.picked_up = true;
        assert!(l.validate().is_err());
        l.mouse_visible = true;
        l.validate().unwrap();
        l.cursor_suppressed = true;
        l.validate().unwrap();
    }
}
