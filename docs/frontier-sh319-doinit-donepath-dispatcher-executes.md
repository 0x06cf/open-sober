# Frontier SH319 — do-init DONE-path dispatcher now EXECUTES (SESSION-CTOR advance)

Status: `dev`, single-agent (cone suppressed). Hermetic: `sh319` (real-image, 5 pins).
Workspace green (arm64jit lib 405/0; elfjit examples 144/0 = 143 + sh319). elfjit.rs held 12 B under
the 1MB pre-commit hook. Recon-v3 re-verified green at this HEAD (24 frames swap Ok, 197 pops,
0 json abort). Route-B live-DM gate UNCHANGED (no DM; DM-root [0x106a68818]=0, MH_* false).

## 1. The finding (measured, UNPERTURBED, real libroblox.so)

On the plain SESSION-CTOR bus run (`--v2boot-skip-appstart --v2boot-session-bus`, full seed set,
single ladder thread) do-init's once now LATCHES (once-guard [0x106a68410]=0x1, once-slot
[0x106a68408]=0x400000b = the DM-ctor fast-path's matched "Execute" service handle, app-data-model
counter [0x106dca000+0xe88]=1 — all SH315/316 state). The NEW reach: the do-init **DONE-path
dispatcher 0x2206db8 now runs**, reaching its non-main-thread box-build branch:

    JIT_DUMP_PC=0x102206e34  ->  DUMPPC pc=0x102206e34  (1 hit, EXIT 124, no crash)

That pc is `mov w0,#0x40` inside 0x2206e28 (the dispatcher's `b.ne` non-main box-build branch:
`bl pthread_self` @0x2206de8, `cmp x0,x20` [main-id 0x106863a68], `b.ne 0x2206e28` @0x2206df0 ->
`mov w0,#0x40; bl operator_new 0x1d96768` -> build 0x40 box -> bl 0x2207118 enqueue-construct).
SH311 measured the done-path as 0-hit ("never entered"); SH115 region-watch on the narrow done-path
band also reported 0 — because 0x2206c88/0x2206cc0 are MID-BLOCK entries (SH301 lesson) reached by
fall-through, not block entry. Only the dispatcher's branch targets (0x2206e28/0x2206e34) and the
once-lambda tail (`b 0x2206c88` @0x2206d88) open block entries. The wide-band run (0x102206c40-0x
102207000) also fired 0x2206cc0/0x2206cc4 + the dispatcher 0x2206db8, but that run SIGSEGV'd as a
region-watch perturbation (SH248b class); the JIT_DUMP_PC single-pc probe is UNPERTURBED and clean.

## 2. What this means for Route B

- Cause-level, SESSION-CTOR: do-init completes its once AND the done-path reads the result. That's
  one gate further than SH316 (which measured the once-latch and stored once-slot but did not
  establish the done-path dispatcher execution). The done-path's `ldr x1,[x8,#1032]@0x2206c8c`
  (reads once-slot) -> builds stack frame -> bl 0x6201bd4/0x220671c/0x2206db8 (dispatcher) ->
  non-main box-build -> enqueue-construct 0x2207118. This is the app-shell/DM world-build
  continuation (SH225-constrained: the main-thread binder-dispatch 0x206df4..0x206e24 still does
  NOT run because main-id != ladder thread).

- HONEST: does NOT manufacture a DM. DM-root stays 0, MH_* false. The once-slot value 0x400000b is
  the task-scheduler "Execute" service handle, not a live DM controller, so the done-path's
  dispatch of it cannot yet yield a session node. Route-B live-DM structural gate UNCHANGED.

## 3. The lever sharpens it

- The dispatcher's MAIN-thread branch (`b.ne` at 0x2206df0 NOT taken) is what reaches the
  binder-dispatch 0x206df4..0x206e24 (`ldr x0,[x19,#32]`; vt+0x30; `br x1` = DM ctor entry). To
  take it, main-id [0x106863a68] must equal pthread_self on the ladder thread — currently it does
  not (ladder is a spawned worker). This is the same SH225 gate; the box-build branch is the reached
  (non-main) half, the binder-dispatch half stays latent.
- Combining with SH316: once the once-path latches, driving the dispatcher to the MAIN branch (e.g.
  driving the ladder on the true main thread, or seeding main-id to match) would engage the DM-ctor
  binder-dispatch — the next forward candidates to test.

## 4. Verify

- `cargo test -p arm64jit --example elfjit -- sh319` (1 passed; 144/0 total).
- Repro capture: `runs/probe_sh319_donepath.sh` (JIT_DUMP_PC=0x102206e34 -> DUMPPC hit, EXIT 124).
- Full elfjit example suite 144/0; `cargo test --workspace` EXIT 0; `cargo build` EXIT 0.
- Recon-v3 re-verified green (`runs/capture_taskv4_frame.sh`: 24 frames, swap Ok(0x1), 197 pops,
  0 json abort, EXIT 124).

## 5. Addendum (measured same cycle): the done-path reaches the SESSION item-queue ENQUEUE

JIT_DUMP_PC=0x102207118 also fires (clean EXIT 124, once-slot=0x400000b): the done-path dispatcher's
box-build (`bl 0x2207118` @0x2206e68) CONTINUES into enqueue-construct 0x2207118 (`sub sp,#0x30`;
mutex_lock [0x106863aa0] = adrp 6863000/#0xaa0; `bl 0x2d9713c` enqueue x0=[0x106863a70]; mutex_unlock;
clean ret 0x220718c). So the once-latched do-init done-path now enqueues an app-shell construct onto
the SAME fixed session queue [0x106863a70] that SH288's consumer half and SH290-295's item-proc
consume. SH281-283 measured 0x2207118 on a DIFFERENT entry (initEngine_ state=5/9 reentry) with a
diagnosed park at the contended mutex (state=2's never-run worker on [0x106863aa0]); this SH319
finding is the do-init done-path, a NEW caller reaching the identical enqueue source. Still no DM;
the enqueue feeds a consumer that (SH288) parked at the SH273 lifecycle-notifier wall. Route-B live-DM
gate UNCHANGED. This confirms the done-path executes to a real session side-effect (enqueue), not
just the dispatcher prologue.

Single-agent. Workspace green. elfjit.rs held 12 B under the 1MB pre-commit hook.