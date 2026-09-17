# SH234 — recon-v3 render-side engine contract byte-pinned (+1 hermetic, real-image guard family)

## Why

The recon-v3 SELF-DRIVEN FRAMES deliverable (docs/recon-selfdrive-seed-jsonfix.md) is the
immediate-priority path where the engine presents REAL task-driven frames (`capture_taskv4_frame.sh`:
24 frames, `swap Ok(0x1)` ×24, no json abort). Its load-bearing ENGINE contract is:

- RENDERINIT enters at guest **0x105b3a280**
- engine make-current **0x105b3b358** (reached via RENDERCTX vtable `0x106731ae0` slot `[vt+16]`)
- frame-fn **0x105b32c00** (the clear-path frame the thunk drives with the fabricated renderer)
- swap / eglSwapBuffers **0x105b3b408** (via `[vt+24]`)

SH230 pinned the type-4 DISPATCH site (`adrp/ldr/cbz/br` into the seeded vector 0x106829ea8), but
these render-side engine functions had NO real-image byte-pin. A silent drift there would make the
type4_frame_thunk call a moved function or wrong vtable slot and break the 24-frame plane with no
loud failure — precisely the class of failure the real-image guard family exists to catch.

## What was pinned (+1 hermetic `sh234_recon_v3_render_side_engine_contract_pinned`)

Verified against the real 109 MB libroblox.so (same `load_real_image()` OnceLock-cache helper as
sh233, so it is safe under the parallel `--examples` batch):

- renderinit 0x105b3a280 = `0xa9bd7bfd` (stp x29,x30,[sp,#-0x30]!), +4 = `0xf9000bf5`
- make-current 0x105b3b358 = `0xa9bd7bfd`, +4 = `0xf9000bf5`
- swap 0x105b3b408 = `0xa9420408` (ldp x8,x1,[x0,#0x20]), +4 = `0xaa0803e0` (mov x0,x8)
- frame-fn 0x105b32c00 = `0xd10303ff` (sub sp,sp,#0xc0), +4 = `0xa9067bfd` (stp)
- window + 4-alignment guards for all four fns + the RENDERCTX vtable 0x106731ae0
- **vtable reloc-band check** via the loader's OWN packed-RELA decode: the ctx vtable band
  (file [0x6731000, 0x6732000)) must hold RELATIVE relocs with addend `0x5b3b358` (make-current)
  AND `0x5b3b408` (swap) — the two pointers type4_frame_thunk derefs through RENDERCTX
  (`[vt+16]`/`[vt+24]`). On-disk those slots are zero (relocation-populated at load), so the
  pin must go through the relocation stream, exactly as sh231 does for the DM-creation world.

## Verify

- `cargo test -p arm64jit --example elfjit sh234` → 1 passed (real-image pins confirmed).
- `cargo build --workspace` EXIT 0; recon-v3 plane verified green at this HEAD before/after.

## Honest boundary

Pure regression hardening of an already-verified deliverable; does NOT manufacture a DataModel,
does NOT lift the Route-B live-DM structural gate (SH209/218/223/224/228/231/232/233 unchanged).