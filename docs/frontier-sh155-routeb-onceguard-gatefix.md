# SH155 — Route-B do-init GATE-FIX: once-guard inversion corrected (disassembly-backed)

## Outcome
Recon deleg_ff125cbc (read-only) disassembled the GlobalInit do-init (guest
0x102206c40) and the SendAppEventOnAppReady ABI, producing the two fixes below.
Workspace stays green (537/0); the real-binary combined ladder runs clean
(EXIT 0, 24 real task-driven frames, 0 crash) AND reaches a new Route-B marker.

## 1. GATE-FIX (core Route-B advance) — the once-guard inversion
The `--v2boot` ladder's StartLuaAppDM rung used to force once-guard
`[0x106a68410].bit0 = 1` before driving the rung. The disasm shows this was
WRONG (inverted):

- do-init `0x2206c80` does `ldarb w9,[0x6a68410]; tbz w9,#0 -> construct path`.
- When `bit0 == 0` (NOT set) the do-init runs `std::__call_once` (bl 0x284ce54),
  whose DM-construct lambda (the call chain through 0x2206d6c -> 0x2173b3c)
  builds the DataModel controller and stores it at `[x23,#1032]` = guest
  `[0x106a68408]`, then self-latches `[0x106a68410].bit0` via `stlrb`.
- When `bit0 == 1` (FORCED, as the harness did) the `tbz` is NOT taken; the
  do-init SKIPS `__call_once` entirely and the match path's `cbz [x19,#0x20]`
  at 0x2206df8 reads the appbridge obj's +0x20 field `[0x106a68818]` which
  never gets populated -> the benign Ok(0x3e8) soft-return (no session node).

FIX (elfjit.rs): at the StartLuaAppDM rung, seed only flags-latch
`[0x106a683e8].bit0=1` + main-id `[0x106863a68]=this thread` and LEAVE the
once-guard CLEAR so the guest's own `__call_once` runs, builds the DM, and
self-sets the guard.

VERIFIED on the real binary (post-StartLuaAppDM probe): `once-guard=0x1` — the
guest `__call_once` COMPLETED and self-set the latch (previously always
host-forced, never self-run). app-data-model counter `[0x106dca000+0xe88]`
advanced 0 -> 1 (marker c). Honest: DM-root `[0x106a68818]=0x0` and the once-slot
`[0x106a68408]=0x4000` (not a live image object) — so the do-init still returns
Ok(0x3e8), but the once-guard is now genuinely completing, which is the 
disassembly-predicted prerequisite to `__call_once` populating the DM slot.
This is the next unblocked gate (the construct lambda 0x2173b3c returns 0x4000,
not a DataModel) — tracked for the next cycle.

## 2. SendAppEventOnAppReady ABI fix — event jstring in x5, not x2
Disasm of 0x102bb463c: the four jstring args are converted by helper 0x21e1fec
into `[sp+0x48](x2)`, `[sp+0x30](x3)`, `[sp+0x18](x4)`, `[sp](x5)`; the event
discriminator reads `[sp]` = the **x5** string. The harness passed "Home" in
x2, so the discriminator never saw it (run log `w19-event=0x1` — the value 1 is
the "Chat" path/leftover, not "Home"=0x4). FIX: move the fabricated "Home"
jstring from x2 to x5. Run log now logs the ABI-correct form. Honest: the run
still shows w19-event=0x1 because SendAppEventOnAppReady soft-returns at the
known SH115 singleton-vtable leak before the discriminator executes — latent-but-correct.

## 3. Added Route-B DM-root probes
`SH155 post-StartLuaAppDM` (immediately after the StartLuaAppDM rung returns,
before SH126 clears the once-guard) + the extended `SH155 DM-root probe` after
the ladder, printing once-guard / DM-root[0x106a68818] / once-slot[0x106a68408]
/ app-data-model counter — so a live-DM advance is observable.

## Repro / verification
- Unit: `cargo test --workspace` -> 537/0.
- Real binary (combined ladder, JIT_SH115_SINGLETON_PATCH=1):
  `timeout 100 env JIT_DRIVE_LIFECYCLE=1 JIT_SERIALIZE_RENDER=1 RENDERINIT_WARMUP_MS=1000 V2BOOT_WARMUP_MS=3000 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent --renderinit 0x105b3a280 --renderthunk --renderframe --renderframe-seedgles --taskv4-seed frame --deque-node-live 0x106829f00 --drain-poll 8 --persist-roundtrip --kicker 0x106863af8`
  -> EXIT 0, 0 crash, 24 real task frames, once-guard self-set 0x1, counter 0x1.
  Log: runs/sh155-run4.txt.
- NEXT (Route-B ladder): root-cause why construct lambda 0x2173b3c returns
  0x4000 instead of a live DataModel (the ONE-NEXT-UNSYNTHESIZED-OBJECT after
  __call_once now runs) so DM-root [0x106a68818] populates and the match path
  takes the app-shell ctor branch.

## 4. RECON-CORRECTION (deleg_d8c3d2b2, authoritative, disassembly-backed)
A follow-up deep recon (141 tool calls) corrected section 1's NEXT premise —
the do-init construct chain is NOT a DataModel builder:

- `0x2173b3c` is a thin dispatcher (`strcmp(x0*,"GPU")` vs literals
  x0="App",x1="Execute" -> `b 0x61e30bc`) = the **RTApp registry GetOrCreate**.
- `0x61e3aa4` returns `(0x4000<<app_id)|(count & 0x3fff)` — so the observed
  once-slot `[0x106a68408]=0x4000` is the **CORRECT RTApp app-task-id return**,
  NOT a broken/malformed DataModel pointer. The once-lambda registers the app
  task; it does not construct the DataModel.
- **DM-root `[0x106a68818]` (appbridge obj `0x106a687f8` +0x20) is a distinct
  .bss global with NO static/once writer in the entire binary** (verified:
  zero `str` instructions land on guest 0x6a68818 from any adrp base). It is
  populated ONLY by a live heap store during the real AppBridge V2 app-launch,
  which the headless ladder never reaches. There is NO transfer from the
  once-slot [0x106a68408] to DM-root [0x106a68818] and no static initializer
  writes it.
- The match dispatch (0x2206db8) reads `[x19,#0x20]` = [0x106a68818]; if 0 it
  takes the benign Ok(0x3e8). A non-zero would `ldr x8,[x0]; ldr x1,[x8,#0x30];
  br x1` into the app-shell ctor with the DataModel as `this` — i.e. DM-root
  must hold a live vtable'd DataModel/app-shell object.
- The register-region "keep clearing one-next-unsynthesized-object until the
  do-init owns a live DM" is therefore well-defined: the ONE-NEXT seed IS a
  live DataModel/AppShell object whose pointer lands in guest 0x106a68818,
  with a real [vt+0x30] = the app-shell ctor. That single store turns the
  do-init from "handled" into "constructing an app-shell session node".
- CoreScripts loader 0x1f1d8ac runs AFTER a live DM (no 6a68000 reference),
  confirming SH131d is downstream of owning the DM, not a blocker to it.

Honest status after SH155: the GATE-FIX + probes are committed and the ladder
is measurably deeper (guest __call_once fully executes and self-latches the
once-guard for the first time; app-data-model counter advances). The next
concrete seed is a real DataModel object at [0x106a68818] whose vt+0x30 is the
app-shell ctor (find that ctor address / reuse an existing DataModel vtable in
.data.rel.ro, then host-store the object pointer).