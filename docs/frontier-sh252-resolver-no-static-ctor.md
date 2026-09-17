# SH252 — resolver map has NO lazy-static ctor headlessly-reachable (full-text sweep, corrected)

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green.
Hermetic: `sh252_resolver_map_has_no_lazy_static_ctor` (real-image guard, skip-if-absent).
Correction: an EARLIER draft of this cycle asserted "ZERO adrp+add reaches 0x6dca0e70" — that
was WRONG (a buggy hand-rolled decoder). The authoritative objdump + corrected decoder agree on
the real, defensible closure below.

## Why (answers SH193's explicitly-open lever at the mechanism level)

SH193's "Next lever" was verbatim: *"Locate the RESOLVER map's own constructor. If a resolver-map
getter/ctor exists (static or .init_array), driving it headlessly would construct a valid empty
resolver map on which the bulk registrar could then insert from a seeded source — the missing
bridge."*

Prior closures (SH192/193/194) established the resolver map 0x106dca0e70 is built ONLY by the
bulk registrar 0x2208ae8 iterating the SOURCE vector 0x6dca0ea8 — from **page-WRITER scans**
(who writes 0x6dca0e70). A function-local-static lazy ctor that CONSTRUCTS the map before any
write (getter pattern: adrp the guard word, `__cxa_guard_acquire`, default-construct the map)
would NOT appear as a page writer and would have been missed. This cycle performs the sweep prior
closures never did: a full-text `.text` (exec seg file [0x0,0x62d8190)) scan of EVERY
`__cxa_guard_acquire` (0x284ce54) caller for a resolver-header WRITE.

## Measured (real libroblox.so, exec seg = file [0x0, 0x62d8190))

1. **8,378** direct `bl __cxa_guard_acquire` (0x284ce54) call sites in .text (decoder-verified).
2. Exactly **11** `adrp xN, 0x6dca000 + add xN, xN, #0xe70` pairs form the resolver-map address
   0x6dca0e70 (objdump-verified list, pinned in the hermetic):
   - in-ladder nativeGameGlobalInit header default-construct **0x220841c**
   - bulk registrar 0x2208ae8 dest-map pointers **0x2208b10 / 0x2208be0 / 0x2208c9c**
   - **7 other readers/callers**: 0x25f8fd8 (V2UpdateSurfaceApp classid cache), 0x28442c8,
     0x31fcac4, 0x3ceca04 / 0x3cecd6c (class-reg), 0x4894100, 0x4b547f8.
3. **None of the 11 forms the resolver inside a lazy-static ctor construction body that then
   STORES into the header.** The hermetic sweeps every guard site's construction window (64 insns
   after the guard) for a store (STR X 0xf9000000 / STP X 0xa9000000 / STR Q 0x3d800000 — LDR
   families 0xf9400000/0x3dc00000 correctly EXCLUDED) targeting [0x6dca0e70, 0x6dca0e70+0x30):
   **0 ctors write the header.**
4. The one guard ctor that DOES form the address — **0x25f8fb8** (guard 0x25f8fc0 on once-cell
   0x6dbe888) — only passes &resolver to the **READ-ONLY probe 0x2373cec** (classid cache: adrp
   x0,6dca000; add #0xe70; mov x1,sp; bl 0x2373cec; ldr x8,[x0,#16]) and reads classid from the
   return. It NEVER stores to the resolver header — it is a consumer, not a constructor.

## Verdict (measured closure, do-not-re-tread)

The SH193 "locate the resolver ctor" lever is **answered as a measured negative with the correct,
objdump-grounded method**: there is NO lazy-static ctor that constructs 0x106dca0e70. The 48-byte
resolver header is written ONLY by (a) the in-ladder default-construct header copy at 0x2208418
(3 store ops: str q0,[x8]; stp x9,x10,[x8,#16]; str q0,[x8,#32]), i.e. the map object is a
default-constructed (empty) __hash_table whose bucket array rehash-on-first-insert (0x5fb2948 +
op_new) builds the real container — and (b) the bulk registrar 0x2208ae8 inserting into it. The
bucket-array construction is reachable ONLY via an insert, which requires the SOURCE vector
0x6dca0ea8 to be non-empty (SH194: empty headlessly -> registrar loop at 0x22085c4 early-outs).

**The remaining lever is UNCHANGED and remains the ONE forward step on this sub-frontier:** seed
the SOURCE vector 0x6dca0ea8 (in-ladder registrar loop 0x22085c4 iterates begin/end there; per
element it decodes an SSO string from [element+8], builds a {data,len} tuple, calls 0x2208ae8)
with valid class-name strings BEFORE nativeGameGlobalInit's bulk loop runs — then the engine's OWN
in-ladder code constructs + populates the resolver via its own first-insert/rehash. SH192/194 never
did this (they drove the registrar standalone at StartLuaAppDM against an unmapped/zeroed header →
host slot + bad_weak_ptr; nobody seeded the source vector and let the ladder's own loop drive it).

## Durable guard

`sh252_resolver_map_has_no_lazy_static_ctor` (elfjit.rs ~L13429):
(a) asserts the objdump-pinned 11-site resolver-forming set with exact addresses (a future
    added reader FAILS the count/list diff loudly);
(b) sweeps all 8,378 `__cxa_guard_acquire` sites and asserts ZERO ctor WRITES the resolver header
    (STR/STP/STRQ-only decode, LDR excluded) — the durable "no lazy-static ctor constructs the
    resolver" pin. Skip-if-absent real-image guard family (no libroblox.so -> pass).

## Honest

Does NOT manufacture a DataModel, does NOT lift the Route-B live-DM structural gate (SH174
capture-latch at a real make_shared<DataModel> stays the single forward hook). recon-v3
deliverables stay shipped + verified. Closes one explicitly-open doc lever with the correct method;
the SOURCE-vector seed bridge remains the concrete forward next-step
(matches SH193's "missing bridge" exactly).