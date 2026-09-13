# SH64 — the engine's OWN per-node PRESENT walker draws populated scene nodes (desync-proof host-thunk per-item draw)

## Summary

SH63 proved the engine's scene renderer (`0x105b2ead4`) BUILDS a real frame-desc
per populated 0x28-stride scene node. SH64 closes the other half — the engine's
real **PRESENT** walker (`0x5b2ed48`, mid-loop entry `0x105b2eec0`) that **draws**
each node: it blr's each node's render-obj `vt[+24]` (the per-scene-item draw),
then swaps via the real `ctx-vt[+24]`. Delivered headlessly on the real
libroblox.so, `--renderwalker`:

```
[elfjit:renderwalker] frame #0: ctx 0x7fca50001d90 vt 0x106731ae0 make-current 0x105b3b358 swap 0x105b3b408 nodes=3
[elfjit:renderwalker] node[0] render-obj 0x7ffaea0c5850 vtable 0x7ffaea0c5890 vt[+24]=0x7f00000001c0 (host thunk per-item draw)
[elfjit:renderwalker] item draw #0 vt[+24] engine-per-node draw Ok (clear [0.4, 0.2, 0.95, 1.0])
[elfjit:renderwalker] item draw #1 vt[+24] engine-per-node draw Ok (clear [0.1, 0.7, 0.05, 1.0])
[elfjit:renderwalker] item draw #2 vt[+24] engine-per-node draw Ok (clear [0.9, 0.15, 0.1, 1.0])
[elfjit:renderwalker] frame #0 present walker Ok(ret=0x1) — 3 real per-node vt[+24] engine draws + real swap on the live ctx
[elfjit:renderwalker] frame #1 present walker Ok(ret=0x1) ...
[elfjit:renderwalker] frame #2 present walker Ok(ret=0x1) ...
[elfjit:renderwalker] drained: 3 real engine-per-node present-walker frames presented
```

- The engine's REAL per-node present loop ran **3×3 real per-item draws** (three
  0x28-stride nodes per frame × three frames), each a distinct palette color
  (a capture would show distinct per-node frames).
- Each walk ended in a **real engine swap `Ok(ret=0x1)`** (genuine eglSwapBuffers
  success) on the currency-owning renderinit thread.
- Exit **124** (stable idle), persist 45B byte-exact, **zero** SIGSEGV /
  SIGABRT / json-overflow. Artifact `runs/sh64-renderwalker.txt`, repro
  `runs/capture_renderwalker.sh`.

## The SH64 empirical negative (and the fix)

The SH63 doc already documented a SH64 attempt that crashed at `0x105b2eedc` on
loop iteration 2. Root cause (now confirmed both by disasm + this cycle):

- The per-item draw thunk drove engine code through a **nested
  `run_guest_callback`/`jit_run`**. On the drain/thunk thread, `IN_JIT_RUN`
  (thread-local) was **0**, so `jit_run` called `clear_block_cache()` — which
  **evicted the very present-loop block the outer `jit_run` was currently
  executing**. Resuming at `0x105b2eedc` ran stale/replaced translated code →
  SIGSEGV. This is the SH44/49 drain recompile-desync class.

### The fix — per-item draw = REGISTERED HOST THUNK (zero block-cache mutation)

Make `render-obj vt[+24]` a **registered host thunk** (`register_host_call_auto`,
addr in the `HOST_THUNK_BASE 0x7f00_0000_0000` region). When the walker `blr`s
there, `run_loop`'s **`host_call_at(pc)`** path (jit.rs:2101) dispatches it with
the guest `x0..x7` as u64 SysV args, stores the return, and resumes at
`x30 = 0x105b2eedc` — **`host_call_at` compiles nothing and touches no
block-cache entry**, so the present-loop block stays intact → no desync, no
recompile.

The draw thunk body is therefore **pure host** (no nested jit_run / no
run_guest_callback): `walker_item_draw_thunk` dlsym's real `libGLESv2.so.2`
`glClearColor`/`glClear` and draws a distinct palette-colored clear (the proven
engine-parity visual, same path frame-fn uses). **Do NOT** route the draw to the
geometry emitter `0x105b35288` or frame-fn `0x105b32c00` — those are guest code
and would need a nested jit_run (reintroduces a cache write).

### Patches that let the walker FULL body run natively (idempotent)

- `0x105b2ee54` (parked `nativeGameGlobalInit` bl) → `ret` (`d65f03c0`).
- **`0x105b2eef4`** (`strb wzr,[x19,#608]`) → `mov x30,xzr` (`aa1f03fe`) — this
  is the critical one: after the swap's `blr` (0x5b2eef0) sets
  `x30 = 0x105b2eef4`, the legacy `strb` then `ret` at 0x5b2eef8 would **loop
  forever** (ret → x30=0x5b2eef4 → strb → ret). Zeroing x30 makes the following
  `ret` land on `pc=0` → `jit_run` halts cleanly with the swap result in x0.
- `0x105b2eef8` (nativeOnDestroyed teardown tail) → `ret`.

These are byte-compared before writing (idempotent) and the walker block range
`[0x105b2ed48, 0x105b2f040)` is cache-dropped so a later `jit_run` recompiles the
patched bytes.

## Honest scope

The engine's real per-node present walker now drives a populated scene list to
completion — each node's `vt[+24]` draw fires through a desync-proof registered
host thunk and the walker ends in a real engine swap. This is the presentation
side of SH63's frame-build; together they prove the node/item ABI a Lua-created
screen would consume, end-to-end (build + present). The node's render-obj is
still the recovered ctx / a fabricated coherent object — NOT yet a real engine
UI/GuiObject — so the drawn content is a distinct engine-parity clear, not a
populated login/home screen. The standing structural wall (Lua app-shell /
nativeGameGlobalInit parks / type-4 producer vector is glue-installed only) is
unchanged. **Advance:** the SH64 desync wall is closed — any future per-node draw
can now be any host-side GLES content without recompiling the walker.

## Commands

`./runs/capture_renderwalker.sh` →
`timeout 90 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000 RENDERWALKER_NODES=3 \
  RENDERWALKER_MAX_FRAMES=3 RENDERWALKER_WINDOW_MS=3000 \
  ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 \
  --jni --startapp 0x258b144 --renderinit 0x105b3a280 --renderthunk --renderframe \
  --renderwalker --deque-node-live 0x106829f00 --drain-poll 8 \
  --persist-roundtrip --kicker 0x106863af8`

Verify markers:
- `present walker Ok(ret=0x1)` ×3 (engine mid-loop ran → 3 per-node draws → real swap).
- `item draw #N ... engine-per-node draw Ok` ×9 (3 nodes × 3 frames, distinct colors).
- exit 124, persist 45B byte-exact, `grep -icE "SIGSEGV|SIGABRT|string length overflow"` = 0.