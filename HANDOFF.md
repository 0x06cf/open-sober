# Open Sober — Agent Handoff
## SH329 (Sep 18, 2026, hermes-worker): pinned the app-start AFTER-FORK two-arm closure deterministically —
## both the [AppStarted+0x408] path (arm-1, SH328 default) and the nativePreloadFlagOverrides return
## (arm-2, govflag SET) are null headlessly; the 0x25f503c fork is NOT a seed-side bypass.
Measured (real libroblox.so, full SH328 seed set; govflag arm-2 = JIT_ROUTEB_APPSART_GOVFLAG seeding
[0x106a64da0].bit0): arm-1 `ldr x0,[x19,#1032]`=[AppStarted+0x408]=0; arm-2 `bl 0x2ea3a84` nativePreloadFlagOverrides
ALSO returns x0=0. BOTH converge on `ldr x8,[x0]` @0x25f5050 -> `guestpc=0x1025f501c` fault=0x0. No governor-flag
byte value yields non-null -> the gate is fork-independent SESSION-CTOR class (SH251/SH317/SH324 closure); NO new
seed. Added sh329 hermetic guard (6 pins: 0x1025f502c ldrb govflag, 0x1025f503c cbz fork, 0x1025f5044 bl
nativePreloadFlagOverrides, 0x1025f504c ldr [AppStarted+0x408], 0x1025f5050 ldr x8,[x0]. 3bbd arrows, 0x1025f5058
vt+136) + frontier doc. recon-v3 re-verified (24 frames, present #21..#23 swap Ok(0x1), 395 pops, 0 json abort,
EXIT 124). elfjit examples 153/0 (152+sh329), arm64jit lib 408/0, workspace green, elfjit.rs 1048564 B (12 B under
1MB hook; SH-prose comment condensed + sh314 comment lightly condensed, facts preserved).
HONEST: no DM (DM-root 0, MH_* false); Route-B live-DM gate UNCHANGED. The AppStarted+0x408 /
nativePreloadFlagOverrides objects are both null and built only by an upstream session ctor. NEXT (SESSION-CTOR
PRIMARY LEVER): the REAL Activity/AppBridge session drive so the upstream ctor RUNS and builds +0x408 (not a seed).