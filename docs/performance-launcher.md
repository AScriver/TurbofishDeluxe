# Optimized normal launch

Actionable #295 / PA1 changes the normal launcher to the existing Cargo release
profile. Explicit `-Development` chooses Cargo's debug profile; `-Release`
remains supported. Simultaneous true `-Release` and `-Development` switches fail
before native preflight or staging. A single profile selection drives the Cargo
argument list, executable directory and immutable build receipt.

This uses the existing release configuration (thin LTO, one codegen unit) and
does not change simulation rules, clocks, saves or dependencies. The development
profile uses the unchanged Cargo development defaults. Native hash checks,
DLL/notice staging, offline behavior, `-PrepareOnly`, `-PreflightOnly` and runtime
argument forwarding retain their contracts. `-Release:$false` means no explicit
release request; the normal release default still applies. `-Development:$false`
also retains that default. The PowerShell common `-Debug` parameter controls
diagnostics, so it is not used as a build selector.

## Verification

Eleven launcher contract cases passed on `bb343082` plus the identified launcher
and test changes. The maintained
`scripts/Test-Launcher.ps1` uses a byte-identical copy of the launcher and its
evidence helpers in an ignored disposable Git fixture, with recording Cargo and
runtime executables. Those executables record arguments and copy themselves;
they do not compile or execute the game. Complete receipt readback covers source
association and staged native files. Both real profiles compiled and staged
without launching the game; each receipt passed 117 source/artifact checks.
Cross-profile fixture comparison and coordinator-leased final timings passed.

The first fixture run failed before compilation because it assumed a
`rust-toolchain.toml` existed. The repository currently has none; optional copying
now preserves either checkout configuration. Strict PowerShell lint then found
the test helper's plural noun and non-ASCII source without a BOM. Renaming the
helper and constructing its Unicode argument by code point resolves both while
preserving the same contract. Failed attempt output is retained; real build
receipts are sealed only after final maintained inputs stop changing.

Human gameplay, native startup/FPS, audible behavior and retail fidelity remain
unmeasured by this task. Original installation, live saves and frozen/earned
lineages are outside its resources. Root owns the README and central records.

## Build identity and focused checks

Checked checkout: `bb3430822d63e34a90fff61b9f16e4bd6ec62a78` plus changes only
to the launcher, its new test and this report. Runtime/Cargo sources are
unchanged from `9188cbc`; the report is not a build input. Rust/Cargo 1.95 MSVC,
PowerShell 7.6.6, locked/offline dependencies, two Cargo jobs and worktree-local
`target` were used. No `CARGO_PROFILE_*`, `RUSTFLAGS` or target override applied.
Release uses default opt-level 3/debug 0 plus existing thin LTO/codegen 1; explicit
development uses Cargo defaults opt-level 0/debug 2/codegen 256/no LTO. The
toolchain is installed stable; the repository has no exact channel pin.

| Artifact | SHA-256 |
|---|---|
| Launcher | `90FB74A1821773FD8A28BB7407C9C727C8D12327C6A3C15BD9FF9517795A5737` |
| Launcher test | `1C3C103ED7BF9ACC0556ABA19A56DF60474F06BD447C5649C3B4E2D93074B2C2` |
| Cargo.toml | `0642F06EC0072E64EBE85FA1F5AD547C6AD825F6A57C3BDD025C9B7D2584DDFC` |
| Cargo.lock | `1643FD7BE00AF38DB3DA764BE8E0C03CFC427EE368DCAAB7EF17A49B187F153F` |
| Release game executable | `CFD48F308270276CB247761AD17C230FFE0AB855A97A0609D1306859817F2FDF` |
| Development game executable | `36F2C622000F1F6F0D5857A2282D8665CB6D9F335DAD61CF313D07FC02A6BA8E` |
| Release library | `811C758BA9C36EC1B3BBFCF1460A7C1ECA945301F8783EC6407FAB6D695C8BC7` |
| Development library | `D2BE9B756AC0310B0102C1ECA76500129846E5DA9072A775AC54194DC6B8385B` |
| Private harness source | `8323EE5C95CCEF40ABB0DF652D9C3BE1739932451EA5DC332B094E84D5CFF008` |
| Release harness executable | `18AC9317C2A8F7F57C2D032592E0F6C56C24C95422BD58B616A690E03311793C` |
| Development harness executable | `465E3CE8A1760FB6C06556B6A5EA241B33F4CCF58118577A21C5176AA426F1B4` |

