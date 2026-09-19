# Frontier SH386 — byte-anchor the recon-v3 §A END-STATE engine-producer self-drive contract (the SESSION PRODUCER HANDOFF's node-push/epoch/futex mechanism, previously pinned nowhere)

Date: Sep 19, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green
at start (cargo test --workspace EXIT 0, 620/0; arm64jit lib 440/0) and at end
(621/0 lib after sh386, full workspace gate re-run green).

## Why this cycle
The operator's recon-v3 §A END-STATE (SESSION PRODUCER HANDOFF) specifies the
session-gated type4 producer (`--taskv4-seed session`) must, the instant a live
session advances, "push a real node via engine producer 0x10285682c + epoch bump
([Q]+=0x1_0000_0000 high-32 only) + FUTEX_WAKE_PRIVATE(0x81) on Q'+4;
[node+112]=0x106829f00 -> re-enters vector -> self-drive." The host path that
publishes into the engine's own lock-free task deque (producer 0x10285682c /
drain 0x102856e40, parked in the epoch futex) is implemented in elfjit.rs
(`--deque-node` producer) but was byte-anchored NOWHERE — the code is
comment-anchored only. This cycle pins that exact contract on the real binary so
the end-state self-drive wiring is reproducible and grounded.

## The pins (real-image hermetic sh386, file-offset == guest-0x100000000)
- Producer 0x285682c (guest 0x10285682c): a real 0x60-frame prologue
  `stp x29,x30,[sp,#-0x60]!` (0xa9ba7bfd) + `str x27,[sp,#0x10]` (0xf9000bfb) —
  the node-push self-drive entry the handoff drills.
- Drain 0x2856e40 (guest 0x102856e40): the consumer pop-loop that takes the w4==4
  dispatch -> br [0x106829ea8]; same prologue family (0xa9ba7bfd).
- Drain pop: `ldr x23,[x20]` @0x2856f94 (0xf9400297) reads the head node cell
  [headcell+0]; `ldar x24,[x23]` @0x2856f98 (0xc8dffef8) is the acquire head — exactly
  the [headcell+0] publish + [headcell+8] tag the `--deque-node` producer writes.
- Drain tag-guard 0x2856e6c/0x2856e78 (`ldr x26,[x1,#104]` 0xf940343a; `b.ne` 0x54001041)
  — the node's high-16 tag == headcell+8 word gate.

## What it means
The recon-v3 §A self-drive path is now byte-anchored on the real binary: the
engine's own producer/drain/pop/tag-guard contract the session-gated handoff will
fire through is pinned and reproducible. This is latent-but-correct wiring (the
producer, like the session-gated thunk, only self-drives once MH_APP_READY && a live
DM — which stays false on bare-boot headless) but the MECHANISM is now grounded, so
the "instant a real session advances" claim rests on verified bytes, not comments.

## Honest
Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED (DM-root
[0x106a68818]=0, MH_* false, AppBridgeV2 0). This is pin/verify hardening of the
recon-v3 §A end-state deliverable, not a new session drive. recon-v3 immediate-priority
deliverables re-verified green at HEAD this cycle: (1) capture_taskv4_frame.sh 24 real
task-driven frames `present swap Ok(0x1)`, 197 node pops, 0 json abort, 0 crash, EXIT 0;
(2) JIT_JSON_ZERO_FIX len-clamp present; (3) capture_sh304_session_producer.sh confirms
the session-gated producer is correctly INERT on bare boot (live_dm=true, app_ready=false
-> UNGATED x3, GATED=0, the lone present #0 is the RENDERINIT warmup self-test, 0 crash) —
the exact SH382-documented inverse control. SH174 capture-latch stays the single forward
observer.

## Files
- crates/arm64jit/src/jit.rs: +hermetic `sh386_session_producer_engine_push_contract_pinned`
  (arm64jit lib 440->441). jit.rs 1,014,568 B <1MiB hook.
- Repro logs: /home/hermes-worker/runs/recon-v3-reverify.log, /home/hermes-worker/runs/sh304-gate-reverify.log
  (outside repo).