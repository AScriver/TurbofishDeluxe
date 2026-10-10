# Owned image animation metadata

E112 retains immutable animation metadata in `ImageEntry` and `DecodedImage`; `Presentation` retains it separately from GPU textures and cell dimensions. `AnimationInfo::cel_at_time(ticks)` maps source update age/countdown to a cel; `None` means an explicit hidden frame. Missing metadata is separate from that result. `duration_ticks()` and the typed mode remain read-only; no draw, profile or Board clock advances here.

**Secondary-source-derived:** W1 `f919b3c241cfd611c547f1653fb3514f500f761b`, `source/SexyAppFramework/ResourceManager.cpp:333–375`, `Image.cpp:76–176`, `Graphics.cpp:1059–1067`, and `source/WinFish/GameSelector.cpp:245–246`. Actual owned `properties/resources.xml:176–177,195–196,263,295` supplies the affected fields. [Provenance](provenance.md#runtime-sources) owns study/license boundaries; no XML, C++, art or raw output is bundled.

The decoder supports source loop/play-once/ping-pong, frame-delay defaults, begin/end delay overrides, per-frame delays and explicit frame maps. It bounds sequences to4096 entries, rejects invalid mode/map data and validates sheet admission. Source animation frame count uses max(columns,rows), independently of the full sheet-cell count. A one-cell resource without animation can use its static cel; multi-cell timing is never invented.

Installed WAVE/WAVESIDE use twelve rows, ping-pong, delay4 and begin delay6. MERYLBLINK has a six-entry once map ending in hidden`-1`; MERYLTAIL has reverse eight-entry once mapping with begin delay3/delay2. Selector source passes its countdown directly and skips negative cels. The selector model now distinguishes intentional hiding from unavailable timing; its frontend still awaits the app join. Help reads owned WAVE timing now. Story/Hall adapters use this same immutable API when registered.

One focused parser/timing regression passed: `manifest_animation_preserves_pingpong_delays_and_hidden_once_cel`. Formatting, strict lint and build passed in E112; [STATUS](../STATUS.md#current-build-verification) identifies the dirty source/build. This is source-grounded parser evidence, not native animation, pixel/timing, retail or human acceptance.
