# Normal-game runtime state extraction

Actionable #302 is the normal-game lane in the [E124 ownership contract](refactor-batch-298-302.md).
Only `src/app.rs`, `src/app/runtime_state.rs`, and this report are owned here.

## Decision and evidence

Baseline: clean `a55f50a`, including the root's compile-safe persistence/UI
registrations. Evidence below is **observed source structure**, not a new retail
fidelity claim. No original game, reference checkout, or installed assets are
studied for this extraction.

| Claim | Baseline evidence | Scope and consequence |
|---|---|---|
| RS1 | `app::run` input declarations and reset sites, `src/app.rs:6155-6162,6526-6531,7000-7008,7133-7138,7879-7912,8248-8272` | Feed/fire cancellation, queued actions, pointer release, and hatch exit have different reset boundaries. Preserve each boundary and accepted-action ordering. |
| RS2 | Help opening/return and profile reset, `src/app.rs:6493-6502,6700-6702,6718-6723,6992-7008` | Tank Help restores the previous pause value; a missing return value defaults to paused. Selector/Continue exits do not restore it. Mode and profile transitions differ. |
| RS3 | Singing-fish bookkeeping, `src/app.rs:6202-6216,6689-6696,8138-8235,8394-8410,8499-8500` | Feed detection precedes the same-step countdown. Profile switch clears active/pending playback and feed counts but retains song sequence, ID, and title deadline. Tone output follows physical-frame playback gating. |
| RS4 | Clocks and persistence gates, `src/app.rs:6139-6141,6193,7981-8032,8247-8295,8625-8688` | Gameplay, menu, interlude, preview, wall, and held-input clocks retain their roles. Save/exit accepted actions and once-only result publication stay in the existing application/domain paths. |

These claims are strongly source-supported for the identified baseline.
The extraction does not establish audible behavior, native input, retail
fidelity, or playability. Earlier build/native evidence remains historical.

Three private, concrete state owners are sufficient:

- `InputCapture` owns the queued actions and mouse capture timestamps/flags.
  It exposes distinct hold cancellation, queued-input cancellation, pointer
  release, and hatch reset methods. Feed arming continues to require the
  existing accepted-event route check; `StepClock` supplies press times.
- `ModalPause` owns the current pause flag and the optional tank-Help restore
  value. Help controllers and mode selection remain with their existing owners.
- `FishSongRuntime` owns player/feed/countdown/selection/title bookkeeping.
  It consumes the already loaded catalog and caller-supplied wall times; the
  existing audio owner still plays tones and stops the previous song before a
  profile reset. Song selection and update order are preserved.

No generic application context, GUI framework, clock consolidation, new save
schema, new abstraction for mode admission, or screensaver extraction is needed.
Root's later font/UI joins can still edit the ordinary `app.rs` consumers after
this lane explicitly transfers that file.

## Validation and handoff

The three state owners are registered privately from `app.rs` and integrated
into its production callers. Input timestamps still come from `StepClock` at
the original press sites. Mode admission, event accumulation, accepted-action
save/exit boundaries, paused stepping, and score/result publication stay in
the application and existing domain controllers. `MusicOwner` still stops the
current song before profile reset and plays emitted tones after the frame's
normal event sounds. The song clock callback preserves the two original wall
time samples at successful start/title publication. No root module
registration or additional runtime-state consumer join is needed.

All commands run through the host's `Invoke-CodexPowerShell.ps1 -Execute` after
parser and automatic-variable checks. Task scripts remain ignored under
`.scratch/refactor302`. Cargo uses a private target directory, two build jobs,
the existing read-only libopenmpt 0.8.9 package, and locked/offline dependency
resolution. These are offline agent checks on `a55f50a` plus this contribution,
not a gameplay or retail comparison. New song fixtures are synthetic strings
parsed in memory; their explicit caller contract uses a five-update countdown,
caller-provided times, and existing source selection/reset rules.

| Command | Executed result |
|---|---|
| `cargo test --locked --offline -p turbofish-deluxe --lib app::runtime_state::tests::` | Eleven selected: eight passed; three fixture-syntax failures retained below. |
| Same command with each exact corrected filter below | All three corrected cases passed; all eleven new contracts have passed, without repeating the eight unaffected cases. |
| `cargo test --locked --offline -p turbofish-deluxe --lib <regression-filter>` | Nineteen existing affected regressions passed. One serial timing case stayed ignored. |
| `cargo fmt --all -- --check` | Passed after final code changes. |
| `cargo clippy --locked --offline -p turbofish-deluxe --lib --bin turbofish-deluxe --tests -- -D warnings` | Passed, including the new library test code. |
| `cargo build --locked --offline -p turbofish-deluxe --bin turbofish-deluxe` | Passed, development profile. No launch or shared runtime staging. |
| `scripts/Test-Workflow.ps1` | Passed: 35 required files, 97 documents, 1,047 local links, 351 anchors, 45 included/37 excluded path checks, 206 eligible files. |
| `git diff --check` | Passed. Scratch/target paths are ignored; maintained module/report paths are included. |

Corrected test filters:

- `app::runtime_state::tests::fifth_feed_counts_trigger_step_and_samples_wall_clock_only_at_start`
- `app::runtime_state::tests::latest_eligible_feed_in_a_step_owns_pending_special_selection`
- `app::runtime_state::tests::profile_reset_clears_playback_and_feeds_but_keeps_sequence_and_ids`

