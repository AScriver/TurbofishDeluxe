# Offline profile event performance

Actionable **#292** adds an early return to `Profiles::record_events` for
batches without `StageResultRecorded`, `TimeTrialExpired` or
`ChallengeCompleted`. The normal frame loop calls this method every frame.
Previously, it copied the current name and every profile/session before
discovering that ordinary events could not change any score.

The guard follows the existing missing-current return and precedes both
clones. Its iterator and variant checks allocate nothing. This establishes
the absence of whole-manager allocation on empty/irrelevant batches by source
inspection; allocator calls were not instrumented. Score-bearing batches keep
the original conversion, table validation, ordering, copied names and
save-before-commit path. Overflow/missing-survivor events and nonqualifying
score ties retain that path, including its cloning cost. No signature, returned
boolean, checkpoint API, dependency or saved format changes.

## Fixture and checks

The unchanged [test harness](../tests/profile_event_performance.rs) uses seed
`0xa292`, eight profiles named `Player00` through `Player07`, and deterministic
synthetic profile generations. Each profile has a populated 32-guppy Adventure
board parked by the production mode-switch API while Virtual Tank is active.
Synthetic shop admission is explicitly enabled before purchases. All profile,
retained-scene and score validations pass; the complete manager round-trips
through the current decoder. All 29 score tables are populated to their existing
capacities. These are disposable fixtures, not earned progression.

Six contract regressions passed before and after the change:

- Empty/irrelevant batches preserve serialized state and ignore an unwritable path.
- No current profile returns false even for an invalid score event.
- All three score events retain their units, captured names, durable round-trip,
  one-row Adventure duplicate rejection and mode-table tie insertion.
- Conversion overflow, missing Tank5 survivors and full-table ties publish nothing.
- An invalid score after a qualifying score rolls back the whole batch.
- A failed save leaves the prior in-memory manager and blocking file unchanged.

The ignored `profile_event_hot_path` case warms each batch with 256 calls, then
collects nine samples of 10,000 calls. Fixture setup, cloning, validation and
serialization stay outside the timer. Empty, three-event irrelevant and
nonqualifying duplicate-stage batches each must return false throughout, create
no profile file, and leave the complete serialized-manager SHA-256 unchanged.
There are no timing assertions.

## Results

Observed on 2026-10-11 under the coordinator's serial timing lease, after all
three lanes held heavy compilation. Both retained EXEs ran sequentially; the
validated measurement/comparison script completed in 6.7 seconds. Times below
are nanoseconds per call:

| Batch | Before min / median / max | After min / median / max |
|---|---:|---:|
| Empty | 14205.510 / 14918.450 / 15605.250 | 6.120 / 6.170 / 14.900 |
| Three irrelevant events | 8814.790 / 9374.120 / 10817.330 | 7.580 / 7.920 / 8.960 |
| Duplicate Adventure score | 9615.920 / 10003.750 / 11491.820 | 14723.560 / 15601.650 / 16450.650 |

Empty and irrelevant medians fell 99.96% and 99.92% in this fixture. The duplicate
control retains full cloning and measured 55.96% slower in this ordered run;
the cause was not isolated. Preserve this counter-evidence: no improvement is
asserted for score-bearing batches. These are one before/after run with nine
samples each, subject to host, allocator and compiler variability, rather than
guaranteed game-wide results. No passing timing was repeated.

The comparison script verified identical fixture-source hashes and identical
complete serialized manager input/output for all three before/after cases:
`60d5cd9fa335dcc078bbbbc2e6d8a6431f450ec0633a94ac2059634d4798afe9`.
Every call returned false and no profile file was created. Meaningful score and
failure behavior passed the six explicit contract regressions on both builds.

## Identities and commands

Both builds identify baseline commit
`0d971b3c935841b48703cb757f8c2bf085d099d5` plus the same new test file. Final
production source differs only by the 12-line early guard in `record_events`.
The coordinator owns shared records and other performance lanes.

Host: AMD Ryzen 7 9800X3D, eight cores/16 logical processors, Windows x64.
Compiler: Rust 1.95.0, commit `59807616e1fa2540724bfbac14d7976d7e4a3860`,
LLVM 22.1.2, `x86_64-pc-windows-msvc`. Measurements use release opt-level3 with
`CARGO_PROFILE_RELEASE_LTO=false` and
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16`, locked offline dependencies and the
existing libopenmpt 0.8.9 package. `CARGO_TARGET_DIR` is the worktree's isolated
`.scratch/performance/292/target`; normal launcher artifacts are untouched.

| Identity | SHA-256 |
|---|---|
| Baseline `src/profiles.rs` | `7238FF6D21BE952406E0436CF1308973BA3D757571B1D165F4E0012DD93F664D` |
| Final `src/profiles.rs` | `35DAA25D2B74ADE38DBCDEE9982C0837712267A77A7257260F0D79C3A8E114E5` |
| Unchanged fixture source | `B6875E9E48408462495270550650B0134A5632ADC9325C83932C242E1F45983B` |
| Unchanged `Cargo.lock` | `1643FD7BE00AF38DB3DA764BE8E0C03CFC427EE368DCAAB7EF17A49B187F153F` |
| Baseline test EXE | `BF13CF844815570E34659A8904E7B37F54D66EEC9A578EC4E164F6165DA2D566` |
| Final test EXE | `B54E5A90BB338D890D3121AEE61D3E4711BBED46D1780DB292639440DD795BCB` |
| Isolated final debug game EXE | `DAEC9CE54BF55D1D73300009E8735826866D626E4F6E0EE4F0CA619D6D2ED701` |
| Private baseline timing log | `919A7819AD317A36E22326D373BBE7F15748B87F5B2DB657C43B5CC201695248` |
| Private final timing log | `C1347BFDEC7410C08CB8732380ED61E31B00FEFFE06C4D09B0EFA67A9C98867D` |
| Private comparison receipt | `75E358E53BECC6499E31CBA067EB64A7DA414C6525FEC15EAC962C657A99265D` |

Actual preparation command, with the stated private environment:

```powershell
cargo test --locked --offline --release -p turbofish-deluxe --test profile_event_performance --no-run --message-format=json
```

Each retained test EXE ran `--skip profile_event_hot_path --test-threads=1`.
Measurement runs use
`--ignored --exact profile_event_hot_path --nocapture --test-threads=1`.
The host PowerShell parser/automatic-variable validator executes all scratch
scripts. Formatting, strict production/test lint and an isolated debug build
passed with these exact commands; no broad unit suite or native run followed:

```powershell
cargo fmt --all --check
cargo clippy --locked --offline -p turbofish-deluxe --lib --bins --test profile_event_performance -- -D warnings
cargo build --locked --offline -p turbofish-deluxe --bin turbofish-deluxe
```

The existing workflow checker also passed required-file, Markdown link/anchor,
allowlist/exclusion and whitespace checks. It does not execute gameplay.

Initial harness preparation failed on sha2's unsupported whole-array `LowerHex`
formatting, then on Tank1's synthetic guppy-shop admission. Source reinspection
resolved both before the valid baseline; private failed logs and the second
attempt's test EXE remain retained. No production bug is inferred from those
fixture failures. The unrelated malformed-save diagnostic at `src/cli.rs:1247`
was excluded and remains out of scope.

No native game, desktop/input/audio experiment, live-save read or retail
installation access occurred. These offline call timings cannot establish FPS,
native playability, audible fidelity or human feel.
