//! Durable mode scenes. Profile progress, wallet and scores remain on the session.

use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetainedScene {
    phase: AdventurePhase,
    board: Option<AdventureState>,
    next_seed: u64,
    time_trial: Option<TimeTrialRun>,
    challenge: Option<ChallengeRun>,
    selected_pets: Vec<PetKind>,
}

impl AdventureSession {
    /// Result and confirmation screens bind a receipt to the current wallet.
    /// A cross-mode spend must wait until that receipt has been dismissed.
    pub fn menu_transactions_blocked(&self) -> bool {
        matches!(
            self.phase,
            AdventurePhase::BonusResults { .. }
                | AdventurePhase::TimeTrialTimesUp
                | AdventurePhase::TimeTrialResults
                | AdventurePhase::ChallengeResults
                | AdventurePhase::ChallengeInterlude { .. }
                | AdventurePhase::VirtualPurchaseConfirm { .. }
                | AdventurePhase::VirtualFishSaleConfirm { .. }
                | AdventurePhase::VirtualBackdropSaleConfirm
        )
    }

    /// Switch a draft session. The caller persists and publishes the validated
    /// draft atomically; this method never advances any parked Board clock.
    pub fn select_mode(
        &mut self,
        destination: GameMode,
        now_seconds: u64,
        day_start_seconds: u64,
    ) -> Result<Vec<Event>, String> {
        self.validate()?;
        if destination == self.mode && self.phase != AdventurePhase::GameSelector {
            return Ok(Vec::new());
        }
        if self.menu_transactions_blocked() {
            return Err("finish pending transaction or reward before changing modes".into());
        }
        // Admission is checked before even the disposable Sandbox scene moves.
        match destination {
            GameMode::TimeTrial if self.progress.tank < 2 && !self.progress.adventure_completed => {
                return Err("Time Trial is locked".into());
            }
            GameMode::Challenge if !self.progress.adventure_completed => {
                return Err("Challenge is locked".into());
            }
            GameMode::VirtualTank if !valid_virtual_day(now_seconds, day_start_seconds) => {
                return Err("invalid Virtual Tank day".into());
            }
            _ => {}
        }
        if self.mode == GameMode::Sandbox {
            let mut draft = self.clone();
            let mut events = Vec::new();
            draft.leave_sandbox(&mut events);
            if destination != draft.mode || draft.phase == AdventurePhase::GameSelector {
                events.extend(draft.select_mode(destination, now_seconds, day_start_seconds)?);
            }
            draft.validate()?;
            *self = draft;
            return Ok(events);
        }
        if destination == GameMode::Sandbox {
            let mut draft = self.clone();
            let mut events = Vec::new();
            if !draft.enter_sandbox(&mut events) {
                return Err("Sandbox is locked or cannot start".into());
            }
            draft.validate()?;
            *self = draft;
            return Ok(events);
        }
        if self.mode == GameMode::VirtualTank
            && !matches!(
                self.phase,
                AdventurePhase::VirtualTank
                    | AdventurePhase::VirtualStore
                    | AdventurePhase::VirtualSetup
            )
        {
            return Err("finish Virtual Tank dialog before changing modes".into());
        }
        let mut draft = self.clone();
        let source_mode = draft.mode;
        if draft.phase != AdventurePhase::GameSelector
            && let Some(slot) = draft.slot_mut(source_mode)
            && slot.is_some()
        {
            return Err("active mode already has a retained scene".into());
        }
        if source_mode != GameMode::VirtualTank && draft.phase != AdventurePhase::GameSelector {
            let scene = RetainedScene {
                phase: draft.phase.clone(),
                board: draft.board.take(),
                next_seed: draft.next_seed,
                time_trial: draft.time_trial.take(),
                challenge: draft.challenge.take(),
                selected_pets: draft.progress.selected_pets.clone(),
            };
            *draft.slot_mut(source_mode).expect("durable source mode") = Some(scene);
        }
        let resumed = draft.slot_mut(destination).and_then(Option::take);
        let events = if let Some(scene) = resumed {
            draft.mode = destination;
            draft.phase = scene.phase;
            draft.board = scene.board;
            draft.next_seed = scene.next_seed;
            draft.time_trial = scene.time_trial;
            draft.challenge = scene.challenge;
            draft.progress.selected_pets = scene.selected_pets;
            Vec::new()
        } else {
            draft.mode = GameMode::Adventure;
            draft.phase = AdventurePhase::GameSelector;
            draft.board = None;
            draft.time_trial = None;
            draft.challenge = None;
            let action = match destination {
                GameMode::Adventure => Action::PlayAdventure,
                GameMode::TimeTrial => Action::PlayTimeTrial,
                GameMode::Challenge => Action::PlayChallenge,
                GameMode::VirtualTank => Action::PlayVirtualTank {
                    now_seconds,
                    day_start_seconds,
                },
                GameMode::Sandbox => unreachable!(),
            };
            let events = draft.apply_actions(&[action]);
            if draft.mode != destination
                || events
                    .iter()
                    .any(|event| matches!(event, Event::Rejected { .. }))
            {
                return Err("destination mode could not start".into());
            }
            events
        };
        draft.validate()?;
        *self = draft;
        Ok(events)
    }

    fn slot_mut(&mut self, mode: GameMode) -> Option<&mut Option<RetainedScene>> {
        match mode {
            GameMode::Adventure => Some(&mut self.retained_adventure),
            GameMode::TimeTrial => Some(&mut self.retained_time_trial),
            GameMode::Challenge => Some(&mut self.retained_challenge),
            GameMode::VirtualTank | GameMode::Sandbox => None,
        }
    }

    pub(super) fn validate_retained_scenes(&self) -> Result<(), String> {
        for (mode, scene) in [
            (GameMode::Adventure, &self.retained_adventure),
            (GameMode::TimeTrial, &self.retained_time_trial),
            (GameMode::Challenge, &self.retained_challenge),
        ] {
            let Some(scene) = scene else { continue };
            if self.mode == mode && self.phase != AdventurePhase::GameSelector {
                return Err("active mode also has a retained scene".into());
            }
            if matches!(
                scene.phase,
                AdventurePhase::BonusResults { .. }
                    | AdventurePhase::TimeTrialTimesUp
                    | AdventurePhase::TimeTrialResults
                    | AdventurePhase::ChallengeResults
                    | AdventurePhase::ChallengeInterlude { .. }
            ) {
                return Err("retained scene contains pending reward or transaction".into());
            }
            let mut projected = self.clone();
            projected.retained_adventure = None;
            projected.retained_time_trial = None;
            projected.retained_challenge = None;
            projected.sandbox_board = None;
            projected.sandbox_origin = None;
            projected.mode = mode;
            projected.phase = scene.phase.clone();
            projected.board = scene.board.clone();
            projected.next_seed = scene.next_seed;
            projected.time_trial = scene.time_trial.clone();
            projected.challenge = scene.challenge.clone();
            projected.progress.selected_pets = scene.selected_pets.clone();
            projected.validate()?;
        }
        Ok(())
    }
}
