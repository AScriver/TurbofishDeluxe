# Shared analysis tools

These tools are already installed on the current Windows host. Use them when the [next behavior-recovery task](DESIGN.md#next-task-original-game-contract) needs static analysis or instrumentation; no installation or game analysis is part of this setup checkpoint.

## Locate and read the catalog

The user supplied the shared tool directory. Resolve it without committing a machine-specific absolute path:

```powershell
$analysisRoot = Join-Path $env:USERPROFILE '.codex\tools\reverse-engineering'
$toolRunner = Join-Path $analysisRoot 'Run-Tool.ps1'
```

Before using a tool, read that directory's `README.md`, `manifest.json`, and `verification.json`, then inspect the runner where needed. They own the exact entry points, versions, build/download provenance, recorded checks, and limitations. Recheck them if the installation changes. If another machine lacks them, report the missing capability rather than assuming it exists or installing software without authorization.

## Available tools and verification limits

Observed 2026-10-08: all 10 manifest entry-point files existed and their SHA-256 values matched the manifest. The catalog's earlier smoke tests were recorded 2026-10-03; they were read, not rerun wholesale. Fresh checks in this project exercised only the Frida, TShark, and Python version commands through the runner.

| Tool | Catalog version | Runner selector / role | Verified scope and limits |
|---|---|---|---|
| Ghidra headless / PyGhidra | 12.1.4 / 3.1.0 | `headless`, `pyghidra`, or `python` for bounded API scripts | Catalog records headless/JVM/API startup and decompiler construction; no Insaniquarium import or analysis tested |
| Private Python / Temurin JDK | 3.11.9 / 21.0.12.1+1 | `python`; runner supplies Java environment automatically | Python version freshly checked; JDK/Ghidra startup comes from the earlier catalog checks |
| Frida / Frida tools | 17.22.0 / 14.11.0 | `frida`, `frida-trace`, `frida-ps` | Frida version freshly checked; game attach, tracing, and process enumeration not tested |
| TShark / Wireshark | 4.6.9 | `tshark`, `wireshark` | TShark version freshly checked; GUI, packet capture, and live-capture prerequisites not tested in this project |
| Local Cheat Engine x64 | 7.5.0.7431; source `59930b596dabbd83e37191e4a58e72f09a9beb75` | `cheat-engine`; ordinary GUI/Lua application | Catalog records a successful x64 build. GUI, Lua, process-memory access, scanning, and debugging were not tested; not a verified headless command or installed MCP service |

The Cheat Engine filename `NWPreservation-CE-x64.exe` reflects its local packaging, not a compatibility guarantee. The original game's x86 identity is in [STATUS](../STATUS.md#environment-and-reference-build); compatibility with that target remains unverified. No optional driver/debugger projects or game-specific integrations should be assumed.

## Invocation and private output

Put invocations in a task-specific `.ps1`, with explicit tool names and native argument arrays. Validate and execute that entire script through the existing host PowerShell validator described in [README](../README.md#workflow-checks). For example, this version-only invocation was exercised:

```powershell
$analysisRoot = Join-Path $env:USERPROFILE '.codex\tools\reverse-engineering'
$toolRunner = Join-Path $analysisRoot 'Run-Tool.ps1'
& $toolRunner -Tool frida -ArgumentList @('--version')
```

Keep tool switches inside the task script's `Run-Tool.ps1 -ArgumentList` array. Do not forward nested tool arrays through the validator's own `-ArgumentList`; its `pwsh -File` boundary loses the nested array. The runner supplies and restores its private Java/Python environment without permanently changing PATH.

Prefer bounded headless Ghidra/Python queries for selected functions, offline TShark where relevant, and bounded Lua work over desktop automation. Ghidra rejects database paths containing a dot-prefixed segment: place its databases under ignored `private/ghidra/<run-id>`, **not `.scratch/`**. Analysis scripts and logs may use `.scratch/`. Other raw application-derived output remains in the project's ignored private trees; sanitized findings and source/target identities go in maintained records.

Apply the project's target-identity and authorized-observation boundaries before import, attach, instrumentation, capture, or application changes. Tool availability is not evidence of game behavior or permission for an unrelated target. Record actual invocation, tool/target identity, configuration, outputs, and limitations when used. No game process was accessed, GUI launched, capture started, driver installed, or runtime code implemented during this tools follow-up.

Current local audit evidence is `.scratch/evidence/setup/analysis-tools.local.json`; catalog provenance is recorded in [provenance](provenance.md#local-tool-catalog).
