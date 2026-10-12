//! Transient state for the normal game's input, pause, and singing-fish paths.
//! Clocks, domain controllers, audio output, and checkpoint gates stay with
//! their existing owners in the application.

use crate::{
    fish_songs::{FishSongPlayer, SongData, ToneCommand},
    help::{HelpExit, HelpReturn},
    sim::{Action, Event},
    timing::InputPress,
    virtual_tank::NamedFish,
};
use std::collections::HashMap;

#[derive(Default)]
pub(super) struct InputCapture {
    pub(super) pending_actions: Vec<Action>,
    pub(super) feed_press_at: Option<(InputPress, Option<(i32, i32)>)>,
    pub(super) held_feed_at: Option<InputPress>,
    pub(super) held_fire_at: Option<InputPress>,
    hatch_pointer_owned: bool,
    hatch_background_down: bool,
}

impl InputCapture {
    /// Modal and invasion boundaries cancel holds without discarding actions
    /// already accepted by the window or altering hatch release bookkeeping.
    pub(super) fn cancel_holds(&mut self) {
        self.feed_press_at = None;
        self.held_feed_at = None;
        self.held_fire_at = None;
    }

    pub(super) fn cancel_queued_input(&mut self) {
        self.pending_actions.clear();
        self.cancel_holds();
    }

    pub(super) fn release_left(&mut self) {
        self.hatch_pointer_owned = false;
        self.cancel_holds();
    }

    pub(super) fn begin_hatch_hold(&mut self, left_down: bool) -> Action {
        self.hatch_pointer_owned = true;
        self.hatch_background_down = left_down;
        Action::HatchHold { down: left_down }
    }

    pub(super) fn update_hatch_hold(&mut self, left_down: bool) -> Option<Action> {
        let held = self.hatch_pointer_owned && left_down;
        if held == self.hatch_background_down {
            return None;
        }
        self.hatch_background_down = held;
        Some(Action::HatchHold { down: held })
    }

    pub(super) fn leave_hatch(&mut self) {
        self.hatch_pointer_owned = false;
        self.hatch_background_down = false;
    }

    pub(super) fn arm_held_feed(&mut self, events: &[Event], left_down: bool) {
        if self
            .feed_press_at
            .is_some_and(|(_, gus_click)| super::arms_held_feed(events, gus_click))
            && left_down
        {
            self.held_feed_at = self.feed_press_at.take().map(|(pressed_at, _)| pressed_at);
        }
    }
}

#[derive(Default)]
pub(super) struct ModalPause {
    paused: bool,
    help_restore_paused: Option<bool>,
}

impl ModalPause {
    pub(super) fn is_paused(&self) -> bool {
        self.paused
    }

    pub(super) fn toggle(&mut self) {
        self.paused = !self.paused;
    }

    pub(super) fn enter_help(&mut self) {
        self.help_restore_paused = Some(self.paused);
        self.paused = true;
    }

    pub(super) fn finish_help(&mut self, exit: HelpExit) -> Option<Action> {
        match exit {
            HelpExit::Continue => Some(Action::Continue),
            HelpExit::ReturnTo(HelpReturn::PausedTank) => {
                self.paused = self.help_restore_paused.take().unwrap_or(true);
                None
            }
            HelpExit::ReturnTo(HelpReturn::Selector) => None,
        }
    }

    pub(super) fn resume_mode(&mut self) {
        self.paused = false;
    }

    pub(super) fn reset_profile(&mut self) {
        self.help_restore_paused = None;
        self.paused = false;
    }
}

#[derive(Default)]
pub(super) struct FishSongRuntime {
    player: FishSongPlayer,
    singing_feeds: HashMap<u16, u8>,
    pending_song_ticks: Option<u8>,
    pending_song_kind: Option<NamedFish>,
    song_sequence: usize,
    song_id: i32,
    song_title: String,
    song_title_until: f64,
}

impl FishSongRuntime {
    pub(super) fn record_feed(&mut self, fish_id: u16, named: Option<NamedFish>) {
        let count = self.singing_feeds.entry(fish_id).or_default();
        *count = count.wrapping_add(1);
        if (*count).is_multiple_of(5) && !self.player.busy() {
            self.pending_song_ticks = Some(5);
            self.pending_song_kind = named;
        }
    }

