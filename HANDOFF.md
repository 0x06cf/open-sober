# Open Sober — Agent Handoff
## SH327 (Sep 18, 2026, hermes-worker): AppStarted factory construction made DETERMINISTIC; the app-start
## fencepost 0x1025f5300 ADVANCED one level deeper (SH326's sep-17 "force the construction branch" forward).
Implemented `routeb_patch_appstart_construct_force`: patches 0x2e890f4 `b.ne 0x2e89118` (0x54000121)
-> unconditional `b 0x2e89118` (0x14000009), removing the AppStarted factory's sole non-construction
exit (producer-counter tag==2 -> unconstructed ret). Idempotent (short-circuits [x19,#24]). Gated on
JIT_ROUTEB_DM_SEED/DMFORCE (inert by default). MEASURED 3/3: construction write fires + lands
(`[appstart0x106a6f480]=0x55cb343ab000`, real heap obj, DETERMINISTIC); field-copy helper 0x25f54e8
completes + returns; the fault ADVANCED to `ldr w3,[x20,#320]` @0x25f5328 (fault=0x140) where x20 =
V2StartAppWithParams params object = 0. +sh327 hermetic guard (9 pins) + probe probe_sh327_construct_force.sh.
elfjit examples 151/0, arm64jit lib 408/0, workspace green; elfffijt held under 1MB hook (condensed prose).
HONEST: no DM (DM-root 0, MH_* false); Route-B live-DM gate UNCHANGED. NEXT: the params-obj +0x140 member
(x0/x20 param of fn 0x25f5270 = 0).