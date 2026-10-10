# Hall of Fame and explicit presets

This bounded implementation owns score preset admission and a renderer-independent
Hall of Fame controller/view. App rendering, profile transactions, asset loading,
clock/RNG ownership and native acceptance remain with the primary integration.

## Design and evidence

Base checkout: `d7821846e3dc2d6d2c32157eaf326d109195720f`, initially clean.
Pinned secondary source: W1 `f919b3c241cfd611c547f1653fb3514f500f761b`,
`source/WinFish`. The study/license boundary is already registered in
[provenance](provenance.md). New Rust structures are source-informed; no original
source, proprietary assets, original preset-name catalog or decompiler output is
bundled.

| Claim | Class and scope | Evidence / decision |
|---|---|---|
| HF1 | Secondary-source-derived presets | `HighScoreMgr.cpp::MakeDefaultHighScores`: 20 single-row Adventure stages, each tank's five times 3300/3360/3420/3480/3540 seconds; four five-row Time Trial tables 1000/900/800/700/600 money; four five-row Challenge tables 3300/3360/3420/3480/3540 seconds. Contrary to the initial brief, this function supplies **no Tank5 defaults**. |
| HF2 | Existing project score authority | `src/highscores.rs`: ordinary Adventure capacity1/ascending, Tank5 capacity3/descending, Time Trial capacity5/descending, Challenge capacity5/ascending; strict improvements and stable existing ties. Preserve public methods, serde shape, validation and empty `Default`. |
| HF3 | Secondary-source-derived screen | `HighScoreScreen.cpp::{DrawGameModePage,DrawPersonalRecordsPage}`: four tabs, two-by-two global tanks, four personal Adventure columns plus Time Trial/Challenge sections; global Tank5 section only after game completion. `WinFishCommon.cpp::GetPlayTimeString` supplies seconds formatting. |
| HF4 | Deliberate safety adaptation | Source game-page code can dereference absent Adventure entries/uninitialized iterators. Rust renders missing records explicitly and rejects invalid global snapshots. Source destructor assigns the remembered page in the wrong direction; caller-owned remembered page follows successful selection. |
| HF5 | Ownership invariant | Presets are an explicit fresh-ledger transaction. Build a validated candidate before assigning it; any existing row rejects admission without mutation. Opening/browsing a screen cannot save, seed, earn scores, rename/delete a profile, update gameplay or own global time/RNG. |
| HF6 | Observed static owned catalog | Read-only inspection of the English Steam executable, 3,787,832 bytes, SHA256 `F54C2C6EE54B00AE6DA7F4BDE15FEB90D0A867ABA6B4151B0277C1F81DA3FF66`, located all60 W1 preset-name strings. Numeric file offsets in `highscore_presets.rs` select those strings after full-file identity validation. This establishes string availability, not executed retail preset behavior. |

The source's Tank5 display denominator18 is presentation metadata, not a new score
validation bound. Existing project gameplay permits more physical survivors; the
screen must preserve the supplied numerator. Installed binaries remain game-detail
authority. Native input/render/audio, retail comparison and human judgment are
pending.

## Callable APIs and integration

Both modules are exposed additively from the already registered `highscores`
module: `highscores::presets` and `highscores::screen`. Existing `GlobalScores`
methods and its three-array serialized shape are unchanged.

`presets::PresetNames::load_owned(executable)` opens an owned executable read-only,
checks its size, reads at most the expected size plus one byte, checks SHA256 and
then reads bounded UTF-8/nul-terminated names. An unrecognized build returns
`PresetError::CatalogUnavailable`; inaccessible files return `PresetError::Io`.
There is no bundled catalog or automatic fallback. Callers can explicitly supply
project-authored names through the fixed arrays of `PresetNames`.

`presets::fresh(&names)` returns a validated candidate. For an explicitly fresh
manager, `presets::admit_fresh(&mut scores, &names)` assigns that candidate only
when every existing table is empty. Invalid names, invalid ledgers, or any
existing earned/preset row leave the supplied ledger byte-for-byte equivalent
under its current serializer. Do not call it from load, screen entry, profile
creation, default construction, rename/delete, or an ordinary result callback.
The app decides whether a genuinely new manager receives presets and owns any
atomic write. Tank5 starts empty and admits up to three earned survivor rows.

`screen::HallOfFame::new(remembered_page)` creates transient UI state. The explicit
default is Time Trial, matching W1's initial page1. `input(event, button_height)`
handles pointer movement/press/release, typed widget press/release, cancellation,
keyboard focus cycling, focused/direct activation and Escape. Press requests
`SoundIntent::ButtonClick` once; release on the same armed control selects a page
or requests Menu, without a second sound. Direct keyboard activation combines
press/release. `Intentions::remember_page` updates caller-owned app-lived page
memory; `exit_to_menu` requests screen retirement/selector return. Map the sound
to the existing `SOUND_BUTTONCLICK` owner. Browsing cannot change any score.

`view(ScoreSnapshot, &metrics, HallClock)` returns a bounded640x480 `HallScene`.
The snapshot borrows `GlobalScores`, `PersonalRecords` and the profile's
`adventure_completed` flag. Build the personal projection from these existing
authorities:

| Projection | Existing source |
|---|---|
| Adventure seconds, tanks1–4/levels1–5 | `progress.first_stage_best_seconds` and keyed `progress.later_stage_best_seconds` |
| Tank5 survivors | `progress.tank5_best_survivors` |
| Time Trial money | `time_trial_scores.personal_best`; preserve existing zero-as-absent policy unless an actual earned-zero record is known |
| Challenge seconds | `challenge_scores.personal_best` |

