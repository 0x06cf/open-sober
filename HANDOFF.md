# Open Sober — Agent Handoff

## SH344c (Sep 19, 2026, hermes-worker): CORRECTION to SH344b + continuation cap pinned
Single-agent (cone suppressed). Measurement-only. Workspace green (594 passed).

### Measured (real libroblox.so, SH343 full-ladder env + KEYFIX)
- **App-shell ctor band [0x102207b50..0x102209000] is NOT a stable negative.** A 3-run
  A/B (runs/sh344c_appshell_ab.sh) fires it ~78 distinct pcs on every run (2/3 also
  showed the activity-lifecycle 0x10284cf5c divergence on mid-cycle runs). SH344b's
  "0 hits stable negative" was the divergence arm, not the class. BUT the band is
  recon-sh165fwd's `__cxa_guard` FastLog warmer (0x102207b50 adrp+AcqRel+tbz; deep pcs
  0x208e88/0x208eac are a destructor-style container loop) — firing it warms logging,
  it constructs no DM/GuiObject. Corrects SH344b + SH340's framing; not a Route-B crossing.
- **DMCONT continuation caps at SH285, one hop BEFORE app-start.** Region-watch run 2:
  continuation body runs end-to-end through its flags-serialize (0x102bd1d68..0x102bd1f64
  -> 0x102bd2014) then dives into `bl 0x1d9d8b0` (initStorageManagerNative/LSM lane) and
  faults at 0x101db1b08 (SH285 reader/pop live-object wall, fault ff..ff). The post-app-start
  tail [0x2bd2050..0x2bd2160] gets **0 hits** — the app-start `bl 0x2bd2058` is never reached.
  So SH245 lever #2 (seed F+0x18 for the 0x2bd2080 deref) is NOT the effective next gate;
  it targets a deeper, unreached fencepost. SH285 verdict stands: don't repair-seed 0x101db1b08.
- Recon-v3 re-verified green: capture_taskv4_frame.sh 24 real task-driven frames, present
  #19..#23 swap Ok(0x1), 197 node pops, 0 json abort, 0 crash, EXIT 124.

### Verdict
Route-B live-DM structural gate UNCHANGED (DM-root 0x106a68818=0, MH_* false). No seed
produces a live DataModel. SH174 capture-latch stays the single forward hook. Persistence
lane (SH343) remains committed + green. elfjit.rs untouched (1,048,479 B).

## Next (unchanged, authoritative)
SEP-17 SESSION-CTOR cause-level drive (real Android Activity/AppBridge session so the
upstream ctor constructs the DM world for real) remains the primary forward. The DMCONT
+0x1f0 flag-completion line is measured-complete through its serialize body and capped at
the SH285 reader wall — no product of routing +0x1f0 deeper changes that. Un-driven
SESSION-CTOR candidates SH264 flagged: messageBus experience-launch receive + dataModel
bindings (onGameLoaded / onAppLuaWillStart) live-binder entries. SH174 capture-latch stays
the single forward hook. All research subagents Route-B-scoped / cone still suppressed.