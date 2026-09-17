# Frontier SH288 — drive the never-run WORKER's consumer loop (the missing half of the producer-only SESSION-CTOR path)

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Default-inert (opt-in `--v2boot-session-consumer`).

## tl;dr

SH283/284 measured only the PRODUCER half of the Route-B session pumping: the
initEngine_ state=5/9 reentry continuation enqueues + cond_signals the fixed queue
0x106863a70, but the spawned worker (pthread_create @ 0x2d97d70, start 0x1022076f8)
that should CONSUME the queue never runs headlessly — JIT_ROUTEB_ENG5_QMUTEX_FREE
only steals its mutex, never runs it. This cycle disassembles that worker, finds it
is a **real queue-consumer loop**, drives it for the first time, and **measures its
item-processor 0x102207950 executing headlessly before closing at a KNOWN wall.**

## The disassembly (guest = file + 0x100000000)

- 0x1022076f8 = generic thread-entry wrapper (`shr arg`, pthread_setspecific, then
  `blr [arg+8]` runs the real run-fn). The worker's actual body is the next fn.
- **0x10220778c = the worker CONSUME loop** (the real body the wrapper would run):
  - locks **session mutex 0x106863aa0** (0x2207820 bl 0x2b53a68) — the SAME mutex
    SH283 must steal, owned state=2 by this never-run worker;
  - `cond_wait` on **0x106863ac8** (0x220783c bl 0x2b52ec0) with predicate loop
    `ldr x8,[x25,#2712]` where x25=0x106863a70 — i.e. waits for the queue at
    **0x106863a70** (SH283's enqueue target) to have work;
  - when nonzero, pops an item x22 (0x2207858-0x220786c) and calls
    **0x102207950** (0x2207870) — the per-item PROCESSOR;
  - processor 0x102207950: acquire-loads once-guard [0x106a63b08], __call_once
    0x284ce54, builds an object into **0x106a63b00** (0x2207a70 `str x0,[x8,#0xb00]`),
    calls the GlobalInit sub **0x221942c** (0x22079e8).

So the enqueued item is consumed by a processor whose job is to drive session
state — the missing link between the settings-state producer and app construction.

## New opt-in rung (default-inert)

`--v2boot-session-consumer` (elfjit.rs, inside the engine3 block after the engine9
drive): reads work-flag [0x106863b08], then jit_run's guest 0x10220778c as its own
rung on the single ladder thread (boot_sp/tpidr, steal env set so its 0x2b53a68
lock acquires the contended session mutex). Region-watch on the item-processor.

## A/B measurement (real libroblox.so, runs/capture_sh288_consumer_drive.sh)

- **A (consumer OFF, SH285-B baseline):** 1/1 EXIT 134 parks at guestpc=0x101db1b08
  (LSM reader/pop live-object wall) — baseline parity.
- **B1 (consumer ON):** run-variance — crashed before the engine9/consumer rung
  (state9ok=0, cons=0). Known SH55/64-class ladder flake.
- **B2 (consumer ON):** state=9 body returned Ok(0x0)+state=10; the consumer
  rung FIRED and the region-watch proves the **engine's own queue-consumer path
  executed headlessly for the FIRST time**:
  - region hit 0x102207874 / 0x10220787c (popped queue item, x22=0x5622f196d7f0),
  - region hit **0x102207950 = the item-PROCESSOR executed**,
  - SH123 setfix + json-fix fired inside its body (real processing happened),
  - then SIGSEGV fault=0x50 at **guestpc=0x1021f3748** (lr=0x102270064) — the
    SH273/264-classified Activity-lifecycle-notifier live-object wall, reached
    now via the engine's own queue consumer (caller 0x102270064) instead of a
    native stub. x20=0x10683d210, registry object is the never-constructed
    lifecycle-callback-registry live object (SH174/204 class).

## Honest (do-not-over-claim)

- Converts the producer-only SESSION-CTOR pump (SH283/284) into a full
  producer→consumer measurement: the engine's OWN worker loop now runs and its
  item-processor executes — one full cause-not-symptom gate deeper than any prior
  cycle, on the SEP-17 SESSION-CTOR / manufacture line.
- CLOSES at the SAME SH273 live-object wall (0x1021f3748 fault=0x50) — consistent
  with SH273's "all lifecycle paths converge on one shared notifier" finding, now
  reached from a genuinely new engine-owned caller. Per SH248h/SH256 discipline
  the lifecycle-callback-registry live object is NOT manufactured/seeded; that
  class of repair is a recorded false-advance trap.
- Does NOT manufacture a DataModel; MH_* unchanged; Route-B live-DM structural
  gate UNCHANGED; SH174 capture-latch stays the single forward hook.
- Work-flag probe [0x106863b08] read 0x0 pre-drive yet the consumer still
  proceeded past its guard — exact cell attribution under investigation; the
  pop + item-processor region hits are the authoritative evidence the loop ran.

## Verify

- `cargo build --workspace` + `cargo test --workspace` EXIT 0.
- recon-v3 SELF-DRIVED FRAMES re-verified green at this HEAD (capture_taskv4_frame.sh:
  24 task frames swap Ok(0x1), 0 json abort, 0 crash, EXIT 124) — default-inert change.
- elfjit.rs kept under the 1MB pre-commit hook (condensed SH225/239/267 comment
  blocks; re-verified byte count).

## Files

- `crates/arm64jit/examples/elfjit.rs` (+ `--v2boot-session-consumer` rung).
- `runs/capture_sh288_consumer_drive.sh` (new A/B repro).
- Run captures runs/sh288-a1.txt, sh288-b1.txt, sh288-b2.txt (gitignored).