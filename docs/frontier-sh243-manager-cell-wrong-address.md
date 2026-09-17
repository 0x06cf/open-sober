# SH243 — the NativeDataModelManager getter cell was seeded at the WRONG address for 8 cycles

Date: Sep 17, 2026, hermes-worker. Workspace green (lib 391/0 incl. new sh243, examples 85/0,
recon-v3 plane re-verified 24 frames swap Ok(0x1) no json abort). Single-agent (cone suppressed).

## Question
SH165-240 all seeded the NativeDataModelManager singleton into guest **0x102727550** and
repeatedly concluded "the engine-init dispatcher 0x102bd8ce8 never resumes past the getter
0x102174c04" (a LATENT wall until a real flags-loaded session). Was 0x102727550 actually the
cell the getter reads?

## Measured: NO. The getter's TRUE read cell is 0x107275550 (0x5000000 apart)
- `routeb_dm_manager_guard` writes the fabricated manager M into `HOLDER = 0x102727550`.
- The getter `0x102174c04`: `adrp x8, 0x7275000; add x8, x8, #0x550; ldar x0, [x8]` reads
  **guest 0x107275550**, NOT 0x102727550. Verified by three independent authorities:
  1. Hand ADRP decode of `0xb0028808` at guest pc 0x102174c18: page(0x102174000)+0x5101000
     = 0x107275000, +0x550 = **0x107275550**.
  2. GNU objdump's own symbolic target: `adrp x8, 7275000 <__stop_pb_defaults...+0xa4b1a8>`
     (0x7275000 file vaddr = guest 0x107275000).
  3. The loader's own segment map: 0x107275550 is vaddr 0x7275550 in the RW data seg
     [0x67d67c0, 0x7333c3c); 0x102727550 is vaddr 0x2727550 inside the R-E CODE seg
     [0, 0x62d8190). Seeds to the CODE seg were inert for the getter.

## The A/B (real binary, 4/4 both legs)
SAME completing `--v2boot` ladder, DMFORCE+DMCONT+DM_SEED+HASHFIX+JSONZERO+SETFIX+SH115:
- BASELINE (only 0x102727550 seeded): `StartLuaAppDM returned Ok(0x3e8)` (benign soft-return),
  dispatcher entered once, interior pcs (0x2bd8d18/0x2bd8d30/0x2bd8d54/0x2bd8dac/0xbd1d68)
  = 0 hits.
- FORWARD (ALSO seed 0x107275550 = the getter's true read; DMTRACE probe confirms the guard's
  cell = host-JavaVM-ish value while the true cell held a live host heap object): **`StartLuaAppDM
  returned Ok(M)`** where M = the fabricated manager pointer (4/4, e.g. 0x7f5e645a5970). The
  session now carries M down the call instead of the benign 0x3e8.

## What this does and does NOT do (honest)
- **DOES:** corrects an 8-cycle wrong-address seed. The manager the DMCONT continuation routes
  (vt[+0x1f0]=REAL continueAfterFlagsLoaded_ 0x102bd1d68) now actually reaches the getter the
  dispatcher calls. StartLuaAppDM returns the fabricated manager object.
- **DOES NOT (yet):** the dispatcher interior + sub_2bd8dac + continueAfterFlagsLoaded_
  (0x102bd1d68) still 0 hits under the INTERIOR region watch (0x102bd8d18-0x102bd8d98), and the
  marshaller 0x1023f075c / EC world 0x102e24598 stay 0 hits. So delivering M into the getter is
  necessary-but-insufficient for a live DM — the dispatcher's first-call-boundary leaf path still
  terminates upstream of the bl sub. The wall is NOT merely the seed address; a real
  flags-loaded engine-init session remains the standing structural gate. M is a fabricated
  all-leaf manager, not a live DM; no make_shared<DataModel> captured (JIT_DM_ALLOC_CAPTURE 0).

## Next (honest, single-agent)
The corrected cell is now seeded unconditionally (both cells) so no future cycle re-introduces
the wrong-address seed. The forward hook stays: get the dispatcher past its leaf path to
`bl sub_2bd8dac` -> vt[+0x1f0] -> REAL continueAfterFlagsLoaded_. That requires M's leaves
(vt[+0xf8]/[+0x108]) to return a value that lets the dispatcher's 2nd frame proceed — the same
live-flags state a real session produces. No new lever claimed headlessly.

## Files / verification
- `crates/arm64jit/src/jit.rs`: `routeb_dm_manager_guard` now ALSO seeds 0x107275550 (the
  getter's true cell) under DMFORCE in the fnB region (idempotent, env-gated as before);
  `sh243_page_mapped` helper; new hermetic `sh243_manager_guard_also_seeds_the_getter_true_read_cell`.
- `crates/arm64jit/examples/elfjit.rs`: corrected the sh240 pin's semantic label (getter ldar
  reads 0x107275550, not 0x102727550). Byte words unchanged.
- cargo build/test --workspace green; lib 391/0; examples 85/0; recon-v3 plane re-verified.