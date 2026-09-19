# Frontier SH391 — deterministic fix for the SH345/SH390 render-plane flake (guard the no-op self-copy)

Date: Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green
at start and end (cargo test --workspace EXIT 0; arm64jit lib 444->445 with the new
sh391 hermetic).

## Why this cycle

SH390 byte-anchored the SH345 ~1/25 render-plane SIGSEGV at its exact fault leaf
(guest 0x102859fd0, file 0x2859fd0) — a 16-byte memcpy guard leaf whose `str q0,[x0]`
@0x2859fe4 stores into guest .text from the drain/presenter divergence arm. Until now
the capture script retry-hardened around it (a coin-flip corrected by luck, up to 6
attempts), NOT root-caused. SH391 delivers the deterministic fix the SH390 frontier
doc explicitly prescribed as option *b*: zero x1 so the leaf's `cbz x1` short-circuits
the faulting store.

## The insight that makes it provably safe (not a retry-hide)

The pinned leaf is:
```
0x2859fd0: cmp x0,x1; b.ne out      ; if x0 != x1, exit (real copy NOT our case)
0x2859fd8: cbz x1, out               ; x1==0 -> exit
0x2859fdc: cbz x0, out               ; x0==0 -> exit
0x2859fe0: ldr q0,[x1]
0x2859fe4: str q0,[x0]   <-- crash   ; ONLY reached when x0==x1 and both non-zero
0x2859fe8: ret
```
The store executes **only when x0==x1** — i.e. the leaf is always a *self-copy* of 16
bytes of an address onto itself, a semantic no-op. It only faults when that (self, same)
destination page is non-writable (PROT_EXEC guest .text in the drain quirk). Therefore
skipping the copy when the self-dest is non-writable loses *nothing* — there is no real
data move to lose. It cannot mask a legitimate walker copy, because a real copy has
x0!=x1 and is already exited by the leading `b.ne`.

## The guard

`routeb_render_memcpy16_guard(state, pc)` (arm64jit/src/jit.rs), opt-in via
`JIT_ROUTEB_RENDER_MEMCPY16_GUARD`, wired into the per-block-entry guard dispatch.
Fires at the leaf block-entry pc 0x102859fd0; when `x0==x1 && x0!=0 &&
!page_is_writable(x0)` it zeroes x1 so the JIT block's `cbz x1` takes the `out` edge
before the store. A writable self-copy and every real (x0!=x1) copy are left untouched.
Always-on engine behavior is unchanged when the env is unset (default-inert).

No production path / guest byte permanently modified; no existing test weakened.

## Verification

- New regresser `sh391_render_memcpy16_guard_skips_only_nonwritable_selfcopy`
  (arm64jit lib 445): builds a deterministic PROT_NONE page (non-writable), asserts the
  guard zeroes x1 ONLY at the leaf entry with a non-writable self-dest, and leaves both
  real x0!=x1 copies and writable self-copies alone, and is inert (a) on non-entry pc
  with env on and (b) at leaf entry with env off. 1/1 pass.
- Real-image pin sh390 still green (guard added, leaf bytes unchanged).
- LIVE real-binary run (real libroblox.so, completing taskv4-frame ladder) with
  `JIT_ROUTEB_RENDER_MEMCPY16_GUARD=1`: **confirmed-green attempt 1 — 24 real
  task-driven frames `present swap Ok(0x1)`, 197 node pops, 0 SIGSEGV/ABRT, EXIT 124**.
  Guard firings = 0 on the clean run (correct: no pathological non-writable self-copy
  occurred, so the guard stayed inert and the normal deliverable is undisturbed).
- Workspace green: `cargo test --workspace` EXIT 0, 0 failures; `cargo test -p arm64jit
  --lib` 445/0.

## Honest + next

Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false,
AppBridgeV2 0); SH391 is render-plane determinism, not a DM advance. SH174 capture-latch
stays the single forward observer. The guard is armed by env so a future cycle can run
the capture loop with it ON and observe the `routeb-render-memcpy16` fire line the ~1/25
time the divergence arm hits — turning the previously retry-hidden coin-flip into an
instrumented, deterministic fix at the HARD-GATE artifact. Do-not-re-tread unchanged
(LSM skips, setDataModelToCurrent SH388, EC reader-gate, 0x258b5d8/SetInitParams,
window-attach real, ALooper, governor gates full-ladder, -9 string, map-header repair,
once-lambda store, SH267 node-cell).

## Files

- crates/arm64jit/src/jit.rs: +routeb_render_memcpy16_guard +sh391 hermetic (lib 445).
- Runs verified: `/home/hermes-worker/runs/sh60-taskv4-frame.txt` (out-of-repo capture).