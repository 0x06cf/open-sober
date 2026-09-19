# Frontier SH462 — DM-root store-watch: name the exact guest store that populates (or never reaches) the Route-B DM-holder cells

## Session
Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green
(cargo test --workspace exit 0). One new DEFAULT-INERT host observation extended
onto the existing post-store watch hook (`translate.rs` `canary_store_watch` +
`emit_canary_store_watch`), gated by `JIT_DMROOT_STORE_WATCH=1` (default off =>
product bytes byte-identical; jit.rs untouched, still 1,048,390 B < 1MiB hook).
One new hermetic `sh462_dmroot_store_watch_gated_and_window_classified`
(arm64jit lib 668 -> 669). elfjit.rs / session.rs unchanged.

## Why this cycle
The Route-B structural wall has been measured *only by reading* the DM-holder
cells: SH361 (do-init dyn trace), SH381 (once-lambda ctor return), SH334
(registry live), SH388 (DMSVC getter) all read [0x106a68818] / [0x106a68408],
and every probe reports `DM-root = 0x0` (or, on the legacy SH155 ladders, a
non-zero but non-image host pointer that never became a live DM). NO instrument
ever NAMES the exact guest store that would populate those cells. That is the
operator-requested "re-attack the wall via a dynamic DM-ctor trace rather than a
static seed" gap: we do not yet know whether (a) the once-lambda's DM-ctor
`br x1` dispatch writes the holder but the value is non-live (writer REACHED —
potentially seedable), or (b) no 64-bit store ever targets the holder window on
the headless path at all (wall genuinely structural — not seedable-by-writer).

## The instrument
`emit_canary_store_watch` already emits a post-store host call at every 64-bit
`str`/`stp` emission site (LdStrImm/Wb/LdStrReg/LdStPair), default-inert.
SH462 widens the EMISSION gate to `canary_store_watch_enabled() ||
dmroot_store_watch_enabled()` and, in the host `canary_store_watch`, FIRST
reports any 64-bit store whose destination lands in the DM-holder window
`DMROOT_WIN_LO=0x106a683f0 .. DMROOT_WIN_HI=0x106a68828` (spans once-guard
[0x106a68410], once-slot [0x106a68408], and DM-root [0x106a68818]) whenever
`JIT_DMROOT_STORE_WATCH=1`. Each distinct (pc,value) pair logs one line
`[dmroot-store-watch] pc=… dst=… val=…`; consecutive repeats are de-duped.
Observation-only (post-store, reads nothing, writes no guest bytes, returns 0).

## Interpretation
- **A fire with a non-null `val`** = a writer EXISTS on a reached path and stored
  a real object into the DM holder → the wall is NOT structural-at-writer; the
  next step is why that object is not a live DM (SH388 getter gate).
- **A fire with `val=0`** = a reached writer explicitly clears the holder → the
  once-lambda dispatch bails before constructing (the seedable branch).
- **Full-run silence** = no guest 64-bit store ever lands in the holder window
  headlessly → the DM construction writer is simply never reached by the JIT →
  confirms the wall is genuinely structural (nothing to seed on the write path).

## Honest
Does NOT manufacture a DataModel (this is an observer, not a seed). Answers the
operator's dynamic-trace question one level deeper at the STORE, not just the
read, site. The recon-v3 immediate deliverables remain green at HEAD (this
change is a default-inert `#[cfg]`-free observer on `translate.rs` only; runtime
byte-identical when `JIT_DMROOT_STORE_WATCH` unset). Route-B live-DM gate
UNCHANGED until a real session's do-init world-build owns a live DataModel.

## MEASURED on the real libroblox.so (SH415 env + JIT_DMROOT_STORE_WATCH=1)
EXIT 124 (stable idle), substrate 11/16, probe LIVE DM = false (DM-root
[0x106a68818]=0x0, vt=0x0, counter=0). **11 distinct `[dmroot-store-watch]`
fires**, all in the once-slot/guard region [0x106a68408..0x106a684d0]:

```
pc=0x102206d74 dst=0x106a68408 val=0x400000b   <- SH381 once-lambda store site
                                                  str x0,[x23,#1032] (after bl 0x2173b3c):
                                                  writes the once-state SENTINEL 0x400000b,
                                                  NOT a DM object
pc=0x102206654 dst=0x106a68460 val=0x55bee27d5470   real heap carry (ctor/frame)
pc=0x102206668 dst=0x106a68458 val=0x55bee27d5470
pc=0x102206680 dst=0x106a68468 val=0x1  (then 0x2)   once-state stepping
pc=0x102206a54 dst=0x106a684d0 val=0x7f6ea6234440    host/guest-managed carry
pc=0x102206808/0c/10 dst=0x106a68458/468/468 val=0x0 zeroing
```

**Interpretation (the operator's dynamic-trace answer at the store level):**
- The do-init once-lambda body [0x22065xx..0x2206axx] DOES execute headlessly and
  populates real structure into the once-slot/guard cells ([0x106a68408..0x106a684d0]);
  the writer is REACHED.
- The once-lambda's ctor store at 0x2206d74 writes the once guard-state value
  `0x400000b` into once-slot [0x106a68408] — NOT a DataModel pointer. The `bl
  0x2173b3c` (DM-ctor) return is folded into the once-state sentinel, i.e. the
  ctor path taken headlessly does not produce a live DM object to store.
- **NO guest store ever targets DM-root [0x106a68818] itself.** Every fire is in
  the 0x108-byte once-slot region BELOW it; the DM holder cell is never written.
  That is the dynamic confirmation that, on the headless path the JIT drives,
  no reached writer populates the DM-root holder — the gate is structural at the
  DM-root write site, not merely a read-side artifact. The earlier SH155-ladder
  `DM-root=0x7f…` nonzero values were harness SEED writes (host-side), not guest
  stores — which is why the guest store-watch does not list them.
- Net: the once-world-build runs, but DM-root is never written by guest code
  headlessly → the Route-B live-DM wall is confirmed structural at the write
  site (the real DM is built by an upstream session path the JIT cannot drive;
  building the session/runtime surface remains the operator-aligned lever).

## Files / verify
- crates/arm64jit/src/translate.rs: DMROOT_STORE_WATCH atomics +
  set_dmroot_store_watch_test + WIN_LO/WIN_HI + host-fn DM-window branch +
  emit-gate OR + `sh462_dmroot_store_watch_gated_and_window_classified` hermetic
  (default-inert byte check, dmroot-ON emits, window/outside-window/dedup no-panic,
  flag reset). arm64jit lib 668 -> 669.
- Capture probe: `JIT_DMROOT_STORE_WATCH=1` on the SH415 do-init-completion
  capture (same env + the store-watch flag) → expect the `[dmroot-store-watch]`
  lines naming any DM-holder writer (or confirming silence) alongside the usual
  `EXECUTE-DO-INIT live-DM probe`.
- Workspace green (cargo test --workspace EXIT 0, 0 failures).
- Commit: local `dev` only (operator pushes).