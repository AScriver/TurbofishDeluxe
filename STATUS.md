# Current handoff

Updated: 2026-10-12, America/Phoenix. **#311-318 — all eight Rust review
corrections committed on main and Done.** The [owning record](docs/rust-review-fixes.md#tracking-completion-after-claim-repair)
owns ordered commits, actual checks, failed approaches and terminal readback.
Actionables omitted its own bulk-preparation activity from its shared schema;
repair `e365424` is checked, deployed and merged into its existing AScriver branch.
The managed restart preserves every Actionables database row and configuration.
Each task now reads back Done version12 with Passed validation and no claim.
The joined tree preserves main `5fd37f2` and the completed E131/#319 guppy
correction. All912 tests pass (11 ignored), plus formatting, strict Clippy,
compilation, isolated build and workflow. Root owns the guarded integration;
both histories remain intact. Original claim failures remain historical.
No gameplay, live saves, owned installation, audio device or normal-launch
artifact is used. Earlier human/retail/audio acceptance limits below remain.

Updated: 2026-10-12, America/Phoenix. **E130 development-save compatibility removed; E128 targeted guppy observation complete.** [Follow-up evidence](docs/playtests/2026-10-12-guppy-turn-motion-followup.md) verifies release401da21 and four full-update probes with zero added turn-start delay. A human-authorized isolated1x run reproduces movement before pronounced body rotation;85 sampled turn states match the recovered frame rule. The original also starts animation before integrating reversed motion, and early authored poses remain close to the starting profile. No extra start delay or further runtime defect was established; retail playback timing and human acceptance remain open. All four owned test processes are closed. E128's mirror correction and Actionable#310 Done remain historical. E129's unrelated source edits are preserved; turn probes/captures identify the retained E128 build. E127/#309, E126/#308 and E125/#303-307 remain complete and preserved. E124 engineering is complete with its separately recorded Ready/unclaimed tracking limitation. Earlier source transfers/workers remain closed. Ordinary gameplay remains human-owned; autonomous scenarios and heartbeat remain stopped. Screensaver work stays excluded. Original/project live saves, E80, genuine earning06 and normal-launch artifacts remain untouched; full retail/audio/human acceptance stays pending where unobserved.
Updated: 2026-10-12, America/Phoenix. **E131 Adventure-guppy wall-turn correction checked.** [Boundary evidence](docs/guppy-turn-boundaries.md) reproduces an extra ordinary-fish post-animation bounce that the original common tail does not perform. Ordinary and warded guppies now share the binary-derived clamp/ease/animate/move order; the fixed turn clock is preserved. Three new boundary regressions become green; affected lint, persistence and an isolated build pass. The broader library run retains one independently reproduced baseline Time Trial fixture failure. E128 food/speed/normal-speed observations remain historical and did not cover this wall defect. Human wall-turn acceptance and retail playback remain open. E130/E129 save cleanup and all earlier closed transfers remain preserved. Ordinary gameplay stays human-owned; autonomous scenarios, heartbeat and screensaver work remain stopped. Live saves, E80, genuine earning06, retained builds and normal-launch artifacts stay untouched.

## Current increment

**E131 / Actionable#319 — ordinary guppy wall-turn correction:** Fresh-context
independent static review confirms that PB05 clamps coordinates and eases
outward VX before animation, then integrates without a common-tail bounce.
The ordinary Rust path reversed velocity after animation; its baseline wall
test published the wrong mirror while still in Swim. One shared motion tail
corrects that order and preserves the existing warded behavior and turn clock.
Six binary guppy checks, three warded matches, the existing pulse check and one
affected persistence case pass; formatting, strict affected lint and an isolated
debug build pass. Expanded library verification has688 passing checks, six
ignored and one failing Time Trial admission fixture; the exact failure also
occurs in a disposable clean02a2951 baseline before fish update. [Owning record](docs/guppy-turn-boundaries.md#correction-and-verification)
preserves failures, primary addresses, source/build identity and limits. No
native run occurred for this changed build. All122 source inputs/139 identity
rows and workflow/whitespace pass. #319 is Done version9 with qualifying Passed
validation and released claim, read back. Human wall-turn timing and retail
playback acceptance remain pending.

**E130 — remaining development-save compatibility removed:**48 persistent
Serde defaults, five default-only helpers, four retired migration tags and
automatic standalone-save adoption are removed. Normal startup uses current
profile3 only; current standalone tooling retains session37. Existing files
stay untouched. All118 unique persistence checks plus one affected library
check have passing evidence; formatting, strict lint and an isolated build
pass. [Owning record](docs/refactor-save-codec.md#remaining-development-save-compatibility)
owns baseline failures, fixture corrections, exact identity and limits. No work
item was supplied; Actionables was not updated. E129 and the concurrent E128
guppy observation remain historical and preserved; no native game ran here.

**E129 — requested dead legacy-save cleanup, complete:** Nine reconstruction
functions and the production legacy wrapper/error type are removed. Twelve
persistence tests now use current-format scenarios; retained origin tags have
explicit standalone/profile round-trip coverage. All21 focused checks,
formatting, strict lint and an isolated build pass. Session37/profile3,
completed screensaver code, live saves and normal-launch artifacts stay
preserved. [Owning cleanup record](docs/refactor-save-codec.md#dead-legacy-save-cleanup)
records exact checks, initial failures, source/build identity and limits.
No work item was supplied; Actionables was not updated. The concurrent guppy
motion/pose follow-up below remains separately recorded.

**E128 motion/pose follow-up:** The human reports turns look unchanged after the
normal rebuild and says the animation appears to start after direction changes.
The latest release receipt matches E128's source identity and current artifact.
Primary Update instructions place animation before position integration. Four
isolated full-update food-turn probes on the retained E128 library find zero
extra update delay in both directions, including a zero-speed crossing. Owned
turn-sheet inspection explains the initial silhouette hold. The later authorized
isolated1x run captures gradual turn poses while movement has already reversed;
85 sampled turn states match the binary-derived rule. [Report](docs/playtests/2026-10-12-guppy-turn-motion-followup.md)
records source-guard/focus failures, build identity, paired pixels/state and
unforced cleanup of all four attempts. The visible symptom is observed locally;
retail wall-clock/visual comparison and human acceptance remain pending. No
additional runtime change or Actionables mutation occurred.

**E128 movement-speed follow-up:** The human suggested a missing speed factor.
Primary instructions distinguish a fixed turn clock from speed-dependent swim
cadence and ordinary velocity/divisor movement. All24 isolated retained-build
cases/480 full updates have the same turn sequence; faster fish travel farther
before the head-on pose. [Speed report](docs/playtests/2026-10-12-guppy-turn-speed-followup.md)
owns exact addresses, identities, fixture limits and receipts. No further
runtime change or new defect was established; retail movement/visual comparison
and human acceptance remain pending.

**E128 / Actionable#310 — Adventure guppy turnaround:** The human narrowed the
reported timing issue to Adventure guppies. Fresh read-only PB05 instructions
confirm the existing signed20-count/frame cadence but show that the authored
turn sheet retains its starting mirror through every nonzero timer state.
Rust previously used the new velocity direction immediately. Both guppy draw
layers now use a derived sprite projection, preserving simulation and saves.
Three expected red failures become3/3 green; the existing turn/eat/growth test
passes. Formatting, strict affected lib/bin/test lint and an isolated debug build
pass. [Owning report](docs/fish-turn-animation.md#correction-and-verification)
records exact binary/source/build identity, checks and limits. Workflow/whitespace
and all122 source/139 identity rows pass. #310 is Done version10 with qualifying
Passed validation and released claim, read back. No native run occurred.

**Renewed necessary-fix audit, complete:** Three bounded independent source
reviews found no new verified normal-play defect in Adventure actor lifecycle,
audio ownership or Sandbox entry-character handling. The selector drains the
character queue before checking its entry keys, falsifying the proposed same-frame
Sandbox leak. [Audit evidence and limits](docs/necessary-fixes.md#renewed-necessary-fix-audit)
remain source-only; no tests/builds/native scenarios or saves were run. Source
bytes remain E127's accepted identity. No new task or implementation was dispatched.

**E127 / Actionable#309 — necessary tank chooser dispatch fix, complete:**
Source `e90df05` applies Time Trial/Challenge Select/Menu/Escape before the
parent UI pause gate, preserves higher input owners and consumes the frame
after its first transition. Eight focused regressions, formatting, strict
affected lint and a locked/offline debug build passed. Root matched all122
source inputs,148 worker artifact rows and17 preserved E126 artifact rows,
reviewed the production join and found no additional necessary fix in the
bounded follow-up. The [completion record](docs/necessary-fixes.md#tank-chooser-completion-and-follow-up-audit)
owns exact identity and limits. Done version12, qualifying Passed validation and
released claim were read back; source ownership is closed and no further work
is assigned. No native run or save access occurred.

**E126 — necessary checkpoint error fix, complete:** New worker
source `f75ad30` preserves contextual I/O identity; root joins both optional
observation consumers to the helper. Four actual observer regressions and ten
matched writer contracts passed, including lock/defer/recovery and fatal invalid
destinations. Formatting, strict affected lint, locked/offline debug build and
all122 input/139 source-artifact readbacks passed. The [joined checkpoint](docs/necessary-fixes.md#joined-checkpoint-for-actionable-308)
owns independent red results, exact identity, sources and boundaries. Actual
root joins are pushed at `89dd0e2`; #308 is terminal Done version12 with
qualifying Passed validation and released claim, read back after final
acceptance. Its source transfer is closed. No native game/live-save run occurred.

**E125 — modal frontend completion, complete:** Actionables#303-307 are Done
with qualifying Passed validation and released claims, read back at versions
13/13/14/15/14. Five owned contributions and the Challenge-loss extension are
joined through actual normal input/render/resources, settings/save feedback,
profile transactions, Quit checkpoint, gameplay admission and pause/help/menu
callers at `008eadb`. Final source `ea9ed91` removes the temporary gift collector
and bounds its balance header; `b7c3808` corrects unit-test module ordering.
Root's72 focused checks, formatting, strict production/affected-target lint and
locked/offline release build passed. All122 maintained build inputs and139
source/artifact rows match the final receipt; every owned source commit is an
ancestor. The [completion record](docs/modal-batch-303-307.md#completion-and-joined-verification)
owns reports, identity, actual failures and limits. Shared installed resource
declarations and the W1 pin were read back. Human gameplay, current37/profile3,
saver exclusion and saved lineages remain unchanged; no native/retail/pixel/
audio/playability result follows from these checks.

**E124 — bounded code-quality batch, engineering complete:** Five new
chats/worktrees pushed shared glyph layout, direct current-format save admission,
checkpoint persistence, reusable UI primitives and concrete input/modal/audio
runtime state. Root joined normal/Help text, application button consumers and
CLI/profile storage at `17286a4`. The [completion record](docs/refactor-batch-298-302.md#completion-and-joined-verification)
links each owned commit and report. Final formatting, strict lib/bin/test lint,
34 focused font/UI/writer/checkpoint tests and locked/offline release build
passed. One serial timing benchmark remained ignored. All92 maintained
Rust/Cargo inputs match the checked build; every owned core commit is an
ancestor of pushed main. Worker checks cover affected codec and state
contracts; the two malformed-current-save diagnostics formerly panicking now
pass. Current37/profile3 and publication semantics are preserved. Actionables
#298-302 remain Ready version6/unclaimed under the recorded unavailable-tool
fallback; no tracking completion is claimed. Human gameplay policy is unchanged.

**E123 — optimized launch/loading/active VT batch, complete:** Actionables#295-297 are Done with qualifying Passed evidence and released claims. Three new managed chats/worktrees implemented optimized normal launch, bounded Title loading batches and lightweight active VT refresh/input; owned commits `f3a6738`, `cbf03b8`, `9817545` safely reconcile on main at `d64e0cc`. [Performance](docs/performance.md#launch-loading-and-active-virtual-tank-results) links eleven launcher, twenty loading/clock and seventeen active UI checks, exact state/event/pixel/trace equivalence and serial offline measurements. Normal/20-fish fixture ticks measure about16x faster in release; fast synthetic185/140-task loading uses21/16 yields;20-fish Fish Setup adapter26.312us becomes6.228us. Joined format, strict lib/bin/test-target lint and actual default release staging passed. Root matched all102 build sources,117 source/artifact checks, nine selected report hashes and all three owned ancestors; the [contract](docs/performance-batch-295-297.md#completion-and-joined-verification) owns exact receipt/EXE identities. These results establish no native FPS/startup/retail/human-feel claim. Human gameplay and paused automation policy remain current.

E122 / Actionables#292-294 and E121/#289 are complete; E120's human ordinary-gameplay handoff remains current. Older increment entries retain historical next-run plans; the current `Next action` section supersedes them.

**E122 — bounded parallel performance batch (#292-294), complete:** All three Actionables are Done with qualifying Passed evidence and released claims. Owned commits `2ec1b10` (#292), `2b00568` (#293) and `5fc4ee5` (#294) safely reconciled on main, with final source/report integration at `505947a`. [Performance](docs/performance.md#frame-profile-and-checkpoint-results) links the exact owning reports: ordinary score-free frames avoid whole-profile clones; inactive Adventure/TT refresh pairs avoid both tank captures; ordinary checkpoints borrow state, Sandbox projects one copy, and equivalent result saves publish once. Nineteen focused regressions and all serial measurements passed; four affected score/save checks passed after joining source. Integrated format, strict lib/bin/test lint and isolated build passed. Root verified seven source/fixture/dependency hashes and ancestry instead of repeating Cargo gates. Slower duplicate-score and whole-Sandbox controls remain documented; no native/FPS/fidelity claim follows. Human gameplay and saves were untouched.

**E121 — measured object-search optimization (#289):** Stinky target selection now uses a single pass, removing its temporary coin/note vector and ID sort while preserving current distance, tie, eligibility and contact rules. [Performance](docs/performance.md) owns identified before/after fixtures, timings, hashes and limits. The crowded query median improved 2.72x; crowded whole-tick median fell 20.3%. Complete state and ordered-event digests match after one and 32 ticks. Whole-tick baselines do not justify a spatial index; food/prey searches remain unchanged. Fifteen affected simulation regressions, formatting, strict production lint, both measurement cases and an isolated build passed. An unrelated unchanged save-decoder diagnostic test failed in the broader filter; its exact limitation is retained below and in the report. No native run, normal-launch build refresh, save/profile change or parallel agent followed this work.

**E120 — save publication fix; human playtesting handoff:** Windows host evidence exposed transient checkpoint replacement denial during live observation. Safe Rust now retries only eligible rename errors within a bounded100ms budget, preserving the old complete save on persistent failure; the mode driver's live hash reader explicitly shares deletion and disposes promptly. [DESIGN](docs/DESIGN.md#windows-checkpoint-publication) owns the policy. Two focused Windows persistence regressions, formatting, strict production/persistence lint and locked build/staging passed; no broad suite. Current37/profile3 and model behavior are unchanged.

[A66](docs/playtests/2026-10-10-mode-continuation-a66-01.md) preserves all three attempts.02 observed foreground, Title→Menu and full same-mode save conservation, then access-denied exit1; final telemetry did contain Scene.03 used corrected E120 bytes, observed foreground and Title→Menu/startup readback, but timed out awaiting Adventure publication, then exited0 without force. The native fix and subsequent route are not accepted; mode/pause/reopen remains incomplete. Both positive activation receipts worked, with zero automatic requests. No agent-owned process/lease remains; do not interfere with any new human-owned session. No further native retry follows the user's human-testing direction.

The reported normal launcher error came from a default project profileformat1, while current support is3. Under the user's clearing authorization, only that stale profile was archived byte-exactly in private scratch; retail saves/install, settings, E80 and genuine test lineage were untouched. [A66's startup handoff](docs/playtests/2026-10-10-mode-continuation-a66-01.md#default-profile-and-human-launch-handoff) owns its receipt and command. The next normal startup should create a fresh current manager; human startup/play is not yet confirmed. No legacy migration was added.

**E119 — VT energy-ball presentation joined:** a bounded read-only Investigator found an existing combat actor using the classic missile image in VT. Both renderers now use the existing two-layer energy-ball path, including reflected tint; [VT contract](docs/virtual-tank.md#shared-combat-projectile-presentation) owns the decision and limits. Formatting, strict production lib/bin lint and locked build/staging passed with **zero tests**. First scoped lint failed because its native-library environment was missing; setup was corrected and only failed lint/build ran afterward. Core/model/current37 and prepared fixture admission are unchanged. A66 subsequently froze the existing checked normal-game artifacts without compilation or screensaver packaging; only Title was observed. Current VT visuals remain unperformed. Continue remaining system integration independently of the foreground blocker.

**E118 — current normal-game startup fix; native focus blocked:** production current37/profile3 mode fixture is ready, explicitly prepared-not-earned. Final mode driver`d7e9c2c` was published in`5ae0d74` and returned to root. [A64 report](docs/playtests/2026-10-10-mode-continuation-a64-01.md) preserves four bounded1x attempts and distinct failures. Installed-manifest `Pix118` correction passed format/strict lint/locked build with **zero tests** and reached actual Title/Menu. Driver live-log digest was deferred until cleanup. Attempt03 passed actual startup/reopen comparison but Menu resume did not publish; source review retained focus/capture/transaction distinctions. Attempt04 directly observed the owned window not foreground and Windows denying the explicit activation request; scenario input stopped. Root fixed the first focus integration's accidental `WM_CLOSE` gate, restoring background cleanup. No game process remains. Native acceptance is incomplete; further runs require actual foreground access or the user's deferral choice. No new Cargo gate for PowerShell-only corrections. Source-qualified selector music remains conditional; no unconditional first-entry order45 claim or audio code change follows it.

Four earlier outcome chats/private scenarios are preserved and inactive: Adventure/progression, ordinary VT/store, TT/Challenge/Sandbox, and interface/audio. They establish preparation/admission only. Root alone owns runtime/helper/central engineering and Git; no delegation or native lease is active. The interrupted read-only A66 Investigator made no edits or resource changes. Human ordinary gameplay replaces the planned autonomous lane dispatch.

Song catalog correction`4c87d55`: [installed syntax compatibility](docs/shared-interface.md#installed-song-syntax-compatibility) fixes the reported Entertainer duration error and four later installed syntax forms. All72 scores/47,577 notes load; five focused cases, formatting, strict lint and isolated build passed on `be850c1` plus the identified song parser/test changes. Installed score digests stayed unchanged. [MODLOG](MODLOG.md#2026-10-10--fix-installed-fish-song-catalog-loading) retains reproduction, subsequent failures and receipts. This does not change the broader completion sweep or establish audible/native fidelity.

E117`9f20cb2`, reconciled/published`007792d`, implements actual [Title/Menu UI evidence](docs/playtest-helpers.md#ui-stage-observation) and preserves the completed source-qualified saver cap. Its one integrated cap/accounting regression plus format/strict lint/locked build passed;616 unrelated cases stayed filtered. [MODLOG](MODLOG.md#2026-10-10--cap-saver-production-and-observe-actual-ui-stages) owns the failed fixture/correction. E116`2adf7fe`, E115`5e60d96` and earlier VT/gift/menu/chooser/Story/Hall remain intact. Current37/profile3/bank2 and protected frozen/genuine lineages are unchanged. [Requirements](docs/requirements.md#goal-and-constraints) owns the screensaver scope change. E117's native startup failure is historical; use E118's corrected frozen game for further normal-game work. Broad implementation presence and prepared fixture admission are not playability claims. Foreground access blocks the next native run; records and private scenario preparation remain unblocked.

The latest implementation-first direction permits pinned WinFish for bulk remaining behavior. Keep focused correctness checks and source qualifications; defer polish, exhaustive validation and nonblocking fidelity analysis. Relax's activation remains unknown, and its supported internal configuration does not establish a usable retail entry route.

Current31 [fourth slot](docs/extra-pets.md#current31-implementation-checkpoint) is committed in`e68aa4796f1e6ec06df63b09629b3e74c5899484` and privately frozen as E99. Cursor4/40,000-shell capacity has ordinary/replay/TT fourth-choice and selected-plus-fixed Tank5 consumers; cursor5 waits for Virtual Tank. [C1–C4](docs/extra-pets.md#fourth-slot-integration-contract) owns its contract. One complete636-check gate passed after a static save-label guard correction, with no extra probe matrix or native run. [MODLOG](MODLOG.md#2026-10-10--verify-fourth-slot-and-preserve-current31) owns exact receipts. HEAD now also includes the separate README license notice commit`7e94d27492104b2f878546f8eefc62eae6ab95fd`; preserve it. Current30/Walter remains committed in`f528c98d136f2f2092f2d057e77b3bdfa4b53663` and frozen as E98.

Current29 [Stanley/cursor2](docs/extra-pets.md#current29-implementation-checkpoint) is committed in`ea5a67e2fbe382abafdb507be9ab6ee2a0b3a7b1` and remains frozen as E97. Current28/E96 [Nostradamus](docs/extra-pets.md#current28-implementation-checkpoint) remains committed in`f39c27170a9e147b9e0b7b5ad8eed91e5016addd`; current27/E95 in`4aad58e3d210e17aa3c3d39692a6a0a9a0e5f5ed` and current26/E94 [replay](docs/adventure-replay.md) in`8cfeca34accdb494f4cf4912c00a2677e4132f31` remain preserved. Their verified scopes are historical below; current30 adds no new migration/backfill.

[A36: Tank5-1/Cyrax](docs/adventure-5-1.md) is committed in`286cdbf0f0bb568fae336f4d1fb6a85d5a359403`; it passed the516-check complete code gate with current21 saves, pet-only combat/survival, separate boss/mini control, loss/retry accounting and first completion/Hatch/interlude. Its identified executable/DLLs/notices are frozen. Independent reviews closed Tank5 EnergyBall admission, reflected boss/Bilaterus/pet coverage and saved-boss HP bounds; primary registration instructions corrected the unsupported reflected-mini assumption. E89 owns verification scope. No native/manual acceptance or A36 playable claim is made.

[A37](docs/presto.md)'s ready actor/current22 correction is committed in`1fbb6b48efced7e6d000292714cf7072ee27c1be`; ordered typed OtherPet storage/current23 in`87e884e6470171a4e3ee0334be42f44b8b1e43ea` is frozen (E91). Current24 adds flagged Clyde/Niko/Rufus/Rhubarb constructors, position/clamp/clock regressions and required Niko pose. Current25 integrates logical TT Presto, all20 form mappings, modal continuation and surviving-child/retired-owner guards. E94 extends these through completed replay and selected-plus-fixed Tank5; nested allocator byte-copy remains qualified. Ordinary Adventure results use the bonus tally; the5%-cash branch belongs to Time Trial.

[A38 Time Trial](docs/time-trial.md) is integrated/current24 with four-tank entry, ordinary pet eggs, mode-specific waves, strict deadline/settlement/results, separate loss and project scores. Its547-check bundle is frozen (E92); current25 adds initial logical Presto with separate egg eligibility. Thirteen additive roots plus instructions/reused primary bodies close omitted egg blocks, initial Strong and the seven survivor categories. Ordinary guppy removal precedes corpse creation; the retained Rust inactive record is project storage, not retail list membership. Extra pets/fourth-slot acquisition and retail tie/re-entry/RNG/audio/artwork remain gaps. Completed Adventure replay follows separately.

[A35: Tank4-5, Angie and final-Tank4 completion](docs/adventure-4-5.md) is committed in `246515aeb5f718401cd0aadb01dd17c72f30b5d5` from PB73–75. The complete477-check code gate passed and its executable/DLLs/notices are frozen privately; native behavior remains unverified. The committed [broad recovery map](docs/binary-recovery.md) preserves all12 remaining-family outlines/evidence/gaps. [A34](docs/adventure-4-4.md)'s453-check and [A33](docs/adventure-4-3.md)'s433-check checkpoints remain historical.

[A32: Tank4-2, live Nimbus, Bilaterus and Ultravore](docs/adventure-4-2.md) is implemented at source checkpoint`4b2d53fd6c6711ad615d083b11f9d41f745ad22f`/format17. E80 preserves its executable/source identity; the [partial native report](docs/playtests/2026-10-09-adventure-4-2-native-a32-01.md) owns observed entry/conversions/combat/Ultra/save-pause/two paid eggs and deferred outcomes.

[A31: Tank 4-1](docs/adventure-4-1.md) is the latest native-accepted stage, committed `38c65be9ad439a727b2e7e1b1401db928b8e55f6` (E76/E78). Its frozen bundle, reports and genuine earned saves remain preserved.

## Current build verification

**E127 — accepted chooser gate:** Pushed source `e90df05` matches all122 worker
build inputs and148 preserved checked artifact rows. Eight focused tests,
formatting, strict lib/bin/test lint and the locked/offline normal debug build
passed with Rust/Cargo1.95 MSVC. The [accepted identity](docs/necessary-fixes.md#tank-chooser-completion-and-follow-up-audit)
owns the frozen receipt/EXE hashes and root readback. No normal-launch artifact
replacement, native game run or live-save inspection occurred. E126's17 frozen
artifact rows below still match; its source identity is historical after E127.

**E126 — preserved historical observer/writer gate:** Worker source `f75ad30` plus root's
recorded dirty `app.rs` joins matches source digest
`1fc42c5db070331c525754f6d3407b38b8ed9b583a69b3c85692c7b76fbff37c`.
Four actual observer tests and ten worker writer cases passed; formatting,
strict lib/bin/checkpoint-target lint and `cargo build --locked --offline -j 2 --bin turbofish-deluxe`
passed with Rust/Cargo1.95 MSVC. Frozen private receipt
`.scratch/coordination/necessary-308-final-build.local.json` has ID
`4424c959d520b0907ad99352d29c9aade145c7b2be09b6084b21fe09fece6b7c`;
EXE SHA`b9d8a955b2af7bdf82f8ee7e5e7d8a056c1606c8350a575e477758a7eb5d8a53`.
All122 build inputs and139 source/artifact rows matched. The [joined checkpoint](docs/necessary-fixes.md#joined-checkpoint-for-actionable-308)
owns actual failed red runs and limits. E125's release bytes below are preserved
as historical acceptance. No normal-launch artifact, game/live-save or native
acceptance was changed or inspected.

**E125 — final joined modal gate:** Source `ea9ed918240de62d69bf13ee8b5787546b2c71d2`,
Rust/Cargo1.95 MSVC, `cargo build --locked --offline -j 2 --release --bin turbofish-deluxe`.
Private receipt `.scratch/coordination/modal-final-20261012-065700-release.local.json`
has ID `bd7150279145c784bd0ffa4e64b1980e56e64fac8d465e591ee2715b6683b41e`;
release EXE SHA`fd37c61048abf51ad3e455920866b773bff807a23f20abc0f4c12eae0a75d826`.
All122 build inputs were unchanged during compilation;139 source/artifact rows
and all owned ancestors passed readback. Seventy-two focused checks passed,
including real-writer failure/retry, gameplay admission and retained pause.
Final gift checks, formatting and strict production/five-target lint passed
after the last source change; worker strict unit-test lint passed after the
ordering correction. The [completion record](docs/modal-batch-303-307.md#completion-and-joined-verification)
owns exact commands/logs and preserved failures. Later central-record changes
affect no build input. No normal-launch artifact replacement, native run or
human/live-save inspection occurred.

**E124 — final joined refactor gate:** Source `17286a4ebab8f60efdbb2a1f7caa29039e1afb22`
matches the92 input hashes captured at `3297c62` plus the four recorded root
consumer joins. Formatting, strict lib/bin/test Clippy,34 selected tests and
`cargo build --locked --offline -j 2 --release` passed on Rust/Cargo1.95 MSVC.
Private release EXE SHA`90ABB50B042E878FC515C161E79EC8DFB8362028CE07D5588AC2B091DE8056E7`;
the [completion record](docs/refactor-batch-298-302.md#completion-and-joined-verification)
owns exact commands, artifact/receipt paths and scope. Final workflow checks
passed35 required files/101 documents. Later central-record changes affect no
build input. No native launch, launcher-artifact replacement or live-save check
ran; human gameplay remains the acceptance path.

**E122 — integrated offline performance gate:** [Checkpoint build identity](docs/performance-checkpoints.md#build-identity-and-focused-checks) identifies checked source `a45114d07e7adbd5e084d77beb81352f0a6073be` plus report-only changes, Rust1.95 MSVC, locked/offline dependencies, opt2/debug0/no-LTO dev profile and worker-local target. Four exact affected score/projection/denial regressions, formatting, strict lib/bin/test-target Clippy and normal-bin build passed. Integrated EXE SHA`43B62B673420BE61C6A4E12ED2D2B56C42D33F7C386A927EA3B0F82B19B0D3BD`; receipts remain in the checkpoint worktree's `.scratch/performance/294`. Root's private `batch-292-294/verify-integrated-source.ps1` confirmed seven current source/fixture/Cargo.lock hashes and all three owned ancestors. Later report/central-record edits affect no checked source. Normal launcher artifacts retain their previous handoff; no game/native or live-save check ran.

**E121 — offline search/build gate:** [Performance](docs/performance.md#measured-identities-and-equivalence) records parent `266cd75c84bfab22539706a7b3e5676a02014a39`, dirty source/fixture identities, toolchain, locked dependencies, release test binaries and the isolated debug game EXE. Private evidence is in `.scratch/performance/a289`: `baseline-03.log`, `optimized-01.log`, `comparison-01.json`, `verify-01.log` and `verify-remaining-01.log`. The fixture source stayed byte-identical, and the validated comparison passed all four input/result records. Production code changes only the Stinky query; current37/profile3 and historical frozen lineages are unchanged. This gate establishes offline optimization equivalence for the stated fixtures, not native playability or retail fidelity. E120 below retains the normal-launch artifact and native limits.

**E120 — focused Windows persistence/build gate:** `.scratch/runtime/a66-checkpoint-check-01.log` SHA`A7332B8631A63EC6255239BF1BF03E3E50B6A34231856EEC6E054D8ADE8D70CD` records `cargo fmt --all -- --check`, `cargo clippy --locked --package turbofish-deluxe --lib --bin turbofish-deluxe --test persistence -- -D warnings`, `scripts/Test-Runtime.ps1 -Check Persistence -TestFilter atomic_replace` (**2 passed,107 filtered**) and `scripts/Test-Runtime.ps1 -Check Build`. No unrelated tests or later passing-filter repeat. Parent `.scratch/evidence/20261011T201729537Z-8354303e-build.local.json` SHA`B538434B829ABBBE7C642C9930A3A963BB73166B4998E02DBF4B9821C727BB83` identifies HEAD`85cdb1ec2afbd9161c45ade0ed427ce49aaa3c5a` plus the recorded cli/test/driver/DESIGN changes, source-set SHA`9A78B4CCED4BB89419A4B4C174E41F9B6AD9A42FF89DEC4DCE2172EA1B27807B`. EXE SHA`9AF1484EE7A186880F9F9851EB1DE3B2A881579CAC68CA943225E8A9850C4D93` is staged in `target/debug` and frozen in `.scratch/runtime/a66-game-bundle-03`; parent-linked `.scratch/evidence/a66-frozen-build-03.local.json` SHA`1A9E46D16DFC5C1D6A4FB469A466055B17B8BCCA6A376B2855408E94A1AB01CC` matched14 copied/16 recorded artifacts. This is a normal-game build, not screensaver packaging. Subsequent edits affect records only; current helper/source bytes retain their identified checks. A66attempt03 remains incomplete native evidence.

**E119 — changed renderer compile/build; native pending:** formatting passed in A65Render01; its scoped lint stopped before build at missing`OPENMPT_LIB_DIR`. A65Render02 reused the maintained native preflight/environment and passed `cargo clippy --locked --package turbofish-deluxe --lib --bin turbofish-deluxe -- -D warnings` plus `scripts/Test-Runtime.ps1 -Check Build`; no unit/workspace test or format rerun. Log SHA`F750C3AAF7E70886EA4D97102D5BCB2828A3AEA4A50C99E075A0B21F37CF2DEF`. Launcher`.scratch/evidence/20261011T005522210Z-635e5f79-build.local.json` SHA`A6CD4A5C2BEDC4463FDA0C94F5C06DD851D9475BC6A8B83B12EEB9B6232FFD63` identifies`2b8f5b2` plus app renderer/STATUS/VT contract edits; source-set SHA`3A60E6D4DE5AC116DD567366F6B963442E25D73A814D4C8FC7E6FCC4F26D6FF9`. EXE SHA`6FF6C11F55DCFEA8851B99F487FA43E23742A0246DA88A5EA21E5D63AE075425` is now frozen in`.scratch/runtime/a66-game-bundle-01`; [A66](docs/playtests/2026-10-10-mode-continuation-a66-01.md) owns the parent-linked relocation identity and actual Title-only run. E118's frozen bundle/native failures remain historical and source-unaffected outside the changed energy-ball renderer. A new VT acceptance lease needs these E119-or-later frozen bytes and updated metadata binding; preserve original producer receipts/profile bytes, without repeating unchanged decoder checks. No retail/install/protected-save mutation.

**E118 — observed startup correction/compile; native incomplete:** A64Startup01 passed format, strict workspace/all-target lint and locked build/staging with zero tests. Launcher`.scratch/evidence/20261011T001827180Z-7addc33b-build.local.json` SHA`0E269D46A3C2C3517FE517F7C2CAE9246F3C664093D2D1279B03F1C81FFCE633` identifies`5ae0d74` plus the selector font fix; source-set SHA`F4B19EF78F80F8F1CD44FD9FCED1FA6A2F0AF4DEE95EA060F3B564609C334AF5`. Game EXE SHA`03051CEC81293C15F5096D8E702FC0F9F7565721F4B6E8547A3041F7490551FF` and14 executable/DLL/notice artifacts are frozen in`.scratch/runtime/a64-game-bundle-01`; parent-linked receipt`.scratch/evidence/a64-startup-frozen-build.local.json` SHA`7C877B531DE062B633264C426C8482BD55AF902BF9B4DC3AF39C8EB84ECC9878` binds exact frozen paths,16 artifact matches. This is normal-game freezing, with no screensaver packaging. [A64 report](docs/playtests/2026-10-10-mode-continuation-a64-01.md) owns actual startup/visual/typed-reopen evidence, incomplete mode/save/pause/reopen, denied focus and honest forced attempt04 cleanup. Only PowerShell tooling changed after compile; game bytes/core/save format stayed unchanged. Original installation and genuine E80 lineage remain isolated. Subsequent focus/cleanup and full normal-game acceptance are not tested.

**E117 — observed cap model and UI compile/build; game native deferred:** A63SaverUI02 passed format, strict workspace/all-target lint, one integrated accounting case and locked build/staging;616 unrelated cases filtered. Launcher`.scratch/evidence/20261010T234351777Z-1ad3a58e-build.local.json` SHA`315C5592C50E75E3FE7692F980AE6B4546DC00581D3D9E861543C80198B2C1DC` identifies`2adf7fe` plus app/Title/adventure/saver/VT Board changes, now committed`9f20cb2`; source-set SHA`DFE1941513A4A98B8EC5CAD2B08F9182DF113E5A3A06A6BF04807228EAA5A742`. EXE SHA`B71942C7E48B3B647A50B2CB9D637BF4CD829B28BDBDADED81309A25063017F8` is frozen with its dependencies/notices in `target/screensaver/2a4ff29d667a71c2e39f00c6433e76987afb593a642eec6abb2d64099bd0a353/turbofish-deluxe.exe` (packaged before the scope change; preserve/reuse this game EXE). [MODLOG](MODLOG.md#2026-10-10--remove-further-screensaver-work-from-completion-scope) owns freeze/publication receipts and scope change; no further saver work is scheduled. UI instrumentation has compile evidence only. No native/game/DB/OS/protected-save/install mutation or legacy conversion. Actual normal Title/Menu/input/readback, native catalog/audio/retail/human acceptance remains not tested. Fresh prepared game fixture and finalized mode driver are next; E80's genuine lineage stays with its historical build.

**E116 — observed catalog byte mapping and app compile/build; native deferred:** A62CatalogConfig01 passed format, strict workspace/all-target lint and locked build/staging, with zero tests. Launcher`.scratch/evidence/20261010T232758152Z-b779aa92-build.local.json` SHA`38CA714D9437DB55B299319A2E9A371F53EF977C9605003EBDFAA3154AB70A65` identifies`53cb042` plus app/lib/VT renderer/new catalog changes; source-set SHA`1AA9D99025337CB36E73996862E612616D63EA80FBD6627F20FFDFe34AC27BA9`. Debug EXE SHA`8510F973BB842C68D5B89F0BD0AC5AFDB989D723547A0EA8E33186030883563D`. [MODLOG](MODLOG.md#2026-10-10--join-owned-biographies-and-configuration-ownership) owns failures/checks; [biography recovery](docs/virtual-ui.md#owned-biography-pointer-table-recovery) owns primary data/instruction/readback receipts. Earlier mode conservation evidence remains usable for its unchanged core. No new frozen package or game/native/DB process; original files/saves are unchanged. Actual Rust catalog loading/rendering, Windows owner/lifetime/restoration, file-reopen/cross-mode/retail/audio/human acceptance remain not tested.

**E115 — observed current37 model/decoder/build; native deferred:** A61ModeResume04 passed format, strict workspace/all-target lint, one integrated retained-scene/production-decoder/terminal-exit case and locked build/staging. Launcher`.scratch/evidence/20261010T231552367Z-e69c6106-build.local.json` SHA`2B9129BDBF01459D0837F895F2146D4D9962519AFEA914E07B5517406913153D` identifies`0ba1c8d` plus changed app/music/adventure/cli/new retained-scene module; source-set SHA`423C79BA0DB25D399C588D26DE680B8FDF593B302AD4E6B373591B1E8AE5FAF5`. Debug EXE SHA`249B8C0E84C3D4EC2296C89F56F5F47F39ECC4CA0EC1A58A735D524D4AA62FAB`. [MODLOG](MODLOG.md#2026-10-10--retain-mode-scenes-and-open-startup-menu) owns compiler failures01/02 and superseded narrower03 evidence. No broad suite, native/game/DB process, frozen package or protected-save/install mutation. Native input/render/file-reopen/cross-mode/audio/retail/human checks remain not tested; E80's genuine lineage and prior frozen builds remain historical.

**E114 — observed VT/gift compile/build; native deferred:** A60VTGift01 passed formatting, strict workspace/all-target lint and locked build/staging. Parent review then fixed closing-frame cancellation and immediate-transition tick handling;02 passed the changed-code gate. **No tests or native/DB/game process ran.** Launcher`.scratch/evidence/20261010T225024876Z-830fe635-build.local.json` SHA`5C979196E9D559DC20D4E5D11AD07A075F604649069FC2036D78DED9E4505EA6` identifies`f41308c` plus changed app/adventure/sim/virtual_tank/new input adapter, source-set SHA`427EC20F08FF2FA95A29CD63D4107C6855299A1BDBFD69E6464774EEA3FEE404`; debug EXE SHA`2D3DD0490C055907B709BFF3969A96D44947958038F35CD7DB9D38C4A0BBF14C`. No new frozen package or active earned lineage. [MODLOG](MODLOG.md#2026-10-10--join-virtual-tank-and-shell-gift-frontends) owns results. Native input/render/focus/audio, retail and human judgment remain not tested.

**E113 — observed frontend compile/build; native deferred:** A59Frontend01 stopped at three parent compile joins before build;02 passed formatting, strict workspace/all-target lint and locked build/staging. No tests, native game or Ghidra process ran. Launcher`.scratch/evidence/20261010T223713044Z-f49f9a80-build.local.json` SHA`DB772172D1A1D2E8CB89E8EF94E62A55A28FEB0A6EDC2C16887B467D82DF65E9` identifies`a63ccf8` plus changed app/main/three registered adapters, source-set SHA`09A2976B0ABF3A7FF53152383D1A1A5E7F5A678191334B017FA54368588BFA4F`; debug EXE SHA`AB0BA5EF99BD8DF59DCA44F8AA92335378C7E4D44849D5C1D42621E9FEFB6F82`. No new frozen package or active earned lineage was created; earlier frozen files stay preserved. [MODLOG](MODLOG.md#2026-10-10--join-main-menu-chooser-story-and-hall) owns failures/results. Native visuals/input/focus/audio, retail and human judgment remain not tested.

**E112 — observed animation/admission/interface checkpoint; native deferred:** A58Interface01 stopped on one equivalent-expression style lint before any case ran.02 passed formatting, strict workspace/all-target lint, one animation parser/timing case, one TT authoritative admission case and locked build/staging. No standalone UI/model suite was repeated. Launcher`.scratch/evidence/20261010T222127678Z-2b93ecf1-build.local.json` SHA`ACDD12F90FBF339551964A5100C2A26F78FAACA3A9F7A45DFCAFD32A3B715E50` identifies`afa6650` plus changed source/held adapters, source-set SHA`D2CA5466F6C360297F0B1E5587C6A3F7DA1F47A2EC7A2F7D07648C2FC99A826E`; debug EXE SHA`6098A4F060368483A60DC93FAA210A8C5FC97C4125349BCDF547DF0A76DAD874`. New model exports/held adapters alone do not prove app integration. No native/game/DB process, real save or install mutation. [MODLOG](MODLOG.md#2026-10-10--preserve-animation-and-admission-interfaces-for-parallel-joins) owns failures/results; native/retail/audio/human acceptance remains not tested.

**E111 — observed frontend compile/build; native deferred:** A57UI01 stopped at unsafe Help clipping and four style/unused-code joins. Replaced internal GL with safe CPU cropping and corrected the joins. A57UI02 passed formatting, strict workspace/all-target lint and locked build/staging. No test suite, new presentation case, native game or Ghidra process ran. Launcher`.scratch/evidence/20261010T221049651Z-fc13e044-build.local.json` SHA`ECB242C99F62D2456E033AF5F466CCE280E7C990337D42D5BB39DAFDF4018016` identifies committed`0884c60` plus dirty integration/held adapter inputs; source-set SHA`E64F141D6592B681D6ED15FB56D46BB79D8007904310F20587449D9F3BDF166A`, unchanged across preparation. Debug EXE SHA`0C7EC9E4B715B990303F7580D7071B28B902DA9B7181D648B10094921B811204`. Existing isolated model checks remain historical; exported models/held adapters alone are not runtime integration. Ordinary Title-to-selector routing, missing animation metadata and Help credits particles continue in the next joins; native visuals/focus/cadence/audio, retail and human judgment remain not tested. [MODLOG](MODLOG.md#2026-10-10--integrate-title-illustrated-help-and-full-finale) retains failure/results and receipts.

**E110 — observed scene handoff/build/package; native deferred:** One combined disk-backed prepared-state case passed: zero-income scene checkpoint, successive continuation, saver lease deferral, failed profile write retaining receipt/state, fish/Board/RNG transfer, preservation of Adventure progress/normal RNG/wallet/store, reopen without replay, stale-edit rejection retaining earnings, and explicit reset isolation. Initial style lints and two invalid selector-fixture assumptions are preserved; only the failed case reran after source-based fixture correction. Formatting/build passed; production strict lint passed before fixture-only edits. EXE/byte-identical `.scr` SHA`8C63F0D614D9E6E19A40DA3A91C18D04B18F696E532D76CC22AB6E5288EDCBE2`, frozen at`target/screensaver/ec3e293298c4febe59e1b6c86908a8d4bf8cca5015e2c8d4dbc5c24c6933a9d7`;73 inputs/14 copied artifacts plus host license matched. [MODLOG](MODLOG.md#2026-10-10--integrate-saver-scenes-and-board-bubbles) owns exact receipts. Session36 unchanged; required profile3/bank2 have no migration/backfill. Bubble timing/caps use pinned W1 with independent transient FX RNG; actual visual/audio/RNG fidelity and interlude managers are pending. No game/full-suite/native/multiprocess/power-loss check or OS/original-installation mutation; protected-input evidence remains historical. Failed empty test fixtures were removed; no user save was used.

**E109 — observed profile2 credit contract/build/package; native deferred:** One combined prepared-state file case passed: bank merging under a separate normal-writer lease, source caps, failed wallet save retaining watermark/credit, rename, reopen without recredit and fresh-generation isolation. Formatting/strict lint/build passed; initial hash-format and fixture metadata errors stopped before the case. EXE/byte-identical `.scr` SHA`0E2BD86F89978BC2C269B7309D4C124D6B3A99680AFF1CCC4C9361E0FB730BBB`, frozen at`target/screensaver/f326a0020f32b83606d3068656332e22d6eae29ae76a38a8ba225200bc8a8ab0`;70 inputs/14 copied artifacts plus host license matched. [MODLOG](MODLOG.md#2026-10-10--integrate-screensaver-credit-transactions) owns receipts. Profile format2 has no migration/backfill; session36 unchanged. No native/gameplay/full-suite run or protected-save/OS mutation. Saver fish continuation/production cap, actual multiprocess/native transfer and retail/audio/human comparison remain not tested or unimplemented as specified in the contract.

**E108 — observed host compile/parser/package; native deferred:** Formatting, strict workspace lint, one host parser safety case and locked build/staging passed. Initial CLI borrow/RAII-warning and later style failures stopped before cases. EXE/byte-identical `.scr` SHA`6D18055805A4F4009C15616E65D2DDAD33935D03F5AF04DC565FA912F2F3505D`; package is frozen at`target/screensaver/48e9594ace2be303631b718a4302778e574a8be9eb562d02bb40c15a11437bec`.68 unchanged inputs/14 copied build artifacts plus host license matched. [MODLOG](MODLOG.md#2026-10-10--integrate-windows-screensaver-host) owns receipts. No gameplay/full-suite/native check, profile/save change, system activation or original-installation mutation. Protected-input/cleanup evidence remains historical. `/c` optional parent is validated but dialog-owner parenting is not implemented. Saver income/fish continuation and actual Windows embedding/focus/monitor/render/audio/retail/human behavior remain pending.

**E107 — observed disposable-preview check/build; native deferred:** One prepared-state isolation case passed, proving fish motion/pet collection with byte-identical live state and unchanged persistent wallet. Initial fixture-type/style failures stopped before cases; the first case then exposed real entry stepping before the wallet guard. Action-only entry fixed it; only that failed case and build reran. Formatting/strict lint passed before that one-call lifecycle correction. EXE SHA`733B6186D50368E5E7D618A662DD2FA2B5929E31CB41C29E8E08B40E2CFBEAF3`, frozen at`.scratch/runtime/checkpoints/a53-code-01`;65 associated inputs/14 copies matched. [MODLOG](MODLOG.md#2026-10-10--integrate-disposable-screensaver-preview) owns receipts. Gameplay format remains36; separate auxiliary preferences do not backfill gameplay. No native run or Windows configuration change; protected-input/cleanup evidence remains historical. Windows host support, power-save backend, saver earnings and native/retail/audio/human acceptance remain pending.

**E106 — observed current36 focused checks; native deferred:** Two new Story rule cases, one ownership/browse/reopen/no-repeat-credit case and one opt-in owned-executable lore decoder case passed with formatting/strict lint/build/staging. Initial style lints stopped before tests; competing Story/finale music requests were corrected before the successful run. A later static join check corrected selector Story paused bookkeeping; compile/lint/build passed without repeating those cases. Final EXE SHA`F2D344D89C9E3794CF936C1C32832C11DAD58351BADFE26CD21CA2DF21642BF6`, frozen at`.scratch/runtime/checkpoints/a52-code-02`;65 associated inputs/14 copies matched, including the held unregistered next screensaver module. [MODLOG](MODLOG.md#2026-10-10--integrate-help-and-earned-stories) owns receipts. Required story ownership has no save backfill; pre36 saves are rejected without changing their files. Existing historical constructors remain in source. No game launched, original executable executed, registry altered or protected save changed; previous E80/earned-save/cleanup evidence remains historical. Native/input/visual/audio/retail/human acceptance remains not tested.

**E105 — observed renderer compile/lint/build; no tests executed:** VT gameplay sheets now cover bought species/size/hunger, typed food/prey and distinct coins. An initial compile attempt found one wrong setting field; after correcting it, strict lint and locked build/staging passed. EXE SHA`42550CD8DEE528EA8B25C95EE388454B37D7D3BEF3115255CDEC989335FE06C4`, frozen at`.scratch/runtime/checkpoints/a51-code-01`;63 associated inputs/14 copies matched. Inputs include the held, not-yet-registered next Help module. [MODLOG](MODLOG.md#2026-10-10--render-virtual-gameplay-sprites) owns receipts; [VT](docs/virtual-tank.md#current35-gameplay-sprite-checkpoint) owns source/presentation limits. No game launched or protected path changed; previous E80/save/cleanup evidence remains historical. Visual/audio/retail/human acceptance is not tested.

**E104 — observed current35 audio/trait checks; native deferred:** Formatting, strict workspace lint, one new VT music-route case, one affected current35 feeding/coin-accounting/save case and locked build/staging passed. The first attempt stopped before tests on one collapsible-if lint. EXE SHA`F113E2F27432AEE9632A3DB4E2B7F06983FFBF11185D4D93566D78DB8D8AD302`, frozen at`.scratch/runtime/checkpoints/a50-code-01`;62 associated inputs and14 copies matched. No game launched. Unchanged E80/earned-save readback and process cleanup remain the historical E103 observations; they were not repeated. [MODLOG](MODLOG.md#2026-10-10--join-virtual-audio-and-trait-effects) owns receipts; [VT](docs/virtual-tank.md#current35-audio-and-trait-checkpoint) owns timing/projection limits. No schema/migration/backfill or Ghidra count change. Native/audio/visual/retail/human acceptance is not tested.

**E103 — observed current35 focused checks; native deferred:** Formatting, strict workspace lint, two Virtual Tank combat and one active-state persistence case and locked build/staging passed. Earlier runs stopped on three style lints and two fixture errors (unnamed checkout and objects placed at their expiry floor); after reassessing the fixture, only its failed case reran. EXE SHA`BD113A7FD741A6A751E51FCE37D1847F7E32F56E3C21AAA21E7E03BCE6FABC61`, frozen at`.scratch/runtime/checkpoints/a49-code-01`;62 associated source/tool inputs and14 copied files matched. E80 and its earned save matched; zero owned games. [MODLOG](MODLOG.md#2026-10-10--integrate-virtual-combat-and-pets-and-preserve-current35) owns the compact receipts. Purchased fish are outside transient combat/prey/corpse lists. Native/input/render/audio/retail/human acceptance remains not tested; no migration/backfill or new Ghidra counts.

**E102 — observed current34 focused checks; native deferred:** Formatting, strict workspace lint, two profile/two song/one global-score/one Penta/one finale/one current34 persistence cases and locked build/staging passed across held-source runs. Passed filters were not repeated. An incorrect song fixture pitch and a real early-applause scheduler bug were corrected against pinned W1 after the initial style-only stop. EXE SHA`CDE8E10355A3E4062D06413A183FB915B14AF88253AB259E553C4AC6220DB4A0`, frozen at`.scratch/runtime/checkpoints/a44-code-01`;60 associated source/tool inputs and14 copied files matched. E80 and its earned save matched; zero owned games. [MODLOG](MODLOG.md#2026-10-10--integrate-sandbox-and-shared-interface-and-preserve-current34) owns retained failures and compact receipts. Profiles use the selected whole session and atomic manager writes; Sandbox saves discard the live disposable Board. Global score names survive profile rename/delete. No full-suite, native, render/audio, retail or human acceptance was performed; no new migration/backfill.

**E101 — observed current33 focused checks; native deferred:** Formatting, strict workspace lint, two Virtual Tank/five music/one current33 persistence cases and locked build/staging passed. No full suite or owned-asset/music rerun. EXE SHA`89ED74700B041C4E56BC68B15DF0E95B1FC6F6E421B3B22B32ED566E21148A4D`, frozen at`.scratch/runtime/checkpoints/a42-code-01`;57 source/tool inputs and14 copied artifacts matched the associated launcher. E80 and its genuine earned save matched; zero owned games. [MODLOG](MODLOG.md#2026-10-10--verify-virtual-tank-and-preserve-current33) owns the compact receipt and actual compile/lint failures. Frontend profiles/options/song integration and broader VT actor behavior remain open. No native/manual/render/audio/retail/human acceptance claim or new migration/backfill.

**E100 — observed current32 focused checks; native deferred:** Formatting, strict workspace lint, nine Challenge library/one persistence cases and locked build/staging passed. Full workspace/music reruns were not performed. EXE SHA`87D55431572585747F889A6B78E727996D8F42D36913794212CCD1B697F45207`, frozen at`.scratch/runtime/checkpoints/a40-code-01`;52 source/tool inputs and14 copied artifacts matched the associated launcher. Six selected original/PB05/E80/earned-save inputs matched; zero owned games. [MODLOG](MODLOG.md#2026-10-10--verify-challenge-and-preserve-current32) owns the compact receipt and actual failures. No native/manual/render/audio/retail/human acceptance claim or new migration/backfill.

**E99 — observed current31 checkpoint checks; native deferred:** Formatting, strict lint,523 library/101 persistence/5 asset/4 font/1 binding/2 owned-music cases and locked build/staging passed in one complete gate. Four earlier affected cases overlap that gate. The static format29 relabel bypass was corrected before it; the same persistence case covers29 and30. EXE SHA`65827EE6354405545B59FC2CC8414BAD5C89FFDD952A4450EF7932D9F9834D16`;51 held inputs,14 launcher-byte-matched transfers and one snapshot-only73/73 readback passed. Seventeen selected preserved original/PB05/frozen/earned-save inputs matched; zero owned game processes. [MODLOG](MODLOG.md#2026-10-10--verify-fourth-slot-and-preserve-current31) owns receipts and limits. Native/manual/render/audio/retail/human acceptance is pending; no new migration/backfill.

**E98 — observed current30 Walter checkpoint checks; native deferred:** Full02 passed520 library/5 assets/4 fonts/96 persistence cases, formatting and strict lint; four stale fixture expectations failed. Only the persistence fixtures changed afterward. Guard01 passed those four corrected cases, final formatting/strict lint, the previously unrun1 binding/2 owned-music cases and locked build/staging. Thus632 distinct checks are covered across these runs; no passing code matrix was repeated after the fixture-only correction. EXE SHA`9FB51807E4BEBA2A77E1CABDAEF38B6262C78BAD3F821370A4AD42419738D5F6`; final51 held inputs,14 launcher-byte-matched transfers and one snapshot-only73/73 source/artifact readback passed. Sixteen preserved inputs matched, including E97/E96/E80, genuine earned save and selected installed-game files. [MODLOG](MODLOG.md#2026-10-10--verify-walter-and-preserve-current30) owns exact receipts and failures. No native game process or new migration/backfill; exact Widget input, render/effects, retail RNG/FP, audio and human acceptance remain pending.

**E97 — observed current29 Stanley code checks; native deferred:** Full02 passed619 checks (510 library/97 persistence/5 assets/4 fonts/1 binding/2 owned-music), formatting/strict lint/locked build/staging and50 unchanged inputs. A bounded static challenge led to a required nullable-key serde guard; only `src/missile.rs` and persistence fixtures changed afterward. Guard03 passed two Stanley persistence cases and one direct Board roundtrip; final lint/build passed with50 held inputs. [MODLOG](MODLOG.md#2026-10-10--verify-stanley-and-preserve-current29) owns exact scopes and retained lint/fixture failures. EXE SHA`70BD70FE6E137784C35FEB8CD66E529CEE7080AA0786533AF32FD87769421EAF`; launcher-associated66/66 and snapshot-only frozen72/72 readbacks, plus14 byte-matched transfers, passed. Nine selected source/review receipts and15 preserved inputs matched, including E96/E95/E80 and the genuine earned save. No native process or new migration/backfill; native/manual/render/retail/audible output/human fidelity remains pending.

**E96 — observed current28 Nostradamus code gate; native deferred:** Full03 passed605 checks (497 library/96 persistence/5 assets/4 fonts/1 binding/2 owned-music), formatting/strict lint/locked build/staging and50 unchanged inputs. [MODLOG](MODLOG.md#2026-10-10--verify-nostradamus-and-preserve-current28) owns exact inputs, synthetic-probe scope and retained failures. EXE SHA`BE5202044B55EE8063AC7598DABE312015B7EA57115154CF7803F31DB1BD2DAF`; associated-launcher66/66 and snapshot-only frozen70/70 readbacks passed, with14 byte-matched transfers linking the frozen files to that launcher build. Earlier E95/E80/genuine earned save remain unchanged. No native game launched or new migration/backfill added. Native/manual/render/retail/audible output/human fidelity remains pending.

**E95 — observed current27 first-purchase/Brinkley code gate; native deferred:** Corrected full03 passed587 checks, formatting/strict lint/locked build/staging and50 unchanged inputs. Focused05 passed16; unchanged independent probes passed24/24 after the M1–M4 motion/contact correction. [MODLOG](MODLOG.md#2026-10-09--verify-corrected-brinkley-and-preserve-first-purchase) owns exact inputs, reports, retained fixture failures and launcher identity. EXE SHA`F6F0C5E6762E30DF21F60AFD16442450AD1AD14BEB8583D26C26D829ED111250`; launcher-associated66/66 and frozen70/70 readbacks, plus14 transfers, matched. Static audit passed71 overlapping indexed rows/five explicit bodies/31 instruction-scalar files/12 preserved inputs/11 notes/three receipts. Earlier full01/02 and affected pre-correction checks remain historical. Two owned-music decoder checks passed; no game process launched or new migration/backfill added. Native/manual/render/retail/audible output/human fidelity remains pending.

**E94 — observed current26 replay code gate, native deferred:** Complete gate passed569 checks, formatting/strict lint/locked build/staging;50 held code/tool inputs stayed unchanged. [MODLOG](MODLOG.md#2026-10-09--close-replay-ownership-and-correct-bonus-result-accounting) owns failures and identities. EXE SHA`16CBA9C7D413A7FF0E961A466C0FEB48CBE688A7B57816C8241C2E50F3C0CE8D`; launcher-associated66/66, frozen70/70 and14 transfers matched. Static readback56 indexed export rows/22 instruction-scalar files/11 preserved inputs passed, including current25/E80/genuine earned save. Corrected independent probes reject live-shell empty-wait and completed-profile Boardless ordinary bonus/results while preserving valid controls. No game process launched or new migration/backfill added. Two owned-music checks skipped; native/manual/render/retail/audio/human acceptance remains pending.

**E93 — observed current25 logical Presto code gate, native deferred:** Complete gate passed557 checks, formatting/strict lint/locked build/staging. [MODLOG](MODLOG.md#2026-10-09--integrate-logical-presto-and-recover-bonus-pet-ownership) owns failures and identities. EXE SHA`151BF7FE150BDB6DFD97FD6BC91CEFBF79B0686E1731BA5FEFC1145A35D0C460`; launcher-associated66/66, frozen70/70 and14 transfers matched. Static readback42 indexed bodies/17 instruction-scalar files/ten preserved inputs passed, including current24/E80/genuine earned save. No game process launched or new migration/backfill added. Two owned-music checks skipped; native/manual/render/retail/audio/human acceptance remains pending.

**E92 — observed current24 Time Trial/actor code gate, native deferred:** Complete gate passed547 checks, formatting/strict lint/locked build/staging. [MODLOG](MODLOG.md#2026-10-09--implement-time-trial-and-preserve-the-current24-checkpoint) owns failures and full identities. EXE SHA`F29992F7125D6A9211ECF0D2D133E80698E350F2CBD3A0EAA6DDA5CAF50CFD93`; launcher-associated readback66/66, frozen readback70/70 and14 matched transfers passed. Static readback matched35 exports/14 instruction-scalar files/nine preserved inputs, including E80 and its genuine earned save. Current24 requires TT and actor pose/form fields; historical saves remain with frozen builds and no new migration/backfill was added. No native process launched; two owned-music checks skipped. Its initial Presto gap is closed by E93; extra actors and native/manual/retail/audio/human acceptance remain pending.

**E91 — observed current23 storage code gate, native deferred:** Complete gate passed523 checks, formatting/strict lint/locked build/staging. [MODLOG](MODLOG.md#2026-10-09--preserve-ordered-pet-storage-and-close-time-trial-transactions) owns failures/identities. EXE SHA`792489D5F865476304B21AD6225678DA6D9E03B6F962BCFA78140806712A48FC`; source/artifact readback65/65, frozen readback69/69 and14 matched transfers passed. Static readback matched22 A37/A38 exports, six instruction files and eight preserved inputs; three later TT-v3 hashes also matched. Current23 rejects former Option-shaped saves; historical earned saves stay with their frozen builds. No native process launched; two owned-music checks skipped. Presto replacement and TT runtime, retail/audio/human acceptance remain pending.

**E90 — observed A37 actor/constructor code gate, native deferred:** Current22 passed522 maintained checks plus formatting/strict lint/locked build/staging. [MODLOG](MODLOG.md#2026-10-09--recover-presto-callbacks-and-correct-pet-constructor-mode-rules) owns failures and full identities. EXE SHA`36DD8C09F3CAD5884A53F7673B69AA790B0DFD4B462839E87B4F66E610D5CCE7`; launcher/source/artifact readback65/65, frozen readback69/69 and14 byte-matched transfers passed. Primary readback matched12/12 exports and7/7 preserved inputs. A36's mistaken constructor expectations are corrected; unrelated historical acceptance remains usable for its identified frozen builds. No native game process launched; two owned-music checks skipped. Presto Board/UI entry, audio, retail and human acceptance remain pending.

**E89 — observed A36 code gate, native deferred:** Current21 passed516 maintained checks, formatting/strict lint/locked build/staging. [MODLOG](MODLOG.md#2026-10-09--implement-cyrax-and-preserve-the-tank5-code-checkpoint) owns failures and full identities. EXE SHA`65977F38133219D34688468799FE761AAC02684AE9CA37BDF5503EB5874D1D70`; launcher-source/artifact readback passed65/65, frozen readback69/69 and14 transferred files matched. Primary readback matched21/21 exports and6/6 preserved wrapper/payload/A35/E80/earned-save inputs. No native game process was launched; two owned-music checks skipped. Retail/audio/human fidelity remains pending.

**E88 — binary-derived broad first pass and checkpoint checks complete:**595 functions attempted/594 decompiled/one retained timeout; all594 export hashes match. Inspection logs reconcile290 indexed addresses (188 full/9 partial/93 unspecified scope) plus four historical addresses, separately from export success. Twelve usable subsystem outlines and exact gaps are in the [recovery map](docs/binary-recovery.md), which owns source/index/readback identities and challenged corrections. Documentation/private-ignore/whitespace and preserved A35/E80/earned-save checks passed; [MODLOG](MODLOG.md#2026-10-09--recover-every-remaining-runtime-subsystem-in-a-broad-first-pass) owns results. No runtime code or native acceptance changed. Relax activation, Walter register mapping, offline ageing, profile deletion, detailed navigation/score/audio behavior remain explicit follow-ups.

**E87 — observed A35 code gate, native deferred:** Current20 passed477 maintained checks, formatting/strict lint/build/staging. [MODLOG](MODLOG.md#2026-10-09--implement-angie-and-tank-4-5-and-preserve-the-code-checkpoint) owns counts, retained failures, source/executable and frozen-bundle identities. Source/artifact readback passed65/65; frozen readback passed68/68 with14 copied files matched to the identified launcher build. Angie now stops on first eligible contact even if revival is pending; mixed Tank4 projectiles retain strict ownership checks. E80 and its genuine earned save were rehashed unchanged. No game/native process was launched; two owned-music checks were skipped. Retail/audio/human fidelity remains pending.

**E86 — observed A34 code gate, native deferred:** Current19 passed453 maintained checks, formatting/strict lint/build/staging. [MODLOG](MODLOG.md#2026-10-09--implement-tank-4-4-and-gash) owns exact counts, failures and executable/gate/launcher identities. Source/artifact readback passed65/65, SHA`CD39F25DB575C3FC0CF5C2CB9601408D227ABD425A2FA83ED81FCCBCE01F2B9C`. Gash uses post-contact clock gating and published group coordinates after targeted review. E80 EXE and genuine earning06 save hashes remain unchanged. No native process was launched; two owned-music checks were skipped. Retail/audio/human fidelity remains pending.

**E85 — observed A33 code gate, native deferred:** Current18 passed433maintained checks, formatting/strictlint/build/staging. Counts, retained failure and executable/gate/launcher identities are in [MODLOG](MODLOG.md#2026-10-09--implement-tank4-3-and-defer-native-acceptance-to-continue-reversing). Source/artifact readback passed65/65, SHA`3622436E31F1283FD844B1A08D9E19B02114214BDB0B2C4FFE10E86BC78F8536`. No A33 native process or genuine earned-Amp endpoint was created. Two owned-music checks were skipped; retail/audio/human fidelity remains pending. Earlier E79–E84 retain their historical frozen-build scope.

**E79 — observed build gate:** A32 gate 04 passed formatting, strict workspace Clippy, locked native build/staging and 407 checks: 335 library, 5 assets, 4 fonts, 60 persistence and 3 binding, including 2 owned-music checks. Current17 continuation, required-field and pause regressions passed. Tank 4-2 native gameplay acceptance remains **not tested**.

| Identity | Recorded value |
|---|---|
| Executable | `target/debug/turbofish-deluxe.exe`; SHA-256 `06CB115C9DC4818B416DD17B166D3B22C2A304DE46111E86AE9080180A39D47A` |
| Gate | `.scratch/runtime/gate-adventure32-current-save-04.log`; SHA-256 `1781C3BE954D454E981C46196CE01042D95F5D8142482F3CC46A2775D7284D1E` |
| Checkpoint | `.scratch/runtime/checkpoint-adventure32-01.local.json`; SHA-256 `4D7B7E8A753499C6AD402B66E06925B3A1994DE8AC8A9682FE9E9C4471B2A904` |
| Toolchain | Rust/Cargo 1.95.0, stable MSVC, Rust 2024; pinned dependencies/Cargo.lock. Exact rustup channel pin pending |

The checkpoint identifies base `38c65be` plus A32 inputs now committed in `4b2d53f`. All 37 build-input hashes, 5 artifact hashes and the checkpoint hash matched when this handoff was shortened. The launcher subsequently changed for evidence tooling; its old hash is historical. Rust sources, the executable and other checkpoint artifacts remain unchanged. The integrated tooling gate passed all 407 checks; [MODLOG](MODLOG.md) owns the new receipts. Commands: [README](README.md#workflow-checks).

Focused runtime validation commands are available in the [README](README.md#focused-runtime-checks). All selectors and filter/input guards passed 17 CLI scenarios; the default full gate reran successfully with all 407 checks including owned music and the unchanged E79 executable digest. See [MODLOG](MODLOG.md#2026-10-09--add-focused-runtime-validation-commands) for the historical tooling receipt. Current native progress is below.

## Agent workflow tooling

**E80 — observed resume gate/freeze:** Current `b3e3d0c` plus the reviewed launcher notice-staging correction passed the full407-check gate `.scratch/runtime/gate-adventure32-resume-02.log`, SHA`736F729B1DE635C6FEC67E1AD27AED89C57BA81EA032F4210A69DF9428C83E34`. Focused commands passed8Bilaterus/7Board/4current17 persistence tests. Repeated preparation retains5DLLs/8notices and the unchanged E79 executable. Frozen bundle is `.scratch/runtime/tester/a32-bundle-02`; maintained identity `.scratch/evidence/a32-build-frozen-02.local.json`, SHA`F0E62CBCC2C9230B7E9BE59CC00752BC245C7FCA22E156EE52581B68B602F381`, records launcher-success-with-unchanged-inputs and passes70artifact/source readbacks. Transfer02 SHA`55647E83F3EEA644C81A4D79322B563D51D0BAC63F662D5398E9A46C32B2422F` retains the36unchanged historical inputs/one changed launcher. Preparation01/partial bundle01 are failed historical attempts; no run used them.

**E81 — observed entry, acceptance underway:** Maintained-helper entry02 copied the genuine source byte-exactly, matched complete `identity.start`, entered4-2 with200cash/SmallBreeder2points/NikoItchyNimbus/egg25000/FixedBilaterus, and persisted current17 with exact final/save session equality. Root inspected selector and tank frames. PID43600 exited0 without force; runtime install receipts report unchanged. Entry01's early identity-read failure and clean PID11788 exit remain retained. Natural earning/combat/Ultra, native reopen/pause and full cleanup/lineage acceptance remain pending.

**E82 — observed partial native progress:** E80 run04 survived six Bilaterus waves, recorded50Nimbus coin/131food conversions, bought actual Oscars and reached a normal Playing checkpoint at Board21508/current17. Root inspected the live Oscar/diamond/shop/pet pixels, independently matched complete start/final/save sessions, confirmed PID43680 absent and the unchanged runtime installation receipt, and passed51artifact checks. Private audit02 SHA`CC965CF499B8EF27C83553E5B3EB4F92FBAD0894C3C3A57580C140033B601AD0`. Its private summary failed on legitimate diagnostic rows; native state recovered and exited0. The exact save SHA`EEC50923AE7D9D9BE6AE1135B4DB808286FE132394B379EC670397A682708818` is the active normal-speed continuation source. Ultra/eggs/Amp/selector/pause remain pending.

Separate accelerated-test work changed maintained source/tooling during run04. Its failed current-source readback remains retained; E80's executable/artifacts are unchanged. All48original source bytes were reconstructed from`5a36243` and matched the frozen receipt in ignored `.scratch/runtime/a32-source-5a36243-01`; reconstruction SHA`9D7A7D0ADB32AD7EAA3D2B9D0FE692165888ED79A26DF82DC39D00094D9F7BBC`. This is historical source verification, not a new build. Continue E80 at1x with identified loaded helpers; do not mix the new timing build into its acceptance.

**E83 — observed Ultra, first paid egg and full current17 continuation:** [A32 report](docs/playtests/2026-10-09-adventure-4-2-native-a32-01.md) owns the natural Ultra/Oscar/2000treasure, real predation/hunger losses, first25000egg and exact game-written checkpoint findings. A guarded live-save copy retained Ultra/Oscar/activeBilaterus at Board52555, SHA`0E6395FC1E70C4E9C649FA7AC622076E3330D8A4467E6CCC533052439A08B2F6`. Reopen matched the complete session; pause held all tested live actors/group children/countdown/economy/RNG/IDs while session time advanced. Root inspected its pause frame and independently passed start/final/save/full-pause and64+44artifact checks; audit05 SHA`9582456AD8ABDE86A8A67A98056EFF7CEB4F6C968B5C9E877C2914F141210D92`. Both owned games exited0 without force; oldPID19400 was later reused by a newer unrelated PowerShell process, not left alive. Bare numeric PID absence is insufficient. Current earning06 uses actual05final SHA`5CD39690CA451BB2406CBD7CB3C00D75DF62034D20DC9E36C69BF01573168510`; eggs2/3 and Amp/selector/reload remain pending.

**E84 — observed safe stop at the user's direction:** E80 earning06 ended gracefully at20:30:10UTC, retained handle exit0/forcedfalse, unchanged installation receipt. Complete source/start and final/save comparisons matched. The earned current17 Playing save is `.scratch/playtests/a32-earned-06/save/adventure.json`, SHA`65527921C74FA4AF7CDF1929DAC0A32BC9BEDDC9C37F1E9685EF54FF1782492D`, Board103209/two eggs/21765funds. Manifest artifact readback passed75/75. Third egg/Amp/seventeen-card Locked4-3/reload is deferred; no selector test was launched. The report owns exact receipts and the close-helper's post-close null-exit diagnostic. Earlier E79–E83 entries are historical snapshots.

The separate timing change provides [explicit accelerated tests](README.md#accelerated-gameplay-tests) with isolated saves, muted audio and labelled timing. Its new build/suite/native evidence belongs to that change; E80's stopped lineage is preserved at1x.

- [Maintained playtest helpers](docs/playtest-helpers.md): 26 regressions and four identified Tank 1 tooling runs; owned processes exited and installation inventories matched. [Report](docs/playtests/2026-10-09-playtest-helpers-01.md).
- [Evidence commands](docs/evidence-tooling.md): 47 contracts, prepared-build/source/artifact readback, and a verified adapter for recorded helper telemetry. Report drafts keep behavioral verdicts **not tested**.
- [Compact delegation briefs](docs/delegation-template.md): reusable fields, explicit ownership/isolation and an unexecuted 4-2 review example; linked from AGENTS.

These maintenance checks retain historical records; they do not establish Tank4-2 gameplay acceptance. E80/E81 are the current resumed build/entry evidence.

## Gaps and blockers

- No missing access or information currently requires user input. The user resumed work; full completion is unproved.
- A32's third egg, one Amp unlock and seventeen-card Locked4-3/reload are deferred. E80–E84 establish only observed paths; preserve the actual game-written checkpoint without editing earned progression.
- Tank4-3 through5-1, actor/storage foundations and Time Trial passed complete code gates under [provenance](docs/provenance.md); native acceptance is deferred. The [first recovery pass](docs/binary-recovery.md) supplies every remaining subsystem's usable primary-supported outline and exact gaps. An inventory or successful export alone is insufficient. Remaining implementation scope stays open in [coverage](docs/compatibility.md).
- Retail comparison and human feel remain untested. [Original-game isolation](docs/original-observation.md) is unproved; startup can force fullscreen despite ScreenMode0. [Music](docs/audio.md) has native-load/queue evidence, while audible fidelity, loops/fades/mixing remain untested. Distribution/runtime-code licensing is undecided.

## Ownership

E127/#309's exact source/test/report transfer is closed under its [completion record](docs/necessary-fixes.md#tank-chooser-completion-and-follow-up-audit); Done version12/released was read back after joined acceptance. Root resumes ownership of these paths and owns final records/decisions. E126/#308's writer/test/report transfer remains closed; root's actual observer joins are checked and pushed. No engineering worker is currently assigned.

Root owns synthesis, integration, central records and final decisions. The [E125 completion](docs/modal-batch-303-307.md#completion-and-joined-verification) closes all five modal source transfers with actual production joins checked and pushed. The [E124 completion](docs/refactor-batch-298-302.md#completion-and-joined-verification) closes all five refactor source transfers. The [E123 contract](docs/performance-batch-295-297.md) closes source transfers for #295 launcher, #296 Title loading and #297 active VT views. Earlier#292-294 source transfers are closed; preserve all private build/fixture evidence. Ordinary gameplay is human-owned; no autonomous native lease is active. Subsequent necessary fixes require new scoped Actionables, exact ownership reservations and independent new chats/worktrees where useful. Completed workers and the heartbeat remain closed. A35 code ownership is closed in its [contract](docs/adventure-4-5.md#integration-and-checks). The broad recovery and table below retain historical ownership:

| Owner | Responsibility |
|---|---|
| Primary | `sim.rs`, `ultra.rs`, `rhubarb.rs`, library registration, integration, records/gates/audit/commit |
| Bilaterus owner | `bilaterus.rs`, `invasion.rs`, `rufus.rs`, `adventure.rs`, `cli.rs` |
| Presentation/Nimbus owner | `fish_pet.rs`, `app.rs` |
| Tester | `tests/persistence.rs`, isolated native evidence and owned-process cleanup |

Completed performance worktrees retain separate Cargo targets and disposable offline fixtures; all three timing leases are released. No shared process/database/native lease is assigned. No A32 or A66 agent-owned native process remains; historical cleanup is recorded. Preserve unrelated work and frozen artifacts; keep original installation read-only and project saves separate. Add no new legacy migration, backfill or import.

## Evidence ledger

| ID | Class / current scope | Evidence |
|---|---|---|
| E131 | Binary-derived boundary order; three baseline boundary failures corrected; affected checks/lint/build pass; one unrelated full-library failure reproduced on clean baseline; native pending | [Guppy wall-turn correction](docs/guppy-turn-boundaries.md), PB05 address ledger, private baseline control and identified supporting build |
| E130 | Reproduced remaining development-save acceptance paths;118 unique persistence and one affected library case pass after focused corrections; build/lint pass; native unrun | [Current-only codec cleanup](docs/refactor-save-codec.md#remaining-development-save-compatibility), identified private supporting build and preserved failure logs |
| E128 follow-up | Human-reported motion/pose gap reproduced in authorized isolated1x pixels/state; binary-derived animation-before-motion order; four offline full-update probes find zero added delay;85 sampled turn states match; retail comparison pending | [Motion/pose follow-up](docs/playtests/2026-10-12-guppy-turn-motion-followup.md), retained E128 library and identified normal-release bundle; all owned PIDs closed |
| E128 | Binary-derived guppy clock/mirror; reproduced three wrong-mirror assertions and corrected four focused offline checks; native feel pending | [Guppy turnaround](docs/fish-turn-animation.md), PB76, identified isolated build |
| E127 | Reproduced original extracted chooser gate; eight corrected offline regressions and matched production join; native unrun | [Chooser completion and bounded follow-up](docs/necessary-fixes.md#tank-chooser-completion-and-follow-up-audit), linked worker report and accepted build identity |
| E126 | Observed isolated Windows reader-lock failures and corrected actual optional-observer recovery; native unrun | [Joined checkpoint](docs/necessary-fixes.md#joined-checkpoint-for-actionable-308), writer report and current frozen build identity above |
| E125 | Observed focused offline modal/writer contracts and joined lint/release build; native unrun | [Modal completion](docs/modal-batch-303-307.md#completion-and-joined-verification), five linked reports and current integrated build identity above |
| E124 | Observed focused offline contracts and joined lint/release build; native unrun | [Refactor completion](docs/refactor-batch-298-302.md#completion-and-joined-verification), five linked reports and current integrated build identity above |
| E122 | Observed offline equivalence/measurements and integrated focused checks; native unrun | [Performance index](docs/performance.md#frame-profile-and-checkpoint-results), linked profile/VT/checkpoint reports and current integrated build identity above |
| E120 | Observed focused publication tests/lint/build; native correction not accepted | Build identity above; [Windows publication design](docs/DESIGN.md#windows-checkpoint-publication), [A66 attempts/probes/startup handoff](docs/playtests/2026-10-10-mode-continuation-a66-01.md) |
| E79 | Observed A32 build gate; reviewed rules, native pending | Identities above; [A32 contract](docs/adventure-4-2.md), [PB59-63 primary ledger](docs/behavior-contract.md#tank4-2-primary-ledger), [design](docs/DESIGN.md#second-tank4-integration), [provenance](docs/provenance.md) |
| E76/E78 | Observed A31 frozen build and eight audited native runs | [A31 report](docs/playtests/2026-10-09-adventure-4-1-breeder-rhubarb-nimbus-01.md): three paid 3000 eggs/Nimbus once, sixteen-card Locked 4-2/reload and exact Board pause; owned PIDs absent/install unchanged |

Installed binaries remain authoritative; WinFish is secondary. Tests alone do not prove retail fidelity. The [historical ledger](docs/history/2026-10-09-status-a32.md#evidence-ledger) preserves E1-E72; [later checkpoint entries](docs/history/2026-10-09-status-a32.md#current-build-verification) preserve E73-E79 and their supersessions.

## Next action

Current next action:

E130's requested compatibility removal is complete. Future save changes keep
current-only admission; older development files are preserved without import
or reconstruction. Normal gameplay stays human-owned.

E131/#319 corrects the separately reproduced wall-turn defect; the human can
rebuild normally and compare Adventure guppies at both walls. E128/#310 and its
food/speed observation remain checked historical work. No timer-speed adjustment
is supported by current binary evidence. Retail playback and human timing
acceptance remain pending; no additional autonomous run is scheduled. The
existing Time Trial fixture failure is recorded without changing unrelated code.
E129 cleanup is complete separately.

0. The [renewed necessary-fix audit](docs/necessary-fixes.md#renewed-necessary-fix-audit) is complete with no new verified normal-play finding or Actionable. #309 is complete, checked and pushed through its [completion and bounded clean follow-up](docs/necessary-fixes.md#tank-chooser-completion-and-follow-up-audit); Done version12/released with closed source ownership. #308 is complete, checked and pushed; Done version12/released. E125 is complete and pushed; #303-307 are Done with released claims. Preserve
all closed modal/refactor worktrees, reports and private evidence. E124's
#298-302 retain their separately recorded Ready/unclaimed tooling limitation;
do not retry that route or resume completed workers.

1. Human launches normally with the [startup command](docs/playtests/2026-10-10-mode-continuation-a66-01.md#default-profile-and-human-launch-handoff), plays and reports issues. No accelerated/muted/fixture flags are needed. Omit `--new-game` on later runs to resume. Do not inspect or stop the human session for automated testing.
2. Root checks reported issues and concrete remaining gaps for necessary fixes only. Create an Actionable for each verified new finding before implementation; reserve disjoint source ownership, dispatch new chats/worktrees for independent fixes, and commit/push checked progress to main. Completed #292-307 engineering and evidence stay preserved without redispatch. Do not restart A66 or the four prepared outcome lanes; preserve their work and failed receipts. No speculative polish, repeated intake or legacy migration.
3. Preserve E80 and genuine current17 earning06 unchanged. The old A32 egg/Amp/selector endpoint and other native scenarios remain unperformed historical acceptance, not a mandate to play through them automatically. [Relax](docs/sandbox.md#contract)'s retail activation remains unknown.
4. Keep inaccessible retail comparison, audible output where unobserved, and human feel/fidelity explicitly pending. Record actual human feedback separately from agent checks; build/unit results do not establish full gameplay acceptance or completion.

## Deferred detail pass

- **Malformed-save diagnostics, closed in E124/#299:** direct current-format admission removed the old version35 label-table panic; both affected diagnostics pass. [Codec completion](docs/refactor-batch-298-302.md#completion-and-joined-verification) owns the checked fix. #289's failed broader filter remains historical evidence in [performance](docs/performance.md#focused-checks-and-retained-failures), not an open task.
- [Challenge](docs/challenge.md): exact widget creation/input/dialog presentation, retail RNG/clock/row codecs and live fidelity.
- [Remaining modes/Virtual Tank](docs/binary-recovery.md): secondary-derived stock/properties/offline/coin rules and remaining primary comparisons; do not delay bulk integration for nonblocking parity.
- [Compatibility](docs/compatibility.md): presentation, effects/audio, menus and other detail gaps retain their owning contracts until implemented or compared.
- Historical unperformed native cases, including E80's earned A32 endpoint; inaccessible retail observations and human judgment remain pending. The current testing plan does not require autonomous replay of that backlog.

## Environment and reference build

Recorded Windows `10.0.26200.0`, PowerShell 7.6.6, Git 2.54.0.windows.1. Original reference: Steam App 3320/build250752, English v1.1 x86. Both main EXEs have SHA-256 `F54C2C6EE54B00AE6DA7F4BDE15FEB90D0A867ABA6B4151B0277C1F81DA3FF66`. [Full environment/inventory record](docs/history/2026-10-09-status-a32.md#environment-and-reference-build) is historical; rediscover machine-local paths on another host. Shared tools and limits: [analysis tools](docs/analysis-tools.md).

## Setup verification

[Archived setup checks](docs/history/2026-10-09-status-a32.md#setup-verification) and [failed approaches](docs/history/2026-10-09-status-a32.md#attempted-approaches-and-failures) retain their original scope. A66 helper and corrected driver parser/automatic-variable checks passed without script execution. Workflow-only checks passed35 required files/86 documents/972 links/320 anchors/45 included/37 excluded/189 eligible files. No Cargo/unit suite or second native attempt followed the A66 receipt correction. Current results are recorded in [MODLOG](MODLOG.md). No workItemId was supplied; Actionables was not updated.

Completed detail and former next-step directions belong to the [closed handoff snapshot](docs/history/2026-10-09-status-a32.md), [MODLOG](MODLOG.md) and linked playtest records. Keep this file focused on the active increment.
