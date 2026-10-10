# Virtual Tank screensaver

Selector previews are disposable; standalone savers can bank income and narrow VT continuation. [STATUS](../STATUS.md) owns verified build/checkpoint scope; [design](DESIGN.md#screensaver-integration-decisions) owns ownership decisions. Native execution, embedding and retail/visual/audio/human comparison remain not tested.

## Preferences and preview

Selector Saver opens separate project preferences stored explicitly in `screensaver.json` under the selected project save directory. Sound, owned-backdrop rotation and periodic dimming default on; money overlay and power saving default off. Missing auxiliary configuration uses these source defaults; malformed/unsupported configuration reports an error. Preferences do not backfill or change gameplay save format36. On Windows, power saving releases the saver thread's display-awake request so ordinary Windows policy may turn off the monitor. Off holds and later restores that thread's prior execution state. It cannot override another process's power request; monitor behavior is untested.

Preview clones the selected whole session, enters its actual owned Virtual Tank and advances only that clone on a separate28ms clock. It accepts close input rather than food, combat, profile or save actions, and renders the same owned fish/pets/threats without gameplay controls. Both normal and paused live bookkeeping stay suspended. Closing clears both clocks and restores the existing live music cue/pause state. Sound controls preview music/effects; it cannot establish audible fidelity. No preview scene is assigned or serialized back to the profile. Pet collection advances only the preview counter and discards its temporary wallet credit.

Secondary-source anchors at W1`f919b3c241cfd611c547f1653fb3514f500f761b`: `WinFishApp.cpp:275–280` supplies defaults; `Board.cpp:623–638` rotates among owned alternatives every10800updates; `Board.cpp:5625–5641` dims within a3000update cycle,1320–1680 window,30update ramps and alpha200; `Board.cpp:5619–5623` supplies the optional shell overlay; `Board.cpp:2320–2324` distinguishes the screensaver credit path. Preview rotation uses private projected RNG and dim interpolation uses projected integer rounding. Exact screensaver coin-generation caps and Niko backdrop reposition remain deferred detail leads at the same Board paths. No new Ghidra export or original process supplies the simulation.

## Saver earnings

E109 introduced graceful standalone `/s` income into project-owned `screensaver-bank.json`; E110 checkpoints it with the narrow continuation below. `/p`, `/c` and the in-app preview remain uncredited. The saver never overwrites `profiles.json` or persists its whole cloned session. An existing current profile is required for `/s`; the normal app establishes identity before opening its window. Income is independent of whether the money overlay is visible. Final saver evidence records delta, bank total, scene sequence and profile-generation key.

Current profile3 requires generation, consumed cumulative watermark and scene receipt; session format stays36. Rename preserves the key; creation or explicit new-game reset receives a fresh generation. No older manager/bank or missing metadata is migrated/backfilled. Auxiliary bank2 validates bounded rows, strict keys/schema/scenes and checked credit/sequence overflow; zero credit is valid with a pending scene. Source behavior is **secondary-source-derived** from W1 `ProfileMgr.cpp:230–267,301–329,567–574`: accumulated saver credit, positive claim cap100,000 and wallet cap9,999,999. The primary BR07 reader at PB05`005148c0` supports a token/optional-delete boundary; writer/cadence and retail byte compatibility remain unknown. W1 `GameObject.cpp:243–271,1079–1090` and `Board.cpp:2649–2657` give a5,000..50,000 production threshold derived from Board updates, not a daily cap; that production limit is not yet wired to the saver scene.

The project deliberately replaces erase-before-profile-save with cumulative credit: normal load, profile switch and selector/store/setup entry snapshot the bank, then atomically save wallet and consumed watermark together. Capped excess is consumed too. Later bank appends remain pending; reopening or retrying a successful profile save cannot credit the same total twice. Result phases whose receipts fix the wallet defer ingestion until exit. A held `profiles.lock` rejects another cooperating normal writer; short separate `screensaver-bank.lock` operations permit saver append while normal play continues. Only bank contention retries, with at most250ms sleep; malformed records and write errors remain visible. These are project integrity policies, not proved retail concurrency behavior. Atomic replacement covers process-level transaction structure; no power-loss/fsync guarantee or actual multiprocess/native scenario is claimed. Ungraceful saver exit or failed deposit does not create a durable scene/earnings checkpoint.

E110's scene guard is below. Production limits, exact cadence and native behavior remain gaps.

## Windows boundary

`/s` opens a full-screen readonly clone of the selected profile's owned VT; keyboard/mouse movement dismisses it after a startup grace period. `/p <HWND>` (also `/p:<HWND>`) attaches only the project-owned window to the supplied nonzero decimal parent, checks parent identity/lifetime, resizes to its client area and mutes preview sound. `/c` opens preferences; E116 joins optional parent ownership/lifetime through the guard below. Malformed switches/handles and unsupported gameplay options fail rather than falling through to normal gameplay. Native modes are Windows-only. They do not save the cloned session; configuration Save writes only auxiliary preferences.

### Configuration ownership

Independent host feature`c249298`, reconciled/pushed in`53cb042`, adds
`OwnWindow::attach_configuration(ParentWindow)` and a same-thread RAII
`ConfigurationOwner`. The E116 app captures its current-process configuration
window for `/c <HWND>`, attaches/retains the guard, polls in the graphics loop,
closes its own configuration path on captured parent/window disappearance and
explicitly restores/logs the result before dropping the guard. Error exits retain
RAII cleanup. `/c` without a parent remains standalone. Neither route modifies
the supplied parent, embeds/restyles/resizes it or changes registry/OS activation.

The guard rejects child/self/cyclic owners, rechecks captured identities, changes
only the project window's owner and exposes native errors. Changed ownership is
left alone; a dead prior owner is detached rather than reused. Platform design
uses the specific [SetWindowLongPtrW contract](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowlongptrw)
and [top-level window ownership](https://learn.microsoft.com/en-us/windows/win32/winmsg/window-features),
not game-behavior evidence. Actual owner assignment/readback/destruction,
restoration/focus/DPI and native acceptance remain not tested.

Host-only format/strict lint/locked offline Windows build passed with zero tests;
host source SHA`B76C0DE3C2FA325DF37DAA647C2CDBB1969464C8909521C8847A022724BC50FB`.
The parent app join passed A62CatalogConfig01 format/strict workspace lint/locked
build, also with zero tests. [MODLOG](../MODLOG.md#2026-10-10--join-owned-biographies-and-configuration-ownership)
and [STATUS](../STATUS.md#current-build-verification) own receipts. Existing E110
profile3/bank2 scene evidence remains historical; normal Session envelopes are
now current37 per [retained modes](mode-continuation.md), without legacy conversion.

The `turbofish-screenhost` crate owns unsafe Win32 FFI, process/thread ownership checks, preview attach/restore and monitor-request lifetime. Simulation remains safe Rust. Active-window discovery is limited to the calling UI thread and verifies process/thread/root ownership; no title search or foreign-window mutation is used. Parent/child invalidation is a visible host failure or close, not permission to choose a different window. `/s` and `/p` accept existing normal-speed evidence/capture/deadline options; `/c` rejects scene evidence. Actual focus, parenting/DPI/render-loop interaction, dismissal, monitor state and sound remain untested.

`scripts/New-ScreensaverPackage.ps1` accepts an associated launcher build identity and verifies/copies its executable, five native libraries and eight notices into an immutable ignored `target/screensaver/<build-id>` directory. It renames the same executable bytes to `TurbofishDeluxe.scr`, adds the host license and records copy hashes. E108 exercised this command. Packaging does not install, register or activate the screensaver or alter the original installation. Windows selection/activation remains pending.

## Focused validation

E107 passed one prepared-state isolation regression exercising actual owned-fish motion and pet collection with byte-identical live state and unchanged persistent wallet. Initial compile/lint failures stopped before cases; the case then found real preview-entry stepping before the wallet guard. Entry now applies actions without a tick, leaving the first update to the guarded preview. Only that failed case and build reran; prior format/lint evidence remains qualified in [MODLOG](../MODLOG.md#2026-10-10--integrate-disposable-screensaver-preview). This is a data-integrity contract, not earned gameplay or native acceptance. No default/getter matrix, unaffected gameplay filter or native run was added.

E108 passed formatting, strict workspace lint, one host parser safety case and locked build/staging. Initial borrow/RAII-warning and later style failures stopped before cases. One package command matched68 inputs and14 copied build artifacts, plus the host license. No gameplay/full-suite/native case was run; [MODLOG](../MODLOG.md#2026-10-10--integrate-windows-screensaver-host) owns exact receipts.

E109 passed one combined disk-backed integrity case: separate writer/bank locks, cumulative merge, source claim/wallet caps, failed profile save retaining credit/watermark, rename, reopen without recredit and fresh-generation isolation. Formatting/strict lint/build/package passed; initial hash-format/fixture metadata failures stopped before the case. [MODLOG](../MODLOG.md#2026-10-10--integrate-screensaver-credit-transactions) owns receipts. This prepared-state file transaction is not natural saver income, actual process contention, native acceptance or retail fidelity. No getter matrix, unaffected filter, full suite or native run was added.

## Scene continuation

E110 uses required profile3/bank2 metadata; session36 stays unchanged and older manager/bank formats are rejected without migration. `/s` captures the authoritative VT tank/Board/shared seed digest before offline cleanup or preview preparation. Save timestamps are excluded from conflicts. Only tank, Board, shared VT seed and timestamp travel; unrelated Adventure progress, normal RNG, mode, phase and wallet are preserved. Temporary backdrop rotation and the saver Board-shell display counter are restored during export; income remains cumulative bank accounting. This follows W1 `Board.cpp:2294–2324,2344–2369,2518–2522` with explicit project projection/transaction policies. Primary installed writer/cadence remains unknown.

One saver holds `screensaver-scene.lock` from before profile loading through graceful checkpoint. Normal adoption briefly tries the same lease and defers while a saver is active; income claims remain independent. A subsequent saver resumes an unconsumed matching branch with its original baseline. Scene and income replace the bank file together, even when income is zero. Normal startup reconciles before offline cleanup in selector/VT/setup; runtime adoption happens at selector boundaries. A matching scene advances session ticks by checked nonnegative Board progress and validates against current unlocks/capacity; a conflicting VT edit consumes the scene receipt without replacing current state or discarding income. Adoption/rejection and receipt share one atomic profile save; failures preserve in-memory authority and the unconsumed receipt. Explicit new-game reset starts a fresh bank incarnation. Selector/Windows previews and configuration do not checkpoint scenes or earnings.

The single E110 disk-backed handoff case passed, including successive scenes, lock deferral, failed write/retry, fish/Board/RNG continuity, unrelated state preservation, reopen no replay and stale rejection with income retained. [MODLOG](../MODLOG.md#2026-10-10--integrate-saver-scenes-and-board-bubbles) owns failures/build/package identity. Exit-only checkpoint loses unsaved scene/income on abrupt termination; actual multiprocess, power-loss, normal-speed saver/native and retail/audio/human behavior remain not tested. Exact production cap, Niko backdrop-substitution pose and installed writer scheduling remain deferred source follow-ups.
