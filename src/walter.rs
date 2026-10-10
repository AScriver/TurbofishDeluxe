//! Shared impact state and the physical Walter's nested Glove.
//! Board owns contact admission and target identity; actors own impact decay.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WalterImpact {
    pub recoil: u8,
    pub secondary: u8,
    pub impulse: f64,
}

impl WalterImpact {
    pub fn validate(self) -> bool {
        self.recoil <= 50
            && self.secondary <= 180
            && self.impulse.is_finite()
            && self.impulse.abs() <= 15.0
    }

    pub fn hit(&mut self, amount: f64) {
        self.recoil = 50;
        self.secondary = 180;
        self.impulse = amount;
    }

    /// Called by the target's own post-action movement tail.
    pub fn tick_recoil(&mut self) -> f64 {
        if self.recoil > 0 {
            self.recoil -= 1;
            self.impulse *= 0.9;
            self.impulse
        } else {
            0.0
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Glove {
    pub x: i32,
    pub y: i32,
    pub lifetime: u8,
    pub feedback: u8,
    pub right: bool,
}

impl Glove {
    pub fn new(right: bool) -> Self {
        Self {
            x: -200,
            y: -200,
            lifetime: 30,
            feedback: 0,
            right,
        }
    }

    pub fn validate(self) -> bool {
        (1..=30).contains(&self.lifetime)
            && self.feedback <= 5
            && ((self.x == -200 && self.y == -200)
                || ((-60..=530).contains(&self.x) && (0..=550).contains(&self.y)))
    }

    pub fn center(self) -> (i32, i32) {
        let offset = if self.lifetime > 25 {
            0
        } else if self.lifetime > 20 {
            30
        } else {
            60
        };
        (
            self.x + 80 + if self.right { offset } else { -offset },
            self.y + 40,
        )
    }
}
