# Workflow setup and adaptations

Setup date: 2026-10-08 (America/Phoenix). This checkpoint installs an agent workflow, not a runtime. Source IDs and exact revisions/licenses/paths are authoritative in [provenance](provenance.md); observations and verification results are authoritative in [STATUS](../STATUS.md).

## Source-to-project mapping

| G1 source | Project file | Deliberate adaptation |
|---|---|---|
| `templates/AGENTS-starter.md` | [AGENTS.md](../AGENTS.md) | Standalone Rust rules, startup order, engineer audience, actual evidence, document ownership, applicable inherited Windows conventions |
| `templates/STATUS-handoff.md` | [STATUS.md](../STATUS.md) | Current goal/environment, stable evidence IDs, original-game identity, explicit gaps and next action; remove host/player authority question |
| `templates/MODLOG-template.md` | [MODLOG.md](../MODLOG.md) | Keep changed/why/check/result/failure/next fields and newest-first history; remove transport example |
| `templates/PLAYTEST-report.md` | [docs/playtests/TEMPLATE.md](playtests/TEMPLATE.md) | Identify original and Rust builds, dirty changes, starting state, inputs/time/RNG, independent expectations, runtime results, agent versus human checks |
| `templates/ATTRIBUTION-and-lineage.md` | [docs/provenance.md](provenance.md) | Distinguish studied sources, adapted text, future code reuse, game identity, licensing gaps, AI and human contributions |
| README/index and guides 3, 4, 12, 14 | [README.md](../README.md), [docs/DESIGN.md](DESIGN.md) | Honest current capability, route 5, small vertical slice, explicit unresolved architecture; preserve full product scope in [requirements](requirements.md) |
| `.gitignore`, template rules, guide 6 | [.gitignore](../.gitignore) | Allow maintained Markdown, PowerShell, and reserved Rust source paths; deny private trees even for source-shaped files; verify with quiet ignore checks |
| `LICENSE` | [THIRD-PARTY-NOTICES.md](../THIRD-PARTY-NOTICES.md) | Retain exact MIT notice for adapted guide text; do not assign it to future runtime code |

The guide index is the Guides table in G1's README; no separate index file was assumed. Required templates and chapter-linked S1 workflow/provenance documents were retrieved and read. No existing project files needed merging: the initial directory was empty and was not a Git repository. A local `main` repository is initialized for review and ignore checks; no remote, staging, commit, or push is part of setup.

## Context decision

S1's `CONTEXT.md` describes its maintainer's ignored artifact journal, named iterations, agent clones, and landing/publication tooling. Its distinct functions were considered:

- Private evidence and external checkouts use `.scratch/` and `private/` here.
- Durable current handoff is STATUS; chronological findings and failures are MODLOG; architecture/acceptance is DESIGN; original/Rust execution reports are in `docs/playtests/`.
- Completed evidence/reports remain historical, with corrections added later.

These roles are already covered by the installed records. Adding `CONTEXT.md` would duplicate ownership and reading rules, so **it is not added**. Useful sanitized handoffs remain suitable for Git; an ignored local journal is not the sole record available to a future session.

## Other deliberate differences

Use available project facts and recovered runtime requirements; no ten-question beginner interview. Replace host/guest, loaders, and movement-control transport assumptions with a Rust-owned simulation and user-supplied assets.

Use instrumentation and visual inspection when available, with human playtests identified separately. G1's blanket prohibition on agent visual checks is inappropriate to the requested workflow. Require actual runtime evidence and independent expectations; logs and tests have explicit limits.

Keep meaningful regression tests, useful explanatory comments, and existing history. Do not adopt S1's test deletion, automatic soft-reset/squashing, fixed `master` branch, Makefile/`cargo xtask mr`, copied `.env`, or deployment infrastructure. Record failures, reassess an approach after repeated failures, and keep progressing on unblocked work.

No hook, worktree manager, Cargo crate, runtime dependency, game extractor, or analysis database is added. The only maintained helper is [scripts/Test-Workflow.ps1](../scripts/Test-Workflow.ps1), needed for repeatable setup verification. Existing machine helpers validated the PowerShell acquisition/inspection/check scripts. The [README](../README.md#workflow-checks) documents the working commands and optional host validator.

The ignore allowlist handles the actual asset extensions and private locations found during inspection by default-denying data files. It is a path rule, not proof of ownership or a defense against forced additions. Review proposed additions and licenses before committing. No original game content was copied into maintained files.

## Verification and handoff

See [STATUS setup verification](../STATUS.md#setup-verification) for exact executed checks and limits, and [MODLOG](../MODLOG.md) for changes and failed attempts. The next concrete task and acceptance criteria are in [DESIGN](DESIGN.md#next-task-original-game-contract). Setup deliberately leaves game observation, behavior recovery, framework selection, and gameplay implementation for that task.
