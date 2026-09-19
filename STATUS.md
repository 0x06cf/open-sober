# Open-Sober run status (hermes-worker)

Updated this cycle: SH426 — hermetic coverage of the x86-64 code emitter backend
(x86.rs). The final codegen surface every translated block emits bytes through
(CodeBuf + rex/modrm/disp_mod ModRM resolution + the mov* family + patch_rel32
for internal control-flow) had ZERO tests; SH426 pins all of it byte-exactly
with 12 deterministic hermetics (REX.W/R/X/B bit layout, ModRM field placement,
disp8-vs-disp32 selection, REX insertion for r8-r15, mov imm64/imm32/rr64/load/
store/eax32, cqo/cdq, rel32 displacement arithmetic forward & backward).
recon-v3 deliverables re-verified green at this HEAD (capture_taskv4_frame.sh
attempt 1: 24 real task-driven frames, 196 node pops, 0 json abort, 0 crash).
Workspace green (cargo test --workspace EXIT 0; arm64jit lib 502/0 incl. 12 new
sh426 hermetics; cargo build --workspace OK). Production code only in x86.rs
(off-hook; jit.rs/elfjit.rs byte-unchanged).

## Current state

- `dev` HEAD: SH426. recon-v3 immediate-priority deliverables green at HEAD
  (capture_taskv4_frame.sh: 24 real task-driven frames, 196 node pops, 0 json
  abort, 0 crash). Workspace green (arm64jit lib 502/0; cargo build --workspace
  OK). jit.rs 1,048,417 B < 1MiB hook; elfjit.rs untouched (at/near the hook).
- x86-64 emitter backend now hermetically pinned (was ZERO tests): 12 new tests
  cover REX/modrm/disp_mod encoding, the mov* family (imm64/imm32/rr64/load/
  store/eax32 + REX insertion for r8-r15), cqo/cdq, and patch_rel32.
- Live-DM gate UNCHANGED: the SH415 do-init probe under the COMPLETE substrate
  (11/16 Ok, MH_FLAGS_LOADED/ENGINE_INITIALIZED/APP_READY all latch true,
  AppBridgeV2 genuine vt 0x1063a3410) still reports DM-root [0x106a68818]=0x0,
  vt=0x0, app-DM-counter=0 -> LIVE DM = false. Runtime surface complete +
  exercised; a live DM needs a real session ctor the JIT cannot reproduce
  headlessly.

## This cycle's advance

- SH426: BUILD-THE-RUNTIME codegen-surface coverage completion on x86.rs (the
  x86-64 emitter every translated block encodes through) — previously exercised
  nowhere of its own. 12 deterministic hermetics in `x86::tests`; no production
  path / guest byte / JIT-hook-default changed.

## Honest status

- No DM (DM-root [0x106a68818]=0, no make_shared, MH_GAME_LOADED false) —
  Route-B live-DM structural gate UNCHANGED. Content (fsmap remap, R1
  CoreScript stage both-roots, G3 files-dir), lifecycle (MH_*), input
  (ainput bridge + X11 loop), audio (SH132 AAudio), boot-stack, signal core,
  and now the x86 emitter are all latent-but-correct, firing the instant a live
  DM owns a session.

## Next-forward candidates

1. (standing, TOP — Route B) do-init completeness / live-DM. SUBSTRATE REPORTS the
   markers; marker non-live until the engine's own session ctor builds the DM world.
2. GENUINE canary wall NAMED (SH420): a `__stack_chk_fail` host shim pins the real
   wall to `__guest_pc=0x102206d90` = nativeGameGlobalInit body, canary slot
   [x29,#-8] ZEROED during the do-init once-path. CLOSED as separate-seedable-wall:
   disassembly of the four between-store/compare callees shows small frames writing
   own-locals only — the zeroing is a SYMPTOM of the deep do-init once-lambda
   dispatch draining into the SH285/LSM lane (the Route-B once-path itself). Do NOT
   spend a narrow store-watch on it (re-tread).
3. DMCONT 0x102bd1d68 = 0 from the MAIN arm (unchanged standing gate).
4. Do-not-re-tread unchanged: LSM skips/rebuilds, setDataModelToCurrent, EC
   reader, window-attach real, ALooper, governor gates.
5. Do NOT run the SH174 latch without JIT_DM_ALLOC_CAPTURE_DELEGATE=1.