Private source-associated receipts, full compiler arguments, library identities,
fixture bytes and logs remain under `.scratch/performance/295/`. The pinned
libopenmpt 0.8.9 package was read and verified; five DLLs and notices were staged
only into the owned target. Both actual build receipts and readbacks preserve
package/library digests. No original-game version was accessed; its identity is
explicitly unknown for these checks.

Executed commands, with package/output paths supplied as isolated parameters:

```powershell
.\scripts\Test-Launcher.ps1 -NativePackageDirectory <pinned-package>
.\scripts\Start-TurbofishDeluxe.ps1 -Offline -PrepareOnly -NativePackageDirectory <pinned-package> -BuildIdentityPath .scratch/performance/295/release-build-02.local.json
.\scripts\Start-TurbofishDeluxe.ps1 -Development -Offline -PrepareOnly -NativePackageDirectory <pinned-package> -BuildIdentityPath .scratch/performance/295/development-build-02.local.json
.\scripts\Test-Evidence.ps1 -RecordPath <profile-build-receipt> -OutputPath <new-private-readback> -CheckSources
.\scripts\Test-Workflow.ps1
git diff --check
```

Scripts were validated/executed through the host PowerShell wrapper. Strict
`Invoke-ScriptAnalyzer -Severity Warning,Error` passed on both maintained scripts.
The final contract run passed 11 cases; workflow passed 35 required files,
93 documents, 1027 local links, 344 anchors, 45 included/37 excluded path checks
and 199 eligible files. No Rust source changed, so no unrelated regression suite
was repeated. The harness compiled with warnings denied, using each real
profile's library and matching release/development optimization/LTO settings.
The only maintained launcher consumer is `Test-Runtime.ps1`; its final `All`/
`Build` staging now inherits release, while direct Cargo lint/test commands stay
unchanged. Native playtest helpers take an explicit executable and digest.

## Deterministic offline fixtures

Seed `0xa295` (41621) and exactly 32 updates at the unchanged 28ms logical step
produce complete serialized Boards and ordered event arrays. The normal fixture
uses the first-stage constructor, two starter fish and one ordinary food drop.
The populated fixture uses Tank3-1 with Stinky/Meryl, synthetic initial cash/
unlock flags, 18 production guppy purchases and nine ordinary food drops;
twenty fish start with hunger 250. Both inputs and outputs pass production
validation. These fixtures contain no held-input state, live save or earned
progression. Both profile outputs/event arrays match byte-for-byte.

The asset fixture creates original synthetic PNG bytes and a minimal resource
manifest. The actual `GameAssets::load_image` path loads a 128x64 color sheet,
four columns/two rows and a 32x32 grid-alpha mask. All 8192 resulting RGBA pixels
match independently specified coordinate/channel formulas, and output bytes
match across profiles. This checks CPU decode/mask preparation, not GPU rendering
or original artwork.

