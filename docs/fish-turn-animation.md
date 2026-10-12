# Adventure guppy turnaround

Actionable#310 addresses the human's 2026-10-12 report that Adventure guppy
turnaround timing feels off. The installed binary confirms that the current
turn clock already matches; the actual Rust sprite mirror switches too early.
This is a bounded binary/source comparison and offline correction. Human
comparison of the corrected animation remains unperformed.

## Target and evidence identity

Freshly checked owned Steam App3320/build250752, English v1.1 x86:

- Installed `InsaniquariumDeluxe.exe` SHA-256
  `F54C2C6EE54B00AE6DA7F4BDE15FEB90D0A867ABA6B4151B0277C1F81DA3FF66`.
- Its already registered [PB05 payload](behavior-contract.md#primary-binary-findings)
  SHA-256 `312021E970ABD8AFE417D24E99FF75AC8379EF2A93BC1E04D52D4F3D7F070BB1`.
- Ghidra12.1.4, x86 little-endian32, image base`00400000`; existing identified
  project opened with `-process embedded-popcapgame.exe -readOnly -noanalysis`.
  No original executable was launched or used by the Rust runtime.
- Raw query scripts/logs and instruction/decompiler exports remain ignored in
  `.scratch/turn-animation-20261012`; the existing database remains private.
  Animation instruction receipt SHA-256
  `7BF4318ED14BD4A242CA23C51D4D80BFAAAB1C4047A8454A51B1068D8E451BCF`;
  draw instruction receipt SHA-256
  `80286F67CA300DB203DF4878CA1546E5917A5EE04602BD5DFA487B35E6970BF2`.

Pinned W1`f919b3c241cfd611c547f1653fb3514f500f761b`
`source/WinFish/Fish.cpp:550-575,1355-1397` supplies names and the initial
crosswalk under the existing [provenance](provenance.md#runtime-sources)
boundary. The material clock/mirror rules below were independently checked
in the installed payload's instructions. No upstream bodies or raw output are
copied into maintained files.

## Turn frame clock and facing

| Claim | Evidence class and scope | Exact support |
|---|---|---|
| GT1: a strict nonzero velocity sign change sets the signed timer to20 or-20, then immediately advances it toward zero | Binary-derived, ordinary Fish | Table`00597d54+164`→`004de840`; instructions`004de861-004de8d8`, retained/current velocity at`+188/+168`, timer`+1c4` |
| GT2: turn frames are`0,0,1,1,2,2,3,3,4,4,5,5,6,6,7,7,8,8,9`; the20th update returns to eating/swimming | Binary-derived, uninterrupted turn | Instructions`004de8de-004de96c`, frame`+1c0`; signed division truncates toward zero. Rust's existing sequence matches |
| GT3: keep the starting sprite mirror through every nonzero turn state; switch at timer zero | Binary-derived, ordinary forward guppy | Table`00597d54+68`→`004e4a90`; `004e4b4e-004e4b7c` and`004e4bcf-004e4be5` pass positive timer→`!byte+f5`, negative timer→`byte+f5` to virtual`+14c` |
| GT4: the zero-timer draw uses negative velocity directly, otherwise retained velocity when current velocity truncates to zero | Binary-derived, ordinary forward guppy | `004e4b80-004e4bcb` reads`+168/+188` and converts current velocity with`00564160` |
| GT5: existing Rust flips the guppy sheet at the start of the turn | Reproduced offline/source-supported at clean main`8992664` | Both guppy update paths assign durable facing from current velocity. Actual base/additive draw calls consume that field; three new regressions fail against that original projection |

An uninterrupted turn has19 visible update states. At the configured28ms
period ([PB06](behavior-contract.md#primary-binary-findings)), that corresponds
to532ms of turn-pose display. Frames0-8 each hold two updates; frame9 holds
one. The first displayed state already has timer magnitude19. This is nominal
arithmetic, not a measured retail wall-clock duration. A new sign reversal can
restart the sequence. There is no evidence here for speeding/slowing its clock.

The stored facing flag and the turn sheet's mirror serve different purposes:
the turn artwork itself changes silhouette across its frames. Mirroring it to
the new swimming direction before the sequence ends produces the wrong visual
transition even though the frame timer is correct.

## Correction and verification

`Fish::sprite_facing_right` derives the live sprite mirror from the recovered
turn/velocity fields. Both actual guppy base and additive overlay draw calls use
it. Durable state, save schema, animation clock, movement and other fish/modes
are unchanged. Rendering a saved or paused turn also uses its existing timer
without needing another simulation tick.

Three focused regressions cover both turn directions over all19 visible frames,
completion facing/frame, and zero-speed retained-direction cases. Expected
values come from GT1-GT4, not the new implementation. Before correction, the
projection returned the same stored field as the original renderer and all
three tests failed with wrong mirror values; those red failures are retained.
The existing tick-level turn/eat/growth regression is also retained.

Final source starts from clean main`8992664713d5da1f0a4fb4d4473f02e7e16f1f61`
with only `src/sim.rs` and `src/app.rs` changed among build inputs. The sealed
122-input digest is
`b9e404d93d74653b0f0c31f887fb9eff9118c378305a7ada1ce2b1bc86fa973f`;
Cargo.lock SHA-256 is
`1643FD7BE00AF38DB3DA764BE8E0C03CFC427EE368DCAAB7EF17A49B187F153F`.
Rust/Cargo1.95.0, default features, debug MSVC build; tests use seed21 and
explicit animation updates, with no wall-clock or external save fixture.
The supporting executable SHA-256 is
`af575ef476484521a69ba6e712a39afe001a9fee28791ea716ec076e12b2a5db`.
Its private build receipt is `.scratch/turn-animation-20261012/final-build.local.json`,
ID`4c972f7fc65f7a93a6dc736d2a570f9a09fb1a949299dc210925611e87e22457`,
receipt SHA-256
`524aab27dbf5f2c474538b6eb70b666b6832a8c2c7bb707dec379c76acde7aaa`.
The maintained identity helper checked source inputs before/after the successful
build; five verified cached native DLLs and their notices were staged only in
the isolated supporting directory.

| Executed check | Result |
|---|---|
| `cargo test --locked --offline -p turbofish-deluxe --lib binary_guppy_` | Original projection: three expected assertion failures. Corrected projection:3/3 passed |
| Same command with `source_turn_eat_and_growth_pulses_expose_render_state` |1/1 passed |
| `cargo fmt --all -- --check` | Initial new assertion layout failed; layout corrected, final check passed |
| `cargo clippy --locked --offline -p turbofish-deluxe --lib --bin turbofish-deluxe --tests -- -D warnings` | Passed; rerun after the local formatting correction |
| `cargo build --locked --offline -p turbofish-deluxe --bin turbofish-deluxe` with isolated `CARGO_TARGET_DIR` | Passed and associated with unchanged final source inputs |

All PowerShell experiments passed parser/automatic-variable validation through
the host wrapper before execution. Commands and tail logs remain private in
the named task directory. Workflow and whitespace checks passed; all122 source
inputs and139 artifact/source checks read back with zero failures. Actionable#310
is Done version10 with qualifying Passed validation and its claim released,
read back after completion.

Original/native gameplay, rendered-pixel comparison, effective retail cadence,
human feel and other species/Virtual Tank animation are outside this check.
No human game process, original/project live save or normal-launch executable
is accessed or replaced by this validation. The supporting build stays isolated.
