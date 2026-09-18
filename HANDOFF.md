# Open Sober — Agent Handoff
## SH343 (Sep 19, 2026, hermes-worker): CROSSED the SH341 LSM poison fencepost — the persistence-lane terminal wall (guestpc 0x101d9a528, the LSM free-list pop that SIGABRT'd every full-ladder Route-B run) now completes via routeb_lsm_keyfix_guard (JIT_ROUTEB_LSM_KEYFIX=1): the single poisoned .text KEY (0x101d968e4, from the 0x626b6d0 pool-pop wrapper) at the pop write-site is redirected to a valid host-heap write target, so `str x8,[x1]` lands in real memory. Measured real libroblox.so: keyfix fired once on the crashing iteration; the ladder advances ONE fencepost to guestpc 0x101db1b08 (SH285's LSM reader/pop terminal, fault=0xffffffffffffffff). Workspace green (arm64jit 414/0, +3 hermetic tests); recon-v3 deliverables re-verified present.

Single-agent (cone suppressed). One new default-inert env-gated guard + 3 hermetic unit
tests + frontier doc + repro capture script. No production path edited outside the new
guard (which is opt-in and off by default). Workspace green.

SH343 (committed): with JIT_ROUTEB_LSM_KEYFIX=1 the full-ladder send-appevent + session
route no longer ABRTs at the SH341 terminal 0x101d9a528 (persistence-lane pop); instead it
advances to the SH285 reader/pop terminal 0x101db1b08 (fault=0xffffffffffffffff, RBX-poisoned
live-object pointer) — a concrete measured CROSS on STATUS candidate #2. Pre/post identical
env except KEYFIX. This is a persistence-lane advance, NOT a session crossing: DM-root 0,
MH_* all false, Route-B live-DM gate UNCHANGED; SH174 capture-latch stays the forward hook.

## SH342 (Sep 19, 2026, hermes-worker): measured the SETWORLDBUILD world-build continuation (do-init -> fn 0x102ea3b14 -> nativeAppBridgeAppStart__) is UNREACHABLE from every driven rung — V2InitWithParams benign-soft-returns Ok(0x3e8) before its world-build gate block 0x102368100 executes so the seed never fires (0 world-build/appstart region hits w/ and w/o V2_ONDEMAND); full ladder dies at new live-object site 0x1026d63c0 (SH324-class). Refines STATUS candidate #2: world-build gate premise broken, continuation gated behind the live-DM/session-ctor wall (SH324/SH340). recon-v3 frame plane re-verified green (real mesh, 0 crashes, EXIT 124).

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