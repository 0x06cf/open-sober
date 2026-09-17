# SH245 — getter tail→ret lets the dispatcher resume; the REAL continueAfterFlagsLoaded_ executes headlessly for the first time

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green
(arm64jit lib 394/0 incl. 2 new hermetic; elfjit examples 87/0; recon-v3 plane
re-verified earlier this cycle).

## Question (SH244's "next")
SH244 (mgr30=-2) made the engine-init getter 0x102174c04 self-invoke StartLuaAppDM,
but the increase in the FMOD-tail (0x624e6c0) consumed control and the dispatcher
"never resumed". Two open questions: (a) does the FMOD tail ever return to the
dispatcher 0x2bd8d18, and (b) can the real continueAfterFlagsLoaded_ ever run?

## Measured #1 — why the dispatcher never resumed (static + runtime)
The getter 0x102174c04 ends with an UNCONDITIONAL tail `b 0x624e6c0` (file 0x2174c80,
word 0x15036690) REGARDLESS of verb. The FMOD function 0x624e6c0 (`Java_org_fmod_...`)
reads `[x0+8]` (0x624e6e8). Since the vt[+0x30] write-leaf writes the manager M into the
dispatcher's out-field [sp+16] in BOTH verbal arms (a0=M -> a1=[sp+16]), `[x0+8]`
resolves to M (non-NULL), so `cbz` (0x624e6ec) is NOT taken and control falls into the
FMOD audio body (moving vt[+0x720]=0 -> NULL blr, or the run-variable FMOD crash). The
tail never returns to the dispatcher. `bl 2174c04` is also unfused-inlined (it was a
dispatch), so region-watch on 0x2bd8d18 (correctly block-entry-blind) read 0.

## THE LEVER (tail `b` -> `ret`)
The getter restores x30 = dispatcher return (0x2bd8d18) at 0x2174c7c BEFORE the tail, so
patching the getter's `b 0x624e6c0` (0x102174c80) to `ret` returns straight to the
dispatcher. `routeb_patch_getter_fmod_tail_ret()` (opt-in JIT_ROUTEB_GETTER_TAIL_RET,
writes the mapped .text word b->ret, drops the block cache, idempotent).

## Measured #2 — dispatcher resumes and the REAL continuation runs (+1 gate cleared)
A/B on the real binary (canonical completing --v2boot ladder, EXIT 124/139, 0 crash):
- OFF (default): getter dives into FMOD; 0x2bd8d18/0x2bd8dac/continueAfterFlagsLoaded_
  all 0 hits (run-variable FMOD crash ~1/3, guestpc 0x106240b44 lr 0x1062514e4).
- ON (GETTER_TAIL_RET): **0x2bd8d18 -> 0x2bd8d30 -> 0x2bd8d54 -> sub_2bd8dac -> the
  CONTINUED dispatcher reaches `bl sub_2bd8dac` -> vt[+0x1f0] -> REAL continueAfterFlagsLoaded_
  0x102bd1d68 FIRES (region-watch) — the FIRST headless execution of the real continuation.**
- ON + M+0x48 seed: continuation executes its FULL body (serializes flags, calls
  nativeAppBridgeAppStart 0x2338ef4) and reaches 0x102bd1dfc (past the app-name guard),
  then dies differently (see #3).

## Measured #3 — the next structural gate: bad_alloc in the 0x28 closure alloc
With the M+0x48 app-name seed, continuation ran `0x2bd1d68 -> 0x2bd1dfc`, serializing
flags + calling nativeAppBridgeAppStart, then entered operator-new 0x1db1a38
(block-entries 0x1db1a38..0x1db1b78) which THREW `std::bad_alloc` (libc++abi terminate).
Root cause (disasm 0x1db1a68-0x1db1a80): operator-new's fast path returns NULL for
size>0xa when the global allocator-activation byte [0x10727570c].bit0 == 0 (headlessly
clear); a NULL operator-new return -> the new-handler throws std::bad_alloc. The 0x28
closure (>0xa) therefore bad_allocs.

## Measured #4 — the broad alloc-flag seed is a REGRESSION (do-not-re-tread)
Seeding [0x10727570c].bit0=1 routes ALL operator-new calls (many callers, e.g.
0x1db19fc/0x1e44d0c/0x21e3114/...) into the REAL allocator path (1db1a84 canary + tail),
which fails headlessly: 3/3 on.full runs abort early (guestpc=0x0 SIGABRT, secondary
thread) BEFORE reaching the continuation. The real-alloc path is NOT headless-producible.
Reverted the seed (removed routeb_seed_dm_alloc_flag). Next scoped lever = patch only the
size==0x28 flag-clear case to return a stable leaked box (unimplemented: operator-new is
shared; 2-slot window can't hold cmp+materialize+branch), OR seed a real post-app-start
controller F+0x18 so the continuation's 0x2bd2080 deref (`*(*(F+0x18)+16)`) is valid.

## Honest (do-not-over-claim)
Opens a genuinely NEW engine-init execution path and makes the REAL continueAfterFlagsLoaded_
execute its full body (incl. nativeAppBridgeAppStart) headlessly — a first. Does NOT
manufacture a DataModel; the continuation is still fed a fabricated flags-holder and now
stops one gate deeper (bad_alloc at the 0x28 app-controller closure). Route-B live-DM
structural gate is NOT lifted, but it MOVED forward: the DMCONT continuation is now
WIRED+ARMED+ACTIVE (was "latent"), backed by a measured runtime path.

## Code / files
- crates/arm64jit/examples/elfjit.rs: `routeb_patch_getter_fmod_tail_ret()` (+
  `sh245_getter_tail_words` pure helper). Hooks into the --v2boot block, opt-in
  JIT_ROUTEB_GETTER_TAIL_RET.
- crates/arm64jit/src/jit.rs: `routeb_dm_manager_cont` gains an opt-in
  JIT_ROUTEB_DM_CONT_M48_SEED block seeding M+0x48 as a long "Home" string.
- +2 hermetic (sh245_getter_fmod_tail_word_roundtrip_and_real_site_guard;
  sh245_m48_long_string_seed_passes_continuation_appname_guard).
- Repro runs/capture_sh245_getter_tail_ab.sh (off|on|on.m2|on.m48|on.m2m48).
- cargo build --workspace EXIT 0; cargo test -p arm64jit --lib + --examples sh245 pass.

## Next (honest, single-agent)
The bad_alloc at the 0x28 closure (0x2bd2128 -> operator-new 0x1db1a38, NULL-return for
size>0xa when [0x10727570c].bit0 is clear) is the concrete next gate. Candidate levers,
in preference: (1) scoped materialize a stable leaked 0x28 box for exactly the
continuation's closure (needs a 3-slot code patch; unimplemented, operator-new shared);
(2) seed a valid post-app-start controller into F+0x18 so the continuation's
0x2bd2080 (`ldr x8,[x8,#24]; ldr x21,[x8,#16]`) deref is valid instead of null. Either
continues the continuation; after that the standing Route-B live-DM wall remains.