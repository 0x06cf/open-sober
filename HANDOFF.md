# Open Sober — Agent Handoff
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