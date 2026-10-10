# Retained mode scenes

E115 implements the [retained-scene decision](DESIGN.md#retained-mode-scenes-and-startup-menu).
Normal Title Start and the ongoing tank's Menu show a transient selector over
the saved Session. Opening it does not replace the current phase, Board, run,
wallet or clocks. Selecting the same ongoing mode resumes it. Profile changes
open this selector over that profile's own scene and discard transient owners.

## Ownership and transactions

`src/adventure/retained_modes.rs` parks Adventure, Time Trial and Challenge
payloads inside the current Session: phase, Board, next seed, matching run and
working pet selection. One profile continues to own progress, unlocks, shells,
scores and the monotonically advancing global Session tick. Parked actors, RNG
and Board/run clocks do not evolve. Captured creation rosters remain valid after
additional entitlements; resuming does not repopulate an existing Board.

The selector has no active scene, even when its mode label is Adventure. After
a terminal Time Trial/Challenge exit, selecting Adventure first takes its parked
payload. Virtual Tank keeps its existing independent ledger/Board/seed/time;
Sandbox keeps its existing disposable Board/origin behavior. Neither acquires
a second shared progress ledger.

`AdventureSession::select_mode` rechecks admission before moving a draft and
validates active and parked scenes against current profile authority. The app
uses `Profiles::transact` under its existing writer lease to validate and write
the complete manager atomically before publishing the selected route. Failure
keeps the prior manager and displays the error. UI entry/closing frames consume
input and clear gameplay backlog; no extra simulation tick is invented.

Receipt-bound Bonus results, Time Trial settlement/results, Challenge results/
interlude and pending Virtual Tank transactions block cross-mode moves and
gifts. Same-mode resume and nonmutating panels remain available. Ongoing Bonus
is resumable: its final receipt captures the then-current wallet only at
completion. These integrity gates are project policy; exact retail cancellation
and result retention remain unverified.

## Sources and adaptations

**Secondary-source-derived:** W1 revision
`f919b3c241cfd611c547f1653fb3514f500f761b`,
`source/WinFish/GameSelector.cpp:378–440` and
`source/WinFish/WinFishApp.cpp:1869–1903,1945–1956,2111–2123`
support separate mode saves, Adventure/VT resume and reload-before-chooser.
The project adapts this into typed payload moves and an atomic shared manager,
rather than separate retail files. `GameSelector.cpp:86` selects music2/offset0;
the app's menu cue uses its existing owned `Insaniq2.mo3` binding. No new primary
recovery or native observation supports these additions; [provenance](provenance.md)
retains source/license boundaries.

Current37 requires nullable `retained_adventure`, `retained_time_trial` and
`retained_challenge` keys, with required actor/run field guards applied to parked
Boards as well as the active one. No defaults or legacy conversion are added.
Profile3/bank2 remain unchanged. Historical saves stay with their identified
frozen builds, including E80's genuine earned Nimbus save.

## Verification and limits

A61ModeResume04 passed formatting, strict workspace/all-target lint, one focused
integrated conservation case and locked build/staging. The case switches an
actual Adventure Board to Time Trial and back, preserves both Board snapshots,
seed/selection and clocks, reopens through the production current37 decoder,
then exercises terminal Time Trial Continue/Menu and resumes the original
Adventure Board. The terminal settlement is fixture-driven, not earned gameplay.
There is no presentation matrix;616 unrelated unit cases were filtered out.

Earlier compiler attempts01/02 failed before tests/build;03 passed the narrower
serialization case before the production-decoder and terminal-exit corrections.
[MODLOG](../MODLOG.md#2026-10-10--retain-mode-scenes-and-open-startup-menu)
owns failures and check receipts; [STATUS](../STATUS.md#current-build-verification)
owns current source/executable identity. Native input/render/focus, actual file
reopen under the app, Challenge/VT/Sandbox cross-mode paths, retail/audio and
human judgment remain not tested. This model/decoder check establishes neither
playability nor full runtime acceptance.
