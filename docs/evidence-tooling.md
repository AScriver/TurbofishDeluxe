# Evidence bookkeeping

Maintained tooling creates immutable version1 JSON receipts and playtest drafts. Raw receipts remain in ignored project directories; only reviewed, sanitized reports travel with Git. This tooling measures identity and artifact integrity. Behavior, discrepancies, visual/audio comparison and human judgment still require review.

## Commands

Run from the repository root in PowerShell7. After changing scripts, validate them with the host `Invoke-CodexPowerShell.ps1` described in the [README](../README.md#workflow-checks). Put invocations with arrays or multiple steps in a private task script and validate/execute that script; the host wrapper cannot forward nested native argument arrays reliably.

Every successful launcher preparation now creates a build receipt automatically, including when invoked by the full runtime gate. Preflight and focused formatting produce no build identity. Choose a stable output path when the next steps need to reference it:

```powershell
.\scripts\Start-TurbofishDeluxe.ps1 -Offline -PrepareOnly -BuildIdentityPath .scratch/evidence/example-build.local.json
.\scripts\Test-Evidence.ps1 -RecordPath .scratch/evidence/example-build.local.json -CheckSources
```

Preparation builds and stages the project runtime without launching it. With no output parameter, JSON generators and readback choose a timestamp/GUID filename under `.scratch/evidence/` and return its path, ID where applicable, and SHA-256. Existing files are immutable: use a new path for each attempt, correction or updated observation.

To inventory an already frozen bundle, use the standalone generator. Its build/source association is explicitly **unverified**; a snapshot cannot establish which source produced an existing executable:

```powershell
.\scripts\New-BuildIdentity.ps1 -ExecutablePath .scratch/runtime/bundle/turbofish-deluxe.exe -OutputPath .scratch/evidence/bundle-build.local.json
```

`-BuildProfile`, `-Features`, `-BuildArguments` and `-NativePackageDirectory` describe a standalone capture. `-OriginalExecutablePath` optionally captures the original executable's name, version strings and digest by reading it. `-AdditionalArtifacts` includes explicit configuration, validation logs, fixtures or other evidence. The launcher forwards `-BuildEvidencePaths` into that artifact list. These arrays belong in the validated task script. Configurations or build inputs outside the captured surface need explicit artifacts; this is identity bookkeeping, not a certification of reproducible builds.

After an owned run has finished writing its telemetry, consume **one explicit run directory**:

```powershell
.\scripts\New-RunManifest.ps1 -RunDirectory .scratch/evidence/runtime/example -BuildIdentityPath .scratch/evidence/example-build.local.json -ScenarioPath .scratch/evidence/example-scenario.local.json -StartingSavePath .scratch/playtests/example/start.json -FinalSavePath .scratch/playtests/example/adventure.json -OutputPath .scratch/evidence/example-run.local.json
.\scripts\Test-Evidence.ps1 -RecordPath .scratch/evidence/example-run.local.json -CheckSources
.\scripts\New-PlaytestReport.ps1 -RunManifestPath .scratch/evidence/example-run.local.json -OutputPath .scratch/evidence/example-draft.md -Executor agent -Goal 'Describe the actual scenario and claim IDs'
```

The example files/directories are placeholders; no scenario is launched by these commands. Starting/final saves and scenario files are optional. Include a scenario file rather than silently treating missing inputs as known. `-RunProcessId` optionally checks whether the supplied PID is absent at capture; a present or reused PID requires review. It does not kill a process, establish its historical identity or invent an exit code. `-AdditionalArtifacts` includes separate cleanup/lineage receipts, input-driver logs and other required evidence.

A private scenario is a JSON object. Keep exact launch arguments, ordered inputs or input-log references, seed/RNG context, time source, normal-speed/settings evidence, reset/reproduction steps, independent expected evidence and save origin there. For example:

```json
{
  "launch_argument_list": ["--evidence-dir", "<private-run-directory>", "--save-dir", "<isolated-save-directory>"],
  "inputs": "<ordered input log or procedure with times/ticks>",
  "time_source": "<observed timing and fixed-tick configuration>",
  "expected_evidence": "<contract/claim IDs and independently supported expectations>",
  "save_origin": "<genuine earned source, new game, or explicitly controlled fixture>"
}
```

This object preserves supplied context, including unresolved fields; it does not validate a behavioral procedure. The runtime identity artifact retains the complete initial session/RNG state. Seed equality alone does not establish retail RNG equivalence.

New runtime telemetry identifies `test_speed`, `time_mode` (`normal` or `accelerated-test`) and `session_elapsed_seconds`. `elapsed_seconds` remains wall time; session elapsed derives from the run-start tick delta and includes paused session updates. Identity also records held-input clock and physical-frame connector-observation policies. Manifests retain these fields and flag inconsistent, missing or invalid timing labels when an identified run supplies them. Historical telemetry without timing fields stays `unrecorded`; the generator does not invent a 1x claim. Report drafts show the factor and both clocks and keep accelerated exploratory/stress evidence separate from normal-speed acceptance.

Reports reuse [the maintained template](playtests/TEMPLATE.md). Template links are rebased for the draft's location. A new `docs/playtests/<name>.md` is also an allowed destination; the generator never overwrites a report or TEMPLATE. It fills receipt time, build/source/executable identity, seed/tick/elapsed fields and a mechanical artifact summary. It does not export private argument lists, environment values, absolute artifact paths or dirty-path lists. Replace the default scenario rows with the actual claimed procedure, supply portable reproduction details, assess behavior/discrepancies and review the draft before sharing it. All behavioral/original/visual/audio/human verdicts start **not tested**.

## Receipt contract and freshness

| Record | Captured fields and limits |
|---|---|
| Build, schema1 | Commit/branch/dirty status, tracked-diff digest, sorted per-file source digests, Rust/Cargo/rustup/PowerShell/OS identity, declared profile/features, exact supplied build arguments and listed build environment, executable, five required native DLLs, discovered DLL/notice inventories, available native archive/import library and explicit artifacts. Optional original metadata stays separate from fidelity. |
| Run, schema1 | Build-receipt ID/digest, one run directory, supplied scenario, actual seed/tick/elapsed telemetry, event/frame counts, executable/install consistency, optional PID check, identity/events/final/optional-state/frame/save/additional-file digests and issues. The receipt does not count prep/fixture directories as runs. |
| Readback, schema1 | Checked receipt digest/ID, individual file matches/missing/changed/added statuses, optional source checks and retained run issues. A failed check writes a new failure receipt and returns an error. Malformed or unsupported receipts fail before a misleading readback is created. |
| Report draft | Template plus sanitized identity and mechanical summary. Artifact integrity and behavioral acceptance have separate verdicts. |

Build/run IDs hash their captured contents excluding ID and capture time. This checks internal consistency; it does not authenticate the author or truth of supplied observations. Outputs are private project files except deliberately chosen new Markdown report drafts. Output containment also rejects junctions/symlinks along the destination path. No output goes into the owned installation.

The source set is deliberately conservative: tracked and eligible untracked Rust, crate/build/test/Cargo/toolchain inputs and **every maintained `scripts/*.ps1`**. It excludes prose, private trees, raw assets and compiled outputs. Per-file readback tells the reviewer exactly what changed. A changed test or tooling script makes its recorded check historical; it does not automatically invalidate unaffected gameplay evidence. Committing unchanged input bytes changes Git metadata without failing source freshness. Include noneligible/local/external configuration and dependencies explicitly when they affect a build.

The launcher captures inputs before Cargo and again after staging; a successful unchanged-input preparation labels the association `launcher-success-with-unchanged-inputs`. Without that observation, the standalone receipt remains a snapshot. The executable digest and toolchain/configuration still need assessment within this limit.

Run capture retains missing or malformed required telemetry, invalid numeric/hash/type fields, build/executable/install mismatches and supplied live PIDs as issues. Syntactically valid event rows require time/tick identity; a quiet run may have zero events, explicitly counted. `identity.local.json`, `events.local.jsonl` and `final.local.json` are required. `state.local.json` is optional. Frames are hashed as files; their pixels are not assessed. Save hashes establish byte identity only; they do not prove typed lineage or reload equivalence.

Default readback verifies run artifacts **and the referenced build's artifacts**, including newly added DLLs/notices/frames. `-CheckSources` also checks current source membership/bytes and catches additions/deletions. Preserve frozen bundles at durable private paths; replacing or removing a referenced executable correctly prevents its old receipt from passing artifact readback. Installation consistency comes from the runtime's matching before/after inventory digests and `game_unchanged`; it is not a new independent live installation audit or an original-game observation.

## Verified checkpoint

On 2026-10-09, host parser/automatic-variable validation passed the evidence commands, launcher and workflow checker. [The synthetic contract suite](../scripts/Test-EvidenceTools.ps1) passed 47 checks, including immutable outputs, private-path restrictions, altered/missing/added artifacts, receipt-content tampering, incomplete/malformed telemetry, invalid scalar fields, executable/install/PID discrepancies, source additions/deletions, quiet runs, transitive build readback and complete Markdown bullets. Run it with the host validator's `-Execute` switch. Its receipt and fixture build/source identity are under `.scratch/evidence/tool-tests-d95cc72a632e441b9364d8670ff1e09a/`.

Independent Tester challenges first falsified transitive build readback, scalar validation and draft formatting. The failed fixtures remain in `.scratch/evidence/tester-bookkeeping-1830a/`; the corrected recheck passed all three at `.scratch/evidence/tester-bookkeeping-recheck-7c914/correction-results.local.json`. An initial recheck had an invalid event fixture and is explicitly superseded. These are synthetic tooling checks, with no gameplay claim.

Offline real preparation/readback passed at `.scratch/evidence/prepared-build-20261009-02/integration.local.json`: 48 conservative inputs, 23 actual artifacts, associated build ID `098d139c95518d7058f4d8706e78666f7e93d98d133c3c037040dde5767068d8`. Its executable retains E79 SHA-256 `06cb115c9dc4818b416dd17b166d3b22c2a304de46111e86ae9080180a39d47a`. Preparation01 remains historical after the final workflow/test changes. No game was launched.

The adapter also consumed existing A31 native telemetry read-only: five historical input files unchanged, four event rows parsed, exactly the expected A31-executable/A32-build mismatch retained, and a draft explicitly marked failed/incomplete and not tested. Receipt: `.scratch/evidence/historical-consumer-20261009-01/adapter.local.json`. This validates the real telemetry schema adapter; it is no new gameplay acceptance. A32 remains paused. Existing per-campaign helpers, reports and receipts are preserved; no historical receipt is silently converted or overwritten.

The coordinating chat independently passed the full integrated 407-check runtime gate, strict lint/build, and this 47-check suite. It also consumed retained current-build helper telemetry through a run manifest, 85-file transitive artifact/source readback and draft generation with zero bookkeeping issues and verdict **not tested**. Receipt: `.scratch/efficiency-coordination/integration-0fc896b480f847fc88ceb901c45b2c3f/integration.local.json`. That consumer check launched no game; helper execution remains tooling evidence rather than A32 acceptance.
