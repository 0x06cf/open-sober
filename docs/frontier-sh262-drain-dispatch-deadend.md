# Frontier SH262 — app-start drain dispatch at [0x106a70c90] is a measured dead-end (reverted)

## Session
Sep 17, 2026, hermes-worker. Single-agent (cone suppressed).
Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays the single
forward hook. No production code path edited (guard implemented + A/B'd + REVERTED);
workspace green.

## Why (the closes t untried lever on the newest surface)

SH261 measured a genuinely new surface: the deep app-start self-drive message-loop
drain (region [0x10233a000,0x102350000)) fires 100+ distinct pcs including
0x233bc88..0x233bcf0, and terminates run-variably (LSM insert-leaf SIGSEGV /
`std::bad_function_call` / engine-init SIGSEGV). SH261 classified the
`bad_function_call` as "an EMPTY std::function in a LIVE host-heap app-start object,
NOT a fixed .bss global -- no constant seed lever." This cycle re-examined the ONE
fixed-.bss component of that drain SH261 flagged as anode: the dispatch at

    0x233bcb8 adrp x8, 6a70000
    0x233bcbc ldr  x0,[x8,#0xc90]   ; x0 = [0x106a70c90]
    0x233bcc0 cbz  x0, 0x233bcf8     ; if NULL, SKIP the dispatch (benign)
    0x233bcc4 ldr  x8,[x0]
    0x233bcc8 ldr  x8,[x8,#48]       ; vt+48 / slot 6
    0x233bccc blr  x8

`[0x106a70c90]` (file 0x6a70c90) is RW .bss on a heavily-referenced page
(0x106a70000). It is a constant address, so SH261's "no constant address to
hand-seed" verdict looked re-testable on exactly this cell -- the SH248f adapter
pattern (fixed .bss global holding a vt-dispatched live object).

## Measured result (A/B on real libroblox.so, full SH259 seed set)

Implemented a default-inert guard (JIT_ROUTEB_APPSART_DRAIN_SEED) at block-entry
0x10233bcb8 that, when `[0x106a70c90] != 0`, replaces it with the SH248f all-leaf
adapter (so vt+48 resolves a benign leaf). Batch-tested (5-run baseline vs 5-run
arm, then +9 arm, then diagnostic runs):

- The seed NEVER fired in any drain-firing run: `[0x106a70c90]` is **0 every time**
  the drain block executes, so the `cbz x0, 0x233bcf8` skip path is taken and the
  vt+48 dispatch never runs. The drain-firing runs still ended in
  `std::bad_function_call` with the seed inert.
- Decisive control: a run (r7) threw `std::bad_function_call` with **zero** drain
  region hits (JIT_REGION_WATCH [0x10233bc80,0x10233bd00) = 0). So the
  empty-function terminal is NOT the [0x106a70c90] dispatch at all -- it is a
  SEPARATE live host-heap object (invoked via the drain's `bl 0x21daef8` registry
  builder / a different spin path), with no constant seed lever. SH261's original
  classification is CONFIRMED.
- A per-block-entry eprintln diagnostic (temporarily added, then removed) is the
  SH248b contamination class: it perturbed the run enough that the drain stopped
  firing (0/8 under the diagnostic vs ~2/5 clean), so the clean picture required
  removing it.

## Decision (do-not-re-tread)

`[0x106a70c90]` is the **9th adjacent closure** at the app-start live-object wall
(static-seed SH248g / runtime-repair SH248h / segment-protected SH249 / count-clamp
SH250 / dynamic-ctor-trace SH251 / resolver-source SH253 / settings-once SH259 /
LSM SH260). The drain dispatch terminal is a live host-heap object with no fixed-.bss
lever, identical to the standing Route-B structural gate. GUARD REVERTED (a lever
provably unable to fire on the failing case adds no value -- same standard as
SH248h/SH256). Factory clean vs HEAD (only the A/B repro script
runs/capture_sh262_drain_ab.sh remains, new/untracked, as the documented artifact).

## Honest (do-not-over-claim)

Does NOT manufacture a DataModel; does NOT lift the Route-B live-DM structural gate.
Confirms SH261's classification with a targeted A/B. Route-B live-DM structural gate
UNCHANGED; SH174 capture-latch (arm *(0x106391908) at a real make_shared<DataModel>)
stays the single forward hook. recon-v3 deliverables (type4 self-driven frames + json
zero-fix) re-verified green at HEAD (24 task-driven frames, swap Ok(0x1), 196 node
pops, no json abort, 0 crash, EXIT 124).

## Artefacts
- repro runs/capture_sh262_drain_ab.sh (A/B script; default no-op, inert).
- Verify: workspace green (cargo build + cargo test --workspace exit 0).