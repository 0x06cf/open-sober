# Open Sober — Agent Handoff
## SH330 (Sep 18, 2026, hermes-worker): crossed the SH329 app-start fork gate one deterministic step —
## a block-entry guard seeds AppStarted+0x408 with a benign vt[+136]-leaf object -> 0x25f5050 no longer SIGSEGVs;
## downstream is run-variable (SH320-class host-ptr leak / FMOD AAudio), NOT a stable gate. recon-v3 green at HEAD.
Measured (real libroblox.so, full SH328 seed set + JIT_ROUTEB_APPSART_408SEED=1, 3/3): the SH329 fork at
0x25f503c converges on `ldr x0,[x19,#1032]` = [AppStarted+0x408] (runtime heap x19) = 0 -> `ldr x8,[x0]`
@0x25f5050 SIGSEGV. SH329 closed only the fork (no govflag value yields non-null). SH330 discovered the
+0x408 dispatch @0x25f5058/5c is a PLAIN `ldr x8,[x8,#136]; blr x8` — NOT the x8-out-param ABI SH324
proved un-crossable. New default-inert guard (env JIT_ROUTEB_APPSART_408SEED) fires at block-entry
[0x1025f501c,0x1025f5060] and seeds the RUNTIME-[x19+0x408] with a leaked benign object whose vt[+136]
is a host leaf. 3/3: gate deterministically passed. Downstream run-variable on 3 runs: clean EXIT 0
(vt[+136] leaf -> canary epilogue + ret) / FMOD AAudio SIGSEGV guestpc=0x106240cb8 (SH212-crash-A) /
leaked-host-pc (SH320 ASLR-flaky). So the next wall is run-variable, not stable. This answers SH326's
open "sync/determinism gap" question: seeding the member deterministically crosses the gate.
+sh330 hermetic guard (5 pins: 0x25f504c ldr x0,[x19,#1032], 0x25f5050 ldr x8,[x0], 0x25f5058 ldr
x8,[x8,#136], 0x25f505c blr, 0x25f5080 ret) + frontier doc. elfjit examples 154/0 (152+sh329+sh330),
arm64jit lib 408/0, workspace green; elfjit.rs 1048570 B under 1MB hook (SH320-329 prose condensed,
facts preserved). recon-v3 re-verified green (24 frames, present #21..#23 swap Ok(0x1), 0 json abort,
EXIT 124).
HONEST: no DM (DM-root 0, MH_* false); Route-B live-DM gate UNCHANGED. SH330 is a fencepost advance
(crosses one deterministic app-start gate) + documents the next wall is run-variable (host-ptr leak /
FMOD AAudio — SH320/SH212 class), NOT a session boot or rendered screen. NEXT = drive the REAL
Activity/AppBridge session so the upstream ctor builds a REAL +0x408 object (real vt[+136] work), not
a fabricated leaf (SEP-17 SESSION-CTOR PRIMARY LEVER, ref SH184).