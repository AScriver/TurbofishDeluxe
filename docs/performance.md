# Offline object-search performance

Actionable **#289** measured the current Rust simulation before changing its
object searches. These checks are offline and use deterministic, disposable
in-memory Boards. They do not establish rendering speed, FPS, original-game
performance, native acceptance, or human feel.

## Measurement contract

The ignored `sim::search_performance::a289_*` tests use seed `0xa289`, nine
samples, and the release profile. A normal fixture has 20 guppies, 9 ordinary
pellets, 16 coins and 2 notes; the crowded fixture has 200 guppies, 9 ordinary
pellets, 256 coins and 32 notes. Each uses a Tank3-1 Board with Stinky and Meryl. The
fixtures pass `AdventureState::validate` before use; none is earned progression.
Nine is the maximum player food-quantity upgrade, not an asserted limit for
every pet-produced object or every mode.

The query check repeats the actual Stinky target helper 100,000 times per sample.
The Board check times 4,096 first updates per sample, cloning fresh batches
outside the timer so every measured tick has the full starting population.
Input, complete output Board and ordered-event SHA-256 values make the before
and after state comparison explicit; an untimed 32-update continuation also
checks the complete state and event sequence. `black_box` keeps the work observable.
There are no timing assertions: host load and scheduling can affect results.
Query fixtures use 20 fish in both cases because that helper reads only Stinky,
coins and notes. Board fixtures exercise hungry-guppy food selection, pet updates,
coin handling and the rest of the ordinary tick. They do not cover every prey,
combat or mode workload.

Run from the repository root after the existing native-library preflight. Use
the same compiler, locked dependencies, release profile and host for both
versions:

```powershell
cargo test --locked --offline --release -p turbofish-deluxe --lib sim::search_performance::a289_ -- --ignored --nocapture --test-threads=1
```

The project's runtime helper supplies the required `OPENMPT_LIB_DIR` and runtime
DLL path; the command itself launches no game window. Set `CARGO_TARGET_DIR` to
an isolated build directory when a human session owns the normal game build.
Default unit runs leave the two measurement tests ignored.

## Query invariants

