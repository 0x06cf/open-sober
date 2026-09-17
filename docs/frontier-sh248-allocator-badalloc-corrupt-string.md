# SH248 — measure the real allocator at runtime: the continuation's `bad_alloc` is a CORRUPTED-STRINGOBJECT assign, NOT an allocator-capacity wall (corrects SH245-247's framing)

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green.

## Context (do-not-re-tread)
SH245 ACTIVATED the real continueAfterFlagsLoaded_ (0x102bd1d68) headlessly and stopped
one gate deeper: `std::bad_alloc`. SH246/247 attributed it to the engine allocator: "op_new
(both variants) returns NULL for size>0xa when [0x10727570c].bit0 is clear; the descriptor
path can't serve >0xa (3 measured mechanisms); the 0x70c size-class bootstrap is THE single
enabler for all of Route B." This cycle MEASURES the allocator at runtime and finds the
bad_alloc comes from a DIFFERENT, more precise mechanism.

## The live diagnostic (new, opt-in JIT_ROUTEB_ALLOC_PROBE=1)
A block-entry probe guard (~120 lines, zero default impact) fires across the whole real
allocator path — operator_new variant A 0x1db1a38, variant B 0x1d96768, the tail wrapper
0x1db1c60, and the free-list allocator tail 0x623fe1c — logging x0/x1/x2/x19/x20/x30 + the
allocator-activation byte [0x10727570c].

## Measured (real libroblox.so, canonical completing --v2boot ladder, DMFORCE+DMCONT
## +SH245 env, EXIT 134/139)
1. The allocator IS reached with sizes >0xa. `WRAP 0x1db1c60` fires with sizes 0x90, 0xa0,
   0x69, 0xe61 and the ladder proceeds past them — **the real allocator provably serves
   sizes >0xa headlessly.** So the ≤0xa-vs->0xa gate in SH245-247 is NOT the continuation's
   wall.
2. The free-list allocator tail 0x623fe1c is NEVER a block entry (it is the target of a
   direct `b` tail-jump, so it is mid-block — the SH217 class). Not observable via
   block-entry guards.
3. The continuation enters 0x102bd1d68, runs to 0x102bd1dfc, then the NEXT allocation is
   `OPERATOR_NEW x0(size)=0xfffffffffffffff7 (=-9) x30(caller)=0x102b506bc`, which returns
   NULL -> `std::bad_alloc`. **Deterministic** (identical bytes across every completing run;
   the -9 line precedes the libc++abi terminate every time).

## Root-cause decode (fresh disasm of 0x2b50600, the caller x30=0x102b506bc)
0x2b50600 is a libc++ `std::string::assign`/operator= leaf. It reads the destination
string object's capacity word `x8 = [this]`, clears the low owned-flag bit (`x9 = x8 & ~1`),
and when that OLD CAPACITY exceeds the max_size sentinel `0x7ffffffffffffff2`
(`cmp x9,x8; b.hi 0x2b50690`) it deliberately requests an allocation of `x23 = -9 =
0xfffffffffffffff7` (`mov x23,#-9` at 0x2b50694) then `bl operator_new` (0x2b506b4).
operator_new(-9): `cmn x2,#0xa` (=-9+10) overflows -> C=1,Z=0 -> `b.ls` NOT taken ->
`mov x19,xzr` -> NULL -> libc++ throws bad_alloc.

So the -9 is a SENTINEL emitted when the destination string's [this] word is garbage-huge
(> 0x7ffffffffffffff2). **The continuation assigns into a std::string object whose capacity
word is uninitialized garbage** — a corrupted guest-string state in the app-start/controller
construction, NOT an allocator capacity limit.

## Honest correction (do-not-re-tread the allocator-wall framing)
- REFUTED: "no size>0xa can be served headlessly" (WRAP serves 0x90/0xa0/0xe61).
- REFINED: the continuation's bad_alloc is ONE specific corrupted string object whose
  capacity word must be repaired/seeded (set to a valid empty/short-string word so the
  assign takes its normal realloc path), then the continuation can run its full body
  (nativeAppBridgeAppStart 0x2338ef4).
