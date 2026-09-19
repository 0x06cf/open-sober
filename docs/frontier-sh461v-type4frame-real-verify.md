# SH461-VERIFY — recon-v3 §A type4_frame_thunk REAL-BINARY VERIFICATION (the self-driven-frame artifact)

Verified green on the real libroblox.so this cycle (single-agent, cone suppressed)
via the existing runbook `runs/capture_taskv4_frame.sh` — the EXACT command +
verify markers of `docs/recon-selfdrive-seed-jsonfix.md` §A, after the SH455-461
test-only codegen-pin lineage kept the runtime byte-identical to the SH445
baseline.

## Measured (real run, real 104MB ARM64 client, Mesa llvmpipe)

- command: `elfjit .../libroblox.so 0x2173ff4 --jni --startapp 0x258b144
  --renderinit 0x105b3a280 --renderthunk --renderframe --taskv4-seed frame
  --deque-node-live 0x106829f00 --drain-poll 8 --persist-roundtrip
  --kicker 0x106863af8` with `JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=1000
  JIT_ROUTEB_RENDER_MEMCPY16_GUARD=1` (the guard makes the artifact
  deterministic-by-construction), `timeout 50`.
- **24 real task-driven frames** `[elfjit:taskv4-frame] present #N swap
  Ok(0x1) color=[...]` — each a REAL engine frame (make-current 0x105b3b358 ->
  frame-fn 0x105b32c00 -> swap 0x105b3b408 on the recovered RENDERCTX vtable
  0x106731ae0), not a harness-composed object.
- `dispatch #2165000` w4=4 dispatches reaching the thunk; **196 real node pops**
  (task-driven drain activity through the real drain pop-loop).
- seed markers: `[elfjit:taskv4] type4_frame_thunk registered at 0x7f00000001d0`,
  `seeded dispatcher type-4 vector [0x106829ea8] = 0x7f00000001d0`, the two
  heartbeat `mov w4,#2/#3 -> #4` patches (0x102856f24/0x102856f68), `renderthunk
  published RENDERCTX 0x7f9530111b20`.
- **0 json abort** (`RBX::json` / `string length overflow` absent) — the 
  JIT_JSON_ZERO_FIX path stays clean; **0 crash-signals**; EXIT 124 (stable idle
  loop), confirmed-green on attempt 1/6.

## Concurrent evidence on the same run

- `[persist] live datastore roundtrip: write=45B fsync=0 read_back_byte_exact=true
  on_disk=Some(true)` — the durable-persistence (objective 2b "remembers sign-in")
  contract for `session.db` fires ON the real session, backing SH423's hermetic.

## Honest

NOT a live DM (Route-B live-DM structural gate UNCHANGED — SH415 do-init probe
DM-root [0x106a68818]=0x0). This is the recon-v3 §A self-driven-FRAME deliverable
verified as a REAL capture artifact (the frame present path proven on real
hardware/llvmpipe), distinct from the DM-construction wall. No re-treads.

Files: this doc + the raw capture log at `/home/hermes-worker/runs/sh60-taskv4-frame.txt`
(340K, kept on disk, not committed — under the repo 1MiB hook it would be fine but
the CLAUDE.md discipline prefers small committed repros).