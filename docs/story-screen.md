# Story screen composition

This bounded increment adds `src/story_screen.rs`, a renderer-independent,
transient controller and concrete 640×480 view. The primary chat owns library
registration, assets, rendering, audio, input and session integration. It does
not change awards, the catalog, saves, profiles, Challenge return transactions
or finale/interlude behavior.

## Evidence and decisions

E113 registers the actual [runtime frontend join](shared-interface.md#runtime-frontend-joins), including owned talk-show/portrait/WAVE resources, typed native input, scroll and separate effects. Challenge navigation retains Session authority; standalone browsing retains earned ownership without advancing simulation. Formatting/lint/build passed; no Story model cases were repeated and native/retail/audio/human acceptance remains not tested.

All presentation facts below are **secondary-source-derived**, studied read-only
at W1 revision `f919b3c241cfd611c547f1653fb3514f500f761b`. Source-informed work follows
the existing [provenance](provenance.md#runtime-sources) boundary; no upstream
implementation, original graphics or story prose is incorporated. Installed
binary authority and PB05's corrected award rules stay in `src/story.rs`.

| Claim | Pinned W1 source | Scope |
|---|---|---|
| SS01 | `StoryScreen.cpp:87–131`, `WinFishCommon.cpp:97–109` | Ordinary bubbles, hyperlink font/colors and measured control positions |
| SS02 | `StoryScreen.cpp:165–173`, `Widget.cpp:208–211`, `WidgetContainer.cpp:474–477` | Widget portrait counter advances per supplied update; bubbles/redraw require focus |
| SS03 | `StoryScreen.cpp:175–216,235–255`, `Image.cpp:179–185`, `Res.h:540–576` | All 33 portrait mappings, font roles, resource animation, Rufus row, text origins and chrome |
| SS04 | `StoryScreen.cpp:217–233` | Eight-part Bilaterus assembly, mirrored tail and two distinct 20-update cel clocks |
| SS05 | `StoryScreen.cpp:177–186,236–242,307–311` | Cyrax talkshow lookup, ordinary fallback, omitted boss portrait and changed body font/origin when available |
| SS06 | `StoryScreen.cpp:258–304` | Press sound, Menu music-stop intention and earned-only wrapping navigation |

Source navigation visibility counts only ordinary seen-mask bits, ignoring the
separate Cyrax flag. Thus one ordinary story plus Cyrax hides source buttons
despite two eligible pages. The project deliberately uses
`StoryState::unlocked_count() > 1` for visibility and `StoryBrowser::step` for
navigation; it never weakens earned-story semantics or changes unlock bits.

Before implementation, these local design choices are recorded:

- Presentation has no file access, rendering, audio playback, wall clock or
  serialized fields. Compose is immutable. The caller supplies identified
  assets, resource-animation results and font metrics.
- `FocusedOnly` holds portrait animation during focus loss. `WidgetUpdates`
  exposes the source's distinct widget-counter policy. Both require explicit
  caller update counts; bubble advance intentions require focus in either mode.
- Ordinary background uses the source fill and bubble configuration. Talkshow
  availability is checked each composition. Missing talkshow selects the ordinary
  scene and requests the actual boss portrait; missing/invalid portraits remain
  explicit asset issues, never substituted art or fabricated animation.
- Source body wrapping has no height limit. Project safety/accessibility bounds
  it above the bottom chrome, adds line scrolling and preserves UTF-8 boundaries.
  Work/allocation caps are project policy, not recovered retail behavior. No lore
  is authored as a substitute for an unavailable catalog.

## Integration

The controller's API is deliberately independent of the frontend:

| API | Caller responsibility / result |
|---|---|
| `StoryScreenController::new(StoryBrowser)` | Supply the already selected current story. Use `StoryState::open_browser` for selector eligibility; the Challenge award is already authoritative |
| `sync_browser(StoryBrowser)` | Follow caller-owned page changes; reset scroll/pressed state on change while preserving the portrait counter |
| `advance(updates, focused, StoryClockPolicy)` | Supply actual UI update counts; receive portrait/bubble update counts and redraw intention. Call with zero updates on focus loss to cancel an outstanding press |
| `view(&StoryState, Option<&StoryEntry>, &impl StoryAssets, &impl StoryTextMetrics)` | Supply immutable earned state and the matching `catalog.entry(controller.current())`; receive a pure composition or explicit font/layout error |
| `input(StoryInput, &StoryState, &StoryScreenView)` | Feed only focused, active input and a current view; receive sound, music, changed-page/return and scroll intentions |

`StoryAssets::image_metrics` reports available, decoded cel geometry.
`animation_cel` must implement the actual resource-animation lookup at the
provided update count. A verified resource without animation metadata returns
zero, matching `Image::GetAnimCel`; unavailable/unimplemented metadata returns
`None`. Do not manufacture a loop from the number of sheet columns. Bilaterus
uses its explicitly specified cels and does not require resource animation.

Every portrait manifest ID is exposed by `StoryPortrait::resource_id`, in all 33
Story ID order. The exceptional lookup is `StoryAsset::TalkShow.resource_key()`
→ `images/talkshow`, an install-relative image stem. When available, that image
draws at natural size at (0,0) and the boss portrait is omitted. When unavailable,
`asset_issues` records it and ordinary composition requests `IMAGE_BOSS`.
Missing/invalid portraits and unavailable/out-of-range animation cells are
reported and omitted; there is no art substitution. This implementation does
not itself verify installed asset existence or animation metadata.

`StoryTextMetrics` supplies font height, ascent, line advance and measured text
width. Missing fonts, nonfinite/negative measurements, unusable line metrics and
a body line taller than the text clip return `StoryLayoutError`; the caller can
show a diagnostic. Required font IDs are exposed by `StoryFont::resource_id`.
On the inspected `d782184` frontend, JungleFever10/15/17 outline and
ContinuumBold12/14 are already loaded. Liddie15, Liddie18 and
ContinuumBold12Outline still need actual owned-font loading and adapters;
`Presentation::image_cells` also lacks resource-animation metadata. These are
primary-owned joins, not permission to guess font or animation behavior.

Render in `STORY_PAINT_ORDER`: background/bubbles, heading, title, ordered
portrait parts, body, black top/bottom chrome, page counter, then hyperlinks.
Honor every text/sprite `clip`, natural cel dimensions and `mirror_x`.
Hyperlinks use the supplied font, no underline, grey normal/white hover colors,
measured text width plus 20 and the returned hit bounds. Paint their label
centered within that bound using the caller's actual font ascent. Do not reuse
the old bottom-row generic buttons or old scaled first portrait cel.

The existing `BubbleManager` can consume the ordinary bounds/configuration via
`set_bubble_bounds` and `set_bubble_config(10, 3)`. Initialize/prewarm once with
`update_a_lot`, then advance only by `StoryUpdate::bubble_updates`, including
when talkshow temporarily hides the bubbles. Use a transient presentation RNG
and preserve the caller's existing frozen-Board ownership. A `prewarm_once`
descriptor is an initialization intention, not work to perform every render.

Pointer Down emits `SOUND_BUTTONCLICK` only for a visible eligible control.
Pointer Up activates only the pressed control under the pointer; drag-away,
cancellation, focus loss and stale-page views cannot commit it. Keyboard
`Activate` combines those intentions as project accessibility policy.
`StoryAction::Changed` identifies both pages and the direction using the
existing browser's eligibility/wrap rules. Challenge callers retain their
existing action/phase authority and synchronize the resulting page; selector
callers retain their own transient browser. Do not step a durable transaction
twice. `ReturnToCaller` plus `StopStoryMusic` requests exit; the caller decides
the actual selector/interlude destination and music synchronization.

Body text wraps by supplied widths inside `(50,280,540,160)` ordinarily or
`(50,330,540,110)` for talkshow. Line scrolling clamps to `max_scroll`; repeated
events accumulate, and navigation resets scroll. Explicit newlines/blank lines
are preserved; other whitespace is normalized between words. Oversized tokens
split at Unicode character boundaries, not grapheme-cluster boundaries. A glyph
wider than the region remains clipped and flagged. Title input is capped at 256
bytes, body input at 8192 bytes and wrapping at 512 lines, always at UTF-8 boundaries.
`clipped`, `text_truncated` and `horizontal_overflow` expose these limits rather
than silently claiming complete content. The recognized catalog's own 80/1024
byte limits are unchanged. Missing catalog entries yield absent title/body and
`catalog_unavailable`; the caller supplies diagnostic UI, never replacement lore.

## Focused verification

Observed in this worktree on 2026-10-10: clean base
`d7821846e3dc2d6d2c32157eaf326d109195720f` plus only new maintained
`src/story_screen.rs` and this document. The private harness imports the actual
maintained `story_screen.rs`, `story.rs` and `install.rs` by path, uses existing
pinned serde 1.0.229/sha2 0.11.0 dependencies, and keeps all build output inside
`.scratch/story-screen/`. Root Cargo files and library registration are untouched.

Rust `1.95.0 (59807616e 2026-04-14)`. The task-specific PowerShell script passed
parser and automatic-variable validation through the host validator. Commands:

```text
rustfmt --edition 2024 src/story_screen.rs
rustfmt --edition 2024 --check src/story_screen.rs
cargo generate-lockfile --offline --manifest-path .scratch/story-screen/harness/Cargo.toml
cargo clippy --offline --locked --manifest-path .scratch/story-screen/harness/Cargo.toml --target-dir .scratch/story-screen/target --all-targets -- -D warnings
cargo test --offline --locked --manifest-path .scratch/story-screen/harness/Cargo.toml --target-dir .scratch/story-screen/target --lib story_screen::tests::
cargo build --offline --locked --manifest-path .scratch/story-screen/harness/Cargo.toml --target-dir .scratch/story-screen/target
```

Formatting, strict lint and harness build passed. **Seven focused tests passed**;
four unrelated imported tests remained filtered. Cases cover all33 portrait
slots/Rufus row, Bilaterus draw order and clock boundaries, both Cyrax scenes and
explicit asset failures, pure repeated composition/focus policies, immutable
earned-only input/wrap/Menu intentions, bounded Unicode/newline/scroll behavior,
and invalid animation/font reports. Expected presentation facts come from
SS01–06; metric fixtures and safety/accessibility expectations are explicit
project contracts, not retail observations. No failed implementation check occurred.

The validated `scripts/Test-Workflow.ps1` passed: 35 required files, 74 documents,
842 local links, 231 heading anchors, 45 allowlist checks, 37 excluded-path checks
and 148 eligible files. Explicit ignore readback confirms that harness source,
its Cargo manifest and test executable stay private. This is workflow/path
verification, not runtime or content-origin evidence.

Harness lock SHA-256:
`44EFAC83E261F97B976AEE46E64CE186F85E31D1D13B718C74977C9028B69C51`.
Executed harness test EXE SHA-256:
`3FBFBCD6D25108F86945CB45A1C939070C97F7606ACFB5395E0A16423F740D7A`.
Unchanged production input SHA-256: `story.rs`
`8871B694C2F03CD6294AF682D92CC5F0646BEC2B698C2F64C24B7AAF85439C18`,
`install.rs` `9ECFBF48E32C78D89070238C901A56CB4BAAF93F71803106E2AA189202820227`.
No full runtime suite or gameplay process was run; unit checks have no gameplay
speed/mute flags. Private harness/scripts/artifacts remain ignored for local
reproduction. No shared service, port, database, game save or installation was
mutated; no cleanup of those resources is needed. Actionables was not updated:
no `workItemId` supplied.

Normal-speed native input, visual, audio, retail comparison and human judgment
remain **not tested**. A module/harness check is not a playability claim.
