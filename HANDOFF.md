# Open Sober — Agent Handoff
## SH342 (Sep 18, 2026, hermes-worker): measured the SETWORLDBUILD world-build continuation (do-init -> fn 0x102ea3b14 -> nativeAppBridgeAppStart__) is UNREACHABLE from every driven rung — V2InitWithParams benign-soft-returns Ok(0x3e8) before its world-build gate block 0x102368100 executes, so the seed never lands; sending-appevent path 0 world-build hits (even with V2_ONDEMAND), full ladder dies at new live-object site 0x1026d63c0. Workspace green; recon-v3 frame plane re-verified green (real mesh, 0 crashes, EXIT 124).

Single-agent (cone suppressed). Measurement-only (region-watch, no production seed added). One new
probe + frontier doc. No production path edited. Workspace green.

SH342 (committed): with JIT_ROUTEB_SETWORLDBUILD=1 + region-watch on the world-build body
[0x102ea3b14,0x102ea3bf0) and nativeAppBridgeAppStart__ [0x102338510,0x102338700): the send-appevent
reach (SH310-completing) shows `routeb-worldbuild` gate fired 0 times — SETWORLDBUILD seed at gate 0x102368100
NEVER lands because the V2InitWithParams rung soft-returns Ok(0x3e8) without executing its gate block
(0 world-build / 0 appstart hits; same with V2_ONDEMAND=1). Full ladder (--v2boot-skip-appstart, no
send-appevent) with V2_ONDEMAND dies at NEW crash site guestpc=0x1026d63c0 fault=0x0 (StartLuaAppDM nested
frame prologue, `ldr x0,[x0]` null-this — SH324-class live-object wall), 0 world-build hits. REFINES
STATUS candidate #2: the world-build gate premise is BROKEN (gate block unreachable from the driven rung),
not merely un-triggered; the nativeAppBridgeAppStart__ continuation is gated behind the live-DM/session-ctor
wall (SH324/SH340) and NO seedable lever reaches it. DM-root 0, MH_* false, Route-B live-DM gate UNCHANGED;
SH174 capture-latch stays the single forward hook.

## SH341 (Sep 19, 2026, hermes-worker): measured the LSM free-list `str x8,[x1]` write-off is a SINGLE poisoned pointer from the 0x626b6d0 pool-pop wrapper (the SH212 "FMOD AAudio" site), not an intrinsic unwritable-write — root-cause attribution, the SH268 verdict refined.

Single-agent (cone suppressed). One default-inert READ-ONLY instrumentation guard + one
hermetic lib pin + frontier doc + repro probe. No production path edited. Workspace green.

SH341 (committed): `routeb_lsm_keytrace_guard` (jit.rs, JIT_ROUTEB_LSM_KEYTRACE=1, inert)
fires at the LSM pool-pop fn entry 0x101d9a5a0 + pop write-site 0x101d9a528, logging
x0=KEY + x30=LR (caller), classifying each key (EXEC/.text vs guest-data vs host-leak).
MEASURED on the real libroblox.so (SH267 ON arm, EXIT 134 at 0x101d9a528): **exactly ONE
poisoned key** — 0x101d968e4, caller LR=0x10626b6dc — is the crash; all other 390 pops
carry valid host-heap keys and complete. Disasm confirmed file 0x626b6d0 is a thin
trampoline that forwards x0 unchanged into the pool-pop (`bl 0x1d9a5a0`) — this is the
"FMOD AAudio 626b6d0" site cited since SH212; it is the LSM pool-pop wrapper, not FMOD
audio output. => SH268's "proven-unwritable R-E live-object" is a SYMPTOM: the persistence
lane never writes code memory; a single stale .text pointer is fed as the pool key once,
by the FMOD/0x626b6d0 caller. Fix target = that upstream caller, NOT the LSM pop (which is
well-behaved for all valid keys). No crossing: DM-root 0, MH_* false, Route-B live-DM gate
UNCHANGED; SH174 capture-latch stays the single forward hook. `sh341` hermetic pin lives in
the arm64jit LIB suite (held the elfjit example under its 1MiB hook; the pop mechanism
bytes were already pinned by sh267/sh268).

HONEST: measurement/attribution, not a session advance — but it converts a long-standing
static verdict (SH268 "no lever can reach it") into a measured single-source pointer leak,
correcting the "FMOD AAudio" label and narrowing the fix to the 0x626b6d0 caller.
NEXT (unchanged): the real Activity/AppBridge session drive (SEP-17 SESSION-CTOR lever);
V1-AppStart-on-populated-registry measured-negative (SH337).