# Frontier SH341 — measure: the LSM free-list `str x8,[x1]` write-off is a SINGLE poisoned pointer from the FMOD region (0x626b6d0 wrapper), not an intrinsic unwritable-write

## Session
Sep 19, 2026, hermes-worker. Single-agent (cone suppressed). One new default-inert
instrumentation guard (JIT_ROUTEB_LSM_KEYTRACE=1, READ-ONLY) + one hermetic real-image
pin. Workspace green (arm64jit lib 410/0, elfjit example 156/0). recon-v3 re-verified
green earlier this cycle (24 self-driven frames, swap Ok(0x1), 0 json abort, EXIT 124).

## Question
SH268 pinned the persistence-lane terminal guestpc=0x101d9a528 (LSM free-list pop)
whose trailing `str x8,[x1]` faults at 0x101d968e4 — inside the R-E exec LOAD segment
[file 0x0, 0x62d8190) (write:off) — and classified it "SH249/SH258 proven-unwritable
live-object." But that verdict describes the *write target*, not the *source* of x1. An
LSM map KEY is never legitimately a code address; the key here is the pop's own head-cell
(the pop does `mov x1,x0` @0x1d9a530 then `str x8,[x1]`). So the wall is a SYMPTOM: some
caller passes a poisoned pointer as the key. SH268 never attributed WHICH caller.

## The guard (default-inert, READ-ONLY)
`routeb_lsm_keytrace_guard` (jit.rs, JIT_ROUTEB_LSM_KEYTRACE=1) fires at the pool-pop fn
entry block 0x101d9a5a0 (a real `bl` target, 9 in-image callers) logging x0=key +
x30=LR (call site), and at the pop write-site block 0x101d9a528 logging the key that
`str x8,[x1]` will write. Classifies each key: EXEC(.text,write:off) vs guest-data vs
sub-image/host-leak vs foreign.

## MEASURED (real libroblox.so, SH267 ON arm + KEYTRACE, EXIT 134 at 0x101d9a528)
The guard captured 392 pop executions. Decisive result:

- **EXACTLY ONE poisoned-key event** — key=x0=0x101d968e4, caller=LR=0x10626b6dc,
  classed EXEC-SEGMENT(.text,write:off) POISONED — and it is the crash (fault=0x101d968e4,
  guestpc=0x101d9a528). Confirmed live, deterministic on the crashing iteration.
- **All other 390 pops use valid HOST-heap keys** (0x557e…, 0x7f81…) and complete — the
  write lands in real memory; no fault. So the engine is NOT trying to write code memory
  on the persistence lane: a single stale pointer is fed as the key exactly once.

### Caller attribution
- The poisoned key is passed to pool-pop 0x1d9a5a0 with LR=0x10626b6dc (file 0x626b6dc).
- Disassembly: file 0x626b6d0 is a thin trampoline — `stp x29,x30,[sp,#-16]!; mov x29,sp;
  bl 0x1d9a5a0` (@0x626b6d8); `ldp x29,x30,[sp]; ret` (@0x626b6dc). It forwards x0
  unchanged into the pool-pop. **This is the "FMOD AAudio 626b6d0" site cited as the SH212
  wall** — it is actually the LSM pool-pop wrapper, not FMOD audio output.
- So the poisoning originates upstream of 0x626b6d0's caller (the run-variable FMOD/audio-
  init path, SH212 family: 0x106240cb8 / 0x626b600 region), which passes a stale .text
  pointer as a bucket/pool key.

## What this REFINES (do-not-over-claim)
- SH268's "proven-unwritable R-E live-object, no lever can reach it" is correct that the
  0x101d968e4 write can never succeed — but the MECHANISM is now shown to be a single
  poisoned-pointer passthrough from the FMOD/0x626b6d0 region, not an agent of the LSM
  persistence write itself. The fix target is the FMOD-side caller that leaks the stale
  key, NOT the LSM pop (which is well-behaved for all valid keys).
- This is a measurement/attribution, NOT a crossing: no session advance, DM-root 0,
  MH_* false, Route-B live-DM gate UNCHANGED. No production path edited (guard inert).
- It narrows SH212's "FMOD AAudio 626b6d0" wall to the LSM pool-pop wrapper, correcting a
  long-held label in my own run-notes.

## Code / verify / artefacts
- jit.rs: `routeb_lsm_keytrace_guard` (default-inert, JIT_ROUTEB_LSM_KEYTRACE=1, READ-ONLY)
  wired into the block-entry dispatch.
- elfjit.rs: `sh341_lsm_pop_keytrace_caller_attribute_contract` (skip-if-absent) pins the
  pool-pop fn entry (sub sp,#0x70), `mov x1,x0` (key->write target), `str x8,[x1]` fault
  site, the 9 in-image callers (guest low-48), and the EXEC-vs-data classifier
  (0x101d968e4 EXEC; 0x106a705e8 / 0x10726f8c0 NOT).
- Repro: `runs/capture_sh341_lsm_keytrace.sh` (SH267 ON arm + JIT_ROUTEB_LSM_KEYTRACE=1).
  Live capture /home/hermes-worker/runs/sh341-lsm-keytrace.txt (gitignored per CLAUDE.md).
- Verify: `cargo build --workspace` green; `cargo test -p arm64jit --example elfjit sh341`
  = 1 passed; `cargo test -p arm64jit --lib` = 410/0; workspace green.
- Commit: local `dev` only (operator pushes).