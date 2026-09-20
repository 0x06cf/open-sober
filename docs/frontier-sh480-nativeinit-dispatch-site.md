# Frontier SH480 — extend the SH200 singleton-dispatch regime with the nativeInitializeNativeFlags task-singleton site (session substrate atom 1/16)

## Session
Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). recon-v3 immediate-priority
deliverables re-verified GREEN at SH479 HEAD first (capture_taskv4_frame.sh: 24 real
task-driven frames `present swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 124 stable) and the
Route-B do-init baseline re-probed (substrate 14/16, once-guard bit0=1, DM-root
[0x106a68818]=0x0 -> LIVE DM=false; the two failing atoms are nativeInit "outside image").

## The gap closed
The ordered session substrate drives 16 atoms; exactly TWO have always stopped with
run-variable garbage pcs (nativeInitializeNativeFlags 0x10232048c + V2InitWithParams
0x102365c54) — the "nativeInit outside image" wall. SH200 already root-caused the CLASS
(singleton-dispatch: `bl 0x6249eb8` objB getter -> `ldr x8,[x0]` vtable -> `ldr x8,[x8,#N]`
slot past the 0x60 all-leaf seed -> `blr x8` reads host-allocation bytes) and patched FOUR
V2Init/V2Start sites. But nativeInitializeNativeFlags faults at a site SH200 did NOT cover.

SH480 disassembled the fault transitions (JIT_OUTSIDE_TRACE ring, real libroblox.so):
- nativeInitializeNativeFlags: last in-image 0x106251438 -> [0x1062514c0..0x1062514e0];
  fn 0x6251490 at 0x62514bc `bl 0x6249eb8` -> 0x62514c0 `ldr x8,[x0]` -> 0x62514d4 `ldr x8,[x8,#232]`
  (+0xe8, PAST the 0x60 seed) -> 0x62514e0 `blr x8` -> **`str x0,[x8]`** (0x62514e8: the COHERENT
  x0-store SH200 contract — patchable, the trailing store receives the stable singleton).
- V2InitWithParams: fn 0x62599e0, 0x6259a50 `blr x8` -> **`str s0,[x8]`** (0x6259a58, a
  float into the SHARED vtable) and the sibling 0x106260bf4 `blr x8` -> **`str w0,[x8]`**
  (word). Both are the SH200 FALSE-POSITIVE class: materializing the singleton into x0 does
  not satisfy the trailing store (it writes s0/w0, not x0) and NOPing the dispatch would
  clobber the shared vtable word 0. NOT patched — leave them, exactly as SH200's low-ROI
  boundary dictated.

So SH480 adds ONE safe site (the x0-store nativeInit site) as a 5th entry in
`routeb_patch_v2_dispatch`'s window list: patch [0x1062514c0..0x1062514e0] (9 words):
materialize `routeb_singleton_obj_addr()` into x0 (movz+3 movk) + NOP the blr; the existing
trailing `str x0,[x8]` stores the coherent object. Guarded by the same word0 check
(`ldr x8,[x0]` = 0xf9400008) so a wrong/unrelocated site is skipped, mprotect RW->RX around
the write, block-cache drop, idempotent (once-guard static). Env-gated (JIT_SH115_SINGLETON_PATCH,
already the SH200 gate); default path byte-identical.

## Measured result (real libroblox.so)
- "SH200 patched V2 dispatch @0x1062514c0 36B" now fires every run (5 sites, +1 new).
- nativeInitializeNativeFlags ADVANCES decisively: it logs the REAL engine line
  `[roblox:rbx.JNIRobloxSettings] nativeInitializeNativeFlags: Registered Flag Provider ID
  from Java` (never reached before — it previously died at the singleton dispatch before any
  flag-provider registration), and the fault moves from the patched wall to a DEEPER sibling
  accessor site in the same large family (run-variable: 0x106260bf4 / 0x21 — the documented
  ~600-site family, deterministic-only-with-a-discriminator-scanner per SH200).
- Honest: atom 1/16 still does NOT fully complete (a deeper sibling singleton site remains),
  so the substrate still reports 14/16 (2 stopped). This is a real, verified ADVANCE of the
  substrate's first atom (it crosses its first singleton wall and performs real
  flag-provider registration) without any regression — the change is inert+armable, the
  default 24-frame deliverable is byte-identical.

## Honest boundary
NOT a DM / NOT a live-DM step (Route-B live-DM gate UNCHANGED; DM-root 0). NOT a full clear
of the singleton family (SH200's own de-prioritized low-ROI boundary stands — the rest need
the discriminated scanner and are not the Route-B critical path). This is BUILD-THE-RUNTIME
substrate hardening: one more of the ~600 singleton-dispatch sites is deterministically
neutered, moving nativeInitializeNativeFlags' wall a stage deeper. No re-treads (SH200
patched the 4 V2 sites; this is the distinct nativeInit x0-store site SH200 never covered).

## Files
- crates/arm64jit/examples/elfjit.rs: 5th entry in `routeb_patch_v2_dispatch` sites + new
  hermetic `sh480_nativeinit_dispatch_site_x0_store_contract` (window length 9, guard
  `ldr x8,[x0]`, slot +0xe8 load word 0xf9407508, movz/movk round-trip, tail-nop).
- docs/frontier-sh480-nativeinit-dispatch-site.md (this).
- Commit on local dev only (operator pushes).

## Verify
- `cargo test -p arm64jit --example elfjit sh480` = 1 passed.
- `cargo test --workspace` EXIT 0 (arm64jit lib 684/0; elfjit examples +1).
- Real-binary re-verify: capture_taskv4_frame.sh still GREEN (24 frames) at this HEAD;
  sh415 do-init baseline unchanged.