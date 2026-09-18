# Frontier SH298 — coherent registration object crosses SH297's wall into the EC world

Date: Sep 18, 2026, hermes-worker. Single-agent (cone suppressed). Default-inert
(opt-in `--v2boot-session-dmfn` + `JIT_ROUTEB_DMFN_FIELDS=1` +
`JIT_ROUTEB_DMFN_REGISTER=1`).

## tl;dr

SH297 drove the DM-construction fn 0x1023f03b4's body through its arg1-clobber
gates and died in the application-server registration fn **0x21e45c8** (fault
guestpc=0x1021e460c, `ldr x0,[x21,#8]`) — explicitly "one BL before
nativeAppBridgeAppStart". SH298 fixes the registration object the body passes
and **crosses that wall entirely: the construction fn now drives into the
ExperienceController world entry 0x102e24598 (`stp x29,x30,[sp,#-96]!`) — the
region SH235/255 measured as ZERO headless hits — and terminates for the first
time at a NULL+0x48 live-object deref inside the EC marshaller-adjacent fn
0x1023f1354.** Deterministic 3/3 at guestpc=0x102e245f4.

## Why SH297 faulted (the mis-seeded registration object)

The construction body's post-gate path builds the app-server box (x26 from
0x23f0558) then calls the registration fn:

```
23f05b0: mov x1,x26          ; arg1 = app-server box
23f05b8: bl  0x21e45c8       ; registration (arg0 = x0)
```

`arg0` = `x0` = `[this+136]` **double-deref**: the body loaded `x25=[this+136]`
(from `ldr x25,[x20,#17]` @0x23f0520) then `x0=[x25]` (@0x23f0594). SH297 seeded
`arg1[8]=singleton` and the body CLOBBERS `this+136` ← `arg1[8]`, so
`x25 = singleleton`, `x0 = [singleton] = singleton's VTABLE` (a host-leaf table).
Inside 0x21e45c8, `[arg0+8]` read the vtable's slot-1 = leaf addr
`0x7f00000001d0` → `ldr x8,[x22,#24]` @0x1021e4620 derefed `0x7f00000001e8`
(a host-thunk/leaf region, not an object) → SIGSEGV. SH297's "fault at
0x1021e460c" was the JIT reporting the block-entry pc; the real fault is the
`[x22+24]` deref.

## SH298 fix

`arg1[8]` now points to a **CELL** whose value is a coherent registration object
R: `[R+0]` = benign leaf-vtable, `[R+8]` = `routeb_singleton_obj_addr()` (a VALID
nonzero host object; `[o+24]=0` so the fn's `[x22+24]` read is benign). The double
deref then resolves `arg0 = [cell] = R`, a real object, and 0x21e45c8 COMPLETES.
The body falls through to `bl nativeAppBridgeAppStart` (@0x23f05f8) and, driven
further, reaches the EC world.

## Measured (real libroblox.so, full SH297 base env + JIT_ROUTEB_DMFN_REGISTER=1)

- SH297 (arg1[8]=singleton): 3/3 fault addr **0x7f00000001e8** (host-leaf) at the
  0x21e45c8 registration block.
- SH298 (arg1[8]=CELL->R): the 0x21e45c8 wall is GONE. 3/3 terminal SIGSEGV
  **guestpc=0x102e245f4** (fault=0x48, x21=x23=x28=0) — the NULL+0x48 live-object
  deref inside fn 0x1023f1354, reached via the EC entry's `bl 0x102e245f0`.
- The EC entry **0x102e24598** is exactly SH235's pinned genuine DM-creation world
  entry (`stp x29,x30,[sp,#-96]!` = 0xa9ba7bfd) — previously measured 0 headless
  hits across SH231/235/255. This is the first headless penetration of that region.

## VERDICT (do-not-re-tread)

- The construction fn's one-next-unsynthesized-object loop advanced one full gate:
  past the SH297 registration wall and INTO the ExperienceController world.
- Route-B live-DM structural gate STILL holds (no make_shared<DataModel>; MH_*
  false, DM-root 0) — the new terminal is a NULL+0x48 class live-object deref
  inside the EC marshaller, the SH174/SH204 fabricatable-object-graph class,
  not a fixed-.bss seed.
- This is cause-not-symptom SESSION-CTOR: the EC world (the genuine DM-creation
  machine) is now headlessly REACHED, narrowing the "unreachable/measured-inert"
  history (SH231/235/255/232) to "reached, needs a live sub-object at +0x48".
- SH174 capture-latch (arm *(0x106391908) at a real make_shared<DataModel>) stays
  the single forward hook.

## Verify

- `cargo test -p arm64jit --example elfjit -- sh298` = 1 passed.
- `cargo build -p arm64jit --example elfjit` EXIT 0 (117/118 warnings, pre-existing).
- elfjit.rs 1,048,408 + HANDOFF under the 1MB pre-commit hook (prose trimmed).

## NEXT GATE (for the next frontier session)

The new terminal is `SIGSEGV [x25,#72] @0x102e245f8` (fault=0x48) where
`x25 = x21 = x1(EC arg1) = 0` (EC entry `mov x21,x1` @0x102e245e4, after the
`bl 0x1023f1354` helper completes). The EC marshaller reads `[arg1+0x48]` as a
live byte/valid-object flag immediately — exactly SH235's "immediately derefs
live objects, NOT a static seed". This is the SH174/SH204 live-object class,
now REACHED INSIDE the genuine DM world (was: only located/measured-unreached).
A static arg1 seed would likely advance one fencepost then hit the next live-object
deref; the cause-not-symptom path is a REAL session producing a coherent
ExperienceController/app-request object (SESSION-CTOR). Do-not-re-tread broad
static seeds per SH174/204/248h/256 doctrine; per the operator's MIGRATION-IS-NOT-
A-STOPPING-POINT, keep grinding the DM-construction/DMCONT/PATH-B line inside
this JIT rather than folding to a host-migration verdict.

- `crates/arm64jit/examples/elfjit.rs` (SH298 stage3: CELL->R coherent registration
  obj `[R+8]=singleton` under `JIT_ROUTEB_DMFN_REGISTER`; +hermetic sh298).
- `runs/capture_sh298_register.sh`, `runs/sh298-a-sh297-*.txt`,
  `runs/sh298-b-register-*.txt`, `runs/sh298-b2-r*.txt`.
- `HANDOFF.md`, `runs/STATUS.md` (ledger). `docs/frontier-sh298-registration-ecworld.md` (this).