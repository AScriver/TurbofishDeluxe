# Runtime requirements

Authoritative product scope, recovered from the runtime prompt supplied in the earlier project conversation on 2026-10-08. The user authorized continuing implementation on 2026-10-08 through the full requirements, automatically selecting and implementing the next milestone after each validation. The guppy/food/coin loop is an internal milestone, not a stopping point. Continue until completion is verified or a concrete blocker requires user input.

## Goal and constraints

Reimplement the full Insaniquarium Deluxe runtime faithfully in Rust. Use Turbofish Deluxe as the project name. Target Windows 11 first, with portability and future extensibility supported by understandable game behavior and practical separation of responsibilities.

Game logic and simulation execute in Rust. General-purpose graphics, audio, and platform libraries are acceptable. The finished program must run independently without launching or embedding the original executable. The original is a reference for analysis and comparison. Players supply their own legitimate game installation; preserve it read-only and use separate project saves.

The MW2/Skate 3/Minecraft Rust mashup is inspiration for recovering and rebuilding a runtime, not a requirement to embed another game or copy its engine architecture.

## Establish the reference

The user's 2026-10-08 clarification makes the currently installed game binaries the source of truth for game details. WinFish is secondary, used as a starting point. Confirm or correct secondary-source mechanics against identified binary evidence and controlled original-game observations. The user also authorized local commits as work progresses; retain coherent verified checkpoints.

- Locate the installation through Steam library metadata and likely installation directories; support an explicit `--game-dir` override. Identify executable version, architecture, SHA-256, and asset inventory. Ask for a path only if discovery fails, while continuing independent work.
- Investigate [WinFish](https://github.com/Vindirect/WinFish), [insaniquarium-port](https://github.com/SaMeiers/insaniquarium-port), and the [mashup inspiration](https://github.com/chasmlol/2010-rust-rewrite-mashup) before duplicating their work. Pin revisions and inspect actual code.
- Check WinFish's reported PopCap.com Windows 1.1 target against the installed executable. Do not infer compatibility from a matching version string. Record differences and unknowns.
- Record provenance and applicable licenses before incorporating or translating code. Separate recovered game code, framework code, and contributor changes. Source-informed work must not be called clean-room after consulting decompiled source. No proprietary game files enter project commits.

## Recover behavior

Use resource inspection, static analysis, disassembly/decompilation, debugger traces, and controlled original-game runs where needed and permitted. Begin with the entry point, resource loading, main loop, screen transitions, and first Adventure tank. Prefer existing format documentation and directly observable behavior before duplicating analysis.

Recover the behavior needed for each playable increment: simulation timing and update order, RNG, fish states and movement, feeding, hunger, growth, money, purchases, progression, input handling, animation, and sound events. Maintain an evidence map linking implemented behavior to a pinned source symbol/path or repeatable original-game observation. Separate observed behavior, source-derived behavior, hypotheses, and unknowns; investigate discrepancies. Invented constants and approximations must never be described as recovered behavior.

Inspect actual asset formats and metadata. Match sprite frames, transparency and masks, offsets, fonts, layout, and audio interpretation where they affect fidelity. Any conversion runs locally against user-supplied assets and is reproducible.

Choose a small Rust architecture after inspecting the game. Separate simulation and state from rendering, audio, input, and file access. Preserve recovered timing and update ordering. Make inputs, time, and RNG controllable for repeatable scenarios. Support future mods or integrations without first building a general-purpose engine.

## Playable milestones

### M1: First real tank interaction loop

- Launch the Rust executable and load the original tank and fish assets.
- Display a correctly animated, moving guppy.
- Click to drop food, have the fish consume it, and collect a genuinely produced coin that updates the balance. Include any growth or other prerequisites required by the original rules; do not substitute a forced coin, debug money, or a scripted mock.

### M2: Complete the first Adventure tank

- Match the original starting state, prices, available purchases, and progression rules.
- Implement feeding, hunger, growth, death, coin production and collection, and the fish or enemies actually required by that stage.
- Complete egg purchases, trigger the correct victory behavior, and advance to the next stage.
- Save and reload progression using separate project data. State original-save compatibility separately; it is not established by project-save support.
- Run at normal gameplay speed without debug money, forced victories, or scripted substitutes.

### M3: Full runtime coverage

Continue through the remaining Adventure content, pets, enemies, upgrades, other modes, Virtual Tank, menus, audio, and persistence present in the reference build. Maintain a behavior-by-behavior compatibility checklist as systems are recovered. M1 and M2 are intermediate milestones; the goal remains the full runtime.

## Acceptance and working style

A milestone requires an actual run of its path. Compilation, a static screenshot, a mock scene, or self-authored tests alone do not establish playability or fidelity.

Capture repeatable original-game scenarios with build identity, starting state, actions, and timing. Compare events, state transitions, timings, and visuals/audio. Recover and control randomness where possible. Until equivalent RNG state is established, compare rules, ordering, and bounded behavior rather than asserting exact replay parity.

Retain regression tests for recovered rules, asset decoding, transitions, and concrete bugs, with independent expected results or explicit specifications. Verify visible gameplay and input alongside simulation. Record working build/run commands and actual execution evidence. If the original or a GUI cannot be exercised, keep those acceptance checks open and state the exact limitation.

Proceed in small integrated increments and make routine implementation decisions autonomously. Parallel work, when useful and authorized, follows the applicable agent ownership and isolation rules. Preserve unrelated changes and reviewable history. Use the current handoff and relevant design notes instead of repeating completed research or the intake interview. Ask only when missing access, an artifact, or a consequential choice materially prevents progress. A blocked subsystem does not stop useful independent work.

Current progress is authoritative in [STATUS](../STATUS.md); workflow and evidence procedures are in [AGENTS](../AGENTS.md), and next-step acceptance is in [DESIGN](DESIGN.md).
