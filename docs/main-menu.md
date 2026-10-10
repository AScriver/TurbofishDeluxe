# Main-menu controller and scene

## Ownership and design

`src/game_selector.rs` supplies a transient, renderer-independent `GameSelector`.
The production adapter owns profile identity and progression, mode admission,
modal lifetime, asset/font/animation loading, Sandbox input, saves and navigation.
This controller receives immutable facts on every call; it never resets a
profile, loads a Board, spends shells or changes an unlock. It is deliberately
not registered in `src/lib.rs` in this independent implementation chunk.

The caller advances one UI tick with `update`. Pointer events change capture and
pointer position; `view` only projects state into ordered image, text, clip and
layer descriptions. Seeded character timers and effect sampling receive an
injected RNG; trophy sampling receives read-only pixel-mask access. Missing or
invalid dimensions, font metrics, animation mappings or masks yield explicit
unavailable entries. The known 640x480 selector coordinate system and four mode
button rectangles come from W1; other dimensions must come from decoded assets.

## Evidence and adaptation

All source-shaped behavior here is **secondary-source-derived**, not retail
acceptance. Read-only reference: W1 revision
`f919b3c241cfd611c547f1653fb3514f500f761b`, verified locally before study.
The already registered [provenance](provenance.md) and its recovered-game-code
license/study boundaries apply. This is original source-informed Rust, with no
upstream implementation, proprietary assets, decompiler output or Meryl prose
incorporated. Existing installed-binary corrections and production authorities
take precedence over W1. The normal-game reset in W1's constructor is excluded.

| Claim | Reference at the W1 pin | Scope |
|---|---|---|
| GS1 | `source/WinFish/GameSelector.cpp:12-107,228-289,568-585` | Layout, images/fonts, Adventure labels, stage/shell overlay, trophy and Meryl placement |
| GS2 | `GameSelector.cpp:137-226,553-565` | Blink/tail clocks, six divergent-hover updates, same-tick fade decrement, profile link visibility, sparkle lifecycle |
| GS3 | `GameSelector.cpp:373-445` | Press sound, typed navigation/dialog requests, locked mode explanation, strict Meryl click bounds |
| GS4 | `GameSelector.cpp:448-536` | Hint keys, speech crossfade and welcome/profile composition; original hint prose excluded |
| GS5 | `source/WinFish/WinFishCommon.cpp:25-35,54-70,215-232` | Lower button height and three horizontal states; Cyrax ordinal formatting |
| GS6 | `source/SexyAppFramework/{ButtonWidget,DialogButton}.cpp` | Pressed means captured and over; font baseline, pressed text offset, component image-box intent |
| GS7 | `GameSelector.cpp:102-104,353-370`; `source/WinFish/WinFishApp.h:85-130` | Lowercase, case-sensitive `give` character trigger, mismatch reset and noncharacter-key preservation |

Capture cancellation and release revalidation are project safety requirements:
leaving a button before release cannot activate it; focus loss, modal entry,
profile/context epoch change and explicit cancellation retire capture. Locked
mode buttons remain visible and clickable to request an explanatory dialog,
but cannot emit mode navigation. The caller must revalidate every intention at
the authoritative boundary before performing it.

The W1 hover-change counter does not reset when pointer returns to its current
hint or moves among different candidate hints. Preserve that observed source
rule rather than describing it as six consecutive stable-hover updates. On
change the fade is assigned 5 then decremented to 4 in that same tick, yielding
current/previous text alphas 51/204; four further ticks finish the fade.

Gold trophy sparkle starts after update 40, uses the silver trophy's opaque
mask as W1 does, tries at most 100 points, and has one active 1..59 clock before
reset at 60. Validated sheet columns determine which clocks draw a cel. Missing
mask/metadata or no opaque sampled point suppresses that cycle and reports
why; W1's unchecked final transparent sample is not rendered. RNG stream
identity/retail parity is not established.

## Integration contract

1. Register the module in `src/lib.rs` in the primary-owned integration change.
   Construct `GameSelector::new(&mut random)` on real selector entry. Preserve
   production's existing startup/profile normalization and `MusicCue::PetSelection`
   routing; this module does not independently reset a profile or request music.
