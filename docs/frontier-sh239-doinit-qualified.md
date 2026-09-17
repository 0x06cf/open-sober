# SH239 — Do-init once-lambda A/B FALSIFIED + app-shell ctor body MEASURED DEEP (qualifies "never completes")

Session: Sep 17, 2026 (hermes-worker). Real libroblox.so (host
`~/.cache/open-sober/robbox/libroblox.so`, 109,193,800 B). +2 hermetic
(`sh239_doinit_oncelambda_intern_store_and_ctor_deep_body_pinned`,
`sh238_real_image_receivecall_dispatchband_select_slots_relocated`), real-image
guard family (skip-if-absent). Workspace green. Single-agent (cone suppressed).

## Why

The operator's EXECUTE-DO-INIT-GATES direction hypothesized that "LETTING the
do-init once-lambda populate [0x106a68818]" was the correct gate action — i.e.
that the SH156 fabricated DM-root seed is *interfering* and that without it the
once-lambda would manufacture a real DM controller. That premise had never been
A/B'd on the real binary. This cycle A/Bs it fresh AND, because the once-guard
of the app-shell ctor 0x102207b50 self-set to 1 in the ladders (a real
construction signal), measures how deep that ctor body actually executes — a
qualification of the standing "StartLuaAppDM's do-init never completes
app-shell construction" label.

## Measured result 1 — A/B: the once-lambda NEVER populates a genuine DM controller

Two canonical completing --v2boot ladders (EXIT 124, 0 crash), identical env
EXCEPT the SH156 fabricated DM-root seed (`JIT_ROUTEB_DM_SEED`):

| state | once-guard[0x6a68410] | DM-root[0x106a68818] | once-slot[0x106a68408] |
|---|---|---|---|
| WITH seed  | 0x1 (self-latched) | 0x7f... heap box (our seed) | 0x400000b (intern) |
| WITHOUT seed | 0x1 (self-latched) | **0x0 (empty)** | 0x400000b (intern) |

The once-lambda runs either way (oncel-guard bit0 self-latches to 1) but the
`str x0,[x23,#1032]` at file 0x2206d74 (x23=adrp 6a68000 -> guest once-slot
[0x106a68408]) stores the **intern 0x400000b, NOT an in-image DM controller**.
Without the seed, [0x106a68818] stays 0.

**Conclusion:** the operator's "LET the once-lambda populate [0x106a68818]"
premise is **FALSIFIED headlessly** — the lamba's __call_once completes but
manufactures an intern (RTApp registry key), never a DataModel. The SH156
fabricated DM-root seed is **necessary-but-insufficient** (without it the root
is empty; with it, it's a box whose vt[+0x30] still doesn't yield a make_shared).
The live-DM wall is NOT seed-caused. (This matches SH206/155's intern finding but
now with the fresh A/B that removes the "maybe our seed masks the lambda's output"
residual.)

## Measured result 2 — the app-shell ctor 0x102207b50 body RUNS DEEP (61+ blocks)

Fresh region-watch on [0x102207b50, 0x102209000) with the SH156 seed present:
**61 distinct block-entry pcs** walked through the ctor body to 0x102208eac,
including the app-data-model register appends (0x102208354, SH203-known),
store/verify, and a terminal tail whose target is the **FMOD/AAudio iterate
region 0x5fb30b4 (`sub sp,#0x60`, then adrp x20,67d1000)** — the sound pillar
first-contact (SH212/213-class), not a soft-return at the ctor head.

**Qualification:** "do-init never completes app-shell construction" is too
coarse. The do-init's app-shell/global-init ctor **executes a large real body**
when the gen DM-root seed is present, and terminates by tailing into the audio
subsystem. What never happens is a `make_shared<DataModel>`: the SH174 capture
latch fired only small 0x18-byte FIRST allocs (theme/registry objects), none
validated as a DataModel. So the Route-B live-DM structural gate (SH209/218/223/
224/228/231/232/235/236/237/238) is **UNCHANGED**, but the mechanism label is
corrected: the wall is at make_shared<DataModel>, downstream of a fully-executing
app-shell ctor.

## Code (+2 hermetic, real-image guard family as sh237/236/235, skip-if-absent)

