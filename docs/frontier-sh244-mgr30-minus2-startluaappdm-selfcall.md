# SH244 — the engine-init getter's vt[+0x30] verb selects StartLuaAppDM self-call

Date: Sep 17, 2026, hermes-worker. Workspace green (arm64jit lib 392/0 incl. new sh244,
examples 85/0, recon-v3 plane re-verified earlier this cycle). Single-agent (cone suppressed).

## Question
SH243 corrected the manager getter cell (0x107275550) and measured that the fabricated
NativeDataModelManager M is carried down, but the engine-init dispatcher 0x102bd8ce8
"never resumes past the getter" — interior pcs (0x2bd8d18), sub_2bd8dac, and
continueAfterFlagsLoaded_ (0x102bd1d68) all persistent 0 hits. What exactly does the
getter 0x102174c04 do after it dispatches M.vt[+0x30], and is there an un-driven branch?

## Measured: the getter's vt[+0x30] return word gates a StartLuaAppDM SELF-call
Fresh disasm of the getter 0x102174c04 (= JNI_OnLoad+0xc10), the function the dispatcher
calls via `bl 2174c04` @ 0x2bd8d14:

- 0x2174c20: reads the manager cell [0x107275550] via ldar; if NULL -> ret (benign).
- non-NULL: 0x2174c40 ldr vt[+0x30] (slot 6); 0x2174c44 `blr x8` -> dispatches M.vt+0x30.
- 0x2174c48: `cmn w0,#0x2` (0x2 + w0 == 0, i.e. w0 == -2 = 0xFFFFFFFE).
- 0x2174c4c: `b.ne 0x2174c6c` — if w0 != -2, SKIP the next call, fall to the tail.
- 0x2174c50: `bl 0x10242a5e4` — w0 == -2 -> `nativeAppBridgeStartLuaAppDM` body (0x3a7b8
  into the function whose entry is 0x1023efe2c), a SELF-invocation from inside engine-init.
- 0x2174c80: `b 0x624e6c0` — in BOTH cases, tail-seqs into the FMOD/AAudio region.

Our fabricated manager's vt[+0x30] is a write-leaf returning 0, so the getter ALWAYS took
`b.ne` and never executed its own StartLuaAppDM call; it went straight into the FMOD tail
— which never returns to the dispatcher (0x102bd8d18 stays 0). That tail divergence is the
SH243 "never resumes past the getter" mechanism.

## The A/B (real binary, 1 completing run each, EXIT 124, 0 crash)
Same canonical --v2boot ladder (DMFORCE+DMCONT+DM_SEED+HASHFIX+JSONZERO+SETFIX+SH115).
Region-watch = getter / SLADM-body / FMOD-tail / dispatcher resume / sub / continueAfterFlagsLoaded_.

- BASELINE (vt+0x30 returns 0): getter hit at 0x102174c04+0x102174c48; **SLADM body
  0x10242a5e4 = 0 hits**; FMOD tail 0x10624e6c0 fired. Dispatcher resume 0x2bd8d18 = 0,
  sub/continueAfterFlagsLoaded_ = 0. StartLuaAppDM returned Ok(M) (ladder-driven path).
- FORWARD (JIT_ROUTEB_DM_MGR_MINUS2=1, vt+0x30 returns -2): **SLADM body 0x10242a5e4
  ENTERED (1 hit) + 0x10242a5f8 entered** — engine-init self-executes deeper into the
  `nativeAppBridgeStartLuaAppDM` function body, a region with 0 hits in baseline; FMOD tail
  still reached; dispatcher resume 0x2bd8d18 still 0. StartLuaAppDM returned Ok(M). EXIT 124.

## What this does and does NOT do (honest)
- **DOES:** opens a genuinely-new execution path that was headless-unreachable for the whole
  SH165-243 era. The -2 verb (JIT_ROUTEB_DM_MGR_MINUS2, default-inert) makes the engine-init
  getter self-invoke into the StartLuaAppDM function body (0x10242a5e4/0x10242a5f8) instead of
  b.ne-skipping every time. End-to-end stable (EXIT 124, 0 crash, both arms).
- **DOES NOT (yet):** a live DataModel. The FMOD tail still consumes control after the
  StartLuaAppDM self-call (0x10624e6c0 fired both arms), so the dispatcher still does not
  resume at 0x2bd8d18 and continueAfterFlagsLoaded_ (0x102bd1d68) stays 0. The lever makes
  the engine-init execute StartLuaAppDM's body itself — necessary-forward-motion evidence,
  not a manufactured DM. Route-B live-DM structural gate UNCHANGED.

## Code / files
- crates/arm64jit/src/jit.rs: new `routeb_dm_manager_write_leaf_minus2` (writes a0 into a1
  out-field, returns 0xFFFFFFFE) + `routeb_manager_mgr30_leaf()` (env JIT_ROUTEB_DM_MGR_MINUS2
  selects -2 vs default 0 at vt[+0x30]) wired into both `routeb_dm_manager_fabricated` and
  `routeb_dm_manager_cont`. Default path unchanged (env off -> write-leaf 0, the verified
  baseline). + hermetic sh244 (verbs return 0 / -2 and both write the out-field; env gate).
- Repro runs/capture_sh244_ab.sh (A/B off|on).
- cargo build --workspace EXIT 0; cargo test -p arm64jit --lib sh244 + sh243 pass.

## Next (honest, single-agent)
The FMOD tail 0x624e6c0 is now the concrete next gate: with the StartLuaAppDM self-call
ENTERED, does the tail's return path ever come back? If the tail is a genuine audio-init that
can return, the dispatcher could resume -> continueAfterFlagsLoaded_. The tail calls
vt[+0x30] of [x0+8] (the out-field M) which is a leaf — so the tail's early-return path
(0x624e708: cbz [x0+8] -> canary-check -> ret) is the optimistic exit. Whether it returns to
0x2bd8d18 is the next measurement. Route-B live-DM = standing structural gate.