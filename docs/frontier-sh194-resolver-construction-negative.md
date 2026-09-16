# SH194 — resolver-map construction: mechanism-level recon-negative (do-not-re-tread)

## Date / context
Sep 16, 2026 (hermes-worker). Route-B line after SH193. SH193's stated next lever
was: "locate the RESOLVER map's own constructor/insert helper (sibling to the
register writer but with {key_ptr,key_len,classid} element layout) and drive it
headlessly to construct+populate 0x106dca0e70 via engine code — the missing
bridge." This doc answers that lever **at the mechanism level** (fresh
disassembly + region-watch + an empirical registrar drive), and records a
do-not-re-tread recon-negative.

## Findings this cycle (fresh disassembly, real libroblox.so)

1. **The resolver map 0x106dca0e70's ONLY writer is the bulk registrar 0x2208ae8**
   (`nativeGameGlobalInit+0x26e4`). ABI (disasm): `x0 = &{key_ptr, key_len}`
   (16 bytes), it hashes the key into the open-addressing table (stride 0x18,
   `classid@+0x10`), rehashes via 0x5fb2948 if the load factor is exceeded, and
   returns `x0 = &element.classid` for the caller to fill. Its ONLY reader is the
   resolver 0x2373cec (`ldp begin,end,[x0]; b.eq not-found` = read-only probe),
   reached from the getService walker 0x105e09bc8 and ~8 service consumers.

2. **The reg's per-class register writer 0x1dc4bc8 targets the REGISTER map
   0x106dca0f60 — NOT the resolver.** The resolver map is populated by a *bulk*
   registrar that runs inside nativeGameGlobalInit, driven from a SOURCE
   descriptor list (the source vector at 0x6dca0ea8, iterated at 0x22085c4).
   When the source list is empty (pre-world-build) the registrar has nothing to
   insert — the resolver map stays {0,0} EMPTY.

3. **The 48-byte map header IS written in-ladder** — but it is written as
   zeroed `.bss` from a stack default-ctor (copy at nativeGameGlobalInit+0x2014
   / 0x2208418), i.e. `{begin=0,end=0,bucket=0,...}`. Region-watch on
   `0x102208418-0x102208620` fires **6x every clean ladder run** (pcs
   0x10220843c/450/47c/504/584/5c0). So the ladder DOES execute the header
   write + the registrar region — but the header is a DEFAULT-EMPTY
   construction, NOT a live-unordered_map-with-bucket-sentinel.

## Empirical result (this cycle, real binary, llvmpipe, EXIT 124/139)

A new opt-in drive was added then REMOVED (kept only the diagnostic header
read-back). The attempt drove 0x2208ae8 **in-context** (at StartLuaAppDM, post
nativeGameGlobalInit, after ensure-writable on the resolver pages) with a
fabricated `{&"PlayerGui", 9}` key tuple, then wrote 0x87e into the returned
classid slot:

```
SH194: resolver-map header 0x106dca0e70 -> [ff..f x8]   (page_is_mapped=false => unmapped at guard time)
SH194: registrar DROVE ok classid-slot=0x7fce84035490 ... (a HOST addr, NOT a guest element)
libc++abi: terminating due to uncaught exception ... bad_weak_ptr
```

**The registrar returned a HOST slot (0x7fce...) and the run aborted with
`bad_weak_ptr` — the exact SH192 symptom.** The resolver map page reads as
unmapped at guard time (the JIT lazily maps guest pages on first guest access,
which is why the walker's own probe reads it without faulting); the map is
genuinely **unconstructed** — its libc++ bucket-sentinel / allocator (built only
by a real unordered_map ctor inside a live world-build) is absent. The registrar
hashes against the null bucket state and writes host-side, corrupting shared
state.

## Conclusion — mechanism-level recon-negative

- `getService("PlayerGui")` name→classid resolution (and thus the linked-node
  RETURN from the SH191 walker drive) sits behind a **live class-registry
  world-build** that constructs 0x106dca0e70's bucket array via a real lazy-static
  ctor. That ctor is **not headless-reachable** (SH192 + SH194 both corrupt on
  a registrar drive; SH193 found no static/.init_array ctor for the resolver map).
- This is the same live-DM-migration pattern as the type-4 producer, the DM
  capture latch, and every other Route-B lever: the plumbing is live-correct and
  latent; the firing event is the real app-launch session.

## Do-not-re-tread
- Do NOT drive 0x2208ae8 (or hand-write the resolver hash element) headlessly —
  it returns a HOST slot + bad_weak_ptr (SH192 3/3, SH194 1/1). Not constructive.
- Do NOT re-tread the register writer 0x1dc4bc8 → it targets 0x106dca0f60, a map
  the resolver does not read.

## What IS shipped
- A benign diagnostic in `routeb_dm_service_resolve_guard` (jit.rs): reads back
  the full 8-word resolver-map header (0x106dca0e70) at drive time and prints it,
  so future runs can cheaply confirm the map remains EMPTY/non-constructed.
  Default-inert under JIT_ROUTEB_DM_SERVICE_NODE; no behavior change on the
  clean SH191 walker drive (re-verified EXIT 124, 0 SIGSEGV/SIGABRT, walker
  executes clean not-found). Workspace green (558/0).

The Route-B sub-frontier (PlayerGui self-construct → service node → getService
resolution) is now closed at the same evidentiary depth as the live-DM gate:
instance self-construction (SH190d/e) + service-node attach (SH191) + clean
walker execution are real, shipped, reproducible; **name→classid resolution is
the one remaining piece and it is live-world-build-gated**.