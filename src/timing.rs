//! Process-local scheduling for explicitly accelerated test runs.

use crate::sim::TICK_MS;

const STEP_SECONDS: f64 = TICK_MS as f64 / 1000.0;

pub fn wall_deadline_reached(elapsed_seconds: f64, limit: Option<f64>) -> bool {
    limit.is_some_and(|limit| elapsed_seconds >= limit)
}

#[derive(Clone, Copy, Debug)]
pub struct InputPress {
    wall_seconds: f64,
    logical_ms: u64,
}

#[derive(Debug)]
pub struct StepClock {
    factor: u8,
    accumulator: f64,
    logical_input_ms: u64,
}

impl StepClock {
    pub fn new(factor: u8) -> Self {
        assert!((1..=8).contains(&factor));
        Self {
            factor,
            accumulator: 0.0,
            logical_input_ms: 0,
        }
    }

    pub fn press(&self, wall_seconds: f64) -> InputPress {
        InputPress {
            wall_seconds,
            logical_ms: self.logical_input_ms,
        }
    }

    pub fn held_elapsed_ms(&self, press: InputPress, now: f64) -> u32 {
        if self.factor == 1 {
            ((now - press.wall_seconds) * 1000.0).max(0.0) as u32
        } else {
            self.logical_input_ms
                .saturating_add(u64::from(TICK_MS))
                .saturating_sub(press.logical_ms)
                .min(u64::from(u32::MAX)) as u32
        }
    }

    pub fn add_frame(&mut self, frame_seconds: f32, paused: bool) {
        let factor = if paused { 1 } else { self.factor };
        self.accumulator += f64::from(frame_seconds).min(0.2) * f64::from(factor);
    }

    pub fn step_due(&self) -> bool {
        self.accumulator >= STEP_SECONDS
    }

    pub fn consume_step(&mut self, paused: bool) {
        self.accumulator -= STEP_SECONDS;
        if !paused {
            self.logical_input_ms = self.logical_input_ms.saturating_add(u64::from(TICK_MS));
        }
    }

    pub fn clear_backlog(&mut self) {
        self.accumulator = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn steps_for(factor: u8, frames: &[f32]) -> Vec<u32> {
        let mut clock = StepClock::new(factor);
        let press = clock.press(10.0);
        let mut ages = Vec::new();
        for &frame in frames {
            clock.add_frame(frame, false);
            while clock.step_due() {
                ages.push(clock.held_elapsed_ms(press, 10.0));
                clock.consume_step(false);
            }
        }
        ages
    }

    #[test]
    fn default_and_explicit_one_have_the_same_wall_input_schedule() {
        let frames = [0.016_f32, 0.016, 0.05, 0.003, 0.2];
        let mut clock = StepClock::new(1);
        let press = clock.press(0.0);
        let mut old_accumulator = 0.0_f64;
        let mut wall = 0.0_f64;
        let mut expected = Vec::new();
        let mut actual = Vec::new();
        for frame in frames {
            wall += f64::from(frame);
            old_accumulator += f64::from(frame).min(0.2);
            clock.add_frame(frame, false);
            while old_accumulator >= STEP_SECONDS {
                expected.push((wall * 1000.0).max(0.0) as u32);
                old_accumulator -= STEP_SECONDS;
            }
            while clock.step_due() {
                actual.push(clock.held_elapsed_ms(press, wall));
                clock.consume_step(false);
            }
        }
        assert_eq!(actual, expected);
        assert_eq!(actual.len(), 10);
    }

    #[test]
    fn accelerated_hold_age_advances_once_per_unpaused_step() {
        let ages = steps_for(4, &[0.028, 0.028]);
        assert_eq!(ages, [28, 56, 84, 112, 140, 168, 196, 224]);
        assert!(ages[6] <= 200 && ages[7] > 200);
        assert!(ages[2] <= 100 && ages[3] > 100);
    }

    #[test]
    fn clamp_precedes_factor_and_fractional_remainder_is_retained() {
        let mut clock = StepClock::new(8);
        clock.add_frame(5.0, false);
        let mut count = 0;
        while clock.step_due() {
            clock.consume_step(false);
            count += 1;
        }
        assert_eq!(count, 57); // floor((200 ms * 8) / 28 ms)
        clock.add_frame(0.001, false);
        assert!(!clock.step_due());
        clock.add_frame(0.004, false);
        assert!(clock.step_due());
    }

    #[test]
    fn pause_uses_one_x_and_cleared_backlog_cannot_replay_holds() {
        let mut clock = StepClock::new(8);
        let press = clock.press(0.0);
        clock.add_frame(0.2, false);
        clock.consume_step(false);
        clock.clear_backlog();
        clock.add_frame(0.028, true);
        assert!(clock.step_due());
        clock.consume_step(true);
        assert!(!clock.step_due());
        assert_eq!(clock.held_elapsed_ms(press, 10.0), 56);
    }

    #[test]
    fn deadline_stops_within_a_batch_without_a_final_step() {
        let mut clock = StepClock::new(8);
        clock.add_frame(0.2, false);
        let mut steps = 0;
        while clock.step_due() {
            let wall_elapsed = if steps == 3 { 1.0 } else { 0.9 };
            if wall_deadline_reached(wall_elapsed, Some(1.0)) {
                break;
            }
            clock.consume_step(false);
            steps += 1;
        }
        assert_eq!(steps, 3);
        assert_eq!(clock.logical_input_ms, 84);
    }

    #[test]
    fn phase_barrier_discards_remaining_batch() {
        let mut clock = StepClock::new(8);
        clock.add_frame(0.2, false);
        assert!(clock.step_due());
        clock.consume_step(false);
        clock.clear_backlog();
        assert!(!clock.step_due());
        assert_eq!(clock.logical_input_ms, 28);
    }
}
