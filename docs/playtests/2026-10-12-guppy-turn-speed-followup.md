# Adventure guppy turn speed follow-up

## Question and primary evidence

The human suggested that turning might not account for movement speed. This
follow-up checks the turn clock and distance during the turn separately. No
native game was launched, and no save was accessed.

The [E128 binary identity](../fish-turn-animation.md#target-and-evidence-identity)
remains current: fresh readback matches the installed Steam250752 wrapper,
registered PB05 payload and both animation/update instruction digests. No new
source or decompiler export was introduced.

| Claim | Class and scope | Exact support |
|---|---|---|
| GS1: turn duration is independent of velocity magnitude and movement divisor | Binary-derived, uninterrupted ordinary Fish turn | `004de863` loads20; `004de8ae-004de8d8` decrements by one toward zero; `004de8de-004de904` selects frames directly from timer. The nonzero turn branch does not read speed |
| GS2: ordinary swim cadence has a separate speed threshold | Binary-derived, no active turn/eating state | `004de930-004de96c` reads integer velocity magnitude at`+1bc`; values above1 advance the swim counter by2, otherwise1. Update writes that magnitude after `FABS`/conversion, for example`004f73a2-004f73af` |
| GS3: ordinary movement keeps using velocity divided by the fish's movement divisor | Binary-derived, normal non-Speedy guppy | Animation call`004f7a4c-004f7a56` precedes divisor load`004f7a58` and integration`004f7a80-004f7aa6`. Turn frame progression does not rescale that division |

Current clean checkout is`2dd559d474fdbe929100f7354311e02e4e24ec50`.
Its `src/sim.rs` uses the fixed signed20-count turn rule and the separate
swim-speed threshold. The diff from401da21 changes serialization/unused legacy
code and unrelated tests; the active turn/movement routines are unchanged.

## Controlled full-update probe

The private `speed-probe.rs` links the identified retained E128 library used in
the [motion follow-up](2026-10-12-guppy-turn-motion-followup.md#full-update-probe).
Its source digest is
`b9e404d93d74653b0f0c31f887fb9eff9118c378305a7ada1ce2b1bc86fa973f`;
all17 captured supporting artifact checks match. This is retained-build
simulation, not a new native run or complete verification of the current build.

Seed21, one Small guppy at(300,150), hunger500, no food, no purchase/growth/eat
animation, and a seeded previous/current velocity sign change isolate the
clock. Movement-state timers keep velocity constant for20 real
`AdventureState::tick` calls. Eight signed velocities (`0.5,1,2,4` in both
directions) times all three ordinary divisors (`1.6,1.8,2`) give24 cases and480
full updates. Expected frames come from GS1, independently of the Rust code.

All24 cases pass:19 visible Turn states, head-on frame5 on update11, and Swim
on update20. Each position increment also matches velocity/divisor within
0.002 playfield units. Before the head-on frame, the slowest case
(`abs(vx)=0.5`, divisor2) travels2.5 units, and the fastest
(`abs(vx)=4`, divisor1.6) travels25 units. Their turn frame sequences are equal.
The larger displacement explains a stronger motion/pose gap at higher speed.
The test seeds a turn onset; it does not compare natural retail acceleration,
target choice, repeated reversal or actual retail wall-clock playback.

`Probe-Speed.ps1` was parser/automatic-variable validated and executed through
`Invoke-CodexPowerShell.ps1 -Execute`; Rust1.95.0. Private SHA-256 receipts:

- Source: `1D7250513261C43FFBC7FE1678F42D79249498124B420F3DD60374302EA3AA65`.
- Executable: `787854986744EEAC6D98D11801A1C7C6F5AA006773C6F051D7BFB2DC6B9C1D37`.
- Log: `3BDC8033043302653E7D150DC69933672C3E87965E5060645DA7C007C376A432`.

## Conclusion and limits

The user's hypothesis correctly describes the turn clock: faster fish do not
finish turning sooner. The installed binary follows that same rule. Scaling
turn frames by movement speed would therefore change the recovered behavior;
no runtime change is justified by this finding. No new defect or Actionable
was established, and Actionables was not updated. Actual retail motion curves,
rendered speed comparison and human timing acceptance remain unmeasured.