- `sh239_..._pinned`: byte-pins the once-lambda completion store (0x102206d74 =
  0xf90206e0 `str x0,[x23,#1032]` -> once-slot [0x106a68408]; 0x102206d78 =
  0xd0024300 adrp x0,6a68000), the do-init match terminal (0x102206e24 =
  0xd61f0020 `br x1`), the app-shell ctor entry/prologue/oncel-guard
  (0x102207b50=0x14000001 `b +4`, 0x102207b54=0xa9be7bfd `stp`, 0x102207b68=
  0x08dffd08 `ldar` reads [0x106a64d70]), and the ctor terminal-tail FMOD iterate
  (0x105fb30b4=0xd10183ff `sub sp,#0x60`) + 4-alignment/in-window.
- `sh238_..._select_slots_relocated`: pins the receiveCall dispatch band
  [0x635d970,0x635e700) still carries RELATIVE relocs covering the two select
  slots (+0x20=0x635dd88, +0x28=0x635dd90) so the SH237 loader-synthesis
  conclusion fails loudly on drift.

No production path edited. Both pass on the real libroblox.so (1 passed each,
83 filtered).

## Reproduce / verify

```
# A/B (seed off): the once-lambda leaves DM-root [0x106a68818]==0 even though
# oncel-guard self-latches 0->1 (once-slot intern 0x400000b).
timeout 115 env JIT_DRIVE_LIFECYCLE=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
  JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 JIT_ROUTEB_V2_ONDEMAND=1 \
  JIT_DM_ALLOC_CAPTURE=1 JIT_DM_ALLOC_CAPTURE_DELEGATE=1 JIT_GUEST_STACK_DUMP=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-send-appevent \
  | grep -E "SH155 post-StartLuaAppDM"
# expected: ... DM-root[0x106a68818]=0x0 ... once-slot ... =0x400000b
```

## Measured result 3 — the do-init match does NOT dispatch through the DM-root cell (closes the "surface the genuine DM into the match" lever)

Fresh A/B with an opt-in `JIT_ROUTEB_DM_ROOT_GENUINE=1` (SH156 seed-site variant):
instead of the fake 0x10-byte box (vt 0x10635cce0), plant the **manufactured
genuine-vptr RBX::DataModel** (vt 0x1067162e8, SH187) at `[0x106a68818]+0x20` and
pin its primary vtable `+0x30` -> the real DM app-shell ctor **0x1057d6ef4**.
Region-watch [0x1057d6ef4,0x1057d7100) + the GlobalInit ctor [0x102207b50,0x102209000):

| DM-root variant | app-shell ctor 0x1057d6ef4 hits | GlobalInit ctor body pcs |
|---|---|---|
| fake box (SH156 default) | **0** | ~79 blocks |
| genuine-vptr DM (SH239 opt-in) | **0** | ~79 blocks |

**Conclusion:** the two DM-root layouts dispatch IDENTICALLY — the do-init match
does NOT read `[0x106a68818]+0x20` for its dispatch target; it selects through the
StartLuaAppDM stack-union table (0x635dd68 [+0x30] -> 0x1023eff4c, SH156/237) and
the GlobalInit vtable path, never through the DM's own vtable. `[0x106a68818]` is a
sink for the once-lambda result (the intern 0x400000b), not the source of the
dispatch. Both runs EXIT 124, 0 crash. So the carefully-planted genuine DataModel
vtable in the DM root changes nothing — the match was never going to reach
0x1057d6ef4 this way. This closes the "surface the manufactured DM into the do-init
match" lever with a clean A/B (single-agent, no fabricatable-object-graph required).
The DM's own app-shell ctor 0x1057d6ef4 remains reachable only via its owner
dispatch (BPATH / a live engine session), the standing live-DM gate.

This cycle is forward recon + a measured A/B falsification + regression pins —
it does NOT manufacture a DataModel and does NOT lift the Route-B live-DM
structural gate. It removes a wrong "maybe our seed masks the once-lambda
output" residual (SH239) by proving the lambda yields an intern regardless. The
single forward hook stays SH174 capture-latch arming *(0x106391908) at a real
make_shared<DataModel>. recon-v3 deliverables (24 frames / json zero-fix) stay
shipped + verified.