# SH85 — qsort interpose (guest comparator) + generalized map +0x18 repair

## Result (wall ADVANCE: 4 gates cleared this session)

SH83 cleared the insert hash-dispatch gate (+0x18 garbage). SH84 generalized it
to ALSO repair +0x18 whenever it holds a NON-IMAGE address (any map of the family,
not just the +0x10==0x102a25dec string map) and to seed the empty-map header +
zeroed bucket array once per map at the insert entry. SH85 added a `qsort`
interpose: host glibc qsort was natively executing the guest's ARM64 comparator as
x86 (SIGSEGV at file 0x28bbfc0 — the enum-descriptor sort). The ladder now runs
deeper than ever, past all these and into the OTel/pb_defaults descriptor
registration (deepest fault guestpc 0x1029b43f0). Product path (no --v2boot) and
workspace green.

## SH85a — real glibc qsort drove a GUEST comparator natively (the 0x28bbfc0 gate)

Read-only recon deleg_178d6f09 root-caused the qsort-comparator SIGSEGV: `qsort@plt`
binds to REAL glibc qsort (resolver.rs resolve → dlsym, no interpose). glibc then
natively `call`s the guest comparator 0x28bbfc0, executing ARM64 bytes as x86 — the
first two bytes `08 18` decode `or [rax],bl`, and with leftover `rax=0x2` the memory
op hits ~address 2 → SIGSEGV fault=0x2 (matches the observed crash exactly). This is
the SAME class the codebase already handles for `pthread_once`, `dl_iterate_phdr`,
`cxa_thread_atexit` ("guest bytes are ARM64, not x86") — `qsort` was simply never
covered.

### Fix (shims.rs `bionic_qsort` + jit.rs `run_guest_callback_on`)
- Register `qsort` as a named shim (register_shims → register_named) so a guest
  `bl qsort@plt` binds to OUR interpose instead of real glibc qsort.
- The interpose runs REAL glibc qsort with a HOST comparator trampoline.
- **Primary path (desync-safe):** for the KNOWN +24-u32-key comparator
  (file 0x28bbfc0, `sign(u32[a+24]-u32[b+24])` over 0x50-byte records), sort ENTIRELY
  HOST-SIDE reading the +24 u32 keys directly — zero JIT re-entry. This matters
  because the ladder runs qsort on a detached thread CONCURRENTLY with the render
  thread's jit_runs, and re-entering the guest comparator ~N·log2(N) times via
  `run_guest_callback` (a fresh nested jit_run each) trips the SH55/SH64 shared
  block-cache desync → native SIGSEGV. A pure host comparator has zero JIT
  involvement.
- **Fallback (generic correctness):** for any OTHER comparator address, re-enter the
  guest comparator through the JIT via `run_guest_callback_on` on a CACHED per-thread
  1 MiB stack (the existing `dl_iterate_phdr` trampoline pattern). Added
  `run_guest_callback_on(fn, args, tp, stack, stack_size)` so a hot comparator can
  reuse one stack instead of run_guest_callback's fresh 1 MiB leak per call
  (~1.5 GiB for the 167-record descriptor sort).

## SH85b — generalized the map +0x18 repair to the whole map family

After the qsort gate cleared, the ladder advanced into a SECOND sphere of the engine
map family: the rehash/grow op at file 0x29f4258 (`ldp x1,x8,[x19,#16]; blr x8` over
a NEW map after expansion), whose +0x10 is a DIFFERENT span/string-view hash
(0x1029b4a84) and whose +0x18 held garbage 0x1800064 → SIGSEGV guestpc 0x1029f4284.
The old +0x10==0x102a25dec guard skipped this map. Refactor: fire the SH83/84 hook at
ALL the map-op entries (0x1029f3e70 insert, 0x1029f4258 rehash/grow, 0x1029f4088,
0x1029f4348) and:
- REPAIR `+0x18 → 0` whenever it is a NON-IMAGE address (a real second-level hash
  always lives in .text; garbage host-heap/ints do not) — safe for every map of the
  family, regardless of the +0x10 hash.
- SEED the empty-map header + zeroed 1024x8 bucket array ONLY at the insert entry
  (0x1029f3e70), once per distinct map — the rehash/grow ops reuse an already
  populated array and must NOT be force-emptied.

## New (hermetic) regression

`qsort_interpose_hostside_u32_key_comparator_orders_and_is_named` pins the known
comparator address (file 0x28bbfc0 / guest 0x1028bbfc0) and the +24-u32-key ordering
semantics the interpose's host-side path sorts with. Workspace **514/0** (was 513/0).

## Repro

`runs/capture_v2boot_sh82.sh` (JIT_ROUTEB_HASHFIX=1). Expect the seeded
`main-thread-id`, the map `+0x18 -> 0` + `empty header ... seeded` lines, the OLD
gates (0x1029f3f7c insert / 0x1029f3f84 bucket / 0x1028bbfc0 qsort comparator)
ALL GONE, and the ladder faulting deepest at guestpc 0x1029b43f0 (pb_defaults /
OTel descriptor registration). On a lucky timing the run even reaches exit 124
(clean stable idle — the qsort gate is fully cleared either way).

## Next (ranked)

The ladder now faults deepest at guestpc 0x1029b43f0 (file 0x29b43f0): this is the
OTel/pb_defaults descriptor-registration path (`adrp 0x6838000` / `ldr [x #0x378]`
map slot / `blr` through a fn-ptr slot of the descriptor table). fault=host heap ptr
→ the code dispatched through a host-heap fn-ptr (uninitialised registration table
slot) — the same "uninitialised-heap fn-pointer slot" class as every earlier gate.
Next: (a) identify the descriptor table/vtable whose per-key fn-ptr slot holds the
garbage and seed it (likely a per-OTel-resource-schema callback / descriptor stable
fn-ptr), mirror SH81's routeb_singleton_leaf / SH83-84's block-entry hook; (b) keep
advancing toward nativeGameGlobalInit returning → rung 2 nativeUpdateAdapterInit
(0x10221c3ec) → check rungs 2-6 install the type-4 producer vector [0x106829ea8];
(c) wire NativeHelper callbacks. Standing structural wall otherwise unchanged (real
self-constructed session still gated behind the full GlobalInit completion).