2. Project `SelectorFacts` from the active profile and existing mode rules.
   `ProfileFacts` supplies ID/name, tank/level, completed/returning flags, prior
   Cyrax defeats, shells and an earned `Trophy`. Supply `Trophy::None` unless
   the authoritative bonus-purchase gate is satisfied; then silver/gold follows
   the authoritative champion fact. These are presentation facts, not new
   progression authorities. `ModeAvailability` is supplied separately. Missing
   profile still prevents mode navigation even if the caller reports available.
   Increment `context_epoch` whenever replacing the profile, session or input
   context. Supply actual modal/debug-key facts on every input/update/view call.
3. Implement the three narrow traits over immutable loaded data and caller RNG.
   `SelectorResources::image` returns decoded width/height/columns/rows;
   `font` returns loaded height/ascent/padding; `text_width` uses actual glyphs;
   `animation_cel` resolves actual AnimInfo for the supplied countdown.
   `hint_text` and `profile_link_text` optionally supply locally available text.
   Their defaults are short project-authored guidance and "Switch profile".
   `TrophyMask` exposes loaded silver-award alpha; it is never called by `view`.
   `SelectorRng` distinguishes character and effect draws. Do not invent frame
   counts, animation timing, image heights or mask pixels to fill missing data.
4. Transform OS pointer coordinates into the 640x480 selector coordinate system.
   Forward primary down/up, move, leave, explicit cancel and focus loss once.
   Deliver character events once through `SelectorInput::Character`; deliver
   noncharacter keys through `NonCharacterKey`, which leaves the `give` prefix
   alone as W1 does. No ASCII keydown/character double-feed. Modal/debug-key input
   cannot activate `give` or Sandbox. Focus loss, context change and modal/debug
   suppression clear partial `give` input as a project safety adaptation.
5. Keep the existing `sandbox::EntryCode`/`SelectorKey` in the production adapter.
   Reset that matcher on modal/debug/focus/context interruption. Only when it
   fires, project `sandbox::admitted` into `facts.modes.sandbox` and call
   `sandbox_activated(&facts)`. The controller implements no competing matcher,
   cursor/capacity mutation, wallet authority or Relax entry.
6. Call `update(&facts, &resources, &mut random, &mask)` once per caller-owned UI
   tick. It returns capture/cursor intentions, never performs transitions.
   Do not advance from rendering, elapsed wall time or repeated input polling.
   Call `view(&facts, &resources)` whenever drawing; handle its unavailable list
   explicitly. `SelectorScene` retains all nine button descriptions, including
   visible locked modes. A missing lower-button image or profile-link font/width
   has no invented hit rectangle. The four source-defined mode rectangles remain
   valid geometry even if their art is missing; the scene reports missing art.
7. Render `commands` in order, with canvas plus per-command clipping. `ImageDraw`
   contains resource/frame, validated source/destination rectangles, natural or
   component image-box sizing, alpha, blend and layer. Component button atlases
   use horizontal thirds for normal/hover/down. `TextDraw` has font, actual text,
   optional hint key, baseline/center or wrapped-center layout, color and underline.
   Wrapped hints use the 214x60 box at (64,40) and 20-pixel line spacing, with
   caller glyph metrics. Outgoing/current speech text crossfades; bubble remains
   opaque. Welcome artwork centers using its own decoded width, a deliberate
   correction to W1's use of the WelcomeTo width for both welcome resources.
8. Process `SelectorIntent` in emitted order. Pointer intentions let the adapter
   maintain native capture/cursor; `Button::Pressed` and `Button::Activated` are
   notifications. Play only `Sound::ButtonClick` on press, using `SOUND_BUTTONCLICK`.
   A matched release emits one navigation/dialog request; duplicate releases do
   not repeat it. Revalidate requests at the production boundary. Map mode
   navigation to existing admission/load/selection code, HallOfFame to the score
   screen, Help to selector-return Help, and Options/Quit/ChooseProfile to existing
   dialogs. W1 clears its Relax flag at button activation; that domain state
   remains the primary adapter's responsibility. `LockedMode` carries mode and
   caller lock reason (W1 dialog14); authored `LockReason::guidance()` can explain
   it. `OpenGiveDialog` is emitted after `give`; the separate shell-gift module
   owns actual transfer/dialog state. No gift occurs in this controller.

