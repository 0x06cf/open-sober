# SH152 — type-4 task-driven frames carry REAL home artwork (cached-program design)

## Status
The self-driven task-frame plane (recon-v3 deliverable 1) now presents the
**real Roblox home surface** — FPSBackground.png launcher backdrop + the
RO-BLOX wordmark — through the engine's OWN geometry emitter, driven by type-4
task dispatch. The flat-palette path REMAINS the default and is unregressed.

## What changed (elfjit.rs, `present_one_task_frame`)
- Threaded the renderinit thread's guest `iimg`/`ibase`/`isp` into
  `present_one_task_frame` (both call sites in the presenter window).
- New opt-in env **`RENDER_TASKFRAME_HOME=1`**: when set, the type-4 dispatch
  calls `render_engine_emitter_multi` (real artwork, `RENDEREMITTER_HOME=1`)
  or `render_engine_emitter_home` (palette fallback) as a SEPARATE top-level
  `jit_run`, then presents via the SAME real ctx-vt[+24] swap — a genuine
  task-driven present of real engine content.
- Selector is the ENV, not the emitter return value: these engine callbacks
  `Ok(ret)=0` on success AND on early `return 0`, so the return value cannot
  distinguish success from failure. Env-off path is byte-identical (verified).

## Why this is a real advance (not the SH151 negative)
SH151 **hard-failed**: feeding real mesh geometry into a task frame via
NESTED guest-bridge GLSL compile + GL object allocation (inside the frame-fn
callback) on the currency thread corrupts the host heap (`malloc(): corrupted
top size` / SIGSEGV). SH151's own "do not reopen" note sanctioned ONE safe
design: *"reuse the renderframe-thread's already-current program."*

SH152 does exactly that: the emitter-home/multi path uses the **cached**
`emitter_tex_program` + a **pre-uploaded** texture and runs as its own
top-level `jit_run` — zero nested GLSL compile, zero nested GL allocation on
the presenter thread. This is the proven SH66/SH150 emitter machinery, now
driven by the type-4 dispatcher rather than the walker block.

## Verified (real libroblox.so, headless llvmpipe)
- `RENDER_TASKFRAME_HOME=1 RENDEREMITTER_HOME=1`: type-4 dispatch logs
  `task frame #0 REAL HOME ARTWORK (engine emitter) ret=0x0`; the multi-emitter
  emits `count=78 sprites=12 swap=Ok(1)`; a real FPSBackground probe readback
  on the task frame reads `(175,95,59,255)` — real artwork texels, not the flat
  palette — 0 SIGSEGV/SIGABRT, EXIT 124 (clean idle).
- Default (env unset): **EXIT 124, 24 task-driven frames, 0 crashes, home
  branch not entered** — no regression from the signature threading.

## Honest scope
Harness gates the emitter call; the artwork is the real APK's. This is the
engine's own GLES path presenting real content, but it remains
harness-authored geometry/selection (SH131d structural wall: engine
self-constructed login/home still behind the Lua app-shell — unchanged). The
advance here is that the *task-driven* producer (deliverable 1) now carries
real engine content instead of a flat color, closing the SH151 value gap via
its own sanctioned design.

## Repro
`runs/capture_taskv4_frame_home.sh` (log `runs/sh152-taskframe-home.txt`).