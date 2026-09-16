# SH193 — class-name registry is a set of lazy-static unordered_maps; getService name->classid resolution is live-ctor-gated

## Date / context
Sep 16, 2026 (hermes-worker). Route-B line after SH192. The frontier is: the
engine SELF-CONSTRUCTED PlayerGui (0x106648950) + ScreenGui (0x106649ce0)
instances (SH190d/e), the PlayerGui instance is a real service NODE on
[dm+0x68] with classid 0x87e (SH191), and the engine's OWN getService walker
0x105e09bc8 EXECUTES CLEANLY but returns not-found because the name->classid
RESOLVER map 0x106dca0e70 is empty. SH192 located the bulk registrar 0x2208ae8
and proved forcing it standalone corrupts the shared map (bad_weak_ptr 3/3).

## New finding (fresh disassembly this cycle): the registry is three lazy-static maps

The class-name registry on page 0x6dca000 holds THREE distinct containers, each
a function-local-static std::unordered_map with its OWN __cxa_guard lazy ctor —
NOT one shared table:

| guest addr | role | builder / reader |
|--------|------|------------------|
| 0x106dca0e70 | RESOLVER map (name->classid, stride 0x18, classid@+0x10) | read by resolver 0x2373cec (called by getService walker 0x105e09bc8 + ~8 service consumers: 0x5e0b8.., 0x5e0f.., 0x5e09..). NO reachable ctor found. |
| 0x106dca0e90 | bulk-registrar SOURCE map | read by bulk registrar 0x2208ae8 (in nativeGameGlobalInit) |
| 0x106dca0f60 | REGISTER map (per-class write target) | written by per-class register writer 0x1dc4bc8 (via insert 0x1dc4c3c -> rehash 0x202d858); **lazy ctor located: 0x5fb2a78 (guard word 0x6dca0f90)** |

The register map's lazy ctor 0x5fb2a78 (freshly located):
```
0x5fb2a70  stp x29,x30,[sp,#-16]!
0x5fb2a78  adrp x0,6dca000; add x0,#0xf90     ; &guard 0x6dca0f90
0x5fb2a80  bl 284ce54                          ; __cxa_guard_acquire
0x5fb2a88  cbz w0, ret                        ; already-constructed -> ret
0x5fb2a8c  movi v0.2d,#0
0x5fb2a90  adrp x8,6dca000; add x8,#0xf60     ; &register map 0x6dca0f60
0x5fb2a98  adrp x0,6dca000; add x0,#0xf90
0x5fb2aa0  str xzr,[x8,#32]
0x5fb2aa4  stp q0,q0,[x8]                      ; zero [0x6dca0f60,+0x20)
0x5fb2aa8  b 284cf5c                           ; __cxa_guard_release
```
i.e. the register map's "construct" is a default-init that zeroes its 32-byte
container header. Because .bss is already zero, this ctor is functionally a
no-op headlessly — the header is already a valid default-constructed-empty
unordered_map. The bucket array is allocated LAZILY on first insert (rehash
0x202d858 from the insert helper 0x1dc4c3c).

## The getService gate, precisely

getService resolution is: walker 0x105e09bc8 -> resolver 0x2373cec reads RESOLVER
map 0x106dca0e70 {begin@0,end@8,bucket?,0x18,probe-ptr@0x20,classid@+0x10}.
0x2373cec is read-only: `ldp x25,x21,[x0]` (begin,end), `b.eq not-found` on
begin==end, then open-address probe. It NEVER constructs the map. When the
resolver map is empty {0,0} the walker cleanly returns not-found — exactly the
observed SH191 result.

Headlessly, even after the FULL ladder (nativeGameGlobalInit ran, rung 2), the
resolver map reads {0,0} (measured this cycle) — so the resolver map is
instantiated/populated only by a LIVE nativeGameGlobalInit world-build, and
there is no headless-reachable constructor for it (unlike the register map,
whose lazy ctor 0x5fb2a78 is at least a known function). The bulk registrar
0x2208ae8 is a mid-fragment of nativeGlobalInit (symbol
Java_..._nativeGameGlobalInit+0x26e4); driving it standalone with fabricated
keys but no constructed registry handed corrupts the shared page (SH192's
bad_weak_ptr 3/3) — do not repeat.

## Consequence for the Route-B line (honest)

SH191's service-node attach + clean walker drive are correct and LATENT: the
walker, the linked PlayerGui node (classid 0x87e), the self-constructed
instance, and the output materialization all fire the instant a live session
constructs the resolver map (real nativeGameGlobalInit world-build). This is the
same live-DM-migration pattern as the type-4 producer, the DM capture latch, and
every other Route-B lever on this box: the plumbing is live-correct, the firing
event is the real app-launch session. It is NOT a judgement — it is a concrete
control-flow gate: no headless-reachable code path constructs 0x106dca0e70.

## Next lever (for a future Route-B push, if the operator re-admits it)

Locate the RESOLVER map's own constructor. Unlike the register map
(0x5fb2a78), no lazy-static ctor for 0x106dca0e70 was found in this sweep — the
only page-0x6dca000 writers are the live nativeGameGlobalInit bulk registrar
(0x2208ae8) and the register writer (0x1dc4bc8, writes 0x6dca0f60). If a resolver-map
getter/ctor exists (static or .init_array), driving it headlessly would construct
a valid empty resolver map on which the bulk registrar could then insert from a
seeded source — the missing bridge. Until then, keep SH191's drive default-inert
and do NOT hand-write hash elements (24-byte open-addressing is undocumented and
SH192 proved corrupting).