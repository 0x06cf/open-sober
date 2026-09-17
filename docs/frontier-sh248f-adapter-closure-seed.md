# SH248f — fabricate the app-lifecycle adapter at [0x106b0bde0]; the DMCONT continuation
# clears the closure-dispatch NULL wall and advances ~0x500 deeper (to 0x1021dde34)

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green.

## Context (SH248e, this session)
SH248e seeded the once-cell global [0x106b0bdf0] -> -1, which cleared the SIGSEGV at
0x102339208 and advanced the DMCONT continuation into nativeAppBridgeAppStart to the NEXT
fencepost at 0x102339020 (NULL adapter object from fixed global [0x106b0bde0]). This cycle
resolves it.

## The wall (measured, real libroblox.so)
After the once-cell seed, the app-start continues at 0x2339020 (guest 0x102339020):
setActive (fn 0x21f5f80, bl'd @ 0x233901c) copies the app-lifecycle adapter triplet from
fixed globals **[0x106b0bde0]**/[0x106b0bde8] into the frame, then
`ldp x0,x19,[sp,#16]; ...; ldr x8,[x0]; ldr x8,[x8]; sub x2,x29,#0x40; mov w1,#3; blr x8`
(0x2339020..0x233904c) virtual-dispatches the adapter's vt[0] (a std::function closure).
[0x106b0bde0] is NULL headlessly -> `ldr x8,[x0]` (x0=0) SIGSEGV fault=0x0 at
guestpc=0x102339020. This is the SH101/SH165 fabricated-object class (a fixed .bss global
holder), NOT a live-host member.

## What was done (one default-inert, opt-in guard)
New `routeb_appstart_adapter_seed_guard` (opt-in `JIT_ROUTEB_APPSART_ADAPTER_SEED=1`) fires
at the block entry 0x102339018 (the resume of the once-fn `bl`, since the setActive call +
closure dispatch are MID-BLOCK under that entry) through 0x102339050, and seeds
[0x106b0bde0] = a fabricated all-leaf-vtable object (`routeb_appstart_adapter_object()`:
leaked 0x100 object whose vtable is a leaked 0x200 all-leaf region), so the
`ldr x8,[x0] -> vt[0] blr` resolves to a benign host leaf. [0x106b0bde8] is left 0
(setActive's `cbz x9, ret` skips the refcount when it's 0 — benign). Idempotent (only when
the global is 0).

## Measured (real libroblox.so, canonical --v2boot ladder, full DMCONT env + SH248e jar+once)
- Before (SH248e): SIGSEGV guestpc=0x102339020 every run.
- After (adapter seed ON): `[routeb-sh248f] fabricated app-lifecycle adapter object ...`
  + `... seeded app-lifecycle adapter global [0x106b0bde0] = <obj> (all-leaf-vt object)`
  fires; the 0x102339020 crash is GONE; the continuation advances **~0x500 further**
  through nativeAppBridgeAppStart's body — region-watch now reports hits at
  0x102339050, 0x10233907c, 0x102339c3c, 0x102339d0c, 0x102339d44 — before the NEXT fault:
  `SIGSEGV guestpc=0x1021dde34 ... x0=0x7fdfc1039140 (host), x19=0x40c29..., x20/x21=0x100548ca9`
  = a separate construction fn (JNI_OnLoad+0x69e34 region) computing element counts from
  LIVE host-heap size fields (x19/x20/u64 bit-twiddle), the SH174/SH204 live-object class.

## Honest (do-not-over-claim)
- Does NOT manufacture a DataModel; Route-B live-DM structural gate UNCHANGED; SH174
  capture-latch stays the single forward hook.
- What IS new + measured: the app-lifecycle-adapter holder [0x106b0bde0] is a seedable fixed
  .bss global; the fabricated all-leaf object clears the 0x102339020 closure-dispatch NULL;
  and the continuation now runs ~0x500 deeper into nativeAppBridgeAppStart than at ANY prior
  SH (SH248d stuck at 0x102339208, SH248e at 0x102339020, SH248f reaches 0x102339d44). Then it
  transitions to a NEW live-object construction fn at 0x1021dde34.

## Next (honest, single-agent)
0x1021dde34 is a live-object-size-arithmetic wall (host-heap size fields in x19/x20/x21,
0x40c29...-class) — the SH174/SH204 live-object class, NOT a fixed .bss seed. Determine if it
is a seedable singleton-size global or a live-member wall; if live, it is the standing
Route-B live-DM structural gate reached from deeper in app-start. Standing forward hook
unchanged.

## Code / files
- crates/arm64jit/src/jit.rs: `routeb_appstart_adapter_seed_guard` (opt-in
  JIT_ROUTEB_APPSART_ADAPTER_SEED, fires on [0x102339018,0x102339050), seeds [0x106b0bde0] =
  fabricated all-leaf app-lifecycle-adapter object, idempotent) + `routeb_appstart_adapter_object`
  (leaked 0x100 obj, 0x200 all-leaf vtable); wired after `routeb_appstart_once_seed_guard`;
  + hermetic sh248f unit test (env-gated / pc-gated / seeds leaf object / idempotent),
  serialized with CONT_MGR_TEST_LOCK for the fixed-.bss parallel-test race. Default-inert.
- repro runs/batch_sh248f_adapter_seed.sh.
- cargo build --workspace + cargo test --workspace green (arm64jit lib 397/0, sh248f passes).