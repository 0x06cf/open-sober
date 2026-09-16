# SH197 — Route-B do-init continuation chain executes REAL code headlessly at HEAD

Worker: hermes-worker · date 2026-09-16 · workspace green (arm64jit 381/0 incl. new test)

## 1. Region-watch multi-range fix (code)

`JIT_REGION_WATCH` previously parsed a **single** `lo-hi` pair (`split_once('-')`); a
comma-separated multi-region value was **silently ignored** (never matched, no warning),
which a diagnostic run reads as "the region wasn't entered" — i.e. it **fabricates a
recon/region negative**. That is exactly the class of tooling footgun that leads Route-B
recon to declare a gate "not reached" when it was simply never *watched*.

Fix (crates/arm64jit/src/jit.rs):
- new pure helper `pub fn region_watch_contains(spec: &str, pc: u64) -> bool` — parses
  comma-separated `lo-hi` ranges ([lo,hi) semantics), a malformed entry is skipped, a
  wholly-malformed/empty spec matches nothing and never crashes.
- the block-entry site now iterates all comma-separated ranges, dedups logged pcs, and
  emits a once-only WARN when the spec is unparseable ("no region will be watched").
- +1 hermetic test `region_watch_contains_multiple_ranges_and_parse_err`
  (back-compat single range, boundary [lo,hi), multi-range, malformed-sibling, garbage spec).

## 2. Empirical HEAD finding: do-init continuation executes real engine code

Earlier sessions (SH155/SH158) concluded "the do-init body 0x1023eff4c reaches the
AppBridgeV2 governor, then the governor is NOT entered (region-watch empty)". That negative
was measured with the fragmented single-range watch. With the fix, one seeded ladder run
watches all three regions simultaneously and reports:

```
[region-watch] region hit at guest pc=0x1023eff4c   <- do-init post-body
[region-watch] region hit at guest pc=0x1023effa0/0xac/0xc0/0xc8/0xd0/0xfc
[region-watch] region hit at guest pc=0x102207b50    <- app-shell ctor ENTRY
[region-watch] region hit at guest pc=0x102207b88/0xb90/0xbbc/0xbe0/0xbe4/0xc28/0xc2c
[region-watch] region hit at guest pc=0x102e9fa84    <- governor 0x102e9fa84 ENTRY
[region-watch] region hit at guest pc=0x102e9fb58 ... 0x102e9fdc8   <- governor tail (SH161)
[region-watch] region hit at guest pc=0x102e9fe0c/0xfea4/0x102ea3084/0x102ea30dc
```

That is the **entire do-init -> app-shell ctor -> governor -> governor-tail continuation
running real relocated engine code headlessly**, 0 SIGSEGV/0 SIGABRT, EXIT 124 (stable
idle), StartLuaAppDM returns a real heap addr (not the benign 0x3e8), governor vtable
resolves to real `0x102e9fa84` at runtime (`vt[+0x18]@0x1063a3428=0x102e9fa84,
vt18_is_gov=true`).

This matches and CONFIRMS SH196's "do-init once-lambda COMPLETES" at the moved frontier:
the engine now self-drives from GlobalInit do-init through the app-shell ctor (its own
`__call_once` on ctor-guard [0x6a64d70] engages 0->1) into the AppBridgeV2 governor and its
tail — the manufacture/DMCONT continuation the operator's "MIGRATION IS NOT A STOPPING
POINT" directive demands is not parked; it executes. The remaining hard gate beyond the
governor tail is the live-DM world-build (once-slot 0x400000b is an RTApp intern, not a
DM) and the R1 content path — SH196/SH170 standing.

## 3. Reproduce

```
# one run, all three continuation regions watched:
JIT_DRIVE_LIFECYCLE=1 JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
JIT_REGION_WATCH=0x1023eff4c-0x1023f0000,0x102207b50-0x102207c40,0x102e9fa84-0x102ea3b40 \
./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
--jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent
# expect: region hits in all three regions, EXIT 124, 0 SIGSEGV/0 SIGABRT
```

## 4. Honest boundary

Run-variable: V2InitWithParams/V2StartAppWithParams occasionally stop at a tiny
`pc 0x229/0x188 outside image` (unseeded singleton-vtable host-pointer class, SH176/SH103/
SH109) — not this cycle's focus, and the continuation-chain finding is independent of it
(the rung that matters, StartLuaAppDM -> do-init continuation, completes cleanly).