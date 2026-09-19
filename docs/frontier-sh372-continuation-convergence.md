# Frontier SH372 — DM-continuation and settings-state init paths CONVERGE on the identical SH285 unconstructed-manager leaf (answers SH371's explicit "same object or different?" with a fresh register dump)

Session: Sep 19/20, 2026, hermes-worker. Single-agent (cone suppressed). One new
probe runs/capture_sh372_continuation_terminal.sh (real-image, JIT_DUMP_PC +
JIT_GUEST_STACK_DUMP + JIT_REGION_WATCH on the continuation body). No production
path edited. recon-v3 deliverables re-verified green at HEAD this cycle
(capture_taskv4_frame.sh attempt 1: 24 real task-driven frames `present swap
Ok(0x1)`, 0 json abort, 0 crash). Workspace green (cargo test --workspace EXIT 0).

## The genuinely-new datum this cycle

SH371 measured that the DM-creator continuation continueAfterFlagsLoaded_
(0x102bd1d68) now RUNS DEEP headlessly under the full Route-B env (25+ block-entry
pcs) and then dies at the standing SH285 persistence-lane wall
(guestpc=0x101db1b08). SH371 explicitly left open whether the continuation hits the
**same** [obj+0x50]=0xff..ff object the settings-state path (SH285 doc) faulted on,
or a **different** unconstructed field. SH372 closes that gap with a fresh full
register + guest-stack dump from the continuation path:

```
[SIGSEGV] guestpc=0x101db1b08 rbx_matches_gueststate=true (reg_state=... tid=0)
  x0=x19=0x7f9d70fb7f80  x20=x1=0xffff80628f048080  x10=0xffffffffffffffff
  x21=sp  lr(x30)=0x101db1b18  lsm_map_global=0x7f9d7ba71010
  GSDSP[+0=0xc0 +8=0 +16=0 +24=0 +32=0 +40=0x200 +48=0x1 ...]
BT: -> HOST(...)
```

- Continuation FIRED (1 block-entry at 0x102bd1d68) and its deep pcs all executed.
- Terminal pc 0x101db1b08 with lr=0x101db1b18 — the SAME leaf (`bl` returns to
  0x1db1b18 = the sh285 reader caller inside initStorageManagerNative 0x101d9d8b0).
- Fault target x20=x1=0xffff8062... = the SAME 0xff..ff-prefixed uninitialized
  internal data-pointer (the SH285/[obj+0x50] family). x0=x19=0x7f9d... = a
  guest-constructed host-heap object, exactly as SH285 classified.

## Interpretation (map refinement, not a new wall)

**Both independently-reached engine init paths converge on the identical
unconstructed-manager leaf.** The settings-state self-drive (SH284/285) and the
DM-creator continuation (SH371, SH372) fault at the same pc 0x101db1b08, lr
0x101db1b18, on the same 0xff..ff buffer pointer. This proves the SH285 terminal is
**path-independent** — it is NOT a path-specific divergence (a benign-body branch
that one path takes and the other misses); it is the manager object's own
unconstructed string buffer, which no seed manufactures (SH248h/SH256 rule) and
which only a REAL LocalStorageManager/session ctor owns (SH174/204 live-object
class). It also corroborates SH371's STRAIGHT-LINE finding: the dispatcher has no
benign branch, so both paths are forced into the same leaf.

Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false).
SH174 capture-latch stays the single forward hook. This cycle is a measured
refinement that strengthens the "measured-closed SH285 lane" record with a
second-entry confirmation, not a session advance.

## Files
- runs/capture_sh372_continuation_terminal.sh (probe, new).
- Live dumps runs/sh372-continuation-terminal.txt (gitignored per CLAUDE.md).