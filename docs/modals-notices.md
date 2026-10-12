# Quit and information modals

Actionables#305 owns `src/notice_modal.rs`, `src/app/notice_runtime.rs` and
`tests/notice_modal.rs` under the [modal contract](modal-batch-303-307.md).
Parent integration and final acceptance remain root-owned. This report records
the original Rust design before implementation and the bounded source checks
on2026-10-11, America/Phoenix.

## Design and source boundary

Start checkout: clean detached `4a63144`. Read-only W1 verified clean at
`f919b3c241cfd611c547f1653fb3514f500f761b`. These functional facts are
**secondary-source-derived**, with the [existing provenance](provenance.md#runtime-sources)
study/license limits:

| Fact | Pinned W1 path and symbol |
|---|---|
| Quit has a named affirmative control and Cancel | `source/WinFish/WinFishApp.cpp`, `DoQuitDialog:1904-1907`; `source/SexyAppFramework/Dialog.cpp`, constructor `45-65` |
| Quit and information dismissal have separate outcomes | `source/WinFish/WinFishApp.cpp`, `ButtonDepress:732,863-874` |
| Dialogs use locally supplied dialog/button artwork | `source/WinFish/WinFishApp.cpp`, `NewDialog:693-706` |
| Dialog title/body/button fonts and content insets are distinct | `source/WinFish/WinFishCommon.cpp`, `DefaultDialogSettings:38-51` |

Current production `src/app.rs:5554-5561,5775-5781,5838-5851` uses press-only
Quit and generic information panels. `6644-6647` owns save-before-exit; it must
remain the real transaction authority, with a failed write returned to the
modal instead of exiting the runtime.

The Rust model owns an opening identity, typed lock identity, focus/capture,
pending confirmation and original view geometry on the 640x480 canvas. Caller
supplies all informational body/guidance/errors; lock messages use existing
project-authored guidance only when the caller requests that fallback. Quit
defaults to Cancel focus. Information has one acknowledgment control. Escape
cancels Quit and dismisses information once. Focus loss, covering modals and
opening frames retire capture and require neutral input before rearming.

Only a matching pending Quit request can accept save feedback. Failure keeps
guidance and exact error visible, disables accidental repeated confirmation
until neutral input, and permits a new request with a distinct attempt identity.
Success closes once. Pending requests disable every control and cancellation;
stale/duplicate feedback changes nothing. These are explicit project control
and transaction-preservation rules, not recovered retail behavior.

Panel/text/button bounds belong to the view. The adapter draws through
root-owned `app::modal_runtime` helpers using existing owned images/fonts; it
does not poll native events, drain text, draw another dim layer, save, change
profiles, choose modes, or own clocks/resources. No C++ bodies, extracted prose,
assets or raw derived output enter Git. Exact retail layout/input/feel/audio
and human gameplay remain unverified.

## Parent API and joins

Pure model: `NoticeModal::{quit(id, guidance), information(id, title, body,
guidance), locked_mode(id, mode, reason, guidance)}`. Runtime mirrors those
constructors as `NoticeRuntime::{quit, information, locked_mode}` and exposes:

- `input(&ModalInput) -> NoticeFrame { consumed, intent }`; closing frames are
  consumed. It never polls/drains native input. Provide a neutral focused frame
  after opening/focus loss before accepting new actions.
- `is_open()`, `identity()`, `view() -> Option<NoticeView>` and
  `draw(&Presentation)`; view owns all hit/text/control geometry.
- `cancel_capture()` when covered; `show_error(NoticeIdentity, error) -> bool`
  displays an exact caller diagnostic only on the matching open notice.
- `quit_finished(NoticeRequest, Result<(), String>) -> bool` accepts only the
  matching pending save. Caller's fresh opening ID plus model attempt ID prevent
  stale/duplicate feedback. No pending action may be dismissed.

`NoticeIntent::{Quit(NoticeRequest), Cancel(NoticeIdentity),
Dismiss(NoticeIdentity)}` retains opening and typed lock identity. Cancel and
Dismiss require no transaction. Locked-mode dismissal carries the original
`Mode`/`LockReason` but never requests entry to that mode. A new failed-save retry
has a new request attempt; it must use current caller-owned Profiles authority.

Root must register the model/runtime, normalize one `ModalInput` for the active
modal, consume opening/closing frames, hold the underlying scene and clear
queued input/backlog on return. Root creates each notice with a fresh opening
identity and routes Quit, locked-mode guidance and actual caller errors.
Root performs the existing save-before-exit transaction, returns its actual
result to the exact emitted request and exits only after successful publication.
Excluded saver callers remain unchanged.

## Verification and retained failures

Supporting checkout is `9d29e1e622f79909ae81bf02b2d4f2e842e08245` plus the
three owned Rust files below and the contract-permitted temporary registration
of only `notice_modal` and `notice_runtime`. Runtime registration temporarily
allows dead code because parent consumers do not exist yet; no owned lint is
suppressed. That exact registration is restored byte-for-byte before staging.
Rust1.95.0 (`59807616e`, MSVC), Cargo1.95.0 (`f2d3ce0bd`), edition2024,
default features, debug/test profile, locked/offline dependencies, bounded
`-j 2` and private target `.scratch/notices-305/target`. Read-only libopenmpt0.8.9
package supplies the import library and process-local DLL search path.

| Input | SHA-256 |
|---|---|
| `src/notice_modal.rs` | `20A00D489818438E2A3F6A43B39A9EFF198F02E5EEFA47A5B3791567DA3E8C5D` |
| `src/app/notice_runtime.rs` | `F19B15A2F21C660F14A4F2BAF252C56D21B86DF8D986340D85B96620BDACE3A5` |
| `tests/notice_modal.rs` | `74214BDB0125CB3BC945C632D01BC17F5F9CFAF486B3E3B9DEF3ADDFDDA3CA40` |
| `Cargo.lock` | `1643FD7BE00AF38DB3DA764BE8E0C03CFC427EE368DCAAB7EF17A49B187F153F` |

Exact completed commands:

```text
rustfmt --edition 2024 src/notice_modal.rs src/app/notice_runtime.rs tests/notice_modal.rs
rustfmt --edition 2024 --check src/notice_modal.rs src/app/notice_runtime.rs tests/notice_modal.rs
cargo test --locked --offline -j 2 --test notice_modal
cargo test --locked --offline -j 2 --lib notice_runtime_consumes_opening_and_closing_frames
cargo clippy --locked --offline -j 2 --lib --bin turbofish-deluxe --test notice_modal -- -D warnings
cargo build --locked --offline -j 2
pwsh -NoProfile -File scripts/Test-Workflow.ps1
```

Formatting passed. Eleven offline notice tests and one runtime closing-frame
test passed;661 unrelated unit tests were filtered. Tests supply deterministic
`ModalInput` sequences and success/failure feedback directly, with no RNG,
clock, asset load or persistence write. These prove the explicit model/control
contract; they do not prove the caller's actual save transaction. Workflow
passed35 required files/103 documents/1073 links/362 anchors/45 included and37
excluded paths. Private logs are under `.scratch/notices-305/logs`.

Strict affected-target lint and locked/offline debug build passed. Supporting
EXE SHA-256 is `2F23AB1F0345912FE6A0CFF9E90AFBEA2C646B42E3F36531B964909878B4104C`.
It was built with temporary registration and was not launched. No Rust
compile/test failure observed. An initial read-only bounded-range helper call
used unsupported `StartLine`/`EndLine` names; corrected to `Range` before source
edits or checks. No `rust-toolchain.toml` exists in this supporting checkout;
actual installed toolchain above is the verified supporting identity.

| Claim | Evidence status and limit |
|---|---|
| N1: old Quit fires on press; generic notices have no acknowledgment | Strongly source-supported at the starting checkout's app paths above; historical after root joins |
| N2: distinct Quit/Cancel and information roles use owned dialog/button art | Strongly source-supported, secondary-source-derived at the clean W1 pin; no retail observation |
| N3: new capture/focus/cancel/identity/feedback/layout/closing-frame rules | Reproduced offline on the source hashes and explicit contract above |
| N4: production save/input/draw/modal-priority/clock joins | Unknown until root joins and verifies actual callers |
| N5: visible/native/retail/audio/feel and ordinary gameplay | Unperformed; human-owned acceptance remains separate |

No native launch/input/audio, real/frozen/earned save access, original
installation, service/port/tool install or benchmark/saver work occurred.

Actionables#305 claimed through the ordinary flow; Researching findings/readiness
were reconciled, then Ready and In progress. No tracking failure occurred.
Done remains gated on root's verified production joins.

## Source handoff and cleanup

Commit only the four owned maintained paths. Fetch/reconcile `origin/main` and
push `HEAD:main` without force under the validated task-specific script. Source
commit and final pushed head are reported by the worker; source hashes above
identify the checked implementation even before parent registration. Root can
register it at the normal module declarations, then route the real callers and
run the affected joined checks. This report is source completion, not full
modal/runtime acceptance or Actionables completion.

Temporary registration is restored before reconciliation/staging. Ignored
scratch retains validated scripts, registration backups, private Cargo target
and check logs. All owned check processes exited. No native runtime process or
external resource was started. No fixture/profile/save/configuration cleanup
is needed because none was accessed or changed. The Actionables claim remains
with this chat while root checks and joins the handoff; status is In progress.
