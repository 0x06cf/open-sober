# Open Sober — Agent Handoff
## SH331 (Sep 19, 2026, hermes-worker): characterized the SH330 app-start +0x408 gate (8-run A/B: guard
## 100% deterministic, downstream run-variable 7/8-clean) + SH348 once-slot DM-ctor lever measured-negative (reverted).
Measured (real libroblox.so, full SH330 recipe + JIT_ROUTEB_APPSART_408SEED, 8 runs): the guard fires
100% (8/8) and the gate crosses 100% (8/8, DUMPPC 0x1025f5060), but the DOWNSTREAM is genuinely
run-variable — 7/8 clean EXIT 0, 1/8 abort (host-pc/AAudio, SH320/SH212-class). So the crossed gate is
crossed-but-not-completed; the next wall is a run-variable host-pc leak, NOT a stable gate — matching
SH330's original doc exactly. SH330b (defensive epilogue x21 reseed) tested-and-reverted: 0 firings +
no distribution change = a no-op against an empty endpoint (the host vt[+136] leaf preserves CpuState
callee-saved regs, x21 === guardGOT already); SH184/SH186 no-cruft. +probe_sh330_repro.sh (canonical
8-run repro). recon-v3 re-verified green (24 frames, present #21..#23 swap Ok(0x1), 0 json abort, EXIT
124). Withdrawn probe runs for app-start forward + SetInitParams host-pc leak = same already-documented
SH320-class family (do-not-re-derive).
HONEST: no DM (DM-root [0x106a68818]=0, MH_* false); Route-B live-DM gate UNCHANGED; the +0x408
fencepost is stable at the fork only, downstream run-variable. NEXT = SESSION-CTOR real
Activity/AppBridge session drive so the upstream ctor builds a REAL +0x408 (SEP-17 PRIMARY LEVER,
ref SH184); App service registration (SH313/316/317/318) is the precise route — do NOT seed once-slot
[0x106a68408] (SH348 negative: the done-path dispatcher 0x2206db8 consumes do-init's original arg1,
not the once-slot; guard never fired, A/B crash-site changed run-variably = noise).