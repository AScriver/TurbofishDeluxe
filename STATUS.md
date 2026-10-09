# Current handoff

Updated: 2026-10-09, America/Phoenix. **Active: resumed A32 native acceptance at the user's request.** Continue automatically after verified milestones toward the full [requirements](docs/requirements.md).

## Current increment

[A32: Tank 4-2, live Nimbus, Bilaterus and Ultravore](docs/adventure-4-2.md) is implemented with required current-save format 17. Source checkpoint: `4b2d53fd6c6711ad615d083b11f9d41f745ad22f`. Build/regression checks passed; entry, natural conversions and Bilaterus combat are observed, while full native acceptance remains pending.

[A31: Tank 4-1](docs/adventure-4-1.md) is the latest native-accepted stage, committed `38c65be9ad439a727b2e7e1b1401db928b8e55f6` (E76/E78). Its frozen bundle, reports and genuine earned saves remain preserved.

## Current build verification

**E79 — observed build gate:** A32 gate 04 passed formatting, strict workspace Clippy, locked native build/staging and 407 checks: 335 library, 5 assets, 4 fonts, 60 persistence and 3 binding, including 2 owned-music checks. Current17 continuation, required-field and pause regressions passed. Tank 4-2 native gameplay acceptance remains **not tested**.

| Identity | Recorded value |
|---|---|
| Executable | `target/debug/turbofish-deluxe.exe`; SHA-256 `06CB115C9DC4818B416DD17B166D3B22C2A304DE46111E86AE9080180A39D47A` |
| Gate | `.scratch/runtime/gate-adventure32-current-save-04.log`; SHA-256 `1781C3BE954D454E981C46196CE01042D95F5D8142482F3CC46A2775D7284D1E` |
| Checkpoint | `.scratch/runtime/checkpoint-adventure32-01.local.json`; SHA-256 `4D7B7E8A753499C6AD402B66E06925B3A1994DE8AC8A9682FE9E9C4471B2A904` |
| Toolchain | Rust/Cargo 1.95.0, stable MSVC, Rust 2024; pinned dependencies/Cargo.lock. Exact rustup channel pin pending |

