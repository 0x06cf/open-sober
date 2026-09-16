# Frontier SH206 — recon-v3 deliverables both verified GREEN at HEAD + w19-event PIN settled (benign soft-return, not a branch) + ladder 5/6 clean

**Session type:** verification/characterization cycle (no new seed warranted). Workspace green
(arm64jit lib 386/0 + all crates; 9 pre-existing warnings, no errors). Commit <SH206>.

## 1. Both immediate-priority recon-v3 deliverables verified green at SH205b HEAD

Re-ran `runs/capture_taskv4_frame.sh` (real libroblox.so, llvmpipe, EXIT 124):

```
[elfjit:taskv4] type4_frame_thunk registered at 0x7f00000001d0
[elfjit:taskv4] seeded dispatcher type-4 vector [0x106829ea8] = 0x7f00000001d0
[elfjit:taskv4-frame] patched heartbeat w4#2/w4#3 -> w4=4 (both)
[elfjit:renderthunk] published RENDERCTX 0x7f..110a00
present #19..#23 swap Ok(0x1) (5 distinct colors) — real task-driven frames
present count = 24 ; node pops = 195 ; (no json abort) ; 0 SIGSEGV/SIGABRT
```

- (1) SELF-DRIVEN FRAMES (`type4_frame_thunk`, `--taskv4-seed frame`): implemented + GREEN.
- (2) JSON-ABORT (`JIT_JSON_ZERO_FIX=1` length-clamp at 0x102355d40): GREEN, no RBX::json
  overflow in any run.

## 2. Ladder characterization at HEAD: 5/6 clean (was 8/12-10/10 run-variable at SH205b)

`runs/capture_sh205_postfamily.sh` + a fresh 6-run with `JIT_GUEST_STACK_DUMP=1` =
**5 clean / 1 fault** (matching SH205's corrected 8/10-10/10 run-variable range). The lone
fault = `guestpc=0x102b9dee0 fault=0x10` = the **documented SH198/SH55 pre-existing V2
singleton-vtable host-pointer flake** (SH204 listed the same class). NOT seedable, NOT a new gate.
Also confirmed the residual do-init __call_once (0x10284ce54) + system-dialog (0x104c393f0)
sites remain the known SH55/64 run-variable class. SH116b (flag-manager) fired on every run.

## 3. w19-event=0x1 PIN — SETTLED: benign soft-return artifact, NOT the 'Home'/w19 branch

Operator's STEP-2 note asked to PIN whether SendAppEventOnAppReady yields w19=0x4 ('Home')
vs 0x1. Fresh disasm of fn 0x102bb463c + discriminator blocks:

- Discriminator at file 0x2bb47c4: `mov w19,#0x4` (offset 0) is the 'Home'-magic path;
  `mov w19,#0x1` (offset 8) is the alt path. The earlier block at 0x2bb46b8 compares the
  event jstring length: len==4 (['H','o','m','e']) -> `movk w10,#0x656d<<16; add #0xe01`
  = 0x656d6f48 = 'Home' LE -> b.eq to the w19=4 path. So the ABI (event 'Home' in x5 ->
  w19=4) is correctly wired and static-analysis-wise correct.
- Observed `w19-event=0x1` in the run log is the fn entry's `mov x19,x5` (x19 = the x5
  jstring handle low word = 0x1), left intact because SendAppEventOnAppReady **soft-returns
  Ok(0x3e8) before the discriminator body runs**. It is NOT taking the w19=1 alt branch.
  => No harness bug; the step-2 ABI wiring (Home in x5, XID 0x200000, MH_* set-only) is
  correct-but-latent, fires only when a real session reaches the discriminator.

## 4. Route-B live-DM do-init cells reconfirmed unchanged at HEAD (SH196 still holds)

`JIT_DMCELLS=1 JIT_THREADS=1` on the full --v2boot ladder (clean run):

```
once-guard[0x106a68410]=0x1  once-slot[0x106a68408]=0x400000b (strcmp intern, NOT a DM)
DM-root[0x106a68818]=<SH156 host seed>  flags-latch[0x106a683e8]=0x0
app-data-model[0x106dca0e88]=0x0  holder[0x106391908]=0x106358d40
map-resolver[0x106dca0e70]={0,0,n=0 live=false}  src={0,0}  register={0,0}
```

The do-init `__call_once` self-latches (once-guard 0->1) but yields an RTApp app-registry
intern (0x400000b), NOT a live DataModel; DM-root holds the SH156 host seed; resolver map
EMPTY; all five MH_* flags stay false. **Route-B live-DM world-build = structural gate,
UNCHANGED** — reconcilable with SH196/203/204 and ~30 prior recon angles.

## 5. Genuine-new-item triage: the run-1 frame-timing fault (0x102306158) is NOT seedable

An earlier 6-run batch surfaced a fault site NOT in SH205's residual list
(`guestpc=0x102306158, fault=0x210`). Traced it to fn 0x2306130 (frame-time accumulator:
`[this+528]`=accum time, `[this+552]`=mode) called with `this=0` from the single caller
file 0x5fcbc48. The caller threads `this` from a **live enclosing object** (`[this+536]/[this+552]`
frame-timing members), NOT a fixed `.bss` pointer cell — so it is NOT a SH116b-class
seedable singleton (unlike the SH205 flag-manager, which read a fixed `[0x10672739b0]`).
It is the known SH55/64 run-variable NULL-object flake (0 occurrences in the subsequent
6-run GSDSP batch). Do-not-chase as a seed.

## Standings / do-not-re-tread
- Route-B live-DM = structural gate (SH174/196/203/204/206). No new headless seed warranted
  this cycle; both recon-v3 immediate-priority deliverables are shipped + verified green.
- The step-2 ABI is correct-but-latent (fires only post live-DM). MH_APP_READY stays false
  headlessly — the session-producer handoff stays latent-but-correct.