//! Order-based tracker music from the user's owned installation.
//! The decoder and output device live on one dedicated thread. Source cue
//! mappings are secondary-source-derived; retail mixing/fades remain unproved.

use crate::adventure::{AdventurePhase, AdventureSession};
use rodio::{Decoder, DeviceSinkBuilder, Player, Source, buffer::SamplesBuffer};
use serde::Serialize;
use std::{
    fs,
    io::Cursor,
    num::{NonZeroU16, NonZeroU32},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
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
    Menu,
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
            Self::Menu => ("Insaniq2.mo3", 0),
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
            AdventurePhase::BonusResults { .. }
            | AdventurePhase::Hatch { .. }
            | AdventurePhase::TankFourFinaleHatch { .. } => Some(Self::Hatch),
            AdventurePhase::PetSelection { .. }
            | AdventurePhase::PetSelectionConfirmation { .. }
            | AdventurePhase::TimeTrialPetSelection { .. }
            | AdventurePhase::TimeTrialTankSelection
            | AdventurePhase::TimeTrialTimesUp
            | AdventurePhase::TimeTrialResults
            | AdventurePhase::ChallengeTankSelection
            | AdventurePhase::ChallengePetSelection { .. }
            | AdventurePhase::ChallengePetSelectionConfirmation { .. }
            | AdventurePhase::ChallengeResults => Some(Self::PetSelection),
            AdventurePhase::Playing
            | AdventurePhase::TimeTrialPlaying
            | AdventurePhase::TimeTrialPrestoDialog { .. }
            | AdventurePhase::AdventurePrestoDialog { .. }
            | AdventurePhase::TimeTrialInvasionTutorial { .. }
            | AdventurePhase::ChallengePlaying
            | AdventurePhase::SandboxPlaying
            | AdventurePhase::SandboxPrestoDialog { .. }
            | AdventurePhase::ChallengePrestoDialog { .. }
            | AdventurePhase::ChallengeInvasionTutorial { .. }
            | AdventurePhase::FirstTankRescue
            | AdventurePhase::InvasionTutorial { .. } => {
                let board = if matches!(
                    session.phase,
                    AdventurePhase::SandboxPlaying | AdventurePhase::SandboxPrestoDialog { .. }
                ) {
                    session.sandbox_board.as_ref()?
                } else {
                    session.board.as_ref()?
                };
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
            AdventurePhase::AdventureFinaleInterlude
            | AdventurePhase::ChallengeInterlude { .. } => Some(Self::Interlude),
            AdventurePhase::VirtualTank | AdventurePhase::VirtualPrestoDialog { .. } => {
                let combat = &session.virtual_board.as_ref()?.combat;
                if combat.wave.has_live_alien() || !combat.missiles.is_empty() {
                    Some(Self::AlienBattle)
                } else if combat.wave.countdown == 1 {
                    Some(Self::AlienWarning)
                } else if (2..=275).contains(&combat.wave.countdown) {
                    Some(Self::AlienApproach)
                } else {
                    // WinFish Board::StartMusic stops the tracker in a quiet
                    // Virtual Tank, including after an encounter retires.
                    None
                }
            }
            AdventurePhase::VirtualStore
            | AdventurePhase::VirtualPurchaseConfirm { .. }
            | AdventurePhase::VirtualSetup
            | AdventurePhase::VirtualRename { .. }
            | AdventurePhase::VirtualFishSaleConfirm { .. }
            | AdventurePhase::VirtualBackdropSaleConfirm => Some(Self::PetSelection),
            AdventurePhase::GameOver { .. }
            | AdventurePhase::TimeTrialGameOver { .. }
            | AdventurePhase::ChallengeLost => None,
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

struct AudioGains {
    music: AtomicU32,
    effects: AtomicU32,
}

impl AudioGains {
    fn full() -> Self {
        Self {
            music: AtomicU32::new(1.0_f32.to_bits()),
            effects: AtomicU32::new(1.0_f32.to_bits()),
        }
    }
}

struct DecoderShared {
    stop: Arc<AtomicBool>,
    failure: Arc<Mutex<Option<String>>>,
    dropped_reports: Arc<AtomicU64>,
    gains: Arc<AudioGains>,
}

enum EffectCommand {
    Play {
        bytes: Arc<[u8]>,
        gain: f32,
        speed: f32,
        song_id: Option<i32>,
    },
    StopSong(i32),
}

struct ActiveEffect {
    player: Player,
    gain: f32,
    song_id: Option<i32>,
}

pub struct MusicOwner {
    controls: SyncSender<DesiredState>,
    effects: SyncSender<EffectCommand>,
    gains: Arc<AudioGains>,
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
        let gains = Arc::new(AudioGains::full());
        let worker_stop = stop.clone();
        let worker_failure = failure.clone();
        let worker_dropped_reports = dropped_reports.clone();
        let worker_gains = gains.clone();
        let thread = thread::Builder::new()
            .name("turbofish-music".into())
            .spawn(move || {
                run_decoder(
                    game_root,
                    commands,
                    effect_commands,
                    reports_sender,
                    DecoderShared {
                        stop: worker_stop,
                        failure: worker_failure,
                        dropped_reports: worker_dropped_reports,
                        gains: worker_gains,
                    },
                );
            })?;
        Ok(Self {
            controls,
            effects,
            gains,
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
        self.sync_cue(MusicCue::for_session(session), paused);
    }

    /// Frontend-owned transient scenes can choose a cue without duplicating
    /// decoder ownership or changing the durable gameplay session.
    pub fn sync_cue(&mut self, cue: Option<MusicCue>, paused: bool) {
        if self.unavailable.is_some() {
            return;
        }
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
        self.send_effect(EffectCommand::Play {
            bytes,
            gain: 1.0,
            speed: 1.0,
            song_id: None,
        })
    }

    pub(crate) fn play_song_command(
        &self,
        bytes: Arc<[u8]>,
        command: crate::fish_songs::ToneCommand,
    ) -> Option<MusicReport> {
        if command.sample == crate::fish_songs::ToneSample::Applause {
            self.play_effect(bytes)
        } else {
            self.play_tone(
                bytes,
                command.song_id,
                command.pitch_semitones,
                command.gain,
            )
        }
    }

    pub fn set_music_volume(&self, value: f32) -> Result<(), String> {
        set_gain(&self.gains.music, value)
    }

    pub fn set_effects_volume(&self, value: f32) -> Result<(), String> {
        set_gain(&self.gains.effects, value)
    }

    /// A score voice uses a bounded effect instance on the existing output
    /// thread. Pitch changes sample rate; source samples play to completion.
    /// Score durations schedule the next note in FishSongPlayer.
    pub fn play_tone(
        &self,
        bytes: Arc<[u8]>,
        song_id: i32,
        pitch_semitones: f32,
        gain: f32,
    ) -> Option<MusicReport> {
        if song_id < 0
            || !pitch_semitones.is_finite()
            || !(-96.0..=96.0).contains(&pitch_semitones)
            || !gain.is_finite()
            || !(0.0..=1.0).contains(&gain)
        {
            return Some(MusicReport::EffectFailed {
                reason: "Invalid fish-song tone parameters".into(),
            });
        }
        self.send_effect(EffectCommand::Play {
            bytes,
            gain,
            speed: 2.0_f32.powf(pitch_semitones / 12.0),
            song_id: Some(song_id),
        })
    }

    pub fn stop_song(&self, song_id: i32) -> Option<MusicReport> {
        self.send_effect(EffectCommand::StopSong(song_id))
    }

    fn send_effect(&self, command: EffectCommand) -> Option<MusicReport> {
        match self.effects.try_send(command) {
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

fn set_gain(destination: &AtomicU32, value: f32) -> Result<(), String> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err("Audio volume must be between zero and one".into());
    }
    destination.store(value.to_bits(), Ordering::Relaxed);
    Ok(())
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
    effect_commands: Receiver<EffectCommand>,
    reports: SyncSender<MusicReport>,
    shared: DecoderShared,
) {
    let DecoderShared {
        stop,
        failure,
        dropped_reports,
        gains,
    } = shared;
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
    let mut effects: Vec<ActiveEffect> = Vec::new();
    loop {
        if stop.load(Ordering::Relaxed) || callback_failed.load(Ordering::Relaxed) {
            break;
        }
        effects.retain(|effect| !effect.player.empty());
        let music_gain = f32::from_bits(gains.music.load(Ordering::Relaxed));
        let effects_gain = f32::from_bits(gains.effects.load(Ordering::Relaxed));
        player.set_volume(0.30 * music_gain);
        for effect in &effects {
            effect.player.set_volume(0.75 * effects_gain * effect.gain);
        }
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
                    player.set_volume(0.30 * music_gain);
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
            Ok(EffectCommand::StopSong(song_id)) => {
                did_work = true;
                effects.retain(|effect| {
                    if effect.song_id == Some(song_id) {
                        effect.player.stop();
                        false
                    } else {
                        true
                    }
                });
            }
            Ok(EffectCommand::Play {
                bytes,
                gain,
                speed,
                song_id,
            }) => match decode_effect(bytes) {
                Ok(decoded) => {
                    did_work = true;
                    if effects.len() < MAX_ACTIVE_EFFECTS {
                        let effect = Player::connect_new(stream.mixer());
                        effect.set_volume(0.75 * effects_gain * gain);
                        effect.append(decoded.speed(speed));
                        effects.push(ActiveEffect {
                            player: effect,
                            gain,
                            song_id,
                        });
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
        effect.player.stop();
    }
    drop(player);
    drop(stream);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::invasion::Invasion1_2;
    use crate::missile::ClassicMissile;
    use crate::virtual_board::VirtualBoardState;

    fn owner_without_device() -> (MusicOwner, Receiver<DesiredState>) {
        let (controls, commands) = mpsc::sync_channel(MAX_COMMAND_BACKLOG);
        let (effects, _effect_commands) = mpsc::sync_channel(MAX_COMMAND_BACKLOG);
        let (_report_sender, reports) = mpsc::sync_channel(MAX_REPORT_BACKLOG);
        (
            MusicOwner {
                controls,
                effects,
                gains: Arc::new(AudioGains::full()),
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
    fn virtual_tank_routes_only_encounters_and_selection_screens() {
        let mut session = AdventureSession::new(1);
        session.virtual_board = Some(VirtualBoardState::new(&session.virtual_tank));
        session.phase = AdventurePhase::VirtualTank;
        let combat = &mut session.virtual_board.as_mut().unwrap().combat;
        combat.wave.countdown = 3_000;
        assert_eq!(MusicCue::for_session(&session), None);

        session
            .virtual_board
            .as_mut()
            .unwrap()
            .combat
            .wave
            .countdown = 275;
        assert_eq!(
            MusicCue::for_session(&session),
            Some(MusicCue::AlienApproach)
        );
        session
            .virtual_board
            .as_mut()
            .unwrap()
            .combat
            .wave
            .countdown = 1;
        assert_eq!(
            MusicCue::for_session(&session),
            Some(MusicCue::AlienWarning)
        );
        session
            .virtual_board
            .as_mut()
            .unwrap()
            .combat
            .missiles
            .push(ClassicMissile::launch(1, 2, 100, 100, 0));
        session.phase = AdventurePhase::VirtualPrestoDialog { pressed: None };
        assert_eq!(MusicCue::for_session(&session), Some(MusicCue::AlienBattle));

        session.phase = AdventurePhase::VirtualStore;
        assert_eq!(
            MusicCue::for_session(&session),
            Some(MusicCue::PetSelection)
        );
        session.phase = AdventurePhase::VirtualSetup;
        assert_eq!(
            MusicCue::for_session(&session),
            Some(MusicCue::PetSelection)
        );
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
    fn fish_song_applause_uses_effect_ownership_and_tones_keep_validation() {
        use crate::fish_songs::{FishSongPlayer, SongData, ToneCommand, ToneSample};

        let (mut owner, _controls) = owner_without_device();
        let (effects, commands) = mpsc::sync_channel(MAX_COMMAND_BACKLOG);
        owner.effects = effects;
        let bytes = Arc::<[u8]>::from(b"queue fixture".as_slice());
        let score = SongData::parse("applause.txt", "*speed 2\n*line 1\nc4 q\n").unwrap();
        let mut player = FishSongPlayer::default();
        player.start(&score, 7, 1_000, true).unwrap();
        let tone = player.advance(1_000)[0];
        assert!(owner.play_song_command(bytes.clone(), tone).is_none());
        assert!(matches!(
            commands.try_recv().unwrap(),
            EffectCommand::Play {
                song_id: Some(7),
                ..
            }
        ));

        let applause = player.advance(1_400)[0];
        assert_eq!(applause.sample, ToneSample::Applause);
        assert_eq!(applause.song_id, -1);
        assert!(owner.play_song_command(bytes.clone(), applause).is_none());
        assert!(matches!(
            commands.try_recv().unwrap(),
            EffectCommand::Play {
                song_id: None,
                gain: 1.0,
                speed: 1.0,
                ..
            }
        ));

        for sample in [
            ToneSample::Normal,
            ToneSample::High,
            ToneSample::SuperHigh,
            ToneSample::Low,
        ] {
            let command = ToneCommand {
                sample,
                pitch_semitones: 12.0,
                gain: 0.5,
                ..tone
            };
            assert!(owner.play_song_command(bytes.clone(), command).is_none());
            assert!(matches!(
                commands.try_recv().unwrap(),
                EffectCommand::Play {
                    song_id: Some(7),
                    gain: 0.5,
                    speed: 2.0,
                    ..
                }
            ));
        }
        for invalid in [
            ToneCommand {
                song_id: -1,
                ..tone
            },
            ToneCommand {
                pitch_semitones: f32::NAN,
                ..tone
            },
            ToneCommand {
                pitch_semitones: 97.0,
                ..tone
            },
            ToneCommand {
                gain: f32::INFINITY,
                ..tone
            },
            ToneCommand { gain: 2.0, ..tone },
        ] {
            assert!(matches!(
                owner.play_song_command(bytes.clone(), invalid),
                Some(MusicReport::EffectFailed { .. })
            ));
            assert!(matches!(
                commands.try_recv(),
                Err(mpsc::TryRecvError::Empty)
            ));
        }
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