The checkpoint identifies base `38c65be` plus A32 inputs now committed in `4b2d53f`. All 37 build-input hashes, 5 artifact hashes and the checkpoint hash matched when this handoff was shortened. The launcher subsequently changed for evidence tooling; its old hash is historical. Rust sources, the executable and other checkpoint artifacts remain unchanged. The integrated tooling gate passed all 407 checks; [MODLOG](MODLOG.md) owns the new receipts. Commands: [README](README.md#workflow-checks).

Focused runtime validation commands are available in the [README](README.md#focused-runtime-checks). All selectors and filter/input guards passed 17 CLI scenarios; the default full gate reran successfully with all 407 checks including owned music and the unchanged E79 executable digest. See [MODLOG](MODLOG.md#2026-10-09--add-focused-runtime-validation-commands) for the historical tooling receipt. Current native progress is below.

## Agent workflow tooling

**E80 — observed resume gate/freeze:** Current `b3e3d0c` plus the reviewed launcher notice-staging correction passed the full407-check gate `.scratch/runtime/gate-adventure32-resume-02.log`, SHA`736F729B1DE635C6FEC67E1AD27AED89C57BA81EA032F4210A69DF9428C83E34`. Focused commands passed8Bilaterus/7Board/4current17 persistence tests. Repeated preparation retains5DLLs/8notices and the unchanged E79 executable. Frozen bundle is `.scratch/runtime/tester/a32-bundle-02`; maintained identity `.scratch/evidence/a32-build-frozen-02.local.json`, SHA`F0E62CBCC2C9230B7E9BE59CC00752BC245C7FCA22E156EE52581B68B602F381`, records launcher-success-with-unchanged-inputs and passes70artifact/source readbacks. Transfer02 SHA`55647E83F3EEA644C81A4D79322B563D51D0BAC63F662D5398E9A46C32B2422F` retains the36unchanged historical inputs/one changed launcher. Preparation01/partial bundle01 are failed historical attempts; no run used them.

**E81 — observed entry, acceptance underway:** Maintained-helper entry02 copied the genuine source byte-exactly, matched complete `identity.start`, entered4-2 with200cash/SmallBreeder2points/NikoItchyNimbus/egg25000/FixedBilaterus, and persisted current17 with exact final/save session equality. Root inspected selector and tank frames. PID43600 exited0 without force; runtime install receipts report unchanged. Entry01's early identity-read failure and clean PID11788 exit remain retained. Natural earning/combat/Ultra, native reopen/pause and full cleanup/lineage acceptance remain pending.

**E82 — observed partial native progress:** E80 run04 survived six Bilaterus waves, recorded50Nimbus coin/131food conversions, bought actual Oscars and reached a normal Playing checkpoint at Board21508/current17. Root inspected the live Oscar/diamond/shop/pet pixels, independently matched complete start/final/save sessions, confirmed PID43680 absent and the unchanged runtime installation receipt, and passed51artifact checks. Private audit02 SHA`CC965CF499B8EF27C83553E5B3EB4F92FBAD0894C3C3A57580C140033B601AD0`. Its private summary failed on legitimate diagnostic rows; native state recovered and exited0. The exact save SHA`EEC50923AE7D9D9BE6AE1135B4DB808286FE132394B379EC670397A682708818` is the active normal-speed continuation source. Ultra/eggs/Amp/selector/pause remain pending.

Separate accelerated-test work changed maintained source/tooling during run04. Its failed current-source readback remains retained; E80's executable/artifacts are unchanged. All48original source bytes were reconstructed from`5a36243` and matched the frozen receipt in ignored `.scratch/runtime/a32-source-5a36243-01`; reconstruction SHA`9D7A7D0ADB32AD7EAA3D2B9D0FE692165888ED79A26DF82DC39D00094D9F7BBC`. This is historical source verification, not a new build. Continue E80 at1x with identified loaded helpers; do not mix the new timing build into its acceptance.

**E83 — observed Ultra, first paid egg and full current17 continuation:** [A32 report](docs/playtests/2026-10-09-adventure-4-2-native-a32-01.md) owns the natural Ultra/Oscar/2000treasure, real predation/hunger losses, first25000egg and exact game-written checkpoint findings. A guarded live-save copy retained Ultra/Oscar/activeBilaterus at Board52555, SHA`0E6395FC1E70C4E9C649FA7AC622076E3330D8A4467E6CCC533052439A08B2F6`. Reopen matched the complete session; pause held all tested live actors/group children/countdown/economy/RNG/IDs while session time advanced. Root inspected its pause frame and independently passed start/final/save/full-pause and64+44artifact checks; audit05 SHA`9582456AD8ABDE86A8A67A98056EFF7CEB4F6C968B5C9E877C2914F141210D92`. Both owned games exited0 without force; oldPID19400 was later reused by a newer unrelated PowerShell process, not left alive. Bare numeric PID absence is insufficient. Current earning06 uses actual05final SHA`5CD39690CA451BB2406CBD7CB3C00D75DF62034D20DC9E36C69BF01573168510`; eggs2/3 and Amp/selector/reload remain pending.

The separate timing change provides [explicit accelerated tests](README.md#accelerated-gameplay-tests) with isolated saves, muted audio and labelled timing. Its new build/suite/native evidence belongs to that change; E80 acceptance continues at1x.

- [Maintained playtest helpers](docs/playtest-helpers.md): 26 regressions and four identified Tank 1 tooling runs; owned processes exited and installation inventories matched. [Report](docs/playtests/2026-10-09-playtest-helpers-01.md).
- [Evidence commands](docs/evidence-tooling.md): 47 contracts, prepared-build/source/artifact readback, and a verified adapter for recorded helper telemetry. Report drafts keep behavioral verdicts **not tested**.
- [Compact delegation briefs](docs/delegation-template.md): reusable fields, explicit ownership/isolation and an unexecuted 4-2 review example; linked from AGENTS.

These maintenance checks retain historical records; they do not establish Tank4-2 gameplay acceptance. E80/E81 are the current resumed build/entry evidence.

## Gaps and blockers

- No missing access or information currently requires user input. The user resumed work; full completion is unproved.
- A32 still needs eggs2/3, one Amp unlock, seventeen-card Locked4-3/reload and final cleanup/lineage acceptance. E80 freezes the resumed bundle; E81–E83 establish only the paths actually observed.
- Tank 4-3 is temporarily gated; live Amp is unimplemented. Bounded primary study is underway under the registered [provenance](docs/provenance.md). Remaining stages, fish/pets, modes, menus/profiles/highscores and effects remain open in [coverage](docs/compatibility.md).
- Retail comparison and human feel remain untested. [Original-game isolation](docs/original-observation.md) is unproved; startup can force fullscreen despite ScreenMode0. [Music](docs/audio.md) has native-load/queue evidence, while audible fidelity, loops/fades/mixing remain untested. Distribution/runtime-code licensing is undecided.

## Ownership

The primary session owns synthesis and final decisions. Retain the [A32 file/resource contract](docs/adventure-4-2.md#ownership-and-durable-state) when resuming:

| Owner | Responsibility |
|---|---|
| Primary | `sim.rs`, `ultra.rs`, `rhubarb.rs`, library registration, integration, records/gates/audit/commit |
| Bilaterus owner | `bilaterus.rs`, `invasion.rs`, `rufus.rs`, `adventure.rs`, `cli.rs` |
| Presentation/Nimbus owner | `fish_pet.rs`, `app.rs` |
| Tester | `tests/persistence.rs`, isolated native evidence and owned-process cleanup |

Serialize shared Cargo/fmt/process/database work. The A33 investigator owns the private Ghidra database; the separate timing chat owns Cargo/fmt/staging until its gate finishes. Tester alone owns A32 runtime processes. Preserve unrelated work and A31 artifacts; keep original installation read-only and project saves separate. Add no new legacy migration, backfill or import.

## Evidence ledger

| ID | Class / current scope | Evidence |
|---|---|---|
| E79 | Observed A32 build gate; reviewed rules, native pending | Identities above; [A32 contract](docs/adventure-4-2.md), [PB59-63 primary ledger](docs/behavior-contract.md#tank4-2-primary-ledger), [design](docs/DESIGN.md#second-tank4-integration), [provenance](docs/provenance.md) |
| E76/E78 | Observed A31 frozen build and eight audited native runs | [A31 report](docs/playtests/2026-10-09-adventure-4-1-breeder-rhubarb-nimbus-01.md): three paid 3000 eggs/Nimbus once, sixteen-card Locked 4-2/reload and exact Board pause; owned PIDs absent/install unchanged |

Installed binaries remain authoritative; WinFish is secondary. Tests alone do not prove retail fidelity. The [historical ledger](docs/history/2026-10-09-status-a32.md#evidence-ledger) preserves E1-E72; [later checkpoint entries](docs/history/2026-10-09-status-a32.md#current-build-verification) preserve E73-E79 and their supersessions.

## Next action

Current next action:

1. Continue identified native A32 acceptance on E80 bundle02 using maintained playtest/evidence tools. Root owns build/source/Git; tester owns isolated run data and PIDs. Coordinate any observed defect before changing frozen inputs, and revalidate affected checks.
2. Preserve the completed genuine source chain from `.scratch/playtests/a31-nimbus-locked-reload-01/adventure.json`, SHA`048319F4E63A9BB3BACC090D21FFBAE8E2ACA1D6C19CADDA7264FDBEA563C38A`. Continue actual05final through earning06; keep the guarded live checkpoint/pause sidebranch separate. Do not convert or edit gameplay state.
3. Execute [A32 acceptance](docs/adventure-4-2.md#acceptance): genuine entry, natural Nimbus conversion/Bilaterus lifecycle/Ultra income, current17 reopen/pause, three paid 25000 eggs/Amp once and seventeen-card Locked 4-3/reload. Inspect events/pixels and audit build, save lineage, original-install integrity and owned-PID cleanup.
4. After acceptance, commit results and automatically implement4-3 from the registered primary study and adjudicated contract. Full requirements remain the goal.

## Environment and reference build

Recorded Windows `10.0.26200.0`, PowerShell 7.6.6, Git 2.54.0.windows.1. Original reference: Steam App 3320/build250752, English v1.1 x86. Both main EXEs have SHA-256 `F54C2C6EE54B00AE6DA7F4BDE15FEB90D0A867ABA6B4151B0277C1F81DA3FF66`. [Full environment/inventory record](docs/history/2026-10-09-status-a32.md#environment-and-reference-build) is historical; rediscover machine-local paths on another host. Shared tools and limits: [analysis tools](docs/analysis-tools.md).

## Setup verification

[Archived setup checks](docs/history/2026-10-09-status-a32.md#setup-verification) and [failed approaches](docs/history/2026-10-09-status-a32.md#attempted-approaches-and-failures) retain their original scope. Current documentation checks are recorded in [MODLOG](MODLOG.md). No workItemId was supplied; Actionables was not updated.

Completed detail and former next-step directions belong to the [closed handoff snapshot](docs/history/2026-10-09-status-a32.md), [MODLOG](MODLOG.md) and linked playtest records. Keep this file focused on the active increment.
