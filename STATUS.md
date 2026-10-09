# Current handoff

Updated: 2026-10-08, America/Phoenix. **Active goal: implement and verify the complete runtime.** Setup is complete. Normal-speed Rust runs cover feeding/growth/coins, earned1-1/1-2 victory, Stinky, weak combat/upgrades, Niko flight/reload and Game Over/reentry; corrected fish facing was visually checked. Full retail fidelity remains unverified. Installed binaries are authoritative; WinFish is secondary. Existing work is retained. Prior commits: workflow `b50225b`, first loop `5799557`, progression/rescue `162c501`, Stinky/score `64988c6`. E22/E23 identify this frozen1-2 checkpoint;1-3 is the next integration.

## Goal and scope

Implement and verify the full [runtime requirements](docs/requirements.md). After each validated milestone, choose and implement the next automatically. Milestones do not end the task; continue until all requirements are implemented and verified or a concrete blocker requires user input. [Coverage](docs/compatibility.md) separates the working first loop from missing systems. No complete or faithful runtime is claimed.

## Environment and reference build

Observed during this task; this section owns the current environment facts.

| Item | Observed identity / limit |
|---|---|
| Project | Turbofish Deluxe; initial directory was empty. Now a single Rust2024 crate with install discovery/identity, owned-image/effect/font adapters, first-tank simulation, window/input/telemetry and separate project saves |
| OS | Windows, version `10.0.26200.0`; Windows 11 is the product target |
| Shell / Git | PowerShell `7.6.6`; Git `2.54.0.windows.1` |
| Rust | `rustc 1.95.0 (59807616e 2026-04-14)`, Cargo `1.95.0`, active `stable-x86_64-pc-windows-msvc`; manifest requires1.95, direct dependencies and Cargo.lock pinned; exact rustup channel pin pending |
| Native toolchain check | Isolated Rust 2024 compile with warnings denied, native link and execution passed; output `turbofish-toolchain-probe-ok`. No game/runtime code involved |
| Game discovery | Steam library metadata and local install found; App ID `3320`, build ID `250752`, English. Absolute installation path is kept in ignored local evidence |
| Main game executable identity | `Insaniquarium.exe` and `InsaniquariumDeluxe.exe`: each 3,787,832 bytes; file version `1, 1, 0, 0`, product version `1.1`; PE machine `0x014C` (x86) |
| Main executable SHA-256 | Both files: `f54c2c6ee54b00ae6da7f4bde15feb90d0a867aba6b4151b0277c1f81da3ff66` |
| Other executable | `WinFish_Scr.exe`: 2,172,232 bytes; version 1.1, x86; SHA-256 `c904b14cfbf985043aca01b7d77bc06d0fd24101800e49fa6fc374ce5b92b878`. Not launched |
| Asset inventory | 657 files,19,098,646bytes. Actual Rust adapter decoded246 manifest image entries and65 effects; all15 font scripts and their atlases parsed after the Windows-1252 fix. No game assets copied into maintained files |
| File extensions | 420 GIF,93 TXT,50 OGG,38 JPG,15 AU,14 LUC,11 PNG,4 XML,3 EXE,3 MO3,3 SIG,1 DLL,1 HTML,1 SCR; declared images/effects/fonts decoded; MO3 music unimplemented |
| Source identity | Pinned G1 guide checkout and S1 chapter-linked documents, with exact revisions/paths/licenses in [provenance](docs/provenance.md) |
| Shared analysis tools | User-supplied host catalog read and reconciled; available tools, versions, invocation, and fresh versus historical verification are authoritative in [analysis tools](docs/analysis-tools.md) |
| Tracking | Actionables owned-task query returned no tasks, `hasMore: false`; no workItemId supplied, no task claimed or updated |

Machine-local inspection records are `.scratch/evidence/setup/environment.local.json` and `game-inventory.local.json`; raw reference files are under `.scratch/references/`. These do not travel with Git. This handoff retains useful sanitized findings. Re-discover local paths on another machine; do not require the original user's path.

## Evidence ledger