`SelectorImage::resource_id` maps all18 image roles to registered W1 resource IDs;
`SelectorFont::resource_id` maps the five font roles. `ButtonId::source_id` exposes
1/2/3/4/6/7/8/9/9999 for crosswalks; no dispatcher needs to depend on raw integers.
The four mode positions are Adventure(357,48,217,66), TimeTrial(359,142,213,48),
Challenge(359,212,213,48), VirtualTank(357,287,217,66). Lower button heights come
from actual decoded images, never a fixed fallback. Overlay text centers at
x465 with baseline107 for stage and346 for positive shells. Initial1-1 has no
stage subtitle; bonus level6 names its tank; Tank5-1 formats the next Cyrax attempt
with correct teen/ordinal suffixes. Profile-link geometry uses actual text width
at (97,90), and is hidden while the current delayed hint is not Welcome.

Actual app/render/input/audio/clock/Board/save joins remain owned by the primary
chat; none are claimed by this independent chunk. Source countdowns and update
units do not independently prove retail cadence. Every scene layer and missing
resource policy is renderer-independent; a descriptor alone proves no pixels.

## Focused verification

Observed on2026-10-10 in the dedicated worktree, base
`76931a8455daf3104e052c8f99f30b17cbc80524` plus only the two owned maintained files.
Toolchain: Rust1.95.0(`59807616e`, x86_64-pc-windows-msvc), Cargo1.95.0.
An ignored standalone Cargo harness references this maintained module by path,
has no dependencies, and keeps its target in this worktree's ignored scratch.
No registration or root-crate modification was needed.

Exact checks, executed through the inherited validated PowerShell wrapper:

```text
rustfmt --edition 2024 src/game_selector.rs
rustfmt --check --edition 2024 src/game_selector.rs
cargo check --offline --manifest-path .scratch/game-selector/harness/Cargo.toml --target-dir .scratch/game-selector/target
cargo clippy --offline --manifest-path .scratch/game-selector/harness/Cargo.toml --target-dir .scratch/game-selector/target --all-targets -- -D warnings
cargo test --offline --manifest-path .scratch/game-selector/harness/Cargo.toml --target-dir .scratch/game-selector/target
cargo build --offline --manifest-path .scratch/game-selector/harness/Cargo.toml --target-dir .scratch/game-selector/target
scripts/Test-Workflow.ps1
```

Formatting, check, strict Clippy, isolated library build and all six focused cases
passed. Cases cover capture/release with locks and context interruptions; six
divergent hover updates/same-tick fade/read-only ordered scenes; Meryl strict
click/countdown boundaries and unavailable animation metadata; silver-mask gold
sparkle onset/cel interval/lifecycle/100-attempt limit/unavailable mask; labels,
Cyrax ordinals/shell overlay/profile link/component states; and case-sensitive
`give`, interruption gates and externally supplied Sandbox admission. Expectations
come from the GS references and explicit safety contracts; fixture dimensions
are declared test data, not installed asset claims. No full runtime suite or
unaffected filter reruns were used. First Clippy attempt stopped on
`manual_is_multiple_of`; source-equivalent correction preceded the passing run.
No behavioral test failed.

Workflow check passed35 required files/77 documents/850 links/236 anchors,
45 allow/37 exclude/155 eligible files with the final integration handoff text.
Verified module SHA-256:
`2F52270F8820C6EACFE7F71E214D143B7C58D76DB88506FCB1AE945DD35F995F`.
Scratch retains the
isolated harness, artifacts and validated task scripts. No service, port, game
process, real save, shared analysis database or original installation was touched.
No workItemId was supplied; Actionables was not updated.

## Deferred details

Exact original Meryl prose/catalog, retail animation metadata, pixel/render/audio
comparison and human feel remain pending. No normal-speed native run, original
game launch or playability claim accompanies isolated code checks.
