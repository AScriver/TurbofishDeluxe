# Shared interface and ending

The bulk current34 increment and current36 Help/Story join passed focused compilation/lint and affected data-integrity/behavior checks; native execution remains not tested. [STATUS](../STATUS.md) owns the current goal/checkpoint; [coverage](compatibility.md) retains the full requirements and deferred acceptance.

## Ownership and usable paths

`Profiles` owns stable profile IDs and whole sessions. The selected profile's session is the sole live gameplay state; create/select/rename/delete use the existing atomic project-write boundary. `profiles.json` becomes the current manager checkpoint. Adopt an already valid current-format `adventure.json` only when the manager file is absent, leaving that file intact. This does not migrate old project or original saves. Sandbox saves use the retained-scene projection before manager serialization and cannot overwrite earned Adventure/TT/Challenge state.

The frontend exposes profile management, shared options, score tabs and Help through selector panels. Options apply actual music/effects gain, fullscreen and cursor APIs before committing settings; unsupported audio controls stay disabled. A hardware acceleration switch is omitted because this renderer has no corresponding backend operation. Exact retail screen art, focus/input behavior and option persistence require later comparison.

Global score rows are a separate copied-name ledger from personal session bests. Renaming or deleting a profile must retain earned rows and their recorded names. The recovered ordering/capacity is one ascending time row per ordinary Adventure stage, three descending survivor rows for Tank5, five descending Time Trial rows and five ascending Challenge rows. Ties follow existing equals; repeated names are valid. Record actual completion/expiry events once, never from every autosave. Retail preset rows and exact binary row codecs remain deferred.

## Singing fish and finale

Installed `fishsongs/*.txt` supply bounded parsed scores, three timed voices and optional applause. The app owns transient song admission/title and pause/exit handling; the audio owner plays the original tone samples at the requested pitch/gain. Parsed score lengths schedule notes, not sample truncation. Exact song selection/RNG, rare timing and audible fidelity remain pending.

The finale retains the actual absent fixed Tank5 pet kinds before retiring the Board. A transient Interlude controller uses that saved snapshot for scroll, pet return/heal cues, later Lullaby selection and menu exit. Text rows are short project summaries, with owned sprites loaded at runtime; complete retail narration/credits, bubbles, exact layout/fades and human judgment remain deferred. A renderer/controller alone does not establish playable or retail-equivalent behavior.

## Help and earned stories

Current36 joins the existing instruction phase with an app-lived Help controller: page0 instructions, remembered ordinary pages1–8, direct tabs and wrapped navigation. Selector Help and tank F1 use explicit return destinations; tank Help restores its prior pause state. Both Help and selector Story browsing suspend ordinary and paused session updates and clear accumulated time on return. Their text/layout is project-authored; original Help art and animated examples remain pending.

Challenge Story selection is a once-only session transaction. The profile owns a32-bit seen mask and separate Cyrax flag. PB05`00552e50` supports the four eight-story groups and gates only extra-pet stories20–23, correcting W1's all-pet gate. After all32 ordinary stories and purchase cursor>5, selection earns Cyrax32; later selection draws among earned possibilities. Browsing wraps33 IDs, skips unearned stories, and cannot unlock, advance a Board or repeat a result credit. Challenge interlude retains its earned Story ID across current36 save/reopen. Required ownership fields reject missing/pre36 saves without migration/backfill; historical constructors retained in source do not provide an import route.

The catalog loads33 titles and bodies read-only from the identified owned English Steam executable. Its digest and study boundary belong to [provenance](provenance.md); no extracted lore enters Git. Unsupported identities show unavailable text. Selector Stories and Challenge interlude use this catalog, owned portraits, bounded scrolling and the existing music owner. One presentation cue function chooses Story/interlude/finale music consistently at all sync sites, avoiding competing requests each frame. Exact retail talk-show art, portrait animation, screen input and audible fidelity remain pending.

E106 passed two new Story-rule cases, one Challenge ownership/browse/reopen/no-repeat-credit case and one opt-in actual owned-catalog case. An initial lint stop ran no cases. A later static join check found selector Story browsing still invoked paused bookkeeping; the UI hold was corrected, then compile/lint/build alone revalidated that presentation change. The four passing cases were not repeated. [MODLOG](../MODLOG.md#2026-10-10--integrate-help-and-earned-stories) owns the retained receipts. No native/manual/visual/audio/retail/human acceptance was performed.

## Source and verification boundaries

Installed PB05 remains authoritative. [BR08–11](binary-recovery.md#br08--menus-profiles-and-options) owns the primary profile/menu/options/score outline and explicit gaps. Pinned W1 `f919b3c241cfd611c547f1653fb3514f500f761b` supplies `ProfileMgr`, `OptionsDialog`, `HighScoreMgr`, `GameSelector`, `FishSongMgr`, `GameObject`, `Board` and `InterludeScreen` behavior as secondary evidence. [Provenance](provenance.md) owns licenses and adaptations. No upstream source file, proprietary asset or raw decompiler output is tracked.

E102 covers two profile, two song, one global-score, one Penta, one finale and one Sandbox persistence case across focused runs. The profile case exercised atomic save/reload, selected-session conservation and copied score names after rename/delete. The finale case checked actual survivor/fallen-pet capture and once-only completion. The song fixture pitch was corrected against source; a later count failure exposed early applause and its scheduler was corrected before the failed case passed. [MODLOG](../MODLOG.md#2026-10-10--integrate-sandbox-and-shared-interface-and-preserve-current34) owns receipts and failures. No unaffected suite, native/manual input, render/audio, original-game comparison or human feel check was repeated or performed.