    /// Called after feed detection in the same unpaused Virtual Tank step.
    /// Sample wall time only when starting a song and publishing its title,
    /// preserving the original call order and keeping simulation time separate.
    pub(super) fn advance_pending(
        &mut self,
        songs: Option<&[SongData]>,
        mut wall_seconds: impl FnMut() -> f64,
    ) {
        let Some(left) = self.pending_song_ticks.as_mut() else {
            return;
        };
        *left = left.saturating_sub(1);
        if *left != 0 {
            return;
        }
        self.pending_song_ticks = None;
        let Some(songs) = songs else {
            return;
        };
        let special = self.pending_song_kind.take();
        let eligible: Vec<_> = songs
            .iter()
            .filter(|song| {
                if song.property("test").is_some() {
                    return false;
                }
                match special {
                    Some(NamedFish::Kilgore) => song.property("kilgore").is_some(),
                    Some(NamedFish::Ludwig) => {
                        !song.is_long()
                            && (song.property("beethoven").is_some()
                                || song.property("beethovenrare").is_some())
                    }
                    Some(NamedFish::Santa) => {
                        !song.is_long()
                            && (song.property("santa").is_some()
                                || song.property("santarare").is_some())
                    }
                    _ => {
                        !song.is_long()
                            && song.property("beethovenrare").is_none()
                            && song.property("santarare").is_none()
                    }
                }
            })
            .collect();
        let Some(song) = eligible.get(self.song_sequence % eligible.len().max(1)) else {
            return;
        };
        let selected = if self.song_sequence % 5 == 4 {
            song.property("short")
                .and_then(|stem| {
                    songs.iter().find(|candidate| {
                        candidate.is_long() && candidate.property("long") == Some(stem)
                    })
                })
                .unwrap_or(*song)
        } else {
            *song
        };
        self.song_sequence += 1;
        self.song_id = self.song_id.saturating_add(1);
        let applause =
            selected.is_long() && !matches!(special, Some(NamedFish::Kilgore | NamedFish::Santa));
        if self
            .player
            .start(
                selected,
                self.song_id,
                (wall_seconds() * 1000.0) as u64,
                applause,
            )
            .is_ok()
        {
            self.song_title = selected
                .property("title")
                .unwrap_or(&selected.file_name)
                .to_owned();
            if selected.is_long() {
                self.song_title.push_str(" (extended)");
            }
            self.song_title_until = wall_seconds() + 5.0;
        }
    }

    pub(super) fn current_song_id(&self) -> i32 {
        self.song_id
    }

    /// The caller stops this ID in the audio backend before clearing runtime
    /// playback. Sequence/ID/title deadline remain process-lived, as before.
    pub(super) fn reset_profile(&mut self) {
        self.player.clear();
        self.singing_feeds.clear();
        self.pending_song_ticks = None;
        self.pending_song_kind = None;
        self.song_title.clear();
    }

    pub(super) fn pause(&mut self, now_ms: u64) {
        self.player.pause(now_ms);
    }

    pub(super) fn advance_frame(&mut self, now_ms: u64, active: bool) -> Vec<ToneCommand> {
        if active {
            self.player.resume(now_ms);
            self.player.advance(now_ms)
        } else {
            self.pause(now_ms);
            Vec::new()
        }
    }

