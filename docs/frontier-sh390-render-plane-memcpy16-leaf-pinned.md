# Frontier SH390 — byte-anchor the SH345 render-plane flake's exact fault leaf (memcpy16 into guest .text)

Date: Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green
at start (cargo test --workspace EXIT 0, 443 arm64jit lib tests) and at end (added
sh390 hermetic, 444 lib).

## Why this cycle

recon-v3 immediate-priority deliverables (self-driven frame + JSON-abort) are
committed and green at this exact HEAD (re-verified live this cycle:
capture_taskv4_frame.sh attempt 1 = 24 real task frames `swap Ok(0x1)`, 197 node
pops, 0 json abort, 0 crash; capture_sh304_session_producer.sh = session-gated
producer correctly INERT 0 GATED / 0 fabricated frames). Every named Route-B cone
is measured-closed (SH385 closes even the SH384 manufactured-manager wiring-into-
the-lane target as an LSM-family re-tread to NOT re-drive).

That leaves ONE genuinely-open, in-scope correctness defect: the SH345 render-plane
flake — a ~1/25 serial SIGSEGV in the primary deliverable capture. SH345 recorded it
in prose only: "store into guest .text 0x102859fd0 from a non-guest thread,
rbx_matches_gueststate=false, host store mov [rax],rcx with rdx=0x3333333333333333."
It was never byte-anchored, and it is currently mitigated by RETRY-HARDENING the
capture script rather than root-caused. That makes the HARD-GATE "reproducible
artifact" (a real 24-frame capture on every invocation) a coin-flip corrected by
luck, not a deterministic deliverable. SH390 byte-anchors it.

## The fault leaf (real-image disasm, file offset = vaddr)

guest 0x102859fd0 (file 0x2859fd0) is a small memcpy16 guard leaf:
```
0x2859fd0: eb01001f   cmp x0,x1
0x2859fd4: 54000160   b.ne out (to 0x10285a000)
0x2859fd8: b40000a1   cbz x1, out
0x2859fdc: b4000140   cbz x0, out
0x2859fe0: 3dc00020   ldr q0,[x1]
0x2859fe4: 3d800000   str q0,[x0]   <-- THE present-walker crash store
0x2859fe8: d65f03c0   ret
```
The crash class (SH344b/344c "activity-lifecycle divergence arm" / drain quirk):
the drain dispatches a cloned task node while the presenter thread has run its
walker/nativeOnDestroyed-family path; the leaf's destination x0 computes (from an
uninitialized buffer member) to a guest .text address (PROT_EXEC), so the 16-byte
copy faults. This is the same run-variable live-object surface SH346/353 documented
on the full ladder, now pinned at the leaf instead of only the region.

## What landed

- New real-image hermetic `sh390_render_plane_memcpy16_fault_leaf_pinned`
  (arm64jit lib 443->444): pins all 7 words of the leaf + alignment. Index by FILE
  OFFSET (guest-0x100000000) per house style. Skip-if-absent on CI-less image.
- No production path / JIT hook / guest byte touched (pure pin; adds coverage, does
  not weaken any test).

## How this unblocks a real fix (not just retry-hide)

With the leaf pinned, the next targeted step is a narrow guard at block-entry of
0x102859fd0 in the drain-entry window: if x0 resolves to a PROT_EXEC / non-writable
guest .text address, either (a) RET the leaf before the store (byte-patch
0xd65f03c0 at entry, single-caller-scoped), or (b) zero x1 so the cbz x1 short-circuits
the copy — a real determinism fix for the artifact the HARD GATE depends on, rather
than the current capture-script 6-attempt retry. Deliberately NOT implemented this
cycle (a ret-to-leaf could mask a real walker copy; the guard needs a live
JIT_REGION_WATCH gate on when the divergence arm fires first, which is a follow-up
measurement).

## Honest + do-not-re-tread

Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false,
AppBridgeV2 0); this is render-plane determinism, not a DM advance. SH174 capture-
latch stays the single forward observer. Do-NOT-re-tread unchanged (LSM skips,
setDataModelToCurrent SH388, EC reader-gate, 0x258b5d8/SetInitParams, window-attach
real, ALooper, governor gates full-ladder, -9 string, map-header repair, once-lambda
store, SH267 node-cell). SH390 ADDS: the render-plane flake is byte-anchored — do NOT
treat it as an un-fixable run-variable coin-flip; guard the pinned leaf (see above).

## Files

- crates/arm64jit/src/jit.rs: +sh390 hermetic (arm64jit lib 443->444). jit.rs
  1,029,260 B (<1MiB cap).
- recon-v3 deliverables re-verified green live at this HEAD (capture_taskv4_frame.sh
  attempt 1: 24 frames swap Ok(0x1), 197 pops, 0 json, 0 crash; capture_sh304: 0 GATED).
- Reproducible hi-verification: `cargo test -p arm64jit --lib sh390` passes.