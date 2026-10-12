# Shared UI primitives

Actionable #301 is the bounded UI lane in the [E124 contract](refactor-batch-298-302.md).
Root owns `app.rs` consumer joins after #302 releases that file. This report
records engineering evidence, not gameplay or retail fidelity.

## Decision and source boundary

Extract only existing text geometry and the fallback button's paint. Keep
screen-specific skins, font selection, colors, clipping, paint order and input
models in their current adapters. No new controller, saved state, dependency,
asset, schema or GUI framework is introduced.

The inspected baseline is `a55f50a`, including root's compile-safe `ui` module
registration, with no dirty source. The pre-refactor sources are
`src/app.rs` (`centered_text`, `main_button`, `ui_button`),
`src/app/records_runtime.rs` (`draw_story_view`, `draw_hall_scene`) and
`src/app/virtual_runtime/render.rs` (`Renderer::text`, `inset`). This is
**strongly source-supported project behavior**; no new retail, binary or
secondary-source recovery was performed.

## Public API and consumers

- `inset(Rect, amount)` keeps zero-clamped dimensions and the original origin.
- `centered_text_x(Rect, width)` preserves unclamped Records/MainButton centering.
- `centered_text_x_clamped(Rect, width)` preserves VT's nonnegative offset before
  adding the origin. Overflow stays left-aligned within its clip.
- `centered_label_x(center, width)` keeps the original fixed-center subtraction.
- `centered_ascent_baseline(Rect, ascent)` preserves Story's first-layer ascent.
- `text_block_baseline(Rect, ascent, line_height, line_count, centered)` preserves
  VT's first multiline baseline. Metric derivation, wrapping, later baselines
  and clipping remain in the adapter.
- `FALLBACK_BUTTON_STYLE`, `FallbackButtonStyle::label_position` and
  `fallback_button(Rect, label)` preserve the existing flat skin, one-pixel
  yellow border, white 18-point label, eight-pixel inset and 0.7 baseline.

VT and Records both use shared centering geometry through the same private
`centered_text_offset`. Records uses unclamped
centering; VT uses its explicit clamped variant. Story/Hall and VT retain their
existing view/control models. In VT the skin precedes text; in Hall the image
box precedes the conditional state outline and then text. The extraction
changes only calculations at those existing paint sites, never their order.
Story still paints background/bubbles, heading/title/portraits/body, chrome,
page counter and controls in that order. Fallback paint remains fill, border,
label; it adds no clipping.

## Root joins

After #302's source handoff, root replaces `app.rs::main_button`'s label x with
`ui::centered_text_x(destination, width)`, keeping its image box, font,
0.72 baseline and `(255,240,0,255)` color. Root replaces `ui_button`'s body with
`ui::fallback_button(rect, label)`. Screen-centered text uses
`ui::centered_label_x(320.0, width)` to preserve its original subtraction
association. These production joins are pending.

## Validation and identity

**Observed: 19 focused checks passed**, plus formatting, strict affected-target
lint and locked/offline normal-bin compilation. Expected
coordinates/styles come from the identified pre-refactor paint sites and
existing controller geometry, not generated screenshots or the extracted code.
Checks use this worktree's ignored Cargo target and the contract's read-only
libopenmpt package; no native window, input/audio run or real save is used.

All Cargo commands run with the existing native-library environment,
`CARGO_TARGET_DIR=.scratch/refactor-301/cargo-target`, two build jobs and
dev/test debug information disabled. Rust/Cargo identity and every checked
Rust/Cargo input are captured in the private build receipt. Commands:

```powershell
cargo fmt --all -- --check
cargo test --locked --offline --package turbofish-deluxe --test ui_primitives
cargo test --locked --offline --package turbofish-deluxe --lib virtual_ui::tests::
cargo test --locked --offline --package turbofish-deluxe --lib highscores::screen::tests::views_use_immutable_snapshots_and_keep_empty_records_safe -- --exact
cargo test --locked --offline --package turbofish-deluxe --lib story_screen::tests::long_unicode_text_is_bounded_scrollable_and_clipped_above_chrome -- --exact
cargo test --locked --offline --package turbofish-deluxe --lib story_screen::tests::composing_is_pure_and_focus_policy_is_explicit -- --exact
cargo clippy --locked --offline --package turbofish-deluxe --lib --bin turbofish-deluxe --test ui_primitives -- -D warnings
cargo build --locked --offline --package turbofish-deluxe --bin turbofish-deluxe
```

