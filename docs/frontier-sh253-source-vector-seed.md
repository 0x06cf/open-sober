# SH253 — seed the bulk-registrar SOURCE vector so the engine's OWN in-ladder loop builds the resolver (MEASURED INERT)

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green.
Hermetic: `sh253_source_vector_guard_is_env_pc_gated_and_seeds_registrar_source`.
Env: `JIT_ROUTEB_SOURCE_SEED=1` (default-inert).

## Why (implements SH252's explicitly-remaining forward lever)

SH252 closed "the resolver map has no lazy-static ctor" and left ONE forward step
verbatim: *"seed the SOURCE vector 0x6dca0ea8 so the engine's OWN in-ladder
registrar loop populates the resolver — never attempted in SH192/194 (they drove
0x2208ae8 standalone against a zeroed header)."* This cycle implements exactly that
and MEASURES it.

The resolver 0x106dca0e70 — the name->classid map the getService walker
0x105e09bc8 resolves through (resolver 0x2373cec), whose [element+16]=classid
dispatches the DM service-list walk — is built ONLY inside nativeGameGlobalInit by
(a) the 48-byte header default-construct at 0x2208418 and (b) the bulk registrar
0x2208ae8, which is DRIVEN by the in-ladder SOURCE loop at 0x22085c4:
`adrp x19,6dca000; add x19,#0xea8; ldp x21,x22,[x19]` reads the SOURCE vector
{begin@0x6dca0ea8, end@+8}; `cmp x21,x22; b.eq skip` early-outs when EMPTY
(headless: begin==end==0). The missing bridge was: nobody seeded the source and
let the engine's loop drive the registrar.

## Implemented (default-inert, opt-in `JIT_ROUTEB_SOURCE_SEED`)

`routeb_source_vector_seed_guard` (jit.rs), fired on ANY block entry inside the
nativeGameGlobalInit body [0x102206404, 0x10220881c) (the rung-1 jit_run target, so
the seed is installed before the registrar block executes). It builds a leaked
array of 3 class-name descriptors (PlayerGui 0x87e [SH189-proven], ScreenGui
0x892, CoreGui 0x77f) — each descriptor [desc+8]=ptr to a LONG-form SSO class-name
string (byte0 cap bit0=1|len<<1, [8]=size, [16]=data), [desc+16]=classid (the
walker reads [element+16]) — and writes {begin=arr, end=arr+3*8} into every empty
slot of the source family {0x106dca0ea8 (loop-read), 0x106dca0e08 (sibling), 
0x106dca0e90 (registrar source map)}. Idempotent; OnceLock.

## Measured (real libroblox.so, canonical completing --v2boot ladder, 3-run batch)

- **Seed fires on every run** (3 descriptors + 3 source containers logged), well
  before the registrar block: the source containers ARE non-empty at the registrar
  block (readback: SRC 0x106dca0ea8 = {0x55ccc8077310,0x55ccc8077328} — the 
  seeded 3-element array).
- **BUT the resolver is NOT populated**: resolver header 0x106dca0e70 stays
  {0,0} and the SOURCE-loop body (block entry 0x1022085d4) is NEVER entered
  (region-watch 0 hits across all runs while block 0x1022085c0 fires). The
  registrar's source-loop does not translate/execute its per-element body on the
  completing DMCONT ladder EVEN with a non-empty source vector.
- Termination UNCHANGED and STABLE: 2x EXIT 134 / 1x EXIT 139, all at the standing
  app-start live-object map wall guestpc **0x1021dde34** (SH248g/h/249/250/251
  class — the live host-heap map construction that needs a real DM ctor).

## Verdict (do-not-re-tread beyond this)

The SH252 "source-vector bridge" is now MEASURED, not just judged: seeding the
source so the engine's own registrar could build the resolver does NOT advance
past the live-DM wall on the DMCONT ladder — the registrar's source-loop is
reached as a block but its body (per-element registrar drive + resolver insert)
does not execute headlessly, and the resolver stays empty. This is consistent
with SH192/194's warning (the registrar path needs a genuine live world-build
state) and the SH174 live-DM structural gate: the resolver is a downstream
consumer of a session, not its cause. The three levers at 0x1021dde34
(static-seed SH248g, runtime-repair SH248h, segment-protected SH249, count-clamp
SH250, dynamic-ctor-trace SH251) all stay measured-closed; the resolver-source
seed is a 6th adjacent measured closure.

## Honest (do-not-over-claim)

Does NOT manufacture a DataModel; Route-B live-DM structural gate UNCHANGED
(SH174 capture-latch at a real make_shared<DataModel> stays the single forward
hook). recon-v3 (24 task-driven frames / json-zero-fix) stays shipped + verified.
Single-agent; no production code path edited (all guards default-inert). The guard
is SHIPPED (default-inert) as the durable, hermetic-tested implementation of the
SH252 named lever plus its measured inert result — a future session gets the
exact "seed fires but loop body never enters / resolver stays {0,0} / EXIT 134 at
0x1021dde34" marker instead of re-treading.