At baseline `266cd75c84bfab22539706a7b3e5676a02014a39`,
`AdventureState::stinky_target_x` allocates coin/note views, sorts them by
registration ID, then scans for the strict nearest candidate. Coin coordinates
truncate to integer widget positions; each double center delta then truncates
toward zero before squaring. [PB10](behavior-contract.md#primary-binary-findings)
owns the installed metric correction; [SK2–SK6](behavior-contract.md#stinky-contract)
own the update/contact qualifications. The existing project adaptation merges
coins and notes by ID; it does not prove original cross-list ordering.

Any optimization must retain that current merge order for equal distances,
exclude claimed coins and bombs, reserve stars for a live Starcatcher, and keep
notes eligible but uncollectible. Contact continues to walk the original coin
vector and credit at most one overlap. Update order, RNG consumption, saves,
and the 28ms simulation step stay unchanged. A persistent spatial cache is not
justified before measurements show a material need.

## Results

Observed on 2026-10-11, with one baseline and one optimized executable run,
each containing nine samples. Times below are nanoseconds per query or tick:

| Case | Before min / median / max | After min / median / max | Median reduction |
|---|---:|---:|---:|
| Stinky query, normal | 229.0 / 243.3 / 264.8 | 63.6 / 69.1 / 81.1 | 71.6% |
| Stinky query, crowded | 2568.0 / 2798.7 / 3116.3 | 986.0 / 1028.7 / 1289.1 | 63.2% |
| Whole Board tick, normal | 1497.1 / 1865.0 / 2942.9 | 1208.1 / 1305.8 / 1595.1 | 30.0% |
| Whole Board tick, crowded | 8785.8 / 10575.1 / 11990.8 | 6962.8 / 8431.5 / 9588.9 | 20.3% |

The crowded query median improved from 2.799 to 1.029 microseconds (2.72x).
The crowded Board median improved from 10.575 to 8.432 microseconds. Whole-tick
sample ranges overlap, and timings remain subject to host load and compiler
layout; these results are observations for the stated fixtures, not a guaranteed
game-wide gain. Both whole-tick baselines are far below the 28ms simulation step.
There is no measured need for a spatial index in these fixtures and no FPS claim.

The production change replaces the temporary candidate vector and stable ID
sort with a single minimum keyed by `(truncated squared distance, registration
ID)`. The initial key `(100_000_000, 0)` keeps the old strict distance cutoff;
strict key comparison retains first occurrence for duplicate IDs. Source review
establishes one temporary candidate vector and one sort in the old helper,
neither in the new helper; individual allocator events were not instrumented.
The helper runs once per eligible Stinky target update. Food/prey searches,
contact, Board phases and RNG calls are unchanged. No new dependency, saved
field, persistent cache or upstream implementation was introduced.

## Measured identities and equivalence

Both runs use parent `266cd75c84bfab22539706a7b3e5676a02014a39` plus their
recorded dirty source. Only the benchmark hook/fixture and records were added
at baseline; optimized source also includes the target change and two focused
regressions. Other production sources are unchanged from that parent.

Host: AMD Ryzen 7 9800X3D, 8 cores / 16 logical processors, Windows x64 MSVC.
Compiler: Rust 1.95.0, commit `59807616e1fa2540724bfbac14d7976d7e4a3860`,
LLVM 22.1.2. Release profile retains thin LTO and one codegen unit. Native
dependency is the existing libopenmpt 0.8.9 package. Both runs use locked,
offline dependencies, serial test execution and isolated
`CARGO_TARGET_DIR=.scratch/performance/a289/target`; neither launches a game.

| Identity | SHA-256 |
|---|---|
| Baseline `src/sim.rs` | `EDA34BA3F5B0AF4577809026E3BE05EE725A44966F529AF1710FA2C524F4CCF2` |
| Optimized `src/sim.rs` | `DFD5B25F7DD5811E7202DC5F63928D0D29726D601D20547EFC6A5303301F087E` |
| Unchanged benchmark fixture | `D0C016BAD984EADB4D80581E18812E210B357BCE46B246F857B17324BCBD0D51` |
| Unchanged `Cargo.lock` | `1643FD7BE00AF38DB3DA764BE8E0C03CFC427EE368DCAAB7EF17A49B187F153F` |
| Baseline release test EXE | `E60C36E675A9D9F4BA08D76916597608ACF9AD5FD170B36134AF6B396C9DF402` |
| Optimized release test EXE | `42AB199889A2ADC9D1ADA71410BAEF54550003F5D944993BF8A3F80F88A29648` |
| Isolated optimized debug game EXE | `4D01ADF0E3AC436AD4229A5226464A8A6C0D78054410DB4D3D466BE31390FD28` |
| Private `baseline-03.log` | `33082155FA37A402FD36489648ACFB4E537DC15BA3543364913706A3E8193736` |
| Private `optimized-01.log` | `5B88B447C326B1890688F11A1AF397BE1B1135D3F6BA4B0A45C60DAB7B252FAD` |

The validated private comparison script reads both retained logs and requires
identical fixture-source hashes and all four complete fixture records. It
passed. Each Board record's input, first-tick output, ordered events,
32-tick output and event trace matched exactly. Shared output digests are:

| Result | Normal SHA-256 | Crowded SHA-256 |
|---|---|---|
| First-tick Board | `481331f328c641cb2dadb55f1427cfbbb92258f49c7884ae8e55d26afeb6de8a` | `43df8612b6613a09533cb9c3fe1c703a2c16abee00ab840cfef77a6ef0c5a1c3` |
| First-tick events | `4f53cda18c2baa0c0354bb5f9a3ecbe5ed12ab4d8e11ba873c2f11161202b945` | `d6eb07ee70af445f7a9242b78bdf7face760f3e8e8d8a1ac1f49187290cc2e9a` |
| 32-tick Board | `571ccfcd3e952dd3e45d0d662ebdd2d225b78ac6d56c47c44f3bd5216fd29135` | `33912b7dcef9b42375a675939919fefc9ff83852c96968a18653c1ac37d2e6a6` |
| 32-tick events | `057f0f27f8f1350529ddaf849f0a80379d526799aca67544a0293fa574e82137` | `8d1f05f7b2574aa269d5c2192559773e610eecba778ec6c04868a229656938da` |

Raw logs, original input digests, comparison output and validated machine-local
scripts remain private under `.scratch/performance/a289`. Matching these
outputs establishes optimization equivalence for these fixtures, not new retail
fidelity or exhaustive gameplay coverage.

## Focused checks and retained failures

Formatting, strict production Clippy, the isolated locked/offline debug build,
15 affected simulation regressions and both optimized measurement cases passed.
The 15 regressions cover existing Stinky movement/contact/collection/save-state
rules, the installed center metric, bomb exclusion and the two new tie/subpixel
and changing-eligibility cases. Fixture admission passed before the baseline.
The new fixture and measurement module is test-only; normal game binaries do
not include it. Successful commands, besides the release measurement above:

```powershell
cargo fmt --all -- --check
cargo clippy --locked --offline -p turbofish-deluxe --lib --bin turbofish-deluxe -- -D warnings
cargo test --locked --offline -p turbofish-deluxe --lib installed_payload_nearest_center
cargo test --locked --offline -p turbofish-deluxe --lib collectors_ignore_bomb
cargo build --locked --offline -p turbofish-deluxe --bin turbofish-deluxe
```

The broader `cargo test --locked --offline -p turbofish-deluxe --lib stinky`
passed its 13 affected simulation cases but also selected the unrelated
`cli::current_twenty_one_tests::current_save_requires_present_stinky_combat_identity_field`.
That case panicked at `src/cli.rs:1247`: the required-field error-label match
ends at version 35, while current format is 37. The decoder is byte-unchanged
from the parent; the failure occurs before target selection. Record this as a
separate malformed-save diagnostic follow-up, not a passed broad filter or a
search regression. Private `verify-01.log` retains the failure; only the remaining
metric/bomb/build commands ran in `verify-remaining-01.log`, with no passing-case
repeat or full suite. Current37/profile3, real saves and human launch artifacts
were not changed or inspected.

Preparation failures also remain recorded: the first harness compilation failed
on the pinned SHA API's missing `LowerHex` implementation, then a stage1-2
fixture failed upgrade admission. Source review also required Meryl in the
note-producing roster. The Tank3 replacement initially missed its coupled
Grubber/food-upgrade gate. After correcting fixture-only preparation, admission
and `baseline-03` passed. None of those failures yielded timing or established
a gameplay bug. Native render/audio acceptance and human feel remain outside
this offline increment.
