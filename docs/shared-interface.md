# Shared interface and ending

The bulk current34 increment and current36 Help/Story join passed focused compilation/lint and affected data-integrity/behavior checks; native execution remains not tested. [STATUS](../STATUS.md) owns the current goal/checkpoint; [coverage](compatibility.md) retains the full requirements and deferred acceptance.

## Ownership and usable paths

`Profiles` owns stable profile IDs and whole sessions. The selected profile's session is the sole live gameplay state; create/select/rename/delete use the existing atomic project-write boundary. `profiles.json` becomes the current manager checkpoint. Adopt an already valid current-format `adventure.json` only when the manager file is absent, leaving that file intact. This does not migrate old project or original saves. Sandbox saves use the retained-scene projection before manager serialization and cannot overwrite earned Adventure/TT/Challenge state.

The frontend uses the typed main menu, four-card chooser, Story and Hall views described in [runtime frontend joins](#runtime-frontend-joins). Profile management and shared options retain the existing atomic panel transactions. Options apply actual music/effects gain, fullscreen and cursor APIs before committing settings; unsupported audio controls stay disabled. A hardware acceleration switch is omitted because this renderer has no corresponding backend operation. Exact retail screen art, focus/input behavior and option persistence require later comparison.

Global score rows are a separate copied-name ledger from personal session bests. Renaming or deleting a profile must retain earned rows and their recorded names. The recovered ordering/capacity is one ascending time row per ordinary Adventure stage, three descending survivor rows for Tank5, five descending Time Trial rows and five ascending Challenge rows. Ties follow existing equals; repeated names are valid. Record actual completion/expiry events once, never from every autosave. Fresh normal startup now admits the explicit [owned presets](highscores.md); existing managers and imported current sessions retain their ledger without backfill. Exact binary row codecs remain deferred.

## Singing fish and finale

Installed `fishsongs/*.txt` supply bounded parsed scores, three timed voices and optional applause. The app owns transient song admission/title and pause/exit handling; the audio owner plays the original tone samples at the requested pitch/gain. Parsed score lengths schedule notes, not sample truncation. Exact song selection/RNG, rare timing and audible fidelity remain pending.

The finale retains the actual absent fixed Tank5 pet kinds before retiring the Board. Its [full runtime renderer](finale.md#runtime-integration) uses the owned narrative/cast/credits catalog, resurrection/Bilaterus layers and two transient bubble owners. The Interlude controller retains scroll, pet return/heal cues, later Lullaby selection and menu exit. Native layout/fades/focus/timing/audio and human judgment remain deferred. A renderer/controller alone does not establish playable or retail-equivalent behavior.

## Installed song syntax compatibility

Read-only installed-score inspection found five syntax forms rejected by the first Rust parser. One failed score disabled the complete catalog at startup. The correction follows pinned W1 `f919b3c241cfd611c547f1653fb3514f500f761b`, `source/WinFish/FishSongMgr.cpp`, `NoteData::Parse:477–617` and `ParseCommand:426–432`:

| Observed installed input | Secondary-source-derived interpretation |
|---|---|
| `entertainer_long.txt:69`, `s+s+` | Consume one final separator; duration remains 24 score units |
| `hammerklavier1_long.txt:73`, `bbe` | Nondigit after a flat keeps default octave4; pitch10 |
| `pathetique1_{long,short}.txt:28`, `an3` | Unsupported modifier returns initialized unshifted pitch0/duration0; do not invent an A-natural interpretation |
| `silentnight.txt:33`, `0k` | Numeric duration uses its integer prefix, zero |
| `williamtell_long.txt:53`, `*localshift 0;` | Integer-prefix pitch shift, zero |

Checked numeric conversion, finite/bounded timing, pitch limits, note/file/catalog limits and rejection of leading/repeated symbolic separators remain project contracts. Complete-file validation remains atomic; no failing song is skipped. The opt-in `owned_fish_song_catalog_loads_every_installed_score` regression checks exact directory/catalog membership and reports every file failure when loading fails.

Agent verification loaded all72 installed scores/47,577 notes, passed five focused song cases and formatting/strict lint/native build in an isolated checkout of `be850c1` plus the identified parser/test changes. All72 installed score digests remained unchanged. These are parser/build observations; interpretation above remains secondary-derived, and native/audible/retail comparison is unperformed. [MODLOG](../MODLOG.md#2026-10-10--fix-installed-fish-song-catalog-loading) owns commands, failed attempts and build receipts.

## Help and earned stories

Current36 joins the existing instruction phase with an app-lived Help controller: page0 instructions, remembered ordinary pages1–8, direct tabs and wrapped navigation. Selector Help and tank F1 use explicit return destinations; tank Help restores its prior pause state. Both Help and selector Story browsing suspend ordinary and paused session updates and clear accumulated time on return. Their text/layout is project-authored; original Help art and animated examples remain pending.

Challenge Story selection is a once-only session transaction. The profile owns a32-bit seen mask and separate Cyrax flag. PB05`00552e50` supports the four eight-story groups and gates only extra-pet stories20–23, correcting W1's all-pet gate. After all32 ordinary stories and purchase cursor>5, selection earns Cyrax32; later selection draws among earned possibilities. Browsing wraps33 IDs, skips unearned stories, and cannot unlock, advance a Board or repeat a result credit. Challenge interlude retains its earned Story ID across current36 save/reopen. Required ownership fields reject missing/pre36 saves without migration/backfill; historical constructors retained in source do not provide an import route.

The catalog loads33 titles and bodies read-only from the identified owned English Steam executable. Its digest and study boundary belong to [provenance](provenance.md); no extracted lore enters Git. Unsupported identities show unavailable text. Selector Stories and Challenge interlude use this catalog, owned portraits, bounded scrolling and the existing music owner. One presentation cue function chooses Story/interlude/finale music consistently at all sync sites, avoiding competing requests each frame. Exact retail talk-show art, portrait animation, screen input and audible fidelity remain pending.

E106 passed two new Story-rule cases, one Challenge ownership/browse/reopen/no-repeat-credit case and one opt-in actual owned-catalog case. An initial lint stop ran no cases. A later static join check found selector Story browsing still invoked paused bookkeeping; the UI hold was corrected, then compile/lint/build alone revalidated that presentation change. The four passing cases were not repeated. [MODLOG](../MODLOG.md#2026-10-10--integrate-help-and-earned-stories) owns the retained receipts. No native/manual/visual/audio/retail/human acceptance was performed.

## Runtime frontend joins

E113 registers `app/menu_runtime.rs`, `app/tank_selection_runtime.rs` and `app/records_runtime.rs` in the actual app. The menu consumes source-shaped scene commands, original resources/fonts, owned Meryl animation metadata, independent effect RNG and a1x UI clock. Immutable facts carry stable profile identity, current locks, shell balance and trophy; the session rechecks every mode action. The welcome-back projection uses `session.ticks > 0` as a qualified project heuristic. Existing Sandbox code entry remains separate; typed `give` currently reaches an explicit pending gift join rather than a duplicate ledger. Quit requests open confirmation. Profiles/options retain their writer; Stories and Saver remain accessible from Options.

The four-card Time Trial/Challenge chooser uses the actual creases/bolts/backdrops/buttons and owned font metrics. It refreshes availability from the existing session authorities before typed move/down/up/focus/modal/cancel events. Menu returns through the existing action; Stories opens only earned pages and retains the chooser. Failed layouts are visible, rather than silently substituting a playable scene. The old vertical-list input path is bypassed by the typed owner.

[Story](story-screen.md) and [Hall](highscores.md) now render actual plans with owned fonts, portraits, talk-show texture, WAVE metadata and independent bubbles. Standalone browsers freeze retained simulation; Challenge Story changes return one existing navigation action to the authoritative session. Mouse press/release capture, focus cancellation, keyboard navigation and scrolling feed the existing controllers. Hall retains its transient page and reads copied global rows/current personal records. Profile switches drop all scene controllers and captures. No schema changes, migrations or second persistent writer were introduced.

Fresh normal managers load preset names from the identified owned executable before their existing atomic checkpoint. Existing managers, current-session imports and saver startup keep their ledger. Missing or unsupported catalogs fail creation before the manager write; no fabricated names are used. An existing empty manager replaced by `--new-game` remains conservatively unseeded because the current CLI API does not report that replacement branch.

A59Frontend01 stopped at three parent compile joins: obsolete Story import, Hall sound enum without a resource method and the saver configuration call's removed Help argument. Corrected those calls and removed unused retired hit helpers.02 passed formatting, strict workspace/all-target lint and locked build/staging. **No tests ran**, no duplicate presentation cases were added and no native/game/DB process or protected save/install changed. [STATUS](../STATUS.md#current-build-verification) and [MODLOG](../MODLOG.md#2026-10-10--join-main-menu-chooser-story-and-hall) own receipts. VT/gift adapters remain separately owned until handoff. Normal Title-to-menu over retained ongoing modes, exact dialogue art/cursor capture, accessible audio, native/retail and human acceptance remain pending.

## Source and verification boundaries

Installed PB05 remains authoritative. [BR08–11](binary-recovery.md#br08--menus-profiles-and-options) owns the primary profile/menu/options/score outline and explicit gaps. Pinned W1 `f919b3c241cfd611c547f1653fb3514f500f761b` supplies `ProfileMgr`, `OptionsDialog`, `HighScoreMgr`, `GameSelector`, `FishSongMgr`, `GameObject`, `Board` and `InterludeScreen` behavior as secondary evidence. [Provenance](provenance.md) owns licenses and adaptations. No upstream source file, proprietary asset or raw decompiler output is tracked.

E102 covers two profile, two song, one global-score, one Penta, one finale and one Sandbox persistence case across focused runs. The profile case exercised atomic save/reload, selected-session conservation and copied score names after rename/delete. The finale case checked actual survivor/fallen-pet capture and once-only completion. The song fixture pitch was corrected against source; a later count failure exposed early applause and its scheduler was corrected before the failed case passed. [MODLOG](../MODLOG.md#2026-10-10--integrate-sandbox-and-shared-interface-and-preserve-current34) owns receipts and failures. No unaffected suite, native/manual input, render/audio, original-game comparison or human feel check was repeated or performed.