    pub(super) fn title(&self, now_seconds: f64) -> Option<&str> {
        (now_seconds < self.song_title_until).then_some(self.song_title.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::timing::StepClock;

    fn captured_input() -> InputCapture {
        let clock = StepClock::new(1);
        InputCapture {
            pending_actions: vec![Action::Continue, Action::OpenMenu],
            feed_press_at: Some((clock.press(2.0), None)),
            held_feed_at: Some(clock.press(1.0)),
            held_fire_at: Some(clock.press(1.5)),
            hatch_pointer_owned: true,
            hatch_background_down: true,
        }
    }

    #[test]
    fn modal_cancellation_preserves_accepted_actions_and_hatch_release() {
        let mut input = captured_input();
        input.cancel_holds();
        assert_eq!(input.pending_actions, [Action::Continue, Action::OpenMenu]);
        assert!(input.feed_press_at.is_none());
        assert!(input.held_feed_at.is_none());
        assert!(input.held_fire_at.is_none());
        assert_eq!(input.update_hatch_hold(true), None);
        input.release_left();
        assert_eq!(
            input.update_hatch_hold(true),
            Some(Action::HatchHold { down: false })
        );
        assert_eq!(input.update_hatch_hold(false), None);
    }

    #[test]
    fn mode_cancellation_clears_queue_without_manufacturing_a_hatch_release() {
        let mut input = captured_input();
        input.cancel_queued_input();
        assert!(input.pending_actions.is_empty());
        assert!(input.feed_press_at.is_none());
        assert!(input.held_feed_at.is_none());
        assert!(input.held_fire_at.is_none());
        assert_eq!(input.update_hatch_hold(true), None);
        input.leave_hatch();
        assert_eq!(input.update_hatch_hold(true), None);
        assert_eq!(
            input.begin_hatch_hold(true),
            Action::HatchHold { down: true }
        );
    }

    #[test]
    fn feed_arming_requires_accepted_route_and_preserves_clock_origin() {
        let mut clock = StepClock::new(8);
        let mut input = InputCapture {
            feed_press_at: Some((clock.press(10.0), Some((300, 200)))),
            ..InputCapture::default()
        };
        let dropped = Event::FoodDropped {
            tick: 9,
            food_id: 17,
            balance: 200,
            potion: false,
        };
        input.arm_held_feed(&[dropped], true);
        assert!(input.held_feed_at.is_none());
        assert!(input.feed_press_at.is_some());
        let accepted = Event::GusInitialFeedAttempt {
            tick: 9,
            x: 300,
            y: 200,
        };
        input.arm_held_feed(std::slice::from_ref(&accepted), false);
        assert!(input.held_feed_at.is_none());
        clock.add_frame(0.028, false);
        clock.consume_step(false);
        input.arm_held_feed(&[accepted], true);
        assert!(input.feed_press_at.is_none());
        assert_eq!(clock.held_elapsed_ms(input.held_feed_at.unwrap(), 10.0), 56);
        input.cancel_holds();
        assert!(input.held_feed_at.is_none());
    }

    #[test]
    fn tank_help_restores_running_and_already_paused_callers() {
        for initially_paused in [false, true] {
            let mut pause = ModalPause::default();
            if initially_paused {
                pause.toggle();
            }
            pause.enter_help();
            assert!(pause.is_paused());
            assert_eq!(
                pause.finish_help(HelpExit::ReturnTo(HelpReturn::PausedTank)),
                None
            );
            assert_eq!(pause.is_paused(), initially_paused);
            // A second return cannot reuse a consumed restore value.
            pause.finish_help(HelpExit::ReturnTo(HelpReturn::PausedTank));
            assert!(pause.is_paused());
        }
    }

    #[test]
    fn selector_and_continue_help_exits_do_not_restore_tank_pause() {
        let mut pause = ModalPause::default();
        pause.enter_help();
        assert_eq!(
            pause.finish_help(HelpExit::ReturnTo(HelpReturn::Selector)),
            None
        );
        assert!(pause.is_paused());
        assert_eq!(
            pause.finish_help(HelpExit::Continue),
            Some(Action::Continue)
        );
        assert!(pause.is_paused());
        pause.finish_help(HelpExit::ReturnTo(HelpReturn::PausedTank));
        assert!(!pause.is_paused());
    }

    #[test]
    fn profile_reset_discards_help_restore_while_mode_resume_retains_it() {
        let mut pause = ModalPause::default();
        pause.enter_help();
        pause.resume_mode();
        assert!(!pause.is_paused());
        pause.finish_help(HelpExit::ReturnTo(HelpReturn::PausedTank));
        assert!(!pause.is_paused());
        pause.enter_help();
        pause.reset_profile();
        assert!(!pause.is_paused());
        pause.finish_help(HelpExit::ReturnTo(HelpReturn::PausedTank));
        assert!(pause.is_paused());
    }

    fn score(file_name: &str, properties: &str) -> SongData {
        SongData::parse(file_name, &format!("{properties}c4 q, d4 q\n")).unwrap()
    }

    fn feed_five(runtime: &mut FishSongRuntime, fish_id: u16, named: Option<NamedFish>) {
        for _ in 0..5 {
            runtime.record_feed(fish_id, named);
        }
    }

    #[test]
    fn fifth_feed_counts_trigger_step_and_samples_wall_clock_only_at_start() {
        let catalog = [score("ordinary.txt", "*attrib title = Ordinary\n")];
        let mut songs = FishSongRuntime::default();
        for _ in 0..4 {
            songs.record_feed(1, None);
        }
        songs.advance_pending(Some(&catalog), || panic!("No pending song."));
        songs.record_feed(1, None);
        for _ in 0..4 {
            songs.advance_pending(Some(&catalog), || panic!("Countdown still pending."));
        }
        let mut samples = [1.0, 7.5].into_iter();
        songs.advance_pending(Some(&catalog), || samples.next().unwrap());
        assert_eq!(songs.current_song_id(), 1);
        assert_eq!(songs.title(12.499), Some("Ordinary"));
        assert_eq!(songs.title(12.5), None);
        let commands = songs.advance_frame(1_000, true);
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].song_id, 1);
        assert_eq!(commands[0].pitch_semitones, 0.0);
    }

