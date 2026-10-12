# Adventure guppy motion and turn follow-up

## Human report and scope

On 2026-10-12 the human ran the normal launcher and reported that guppy turns
looked unchanged. The clarified symptom is: "Flip animation seems to start
after the fish has already changed direction." This follow-up separates motion
reversal from the body becoming visibly turned. It does not accept the original
visual timing issue as resolved. The initial pass was offline. The human then
explicitly authorized a targeted native reproduction; that later observation
is recorded below. No additional runtime change was made.

## Build and primary evidence

The latest normal-launch receipt, captured at16:45:40UTC, identifies clean
`401da2161fa63e42cbb8d5789c74aa49ebf4a897`, release/default features, source digest
`b9e404d93d74653b0f0c31f887fb9eff9118c378305a7ada1ce2b1bc86fa973f` and executable
SHA-256`4695f89fb1ee5d0de77eb8fb5d286afe1ebec133c68ce8b1ec79009be69b8e90`.
Its16 captured artifact checks still pass. This rules out a stale on-disk
release for the prescribed launcher; the exact human process was not observed.
The private receipt is
`.scratch/evidence/20261012T164540505Z-b6be671a-build.local.json`.

The already identified [E128 target](../fish-turn-animation.md#target-and-evidence-identity)
and its retained PB05 instructions remain the primary source. Ordinary Fish
Update at`004f6970` calls animation slot`+164` at`004f7a4c-004f7a56`, then
integrates the new velocity into position at`004f7a80-004f7aa6` and commits widget
coordinates at`004f7aac-004f7ac0`. PB76's sign-change/frame rule runs before that
movement. There is no earlier anticipation of the velocity reversal in this
animation rule. Update instruction receipt SHA-256:
`286F5042B5FD38E9CB756D731A56D7A2052C6413F98074027E33671B494739EC`.

The owned installed `smallturn.gif` was visually inspected read-only; SHA-256
`D654B14667E61BB049812D8497D06CB64B3513C0F7F7261EE4DC43C3F47D3148`.
Its early guppy poses remain close to the starting profile, frame5 faces the
viewer, and frame9 faces the other direction. At the nominal28ms clock, frame5
first appears280ms after turn onset. That is authored-pose inspection plus
binary-derived arithmetic, not measured retail playback. It explains why motion
can visibly reverse before the body has visibly rotated much.

## Full-update probe

An ignored Rust probe links E128's retained debug library, runs the real
`AdventureState::tick`, and never opens a window or reads/writes saves. Seed21,
one Small guppy at(300,150), hunger400, speed modifier1, food on the opposite
side, and explicit updates isolate food-driven reversal. Both directions are
tested with initial speeds0.5 and1.0. The latter crosses zero before reversing.

| Initial velocity | First opposite movement update | First Turn/frame0 update | Added delay |
|---|---|---|---|
|-1.0|4|4|0updates|
|-0.5|1|1|0updates|
|+0.5|1|1|0updates|
|+1.0|4|4|0updates|

All four assertions pass: timer magnitude19, frame0 and the starting mirror
are already selected on the first opposite movement update. The later frame
sequence proceeds while movement continues, as the primary ordering requires.
This is executed simulation evidence; rendering was not exercised.

Raw scripts/logs remain in `.scratch/turn-animation-20261012`:

- `motion-probe.rs` SHA-256
  `989885B6B2EB78F2982A18A65F41E37119F51DF007AE74719EAF70C52C34D629`.
- `motion-probe.log` SHA-256
  `3756B66544A5DB3A66DD8D30F3E0D61C2DEC1494DC5924880790BBD125671E4B`.
- `motion-probe.exe` SHA-256
  `67A2200291D6903EFE3FA967527B8785F1A3815686A65DE229E20EEFB89580B5`.
- Retained library SHA-256
  `4428B85192D7E26B2E374E9617AC0DE5D8600673F17A212B82DA00376C7BEF6C`;
  Rust1.95.0, E128's source identity and17 matching supporting artifact checks.

Two initial guarded attempts stopped before compiling/running the probe because
unrelated concurrent legacy-save edits changed five current build inputs
(`adventure.rs`, `cli.rs`, `invasion.rs`, `sim.rs`, `tests/persistence.rs`). The
diagnostic read showed no turn-path change. The completed probe deliberately
uses the retained identified library rather than those new sources. It is
historical-build evidence, not validation of the concurrent edits. PowerShell
parser/automatic-variable checks pass. No unrelated source is edited or staged.

## Authorized native observation

The human authorized: "you may run the game to test or see an example for
yourself." Root owned the process, input, isolated saves/evidence and cleanup.
The frozen `native-bundle-01` contains the exact release executable identified
above and its checked DLLs. Fresh Adventure1-1, seed21, normal1x timing and muted
audio isolate the guppy turn. No retail process, human game or default/live save
was used. The105-second wall safety bound was retained; the completed run lasted
about21 seconds including startup and18 seconds of active capture.

The maintained `scripts/Playtest-Helpers.ps1` provided ownership-bound normal
clicks, GPU captures and cleanup. Its SHA-256 is
`3969A72903AA6EC3998DB23B0A5DCDA2EEA70E5AFF424FD97E395BE73CDF23F6`.
State and UI captures are accepted together only when their session ticks match;
unpaired captures remain private and are excluded from exact claims. No global
input was posted. After the first focus denial, subsequent runs requested zero
automatic activations and required the owned window to be foreground.

| Attempt | Observation and limit | Owned-process cleanup |
|---|---|---|
|native-run-01, PID1312|Windows denied focus at Title; no input posted|Exited unforced, code1; close error null|
|native-run-02, PID41860|Reached Adventure; focus lost before a food click, which was refused|Exited unforced, code0; close error null|
|native-run-03, PID14724|154 captures,106 paired, one food click; user switched to ChatGPT during observation|Exited unforced, code0; close error null|
|native-run-04, PID45912|69 captures,53 paired, three food clicks; no focus loss|Exited unforced, code0; close error null|

Focus loss ended the earlier attempts through the harness's cleanup policy.
These are not runtime crash observations. Run04 began at
`2026-10-12T17:14:35.9226440Z` and finished at
`2026-10-12T17:14:57.2985131Z`. The helper confirmed the owned PID absent and the
owned installation inventory unchanged. Isolated game-written saves and raw
evidence remain under `.scratch/turn-animation-20261012`; no original/default
save was read or hashed for this comparison.

Representative paired run04 frames show the motion/pose gap in both directions:

| PNG | Fish / tick | Turn frame | Motion | Visible body pose |
|---|---|---|---|---|
|frame-ui-45912-000996.png|1 /149|1|Right|Still close to left-facing profile|
|frame-ui-45912-001019.png|1 /155|4|Right|Near edge-on|
|frame-ui-45912-001054.png|1 /163|8|Right|Right-facing|
|frame-ui-45912-001880.png|1 /368|0|Left|Still close to right-facing profile|
|frame-ui-45912-001923.png|1 /378|5|Left|Head-on|
|frame-ui-45912-001944.png|1 /384|8|Left|Left-facing|

All61 observed nonzero turn-timer rows in run04 and24 in interrupted run03
match PB76's signed timer/frame rule:85 sampled rows, zero mismatches. This is
not85 separate turns. A fresh-context native Tester independently inspected
the paired pixels/state read-only, observed motion preceding visible rotation
in both directions, and found gradual sampled poses rather than an immediate
flip. Root retained responsibility for process/input/save cleanup. Run03 and04
repeat the same seed/start, so they are not independent RNG scenarios.
Sampling gaps prevent an exact claim about the first perceptible rotation or
unseen frames. The original executable was not run; original wall-clock cadence
and a paired retail visual comparison remain unmeasured. Audio was muted and
is not evaluated.

Private analysis receipt `native-turn-rows.local.json` SHA-256:
`C3BB2E673877C1B7878E4EBC596501DDFF6074F3E795BE08B0574DDF1CB083EF`.
The run04 result is `captured`; it is not full gameplay or human acceptance.
These runs identify401da21, not the subsequently committed unrelated E129
legacy-save cleanup at1fa5deb. No turn-path source changed in that cleanup.

## Verdict and next evidence

The original and E128 simulation start turning on the same update as reversed
motion. The initial turn artwork makes motion lead pronounced body rotation.
E128 corrected the sheet mirror only; it did not change that relationship.
The authorized1x observation reproduces the human's visible symptom and shows
the sampled gradual turn sequence. No extra turn-start delay was found, and
there is no primary evidence supporting shortening the motion/pose gap.
Effective retail cadence and visible fidelity still require a paired retail
comparison; the human's timing preference is not accepted as resolved. No
further necessary defect was verified, so Actionable#310 remains its historical
Done record and Actionables was not updated. All owned test processes are closed.

## Documentation verification

The validated private `Check-FollowupDocs.ps1` runs the maintained
`scripts/Test-Workflow.ps1` through `Invoke-CodexPowerShell.ps1 -Execute` on
committed1fa5deb plus the four owned turn documents in a disposable checkout.
Workflow passes35 required files,112 documents,1135 local links,408 heading
anchors,45 allowed and37 excluded paths, and238 eligible files. Parser,
automatic-variable and whitespace checks pass; raw capture/analysis paths
remain ignored. The final native receipt and turn-row digest match, and all
four owned process IDs are absent. The disposable checkout is removed.

The first whole-tree workflow attempt fails at an unrelated in-progress
`docs/DESIGN.md` link to `refactor-save-codec.md#remaining-development-save-compatibility`.
That concurrent work is preserved and excluded from the isolated result.
The first isolated invocation also rejects a positional `-Path` string before
workflow execution; named PowerShell parameter splatting fixes the invocation.
No additional Rust checks are needed for these documentation-only changes;
the new source cleanup is not covered by this turn report's verification.