| Complete fixture/result | SHA-256 |
|---|---|
| Normal input | `00CA35BA6F77965E83C8FFF3BFF283B3C297F65BE6CE752A015402FB2C8BBAFC` |
| Populated input | `C278F8CAE2E7C933729D30B5FB10C813212FC6C2A20E1B38CCCCA021B42EECA5` |
| Normal 32-update output | `A68909573A8C80C5C211148ED2D220CD21121BE1E715D10B881518D195F81E9F` |
| Normal ordered events | `29AED193ACA4965879800C9C6C06229B15BCA8917D61806030AC936BD90F4339` |
| Populated 32-update output | `6CEAC20C2072F1F8EDFD4DC123E6CEB1957D9B9611E335257BCE5CF3AC1C1580` |
| Populated ordered events | `BBBB85D34D6A02CBED3D5094730FB036DE831BA418A20AEB41956952FF1FE0E8` |
| Complete decoded RGBA bytes | `102C88B37DE82FC32024E22B89A8F90E5646751479B7A948F373A2F97A5817D4` |

The first private harness tried to link the thin-LTO release library without
LTO and failed with MSVC LNK1136/unresolved symbols. Aligning compiler flags with
the selected profiles succeeded for both; actual launcher executable readbacks
had already passed. This failed consumer configuration and the earlier build
source-guard rejection are preserved in private attempt notes.

## Offline timing results and method

The prepared private timing command is `measure-profiles.ps1` in the owned
scratch directory; it runs the two already compiled harnesses serially, with
no compilation. The coordinator granted the quiet lease after all other heavy
jobs stopped. Execution fell within 17:31:49–17:32:13 America/Phoenix on
2026-10-11 (00:31:49–00:32:13 UTC on October12); the script exited0 and explicitly
released its lease. All worker-owned build/harness processes had exited.
Each case uses 100 warmup batches and nine samples. Simulation samples contain
500 identical-input batches of 32 ticks; Board cloning and final Board teardown
stay outside the timed region, while returned-event disposal stays inside.
The sample value is nanoseconds per tick. Decode samples contain 500 actual
image/mask loads, including cached-file I/O, allocation and result disposal;
their value is nanoseconds per complete decode. Full distributions are retained,
with no timing assertion or selected best-run claim.

| Actual offline work | Previous debug default median (range), microseconds | New release default median (range), microseconds | Median ratio |
|---|---:|---:|---:|
| Normal two-fish simulation tick | 1.820938 (1.644106–2.084481) | 0.114631 (0.089106–0.218325) | 15.89x |
| Populated twenty-fish simulation tick | 15.227850 (14.184144–17.107500) | 0.901788 (0.640425–1.283469) | 16.89x |
| Synthetic image decode/grid-alpha composition | 1463.1744 (1346.7342–1601.1612) | 910.7672 (847.3820–1029.8514) | 1.61x |

All measured release cases improved; the smaller decode gain and complete
sample ranges remain visible. Release tick samples vary substantially, so these
numbers are descriptive of this controlled host/fixture run. The simulation
region excludes cloning, serialization, file publication and whole frontend
frames. Decode includes cached-file path/open work and result disposal. These
measurements compare effective profiles, including matching harness compiler
settings, not an isolated compiler pass or a human-owned game process. They
do not measure startup wall time, FPS, GPU rendering, audible behavior or retail
fidelity. No original assets were decoded. Normal input remains assigned to the
human; no native playability claim follows.

Complete results/event/pixel outputs still matched after timing. Full samples
and identities remain in `timing-debug-01.log`, `timing-release-01.log`,
`timing-comparison-01.local.json` and `harness-identity-01.local.json` in the owned
private directory. Their respective SHA-256 values are:

- `4FA2774C7B0A509F827E3E6B3B73E692701F3C56539F13317A28005ECBB4B609`
- `258F5A66AC8F44F28390211D2ADC2FBAE8818E316C98D0EC32E81B77AECCB3CD`
- `44DA075CC6CF23D86D7285E8938C4BD1F7E84599EF27E74E2313469F0DBE84B9`
- `BB3367C3DBA8D02DCBA35F5AA8CC21D0628E4FDE206A949C64FAB478DBCF8184`

Only owned maintained files are committed; ignored fixtures/artifacts remain
private and retained. Root owns later joined-source validation and central
records. Earlier/native/human lineages, services and automation remain untouched.
