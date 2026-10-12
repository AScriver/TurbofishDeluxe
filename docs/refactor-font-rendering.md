# Shared bitmap-font placement

Actionable #298 belongs to the [E124 ownership contract](refactor-batch-298-302.md).
This lane owns `src/font.rs`, the existing `tests/fonts.rs` (corrected from the
initial singular path by root), and this report. Root owns the two production
consumer joins in `src/app.rs` and `src/app/help_runtime.rs` after #302 releases
application ownership.

## Decision and behavior contract

The existing normal `RenderedFont::text` and Help `font_text` both walk layers
in order, restart each layer's pen at the supplied x, and place atlas rectangles
using the same glyph offsets and baseline. Extract that behavior into
`BitmapFont::for_each_glyph(text, x, baseline, visit)`. Each `GlyphPlacement`
contains `layer_index`, the unchanged `source: GlyphRect`, and destination
`x`/`y`. The callback keeps textures, color, draw parameters and clipping in the
renderers. It does not allocate a character or placement vector.

Preserve these existing rules, rather than changing measurement or rendering:

- Emit every rectangle in layer order, including zero-area rectangles.
- Missing glyphs emit nothing and do not advance the pen.
- Rectangle-less glyphs emit nothing but still contribute advance, spacing and
  their kerning to the immediately following input character.
- Apply kerning to the next input character even if that character has no glyph;
  do not bridge missing characters to a later supported character.
- Retain the original integer addition/subtraction before conversion to `f32`.
  Ascent padding, line spacing and point-size metadata do not rescale this path.
- Clipping does not affect placement or subsequent pen advances. Help keeps its
  existing crop/image helper, while normal text keeps `draw_texture_ex`.

These are source-supported contracts of the pre-refactor Rust consumers at
`a55f50a`, not newly established retail behavior. No additional reference source
or owned asset was read. Existing [provenance](provenance.md) applies.

## Root consumer joins

Normal text calls `self.metrics.for_each_glyph` and resolves each placement's
atlas with `self.atlases.get(placement.layer_index)`. Pass its x/y to
`draw_texture_ex` and convert its source rectangle into `DrawTextureParams`;
keep color and the other default draw parameters unchanged.

Help calls `font.metrics.for_each_glyph`, resolves the same atlas index, and
passes the source rectangle plus a destination rectangle of the same width and
height to the existing `image(atlas, source, destination, clip, color, false)`.
Using `get` and skipping an absent atlas preserves the old `zip` boundary.

Both joins are required before reporting the complete outcome. Root registers
no new module for this API because `font` is already public.

## Focused validation

Executed 2026-10-11 America/Phoenix (2026-10-12 UTC) from the #298 worktree.
The checked build is base `a55f50afcb272ace4fa1cb775c4c85ec71d7bd88` plus only
the identified font/test changes and this report. Rust `1.95.0`
(`59807616e1fa2540724bfbac14d7976d7e4a3860`, MSVC x64), Cargo `1.95.0`, default
debug profile/features, locked offline dependencies and two build jobs were
used. No seed, gameplay clock or input stream applies to these synthetic
metric fixtures.

| Actual command | Result |
|---|---|
| `rustfmt --edition 2024 src/font.rs tests/fonts.rs` followed by `cargo fmt --all -- --check` | Passed; only owned files were formatted |
| `cargo clippy --locked --offline -p turbofish-deluxe --lib --bin turbofish-deluxe --test fonts -- -D warnings` | Passed; production and affected test target |
| `cargo test --locked --offline -p turbofish-deluxe --test fonts` | 10 passed: four retained font checks and six new placement checks |
| `cargo build --locked --offline -p turbofish-deluxe --bin turbofish-deluxe` | Passed; no launch or runtime staging |
| `scripts/Test-Workflow.ps1` | Passed: 35 required files, 97 documents, 1048 links, 351 anchors, 45 included and 37 excluded paths, 205 eligible files |
| `git diff --check` | Passed |

The new checks use declared synthetic metrics and manually calculated fixed
coordinates. They cover positive/negative kerning and offsets, fractional
origins/baselines, missing input characters without kerning bridges,
rectangle-less advances, ordered layers with independent metrics and point
sizes, zero-area rectangles and Unicode. The clipped-consumer case intersects
the emitted destinations with a CPU clip and checks the still-visible glyph's
unchanged placement after an invisible glyph and space. This is an offline
consumer-boundary check, not execution of Help's native crop/draw path.

Private target, validated scripts and retained logs stay in `.scratch/font-298`.
`CARGO_TARGET_DIR` selects its `target` directory and `CARGO_BUILD_JOBS=2` bounds
compiler parallelism. `OPENMPT_LIB_DIR` and the child process's `PATH` reference
only the existing read-only package reserved by the batch contract. The four
existing font tests use disposable nonce-named temporary directories and remove
them after success. All validation children exited successfully. No native
game, input/audio scenario, service, database, installation or protected save
was changed or inspected for testing.

| Checked bytes | SHA-256 |
|---|---|
| `src/font.rs` | `6DBF68B5A9374F69C9AD44147EBD6B42DD5C6B7E2A75FEB5A60E02EE08B5564B` |
| `tests/fonts.rs` | `E5323E181E07227B71286979F21476B646A68A3B01FA3F1F2067E32DC99C5C09` |
| `Cargo.lock` | `1643FD7BE00AF38DB3DA764BE8E0C03CFC427EE368DCAAB7EF17A49B187F153F` |
| Private debug `turbofish-deluxe.exe` | `5AE8DF65C40CDA1695BB552132BBAA8386822E9FD5C36E1B008A4ACF51BF0BC2` |
| `lint-01.log` | `8A16756F8DF42CA76FF8B229F687D7B76ABBAF215837FB4B8796001063FBEA57` |
| `tests-01.log` | `3D6CAFC14210CCD0A447D5B7BAE1C3B6A540586978FE453463DE3E92367D06BB` |
| `build-01.log` | `52F9B17FB347A996475D657F25F62A33B21231AE32D6375DA7D69AA57455232E` |
| `workflow-01.log` | `7710068986D9D6C10FCD26DECF2C8EF0EEA6FC8501B3AEF3B638BE8A670BFF07` |

`format-01.log` is empty because both formatting commands succeeded silently.
No compilation, lint, font-test or build failure occurred. The tracking failure
below is retained separately.

## Tracking limitation and remaining integration

The claim call failed with `INTERNAL_ERROR`, correlation
`fa1a2b74-22a3-459c-b0fa-a5306577c778`, and `retryMode: never`. Scoped list and
inspection confirmed Ready version 6 with no claim. Root independently confirmed
the shared running Actionables release rejects a stored preparation activity
type (`ZodError` at `activity[1].type`). The required claim tool is unavailable;
root directed the installed workflow's explicit availability fallback. No
denied claim is retried and no alternate API, database or installed release is
changed. Tracking remains Ready/unclaimed; no lifecycle result is claimed.

The shared core contribution is checked and ready for root's two renderer
joins. Neither production caller was edited by this lane; complete normal/Help
consolidation remains pending until those joins and their affected checks pass.
The above build identifies the unjoined consumers. Root owns central records
and joined verification. Native rendering, retail placement and human visual
judgment remain pending.
