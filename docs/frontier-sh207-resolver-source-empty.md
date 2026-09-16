# SH207 — resolver-map gate closed at fresh depth: registrar SOURCE measured EMPTY + registrar is a no-op (not corruption-destined)

**Session type:** verification/characterization + a genuine production fix (futex
flake). No new seed warranted. Workspace green (386/0 arm64jit + all crates / 15
test binaries). Commit <SH207COMMIT>.

## 1. Production fix: the `futex_requeue_actually_moves_waiter` flake (JIT test to green)

`cargo test --workspace` intermittently FAILED `jit::tests::futex_requeue_actually_moves_waiter`
under full parallel load (it passed isolated every time). Root cause:
the waiter's `futex WAIT` uses a **5-second wall-clock timeout** while the
REQUEUE side spins for up to ~1 ms * 20,000 = **20 s**. Under heavy parallel
`cargo test --workspace` load a descheduled waiter thread can sleep past 5 s
BEFORE the REQUEUE lands, unblocking on the timeout — then REQUEUE legitimately
moves 0, the `assert!(moved >= 1)` fails, and the run aborts a **timing flake, not
a regression**.

Fix (production-adjacent, in the test itself, assertion untouched): bump the
waiter timeout `tv_sec` 5 → **60 s** so it safely exceeds the entire 20 s spin
window; only a genuinely-broken requeue can strand the waiter now.

- Verified: the test passed isolated 8/8 and, after the fix, the FULL parallel
  `cargo test --workspace` is green (386/0 arm64jit + all other crates; the
  previously-flaky test now passes under load).

## 2. Fresh measurement: the bulk-registrar SOURCE map (0x106dca0e90)

Prior Route-B recon (SH191/192/194) read only the resolver DEST map
(0x106dca0e70) and the register map (0x106dca0f60), concluding the resolver is
unconstructed headlessly. It **never read the bulk registrar's SOURCE**
(0x106dca0e90) — the container the in-ladder registrar 0x2208ae8 (in
nativeGameGlobalInit) copies class-name entries FROM into the resolver dest.

New default-inert probe in `routeb_dm_service_resolve_guard` (jit.rs, under the
existing JIT_ROUTEB_DM_SERVICE_NODE env; moved to the TOP of the guard so it
always reports regardless of instance-capture state) reads all three maps:

```
2/2 clean runs (real libroblox.so, --v2boot ladder, EXIT 124, 0 crash):
[routeb-dmsvc] SH192+: resolver(dest) 0x106dca0e70 = {0x0,0x0} (EMPTY)
                       registrar-source 0x106dca0e90 = {0x0,0x0} (EMPTY)
                       register 0x106dca0f60 = {0x0,0x0} (EMPTY)
```

And a region-watch run confirms the bulk registrar IS exercised in-ladder
(JIT_REGION_WATCH 0x102208ae8-0x102208e40 fires at 0x102208ae8/0xb6c/0xcf8/0xe38),
reaching its rehash + insert-path blocks. But because BOTH the source (0xe90)
and dest (0xe70) are EMPTY, the registrar's copy loop has nothing to iterate —
it cleanly no-ops. This **reconciles** SH191/192/194's "resolver stays EMPTY"
with SH194's region-watch "registrar region fires every run": the registrar runs
but is a no-op on empty in+empty out, NOT corruption-destined.

## 3. Verdict (do-not-re-tread, now at fresh source-depth)

- The resolver map (name→classid, read by walker 0x105e09bc8 → resolver
  0x2373cec) is populated ONLY from the bulk registrar source 0x106dca0e90, which
  is **EMPTY headlessly** (measured). The source is filled by a live
  class-registry world-build (SH192/193/194 conclusion holds; now with the source
  itself read, not just the dest).
- `getService("PlayerGui")` name→classid resolution (and the linked-node RETURN
  from SH191's walker drive) therefore sits behind the same live class-registry
  world-build gate as the live-DM. SH192/194's `bad_weak_ptr` from *driving the
  registrar standalone with a fabricated key* is a separate hazard (registrar is
  a mid-nativeGlobalInit fragment; standalone it hashes against unconstructed
  container state). We did NOT repeat that; we measured only.
- Route-B live-DM / class-registry world-build = structural gate UNCHANGED (SH174
  /193/194/196/203/204/206 + this). No headless seed populates the resolver.

## 4. Shipped

- Futex-flake timeout fix (`jit.rs` test).
- Source/dest/register three-map read-back diagnostic (jit.rs, default-inert,
  relocated to always-fire at the top of the guard). +1 hermetic pin for the
  three registry addresses (pure address/offset assertions).
- Doc addendum (this file).

## Repro
```
env JIT_DRIVE_LIFECYCLE=1 JIT_ROUTEB_DM_MANUFACTURE=1 JIT_ROUTEB_DM_REALCTOR=1 \
  JIT_ROUTEB_DM_SEED=1 JIT_ROUTEB_DM_SERVICES=1 JIT_ROUTEB_DM_INSTANCE=1 \
  JIT_ROUTEB_DM_SERVICE_NODE=1 JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 \
  JIT_ROUTEB_SETFIX=1 JIT_SH115_SINGLETON_PATCH=1 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent
grep 'SH192+' <log>   # resolver(dest) + registrar-source + register all EMPTY
```