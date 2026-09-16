# SH223 — DM-creator reachability re-measured at the corrected-SH161b state + region addresses pinned as regression anchors

## Why
The operator's Route-B directive names the NativeDataModelManager DM-construction path
(getFlagsFromEngine_/initEngine_ + initializeLuaApp_/startLuaApp_) as the re-attack target
for the live-DataModel wall, and the standing doctrine says "re-verify a premise at the
newest state — that is how the hit-rate climbs." SH209's reachability NEGATIVE was measured
at the post-SH202 state, BEFORE SH217's corrected SH161b window started actually firing the
transition-body bypass (SH217/218 landed later). SH161b alters the governor-tail transition,
so DM-creator reachability on a run where the bypass genuinely engages is a NEW measurement,
not a re-run of SH209.

## Measurement (fresh, at the corrected-SH161b state)
Canonical completing ladder, SH209 harness, 6 runs. SH161b now firing (SH217 correction
live at HEAD). Governor-tail region [0x102e9fa80,0x102ea3b40) used as the in-run calibrated
positive control (the control that proves the run is actually completing).

Result:
- 3/3 clean completing runs (EXIT 124, 0 crash): govtail control fired **25 distinct pcs**
  each (75 total) — the run genuinely walks the governor tail to the canary-ret terminal.
- **DM-creator regions [0x102bd1a30,0x102bd1d08) + [0x102bd21d4,0x102bd2600) = 0 hits in
  ALL runs** (0 total), exactly as SH209.
- 3/6 runs stopped early at the known pre-existing non-seedable V2 singleton-vtable host-pointer
  flake (SH198/SH55 class, guestpc 0x102b9dee0 family) BEFORE reaching the control — not a
  DM-creator pc, not region-watch induced (same run-variable behavior SH205/206/208 document).

## Verdict (do-not-re-tread)
Even with the corrected SH161b truly bypassing the V2Init transition body, the NativeDataModelManager
DM-construction bodies are NOT reached on the completing path. Route-B live-DM world-build =
structural gate reconfirmed at the newest corrected state. Extends SH209's negative across the
only state change (SH161b leveraging) that has landed since — closing the residual "SH209 predated
SH161b firing" gap. No seed warranted; the once-slot is still a strcmp intern (0x400000b), the
resolver maps still empty.

## Code (+1 hermetic sh223, real-image guard family as sh222/sh219/sh213/sh211)
The reachability re-measure (this + the SH209 harness) reads EXACT region-entry addresses; a
single drifted constant would silently report 0 hits and fabricate a false negative. `sh223_
dm_creator_region_entries_pinned` byte-pins the 5 DM-creator region boundary words + the govtail
positive-control entry:
- getFlagsFromEngine_/initEngine_ entry    file 0x2bd1a30 = 0xaa1403e0 (mov x0,x20)
- initEngine_ region exit                  file 0x2bd1d08 = 0xb9401268
- initializeLuaApp_ entry                  file 0x2bd21d4 = 0x912fa063 (add x3,x3,#0xbe8)
- startLuaApp_ entry                       file 0x2bd2504 = 0x910f1063
- startLuaApp_ region exit                 file 0x2bd2600 = 0xf94002a8
- govtail positive-control entry           file 0x2e9fa84 = 0xa9ba7bfd (stp x29,x30,[sp,#-0x60]!)
Each with guest = file + 0x100000000 transform + 4-alignment + canonical-window check. Verified
against real libroblox.so (1 passed; elfjit example tests now 71/0).

## Unregressed at HEAD
recon-v3 plane (capture_taskv4_frame.sh: 24 task-driven frames, present #19..#23 swap Ok(0x1),
196 node pops, no json abort, 0 crash, EXIT 124) + full workspace (567/0) + examples (71/0).