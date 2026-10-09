//! Order-based tracker music from the user's owned installation.
//! The decoder and output device live on one dedicated thread. Source cue
//! mappings are secondary-source-derived; retail mixing/fades remain unproved.

use crate::adventure::{AdventurePhase, AdventureSession};
use rodio::{Decoder, DeviceSinkBuilder, Player, buffer::SamplesBuffer};
use serde::Serialize;
use std::{
    fs,
    io::Cursor,
    num::{NonZeroU16, NonZeroU32},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use turbofish_openmpt::Module;

const SAMPLE_RATE: i32 = 48_000;
const BLOCK_FRAMES: usize = 4_096;
const MAX_QUEUED_BLOCKS: usize = 2;
const MAX_ACTIVE_EFFECTS: usize = 32;
const MAX_COMMAND_BACKLOG: usize = 64;
const MAX_REPORT_BACKLOG: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicCue {
    Tank1,
    Tank2,
    Tank3,
    Tank4,
    Bonus,
    Hatch,
    PetSelection,
    Interlude,
    Lullaby,
    AlienApproach,
    AlienWarning,
    AlienBattle,
}

impl MusicCue {
    pub fn source(self) -> (&'static str, u32) {
        match self {
            Self::Tank1 => ("Insaniq2.mo3", 0),
            Self::Tank2 => ("Insaniq2.mo3", 12),
            Self::Tank3 => ("Insaniq2.mo3", 25),
            Self::Tank4 => ("Insaniq2.mo3", 37),
            Self::Bonus => ("Insaniq2.mo3", 58),
            Self::Hatch => ("Insaniq2.mo3", 49),
            Self::PetSelection => ("Insaniq2.mo3", 45),
            Self::Interlude => ("Insaniq2.mo3", 50),
            Self::Lullaby => ("Lullaby.mo3", 0),
            Self::AlienApproach => ("Alien.mo3", 0),
            Self::AlienWarning => ("Alien.mo3", 1),
            Self::AlienBattle => ("Alien.mo3", 3),
        }
    }

    pub fn for_session(session: &AdventureSession) -> Option<Self> {
        match &session.phase {
            AdventurePhase::Bonus { .. } => Some(Self::Bonus),
            AdventurePhase::BonusResults { .. } | AdventurePhase::Hatch { .. } => Some(Self::Hatch),
            AdventurePhase::PetSelection { .. }
            | AdventurePhase::PetSelectionConfirmation { .. } => Some(Self::PetSelection),
            AdventurePhase::Playing
            | AdventurePhase::FirstTankRescue
            | AdventurePhase::InvasionTutorial { .. } => {
                let board = session.board.as_ref()?;
                if let Some(wave) = &board.invasion {
                    if wave.has_live_alien() || !board.missiles.is_empty() {
                        return Some(Self::AlienBattle);
                    }
                    if wave.countdown == 1 {
                        return Some(Self::AlienWarning);
                    }
                    if (2..=275).contains(&wave.countdown) {
                        return Some(Self::AlienApproach);
                    }
                }
                match board.tank {
                    1 => Some(Self::Tank1),
                    2 => Some(Self::Tank2),
                    3 => Some(Self::Tank3),
                    4 => Some(Self::Tank4),
                    _ => None,
                }
            }
            AdventurePhase::GameSelector | AdventurePhase::HelpScreen => Some(Self::PetSelection),
            AdventurePhase::GameOver { .. } => None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MusicReport {
    Cue {
        file: &'static str,
        requested_order: u32,
        discarded_blocks: usize,
    },
    Queue {
        file: &'static str,
        requested_order: u32,
        rendered_blocks: u64,
        queued_blocks: usize,
        underruns: u64,
    },
    Paused {
        paused: bool,
    },
    EffectFailed {
        reason: String,
    },
    Overload {
        reason: String,
    },
    Stopped,
    Unavailable {
        reason: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DesiredState {
    cue: Option<MusicCue>,
    paused: bool,
}

pub struct MusicOwner {
    controls: SyncSender<DesiredState>,
    effects: SyncSender<Arc<[u8]>>,
    reports: Receiver<MusicReport>,
    stop: Arc<AtomicBool>,
    failure: Arc<Mutex<Option<String>>>,
    dropped_reports: Arc<AtomicU64>,
    thread: Option<JoinHandle<()>>,
    requested: Option<MusicCue>,
    paused: bool,
    unavailable: Option<String>,
}

impl MusicOwner {
    pub fn start(game_root: PathBuf) -> Result<Self, std::io::Error> {
        let (controls, commands) = mpsc::sync_channel(MAX_COMMAND_BACKLOG);
        let (effects, effect_commands) = mpsc::sync_channel(MAX_COMMAND_BACKLOG);
        let (reports_sender, reports) = mpsc::sync_channel(MAX_REPORT_BACKLOG);
        let stop = Arc::new(AtomicBool::new(false));
        let failure = Arc::new(Mutex::new(None));
        let dropped_reports = Arc::new(AtomicU64::new(0));
        let worker_stop = stop.clone();
        let worker_failure = failure.clone();
        let worker_dropped_reports = dropped_reports.clone();
        let thread = thread::Builder::new()
            .name("turbofish-music".into())
            .spawn(move || {
                run_decoder(
                    game_root,
                    commands,
                    effect_commands,
                    reports_sender,
                    worker_stop,
                    worker_failure,
                    worker_dropped_reports,
                );
            })?;
        Ok(Self {
            controls,
            effects,
            reports,
            stop,
            failure,
            dropped_reports,
            thread: Some(thread),
            requested: None,
            paused: false,
            unavailable: None,
        })
    }

    pub fn sync(&mut self, session: &AdventureSession, paused: bool) {
        if self.unavailable.is_some() {
            return;
        }
        let cue = MusicCue::for_session(session);
        if cue == self.requested && paused == self.paused {
            return;
        }
        match self.controls.try_send(DesiredState { cue, paused }) {
            Ok(()) => {
                self.requested = cue;
                self.paused = paused;
            }
            Err(TrySendError::Full(_)) => {
                self.unavailable = Some("Music state queue overloaded".into());
                self.stop.store(true, Ordering::Relaxed);
            }
            Err(TrySendError::Disconnected(_)) => {
                self.unavailable = Some("Music decoder thread stopped".into());
            }
        }
    }

    pub fn drain_reports(&mut self) -> Vec<MusicReport> {
        let mut reports: Vec<_> = self.reports.try_iter().collect();
        let dropped = self.dropped_reports.swap(0, Ordering::Relaxed);
        if dropped > 0 {
            reports.push(MusicReport::Overload {
                reason: format!("{dropped} music reports dropped from bounded telemetry queue"),
            });
        }
        let worker_failure = self.failure.lock().expect("music failure lock").clone();
        if let Some(reason) = worker_failure
            && !reports.iter().any(|report| matches!(report, MusicReport::Unavailable { reason: reported } if reported == &reason))
            && self.unavailable.as_deref() != Some(&reason)
        {
            reports.push(MusicReport::Unavailable { reason });
        }
        for report in &reports {
            if let MusicReport::Unavailable { reason } = report {
                self.unavailable = Some(reason.clone());
            }
        }
        reports
    }

    pub fn unavailable(&self) -> Option<&str> {
        self.unavailable.as_deref()
    }

    pub fn play_effect(&self, bytes: Arc<[u8]>) -> Option<MusicReport> {
        match self.effects.try_send(bytes) {
            Ok(()) => None,
            Err(TrySendError::Full(_)) => Some(MusicReport::Overload {
                reason: "Sound effect queue full; effect dropped".into(),
            }),
            Err(TrySendError::Disconnected(_)) => Some(MusicReport::EffectFailed {
                reason: "Sound output thread stopped".into(),
            }),
        }
    }
}

impl Drop for MusicOwner {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn stop_and_drain(player: &Player) -> Result<usize, String> {
    let discarded = player.len();
    // Rodio drains asynchronously; silence the player before temporarily
    // resuming it so a pending Pause+Cue cannot leak old queued music.
    player.set_volume(0.0);
    player.play();
    player.stop();
    let deadline = Instant::now() + Duration::from_millis(500);
    while !player.empty() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    if !player.empty() {
        return Err(format!(
            "Music queue did not drain: {} blocks",
            player.len()
        ));
    }
    Ok(discarded)
}

fn emit_report(reports: &SyncSender<MusicReport>, dropped: &AtomicU64, report: MusicReport) {
    if reports.try_send(report).is_err() {
        dropped.fetch_add(1, Ordering::Relaxed);
    }
}

fn decode_effect(bytes: Arc<[u8]>) -> Result<Decoder<Cursor<Arc<[u8]>>>, MusicReport> {
    Decoder::try_from(Cursor::new(bytes)).map_err(|error| MusicReport::EffectFailed {
        reason: format!("Sound effect decode failed: {error}"),
    })
}

// One pass must return to command/stop handling even if output consumes
// blocks faster than the decoder can produce them.
fn bounded_render_pass(mut render_one: impl FnMut() -> bool) -> usize {
    let mut rendered = 0;
    for _ in 0..MAX_QUEUED_BLOCKS {
        if !render_one() {
            break;
        }
        rendered += 1;
    }
    rendered
}

fn report_unavailable(
    reason: String,
    reports: &SyncSender<MusicReport>,
    failure: &Mutex<Option<String>>,
    dropped: &AtomicU64,
) {
    *failure.lock().expect("music failure lock") = Some(reason.clone());
    emit_report(reports, dropped, MusicReport::Unavailable { reason });
}

fn run_decoder(
    game_root: PathBuf,
    commands: Receiver<DesiredState>,
    effect_commands: Receiver<Arc<[u8]>>,
    reports: SyncSender<MusicReport>,
    stop: Arc<AtomicBool>,
    failure: Arc<Mutex<Option<String>>>,
    dropped_reports: Arc<AtomicU64>,
) {
    let callback_reports = reports.clone();
    let callback_failure = failure.clone();
    let callback_dropped = dropped_reports.clone();
    let callback_failed = Arc::new(AtomicBool::new(false));
    let callback_failure_flag = callback_failed.clone();
    let builder = match DeviceSinkBuilder::from_default_device() {
        Ok(builder) => builder,
        Err(error) => {
            report_unavailable(
                format!("Music device unavailable: {error}"),
                &reports,
                &failure,
                &dropped_reports,
            );
            return;
        }
    };
    let stream = match builder
        .with_error_callback(move |error| {
            callback_failure_flag.store(true, Ordering::Relaxed);
            report_unavailable(
                format!("Music output callback: {error}"),
                &callback_reports,
                &callback_failure,
                &callback_dropped,
            );
        })
        .open_stream()
    {
        Ok(stream) => stream,
        Err(error) => {
            report_unavailable(
                format!("Music device unavailable: {error}"),
                &reports,
                &failure,
                &dropped_reports,
            );
            return;
        }
    };
    let player = Player::connect_new(stream.mixer());
    player.set_volume(0.30); // Local music player gain; effects use their own mixer path.
    player.pause();
    let mut module: Option<Module> = None;
    let mut current_cue: Option<MusicCue> = None;
    let mut paused = false;
    let mut rendered_blocks = 0_u64;
    let mut underruns = 0_u64;
    let mut effects: Vec<Player> = Vec::new();
    loop {
        if stop.load(Ordering::Relaxed) || callback_failed.load(Ordering::Relaxed) {
            break;
        }
        effects.retain(|effect| !effect.empty());
        let mut did_work = false;
        match commands.try_recv() {
            Ok(desired) => {
                did_work = true;
                let pause_changed = paused != desired.paused;
                paused = desired.paused;
                if paused {
                    player.pause();
                }
                let cue = desired.cue;
                if cue != current_cue {
                    let discarded = match stop_and_drain(&player) {
                        Ok(discarded) => discarded,
                        Err(reason) => {
                            report_unavailable(reason, &reports, &failure, &dropped_reports);
                            break;
                        }
                    };
                    module = None;
                    current_cue = cue;
                    rendered_blocks = 0;
                    underruns = 0;
                    if let Some(cue) = cue {
                        let (file, order) = cue.source();
                        let load = (|| -> Result<Module, String> {
                            let bytes = fs::read(game_root.join("music").join(file))
                                .map_err(|error| format!("{file}: {error}"))?;
                            let mut module = Module::from_bytes(&bytes)
                                .map_err(|error| format!("{file}: {error}"))?;
                            module
                                .set_repeat_count(-1)
                                .map_err(|error| error.to_string())?;
                            module.seek(order, 0).map_err(|error| error.to_string())?;
                            Ok(module)
                        })();
                        match load {
                            Ok(loaded) => {
                                module = Some(loaded);
                                emit_report(
                                    &reports,
                                    &dropped_reports,
                                    MusicReport::Cue {
                                        file,
                                        requested_order: order,
                                        discarded_blocks: discarded,
                                    },
                                );
                            }
                            Err(reason) => {
                                report_unavailable(reason, &reports, &failure, &dropped_reports);
                                current_cue = None;
                            }
                        }
                    } else {
                        emit_report(&reports, &dropped_reports, MusicReport::Stopped);
                    }
                }
                if paused {
                    player.pause();
                } else {
                    player.set_volume(0.30);
                    player.play();
                }
                if pause_changed {
                    emit_report(&reports, &dropped_reports, MusicReport::Paused { paused });
                }
            }
            Err(mpsc::TryRecvError::Disconnected) => break,
            Err(mpsc::TryRecvError::Empty) => {}
        }
        if stop.load(Ordering::Relaxed) {
            break;
        }
        match effect_commands.try_recv() {
            Ok(bytes) => match decode_effect(bytes) {
                Ok(decoded) => {
                    did_work = true;
                    if effects.len() < MAX_ACTIVE_EFFECTS {
                        let effect = Player::connect_new(stream.mixer());
                        effect.set_volume(0.75);
                        effect.append(decoded);
                        effects.push(effect);
                    } else {
                        emit_report(
                            &reports,
                            &dropped_reports,
                            MusicReport::Overload {
                                reason: "Too many simultaneous sound effects; effect dropped"
                                    .into(),
                            },
                        );
                    }
                }
                Err(report) => emit_report(&reports, &dropped_reports, report),
            },
            Err(mpsc::TryRecvError::Disconnected | mpsc::TryRecvError::Empty) => {}
        }
        if module.is_none() {
            if !did_work {
                thread::sleep(Duration::from_millis(5));
            }
            continue;
        }
        if paused {
            if !did_work {
                thread::sleep(Duration::from_millis(5));
            }
            continue;
        }
        if player.empty() && rendered_blocks > 0 {
            underruns += 1;
        }
        bounded_render_pass(|| {
            if stop.load(Ordering::Relaxed)
                || callback_failed.load(Ordering::Relaxed)
                || player.len() >= MAX_QUEUED_BLOCKS
            {
                return false;
            }
            did_work = true;
            let mut pcm = vec![0.0_f32; BLOCK_FRAMES * 2];
            let frames = match module
                .as_mut()
                .expect("module checked above")
                .render_stereo(SAMPLE_RATE, &mut pcm)
            {
                Ok(frames) if frames > 0 => frames,
                Ok(_) => {
                    report_unavailable(
                        "Music reached an unexpected end".into(),
                        &reports,
                        &failure,
                        &dropped_reports,
                    );
                    let _ = stop_and_drain(&player);
                    module = None;
                    return false;
                }
                Err(error) => {
                    report_unavailable(
                        format!("Music render failed: {error}"),
                        &reports,
                        &failure,
                        &dropped_reports,
                    );
                    let _ = stop_and_drain(&player);
                    module = None;
                    return false;
                }
            };
            pcm.truncate(frames * 2);
            player.append(SamplesBuffer::new(
                NonZeroU16::new(2).expect("stereo"),
                NonZeroU32::new(SAMPLE_RATE as u32).expect("nonzero sample rate"),
                pcm,
            ));
            rendered_blocks += 1;
            if (rendered_blocks == 1 || rendered_blocks.is_multiple_of(64))
                && let Some(cue) = current_cue
            {
                let (file, order) = cue.source();
                emit_report(
                    &reports,
                    &dropped_reports,
                    MusicReport::Queue {
                        file,
                        requested_order: order,
                        rendered_blocks,
                        queued_blocks: player.len(),
                        underruns,
                    },
                );
            }
            true
        });
        if !did_work {
            thread::sleep(Duration::from_millis(5));
        }
    }
    let _ = stop_and_drain(&player);
    for effect in &effects {
        effect.stop();
    }
    drop(player);
    drop(stream);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invasion::Invasion1_2;

    fn owner_without_device() -> (MusicOwner, Receiver<DesiredState>) {
        let (controls, commands) = mpsc::sync_channel(MAX_COMMAND_BACKLOG);
        let (effects, _effect_commands) = mpsc::sync_channel(MAX_COMMAND_BACKLOG);
        let (_report_sender, reports) = mpsc::sync_channel(MAX_REPORT_BACKLOG);
        (
            MusicOwner {
                controls,
                effects,
                reports,
                stop: Arc::new(AtomicBool::new(false)),
                failure: Arc::new(Mutex::new(None)),
                dropped_reports: Arc::new(AtomicU64::new(0)),
                thread: None,
                requested: None,
                paused: false,
                unavailable: None,
            },
            commands,
        )
    }

    #[test]
    fn source_orders_match_recovered_music_calls() {
        assert_eq!(MusicCue::Tank1.source(), ("Insaniq2.mo3", 0));
        assert_eq!(MusicCue::Tank2.source(), ("Insaniq2.mo3", 12));
        assert_eq!(MusicCue::PetSelection.source(), ("Insaniq2.mo3", 45));
        assert_eq!(MusicCue::Hatch.source(), ("Insaniq2.mo3", 49));
        assert_eq!(MusicCue::Bonus.source(), ("Insaniq2.mo3", 58));
        assert_eq!(MusicCue::AlienApproach.source(), ("Alien.mo3", 0));
        assert_eq!(MusicCue::AlienWarning.source(), ("Alien.mo3", 1));
        assert_eq!(MusicCue::AlienBattle.source(), ("Alien.mo3", 3));
    }

    #[test]
    fn countdown_one_warning_survives_catchup_and_pause_is_atomic_with_cue() {
        let mut session = AdventureSession::new(1);
        session.board.as_mut().unwrap().invasion = Some(Invasion1_2::new());
        let (mut owner, commands) = owner_without_device();
        session
            .board
            .as_mut()
            .unwrap()
            .invasion
            .as_mut()
            .unwrap()
            .countdown = 2;
        owner.sync(&session, false);
        session
            .board
            .as_mut()
            .unwrap()
            .invasion
            .as_mut()
            .unwrap()
            .countdown = 1;
        owner.sync(&session, true);
        session
            .board
            .as_mut()
            .unwrap()
            .invasion
            .as_mut()
            .unwrap()
            .countdown = 2;
        owner.sync(&session, true);
        assert_eq!(
            commands.try_recv().unwrap(),
            DesiredState {
                cue: Some(MusicCue::AlienApproach),
                paused: false
            }
        );
        assert_eq!(
            commands.try_recv().unwrap(),
            DesiredState {
                cue: Some(MusicCue::AlienWarning),
                paused: true
            }
        );
        assert_eq!(
            commands.try_recv().unwrap(),
            DesiredState {
                cue: Some(MusicCue::AlienApproach),
                paused: true
            }
        );
    }

    #[test]
    fn full_control_queue_fails_explicitly_and_prioritizes_stop() {
        let mut session = AdventureSession::new(1);
        let (mut owner, _commands) = owner_without_device();
        for index in 0..MAX_COMMAND_BACKLOG {
            session.phase = if index % 2 == 0 {
                AdventurePhase::GameSelector
            } else {
                AdventurePhase::Playing
            };
            owner.sync(&session, false);
        }
        session.phase = AdventurePhase::GameSelector;
        owner.sync(&session, false);
        assert_eq!(owner.unavailable(), Some("Music state queue overloaded"));
        assert!(owner.stop.load(Ordering::Relaxed));
    }

    #[test]
    fn corrupt_effect_is_a_local_decode_failure() {
        let result = decode_effect(Arc::<[u8]>::from(b"bad audio".as_slice()));
        assert!(matches!(result, Err(MusicReport::EffectFailed { .. })));
    }

    #[test]
    fn decoder_returns_to_controls_when_output_never_fills() {
        let mut rendered = 0;
        let completed = bounded_render_pass(|| {
            // A perpetually empty output queue is the starvation case:
            // every requested block could be consumed before the next check.
            rendered += 1;
            true
        });
        assert_eq!(completed, 2);
        assert_eq!(rendered, 2);
    }
}
