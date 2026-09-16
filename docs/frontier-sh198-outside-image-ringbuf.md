# SH198 — Deterministic diagnostic for the run-variable "outside image" stop

Worker: hermes-worker · date 2026-09-16 · workspace green (382/0 arm64jit lib, +1 hermetic)

## 1. Context: what the run-variable stop was

Every clean ladder run reports the V2InitWithParams / V2StartAppWithParams rungs
"stopped: run_loop: pc 0x<X> outside image [0x100000000, 0x107333c3c)" where the
stop pc varies run to run (measured this cycle: `0x9e`, `0xcd`, `0x8b`,
`0xc0e0814818e8`, `0x40438948028b4800`, `0xf8838b48028948`, `0x55e749b6b910`).
That is the SH176/SH103/SH109 "singleton-vtable host-pointer" class: an unseeded
object's vtable slot holds a HOST x86-64 mcode pointer (the bytes `48 89 8b 48 ...`
at `0xf8838b48028948` decode as x86 `mov`, confirming a host code address leaked
into a guest vtable slot), and a guest `blr` through it leaves the image.

Because the V2 rungs still return a benign "stopped" (clean EXIT 124, 0 crash)
and the do-init -> app-shell ctor -> governor -> governor-tail continuation is
unaffected, this was treated as a benign run-variable flake. But **the source was
not statically knowable** — the only way to see it was a full `JIT_TRACE`
register/step dump (megabyte logs).

## 2. Code: bounded ring-buffer outside-image tracer (default-inert)

`crates/arm64jit/src/jit.rs`:
- `jit_run_inner` now keeps a tiny `[u64; 16]` ring of the last block-entry pcs
  (2 stores per iteration, unconditional — negligible; 16 is a power of two so the
  `& 15` wrap is exact). This is cheap enough to leave ON production.
- At the `outside image` stop, IF `JIT_OUTSIDE_TRACE` is set, print the ring
  oldest->newest (the last in-image pcs) + the bad pc so the exact transition out
  of the image is visible **without** the megabyte `JIT_TRACE`.
- New pure helper `ring_ordered(&[u64], ring_i) -> Vec<u64>` (module-level,
  power-of-two masks; skips unwritten zeros). +1 hermetic test
  `ring_ordered_reports_oldest_to_newest_and_skips_unwritten`.

## 3. Empirical (real libroblox.so, llvmpipe, EXIT 124, 0 crash)

With `JIT_OUTSIDE_TRACE=1` (no JIT_TRACE) the source of the V2 stop is now
deterministic and visible in a few lines:

```
[nativeInitializeNativeFlags]
  [outside-image] recent block pcs (newest last):
  [0] 0x102b9df10
  [1] 0x106249ecc
  [2] 0x1062514c0
  [3] 0xf8838b48028948        <- bad pc (host x86 mcode bytes 48 89 8b)
  [...] (pattern repeats: loop 0x2b9df10 -> 0x6249ecc -> 0x62514c0)
  pc(now)=0xf8838b48028948 x30=0x1062514e4

[V2InitWithParams]
  [0] 0x102b9df10 [1] 0x106249ecc [2] 0x106251e94 [3] 0x40438948028b4800  <- bad
  [...] pc(now)=0x40438948028b4800 x30=0x106251eb8
```

**Root cause:** the bad pc comes from a `blr x8` at the tail of file 0x6251c..0x6252a
(`ldr x8,[x0]; ldr x8,[x8,#280]; blr x8` at 0x6251eb4 / `[x0,#24]` loaded first),
dispatching through a HOST-HEAP object `x0=0x55e...` (guest heap leak) whose vtable
slot +0x118 holds a host-mcode pointer. It runs in a small loop anchored at
`0x102b9df10` (file 0x2b9df10, a strcmp/hash-bucket walker: `ldr x21,[x0,#8];
cmp x21,x22; b.cs ...; bl 0x2173b94`) — a string/registry lookup that in a bare
boot has no live entry, so it dispatches the not-found/placeholder vtable.

This is the same class as all prior SH176/SH103/SH109 stops — **a HOST pointer,
not a guest address, so it is NOT a static-seedable slot** (the pc is a heap/mcode
address and varies every run). It is a registry/string container reached only
after a real session populates it (same live-world-build dependency as the
Route-B resolver map, SH193/194/197b). Verdict: the residual V2Init/V2Start stop
is a documented non-seedable flake; it does not block the Route-B continuation
chain (SH197 re-verified clean this cycle).

## 4. Why this matters (ROI)

- Turns a "run-variable flake, can't know the source" into a 3-line deterministic
  diagnostic that FALSIFIES the possibility of a static seed (host pointer) and
  pins the exact guest dispatch site (0x6251eb4) + the loop anchor (0x2b9df10) —
  so any FUTURE attempt to fix the V2 rungs starts from evidence, not guesswork.
- No need to re-run megabyte JIT_TRACE to find an outside-image source again.
  Default-inert: only prints under JIT_OUTSIDE_TRACE.

## 5. Reproduce

```
cd /home/hermes-worker/runs/open-sober
cargo build --example elfjit
LOG=/tmp/sh198.txt; timeout 100 env JIT_DRIVE_LIFECYCLE=1 \
  JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
  JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 JIT_OUTSIDE_TRACE=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent \
  > "$LOG" 2>&1
# expect: [outside-image] recent block pcs ... (repeating < 10 lines), EXIT 124, 0 crash
```

## 6. Honest boundary / next

The Ring Buffer + diagnosis is shipped. The V2 stop's underlying cause (live
registry/object world-build) is the SAME migration-gated dependency as the
Route-B live-DM/resolver map — no new headless seed was claimed. The governor
MODERN startAppWithParams blob-path (0x258c6e4) was confirmed REACHED this cycle
(region-watch 0x10258b144 + 0x10258c6e4 both fire), closing SH197 §6 lever (a)'s
"does the blob-path even execute" question with a positive; the residual stop is
non-seedable. Route-B live-DM world-build remains the standing structural gate.

## 7. Governor-tail terminal (extends SH197, same cycle)

Watching the FULL governor region [0x102e9fa84, 0x102ea4000] (past the SH197
0x30dc floor): the tail's deepest execution is `0x102ea3084/0x102ea30d0/
0x102ea30dc` — a stack-canary `ret` thunk (`ldp x29,x30,[sp,#32]; ret`). Execution
NEVER reaches the larger fn at 0x2ea3b14 (`ldr x0,[x0,#688]`, the deeper
app-bridge/world-build body). So the manufacture/DMCONT continuation ends cleanly
at the governor-tail ret; the next real construction fn (0x2ea3b14) is unreached
headlessly — the same live-world-build gate, now measured one level deeper with 0
crash / EXIT 124.