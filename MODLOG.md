# Change and verification log

Newest entries first. Update triggers and evidence rules are authoritative in [AGENTS](AGENTS.md). Each coherent entry records changed files, reason, actual checks/results, failures or not tested work, and next action. Completed entries are historical; add later corrections rather than hiding failed approaches.

## 2026-10-08 — Build first-tank runtime and establish primary binary validation

**Changed:** Added single Rust crate/lockfile, owned-install discovery/hash inventory, assets/font adapters, first-tank simulation, macroquad window/input/effects, event/state/capture instrumentation, separate JSON project saves and maintained validation command. Retained all previous work. The user clarified installed binaries as the game-detail authority and authorized incremental commits; WinFish expectations remain secondary until checked.

**Tested:** Validated maintained runtime script passed formatting, warnings-as-errors Clippy,24 all-target regression tests and native build. Actual owned-install scan decoded246 image entries,65 effects and all15 bitmap fonts/atlases after corrections.

**Failures/corrections:** sha2 digest formatting and macroquad Conf type were corrected from compile errors. Caller review corrected player-food delay, purchase locks, food→fish→coin order, death-tick coin production and prior-integer-Y coin credit. Duplicate click can purchase food after coin hit-testing stops; test checks exactly-once credit with that charge. IMAGE_EDITBOX's mismatched automatic mask is ignored, consistent with the files and secondary loader; declared-mask mismatch remains rejected. Seven fonts rejected UTF-8; strict Windows-1252 decoding preserves their byte glyphs, and all15 now pass. Local Clippy findings were corrected or narrowly justified for the thin sprite-draw helper.

**Limits:** No original or Rust window/input/playtest execution yet; secondary expectations do not prove retail fidelity. Primary binary static analysis is active in ignored private storage. Movement, tutorial/game-over, audio selection, full progression/persistence and later content remain incomplete. No game installation writes or asset bundling.

**Next:** Correct font encoding, exercise the real normal-speed click/feeding/growth/currency path, compare authoritative binary evidence, commit verified changes and continue automatically.

## 2026-10-08 — Activate complete runtime implementation

**Changed:** Updated scope records to the user's full-runtime goal with automatic milestone continuation. Retrieved clean pinned WinFish, framework-port, PopLib and mashup references; added the first source-derived behavior ledger and single Rust crate/dependency manifest.

**Why:** The user explicitly superseded the setup-only checkpoint. Recover the real growth/feeding prerequisite to earning the first coin, preserving the full requested scope.

**Tested how:** Independent read-only behavior and asset/platform investigations against identified checkouts and owned installation; official Rust library API inspection. Reference acquisition scripts passed PowerShell validation.

**Result:** First-tank functional rules and asset boundaries are source-supported. Asset metadata inspection identified sprite sheets, blue-channel companion alpha, non-strict resource XML and μ-law AU effects. Source/runtime identities and limits are in provenance and the behavior ledger.

**Not tested:** Runtime build, source/Steam compatibility, original gameplay, window/input/render/audio, save behavior and human playtests. No assets or game source files incorporated; original install unchanged. Dependency resolution is next.

**Next:** Implement and validate the first real feeding/growth/currency loop, then continue to the first Adventure victory and remaining runtime scope automatically.

## 2026-10-08 — Record available shared analysis tools

**Changed:** Added [analysis-tools guidance](docs/analysis-tools.md); linked it from AGENTS, README, DESIGN, STATUS and provenance; included it in the workflow checker. No runtime implementation started.

**Why:** Preserve the user-supplied tool availability and the actual invocation/verification limits for the next recovery session.

**Tested how:** Read the shared tool README, manifest, verification record and runner; reconciled all 10 entry-point hashes; ran Frida/TShark/Python version commands through a validated scratch script. Re-ran the documented workflow checks on the updated files.

