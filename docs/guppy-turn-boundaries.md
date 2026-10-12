# Adventure guppy boundary turn correction

Actionable#319 follows the human's request to fix the remaining turn issue.
Earlier food and constant-speed fixtures stayed away from walls. Their fixed
clock findings remain valid; they did not test boundary handling.

## Evidence ledger before implementation

Baseline is clean`02a295112ba46b9693eb88ff89c74cbfc630fc48` before the new
regressions. The [E128 target](fish-turn-animation.md#target-and-evidence-identity)
identifies Steam250752/PB05. Existing Update/animation export hashes match
`286F5042B5FD38E9CB756D731A56D7A2052C6413F98074027E33671B494739EC` and
`7BF4318ED14BD4A242CA23C51D4D80BFAAAB1C4047A8454A51B1068D8E451BCF`.
A fresh-context Investigator independently inspected the static boundary tail;
it launched no game and edited no tracked files. Root owns source and tests.

| ID | Claim and class | Support / limit |
|---|---|---|
| GB1 | Strongly binary-supported: common Fish tail clamps coordinates before animation | `004f76d4-004f7726`, conditional lower-Y join`004f79bc-004f79d7`; ordinary default bounds use X10..540, Y95..370 |
| GB2 | Strongly binary-supported: ease outward VX by0.1 before animation; no common-tail bounce | `004f79d9-004f7a42`: X>535 and VX>0.1 subtract0.1; X<15 and VX<-0.1 add0.1 |
| GB3 | Strongly binary-supported: animation sees earlier AI sign changes, then position integrates; final overshoot is possible | `004f7a4c-004f7a56`, integration`004f7a80-004f7aa6`; actual retail playback remains unmeasured |
| GB4 | Reproduced baseline defect: ordinary Rust updates animation, then clamps final positions and reverses VX/VY at bounds | `AdventureState::update_fish`; red boundary test publishes the wrong mirror on a Swim frame at the right wall. Separate red assertions find538.2 rather than538 after reclamping, and95 rather than94.75 at the lower Y bound |

Pinned W1`f919b3c241cfd611c547f1653fb3514f500f761b`,
`source/WinFish/Fish.cpp:457-533`, supplies secondary names/crosswalk only.
Raw source and instruction output remain private. No new source/license is used.

## Correction contract

Use one common motion tail for ordinary and warded guppies: existing
pre-integration bounds, horizontal easing, fixed animation update, facing
projection and final integration. Remove the ordinary path's extra bounce.
Preserve the signed20-count animation, both sprite layers, coin-before-motion
ordering and save schemas. This corrects a binary mismatch rather than scaling
turn duration by speed. Native feel acceptance remains separate from tests.

Three full-update regression functions cover both horizontal edges, reclamping
an overshoot before opposite-sign movement, and both vertical bounds. The
reversal fixture supplies opposite VX/previous-VX signs; it does not test every
AI branch that can choose a reversal. Expected results come from GB1-GB3, not
from the Rust implementation.

## Executed baseline reproduction

At02a2951 plus the three new regressions, the maintained Unit filter
`binary_guppy_boundary_` executes all three and fails all three as expected.
The right-wall case publishes `sprite_facing_right=false` instead oftrue while
still in Swim. The other failures match GB4 above. This is executed simulation;
no rendered current-build claim follows. Private red log SHA-256:
`9734CC1337A32A0C96CFF58199A234699F35DBDCFE28C3602321AD12DAFD9000`.
PowerShell parser/automatic-variable checks pass. The isolated Cargo target is
`.scratch/turn-animation-20261012/boundary-target`; earlier retained builds and
normal-launch artifacts are not overwritten.

## Correction and verification

Original Rust `AdventureState::advance_fish_motion` now owns the common tail.
The warded path retains its existing operation order; the ordinary path uses
the same independently checked order and no longer introduces a final bounce.
`advance_animation`, sprite projection, saved fields and schemas are unchanged.

The six `binary_guppy_` checks pass, including the three new regressions. Three
`warded_` matches and the existing turn/eat/growth pulse check also pass. Since
the ordinary movement tail serves consumers throughout the library, verification
expanded once to the full library:688 passed, six ignored, one failed. That
failure is `adventure::tests::time_trial_final_candidate_maxes_egg_without_adventure_victory`
at `src/adventure.rs:4352`, where `start_time_trial` rejects the fixture before
any fish update. A disposable Git archive of clean02a2951 independently fails
the exact same assertion. It is an existing failure, left unchanged; the full
library gate is not reported as passing. Private logs retain both failures.

The affected current live ward/published-position persistence case passes.
Formatting, strict affected lib/bin/test lint and a locked/offline isolated
debug build pass. Commands from the validated private runners:

```powershell
scripts/Test-Runtime.ps1 -Check Unit -TestFilter binary_guppy_
scripts/Test-Runtime.ps1 -Check Unit -TestFilter warded_
scripts/Test-Runtime.ps1 -Check Unit -TestFilter source_turn_eat_and_growth_pulses_expose_render_state
scripts/Test-Runtime.ps1 -Check Format
cargo test --locked --offline -p turbofish-deluxe --lib
cargo test --locked --offline -p turbofish-deluxe --test persistence current_live_ward_and_published_position_reload_every_next_update
cargo clippy --locked --offline -p turbofish-deluxe --lib --bin turbofish-deluxe --tests -- -D warnings
cargo build --locked --offline -p turbofish-deluxe --bin turbofish-deluxe
```

`CARGO_TARGET_DIR` selects the new private `boundary-target`; the baseline
control uses a separate `boundary-baseline-target`. All scripts run through
`Invoke-CodexPowerShell.ps1 -Execute` with parser and automatic-variable checks.
The full-library failure stopped the first finish script before lint/build;
the corrected runner preserves that result and continues only the remaining
checks. Baseline archive SHA-256:
`DE7654F26CA2132D9B911A4353DF508478B703854EC7B0D9C38F0FBD5B911506`.

Supporting build uses02a2951 plus the reviewed `src/sim.rs` motion/helper/test
diff. Rust1.95 MSVC debug configuration, no optional features and no runtime
seed/input apply because no game was launched. Its immutable private receipt is
`.scratch/turn-animation-20261012/boundary-build.local.json`:

| Identity | Value |
|---|---|
| Source inputs |122; digest `e68a4ef14d6a8cbf86fd500af965be242e0b02832807eebc942acbbb21ed9bdd` |
| `src/sim.rs` SHA-256 | `B08D37208718231AF40F589DCFFE25AF7A81121951EA35124382BDDD495E4B98` |
| Executable SHA-256 | `6d86cc5120da070a0f7fc19a936936cd8ddc28fe69f031e90bf03dc04ea31654` |
| Build record ID | `976dc17a08cd10be7c1752b758b84e6a715d0108c729583d7d7982de459eee7f` |
| Receipt SHA-256 | `537d6aeff4b53a599630e83f2f5b3e51cc70f02cad8aa3fc38ae5fd5cf9187fc` |

The maintained identity helper checks unchanged source inputs across the
build. All122 source inputs and139 identity checks match on readback. The
workflow passes114 documents,1149 local links,416 heading anchors,45 included
and37 excluded paths; whitespace passes. `Verify-Boundary.ps1` runs the
maintained `Test-Workflow.ps1`, identity helper and `git diff --check` through
the host validator. Actionable#319 is Done version9 with qualifying Passed
validation and released claim, read back; the boundary checks qualify while
the unrelated full-library failure remains explicit. Earlier captures remain
historical E128 observations;
they do not validate this changed executable. Original/default live saves,
the human game, retained builds and normal-launch artifacts are untouched.

This correction removes the reproduced wall mismatch. Motion while early
authored turn poses still resemble the starting profile remains supported by
the earlier original clock/pose evidence. Current-build normal-speed wall
comparison, retail playback and human timing acceptance remain pending.