    #[test]
    fn latest_eligible_feed_in_a_step_owns_pending_special_selection() {
        let catalog = [
            score("santa.txt", "*attrib santa\n*attrib title = Santa\n"),
            score("kilgore.txt", "*attrib kilgore\n*attrib title = Kilgore\n"),
            score(
                "test.txt",
                "*attrib kilgore\n*attrib test\n*attrib title = Test\n",
            ),
        ];
        let mut songs = FishSongRuntime::default();
        feed_five(&mut songs, 1, Some(NamedFish::Santa));
        feed_five(&mut songs, 2, Some(NamedFish::Kilgore));
        for _ in 0..5 {
            songs.advance_pending(Some(&catalog), || 1.0);
        }
        assert_eq!(songs.title(1.0), Some("Kilgore"));
        assert_eq!(songs.current_song_id(), 1);
    }

    #[test]
    fn inactive_audio_frames_pause_notes_and_resume_without_wall_time_catchup() {
        let catalog = [score("ordinary.txt", "")];
        let mut songs = FishSongRuntime::default();
        feed_five(&mut songs, 1, None);
        for _ in 0..5 {
            songs.advance_pending(Some(&catalog), || 1.0);
        }
        assert_eq!(songs.advance_frame(1_000, true).len(), 1);
        assert!(songs.advance_frame(1_020, false).is_empty());
        assert!(songs.advance_frame(5_000, false).is_empty());
        assert!(songs.advance_frame(5_000, true).is_empty());
        assert!(songs.advance_frame(5_123, true).is_empty());
        let commands = songs.advance_frame(5_124, true);
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].pitch_semitones, 2.0);
    }

    #[test]
    fn profile_reset_clears_playback_and_feeds_but_keeps_sequence_and_ids() {
        let catalog = [
            score("melody_short.txt", "*attrib title = Ordinary\n"),
            score("melody_long.txt", "*attrib title = Extended\n"),
        ];
        let mut songs = FishSongRuntime::default();
        for expected_id in 1..=5 {
            feed_five(&mut songs, 1, None);
            for _ in 0..5 {
                songs.advance_pending(Some(&catalog), || 1.0);
            }
            assert_eq!(songs.current_song_id(), expected_id);
            assert_eq!(
                songs.title(1.0),
                Some(if expected_id == 5 {
                    "Extended (extended)"
                } else {
                    "Ordinary"
                })
            );
            // A busy song must not schedule another countdown.
            feed_five(&mut songs, 1, None);
            assert!(songs.pending_song_ticks.is_none());
            songs.reset_profile();
            assert_eq!(songs.current_song_id(), expected_id);
            assert!(songs.advance_frame(1_000, true).is_empty());
            assert_eq!(songs.title(1.0), Some(""));
            assert_eq!(songs.title(6.0), None);
        }
    }

    #[test]
    fn missing_catalog_consumes_countdown_without_start_or_clock_sampling() {
        let mut songs = FishSongRuntime::default();
        feed_five(&mut songs, 1, Some(NamedFish::Santa));
        for _ in 0..5 {
            songs.advance_pending(None, || panic!("No catalog."));
        }
        assert_eq!(songs.current_song_id(), 0);
        assert_eq!(songs.song_sequence, 0);
        assert!(songs.pending_song_ticks.is_none());
        assert_eq!(songs.pending_song_kind, Some(NamedFish::Santa));
        assert!(songs.advance_frame(1_000, true).is_empty());
    }
}
