# Frontier SH358 — measured negative (run-variable): the DMCONT-session-ctor + LSM-skip combination is NOT a Route-B forward (arming SH349/350 skips with the session drive parks the run in the persistence/live-object lane BEFORE the continuation region — never reaches it; one fault lane is a fresh 0x1021e1c00 JNIEnv-slot helper on a NULL JNIEnv)

## Session
Sep 19/20, 2026, hermes-worker. Single-agent (cone suppressed). Measurement-only
(one new probe script; no production code edited). Workspace green at HEAD SH357
(cargo test --workspace exit 0, 24 test binaries, 0 failures; arm64jit lib 421/0).
recon-v3 immediate-priority deliverables re-verified present AND green at this HEAD:
`capture_taskv4_frame.sh` — 24 real task-driven frames (`present #N swap Ok(0x1)`),
dispatch #2797000, 197 node pops, 0 json abort, 0 SIGSEGV/ABRT, EXIT 0.
`JIT_JSON_ZERO_FIX` clamped len at 0x102355d40 (jit.rs:6390).

## Why this cycle (a genuine unfired combination, then measured negative)
SH344c (authoritative, measured 3/3) pinned that the DMCONT session-ctor continuation
(vt[+0x1f0]=REAL continueAfterFlagsLoaded_ 0x102bd1d68) dies at the SH285 LSM
reader/pop terminal **0x101db1b08** (fault 0xffffffffffffffff) one hop BEFORE the
post-app-start tail `bl 0x2bd2058`. But SH344c shipped BEFORE the two later LSM-skip
patches that SH349/350 proved clear EXACTLY that class of terminal: SH349
(JIT_ROUTEB_LSM_APPEND_SKIP=1, RET the faulty byte-copy sub-call 0x101d9a15c the SH285
fault lives inside) measured "OLD terminal GONE: no more SIGSEGV guestpc=0x101db1b08",
advancing to 0x101d9a708; SH350 (JIT_ROUTEB_LSM_PACK_SKIP=1, RET the single-caller
name-pack 0x101d9a708) crossed that to 175 LSM pool-pops. Arming both skips TOGETHER
with the DMCONT continuation + session drive was never-run. That is the genuinely-new
combination this cycle tested.

## Measured (real libroblox.so, runs/capture_sh358_dmcont_lsmskip.sh, 3/3)
- SH350 pack-skip FIRES (`ret name-pack @0x101d9a708 d10143ff->d65f03c0`) — the patch
  arms cleanly.
- The run is **run-variable** across the known persistence/live-object lane (SH353
  class, not a single deterministic fault). Across 3 runs it never reaches the DMCONT
  continuation region (0 continuation/tail hits in all 3). Fault lanes observed:
  1. rarest: rung-1 `nativeInitializeNativeFlags` detours into a **freshly-documented
     JNIEnv-slot dispatch helper at 0x1021e1c00** (JNI_OnLoad+0x6dc0c, entry
     `sub sp,#0x70`; sibling of the SH186 jstring->RBX helper 0x21e1fec, here
     dispatching `ldr x8,[x0]; ldr x8,[x8,#248]; blr x8` = JNIEnv vtbl slot 31) with
     x0 = **0 (NULL JNIEnv)** -> fault=0x0, EXIT 134.
  2. common (2/3): the ladder advances through nativeGameGlobalInit -> UpdateAdapterInit
     -> setTaskSchedulerBM -> app-start-driven StartLuaAppDM, then faults at the
     **SH341/SH343 LSM pool-pop write-site 0x101d9a528** (fault=0x0 NULL-write, the
     SH285-family live-object lane) before the continuation region.
- Both lanes are cause-not-symptom: the NULL JNIEnv and the LSM NULL-write are
  lifecycle/live-object preconditions, NOT slot values these skips repair. The skips
  do NOT unlock the DMCONT continuation toward `bl 0x2bd2058` on the session drive.

## Conclusion (do-not-over-claim)
The DMCONT + LSM-skip cross is a MEASURED NEGATIVE: the skips do not unlock the
continuation toward `bl 0x2bd2058`; instead their earlier rung perturbation strands the
run in a NULL-JNIEnv JNI-helper before the continuation region. This is consistent with
the whole Route-B line: the SH285 live-object wall and the upstream session-ctor gap are
cause-not-symptom; no combination of RET-skips manufactures a live DataModel. The
continuation advance SH344c named is NOT gated by the LSM reader terminal in a way these
skips cross on the session drive. Do NOT re-tread this exact env combination.

## Honest
Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED (DM-root
[0x106a68818]=0, MH_* false). SH174 capture-latch stays the single forward observer.
The new single datum is pc 0x1021e1c00 (a real JNIEnv-slot dispatch helper in the
NativeFlags path) — noted for the session-ctor lever but NOT seedable (missing JNIEnv is
a lifecycle precondition, not a slot value).

## Verify / files
- `cargo test --workspace` EXIT 0 at HEAD (no code edited this cycle).
- recon-v3 self-driven frame re-verified green: 24 frame capture above.
- Probe: runs/capture_sh358_dmcont_lsmskip.sh (new). Live capture gitignored
  (/home/hermes-worker/runs/sh358-dmcont-lsmskip.txt).
- No production path edited; elfjit.rs/jit.rs unchanged from HEAD SH357.