`Option` distinguishes absent records from earned zero; no personal defaults are
seeded. Global Adventure has four two-by-two five-stage sections; Time Trial and
Challenge have four two-by-two top-five sections. Adventure's lower tanks move up
when completed, exposing Tank5's earned rows. Personal Records has four Adventure
columns, an optional final-boss row and separate four-tank Time Trial/Challenge
sections. Missing values render `-----`; global Time Trial money is numeric,
personal money is comma-grouped, elapsed seconds use `m:ss` or `hh:mm:ss`, and
Tank5 shows survivor counts. Global invalidity returns `ViewError::InvalidScores`
before drawing, and missing/nonfinite metrics return `InvalidMetric`.

The scene supplies ordered typed fill/HSL, image-box/cel/additive, text/font/outline,
clip/layer and button-state descriptors. These are Hall-specific intentions, not
a new engine. Global names reserve room for the right-aligned value and clip
inside their200-pixel column; personal values clip within their column. The
frontend owns glyph/image-box drawing, HSL conversion, clipping and hovered/
focused/selected/pressed button chrome. Input coordinates use the same logical
space and actual MainButton height as rendering. `HallClock` is caller-owned;
text bobbing is the W1 acceleration effect, unrelated to test speed. The renderer
resolves wave cels through owned animation metadata. No simulation clock changes.

`HallAsset::resource_key` identifies `IMAGE_WAVE`, `IMAGE_SCREENTITLE`,
`IMAGE_SCREENTITLEHOLE` and `IMAGE_MAINBUTTON`. The base frontend already loads
ScreenTitle/MainButton; **Wave and ScreenTitleHole need explicit asset joins**.
Font keys are JungleFever17/12/10 outlines and ContinuumBold14 with its outline.
The latter two Continuum resources need explicit metric/render joins if absent
from the frontend. Unavailable assets/fonts must produce an app-visible error;
do not fabricate retail pixels or hide metric failures.

On entry, give a scene-local existing `BubbleMgr` the public `bubble_config()`:
bubble bounds `(0,60,640,420)`, fish bounds `(0,80,640,350)`, max20 bubbles/max3 fish,
spawn chances3. Caller may use `update_a_lot` to warm it, supplies scene-local RNG,
then ticks through `update` and draws `render_ops` at `HallDraw::CallerBubbles`.
Dispose it on exit. The screen neither allocates particles nor recovers general
bubble behavior, owns RNG, persists bubbles or mutates the underlying gameplay.

## Focused verification

Observed2026-10-10 in this isolated checkout, base `d782184` plus the three owned
Rust files/new modules and this document. The W1 files used above remained clean
at the stated pin. Toolchain: `rustc1.95.0 (59807616e 2026-04-14)` and
`cargo1.95.0 (f2d3ce0bd 2026-03-21)`, MSVC debug harness. Existing dependencies
only: exact serde1.0.229, serde_json1.0.151/float_roundtrip and sha2 0.11.0.

The ignored `.scratch/highscores/harness` registers maintained `highscores.rs`
and the existing `bubbles.rs` by paths. Cargo.lock, targets, scripts and logs
stay inside this worktree. The entire PowerShell validation script was checked
and executed with `Invoke-CodexPowerShell.ps1`; commands inside it were:

```powershell
rustfmt --edition 2024 src/highscores.rs src/highscore_presets.rs src/highscore_screen.rs
cargo clippy --offline --manifest-path .scratch/highscores/harness/Cargo.toml --target-dir .scratch/highscores/target --all-targets -- -D warnings
cargo test --offline --locked --manifest-path .scratch/highscores/harness/Cargo.toml --target-dir .scratch/highscores/target --lib highscores
cargo build --offline --locked --manifest-path .scratch/highscores/harness/Cargo.toml --target-dir .scratch/highscores/target
```

Formatting, strict lint and build passed. Seven filtered cases passed: the
retained copied-name/order/roundtrip case, explicit numeric presets/capacity/ties,
atomic fresh-only admission/no backfill, identity/string rejection, unit
formatting boundaries, immutable/empty/invalid snapshots and completion gating,
and tab/Menu gesture/page-memory behavior. Expected numeric/time rules come from
HF1/HF3 and the explicit project safety contracts. Bubble tests were filtered out;
no broad runtime suite or native gameplay/rendering matrix was executed.

An additional scratch driver ran `PresetNames::load_owned` on the identified
actual executable, constructed/validated all60 preset rows and verified the
empty Tank5 table. Harness executable SHA256:
`F3B51044938643560C54F7D417E3969E09F553126CA6E9973ED9B5C7727ACF34`.
Successful private logs: `validation-Initial.log` SHA256
`CCFE63B5DFCEE0849ECC0091DE4998B43D100A23E8E11CBCC144C8FE05F3D431`,
and `validation-Catalog.log` SHA256
`D2B1EE2E002360C21868652470C629E2AA10F50B7F7E2ECFD87D5A108CB3B8F6`.

Initial compile failed before tests: sha2 0.11's digest array does not implement
`LowerHex` for `format!("{:x}", digest)` (`E0277`). Converting each digest byte to
two hex digits corrected that error; the subsequent focused checks above passed.
No game, shared database/service/port, save or OS setting was used or changed.
The catalog and original installation stayed read-only. Scratch is retained
locally/ignored for receipts; the driver exited normally and owns no live
resource. No workItemId was provided; Actionables was not updated.

## Deferred integration and acceptance

Primary joins actual selector input/render/assets/fonts/bubbles/clock/sound,
the immutable personal projection and any explicitly fresh-manager preset
transaction. Central STATUS/MODLOG/DESIGN/provenance records remain primary-owned.
No source/retail score codec or original-save import is provided. Fine font/color/
image-box/button parity, render/audio/native execution, actual normal-speed
acceptance and human fidelity judgment remain pending. The implemented API and
focused checks do not establish a playable path or retail-equivalent UI.
