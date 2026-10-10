# Adventure finale presentation

## Scope and ownership

This increment completes the source-informed narrative, conditional death-count rows, fish/alien/pet cast and credits model. The existing session remains the sole authority for the fallen-pet snapshot, result credit, progression and menu transition. The interlude controller remains transient. App loading, rendering, input, audio and bubble integration belong to the primary chat.

Exclusive maintained files: `src/interlude.rs`, `src/interlude_catalog.rs` and this document. Base: `4c87d559f590975b036e84eb226bc774405238d2`, initially clean, in the delegated worktree. No primary dirty work is copied. No game process, shared analysis database, real save or system configuration is changed.

## Source and implementation decision

**Secondary-source-derived:** registered W1 revision `f919b3c241cfd611c547f1653fb3514f500f761b`, `source/WinFish/InterludeScreen.{cpp,h}`. Relevant symbols: constructor/update/control, `Draw:161–511`, `DrawStringHelper:541–563`, `DrawAngieHalo:565–575` and `DrawPetAngieSpawns:577–613`. Resource symbols come from its registered `Res.cpp`; profile visibility comes from `UserProfile::IsPetUnlocked`. Existing [provenance](provenance.md#runtime-sources) study/license boundaries apply. This is original Rust informed by recovered behavior, not clean-room work or wholesale source translation.

Before implementation: retain existing constructor/events/control methods and add a separate typed presentation API. The legacy renderer must continue compiling. Model source row spacing, columns, fonts, colors, cast sprites and concealment explicitly, without another progression authority. Load text once from the recognized owned executable using bounded read-only I/O, exact length/digest and sanitized numeric string locations. Never bundle narration, credits or copied C++. Unknown/missing/malformed catalogs report unavailable; no invented narrative substitutes.

Known installed-binary contradictions take precedence. Unchecked W1 layout/control remains secondary-source-derived. Focused source-grounded tests and a build do not establish native playability, retail visuals/audio or human judgment.

## Evidence and deliberate adaptations

| ID | Evidence and implemented contract | Scope |
|---|---|---|
| F1 | Owned English Steam v1.1 x86 executable, 3,787,832 bytes, SHA-256 `F54C2C6EE54B00AE6DA7F4BDE15FEB90D0A867ABA6B4151B0277C1F81DA3FF66`. Read-only source-string probe matched all 116 W1 text calls to exact NUL-terminated byte slices. `src/interlude_catalog.rs` records only source lines, file offsets and lengths. | **Observed static data**, not executed retail Draw behavior. The catalog is identity-specific and contains no bundled narration/credits. |
| F2 | W1 Draw:175–321: start540, spacing35, ten common introductory rows; death-count branches0,1,2–3,4–5,6–10 and no extra survivor rows for11+. The cursor resets before common closing rows. | **Secondary-source-derived**; catalog formatting substitutes the snapshot count only in source line300. |
| F3 | W1 Draw:322–455: seven fish, eight aliens, twenty-four pets. Caption columns use helper sums650/395/885; heading/body use Liddie18/15 and source colors. Sprites include row4 carnivore, row6 breeder, slow Ulysses, eight Bilaterus parts, Amp charge, Angie halo and bonus eggs. | **Secondary-source-derived**; exact native paint/input remains pending. |
| F4 | W1 Draw:461–510:38 credit/branding rows, including14 heading-font rows. Cast/credits position is independent of death count and unlock visibility. Bubble-fish admission cursor5390, first credit5635, final baseline7910, final scroll7645. | **Secondary-source-derived**; the bubble owner consumes the exposed cursor. |
| F5 | Constructor/update/exit and existing Tank5 snapshot contracts are retained. Heal cues use the existing15 slots and first-under-four shift; focus freezes scroll/pulse; music starts once and crosses at the update after scroll3000; menu exit is once-only. | Existing control contract, **secondary-source-derived** with focused regression evidence. No new reward, save field or progression authority. |

W1 line436 queries Brinkley when drawing Stanley. This increment deliberately queries each bonus pet's own existing unlock authority: an unearned Stanley stays concealed even after Brinkley is earned. This prevents leaking unearned identity. The installed Stanley predicate is **unknown**; retail comparison must adjudicate this source discrepancy. All other documented layout branches follow the pinned secondary source.

The model includes source halo/charge layering and return sprites but does not own a BubbleMgr or fish-effect state. Its halo pulse remains transient and updates only with focused controller calls, including after scrolling stops. Existing `fallen()` and `resurrection_poses()` semantics remain intact; fifteen displayed return slots do not change the retained full fallen-kind snapshot.

## Public API and integration handoff

`turbofish_deluxe::interlude::catalog` is a nested public module declared with a path attribute; no `src/lib.rs` edit is needed. `FinaleCatalog::load(&Path) -> io::Result<FinaleCatalog>` reads a regular recognized file once, at most3,787,833 bytes, and checks actual length/SHA-256 before decoding. Missing/type/identity/slice/template failures clearly report `Finale catalog unavailable`. `text(TextId)` borrows catalog data; `TextId::source_line()` exposes its sanitized source crosswalk.

1. At app startup, load `FinaleCatalog` from `game_root.join(&identity.executable)` beside the existing Story catalog. Retain an explicit catalog-unavailable message on error; never replace missing text with invented prose.
2. Continue constructing `InterludeController::new(&session.finale_fallen_pets)` only on the existing finale transition. Keep current result and menu transactions. No catalog load or presentation call may award/unlock anything.
3. Draw `interlude.presentation(|pet| session.progress.has_pet(pet))`. Every returned row/sprite already includes current scroll. Resolve each `FinaleText` through `row.text.resolve(&catalog)`; measure the selected owned font and use `row.column.text_x(measured_width)` for source integer placement. `FinaleFont::resource_key()` and `rgba()` select fonts/colors. Never format `CastMember` as the visible caption, especially for concealed pets.
4. Render typed `FinaleSprite` descriptors through `FinaleImage::resource_key()`. `WholeImage` is a concealed egg; `Cell { column, row }` indexes the owned sheet. Preserve sprite order, mirror, tint/alpha and normal/additive blend. The `sprites` vector already includes return pets and Angie passes; do not render `resurrection_poses()` a second time.
5. Apply background `[25,105,195,255]` and clip `[0,40,640,400]` to rows/sprites, including partial glyphs. Compose rear bubble/fish effects below content, foreground bubbles above content, then source top/bottom40px black masks and the app's exit control. Bubble integration uses `bubble_fish_start_scroll=5390`; W1 manager1/2 both have bounds `(0,-20,640,520)`, config `(10,3)` and prewarming; manager2 uses upward fish velocity−1/unclipped fish, fish config `(0,0)` before5390 and `(3,3)` afterward. At final scroll, its fish bounds switch from `(0,200,640,240)`/velocity−1 to `(0,30,640,420)`/velocity0. These are handoff requirements, not bubble implementation here.

`visible_rows()` remains source-compatible with existing exhaustive `RowContent` matches, but its catalog-free legacy renderer now shows an explicit unavailable notice. Full content requires the new API and primary app wiring. Existing constructor/events/music/exit methods remain compatible. Primary owns source loading/rendering/input/audio joins and central STATUS/MODLOG/DESIGN/provenance/compatibility reconciliation.

## Focused checks and failures

Build state is the clean base above plus only the three owned maintained files. Rust/Cargo1.95.0 MSVC, debug, locked/offline dependencies, isolated `.scratch/finale/target`; native import/runtime files are read from the existing registered libopenmpt0.8.9 cache. Tests use fixed controller updates and snapshot fixtures, without a native game process, real saves or gameplay-speed flags.

One numeric-probe attempt incorrectly required a zero byte immediately before every literal and failed at source line186. Inspection found nonzero preceding binary bytes (`84 40`), not missing catalog text. Exact bounded slice, terminator and whole-file digest checks are sufficient; no preceding-padding assumption remains. Raw probe/receipts stay in ignored `.scratch/finale`. No broad recovery or shared database access occurred.

The first opt-in catalog command used `--exact` with an unqualified name and ran zero cases; it supplies no owned-catalog evidence. Correcting only that filter ran the intended one case successfully, without repeating passing suites.

Observed checks on base `4c87d55` plus owned runtime edits:

- `rustfmt --edition 2024 src/interlude.rs src/interlude_catalog.rs`, then `cargo fmt --all -- --check`: passed.
- `cargo clippy --offline --locked --workspace --all-targets -- -D warnings`: passed.
- `cargo test --offline --locked -p turbofish-deluxe --lib interlude::`: seven passed, one opt-in case ignored. Covers all five conditional branches and the11+ path, fixed common cursor,39-member cast,38 credits, source fonts/columns/frames/segments, individual bonus concealment, snapshot order, focus/pulse freeze, heal/music/exit once-only behavior and malformed/unknown catalog data.
- `cargo test --offline --locked -p turbofish-deluxe --lib adventure::tests::first_finale_completion_awards_once_and_hatch_enters_interlude -- --exact`: one existing result/snapshot/once-credit regression passed.
- With `OWNED_GAME_DIR` set privately, `cargo test --offline --locked -p turbofish-deluxe --lib interlude::catalog::tests::owned_finale_catalog_covers_all_source_rows_without_writing_install -- --ignored --exact`: one passed. All116 slices decode; each death-count branch resolves; executable bytes stayed identical before/after.
- `cargo build --offline --locked --workspace`: passed. Debug EXE SHA-256 `B4322624E0ECC8A565C40905BE1C2FCEE4CB3CD14756FCCF4F405B8FAA773BAD`.
- Validated `scripts/Test-Workflow.ps1`: passed32 required files,73 documents,832 links,224 anchors,42 allowed/35 excluded paths; `git diff --check` passed. This is workflow evidence only.

Runtime input SHA-256: `src/interlude.rs`=`E3F7D32FC92660DF2490B5ECC51B59AC8F14CE8326FBAE45AFCC7E51DE752AE1`; `src/interlude_catalog.rs`=`BB0028E3DD52CD92A6AF830EB164CF3EF649BB99E3FFB532492B13B494CC0BFE`; unchanged `Cargo.lock`=`FF19F66FF0205FD6880C793EE082E26D94397B6EF4CE62DCA80FFB485ACEC5A1`. W1 Draw source SHA-256=`4519D4669918E4E26C22D7A4EB7B06FA3D972580846CC98ADC3E3E6F00913291`. Private receipts: `.scratch/finale/checks-20261010T213226026Z/results.local.json` and corrected owned case `.scratch/finale/checks-20261010T213451059Z/results.local.json`; all PowerShell mutation/check scripts passed parser and automatic-variable validation through the host wrapper.

Nine distinct focused cases passed; no full gameplay suite or native game launched. Native input/render/audio, exact retail Stanley visibility/fades, source button-font geometry and human judgment remain deferred. No playability claim is made. Local commit/push reconciliation is reported in the final handoff; central records remain the primary chat's responsibility. Actionables was not updated because no workItemId was supplied.

Cleanup: no service/port/database/game-process allocation, no real save or install mutation. Test binaries exited; ignored target/logs/numeric probe remain in the owned worktree for reproducibility.

## Main reconciliation

Feature commit `a76fbd6a179d8335d4fbd2e6de85db4f6945ccf8` contains only the three reserved files. Merge `56d3760e8dcdc53cac0c0f762f97870d2052dec7` cleanly incorporates fetched main `c2b4bfb61122ad51cea88dbba3ee10e5e9b3bbf8`, including the other chats' screensaver host/bank, bubble and title work. No primary checkout or unrelated source was edited. The two finale runtime hashes above remain identical; their focused evidence remains applicable. The initial whole-build identity is historical after this merge.

Post-merge `cargo fmt --all -- --check`, strict locked/offline workspace/all-target lint and locked/offline workspace build all passed; only affected compilation/build checks reran. Receipt `.scratch/finale/checks-20261010T213605036Z/results.local.json`. Merged `Cargo.lock` SHA-256 `1643FD7BE00AF38DB3DA764BE8E0C03CFC427EE368DCAAB7EF17A49B187F153F`; debug EXE SHA-256 `39F0D0B81D5ADCF7C319B6DE696DD27A2B8BE23609E7985314D19AC52EFB3C35`. The primary's app still calls the catalog-free compatibility method; complete rendered finale content requires the integration steps above.
