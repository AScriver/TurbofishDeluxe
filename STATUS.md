# Current handoff

Updated: 2026-10-08, America/Phoenix. **Active goal: implement and verify the complete runtime.** Setup is complete. The first Rust feeding/growth/silver loop passed a normal-speed window run; corrected fish facing was visually checked. Full retail fidelity remains unverified. Installed binaries are authoritative; WinFish is secondary. Existing work is retained. Local commits are authorized; workflow checkpoint `b50225b` exists. Tested runtime identity and source hashes are recorded in run03.

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
| E9 | Observed | Maintained runtime checker passed formatting, `clippy --locked --all-targets -- -D warnings`,26 tests(15 library,5 assets,4 fonts,2 persistence), and native build | Identified source tree includes camera/facing and telemetry corrections. Tests do not prove retail fidelity |
| E10 | Observed | Actual installed files decoded:246 manifest images,65 effects,all15 bitmap font scripts and atlas bounds | Legacy Windows-1252 encoding fix verified after the initial7 failures. No music playback; decoding does not establish matching visuals |
| E11 | Binary-derived | Main EXE and identified screensaver imported read-only to private Ghidra databases; startup metadata/Lua host and base-app interval traced | Exact findings/addresses in [behavior contract](docs/behavior-contract.md#primary-binary-findings). Actual Adventure payload association/effective tick still unknown |
| E12 | Observed | [Run03](docs/playtests/2026-10-08-m1-03.md):100.376s/3503 ticks,43 food drops/16 eats,2 Medium growth events,17 natural silver drops,14 credits without duplicate IDs; saved/final balance195, clean exit | Real Rust window input; no forced outcomes. Actual paired frames confirm left/right facing with movement. Original game/audio/human comparison not tested |
| E13 | Observed | Same build survived a2s held snapshot reader: publication deferred tick9, recovered80, clean exit221; old complete snapshot retained during contention | Optional telemetry recovery; separate critical save errors remain visible. Windows sharing and resumed-state correctness fixtures passed |
| E14 | Observed | Run03 before/after install inventory digest exactly `6e03aca251d24dc68b0e72e88e06bd50b40775ebe0a09f182e4c2d35ff32f0ca`; all owned processes exited | Original install read-only; original executable/saves/registry not exercised |

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

Setup checks above are historical. Latest workflow check passed:14 required files,15 Markdown documents,116 local links,25 heading anchors,22 included/27 excluded path cases and32 eligible maintained files; parser/automatic-variable validation passed. Current Rust checks are E9, assets E10, execution E12–E14. Save/reload was exercised in historical run01; run03 verifies current save/clean exit. Not tested: original-game run/playtest, installed Adventure rule compatibility, audible output/music, gameplay parity and human feel. Fresh-session automatic instruction discovery remains untested.

## Attempted approaches and failures

- Initial Git inspection failed because the folder was not a repository. This was an observed starting state, not a runtime defect. Initialize local Git for the setup files; do not invent a previous branch/commit/remote.
- An early secondary-reference read failed with missing IW4L file paths. Reconciled retrieval using its pinned revision; the required files were then retrieved and read successfully. No inaccessible required setup material remains.
- The optional global `Get-RepoSnapshot.ps1` helper failed with Git exit 128 and `ambiguous argument 'HEAD'` on this repository's unborn branch. Its commit-oriented snapshot is unavailable until a commit exists. The documented status command and maintained checker work without HEAD and were verified; do not create a commit or modify the global helper merely to satisfy this optional inspection.
- Runtime failures: initial sha2 0.11 digest hex formatting did not implement LowerHex; fixed explicit byte formatting. Initial macroquad window config used the wrapper Conf instead of miniquad Conf; corrected against the installed API. Duplicate coin-click test omitted the second click's possible food charge; caller review corrected its expectation while retaining exactly-once credit.
- Actual asset scan initially rejected IMAGE_EDITBOX's mismatched automatic mask. Reassessed original files and secondary loader behavior: automatic mismatch leaves color unchanged, explicit mask mismatch remains an error. Corrected and all246 manifest images decode. Seven font scripts rejected UTF-8; strict Windows-1252 decoding then passed all15 with a synthetic non-ASCII regression.
- Window run01 exposed a fully inverted viewport from camera matrix conventions. Corrected the vertical camera scale; run02/03 visibly show the upright world and input alignment. The user's backward-swimming observation led to actual sheet inspection and correction of the mirror predicate; paired run03 frames verify headings.
- Run02 aborted at snapshot replacement with Windows error5. A held-reader experiment reproduced it; optional publication now retains the previous complete JSON and retries without stopping simulation. The same counterexample passed in run03-lock. [Historical failures](docs/playtests/2026-10-08-m1-01.md) remain recorded.
- Primary analysis found a10ms base-app interval and nonstandard Lua chunk header. Neither identifies Adventure cadence; do not replace the28ms hypothesis or call the chunk standard Lua5.1. Wrapper/payload association recovery continues.

## Gaps and blockers

No missing information currently requires user input. Primary payload recovery is active; secondary-source rules remain hypotheses to confirm/correct. Remaining work includes exact movement/order/RNG and retail cadence, complete tutorial/rescue/game-over, victory/hatch/advance/progression persistence, all later content/modes, tracker music, audio/retail comparisons and human playtests. Full scope remains in requirements and coverage; resolved font/window/telemetry issues are no longer blockers.

The template's default results are not evidence. A narrow first Rust interaction path is observed; complete/faithful/source-compatible or clean-room claims are not made. Distribution and the project runtime-code license are undecided.

## Next action

Commit the identified first-loop corrections and reports, then implement [M2 progression/rescue](docs/DESIGN.md#m2-next-integrated-outcome) using exact caller evidence. In parallel, identify the installed wrapper's actual game payload and confirm secondary mechanics against it. Retain current source/tests/saves, update affected evidence and exercise the next normal-speed integrated path. Pending human/retail checks limit claims but do not halt independent work; continue automatically toward the full requirements.
