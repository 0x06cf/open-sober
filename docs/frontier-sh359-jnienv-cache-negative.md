# Frontier SH359 — measured negative on the SH358 NULL-JNIEnv lane's root cause: the JNI_OnLoad cached-JavaVM cell [0x107275550] is NOT the causal seed (GetEnv traces 11x with AND without it on pure boot; the 0x1021e1c00 fault does not reproduce without the full ladder); re-confirmed both recon-v3 deliverables green at this HEAD

## Session
Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). One descriptive
hermetic test module (+2 tests) + one probe script; production elfjit.rs/jit.rs
UNCHANGED from HEAD SH358 (a seed was tried and REVERTED after the control refuted
it — honest measured negative). Workspace green (cargo test --workspace exit 0,
all suites pass, 0 failures).

## Why this cycle (a genuine cause-level attempt on the freshest datum, then measured negative)
SH358's one new datum was a NULL-JNIEnv fault: rung-1 nativeInitializeNativeFlags
detours into the JNIEnv-slot dispatch helper 0x1021e1c00 with x0=0 (slot 31),
declared there "not seedable (missing JNIEnv is a lifecycle precondition)". This
cycle disassembled the boot-side env-establishment chain to test whether the root
is instead a SEEDABLE cell. Found: JNI_OnLoad+0xc10 (file 0x2174c04) does
`adrp x8,7275000; add x8,#0x550; ldar x0,[x8]` (reads guest [0x107275550]); `cbz
x0,+0x80` (if 0, RET with env NULL); else dispatches `vm->GetEnv` (slot 6, 0x10006).
Hypothesis: seeding [0x107275550]=fabricated vm lets GetEnv succeed and JNI_OnLoad
acquire a real env.

## Measured (real libroblox.so, boot 0x2173ff4 --jni, capture_sh359_jnienv_cache.sh + control)
- WITH seed (JIT_ROUTEB_JNIENV_CACHE=1): seed fires, 11 VM_GetEnv traces, 0 fault, 0 crash, EXIT 0.
- CONTROL (no seed): **11 VM_GetEnv traces too**, 0 fault, 0 crash, EXIT 0.
- The pure --jni boot path never exercises the SH358 fault site at all — GetEnv runs
  identical 11x with/without the seed, so [0x107275550] is NOT what gates env
  acquisition on this path. The 0x1021e1c00 NULL-env fault requires the FULL ladder
  (SH358's rung env), which is a different, deeper env-loss path, not this boot cell.

## Conclusion (do-not-over-claim)
The NULL-JNIEnv lane is NOT root-caused by an empty [0x107275550]; seeding that cell
is non-causal on boot. FURTHER: [0x107275550] is the SH243 DM-manager getter cell —
a production seed there would clobber the validated DM-force path. Do NOT re-attempt
a seed of this cell for the NULL-env lane. This closes the one concrete "cell to
seed" candidate SH358's datum surfaced; the NULL JNIEnv in the full ladder remains
a cause-level lifecycle precondition (SH358's conclusion stands, now with the
boot/cell hypothesis eliminated).

## Honest
Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED (DM-root
[0x106a68818]=0, MH_* false). recon-v3 immediate-priority deliverables re-verified
green at this HEAD: capture_taskv4_frame.sh = 24 real task-driven frames
(`present swap Ok(0x1)`), dispatch #2718000, 197 node pops, 0 json abort, 0
SIGSEGV/ABRT, EXIT 0/124; JIT_JSON_ZERO_FIX present (jit.rs:6390). SH174
capture-latch stays the single forward observer.

## Verify / files
- `cargo test --workspace` exit 0 (all suites pass).
- `cargo test --example elfjit sh359` = 2/2 pass.
- Probe: runs/capture_sh359_jnienv_cache.sh (new). Live capture gitignored.
- Hermetic: sh359_jnienv_cache_tests (2 tests, descriptive addressing pins only).
- No production path edited; elfjit.rs/jit.rs product paths identical to HEAD SH358.