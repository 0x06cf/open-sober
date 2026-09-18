# Open Sober — Agent Handoff
## SH328 (Sep 18, 2026, hermes-worker): CROSSED the SH327-declared NEXT wall — fn 0x25f52b4 x20 params-object — the
## app-start fencepost ADVANCED one full step to a real-AppStarted live-member gate (0x25f5050).
Implemented `routeb_patch_startapp_params_x20`: patches `mov x20,x0` @0x25f52d8 (0xaa0003f4) -> `adrp
x20,0x107334000` (0xf00269f4), a single identical-footprint instruction pointing the V2StartAppWithParams
params register x20 at the JIT's mapped zeroed guarded RW tail (402MB@0x107334000). fn 0x25f52b4 only READS x20
(pg ~0x300-byte window: #320/#328/#0x18..#0x110/#0x148), so a benign zeroed base lets it run to completion.
Gated on JIT_ROUTEB_DM_SEED/DMFORCE (inert by default). MEASURED 2/2: patch fires; JIT_DUMP_PC=0x1025f5460
HITS (fn 0x25f52b4 COMPLETES + reaches the [x19]vt+16 AppStarted dispatch — the SH327 wall is crossed); the
fault ADVANCED from guestpc=0x1025f5300 fault=0x140 to guestpc=0x1025f501c fault=0x0 (`ldr x8,[x0]` @0x25f5050,
x0=[x19,#0x408] il 2ea3a84 result = 0; x19=REAL AppStarted heap obj 0x5588b4471ae0). +sh328 hermetic guard
(5 pins) + frontier doc frontier-sh328-startapp-params-x20-cross.md. elfjit examples 152/0, arm64jit lib
408/0, workspace green, elfjit.rs under 1MB hook.
HONEST: no DM (DM-root 0, MH_* false); Route-B live-DM gate UNCHANGED. The new 0x25f5050 gate is a
real-AppStarted LIVE MEMBER (+0x408/+0x418) — the SESSION-CTOR class, where SH251 proved fabricated-seed is a
dead end. NEXT: the REAL Activity/AppBridge session drive so the upstream session ctor RUNS and builds the
+0x408 member (not another static seed).