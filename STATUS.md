# Current handoff

Updated: 2026-10-08, America/Phoenix. **Active goal: implement and verify the complete runtime.** Setup is complete. A first-tank Rust implementation exists and builds, with runtime execution validation in progress. The installed binaries are authoritative; WinFish is a secondary starting point. Existing work is preserved as binary validation is added. Local commits are authorized.

## Goal and scope

Implement and verify the full [runtime requirements](docs/requirements.md). Begin with the original guppy/food/currency contract, then implement its real interaction loop. After each validated milestone, choose and implement the next automatically. Milestones do not end the task; continue until the full requirements are implemented and verified or a concrete blocker genuinely requires user input. No playable build is currently claimed.

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
| File extensions | 420 GIF, 93 TXT, 50 OGG, 38 JPG, 15 AU, 14 LUC, 11 PNG, 4 XML, 3 EXE, 3 MO3, 3 SIG, 1 DLL, 1 HTML, 1 SCR; decoding not tested |
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
| E9 | Observed | Maintained runtime checker passed formatting, `clippy --locked --all-targets -- -D warnings`,24 tests(15 library,5 assets,4 fonts), and native build | Current source tree; image/mask/font fixes and visible pose/font integration included. No runtime playability evidence yet |
| E10 | Observed | Actual installed files decoded:246 manifest images,65 effects,all15 bitmap font scripts and atlas bounds | Legacy Windows-1252 encoding fix verified after the initial7 failures. No music playback; decoding does not establish matching visuals |

## Setup verification

Passed:

- Required reference revision acquisition and actual guidance/template reading; source paths and notice retained.
- Existing host PowerShell wrapper validated and executed reference-acquisition and environment scripts with parser and `PSAvoidAssignmentToAutomaticVariable` checks passing.
- Local game identity and file inventory recorded read-only; native Rust toolchain probe passed.
- Shared analysis tools were reconciled against the host manifest; fresh version checks and prior verification limits recorded in E8 and the analysis-tools note. Local audit receipt: `.scratch/evidence/setup/analysis-tools.local.json`.
- Earlier detailed runtime prompt recovered from the project conversation and preserved in requirements; superseded setup proposals were not imported.
- Local Git initialized on `main`, with no commits, remote, or staged changes. All 14 maintained setup files are eligible for Git; actual `.scratch/` references, local metadata, and probe binaries/source remain ignored.
- `scripts/Test-Workflow.ps1` passed both its documented PowerShell 7 invocation and the host validator invocation after the tools follow-up. All local inline Markdown links and heading anchors, required files, document whitespace/final newlines, and 49 allowed/excluded path cases passed.
- Startup order and linked records read back; cross-document review confirmed one requirements record, explicit unresolved design choices, unchanged full-runtime scope, correct original/Rust playtest fields, source mapping and MIT notice, no duplicate CONTEXT file, and no dangling runtime/helper commands.

Exact usable commands are in [README](README.md#workflow-checks). Checks were rerun after the final handoff edits. Local execution receipts and maintained-file SHA-256 identities are retained in `.scratch/evidence/setup/workflow-verification.local.json` and `maintained-files.local.json`; findings remain portable in this handoff. No fetched guide or chapter-linked project code was executed. Git ignore checks and the maintained-tree scan do not certify licensing or prevent a deliberate forced addition.

Setup checks above are historical setup results. Current Rust build and regression checks are E9; asset checks are E10. Not tested: original-game run/playtest, installed-binary rule compatibility, runtime window/input/render/audio, save/reload execution, gameplay parity and human feel. Automatic instruction discovery by a newly opened Codex session has not been exercised.

## Attempted approaches and failures

- Initial Git inspection failed because the folder was not a repository. This was an observed starting state, not a runtime defect. Initialize local Git for the setup files; do not invent a previous branch/commit/remote.
- An early secondary-reference read failed with missing IW4L file paths. Reconciled retrieval using its pinned revision; the required files were then retrieved and read successfully. No inaccessible required setup material remains.
- The optional global `Get-RepoSnapshot.ps1` helper failed with Git exit 128 and `ambiguous argument 'HEAD'` on this repository's unborn branch. Its commit-oriented snapshot is unavailable until a commit exists. The documented status command and maintained checker work without HEAD and were verified; do not create a commit or modify the global helper merely to satisfy this optional inspection.
- Runtime failures: initial sha2 0.11 digest hex formatting did not implement LowerHex; fixed explicit byte formatting. Initial macroquad window config used the wrapper Conf instead of miniquad Conf; corrected against the installed API. Duplicate coin-click test omitted the second click's possible food charge; caller review corrected its expectation while retaining exactly-once credit.
- Actual asset scan initially rejected IMAGE_EDITBOX's mismatched automatic mask. Reassessed original files and secondary loader behavior: automatic mismatch leaves color unchanged, explicit mask mismatch remains an error. Corrected and all246 manifest images decode. Seven font scripts rejected UTF-8; strict Windows-1252 decoding then passed all15 with a synthetic non-ASCII regression.

## Gaps and blockers

No missing information materially blocks implementation. Primary binary recovery is now active; secondary-source findings are retained as hypotheses to confirm/correct. Remaining work includes runtime execution/input/visual/audio checks, legacy font encoding, exact movement/order/RNG compatibility, first-level tutorial/game-over, victory/advance/save/reload, all later content and modes, tracker music, and human playtests. Full scope remains in [requirements](docs/requirements.md).

The template's default results are not evidence. No playable, faithful, source-compatible, or clean-room implementation is claimed. Distribution and the original runtime-code license are undecided.

## Next action

Execute the [original-game contract task and acceptance criteria](docs/DESIGN.md#next-task-original-game-contract), implement and validate M1, then continue automatically through M2 and the complete runtime scope. Inspect and pin WinFish, the framework port and useful mashup design code; resolve study/reuse boundaries and recover independently checkable behavior before implementing each slice. Keep records current. Human evidence that remains pending limits claims but should not halt independent implementation work.