**Result:** Tool hashes matched and version checks passed. Updated workflow checks passed with 14 maintained files, 11 Markdown documents, all checked links/anchors and 49 included/excluded path cases. Exact tool evidence and its historical/fresh boundaries are in [STATUS](STATUS.md#evidence-ledger) and the analysis-tools note.

**Failures / not tested:** No game import, attach, memory access, tracing, GUI launch or packet capture. Ghidra startup and Cheat Engine build/scanning limits are recorded from the earlier host verification rather than claimed as fresh project tests. No tools installed or global configuration changed. Actionables remains unchanged without a supplied work item scope.

**Next:** Stop at setup; use the existing tools where needed in the next original-game contract task. Ghidra databases belong under ignored `private/ghidra/`, outside dot-prefixed path segments.

## 2026-10-08 — Verify setup checkpoint

**Changed:** Finalized the handoff and runnable PowerShell commands after installing the workflow. Added no runtime scaffold. The local repository uses `main`; no remote, staging, commit, or push was created.

**Why:** Make the setup concrete and reviewable before the next runtime task.

**Tested how:** Executed the maintained workflow checker through the documented PowerShell 7 command and the existing host parser/automatic-variable validator. Checked Git's actual candidate paths, local links/heading anchors, document whitespace, 21 included-path cases and 27 excluded-path cases; read back the startup order, requirements, design, playtest fields, mapping and notices. Repeated affected checks on the final documents.

**Result:** Passed: 13 required/eligible files, 10 Markdown documents, all checked links/anchors, all 48 path cases, and PowerShell validation. Proprietary-data paths, reference checkouts and source-shaped private probes remain ignored. Environment/build identities and local evidence locations are in [STATUS](STATUS.md#setup-verification).

**Failures / not tested:** The maintained setup checks passed. Optional global `Get-RepoSnapshot.ps1` inspection failed on the unborn branch with Git exit 128 / `ambiguous argument 'HEAD'`; the documented direct Git and workflow commands succeeded, so no commit or global-tool change was needed. Original-game execution, Rust gameplay, GUI/audio/input, saves, fidelity, fresh-session Codex instruction discovery, and human playtesting remain not tested. Original install was only read; no assets or decompiled source were incorporated. Tracking was not updated because no Actionable scope was supplied.

**Next:** Stop here. The next task is the original-game behavior contract and its acceptance criteria in [DESIGN](docs/DESIGN.md#next-task-original-game-contract), followed by the first real Rust tank loop.

## 2026-10-08 — Install guide-based agent workflow

**Changed:** Added AGENTS, STATUS, README, preserved runtime requirements, initial DESIGN, playtest template, provenance, setup mapping, third-party notice, Git allowlist/line-ending rules, and the PowerShell workflow checker. Source-to-file mapping is in [workflow setup](docs/workflow-setup.md).

**Why:** Establish a durable, verifiable workflow before implementing this standalone Rust runtime. Start from the guide templates while preserving the user's actual requirements and inherited Windows conventions.

**Tested how:** Read actual pinned G1/S1 source material; inspected the empty project and applicable instructions; recovered the earlier runtime prompt; inspected Steam metadata/executable identity/inventory; validated and executed isolated PowerShell scripts; compiled/linked/ran a scratch-only Rust probe. Reference identities are in [provenance](docs/provenance.md); environment and evidence are in [STATUS](STATUS.md).

**Result:** Required reference material retrieved and read; full requirements retained; game installation identified without running or modifying it; native toolchain probe passed. Final workflow/link/ignore checks are pending, so setup is not yet marked verified.

**Failures / not tested:** Initial Git command found no repository. An early IW4L read found no local files; retrieval was reconciled and the pinned files were read. Original gameplay, all Rust runtime behavior, and human playtesting are not tested. No runtime implementation started.

**Next:** Complete the documented workflow checks, update their actual results, and stop at this checkpoint. The subsequent concrete recovery task is in [DESIGN](docs/DESIGN.md#next-task-original-game-contract).