Existing regression filters and counts:

- `app::feed_input_tests::gus_hold_arms_only_from_the_pressed_click_route`: one.
- `timing::tests::`: six, including wall/accelerated hold origin, pause/backlog,
  deadline, clamp/remainder, and phase barriers.
- `fish_songs::tests::score_lines_rest_and_timed_applause`: one.
- `sim::tests::max_weapon_hold_fires_on_old_board_count_with_strict_elapsed_gate`: one.
- `adventure::tests::manual_pause_decreases_only_pre_pause_delay_and_keeps_board_clock`: one.
- `adventure::tests::hatch_background_hold_skips_intro_but_never_unlocks_continue_early`: one.
- `cli::checkpoint_tests::`: eight passed; `measure_checkpoint_costs` remained
  ignored because no serial timing lease is part of this lane.

The first eight successful state checks cover input cancellation/accepted-action
retention, hatch release, Gus route/clock identity, tank and selector Help
restoration, profile versus mode pause reset, missing catalog, and paused tone
resumption. The three corrected cases cover same-step feed/countdown ordering,
last-eligible named-fish selection, and process-lived sequence/IDs including
the fifth song's extended partner. Expectations preserve the recorded baseline
contract; these checks do not independently establish retail fidelity.

## Failures retained

1. The private caller-join script's first guard rejected the legitimate
   `ToneCommand.song_id` field as an unjoined variable. The guarded script made
   no production write on that failure. The identifier check now excludes
   member access; evidence strings/keys were not rewritten.
2. First test compilation failed with `E0609` at the tank/VT input loops:
   the aggregate name `input` collided with existing `TankInput`/`UiInput`
   locals. The compiler also reported the moved `NamedFish` import unused.
   The aggregate is now explicitly `input_capture`, the import is removed,
   and each cancellation/arming caller was inspected. No tests executed in
   this attempt.
3. The next eleven-case run passed eight and failed three at fixture parsing:
   `unknown command: title` and `unknown command: santa`. Source inspection
   established the existing `*attrib name = text` grammar and filename-derived
   `_short.txt`/`_long.txt` partner mapping. Only those fixture strings changed;
   the three affected cases then passed. Production admission was not loosened.

Failed and successful logs remain separately named in
`.scratch/refactor302/checks`; no receipt or failure log was overwritten.

## Checked build identity

The private immutable `.scratch/refactor302/identity.local.json` records
`a55f50afcb272ace4fa1cb775c4c85ec71d7bd88` plus the two owned Rust-file changes,
synthetic in-memory fixtures, commands, and dependency/toolchain identity.
Rust is `1.95.0 (59807616e 2026-04-14)`, host `x86_64-pc-windows-msvc`,
LLVM `22.1.2`; Cargo is `1.95.0 (f2d3ce0bd 2026-03-21)`. No original-game
version or gameplay input identity is claimed because none was exercised.
The report's captured digest is historical for that receipt; subsequent
documentation-only updates do not change the checked build inputs.

| Input/output | SHA-256 |
|---|---|
| `src/app.rs` | `8571C93AA08AC5D9C69F59C458B1961677FB8AFE00B464E1A8E3BC754D74D9E3` |
| `src/app/runtime_state.rs` | `3AD655BF9916EDFE3A057AAD7D078FF94539F09572FA2429AC9253371614B7C0` |
| `Cargo.toml` | `0642F06EC0072E64EBE85FA1F5AD547C6AD825F6A57C3BDD025C9B7D2584DDFC` |
| `Cargo.lock` | `1643FD7BE00AF38DB3DA764BE8E0C03CFC427EE368DCAAB7EF17A49B187F153F` |
| Private development executable, not launched | `E1C436ECBF556E3C5414523A28D55F937C3DE38BD88A34E0C77FF9D72B749123` |
| Normalized source from `run_screensaver` through the file's end | `8937B8D4DC2C08E5B3A0046793366CEAC6E9A69294488C7560C5088620041458` |

The excluded screensaver and existing test tail match the baseline exactly
after newline normalization. The new module is private to `app`; no temporary
shared module registration was necessary or remains. All invoked compiler/test
processes exited; scratch receipts, synthetic fixtures, and target artifacts
are retained for review. No other process or resource needs cleanup.

## Tracking and transfer

The shared Actionables runtime rejected the claim operation with
`INTERNAL_ERROR`, correlation `02bebb85-ecf9-4349-8d74-c3944da30447`,
`retryMode: never`. Scoped readback confirmed #302 Ready v6, unclaimed, with no
mutation persisted. Root identified an unsupported preparation activity type
in the installed runtime's validation and authorized the installed workflow's
required-tool-unavailable fallback. No claim retry or alternate API bypass is
used. Engineering progress and tracking state are reported separately; no
Actionables In progress or Done status is claimed.

All work uses this managed worktree's ignored `.scratch/refactor302` and private
Cargo target with bounded jobs, plus the existing read-only native package.
No native run, installation change, service/database change, real save access,
or human game interaction is authorized here.

Runtime-state engineering and its production callers are integrated. After
this checked contribution is pushed without force, the lane transfers
`app.rs` back to root for the reserved font/UI joins. There is no additional
runtime-state API join request; all three types and their methods are scoped
`pub(super)` in a private application module. Root's combined batch review and
font/UI joins remain separate from this lane's checks. Actionables remains
Ready v6/unclaimed under the tracking exception; no Done status or claim
release is invented.