| ID | Class | Finding and evidence | Scope / limit |
|---|---|---|---|
| E1 | Observed | Initial directory listing was empty; Git reported `fatal: not a git repository` | Historical pre-setup state; there was no existing work to merge |
| E2 | Observed | G1 cloned and its HEAD, origin, and clean status checked; all required templates/guides read. S1 pinned text retrieval receipts recorded | Documentation reference, not evidence that either example's gameplay works |
| E3 | Observed | Steam metadata plus file versions, PE headers, hashes, and bounded inventory inspected | Metadata only; WinFish compatibility, actual launcher behavior, DRM behavior, and all gameplay unknown |
| E4 | Observed | Validated environment script compiled, linked, and executed the isolated Rust probe successfully | Toolchain availability only; no Cargo project, runtime build, or graphics/audio check |
| E5 | Observed | Workflow checker passed after the tools follow-up: 14 required files, 11 Markdown documents, local links/anchors, 22 allowed paths and 27 excluded paths; PowerShell parser and automatic-variable checks passed | Workflow integrity only; quiet ignore checks include synthetic future paths and actual reference/probe paths |
| E6 | Source-derived | G1 recommends standalone asset-reading rewrites, durable handoffs and iteration; S1 demonstrates explicit provenance/evidence limits | Adapted procedures in AGENTS/setup note; not Insaniquarium behavior evidence |
| E7 | Secondary-source-derived | First-tank functional expectations recovered and caller-reviewed against W1; stable claims and corrections in [behavior contract](docs/behavior-contract.md) | Installed binary is the authority. Steam compatibility remains unverified; tests check these secondary expectations, not retail fidelity |
| E8 | Observed | Shared tool README/manifest/verification/runner read; all 10 manifest entry points exist and their hashes match; fresh Frida/TShark/Python version commands passed via the validated task script | Availability and CLI versions only; other catalog smoke checks are historical, and no game attach/import/GUI/capture was performed |
| E9 | Observed | Progression checkpoint passed formatting, `clippy --locked --all-targets -- -D warnings`,35 tests(22 library,5 assets,4 fonts,4 persistence), and native build | Historical support for162c501; current Stinky/score edits need a new gate. Tests do not prove retail fidelity |
| E10 | Observed | Actual installed files decoded:246 manifest images,65 effects,all15 bitmap font scripts and atlas bounds | Legacy Windows-1252 encoding fix verified after the initial7 failures. No music playback; decoding does not establish matching visuals |
| E11 | Binary-derived | Main EXE and identified screensaver imported read-only to private Ghidra databases; startup metadata/Lua host and base-app interval traced | Historical pass; later PB05/PB06 establish embedded-payload association/constructor timer. Effective live cadence remains unmeasured |
| E12 | Observed | [Run03](docs/playtests/2026-10-08-m1-03.md):100.376s/3503 ticks,43 food drops/16 eats,2 Medium growth events,17 natural silver drops,14 credits without duplicate IDs; saved/final balance195, clean exit | Real Rust window input; no forced outcomes. Actual paired frames confirm left/right facing with movement. Original game/audio/human comparison not tested |
| E13 | Observed | Same build survived a2s held snapshot reader: publication deferred tick9, recovered80, clean exit221; old complete snapshot retained during contention | Optional telemetry recovery; separate critical save errors remain visible. Windows sharing and resumed-state correctness fixtures passed |
| E14 | Observed | Run03 before/after install inventory digest exactly `6e03aca251d24dc68b0e72e88e06bd50b40775ebe0a09f182e4c2d35ff32f0ca`; all owned processes exited | Original install read-only; original executable/saves/registry not exercised |
| E15 | Binary-derived | Reproduced wrapper extraction identifies a distinct embedded WinFish.exe game payload; private import succeeded | Exact hash/range/launcher path in [PB05](docs/behavior-contract.md#primary-binary-findings). Game rules/tick recovery continues; no execution/signature changes |
| E16 | Observed | [M2 window report](docs/playtests/2026-10-08-m2-01.md): clean158.713s starting-roster run earned three eggs and hatched Stinky; close/reload retained hatch and fresh1-2 | Earlier identified M2 build; no forced outcome. First-stage score/settlement/visual details and full next-stage systems remain incomplete |
| E17 | Observed | Corrected build passed held hatch shortcut, next-update rescue autosave and accepted food click+close without extra tick; all final/save/snapshot states agree | Executable64ca99c4…4475 with source hashes; installation digest unchanged/all owned processes exited. Original/audio/human comparison not tested |
| E18 | Binary-derived | PB09–PB12 establish ordinary bottom thresholds, correct Stinky nearest/contact geometry and claimed-value affordability/raw cash subtraction | Exact installed payload identity/functions in [behavior contract](docs/behavior-contract.md); broader modifiers, source update ordering and live cadence remain unmeasured |
| E19 | Observed | Frozen Stinky/score source passed formatting, strict Clippy,50 tests(35 library,5 assets,4 fonts,6 persistence) and native build | Executable353ab1f5…976a4 and dirty-source hashes in [new report](docs/playtests/2026-10-08-stinky-score-01.md). New unregistered alien module is excluded from this gate |
| E20 | Observed | [Stinky/score report](docs/playtests/2026-10-08-stinky-score-01.md): actual earned1-2 migration, natural silver14 caught once, sampled direction/turn poses, typed state/RNG reload; fresh seed42 earned three eggs and recorded126 board seconds, then reloaded | Four normal-speed owned-window runs; install unchanged/all PIDs exited. Whole-JSON numeric-text mismatch resolved by f32 bit comparison. Terminal pending settlement has fixtures only; original/audio/human checks remain not tested |
| E21 | Binary-derived | PB13–PB17 support shot geometry/cooldown/damage, Alien update association, world-coordinate coin forwarding, raw pearl owner marking and weak/strong constructor stats | [Exact ledger](docs/behavior-contract.md#primary-binary-findings) identifies PB05 functions/tables and limits. Exact actor hit-test override/virtual Move semantics and stage dispatch remain unknown; no live original run |
| E22 | Observed | Coherent Adventure1-2 dirty build based on64988c6 passed formatting, strict Clippy,85 tests(67 library,5 assets,4 fonts,9 persistence) and native build | Executable `e685e998a5a9d6f3a1e2bfac9589825d3795a06270d78eae514d9eaf00a07aed`; source/Cargo/Git receipt `.scratch/runtime/tester/build-identity-m3-stage12-01.local.json`; portable identities in the report |
| E23 | Observed | [Adventure1-2 report](docs/playtests/2026-10-08-adventure-1-2-01.md): three natural weak fights/diamonds, earned quality/quantity/three500 eggs, Niko hatch/pearl, exact f64 flight reload and one250 credit; separate natural starvation/Game Over/reentry | Three owned normal-speed runs exited0/install unchanged; one guppy eaten during second fight, retained. Board freeze and early-click rejection observed. Retail/audio/human comparisons and complete effects remain not tested |

## Setup verification

Passed:

- Required reference revision acquisition and actual guidance/template reading; source paths and notice retained.
- Existing host PowerShell wrapper validated and executed reference-acquisition and environment scripts with parser and `PSAvoidAssignmentToAutomaticVariable` checks passing.
- Local game identity and file inventory recorded read-only; native Rust toolchain probe passed.
- Shared analysis tools were reconciled against the host manifest; fresh version checks and prior verification limits recorded in E8 and the analysis-tools note. Local audit receipt: `.scratch/evidence/setup/analysis-tools.local.json`.
- Earlier detailed runtime prompt recovered from the project conversation and preserved in requirements; superseded setup proposals were not imported.
- Historical setup: local Git initialized on `main`, initially without commits/remote/staging. User subsequently authorized commits; workflow checkpoint `b50225b` contains15 files. Private references, local metadata, binaries and proprietary captures remain ignored.
- `scripts/Test-Workflow.ps1` passed both its documented PowerShell 7 invocation and the host validator invocation after the tools follow-up. All local inline Markdown links and heading anchors, required files, document whitespace/final newlines, and 49 allowed/excluded path cases passed.
- Startup order and linked records read back; cross-document review confirmed one requirements record, explicit unresolved design choices, unchanged full-runtime scope, correct original/Rust playtest fields, source mapping and MIT notice, no duplicate CONTEXT file, and no dangling runtime/helper commands.

Exact usable commands are in [README](README.md#workflow-checks). Checks were rerun after the final handoff edits. Local execution receipts and maintained-file SHA-256 identities are retained in `.scratch/evidence/setup/workflow-verification.local.json` and `maintained-files.local.json`; findings remain portable in this handoff. No fetched guide or chapter-linked project code was executed. Git ignore checks and the maintained-tree scan do not certify licensing or prevent a deliberate forced addition.

Setup checks above are historical. Latest workflow check passed:14 required files,20 Markdown documents,169 local links,29 heading anchors,22 included/27 excluded path cases and41 eligible maintained files; parser/automatic-variable validation passed. Alien/invasion/Niko modules are now registered and included in E22. Current Rust checks are E22, assets E10, execution E12–E14/E16–E17/E20/E23. Later clock corrections supersede only affected E17 clock claims; other historical evidence remains usable. Not tested: original-game run/playtest, full installed Adventure compatibility, audible output/music, gameplay parity and human feel. Fresh-session automatic instruction discovery remains untested.

## Attempted approaches and failures

- Initial Git inspection failed because the folder was not a repository. This was an observed starting state, not a runtime defect. Initialize local Git for the setup files; do not invent a previous branch/commit/remote.
- An early secondary-reference read failed with missing IW4L file paths. Reconciled retrieval using its pinned revision; the required files were then retrieved and read successfully. No inaccessible required setup material remains.
- The optional global `Get-RepoSnapshot.ps1` helper failed with Git exit 128 and `ambiguous argument 'HEAD'` on this repository's unborn branch. Its commit-oriented snapshot is unavailable until a commit exists. The documented status command and maintained checker work without HEAD and were verified; do not create a commit or modify the global helper merely to satisfy this optional inspection.
- Runtime failures: initial sha2 0.11 digest hex formatting did not implement LowerHex; fixed explicit byte formatting. Initial macroquad window config used the wrapper Conf instead of miniquad Conf; corrected against the installed API. Duplicate coin-click test omitted the second click's possible food charge; caller review corrected its expectation while retaining exactly-once credit.
- Actual asset scan initially rejected IMAGE_EDITBOX's mismatched automatic mask. Reassessed original files and secondary loader behavior: automatic mismatch leaves color unchanged, explicit mask mismatch remains an error. Corrected and all246 manifest images decode. Seven font scripts rejected UTF-8; strict Windows-1252 decoding then passed all15 with a synthetic non-ASCII regression.
- Window run01 exposed a fully inverted viewport from camera matrix conventions. Corrected the vertical camera scale; run02/03 visibly show the upright world and input alignment. The user's backward-swimming observation led to actual sheet inspection and correction of the mirror predicate; paired run03 frames verify headings.
- Run02 aborted at snapshot replacement with Windows error5. A held-reader experiment reproduced it; optional publication now retains the previous complete JSON and retries without stopping simulation. The same counterexample passed in run03-lock. [Historical failures](docs/playtests/2026-10-08-m1-01.md) remain recorded.
- Primary analysis found a10ms base-app interval and nonstandard Lua chunk header. Neither identifies Adventure cadence; do not replace the28ms hypothesis or call the chunk standard Lua5.1. Wrapper/payload association recovery continues.
- Targeted bookkeeping review found aggregate affordability omitted while coins were travelling, malformed completed legacy-state acceptance and ambiguous missing new fields. Primary PB12 supported the payment correction; validation and explicit format3 requirements prevent silent modern-state repair. Fifty regressions and identified window runs passed afterward.
- Stinky reload's whole-JSON text check failed on six f32 decimal representations; bit comparison established unchanged typed values while retaining the initial failure. A narrowed Ghidra rerun overwrote one private log; its claimed pass was rerun with a distinct retained log. Renaming old DESIGN headings broke local anchors; restore stable headings and recheck links rather than discarding linked history.

## Gaps and blockers

No missing information currently requires user input. Primary payload is identified; constructor28ms timer, raw first-fish/food/egg paths and PB09–PB17 have static support with limits retained. Ordinary Stinky and first-stage score/persistence are committed Rust checkpoints. Food upgrades, weak invasion, general Game Over/reentry, Niko and format4 pass E22/E23. Broader profile/highscores, full hatch/death effects, primary Oscar/score/settlement confirmation, exact movement/RNG/retail cadence, remaining fish/pets/stages/modes, tracker music and audio/retail/human comparisons remain open. Full scope remains in requirements and coverage.

The template's default results are not evidence. A narrow first Rust interaction path is observed; complete/faithful/source-compatible or clean-room claims are not made. Distribution and the project runtime-code license are undecided.

## Next action

Implement [Adventure1-3](docs/adventure-1-3.md): Oscar hunting/production/survival, primary-confirmed strong Sylvester stats and per-board weapon gates, third-egg score/reward/Itchy. Primary RTTI investigation is confirming Oscar's constructor/hunger/contact fields. Retain unconfirmed values explicitly and require the next normal-speed earned-input run before advancing its claim. Continue remaining coverage automatically; retail/human gaps limit claims without stopping unblocked work.