The primitive target passed7, VT passed9, Hall passed1 and Story passed2;
every selected filter executed its stated test count. No broad or native suite
ran. Whole-project format checking followed formatting only the four owned
Rust files. All task-specific PowerShell scripts passed parser and automatic-
variable checks before execution through the inherited validator.
`scripts/Test-Workflow.ps1` also passed35 required files,97 documents,1047
local links,351 anchors,45 included/37 excluded paths and206 eligible files;
this is a workflow-only result.

Private current receipts are
`.scratch/refactor-301/Format-20261012T045314920Z.log`,
`.scratch/refactor-301/Checks-20261012T045335434Z.log` and
`.scratch/refactor-301/Build-20261012T045536944Z.log`.
`build-identity-20261012T045536944Z.json` identifies `a55f50a` plus the four
owned Rust changes/new tests and report. It confirmed all90 Rust/Cargo input
hashes unchanged before/after compilation, with source-set SHA256
`84E13299255F065AFF1693B5E7D1F897C7D8F9A388C38A728D35D251F5B849FB`
and private unoptimized game EXE SHA256
`3FA342B9F197AD1E603B24A7A62C9415EE913BDA188A1ECDAE520BF24494E26B`.
The EXE was built, never launched or staged into the normal launcher.
Subsequent report-only edits do not change checked runtime/test inputs.

Owned code/tests/report were committed as `b360b05`, then reconciled cleanly
with concurrent main at `a5bf857`, preserving #298's font API and #300's
persistence extraction. The font changes add glyph placement without changing
width measurement or parsing, so existing UI checks remain applicable. Strict
affected-target lint and normal-bin compilation passed again for the combined
source, without rerunning unchanged UI tests.
`.scratch/refactor-301/Joined-20261012T045759460Z.log` and
`build-identity-20261012T045759460Z.json` capture the clean reconciled tree:
all91 Rust/Cargo inputs matched before/after build, source-set SHA256
`777C80CC12950E044019A2934D27C102B2FA89A225B2B19CB1803DCBBA46932F`,
EXE SHA256
`1BC7D7E234F36859878F63887AA0582CF309F807F9AF1CD8C38080F3374C0095`,
Rust1.95.0 MSVC and Cargo1.95.0, unoptimized dev profile/debug0/two jobs.
Root's `app.rs` joins and their affected verification remain pending; this
checked core contribution does not claim the whole Actionable is complete.

## Tracking and cleanup

The scoped task was Ready version6 and unclaimed. The sole claim attempt
returned `INTERNAL_ERROR`, correlation `7765257a-9a7b-451d-9ecf-2b5ac440c80d`,
with `retryMode: never`. Readback confirmed no persisted claim. The installed
runtime rejects an activity type while reading task detail. Root authorized
the installed workflow's unavailable-required-tool fallback and records it
centrally; tracking remains **Ready/unclaimed**, not In progress or Done.
No retry, database/service/release change or alternative lifecycle API was used.

Two initial patch attempts expected an incorrect scaffold comment and made no
changes. After the second failure the scaffold was reread before applying the
extraction. The first primitive test run passed six tests and failed the Story
fixture with `Invalid("missing default point size")`; the synthetic font was
given its required image name, point size and default point-size metadata.
Production helpers did not change for that correction. Failed output remains
in `.scratch/refactor-301/Checks-20261012T045212101Z.log`. No temporary module
registration or shared resource was needed.
The worktree is clean after the owned commits and reconciliation. Private
scratch/build evidence is retained. No shared resources, processes, services,
live saves, frozen builds or original installation were acquired or changed.
Production joins remain with root; its final integration receipt should record
their validation before reporting the entire refactor complete.
