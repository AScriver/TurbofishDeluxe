# Virtual Tank screensaver

The implementation keeps screensaver scenes disposable. [STATUS](../STATUS.md) owns verified build/checkpoint scope; [design](DESIGN.md#screensaver-integration-decisions) owns the integration boundary. Native execution, Windows embedding and retail/visual/audio/human comparison remain not tested.

## Preferences and preview

Selector Saver opens separate project preferences stored explicitly in `screensaver.json` under the selected project save directory. Sound, owned-backdrop rotation and periodic dimming default on; money overlay and power saving default off. Missing auxiliary configuration uses these source defaults; malformed/unsupported configuration reports an error. Preferences do not backfill or change gameplay save format36. Power saving is disabled until an actual host throttle exists; OS enable/registration is not simulated by a checkbox.

Preview clones the selected whole session, enters its actual owned Virtual Tank and advances only that clone on a separate28ms clock. It accepts close input rather than food, combat, profile or save actions, and renders the same owned fish/pets/threats without gameplay controls. Both normal and paused live bookkeeping stay suspended. Closing clears both clocks and restores the existing live music cue/pause state. Sound controls preview music/effects; it cannot establish audible fidelity. No preview scene is assigned or serialized back to the profile. Pet collection advances only the preview counter and discards its temporary wallet credit.

Secondary-source anchors at W1`f919b3c241cfd611c547f1653fb3514f500f761b`: `WinFishApp.cpp:275–280` supplies defaults; `Board.cpp:623–638` rotates among owned alternatives every10800updates; `Board.cpp:5625–5641` dims within a3000update cycle,1320–1680 window,30update ramps and alpha200; `Board.cpp:5619–5623` supplies the optional shell overlay; `Board.cpp:2320–2324` distinguishes the screensaver credit path. Preview rotation uses private projected RNG, dim interpolation uses projected integer rounding, and persistent screensaver earnings are deliberately not implemented. Exact screensaver coin-generation caps and Niko backdrop reposition remain deferred detail leads at the same Board paths. No new Ghidra export or original process supplies the simulation.

## Windows boundary

Standard `.scr` full-screen/configure/embedded-preview entry and owned-child window lifetime are the next host increment. The in-app preview does not install or activate a Windows screensaver. Development does not change registry/Windows configuration or the original installation. Packaging and any OS activation must use the project runtime and its dependencies; the retail executable cannot serve as a backend.

## Focused validation

E107 passed one prepared-state isolation regression exercising actual owned-fish motion and pet collection with byte-identical live state and unchanged persistent wallet. Initial compile/lint failures stopped before cases; the case then found real preview-entry stepping before the wallet guard. Entry now applies actions without a tick, leaving the first update to the guarded preview. Only that failed case and build reran; prior format/lint evidence remains qualified in [MODLOG](../MODLOG.md#2026-10-10--integrate-disposable-screensaver-preview). This is a data-integrity contract, not earned gameplay or native acceptance. No default/getter matrix, unaffected gameplay filter or native run was added.
