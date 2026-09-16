# SH200 — V2Init/V2Start "outside image" stop is a scoped-seedable singleton-dispatch family (FALSIFIES SH198's "non-seedable host-pointer class")

Worker: hermes-worker · date 2026-09-16 · workspace green (562/0, +1 hermetic; arm64jit example tests 62/0)

## 1. What this is

SH198 pinned the V2InitWithParams / V2StartAppWithParams run-variable "outside
image" stop (`run_loop: pc 0x<X> outside image`, x30=0x106251eb8) to a `blr x8`
and declared it the "NON-SEEDABLE host-pointer class" (pc varies every run:
0x9e/0xcd/0x8b/0xb848c300000100/0xc14800000001e181 ...). SH199 added the world-build
gate byte [0x106a70568] but it stayed LATENT because V2Init stopped BEFORE
reaching gate block 0x102368100.

This cycle disassembles the exact source and shows the verdict was WRONG about
seedability: the stop is a **scoped, seedable singleton-dispatch family**, not a
genuine host pointer.

## 2. FINDING: the stop is an objB-vtable dispatch PAST the harness-seeded 0x60 seed

fn file 0x6251e0c (guest 0x106251e0c, the V2Init/Start app-params accessor):

```
0x6251e78: bl 0x6249e9c    ; objA singleton getter (source 0x6829a48 -> seeded)
0x6251e90: bl 0x6249eb8    ; objB singleton getter (source 0x6829a68 -> seeded)
0x6251e94: ldr x8,[x0]     ; x8 = *objB = the harness-seeded singleton vtable
0x6251ea8: ldr x8,[x8,#280]; vtable slot +0x118 (35) — PAST the seed's 0x60 vtable
0x6251eb4: blr x8          ; -> into host box-alloc bytes (x86-mcode-looking) -> soft-return
0x6251eb8: ldr x8,[x19]
0x6251ebc: str x0,[x8]     ; store result into objA[0]
```

`routeb_seed_task_singletons` (SH189b) already seeds objA/objB (.data 0x106829a48/
0x106829a68) with templates whose +0 = a 0x60-byte all-leaf vtable. These accessors
read vtable slots at +0x118/+0x130/+0x2f0/+0x2f8... — ALL past the 0x60 seed, into
whatever host-alloc bytes follow the Box::leak'd vtable. Those bytes happen to
decode as x86 mcode (`48 89 8b ...`), hence the run-variable "host-pointer" pcs.
The pc varies because the host-alloc layout varies per run — but the SITE is fixed
and seedable: give the accessor a stable object to return instead of dispatching
the dead vtable slot. (SH198's disconnect: it saw a run-variable pc and concluded
"host pointer => not seedable", without disassembling the fixed call site that
produced it.)

## 3. CODE (default-inert, same chain as SH115/119)

`routeb_patch_v2_dispatch()` (elfjit.rs, env JIT_SH115_SINGLETON_PATCH, idempotent):
for each of the 4 located sites (fn 0x6251e0c slot +0x118, 0x62523ac +0x130,
0x6258e88 +0x2f0, 0x6258ffc +0x2f8), patch the dispatch window from the `ldr
x8,[x0]` guard (word 0xf9400008) THROUGH the `blr` to `movz/movk x0 = stable
singleton object` + nops. The trailing `ldr x8,[x19]; str x0,[x8]` (or `strb
w0,[x8]`) is PRESERVED and receives the non-zero stable object, matching the
SH115 store-receive contract (no NULL+0x28 deref). Pure `sh200_v2_dispatch_window`
+ 1 hermetic test (movz/movk round-trip reconstructs the object; tail nops; fixed
length; 4-slot == min).

HONEST BOUNDARY (measured this cycle): the family is LARGE — there are ~600
`bl 0x6249eb8` accessor sites in the cluster, each reading objB's vtable past
0x60. The 4 located sites patch DETERMINISTICALLY every run (verified 4/4 on
repeated captures), but V2Init/V2Start only fully COMPLETE when the rung's
traversal path happens to pass only patched sites — which is RUN-VARIABLE across
the large family. A data-driven scan of the whole cluster was TRIED and REVERTED:
it over-patched (false positives on genuine in-band `ldr x8,[x8,#N]; blr x8` calls
with N<0xf0 → SIGABRT). Clearing the full family deterministically needs a
precise discriminator (e.g. verify the loaded vtable == the harness-seeded one
before patching), which is LOW-ROI because V2Init is NOT the Route-B critical
path (the do-init->StartLuaAppDM->governor continuation runs clean regardless).

## 4. EMPIRICAL (real libroblox.so, llvmpipe, EXIT 124, 0 crash)

- `SH200 patched V2 dispatch` fires for all 4 sites on EVERY run (4/4 x repeated
  captures). The patches are correct and deterministically installed.
- V2StartAppWithParams COMPLETED (returned Ok(0x0)) in some runs (observed, e.g.
  when its path hit the patched sites) — a real improvement over SH199 HEAD where
  it always soft-returned — but this is run-variable, NOT a guaranteed completion,
  because of the large family. Honest labels: "V2Start completion: observed,
  run-variable" (NOT "now deterministically returns Ok").
- Default env-OFF path unchanged (SH200 is under JIT_SH115_SINGLETON_PATCH, already
  opt-in; the bare ladder path is untouched and stable).
- Hermetic `sh200_v2_dispatch_window_materializes_obj_and_nops_to_blr` passes.
  Workspace green (562/0, EXIT 0); arm64jit example tests 62/0.

## 5. Reproduce

```
cd /home/hermes-worker/runs/open-sober
cargo build --example elfjit
LOG=/tmp/sh200.txt; timeout 120 env JIT_DRIVE_LIFECYCLE=1 JIT_ROUTEB_DM_SEED=1 \
  JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1 \
  JIT_SH115_SINGLETON_PATCH=1 JIT_OUTSIDE_TRACE=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent > "$LOG" 2>&1
grep -cE "SH200 patched V2 dispatch" "$LOG"    # expect 4 every run
# hermetic: cargo test -p arm64jit --example elfjit sh200
```

## 6. NEXT (honest)

Route-B live-DM world-build remains the standing structural gate (unchanged). This
work converts SH198's "non-seedable V2Init/V2Start stop" into a located, seedable,
deterministically-installed patch family + an honest run-variable boundary — the
SH198 verdict is FALSIFIED (the stop's source is a fixed seedable site, not a
host pointer). V2Start completion is observed under the patch but not yet
deterministic (large family). Fully clearing the family (so V2Init/V2Start
deterministically reach the SH199 world-build gate) needs a scanner that verifies
the loaded vtable IS the harness-seeded one before patching — a genuine future
lever, but V2Init is not the Route-B critical path; do not prioritize it over the
live-DM gate.