# Gameplay modal frontend

Actionables #306, E125. This bounded assignment owns only
`src/gameplay_modal.rs`, `src/app/gameplay_modal_runtime.rs`,
`tests/gameplay_modal.rs` and this report. Root owns common controls/drawing,
registration, resources and every production input/render/session/save/clock
join under the [modal contract](modal-batch-303-307.md). Source completion is
separate from production acceptance. Ordinary gameplay remains human-owned.

## Design and evidence

Baseline: clean worker checkout `4a63144`, no uncommitted maintained source,
branch `ascriver/gameplay-modals-306`. All observations below are static;
there was no native launch or original-game observation.

| ID | Evidence and scope | Classification |
|---|---|---|
| G306-1 | Baseline `app.rs:4502-4618,4798-4816,7566-7617,7871-7922` duplicates flat panels and pointer/key geometry for confirmation, rescue, invasion tips and Game Over. | Observed source |
| G306-2 | `adventure.rs:3034-3047,3080-3143,3355-3378`: existing domain actions own rescue, tutorial acknowledgment, confirmation and menu transitions; Game Over admits Continue only for `updates > 30`. | Strongly source-supported, explicit preservation authority |
| G306-3 | `adventure.rs:2525-2596,3330-3378`: Adventure confirmation uses selection capacity; Challenge confirmation opens below three selected pets. Adventure Tank 5 rejects accepted confirmation. These differences must remain. | Strongly source-supported |
| G306-4 | W1 `source/WinFish/WinFishCommon.cpp:38-53` establishes 36-pixel side/bottom content insets and JungleFever header/body/button roles; `WinFishApp.cpp:693-706` creates owned dialog/button art. | Secondary-source-derived |
| G306-5 | W1 `Board.cpp:947-964,1111-1136`, `WinFishApp.cpp:872-886,1562-1566` and `MoneyDialog.cpp:25-57` distinguish tips/rescue/Game Over and delayed buttons. Installed/project corrections and current Rust admission take precedence: no new delay for rescue/tips. | Secondary-source-derived with preserved disagreement |
| G306-6 | [Shared contract](modal-batch-303-307.md#shared-interface-reservation) requires captured release, disabled/stale identity guards and caller-owned transaction feedback. | Explicit contract |

W1 is read-only pin `f919b3c241cfd611c547f1653fb3514f500f761b`; the registered
[provenance boundary](provenance.md#runtime-sources) applies. Framework
`Dialog.cpp:289-307` supplies only the functional equal-width pair versus full
footer arrangement. No C++ source bodies, extracted dialogue, pixels, fonts or
derived output are incorporated. Guidance stays project-authored. Panel sizing
and wrapping are deliberate project layout, not verified retail geometry.

The implementation projects immutable `AdventureSession` phase/admission
into a transient model. Shared controls own capture and keyboard focus. Intent
identity retains mode, stage, selected pets/tip, modal generation and request
serial; parent must revalidate it before applying the existing `Action`. A
pending request disables duplicate dispatch. Rejected transactions keep the
modal open and expose clipped feedback; successful feedback waits for the
authoritative phase change. No independent clock, persistence or game rule is
introduced. Enter defaults to the existing accepted confirmation; cancellation
maps to choosing more pets only for confirmation. Other prompts retain their
required acknowledgment.

## Implementation and parent joins

The pure model exposes `GameplayModalState::from_session(&AdventureSession)`
and `GameplayModal::new(generation, state, button_height)`. Its `view()` owns
panel/title/body/feedback bounds and every `ModalButton`, semantic action,
enabled/pressed/focused flag. The project panel is `(96,110,448,272)` on the
640x480 canvas. Footer side/bottom insets are 36; paired buttons are 180 wide
with a 16-pixel gap. Single-button prompts fill the same 376-wide footer.
Owned-art button height is clamped to 16..40 (nonfinite values use 34), and both
input/drawing use the resulting rectangle. Text helpers wrap/clip within the
provided bounds.

Root-facing adapter is `app::gameplay_modal_runtime::GameplayModalRuntime`:

1. Construct with `Default::default()`.
2. Call `sync(&session, presentation.images["IMAGE_DIALOGBUTTON"].height())`
   before input/drawing on **every** frame, including non-modal phases. Same
   phase preserves capture while admission changes; mode/stage/payload changes,
   observed tick rollback and explicit `clear()` replace the generation.
3. `is_active()` identifies input ownership; `view()` returns the exact draw/hit
   plan. Feed `input(&ModalInput)` once when this modal owns input. Existing Y/N
   aliases may call `request(ConfirmPets)` / `request(ChooseMorePets)`; Enter
   uses shared focused accept. Confirmation cancel requests No; cancel on other
   prompts only retires capture. Higher-priority UI calls `cancel_capture()`.
4. Returned `GameplayModalIntent` retains generation, request serial, complete
   mode/stage/prompt/selection snapshot and semantic action. Immediately before
   dispatch, `action_for(&intent, &session)` revalidates pending identity,
   current phase payload and admission. Only the existing `Continue` and
   `ConfirmPetSelection { accept }` actions are produced. Root executes them
   through its current session/event/save machinery.
5. Return `transaction_result(&intent, Applied)` only after the actual accepted
   transition. If action is refused, return `Rejected(project_authored_error)`;
   this keeps the modal open, shows bounded feedback and allows a fresh request.
   Old generation/request feedback is ignored. Applied feedback leaves controls
   disabled until `sync` observes the real phase change. Re-sync after actions
   and session stepping, before final drawing.
6. Parent draws one dim layer and calls `draw(&presentation)` over retained
   pet-card/board rendering. Root removes the old flat painters and separate
   pointer/Enter/Y/N dispatch for these exact phases. Root owns resource loading
   and missing-art diagnostics; this adapter adds no loader/backend.
7. Call `clear()` on profile/session replacement even when ticks and phase
   happen to match. It keeps generation monotonic. Modal activity must **not**
   create a new global pause or stop existing Game Over session steps: the
   authoritative phase counter must continue to reach `updates > 30`.

Handled phases: Adventure/Challenge pet confirmation, Adventure first-fish
rescue, all three invasion tip kinds in Adventure/Time Trial/Challenge, and
Adventure/Time Trial Game Over. The ordinary pet-selection screen, TT immediate
pet acceptance, Challenge results/loss, hatch, Presto, saver and other UI remain
with their existing owners. Root retains all pause/time, Board/session events,
admission and checkpoint gates. Public adapter methods are `pub(super)` for the
actual parent joins.

## Verification

Executed 2026-10-11 America/Phoenix (2026-10-12 UTC), agent-owned offline run
`check-01` in the worker's ignored `.scratch/gameplay306`. Supporting source
identity is base `9d29e1e` plus exactly the three new owned Rust files below;
no existing source remained modified. Both temporary own-module declarations
in `lib.rs`/`app.rs` restored byte-for-byte in the runner's `finally` block.
The temporary adapter declaration allows dead code only because root has not
joined its callers; no such declaration/allowance is committed by this worker.

| Command | Actual result |
|---|---|
| `cargo fmt --all --check` | Passed after formatting only the three owned files with Rust 2024 rustfmt |
| `cargo test --locked --offline --jobs 2 --test gameplay_modal` | 11 passed; no failures/ignored tests |
| `cargo test --locked --offline --jobs 2 --lib app::gameplay_modal_runtime::tests::` | 3 passed; 661 unrelated cases filtered |
| `cargo clippy --locked --offline --jobs 2 --lib --bin turbofish-deluxe --test gameplay_modal -- -D warnings` | Passed |
| `cargo build --locked --offline --jobs 2 --bin turbofish-deluxe` | Passed, supporting debug build; not a launched/production acceptance bundle |
| Host-validated `scripts/Test-Workflow.ps1` | Passed: 35 required files, 103 documents, 45 allowed/37 excluded path checks |
| `git diff --check` and parent restoration | Passed; only four reserved new maintained paths remained |

Focused checks exercise the independent E125 control/identity contract and
unchanged domain transitions: 30/31 Game Over admission, new press after locked
admission, same-rect release, cross-control/background release, focus/cancel,
confirmation key choices, changed pets/mode/generation, duplicate requests,
rejected/late feedback, bounded layout/capacity text, rescue exactly once,
tutorial acknowledgment without Board replacement/clock advance and Game Over
menu transition without progress changes. Adapter cases verify observed tick
rollback, explicit session replacement, same-phase capture retention and
higher-priority UI interruption.

Toolchain: `rustc 1.95.0 (59807616e 2026-04-14)`,
`x86_64-pc-windows-msvc`, LLVM 22.1.2. Disposable in-memory sessions use seed 42,
explicit fixed steps and synthetic `ModalInput`; no wall clock/game process or
save fixture is used. `CARGO_TARGET_DIR` is private to this worker;
`OPENMPT_LIB_DIR` points to the already supplied read-only 0.8.9 amd64 package.
Full logs, toolchain/configuration and artifact receipts remain ignored.

| Checked input/artifact | SHA-256 |
|---|---|
| `src/gameplay_modal.rs` | `00D485A8170DAFCE57CF35811B824A40C86B8745C6F972FF7C93D84E0E8E1E4B` |
| `src/app/gameplay_modal_runtime.rs` | `950F0BC2FF94218BD06DE56382D00A88C895B0EFB109EDEEFD45F8D0D3A104BA` |
| `tests/gameplay_modal.rs` | `FBC7BAC4A271A46D230EBBFEC4732EF458F96E320E37DE2AEFFFECC1BDC8BDB7` |
| Supporting debug executable | `D9FDFFC2132D0390C8334C4EC662DFA7D3633313456A86DB79806972DE946AB4` |

Root production registration/input/render/resource/event/pause/save joins remain
pending. Human gameplay and native/render/audio/retail visual/feel remain
unperformed; supporting source checks establish no playability claim.

## Failures and cleanup

No compilation, test, lint or build failure occurred. One report-edit patch
failed to match its expected context and changed nothing; the narrower corrected
patch succeeded. Read-only lookups for absent `rust-toolchain.toml`,
`tests/ui.rs` and host `Get-NumberedContent.ps1` produced path errors; actual
toolchain output and existing `tests/ui_primitives.rs` were used instead. No
missing tool was installed or bypassed. Git reported CRLF normalization notices
for temporary registrations; their final byte restoration passed and neither
parent source is staged.

Scoped Actionables ordinary lifecycle succeeded; #306 remains In progress with
this chat's retained claim pending root production acceptance. No bulk
preparation, terminal completion or ownership bypass was used. Shared native
libopenmpt package is read-only. Temporary registrations restored; private target,
logs and supporting unlaunched executable retained for root evidence. No process,
service, real/frozen/earned save, E80/earning06 lineage or original installation
was touched; no cleanup of another worker's resources was performed.
