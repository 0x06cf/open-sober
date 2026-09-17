# SH248e — seed the app-start once-cell global; the DMCONT continuation clears the
# pthread_mutex_lock once-branch and advances one gate deeper in nativeAppBridgeAppStart

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green.

## Context (SH248d, this session)
SH248d seeded the cookie-jar container globals [0x106ed7a20]+[0x106ed7a28], which cleared the
NULL-dest string-copy fencepost (0x102b504e4) and let the DMCONT continuation advance DEEP into
nativeAppBridgeAppStart. The next measured fencepost was `SIGSEGV guestpc=0x102339208 (lr
0x102339018) fault=0x0, x0=0x0, x1=0x0`. This cycle resolves it.

## The wall (measured, real libroblox.so)
The crash is NOT a live-object range (SH248d's initial framing) — it is a **seedable fixed .bss
once-cell global**. Disasm of the once-check fn at guest 0x102339208 (vaddr 0x2339208, `bl
2339208` @ 0x2339014 inside nativeAppBridgeAppStart, lr=0x102339018):

    adrp x9, 6b0b000; ldr x0,[x9,#3568]; ...; ldar x8,[x0]; cmn x8,#0x1; b.eq 0x2339264

`ldr x0,[x9,#3568]` loads a POINTER global **guest 0x106b0bdf0** (vaddr 0x6b0bdf0, in the RW
data segment, fixed address). That global is NULL headlessly, so `ldar x8,[x0]` derefs x0=0 ->
SIGSEGV fault=0x0. If the pointed-to cell holds -1, the `cmn x8,#0x1; b.eq` takes the clean
canary-check+ret path (0x2339264) instead of the pthread_mutex_lock branch (`bl 2b4cd1c`).
This is a thread-safe static-local guard ("already initialized" == -1), so seeding -1 gives the
intended early-return.

## What was done (one default-inert, opt-in guard)
New `routeb_appstart_once_seed_guard` (opt-in `JIT_ROUTEB_APPSART_ONCE_SEED=1`) fires on any
block-entry pc in the once-check fn [0x102339208, 0x102339244) and seeds [0x106b0bdf0] = a
leaked 8-byte cell holding -1 when the global is 0, idempotently.

## Measured (real libroblox.so, canonical --v2boot ladder, full DMCONT env + SH248d jar seed)
- Before: SIGSEGV guestpc=0x102339208 fault=0x0 every run (6/6 in SH248d batch).
- After (once seed ON): `[routeb-sh248e] seeded appstart once-cell global [0x106b0bdf0] = <ptr>
  (-1 cell) at pc=0x102339208` fires, the 0x102339208 crash is GONE, and the continuation
  advances ONE MORE gate into nativeAppBridgeAppStart: region-watch then reports the block at
  guest pc=0x102339018 and the NEXT fault is `SIGSEGV guestpc=0x102339020 ... x0=0x0, x1=0x0`
  (vaddr 0x2339020). 

## The next fencepost (honest)
At 0x2339020 the app-start derefs an adapter/closure object whose pointer is loaded from the
fixed global **guest 0x106b0bde0** (the `JNIAppLifecycleNativeAdapter` triplet; setActive
0x21f5f80 copies [0x106b0bde0]/[0x106b0bde8] into the frame at 0x233901c). That object pointer
is NULL headlessly -> `ldr x8,[x0]` (vtable) faults x0=0 at 0x102339020. This is a
fabricated-object-class wall (the SH174/SH204 live-object-lifetime family) reached from inside
the real app-start path — the DMCONT continuation has now cleared 4 quantized seeding gates
(bad_alloc, app-name NULL-store, cookie-jar, once-cell) and moved the app-start deeper each
time.

## Honest (do-not-over-claim)
- Does NOT manufacture a DataModel; Route-B live-DM structural gate UNCHANGED; SH174
  capture-latch stays the single forward hook.
- What IS new + measured: the once-cell global [0x106b0bdf0] is a seedable fixed .bss cell
  (corrects SH248d's "NULL-range = live session/container array" framing for THIS specific pc),
  the seeding guard fires and is idempotent/env-gated/pc-gated, and the continuation advances
  one more quantified gate (0x102339208 cleared, next wall 0x102339020).

## Next (honest, single-agent)
The 0x102339020 wall is the app-lifecycle-adapter closure at guest [0x106b0bde0]. Fabricated-object
class: seed [0x106b0bde0] with a coherent leaf-vtable object (SH99/SH101 pattern) so the
`ldr x8,[x0]; ldr x8,[x8]; blr x8` dispatch (0x233903c..0x233904c) resolves a benign host leaf,
or determine it is the SH174/SH204 live-object wall and stop seeding there. Standing forward
hook unchanged.

## Code / files
- crates/arm64jit/src/jit.rs: `routeb_appstart_once_seed_guard` (opt-in
  JIT_ROUTEB_APPSART_ONCE_SEED, fires on [0x102339208,0x102339244), seeds leaked -1 cell into
  [0x106b0bdf0] when 0, idempotent); wired into the block-entry dispatch after
  `routeb_appstart_jar_seed_guard`; + hermetic sh248e unit test
  (env-gated / pc-gated / seeds -1 / idempotent). Default-inert.
- repro runs/batch_sh248e_once_seed.sh.
- cargo build --workspace + cargo test --workspace green (arm64jit lib 395/0, sh248e passes).