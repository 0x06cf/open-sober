# SH151 (verified-negative) — feeding real mesh geometry into task-driven frames

**Attempted:** implement the recon deleg_fe889bc2 recommendation to feed the
engine's proven coherent-renderer mesh path into each type-4 task-driven frame
(`present_one_task_frame`, elfjit.rs:4940) so a task frame carries a real
engine-rendered object (smooth sphere + studs.dds) above the flat palette.

**Approach (per recon):** a self-initializing free fn `taskframe_mesh_draw_one`
(on the currency-owning presenter thread, EGL current) that lazily builds its own
GLSL program + studs texture via `run_guest_callback_on` (cached leaked 1 MiB
stack) and per-frame drives the engine geometry wrapper 0x105b35288 through the
coherent renderer, gated on `RENDER_TASKFRAME_MESH=1`. The gated call sits between
the clear-path frame-fn Ok and the swap. Default (env off) = byte-identical to the
proven 24-frame path.

**Outcome — HARD NEGATIVE (reverted, not shipped):**
1. First build: SIGSEGV at `f3 a4`/`rep movsb` inside the nested GLES
   `glShaderSource` bridge (corrected the ABI to `(shader, 1, &slot)` per the
   proven closure path).
2. Second build: SIGABRT with `malloc(): corrupted top size` — host heap
   corruption INSIDE the nested `run_guest_callback_on` GLES bridge on the
   presenter thread, still with 0 frames presented (crash before the sphere draw).

**Why it's a HARD wall (empirically), not an ABI nit:** the presenter thread is
already *inside* a `run_guest_callback` (the frame-fn). Nested GLES bridge calls
that compile/link GLSL and allocate GL objects on the live Mesa context from that
nested callback corrupt the host heap — the same risk #1 ("goal-path mutation")
the recon explicitly flagged. The flat-clear path (`frame-fn` only) works because
the engine's own clear path does not round-trip GLSL compile + GL object
allocation through the host bridge on this thread.

**Cleanup:** reverted both `crates/arm64jit/examples/elfjit.rs` and
`runs/capture_taskv4_frame.sh` to the SH150 commit. Re-verified the default
env-off path is UNREGESSED: 24 real task-driven frames, `present #N swap Ok(0x1)`
for N=0..23, EXIT 124 (clean idle). RENDERCTX published, seed + heartbeat patch
intact.

**Honest status:** the type-4 self-driven-frame deliverable REMAINS SHIPPED and
PROVEN at flat palette color (SH60/130/131). The upgrade to real-engine geometry
in task frames is NOT reachable via nested-run_guest_callback GLSL on the
presenter thread without a deeper design (host-side GLES object owner on a
separate thread, or reusing the renderframe-thread's already-current program) —
both of which put us on the far side of the structural wall's value curve.

**Do NOT reopen** unless a subagent derives a concrete safe design (e.g. a
dedicated GLES host-thread that owns the Mesa context and is handed draw lists by
the presenter, avoiding nested guest-bridge GLSL). Doc this SH as the negative.