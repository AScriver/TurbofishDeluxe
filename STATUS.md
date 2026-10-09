# Current handoff

Updated: 2026-10-09, America/Phoenix. **Paused at the user's request after the A32 source/build checkpoint.** Resume runtime work only on explicit request. Full scope remains in [requirements](docs/requirements.md).

## Current increment

[A32: Tank 4-2, live Nimbus, Bilaterus and Ultravore](docs/adventure-4-2.md) is implemented with required current-save format 17. Source checkpoint: `4b2d53fd6c6711ad615d083b11f9d41f745ad22f`. Build/regression checks passed; native gameplay acceptance is **not tested**.

[A31: Tank 4-1](docs/adventure-4-1.md) is the latest native-accepted stage, committed `38c65be9ad439a727b2e7e1b1401db928b8e55f6` (E76/E78). Its frozen bundle, reports and genuine earned saves remain preserved.

## Current build verification

**E79 — observed build gate:** A32 gate 04 passed formatting, strict workspace Clippy, locked native build/staging and 407 checks: 335 library, 5 assets, 4 fonts, 60 persistence and 3 binding, including 2 owned-music checks. Current17 continuation, required-field and pause regressions passed. No A32 native launch, earning, pixels or audio acceptance occurred.

| Identity | Recorded value |
|---|---|
| Executable | `target/debug/turbofish-deluxe.exe`; SHA-256 `06CB115C9DC4818B416DD17B166D3B22C2A304DE46111E86AE9080180A39D47A` |
| Gate | `.scratch/runtime/gate-adventure32-current-save-04.log`; SHA-256 `1781C3BE954D454E981C46196CE01042D95F5D8142482F3CC46A2775D7284D1E` |
| Checkpoint | `.scratch/runtime/checkpoint-adventure32-01.local.json`; SHA-256 `4D7B7E8A753499C6AD402B66E06925B3A1994DE8AC8A9682FE9E9C4471B2A904` |
| Toolchain | Rust/Cargo 1.95.0, stable MSVC, Rust 2024; pinned dependencies/Cargo.lock. Exact rustup channel pin pending |

The checkpoint identifies base `38c65be` plus A32 inputs now committed in `4b2d53f`. All 37 build-input hashes, 5 artifact hashes and the checkpoint hash matched during this handoff edit. These documentation changes leave that runtime identity intact; the gate was not rerun. Commands: [README](README.md#workflow-checks).

## Gaps and blockers

- No missing access or information currently requires user input. The explicit pause remains in effect; it is not completion or a technical blocker.
- A32 still needs normal-speed input/render/earning, reopen/pause and cleanup/lineage acceptance. No A32 acceptance bundle, save copy or runtime PID was created at the pause.
- Tank 4-3 is temporarily gated; live Amp and its primary study have not started. Remaining stages, fish/pets, modes, menus/profiles/highscores and effects remain open in [coverage](docs/compatibility.md).
- Retail comparison and human feel remain untested. [Original-game isolation](docs/original-observation.md) is unproved; startup can force fullscreen despite ScreenMode0. [Music](docs/audio.md) has native-load/queue evidence, while audible fidelity, loops/fades/mixing remain untested. Distribution/runtime-code licensing is undecided.

## Ownership

The primary session owns synthesis and final decisions. Retain the [A32 file/resource contract](docs/adventure-4-2.md#ownership-and-durable-state) when resuming:

| Owner | Responsibility |
|---|---|
| Primary | `sim.rs`, `ultra.rs`, `rhubarb.rs`, library registration, integration, records/gates/audit/commit |
| Bilaterus owner | `bilaterus.rs`, `invasion.rs`, `rufus.rs`, `adventure.rs`, `cli.rs` |
| Presentation/Nimbus owner | `fish_pet.rs`, `app.rs` |
| Tester | `tests/persistence.rs`, isolated native evidence and owned-process cleanup |

Serialize shared Cargo/fmt/process/database work. The investigator's private Ghidra database is idle at the pause. Preserve unrelated work and A31 artifacts; keep original installation read-only and project saves separate. Add no new legacy migration, backfill or import.

## Evidence ledger

| ID | Class / current scope | Evidence |
|---|---|---|
| E79 | Observed A32 build gate; reviewed rules, native pending | Identities above; [A32 contract](docs/adventure-4-2.md), [PB59-63 primary ledger](docs/behavior-contract.md#tank4-2-primary-ledger), [design](docs/DESIGN.md#second-tank4-integration), [provenance](docs/provenance.md) |
| E76/E78 | Observed A31 frozen build and eight audited native runs | [A31 report](docs/playtests/2026-10-09-adventure-4-1-breeder-rhubarb-nimbus-01.md): three paid 3000 eggs/Nimbus once, sixteen-card Locked 4-2/reload and exact Board pause; owned PIDs absent/install unchanged |

Installed binaries remain authoritative; WinFish is secondary. Tests alone do not prove retail fidelity. The [historical ledger](docs/history/2026-10-09-status-a32.md#evidence-ledger) preserves E1-E72; [later checkpoint entries](docs/history/2026-10-09-status-a32.md#current-build-verification) preserve E73-E79 and their supersessions.

## Next action

On explicit resume:

1. Recheck E79's source/executable/gate/checkpoint identities, then freeze the A32 bundle. `.scratch/runtime/tester/freeze-a32-01.ps1` passed parser/automatic-variable validation but **was not executed**.
2. Byte-copy genuine `.scratch/playtests/a31-nimbus-locked-reload-01/adventure.json` into isolated project saves. SHA-256 `048319F4E63A9BB3BACC090D21FFBAE8E2ACA1D6C19CADDA7264FDBEA563C38A`; format16, Board null, 4-2 PetSelection, Niko/Itchy/Nimbus, sixteen cards, shell2365. Do not convert or edit gameplay state.
3. Execute [A32 acceptance](docs/adventure-4-2.md#acceptance): genuine entry, natural Nimbus conversion/Bilaterus lifecycle/Ultra income, current17 reopen/pause, three paid 25000 eggs/Amp once and seventeen-card Locked 4-3/reload. Inspect events/pixels and audit build, save lineage, original-install integrity and owned-PID cleanup.
4. After acceptance, commit results and automatically study/implement 4-3. Register new Amp source paths in provenance before study. Full requirements remain the goal.

## Environment and reference build

Recorded Windows `10.0.26200.0`, PowerShell 7.6.6, Git 2.54.0.windows.1. Original reference: Steam App 3320/build250752, English v1.1 x86. Both main EXEs have SHA-256 `F54C2C6EE54B00AE6DA7F4BDE15FEB90D0A867ABA6B4151B0277C1F81DA3FF66`. [Full environment/inventory record](docs/history/2026-10-09-status-a32.md#environment-and-reference-build) is historical; rediscover machine-local paths on another host. Shared tools and limits: [analysis tools](docs/analysis-tools.md).

## Setup verification

[Archived setup checks](docs/history/2026-10-09-status-a32.md#setup-verification) and [failed approaches](docs/history/2026-10-09-status-a32.md#attempted-approaches-and-failures) retain their original scope. Current documentation checks are recorded in [MODLOG](MODLOG.md). No workItemId was supplied; Actionables was not updated.

Completed detail and former next-step directions belong to the [closed handoff snapshot](docs/history/2026-10-09-status-a32.md), [MODLOG](MODLOG.md) and linked playtest records. Keep this file focused on the active increment.