- Does NOT manufacture a DataModel; Route-B live-DM structural gate UNCHANGED. This is a
  measured correction of WHERE the continuation is blocked, with a concrete seedable lever
  for a future cycle (repair the garbage [this] capacity word — a different and smaller
  surface than the whole scudo size-class bootstrap).

## Code / files
- crates/arm64jit/src/jit.rs: `routeb_alloc_probe_guard` (opt-in JIT_ROUTEB_ALLOC_PROBE=1,
  wire-only at the block-entry dispatcher; zero default impact; logs the allocator path and
  the string-assign corrupt-size sentinel site 0x102b50600).
- crates/arm64jit/examples/elfjit.rs: +hermetic `sh248_allocator_enabler_family_pinned`
  (real-image byte-pins on the 7 allocator sites: op_new variants, flag-cell adrp/ldrb,
  tail wrapper, free-list tail, StartLuaAppDM own flag-read — drift fails loudly).
- repro runs/capture_sh248_allocprobe.sh + runs/batch_sh248_allocprobe3.sh.
- cargo build --workspace EXIT 0; cargo test --workspace green; elfjit examples green.

## Next (honest, single-agent)
The next gate is now concrete and seedable: find the guest std::string object whose
[this]/capacity word is garbage-huge when the continuation's app-start assign runs (the
caller at 0x2b50600), repair/zero its SSO word so the assign reallocs normally, and re-run
to see the continuation's NEXT fencepost. That is a different, smaller surface than the
whole size-class bootstrap SH247 proposed. Standing forward hook unchanged: SH174
capture-latch arming at a real make_shared<DataModel>.

## SH248b (follow-up, same session): minimal non-perturbing probe + the corrupt object captured
- **PROBE-PERTURBATION LEVER (measured):** the first all-thread/heavy-log probe shifted the
  run to a *deterministic* secondary-thread crash (guestpc=0x101d96768 fault=0x24 at
  operator_new B, `rbx_matches_gueststate=false`, harness-tid=4096) on every run (0/6 to the
  continuation), while probe-OFF reached the continuation bad_alloc 2/2. Cause = per-entry
  gettid syscalls + 265 eprintln lines perturbing the main-thread/secondary-thread race.
  FIX: probe now fires at the op_new entries but eprintlns ONLY the corrupt-size sentinel
  call (~1 line/run, no per-entry syscall). With this minimal probe the clean continuation
  run is RESTORED (3/4 EXIT 139 bad_alloc) and the object is captured.
- **THE OBJECT (deterministic, runs 1/3/4):** `CORRUPT_SENTINEL(size=-9) x1=0x55.. x30(caller)=
  0x102b506bc x19=0 x2=0 x20(this)=0x7f1dfc5a3ac8  [this+0]=0x1  [this+8]=0x7f1dfc033800(host
  ptr)  [this+16]=0x5`. `this` is a HOST-heap object (0x7f range), NOT in-image. Decode of the
  0x2b50600 absent-path: `x8=[this]=1 -> x9=0; cmp x9,x2(0) -> b.ls -> resize path; x9=oldcap-1
  =0xffffffffffffffff; cmp 0xffff..ff vs max 0x7fff..f2 -> b.hi -> x23=-9`. So the -9 fires on a
  LENGTH-0 assign into a host-heap string whose capacity processing underflows — the object is
  a host-heap std::string whose [this+0]=0x1 (owned-flag set, data-word 0) interacts with the
  resize path. Refined mechanism: not "garbage capacity" but a host-heap string state the
  resize underflows on. The engine's guest strings are host-heap objects (SH172-177 class).
- **NEXT lever (refined):** the failing construction is a length-0 assign at 0x2b50600 whose
  caller x30=0x102b506bc. Both the exact site and the object are now captured reproducibly
  with a non-perturbing probe. The fix is either (a) seed/initialize that host-heap string
  object's SSO word so the resize takes its normal growth (>0) path, or (b) identify the
  upstream constructor that left it empty-but-flagged and fix the headless init. Honest:
  does not manufacture a DM; AppBridge lifetime says a live DM is still gated upstream
  (SH174/SH204). The fatal alloc is now located + reproducible at exact addresses.