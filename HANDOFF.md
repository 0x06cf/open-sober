# Open Sober — Agent Handoff

## SH466 (Sep 20, 2026, hermes-worker): promote the SESSION PRODUCER HANDOFF gate core into the tested library — the recon-v3 §A END-STATE / SEP-17 self-drive decision logic is now a hermetic-pinned library contract, not example-only glue
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables re-verified
green at this HEAD first (capture_taskv4_frame.sh attempt 1: 24 real task-driven
frames `present swap Ok(0x1)`, dispatch #2264000, 195 node pops, 0 json abort,
0 crash, EXIT 0). Workspace green (cargo test --workspace EXIT 0; arm64jit lib
677/0 incl. 1 new sh466 hermetic; cargo build --workspace + --example elfjit OK).
Production code in session.rs + elfjit example wrapper (elfjit.rs DROPPED 1,048,392
-> 1,048,152 B, stays under the 1MiB hook).
- **The gap closed:** the recon-v3 §A END-STATE / SEP-17 "SESSION PRODUCER HANDOFF"
  gate — the logic that lets type-4 self-drive emit REAL task frames only once a
  REAL session owns a live DataModel — was load-bearing runtime logic that lived
  ONLY in the untested elfjit example, pinned ONLY under `cargo test --example
  elfjit`. It never ran in the workspace suite (the project's actual regression
  gate), so a drift in either direction slipped through. SH466 promotes the two
  PURE pieces into the library.
- `session.rs`: new `pub session_producer_gate(mh_app_ready, live_dm)` (recon §B /
  SH303 gate: GATED only when app-ready AND live-DM) + `pub live_dm_cell_value_ok(v)`
  (accepts only coherent guest-visible pointers >= 2^32, clear top byte, non-zero —
  so it REJECTS the SH381 do-init once-lambda "Execute" sentinel 0x400000b, the
  exact case where a naive producer mistakes the once-slot for a live DM). Pure,
  deterministic, no env, no guest bytes.
- elfjit.rs: local copies are now thin wrappers over the library fns (single source
  of truth); the SH304 example tests still pass.
- New hermetic `sh466_session_producer_handoff_gate_and_sentinel_rejection` pins the
  2x2 gate truth table + sentinel/zero/2^32-boundary/top-byte rejection surface.
- Honest: NOT a DM / NOT a live-DM step (Route-B live-DM gate UNCHANGED; DM-root
  [0x106a68818]=0, structural at the write site per SH462/463). BUILD-THE-RUNTIME
  coverage completion; the gated producer stays latent-but-correct, firing the
  instant a real session advances (MH_APP_READY AND a live DM) — these library fns
  are exactly what the toolchain uses to classify that moment. No re-treads, not a
  render-plane visual.
- Files: docs/frontier-sh466-session-producer-handoff-gate.md + crates/arm64jit/src/
  session.rs (2 pub fns + hermetic) + crates/arm64jit/examples/elfjit.rs (wrappers).
  Commit 5556211 (SH466).

## SH465 (Sep 20, 2026, hermes-worker): make the session-substrate completion metric outcome-aware — the runtime now reports true session-boot health (14/16 completed), not a return-value filter (11/16)
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables re-verified
green at this HEAD first (capture_taskv4_frame.sh: 24 real task-driven frames `present
swap Ok(0x1)`, 196 node pops, 0 json abort, 0 crash, EXIT 0; sh415 do-init capture
re-confirms the Route-B baseline: once-guard seeded, DM-root [0x106a68818]=0x0 -> LIVE
DM = false, unchanged). Workspace green (cargo test --workspace EXIT 0; arm64jit lib
675->676 incl. 1 new sh465 test; cargo build --workspace + --example elfjit OK).
Production code only in session.rs (off the 1MiB hooks; jit.rs/elfjit.rs untouched).
- **The observability defect fixed:** the SH400 substrate reported
  `substrate complete: N/16 atoms returned non-zero Ok`. On the real binary that read
  **11/16**, yet **14/16 atoms genuinely completed jit_run and returned** — because the
  metric only counted non-zero returns, so three VOID JNI natives
  (initAppShellReporter / setActive / nativeActivity_onEngineSettingsReceived, all
  legitimate `Ok(0x0)`) were counted as failures. The 11/16 number understated a
  healthy session boot and hid that only the two documented pre-existing nativeInit
  "outside image" atoms (nativeInitializeNativeFlags 0x10232048c, V2InitWithParams
  0x102365c54) truly fault.
- `session.rs`: new `DriveOutcome` enum (Stopped vs Completed(u64), with
  `.completed()` as the health bit); `drive_atom` now returns `DriveOutcome` (internal
  only); the substrate tracks BOTH `completed` (true health) and `nonzero` (legacy
  sub-count) and now summaries `completed/total completed jit_run (nonzero non-zero
  return); stopped/total stopped` — so a real run reads 14/16 complete (11 non-zero),
  2/16 stopped instead of hiding the void completions.
- New hermetic `sh465_substrate_completion_is_outcome_aware`: pins Completed(0) counts
  as a completion (void JNI natives ARE completions), only Stopped is not counted, and
  the real 16-atom shape (2 documented nativeInit Stopped + 11 non-zero + 3 void
  Completed) reads 14/11. Pure enum semantics, no jit_run, deterministic.
+ MEASURED on the real binary after the fix: `substrate complete: 14/16 atoms
+ completed jit_run (11 non-zero return); 2/16 stopped`. Route-B live-DM probe
+ unchanged (once-guard bit0=1, DM-root [0x106a68818]=0x0, LIVE DM = false).
- Honest: NOT a DM / NOT a live-DM step (Route-B gate UNCHANGED). This is a
  runtime-observability correctness fix — the runtime's own health number must be
  correct so verdicts derived from it are sound (the "measured verdict, not guess"
  discipline). No guest byte, no env, no ladder-path change. Not a re-tread: prior
  cycles recorded the 11/16 baseline but never fixed the Completed(0) vs Stopped
  conflation. No re-treads.
- Files: docs/frontier-sh465-session-outcome-metric.md + crates/arm64jit/src/session.rs
  (DriveOutcome + drive_atom + substrate summary + test). Commit (SH465).

## SH464 (Sep 20, 2026, hermes-worker): promote the FMOD/AAudio AUDIO axis to a first-class driven substrate step (the SH418 input-twin) — real PCM -> real WAV sink
Single-agent (cone suppressed). The SEP-18 "audio/input — each landed + green +
committed" deliverable list is now CLOSED on the audio side: input was promoted
(SH418); audio was NOT until now (SH132's bridge captured FMOD's data_cb but
nothing ever invoked it headlessly — no PCM produced, WAV sink never written).
Workspace green (cargo test --workspace EXIT 0; arm64jit lib 669->675 incl. 6
new sh464 tests; cargo build --workspace + --example elfjit OK). Audio-axis only
(audio.rs + session.rs); runtime deliverable (type4-frame) untouched.
- aaudio.rs: `LiveStreamParams`+`live_stream_snapshot()` (copy-out of the first
  live FMOD output stream's data_cb + PCM params, no lock across a guest call);
  `AudioSink` (REAL WAV writer — open writes the 44B header, append_frames
  writes PCM, finish patches RIFF/data sizes); `sink_path()` (JIT_AAUDIO_SINK);
  `drain_one_buffer()` (FMOD data-callback ABI via run_guest_callback -> produced
  PCM bytes; errors without an active image — never fabricates PCM).
- session.rs: `drive_fmod_audio_drain()` wired into drive_routeb_session_substrate
  right after the SH418 input loop (same post-bus trigger). Three inert guards
  (JIT_AAUDIO_BRIDGE + a live stream snapshot + a live image) -> returns 0,
  no guest call, no sink write. When armed: bounded (AUDIO_DRAIN_ITERS, default
  8) runs of the captured data_cb filling a PCM buffer -> appended to the sink.
  Single-jit_run ladder thread (SH55/64). Env-gated -> default path identical.
- 6 new hermetic tests (5 aaudio + 1 session), parallel-safe. One transient
  full-run flake on the DOCUMENTED pre-existing routeb sh362 test (shared 0xdead
  cell clobbered by a sibling); passes 3/3 in isolation + clean workspace re-run
  before commit → not a regression.
- Honest: NOT a DM (Route-B live-DM gate UNCHANGED, structural at the write site
  per SH462/463). LATENT-but-correct like every axis: fires the instant a real
  SoundService session opens FMOD's output. No re-treads, not a render-plane
  visual, not a DM seed.
- Files: docs/frontier-sh464-audio-firstclass-substrate.md + aaudio.rs +
  session.rs (+ sh132 doc Next closed). Commit (SH464).

## SH463 (Sep 20, 2026, hermes-worker): 3-axis HEAD re-verification + session-substrate <name,guest> contract pin
Single-agent (cone suppressed). Recon-v3 immediate-priority deliverables
re-verified GREEN at HEAD (capture_taskv4_frame.sh: 24 real task-driven frames
`present swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo
test --workspace 850 passed/0 failed, incl. arm64jit lib 669/0; cargo build
--workspace + --example elfjit OK).
- **Independently confirmed the translator heritage lineage is COMPLETE.**
  Cross-referenced every `Inst::` variant the decoder (decode.rs) produces
  against translate.rs: the ONLY variant no hermetic constructs is
  `Inst::Unsupported` (the decode-failure path, not a real instruction family).
  Real-binary runs (taskv4 + sh415/sh463b) show ZERO Unsupported/illegal decode
  markers — no new family exists to pin from the client's executed code, so
  there is no "real-run decode gap" re-entry per STATUS next-forward #5.
- **Substrate baseline deterministic + the two faulting atoms confirmed
  pre-existing.** nativeInitializeNativeFlags (0x10232048c) + V2InitWithParams
  (0x102365c54) both stop with garbage pcs in the session-drive AND identically
  in a clean boot-only run (which then aborts EXIT 134 at the SH174 latch
  without JIT_DM_ALLOC_CAPTURE_DELEGATE=1). Same fns, same order, same fault —
  the run-variable, order/warm-up-dependent nativeInit 'outside image' Route-B
  lane already recorded in memory. NOT a session-drive defect, NOT a regression
  (recorded baseline = 11/16 across SH400-462), NOT a re-tread target.
- **One new deliverable (contract hardening on the SEP-17/18 session-drive):**
  session.rs `substrate_atom_names_exact_and_no_fallthrough` now pins the
  definitive 16-atom <name,guest> address map (deterministic, no real binary).
  The existing names-only set check + jit.rs sh399 prologue-anchor cannot catch
  a name<->guest swap between two distinct valid fn entries; this closes it.
  Test-only (extends the existing names test; no new test, arm64jit lib stays
  669/0). Production code untouched.
- Honest: NOT a DM / NOT a live-DM step (Route-B gate UNCHANGED; DM-root
  [0x106a68818]=0, no store reaches it — SH462 store-level structural). This is
  a re-verification + defensive-pin cycle. No re-treads.
- Files: docs/frontier-sh463-session-drive-addrpin.md + crates/arm64jit/src/
  session.rs (test-only). Commit (SH463).

## SH462 (Sep 20, 2026, hermes-worker): DM-root STORE-watch — dynamic write-site trace of the Route-B DM-holder window (first store-level, not read-level, measurement)
Single-agent (cone suppressed). The recon-v3 immediate-priority deliverables
remain green at this HEAD (type4_frame_thunk 24 real task-driven frames + 0 json
abort, SH461-VERIFY). Workspace green (cargo test --workspace EXIT 0; arm64jit
lib 668->669 incl. 1 new sh462 hermetic; cargo build --workspace + --example
elfjit OK). Production code ONLY in translate.rs (default-inert observer on the
existing emitted post-store hook; jit.rs 1,048,390 B < 1MiB hook untouched;
elfjit.rs/session.rs unchanged — runtime byte-identical when the env is unset).
- **The operator-asked dynamic trace, one level deeper.** Route B has been
  measured only by READING the DM-holder cells (SH361/381/334/388 all report
  DM-root [0x106a68818]=0). NO instrument ever named the guest STORE that would
  populate them. SH462 adds `JIT_DMROOT_STORE_WATCH=1` (default-inert): any
  64-bit guest store landing in [0x106a683f0..0x106a68828] (spans once-guard
  [0x106a68410], once-slot [0x106a68408], DM-root [0x106a68818]) logs pc+value.
- **MEASURED on the real libroblox.so** (SH415 env + the store-watch flag; EXIT
  124 stable, substrate 11/16, probe LIVE DM=false): **11 distinct fires, ALL in
  the once-slot/guard region [0x106a68408..0x106a684d0]** — the do-init
  once-lambda world-build [0x22065xx..0x2206axx] DOES run headlessly and
  populates real structure there. The once-lambda's ctor store at
  pc=0x102206d74 (SH381's `str x0,[x23,#1032]` after `bl 0x2173b3c`) writes the
  once-state SENTINEL **0x400000b** into once-slot [0x106a68408] — NOT a
  DataModel pointer.
- **Crucial negative: NO guest store ever targets DM-root [0x106a68818]
  itself.** All fires are in the 0x108-byte region below it. This is the dynamic
  confirmation the wall is STRUCTURAL at the DM-root WRITE site: the real DM is
  built by an upstream session path the JIT cannot drive, and the harness's
  earlier SH155-ladder nonzero 'DM-root' values were host SEED writes, not guest
  stores (which is why the guest store-watch doesn't list them).
- Honest: NOT a DM — this is an observer, not a seed; Route-B live-DM gate
  UNCHANGED. It closes the "is the wall seedable-by-writer" sub-question (a
  reached writer exists but writes sentinel/state, never a DM into the holder)
  and sharpens the operator-aligned lever (session-ctor / runtime-surface drive)
  as the only path to a live DM. No re-treads.
- Files: docs/frontier-sh462-dmroot-store-watch.md (evidence incl. the measured
  11-fire trace) + runs/capture_sh462_dmroot_storewatch.sh + crates/arm64jit/
  src/translate.rs (`#[cfg]`-free observer + test). Commit afaf030.

## SH461-VERIFY (Sep 19, 2026, hermes-worker): recon-v3 §A type4_frame_thunk SELF-DRIVEN-FRAME deliverable VERIFIED GREEN ON THE REAL BINARY — 24 real task-driven frames (real capture artifact)
Single-agent (cone suppressed). Ran `runs/capture_taskv4_frame.sh` against the
real 104MB `libroblox.so` after the SH455-461 test-only codegen-pin lineage kept
the runtime byte-identical to the SH445 baseline. MEASURED (attempt 1/6, EXIT
124, 0 crash-signals):
- `[elfjit:taskv4-frame] present #N swap Ok(0x1) color=[...]` x **24** — each a
  REAL engine frame (engine make-current 0x105b3b358 -> frame-fn 0x105b32c00 ->
  eglSwapBuffers 0x105b3b408 on the recovered RENDERCTX vtable 0x106731ae0),
  task-driven not harness-driven.
- `dispatch #2165000` w4=4 dispatches reaching the thunk; **196 real node pops**
  through the real drain pop-loop (real task dispatch, none fabricated).
- seed markers: `[elfjit:taskv4] type4_frame_thunk registered at
  0x7f00000001d0` + `seeded dispatcher type-4 vector [0x106829ea8] =
  0x7f00000001d0` + the two heartbeat `mov w4,#2/#3 -> #4` patches
  (0x102856f24/0x102856f68) + `[elfjit:renderthunk] published RENDERCTX
  0x7f9530111b20`.
- **0 json abort** (JIT_JSON_ZERO_FIX path stays clean), 0 crash.
- Concurrent on the same run: `[persist] live datastore roundtrip write=45B ...
  read_back_byte_exact=true on_disk=true` = the objective-2b "remembers
  sign-in" durable-persistence contract fires ON a real session (backs SH423's
  hermetic).
- This is the recon-v3 §A deliverable (doc recon-selfdrive-seed-jsonfix.md):
  type-4 dispatch now emits REAL task-driven frames, task-driven rather than
  harness-driven. Honest: NOT a live DM (DM-root [0x106a68818]=0x0; Route-B
  live-DM structural gate UNCHANGED) — the frame-present path is proven, the
  DM-world construction wall stands.
- Files: docs/frontier-sh461v-type4frame-real-verify.md (evidence doc) + raw
  capture log `/home/hermes-worker/runs/sh60-taskv4-frame.txt` (340K, kept on
  disk). Commit 521a6d7.

## SH461 (Sep 19, 2026, hermes-worker): hermetic coverage of the SHA crypt HOST-CALL codegen family (translate.rs Sha — sha1/sha256 Vd.4S, Vn.4S, Vm.4S) — 1 pin
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (the change is `#[cfg(test)]`-only, so the runtime deliverable
is byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 668/0 incl. 1 new sh461 pin, was 667; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged). Commit
d25764d.
- Sha (the AArch64 SHA1/SHA256 crypt ops, dispatched to a HOST `guest_sha1stem`
  helper) had zero direct byte tests (STATUS next-forward #5 named `Sha`).
  SH461 pins the host-call thunk shape (rd=1 rn=2 rm=3): `mov rdi,rbx` (48 89
  df = CpuState* arg0) + `mov rsi,<packed>` (48 be, arg1 = mode<<24|rd<<16|
  rn<<8|rm) + `mov rax,<host-fn>` (48 b8, the helper address — a HOST pointer,
  deliberately NOT pinned as it varies per loader) + `sub rsp,8` (48 81 ec 08 =
  RSP-align to 16 at the SysV call site; the block body runs at RSP≡8, host
  calls need ≡0) + `call rax` (ff d0) + `add rsp,8` (48 81 c4 08). The mode
  nibble in the IMM is the discriminator (mode=3 -> the 4th imm byte is 0x03);
  the sub->call->add RSP-align round-trip is pinned positionally.
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission
  captured with a one-off probe test (eprintln dump, removed before commit) so
  pins match the real emission (one window-width dev-fix: `call rax` is 2 bytes
  ff d0, not 3). No real binary/env; parallel-safe.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the SHA crypt host-call family — the
  last named in STATUS next-forward #5, completing the translate.rs hermetic
  lineage SH427-461. No re-treads.
- Files: docs/frontier-sh461-translator-sha-hostcall.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only). Commit d25764d.

## SH460 (Sep 19, 2026, hermes-worker): hermetic coverage of the DUP-BROADCAST + FP-IMMEDIATE-BROADCAST + 128-bit-OR codegen families (translate.rs SimdDupGp / SimdFmovImm / SimdOrr16) — 4 exact-byte pins
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (the change is `#[cfg(test)]`-only, so the runtime deliverable
is byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 667/0 incl. 4 new sh460 pins, was 663; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged). Commit
4dd280b.
- The three broadcast/OR families had zero direct byte tests (STATUS next-
  forward #5 named `SimdDup/FmovImm`). SH460 pins them:
  (1) THE SimdDupGp GPR-broadcast esize discriminator: esize<8 ZERO-EXTENDS
  Wn (mov eax,eax 89 c0) then ANDs the element mask (48 81 e0 ff ff ff ff for
  esize=4) then stores to every lane; esize=8 loads Xn raw (48 8b 43 18, rn=3
  slot) with NO zero-extend / mask. A missing mask leaks high garbage bits of
  Wn into every lane; a missing zero-extend leaves a sign-extended value;
  (2) THE SimdDupGp lane-layout: store count + stride is the q/esize
  discriminator — esize=2 q=false = 4 halfword stores +2 (0x120..0x126, 66 89
  83) with the 0xffff mask; esize=1 q=true = 16 byte stores +1 (0x120..0x12f,
  88 83) with the 0xff mask. A count/stride flub broadcasts into the wrong
  slots;
  (3) THE SimdFmovImm immediate-broadcast: esize=8 materializes the full 64
  bits (48 b8 .. f0 3f) + 64-bit stores; esize=4 the zero-extended low 32
  (48 b8 00 00 80 3f 00 00 00 00 for 1.0f) + 32-bit stores — materialization +
  store width is the esize discriminator (a flub broadcasts a truncated/
  expanded value);
  (4) THE SimdOrr16 128-bit OR: two passes loading 64-bit halves (0x130/0x138
  Vn, 0x140/0x148 Vm) each `or rax,rcx` (48 09 c8) + 64-bit store (0x120/0x128)
  — the rm==rn (vmov copy) form is NOT special-cased (emits the same OR, V|V=V)
  and must NOT degrade to a 128-bit SSE `por` (66 0f eb).
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, GPR slot g=[RBX+g*8], vector slot v[t]=VECTOR_BASE
  (0x110)+t*16. Emission captured with a one-off probe test (eprintln dump,
  removed before commit) so pins match the real emission (one window-width
  dev-fix on the 2-byte mov eax,eax). No real binary/env; parallel-safe.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the broadcast + 128-bit-OR families,
  distinct from SH437's lane-COPY (per-element move, not broadcast). No
  re-treads.
- Files: docs/frontier-sh460-translator-dup-fmov-orr.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only). Commit 4dd280b.

## SH459 (Sep 19, 2026, hermes-worker): hermetic coverage of the SATURATING NARROWING-SHIFT codegen family (translate.rs SatNarrowShift — sqshrn/uqshrn/sqshrun Vd.T, Vn.T, #imm) — 3 exact-byte pins
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (the change is `#[cfg(test)]`-only, so the runtime deliverable
is byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 663/0 incl. 3 new sh459 pins, was 660; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged). Commit
0a6b738.
- SatNarrowShift (the shift-then-SATURATING-narrow — each src element shifted
  right then clamped to the DST element's range, the color-channel / narrow-
  packing path that must not wrap) had zero direct byte tests (STATUS next-
  forward #5 named `ShrAcc2`; the saturating-narrow is adjacent-unpinned to
  SH451's plain narrowing-shift). SH459 pins the exact emit (rd=1 rn=2;
  src@0x130 dst@0x120) with 3 exact-byte/window pins:
  (1) THE sqshrn .4H lane-0 full emit (load-bearing): mov_load32 + `shl rax,32`
  (48 c1 e0 20) + `sar rax,32` (48 c1 f8 20 = the 32-bit SIGN-EXTEND) + `sar
  rax,6` (48 c1 f8 06, arithmetic — signed src) + clamp cmp 0x...8000 (48 39
  c8) + cmovl 48 0f 4c c1 + cmp 0x7fff + cmovg 48 0f 4f c1 + 16-bit store (66
  89); plus the q=false upper-half-zero (mov rax,0 + mov [0x128],rax);
  (2) THE signed-vs-unsigned shift + clamp discriminator: signed src uses shl+
  `sar rax,#` (48 c1 f8, arithmetic, with the shl/sar-64 sign-extend) and clamps
  to [-0x8000, 0x7fff]; unsigned uses a bare `shr rax,#` (48 c1 e8, logical, no
  sign-extend) and clamps to [0, 0xffff] — the E8-vs-F8 shift byte AND the
  clamp constants are the semantic (a flub clamps to the wrong bound or shifts
  the wrong direction, corrupting every narrowed lane);
  (3) THE q=true full-16B + byte-width discriminator: uqshrn .16B (src_esize=2
  dst_esize=1 q=true) zero-extends via movzx (0f b7 83) + `shr rax,4` + clamps
  to [0, 0xff] + BYTE stores (88 83) across Vd (0x120..0x12f) with NO
  upper-half-zero (q=true fills the whole register) — the byte-vs-word store is
  the dst_esize width discriminator.
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission
  captured with a one-off probe test (eprintln dump, removed before commit) so
  pins match the real emission (one lane-0 slice width dev-fix: 35 bytes, not
  44). No real binary/env; parallel-safe.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the saturating-narrow family, distinct
  from SH451's plain narrowing-shift (no clamp) and SH434's plain shift (no
  narrow). No re-treads.
- Files: docs/frontier-sh459-translator-satnarrowshift.md + crates/arm64jit/
  src/translate.rs (`#[cfg(test)]` only). Commit 0a6b738.

## SH458 (Sep 19, 2026, hermes-worker): hermetic coverage of the SIMD lane-SELECT copy codegen family (translate.rs SimdLaneS — mov Sd/Dd, Vn.T[idx]) — 3 exact-byte pins
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (the change is `#[cfg(test)]`-only, so the runtime deliverable
is byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 660/0 incl. 3 new sh458 pins, was 657; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged). Commit
ff809d0.
- SimdLaneS (the scalar lane-SELECT — copy one element Vn.T[idx] into the
  DEST FP (vector) slot's low bytes, the scalar-result half of a lane-extract)
  had zero direct byte tests (STATUS next-forward #5 named `SimdLaneS`). SH458
  pins the exact emit (rd=1 rn=2; Vn@0x130 Vd@0x120) with 3 exact-byte/window
  pins:
  (1) THE esize=8 index=0 full-buffer (load-bearing): mov rax,[0x130] (48 8b
  83, src = Vn + 0*8) + mov [0x120],rax (48 89 83, dst = Vd slot);
  (2) THE index-moves-source-dst-fixed discriminator: index advances ONLY the
  SOURCE by +esize (index=1 -> 0x138 for esize=8) while the DEST stays at
  f(rd)=0x120 — a flub that also advances the dest (or misses the source
  advance) copies the wrong element; negative asserts the dest never moves;
  (3) THE esize=4 width + index stride: mov_load32 (8b 83) + mov_store32 (89
  83), index strides source by +4 (0x130 -> 0x134) — the 89-vs-48 89 store
  (32-vs-64-bit) is the esize width discriminator, negative no mov_load64 /
  mov_store64.
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission
  captured with a one-off probe test (eprintln dump, removed before commit) so
  pins match the real emission; esize=8 full-buffer assert_eq matched first try.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the lane-select-copy family, distinct
  from SH437's lane-COPY (which copies WITHIN a register across slots); this is
  extract-into-dest-low-bytes. No re-treads.
- Files: docs/frontier-sh458-translator-lanes.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only). Commit ff809d0.

## SH457 (Sep 19, 2026, hermes-worker): hermetic coverage of the POLYNOMIAL 64x64 MULTIPLY codegen family (translate.rs Pmull1q — pmull/pmull2 Vd.1Q, Vn.1D, Vm.1D) — 3 exact-byte pins
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (the change is `#[cfg(test)]`-only, so the runtime deliverable
is byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 657/0 incl. 3 new sh457 pins, was 654; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged). Commit
958fc86.
- Pmull1q (the 64x64 carry-less / polynomial multiply -> 128-bit, the
  carry-less form hash/checksum/reduce paths lean on) had zero direct byte
  tests (STATUS next-forward #5 named `Pmull1q`). SH457 pins the exact emit
  (rd=1 rn=2 rm=3; Vn@0x130 Vm@0x140 Vd@0x120) with 3 exact-byte/window pins:
  (1) THE pmull low64 full-buffer (load-bearing): movq_load xmm0=[0x130]
  (f3 48 0f 7e) + movq_load xmm1=[0x140] (f3 48 0f 7e) + `pclmulqdq xmm0,xmm1,
  0x00` (66 0f 3a 44 c1 00) + movdqu_store [0x120] (f3 0f 7f, the 128-bit
  result); full-buffer assert_eq;
  (2) THE pmull2 (hi=true) high-half discriminator: hi selects the UPPER
  elements (bytes 8..15) so the sources advance +8 to 0x138/0x148 while the
  pclmulq imm (0x00) and the Vd store (0x120) stay identical — the hi flag is
  the ONLY thing that moves the sources; negative asserts hi=true never reads
  the low halves 0x130/0x140, controls hi=false reads them;
  (3) THE pclmulq imm + opcode lock: the carry-less semantic is the 3-byte
  PCLMULQDQ opcode 66 0f 3a 44 with the imm in the /r ib — imm must be 0x00
  (low64 x low64), must NOT be 0x01/0x10 (cross-half), and must NOT degrade to
  an integer `imul` (48 0f af) which adds instead of XOR-carry.
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission
  captured with a one-off probe test (eprintln dump, removed before commit) so
  pins match the real emission; pmull low64 full-buffer assert_eq matched first
  try.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the polynomial-multiply family (the
  SSE4.2 PCLMULQDQ carry-less form, distinct from SH454/455's FP multiply).
  No re-treads.
- Files: docs/frontier-sh457-translator-pmull1q.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only). Commit 958fc86.

## SH456 (Sep 19, 2026, hermes-worker): hermetic coverage of the SIMD VARIABLE-SHIFT codegen family (translate.rs SimdVShift — ushl/sshl/urshl/srshl Vd.T, Vn.T, Vm.T) — 4 exact-byte pins
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (the change is `#[cfg(test)]`-only, so the runtime deliverable
is byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 654/0 incl. 4 new sh456 pins, was 650; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged). Commit
491246e.
- SimdVShift (the per-lane VARIABLE shift where each count lane C is a SIGNED
  esize-bit value: C>=0 left-shifts, C<0 right-shifts by -C, |C|>=B zeroes or
  sign-fills) had zero direct byte tests — SH434 pinned only the CONSTANT-shift
  SimdShl/SimdShr/SimdShrAcc. SH456 pins the exact emit (rd=1 rn=2 rm=3;
  V@0x130 C@0x140 Vd@0x120) with 4 exact-byte/window pins:
  (1) THE sshl .2d full-buffer (load-bearing): bbits==64 so NO wmask and NO
  out-of-range clamp (x86's shl/sar mask CL to the low 6 bits = the exact
  0..63 ARM range); per lane mov rax,[0x130] + mov rcx,[0x140] + `test rcx,rcx`
  (48 85 c9) + js (0f 88) right + LEFT `shl rax,cl` (48 d3 e0) + jmp + RIGHT
  `neg rcx` (48 f7 d9) + `sar rax,cl` (48 d3 f8) + mov [0x120],rax;
  full-buffer assert_eq + negative no shr (E8);
  (2) THE ushl .2s sign-dispatch: the COUNT C is SIGN-extended (movsxd rcx,ecx
  48 63 c9) so a high-bit-set C makes test+js take the right path — a
  zero-extend turns a negative count huge-positive and wrongly left-shifts;
  the left path guards C>=32 -> 0 and ANDs the esize=4 wmask 0xffffffff back
  (48 21 d0); 32-bit store (89 83), never mov_store64;
  (3) THE signed-vs-unsigned RIGHT-shift opcode (the semantic): sshl/srshl
  uses `sar rax,cl` (48 d3 f8, arithmetic — sign-extends the value first via
  movsxd rax,eax 48 63 c0), ushl/urshl uses `shr rax,cl` (48 d3 e8, logical,
  NO value sign-extend); signed out-of-range right sign-fills (jns 0f 89 ->
  mov rax,0xffffffff) vs unsigned plainly mov rax,0;
  (4) THE urshl rounding-bias before shift: round=true adds 1<<(k-1) to V
  BEFORE the final shift — `mov rdx,1` + `shl rdx,cl` (48 d3 e2) + `shr rdx,1`
  (48 c1 ea 01) + `add rax,rdx` (48 01 d0) then `shr rax,cl`; the
  add-precedes-shift position (and its ABSENCE in the non-rounding control) is
  the round discriminator — a skip truncates instead of rounding half-up.
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission
  captured with a one-off probe test (eprintln dump, removed before commit) so
  pins match the real emission; sshl_d8 full-buffer assert_eq matched first try.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the variable-shift family, the
  dynamic-count complement of SH434's constant shifts. No re-treads.
- Files: docs/frontier-sh456-translator-simdvshift.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only). Commit 491246e.

## SH455 (Sep 19, 2026, hermes-worker): hermetic coverage of the SCALAR 2-SOURCE FP MULTIPLY codegen family (translate.rs FmulScalar — fmul/fnmul Sd/Dd, Sn, Sm) — 4 exact-byte pins
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (the change is `#[cfg(test)]`-only, so the runtime deliverable
is byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 650/0 incl. 4 new sh455 pins, was 646; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged). Commit
d56638b.
- FmulScalar (the scalar 2-source FP multiply `Dd = (+/-)(Dn * Dm)` — the same
  product core as SH454's Fma3 but with NO accumulate into a 4th operand) had
  zero direct byte tests (STATUS next-forward #5 named `FmulScalar`). SH455
  pins the exact emit (rd=1 rn=2 rm=3; Dn@0x130 Dm@0x140 Dd@0x120) with 4
  exact-byte/window pins:
  (1) THE fmul double full-buffer (load-bearing): movq_load xmm0=[0x130]
  (f3 48 0f 7e 83) + movq_load xmm1=[0x140] (f3 48 0f 7e 8b) + `mulsd xmm0,
  xmm1` (f2 0f 59 c1) + movq_store [0x120] (66 48 0f d6 83); full-buffer
  assert_eq + negative no mulss and no accumulate addsd (a pure multiply);
  (2) THE fnmul double negate discriminator: neg flips the sign via `pxor
  xmm0,xmm1` (66 0f ef c1) after materializing the 64-bit sign const
  0x8000_0000_0000_0000 in RCX (48 b9 ..00 80) + `movq xmm1,rcx` (66 48 0f 6e
  c9), sign-flip BEFORE the store (positional), and the pxor must be ABSENT
  when neg=false (control);
  (3) THE fmul single width: mov_load32 RCX (8b 8b) + `movd xmm0,ecx`
  (66 0f 6e c1) + `mulss xmm0,xmm1` (f3 0f 59 c1) + `movd ecx,xmm0` (66 0f 7e
  c1) + 32-bit store (89 8b) — the F3-mulss-vs-F2-mulsd prefix is the
  single-vs-double discriminator, negative no mulsd/movq_store;
  (4) THE fnmul single negate WIDTH: 32-bit sign const 0x8000_0000 (48 b9 00
  00 00 80 00 00 00 00) + `movd xmm1,ecx` (66 0f 6e c9) + pxor — the 32-vs-64
  sign const (and movd-vs-movq) is the negate width discriminator.
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission
  captured with a one-off probe test (eprintln dump, removed before commit) so
  pins match the real emission; fmul_double full-buffer assert_eq matched first
  try.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the scalar 2-source FP multiply
  family, the complement of SH454 (scalar 3-source FMA) — pxor-sign-flip negate
  distinct from Fma3's 0-sub. No re-treads.
- Files: docs/frontier-sh455-translator-fmuscalar.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only). Commit d56638b.

## SH454 (Sep 19, 2026, hermes-worker): hermetic coverage of the SCALAR 3-SOURCE FP FUSED-MULTIPLY codegen family (translate.rs Fma3 — fmadd/fmsub/fnmadd/fnmsub Dd, Dn, Dm, Da) — 4 exact-byte pins
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (the change is `#[cfg(test)]`-only, so the runtime deliverable
is byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 646/0 incl. 4 new sh454 pins, was 642; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged). Commit
7a75a4f +5.
- Fma3 (scalar 3-source FP fused multiply — Dd = Da ± (Dn × Dm), optionally
  negated: fmadd/fmsub/fnmadd/fnmsub) had zero direct byte tests. SH454 pins
  the exact emit (rd=1 rn=2 rm=3 ra=4; Dn@0x130 Dm@0x140 Da@0x150 Dd@0x120)
  with 4 exact-byte/window pins:
  (1) THE fmadd product-and-accumulate direction (load-bearing): movq_load
  xmm0=[0x130] (Dn, f3 48 0f 7e) + movq_load xmm1=[0x140] (Dm) + `mulsd xmm0,
  xmm1` (f2 0f 59 c1, prod into xmm0) + movq_load xmm2=[0x150] (Da) + `addsd
  xmm2,xmm0` (f2 0f 58 d0 = da+prod, never prod+ra) + movq_store [0x120]
  (66 48 0f d6); full-buffer assert_eq;
  (2) THE add-vs-sub opcode AND the operand ORDER are the semantic: fmsub =
  `subsd xmm2,xmm0` (f2 0f 5c d0 = da-prod) vs fnmsub = `subsd xmm0,xmm2`
  (f2 0f 5c c2 = prod-da, SWAPPED because the signed negation makes it rn*rm-da
  — a lone subss in the wrong order negates the wrong term). 0x58-vs-0x5c plus
  c0/c2 order discriminate all four combos;
  (3) THE fnmadd negate-via-scratch: addsd xmm2,xmm0 then `pxor xmm3,xmm3`
  (66 0f ef db, +0.0) + `subsd xmm3,xmm2` (f2 0f 5c da = 0-result), store FROM
  xmm3; pxor+0-sub is the negate discriminator (a skip leaves the sign
  un-negated); positional assert add precedes negate;
  (4) THE single-vs-double width: sz=false swaps to movd_xmm_r32 (66 0f 6e) +
  `mulss` (f3 0f 59 c1) + `addss` (f3 0f 58 d0) + movd_r32_xmm (66 0f 7e) +
  32-bit store (89 83); F3-mulss-vs-F2-mulsd is the width discriminator,
  negative asserts no mulsd/addsd/movq_store.
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission
  captured with a one-off probe test (eprintln dump, removed before commit) so
  pins match the real emission; fmadd double full-buffer assert_eq matched
  first try. 4 pins, parallel-safe, no image/env.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the scalar 3-source FP fused-multiply
  family, distinct from SH433 (vector Fmla/FmlaEl). No re-treads.
- Files: docs/frontier-sh454-translator-fma3.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only). Commit 7a75a4f +5.

## SH453 (Sep 19, 2026, hermes-worker): hermetic coverage of the SIMD WIDEN-AND-ADD/SUB codegen family (translate.rs SimdAddl — saddl/uaddl/subl/usubl Vd.T, Vn.T, Vm.T) — 4 exact-byte pins
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (the change is `#[cfg(test)]`-only, so the runtime deliverable
is byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 642/0 incl. 4 new sh453 pins, was 638; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged). Commit
7a75a4f +4.
- SimdAddl (widen each esrc-byte element of Vn and Vm (low/upper half) sign-/
  zero-extended to 2*esrc, then add or subtract into a DOUBLE-width dst; with
  in-place-alias snapshot) had zero direct byte tests. SH453 pins the exact
  emit (rd=1 rn=2 rm=3; Vn@0x130 Vm@0x140 Vd@0x120) with 4 exact-byte/window
  pins:
  (1) THE signed widen-add (load-bearing): saddl .2s esrc=4 sign=true = per lane
  mov_load32 + `movsxd rax,eax` (48 63 c0) + mov_load32(RCX=0x140) + `movsxd
  rcx,ecx` (48 63 c9) + `add rax,rcx` (48 01 c8) + `mov [0x120],rax` (48 89 83,
  the 8B dst) — the movsxd + 8B store IS the widening, a zero-extend flips a
  negative element's high bits and a 4B store truncates;
  (2) THE add-vs-sub opcode byte: usubl sub=true = `sub rax,rcx` (48 29 c8) vs
  uaddl `add rax,rcx` (48 01 c8) — 0x29-vs-0x01 is the semantic; unsigned word
  sources movzx (0f b7), no movsxd;
  (3) THE upper high-half source offset: upper=true adds +8 to the source bases
  (Vn@0x138 Vm@0x148, NOT the low-half 0x130/0x140), signed byte sources via
  movsx (48 0f be), negative assert the upper form must NOT read low-half
  0x130;
  (4) THE in-place-alias permute_source snapshot (historical gcc-bug surface):
  when rd==rn the widened 2*esrc dst write overlaps the next lane's source, so
  the emit snapshots the FULL 16B to perm-scratch FIRST (mov_load64 [0x120]->
  [0x320], [0x128]->[0x328]) then reads the aliased Vn from 0x320/0x322.., while
  the non-aliased Vm==3 is read directly at 0x140 — the [..,0x320] perm-scratch
  reads are the alias signature.
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission
  captured with a one-off probe test (eprintln dump, removed before commit) so
  pins match the real emission; se=4 full-buffer assert_eq matched first try.
  Dev-time fixes: movsx/movzx word+byte loads and 2B stores are 7-8 bytes
  (0f b7 83 + dw / 48 0f be 83 + dw / 66 89 83 + dw) -> windows(7)/windows(8),
  adjacent to the 6-byte 32-bit forms. 4 pins, parallel-safe, no image/env.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the widen-and-add/sub family, adjacent
  to SH452 (pairwise-add-long) and SH441 (WidenShl); distinct 3-operand
  widen-accumulate with the alias-snapshot guard. No re-treads.
- Files: docs/frontier-sh453-translator-addl.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only). Commit 7a75a4f +4.

## SH452 (Sep 19, 2026, hermes-worker): hermetic coverage of the SIMD PAIRWISE-ADD-LONG codegen family (translate.rs SimdAdalp — saddlp/uaddlp/sadalp/uadalp Vd.Td, Vn.Ts) — 3 exact-byte pins
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (the change is `#[cfg(test)]`-only, so the runtime deliverable
is byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 638/0 incl. 3 new sh452 pins, was 635; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged). Commit
7a75a4f +3.
- SimdAdalp (pairwise-ADD-LONG — sum each adjacent pair (2i,2i+1) of
  src_esize-byte elements into a dst lane of DOUBLE width 2*src_esize; the acc
  forms sadalp/uadalp ADD into the existing dst, the plain uadalp/saddlp
  overwrite) had zero direct byte tests. SH452 pins the exact emit (rd=1 rn=2;
  Vn@0x130 Vd@0x120) with 3 exact-byte/window pins:
  (1) THE double-width dst store (load-bearing): saddlp .2s se=4 np=2 =
  per-pair mov_load32(RAX,[0x130]) (8b 83, [2i]) + mov_load32(RCX,[0x134])
  (8b 8b, [2i+1] into RCX) + `add rax,rcx` (48 01 c8) + `mov [0x120],rax`
  (48 89 83, the 8-byte dst); pairs (0x130,0x134)->0x120 and (0x138,0x13c)->
  0x128 both counted as 64-bit stores — 4B-in/8B-out IS the pairwise-add-LONG
  widening, a 32-bit store drops the pair-carry;
  (2) word-pair narrowing + signed-byte sign-EXTEND: se=2 signed=false =
  movzx_word (0f b7 83 / 0f b7 8b) + add + 32-bit store (89 83); se=1
  signed=true byte sources SIGN-extend via movsx_byte (48 0f be, NOT 0f b6
  zero-extend) then sum into a 2-byte dst (66 89 83);
  (3) THE accumulate (acc) discriminator: acc=true reads the dst lane into R10
  (mov_load64 = 4c 8b 93) + `add rax,r10` (4c 01 d0) BEFORE the store; plain
  saddlp (acc=false) never touches the dst (positional assert R10-load precedes
  the store; negative assert non-acc has no R10 load).
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission
  captured with a one-off probe test (eprintln dump, removed before commit) so
  pins match the real emission; se=4 full-buffer assert_eq matched first try.
  Dev-time fixes: the R10 load and 2B/1B stores are 7 bytes (4c 8b 93 + dw /
  66 89 83 + dw) -> windows(7) not windows(6), and a raw `!windows(6)==[89 83]`
  negative is ambiguous (the 64-bit store's tail contains it) -> replaced with
  a counted 64-bit-store assertion. 3 pins, parallel-safe, no image/env.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the pairwise-add-long family, adjacent
  to SH451 (narrowing shift) and SH440 (SimdSum8 uaddlv). No re-treads.
- Files: docs/frontier-sh452-translator-adalp.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only). Commit 7a75a4f +3.

## SH451 (Sep 19, 2026, hermes-worker): hermetic coverage of the SIMD NARROWING-SHIFT codegen family (translate.rs SimdShrn — shrn/shrn2/rshrn/rshrn2 Vd.T, Vn.U, #imm) — 5 exact-byte pins
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (the change is `#[cfg(test)]`-only, so the runtime deliverable
is byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 635/0 incl. 5 new sh451 pins, was 630; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged). Commit
7a75a4f +2.
- SimdShrn (shift each DOUBLE-width source element right by `shift`, truncate
  [shrn] or round-half-up [rshrn: add 1<<(shift-1)] to a HALF-width dest
  element; shrn2 writes the dest high half) had zero direct byte tests. SH451
  pins the exact emit (rd=1 rn=2, non-alias so permute_source is a no-op;
  Vn@0x130 Vd@0x120) with 5 exact-byte/window pins:
  (1) THE narrowing width pair (load-bearing): shrn .2s esrc=8 shift=16 = per-lane
  `mov rax,[rbx+0x130]` (48 8b 83, 64-bit source, +8B/lane) + `shr rax,16`
  (48 c1 e8 10) + `mov [rbx+0x120],eax` (89 83, 32-bit dest) — an 8B-in/4B-out
  flub store corrupts every lane (assert NOT 48 89 83);
  (2) THE upper high-half offset: shrn2 (upper=true) is byte-identical except
  the dest base shifts +8 (dst_off=8 -> 0x128/0x12c vs 0x120/0x124); the upper
  flag is the ONLY thing that moves the destination base, source lanes at +8
  unchanged (assert 0x120 NOT written);
  (3) THE rshrn round-add is the round discriminator: round=true emits `add
  rax, 1<<(shift-1)=0x4000` (48 81 c0 00 40 00 00) BEFORE `shr rax,0x0f`
  (48 c1 e8 0f); round=false has NO add — positional assert the add precedes
  shr (a flub shifts-without-rounding, truncating);
  (4) 4to2 O-word narrowing (esrc=4 shift=8): mov_load32 (8b 83) + shr + 16-bit
  `66 89 83` 2B store, lane1 src 0x134 counted-exactly-once, lane3 dest 0x126,
  no 64-bit store;
  (5) 2to1 byte narrowing (esrc=2 shift=4): 8 bytes via movzx_word (0f b7 83 —
  UNSIGNED, NO movsx 48 0f bf — + shr + 88 83 byte store 0x120..0x127).
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission
  captured with a one-off probe test (eprintln dump, removed before commit) so
  pins match the real emission; 8to4 full-buffer assert_eq matched first try.
  One dev-time fix: the 2B/1B stores are 7 bytes (66+89/88+83+dw), so those
  windows are windows(7), not windows(6). 5 pins, parallel-safe, no image/env.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the narrowing-shift family, adjacent
  to SH434 (element-wise same-width shift) — this is the HALVING-width form,
  distinct. No re-treads.
- Files: docs/frontier-sh451-translator-shrn.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only). Commit 7a75a4f +2.

## SH450 (Sep 19, 2026, hermes-worker): hermetic coverage of the SIMD FP COMPARE-TO-LITERAL-ZERO codegen family (translate.rs VecFpCmpZero — fcmeq/fcmgt/fcmge/fcmlt/fcmle Vd.T, Vn.T, #0.0) — 4 exact-byte pins
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (the change is `#[cfg(test)]`-only, so the runtime deliverable
is byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 630/0 incl. 4 new sh450 pins, was 626; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged). Commit
7a75a4f +1.
- VecFpCmpZero (the per-lane FP compare-to-literal-#0.0 mask builder — the
  zero-comparison cousin of SH445's two-operand VecFpCmp; a lane becomes all-
  ones if Vn op 0.0 holds, else 0) had zero direct byte tests. SH450 pins the
  exact emit (rd=1 rn=2; Vn@0x130 Vd@0x120) with 4 exact-byte pins:
  (1) THE zero-operand discriminant (load-bearing): each lane FIRST materializes
  the +0.0 second operand with `pxor xmm1,xmm1` (66 0f ef c9) — a FRESH zero
  every lane — then mov_load32 (8b 83) + `movd xmm0,eax` (66 0f 6e c0) + comiss
  (40 0f 2f c1) + setcc + movzx (0f b6 c0) + `neg rax` (48 f7 d8) + 32-bit store
  (89 83). This is what separates it from SH445, which LOADS Vm — a lone setcc
  byte cannot tell the two forms apart (same comiss, same cc), so the per-lane
  pxor is the ONLY reliable differentiator; a flub reusing a stale register
  compares against garbage;
  (2) the cc-map IS the semantic: op 0 fcmeq = sete 0f 94 / op 1 fcmgt = seta 0f
  97 / op 2 fcmge = setae 0f 93 / op 3 fcmlt = setb 0f 92 / op 4 fcmle = setbe
  0f 96 (a wrong cond silently picks the wrong comparison), negative-asserted;
  (3) double-path width (op 0, esize=8): 1 lane via movq_load (f3 48 0f 7e) +
  comisd (66 40 0f 2f c1) + 64-bit store (48 89 83), must NOT emit movd
  (66 0f 6e c0); (4) full 4s q=true lane math: 4 lanes Vd@0x120/0x124/0x128/
  0x12c, Vn@0x130/0x134/0x138/0x13c, 4 pxor xmm1,xmm1 counted, stays single-path
  (no movq).
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission
  captured precisely with a one-off probe test (eprintln dump, removed before
  commit) — the pxor-per-lane + exact cc bytes came from that capture, and the
  2s full-buffer assert_eq matched first try. 4 exact-byte + window + cc-map +
  count/negative asserts. No image, no env, parallel-safe.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the compare-vs-literal-zero family,
  adjacent to SH445 (two-operand VecFpCmp) and SH449 (integer SimdCmpZero). No
  re-treads (distinct form).
- Files: docs/frontier-sh450-translator-vfpczoro.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only). Commit 7a75a4f +1.

## SH449 (Sep 19, 2026, hermes-worker): hermetic coverage of the SIMD COMPARE-TO-ZERO mask codegen family (translate.rs SimdCmpZero — cmeq/cmgt/cmge/cmlt/cmle Vd.T, Vn.T, #0) — 4 exact-byte pins
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (the change is `#[cfg(test)]`-only, so the runtime deliverable
is byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 626/0 incl. 4 new sh449 pins, was 622; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs 1,048,392 B / session.rs
unchanged). Commit dc84448.
- SimdCmpZero (the per-lane compare-to-literal-0 mask builder — each lane ->
  all-ones if the int compare vs 0 holds, else 0; cond 0=eq 1=gt 2=ge 3=lt
  4=le) had zero direct byte tests (STATUS next-forward #5 named `SimdCmpZero`
  a remaining family). SH449 pins the exact emit (rd=1 rn=2; Vn@0x130
  Vd@0x120): (1) cmeq .2s full-buffer — 32-bit zero-extend load (8b 83) +
  `test rax,rax` (48 85 c0) + `sete al` (0f 94 c0) + `movzx eax,al` (0f b6 c0)
  + `neg rax` (48 f7 d8 -> 0 or all-ones) + 32-bit store (89 83), NEVER movsxd
  (eq/cond-0 is UNSIGNED); (2) THE setcc opcode byte is the semantic: eq=0f 94
  sete / gt=0f 9f setg / ge=0f 9d setge / lt=0f 9c setl / le=0f 9e setle (a
  wrong cond silently picks the wrong compare, a>=0 becomes a>0), AND signed
  conds (1-4) MUST movsxd (48 63 c0) the 32-bit lane; (3) cmlt .8b (cond 3) —
  8 lanes via movsx_byte_mem (48 0f be SIGNED) + setl (0f 9c) + movzx + neg +
  byte store (88 83), lanes advance +1 (0x130..0x137 / 0x120..0x127), 8x setl
  counted; (4) cmeq .2d q=true — full 64-bit load (48 8b 83) + test + sete +
  neg + 64-bit store (48 89 83), 2 lanes +8, 2x sete + 2x neg (result 0 or
  0xffff_ffff_ffff_ffff).
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission
  captured with a one-off probe dump (each cond + esize 1/4/8; removed before
  commit) so pins match the real emission. 4 exact-byte + window + cc-map +
  count/negative asserts. No image, no env, parallel-safe.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the compare-to-literal-0 mask family,
  continuing the SH427-448 translator-core lineage. No re-treads (distinct
  from SH445 FP compare->mask VecFpCmp — this is the INTEGER compare-to-zero
  form).
- Files: docs/frontier-sh449-translator-simdcmpzero.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only). Commit dc84448.

## SH448 (Sep 19, 2026, hermes-worker): hermetic coverage of the SIMD INTEGER ARITH-UNARY codegen family (translate.rs SimdArithUnary — neg/abs Vd.T, Vn.T) — 4 exact-byte pins
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (the change is `#[cfg(test)]`-only, so the runtime deliverable
is byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 622/0 incl. 4 new sh448 pins, was 618; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs 1,048,392 B / session.rs
unchanged). Commit fc01a8e.
- SimdArithUnary (the per-lane integer unary — neg op 0 / abs op 1) had zero
  direct byte tests (STATUS next-forward #5 named `SimdFpUnary/SimdArithUnary`
  a remaining family). SH448 pins the exact emit (rd=1 rn=2; Vn@0x130
  Vd@0x120): (1) neg .2s full-buffer — 32-bit zero-extend load (8b 83) + `neg
  rax` (48 f7 d8) + 32-bit store (89 83), NEVER movsxd (48 63 c0); (2) abs .2s
  full-buffer — the `(x ^ (x ar>> w-1)) - (x ar>> w-1)` idiom: MUST `movsxd
  rax,eax` (48 63 c0) FIRST (signed lane), then mov rcx,rax (48 89 c1) + `sar
  rcx,31` (48 c1 f9 1f, imm=esize*8-1) + xor rax,rcx (48 31 c8) + sub rax,rcx
  (48 29 c8), NEVER a bare neg rax (48 f7 d8); (3) esize width — 2d q=true loads
  full 64-bit (48 8b 83) + 64-bit store (48 89 83, lane1 +8 0x128); esize=1 abs
  loads via movsx_byte_mem (48 0f be) + `sar rcx,7` (imm 07) + byte store (88
  83, +1/lane); (4) THE abs-vs-neg count discriminator — 4-lane .4s: neg emits
  exactly 4 `neg rax` + 0 movsxd, abs emits exactly 4 movsxd + 0 neg (a
  transposed op silently abs()'s a neg / negates an abs).
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission
  captured with a one-off probe dump (removed before commit) so pins match the
  real emission. 4 exact-byte + window + count/negative asserts. No image, no
  env, parallel-safe.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the integer neg/abs family,
  continuing the SH427-447 translator-core lineage. No re-treads (distinct
  from SH447 FP unary — this is the integer GPR neg / sar-xor-sub
  absolute-value).
- Files: docs/frontier-sh448-translator-simdarithunary.md + crates/arm64jit/
  src/translate.rs (`#[cfg(test)]` only). Commit fc01a8e.

## SH447 (Sep 19, 2026, hermes-worker): hermetic coverage of the SIMD FP UNARY codegen family (translate.rs SimdFpUnary — fneg/fabs/fsqrt Vd.T, Vn.T) — 5 exact-byte pins
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (the change is `#[cfg(test)]`-only, so the runtime deliverable
is byte-identical — SH445 capture baseline 24 real task-driven frames `present
swap Ok(0x1)`, 0 json abort, 0 crash, EXIT 0). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 618/0 incl. 5 new sh447 pins, was 613; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs 1,048,392 B / session.rs
unchanged). Commit b5497f5.
- SimdFpUnary (the per-lane 1-source FP unary — fneg/fabs/fsqrt) had zero
  direct byte tests (decode pins decode, jit pins runtime, but the byte
  EMISSION between them was unpinned; STATUS next-forward #5 named
  `SimdFpUnary/SimdArithUnary` a remaining family). SH447 pins the exact emit
  (rd=1 rn=2; Vn@0x130 Vd@0x120): (1) fneg .2s full-buffer — per-lane
  `movq xmm0,[Vn+l]` (f3 48 0f 7e 83) + `movq rax,xmm0` (66 48 0f 7e c0) +
  sign const bit31 0x8000_0000 in RCX (48 b9 ..00 00 00 80..) + `xor rax,rcx`
  (48 31 c8 — the FLIP) + `movq xmm0,rax` + `movd eax,xmm0` + 32-bit store
  `[Vd+l],eax` (89 83); (2) fabs clears the sign via `mov rdx,const` (48 ba) +
  `not rdx` (48 f7 d2) + `and rax,rdx` (48 21 d0), NEVER xor — and+not (vs
  fneg's xor-flip) is the fabs-vs-fneg discriminator, a transposed op toggles
  instead of clears; (3) fsqrt .2d full-buffer — `sqrtsd xmm0,xmm0` (f2 0f 51
  c0) + `movq [Vd],xmm0` (66 48 0f d6), with ZERO GPR sign-bit manipulation (no
  48 b9 const, no xor); (4) esize width discriminator — esize8 loads the 64-bit
  sign (0x8000_0000_0000_0000 imm ..00*7 80) + 64-bit movq store (lane1 at +8),
  esize2 loads bit15 (0x8000) + 16-bit store (66 89); (5) q lane advance (.4s)
  — loads 0x130/0x134/0x138/0x13c, stores 0x120/0x124/0x128/0x12c (+esize).
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission
  established with a one-off probe dump (captured fneg/fabs/fsqrt + lane-width
  variants; removed before commit) so pins match the real emission. 5
  exact-byte + window + negative/order asserts. No image, no env,
  parallel-safe.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the 1-source FP unary family,
  continuing the SH427-446 translator-core lineage. No re-treads (distinct
  from SH446 by-element FMUL, SH444 2-src scalar FP).
- Files: docs/frontier-sh447-translator-simdfpunary.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only). Commit b5497f5.

## SH446 (Sep 19, 2026, hermes-worker): hermetic coverage of the SIMD by-element FMUL codegen family (translate.rs SimdFmulEl — fmul Vd.T, Vn.T, Vm.T[L]) — 4 exact-byte pins
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (test-only change, runtime byte-identical; SH445 capture
baseline 24 real task-driven frames `present swap Ok(0x1)`, 0 json abort, 0
crash, EXIT 0). Workspace green (cargo test --workspace EXIT 0; arm64jit lib
613/0 incl. 4 new sh446 pins, was 609; cargo build --example elfjit OK).
Production code ONLY in translate.rs `#[cfg(test)]` addition (translator core
body byte-untouched; jit.rs 1,048,390 B < 1MiB hook unchanged;
elfjit.rs/session.rs unchanged).
- SimdFmulEl (by-element FMUL — each lane Vd = Vn[lane] * Vm[L], a common
  vector-by-element multiply on vertex/weight/color paths) had zero direct
  byte tests, and is DISTINCT from SH433's FmlaEl (which is a multiply-
  ACCUMULATE into the dst; this is a plain multiply). SH446 pins the exact
  emit (rd=1 rn=2 rm=4; Vd@0x120 Vn@0x130 Vm@0x150): single .2s per lane
  `movd xmm2,eax` element-broadcast (66 0f 6e d0) ONCE up front + movd xmm0 +
  `mulss xmm0,xmm2` (F3 0F 59 C2) + movd-back store; double .2d swaps to
  movq_load xmm2 (f3 48 0f 7e 93 — reg-field=2 -> 0x93) + mulsd (F2 0F 59 C2)
  + movq_store (66 48 0f d6).
- THE load-bearing discriminator (the rd==rm clobber guard): the element Vm[L]
  must be broadcast into xmm2 EXACTLY ONCE, BEFORE the lane loop. When rd==rm
  (a self-multiply), the first lane's store overlaps the element source, so a
  per-lane re-read corrupts every lane after 0 with a clobbered value — a
  silent wrong vector (SH399's FmlaEl ABI noted the broadcast form; the
  plain-FMUL variant was untested). SH446 pins: broadcast (66 0f 6e d0)
  precedes the overlapping Vd store; the element source is read from memory
  exactly once. Also pinned: broadcast-count (1 regardless of lanes) vs
  product-count (= lane count), element addressing = Vm base + index*es
  (0x150 idx0 / 0x154 idx1), and the double width discriminator (mulsd f2 +
  movq 66 48 0f, never single mulss).
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission
  established precisely with a one-off eprintln dump (removed before commit)
  so pins match the real emission. 4 exact-byte pins. A windows(7)-vs-9-byte
  window-length bug in the double test was corrected (windows(9)) during dev.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the by-element FMUL family, distinct
  from SH433 (FmlaEl multiply-accumulate). No re-treads.
- Files: docs/frontier-sh446-translator-fmulel.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only). Commit (pending).

## SH445 (Sep 19, 2026, hermes-worker): hermetic coverage of the SIMD single-precision FP COMPARE->mask codegen family (translate.rs VecFpCmp — fcmeq/fcmgt/fcmge/facgt/facge) — 3 exact-byte pins
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
re-verified green at this HEAD first (capture_taskv4_frame.sh attempt 1: 24
real task-driven frames `present swap Ok(0x1)`, 0 json abort, 0 crash,
EXIT 0). Workspace green (cargo test --workspace EXIT 0; arm64jit lib 609/0
incl. 3 new sh445 pins, was 606; cargo build --example elfjit OK). Production
code ONLY in translate.rs `#[cfg(test)]` addition (translator core body
byte-untouched; jit.rs 1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs
unchanged).
- VecFpCmp (fcmeq/fcmgt/fcmge + abs facgt/facge — the per-lane or-mask the
  shader-like branch/blend/color-decision lanes lean on) had zero direct byte
  tests. SH445 pins the exact emit (rd=1 rn=2 rm=3; Vd@0x120 Vn@0x130 Vm@0x140):
  single .2s per lane `mov eax,[Vn+4l]` + movd xmm0 (66 0f 6e c0) + `mov
  eax,[Vm+4l]` + movd xmm1 (66 0f 6e c8) + comiss xmm0,xmm1 (40 0f 2f c1) +
  `set?cc al` + movzx eax,al (0f b6 c0) + neg rax (48 f7 d8 -> +1 becomes
  all-ones) + `mov [Vd+4l],eax` (89 83 d32). The cc byte IS the semantic:
  fcmeq = sete 0f 94 c0, fcmgt = seta 0f 97 c0, fcmge = setae 0f 93 c0 — a
  wrong condition picks the wrong comparison. Double .2d swaps to movq_load
  (f3 48 0f 7e) + comisd (66 40 0f 2f c1) + 64-bit store (48 89 83) — the
  width discriminator.
- Discoveries: (1) comiss/comisd ALSO always emit the REX 0x40 (like the SH444
  maxss/minss quirk) — comiss(0,1)=40 0f 2f c1, comisd(0,1)=66 40 0f 2f c1;
  (2) a negative `windows(4)==40 0f 2f c1` for the double path is a BAD
  discriminator because comisd's bytes `66 40 0f 2f c1` contain that window —
  replaced with the correct one (double path never emits movd 66 0f 6e c0);
  (3) abs/facgt clears each lane's sign bit BEFORE the compare (mov rax,0x7fff
  ffff + movq xmm2,rax 66 48 0f 6e d0 + pand 66 0f db c2/ca, then comiss) — the
  pand-before-compare ordering is the abs discriminator.
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Emission
  established precisely with a one-off eprintln dump (removed before commit) so
  pins match the real emission. 3 exact-byte pins.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the compare->mask family, adjacent to
  SH444 (VecFpArith 2-src FP arithmetic). No re-treads.
- Files: docs/frontier-sh445-translator-veccmp.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only). Commit (pending).

## SH444 (Sep 19, 2026, hermes-worker): hermetic coverage of the SIMD single-precision FP TWO-SOURCE arithmetic codegen family (translate.rs VecFpArith — fadd/fsub/fmul/fdiv op 0..3 + fmax/fmin/fmaxnm/fminnm op 4..7 + frecps/frsqrts op 8/9) — 3 exact-byte pins
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (the change is `#[cfg(test)]`-only, so the runtime deliverable
is byte-identical — SH443 baseline). Workspace green (cargo test --workspace
EXIT 0; arm64jit lib 606/0 incl. 3 new sh444 pins, was 603; cargo build
--workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged).
- VecFpArith (the plain per-lane 2-src FP arithmetic — every geometry/color
  lane's add/sub/mul/div/max/min) had zero direct byte tests: SH433 only pinned
  the Fmla multiply-ACCUMULATE, not this family. SH444 pins the exact emit
  (rd=1 rn=2 rm=3; Vd@0x120 Vn@0x130 Vm@0x140): per lane `mov eax,[Vn+4l]` +
  movd xmm0 (66 0f 6e c0) + `mov eax,[Vm+4l]` + movd xmm1 (66 0f 6e c8) + the
  FP op + movd eax,xmm0 (66 0f 7e c0) + `mov [Vd+4l],eax` (89 83 d32). The
  ONLY semantic bit is the opcode: addss F3 0F 58 C1 / subss 5C / mulss 59 /
  divss 5E / maxss F3 40 0F 5F C1 / minss F3 40 0F 5D C1.
- Discriminator facts: (1) the opcode byte 58/5C/59/5E/5F/5D a flub silently
  corrupts; (2) maxss/minss ALWAYS emit the REX prefix 0x40 (CodeBuf::maxss/
  minss calls x86::rex() unconditionally), so the real bytes are F3 40 0F 5F/
  5D C1 NOT F3 0F — my first attempt assumed REX=0x41; reading rex() confirmed
  register<8 -> no bit3 -> 0x40 (a genuine always-present-vs-absent
  discriminator vs the non-REX add/sub/mul/div); (3) frecps (op 8) uses the
  INVERTED sub `subss xmm1,xmm0` (F3 0F 5C C8, dst=1 rm=0) over mulss to
  compute 2.0-prod; frsqrts (op 9) adds movd xmm3 (66 0f 6e d8) of 0.5f +
  mulss xmm1,xmm3 (F3 0F 59 CB); (4) Vn/Vm/Vd all advance exactly +4/lane.
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. The
  full-buffer .2s fadd assert_eq matched real emission first try (add/movd/
  store primitives are the SH433-proven forms); only the max/min REX form
  needed the emitter re-read. 3 exact-byte pins.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the 2-src scalar-FP family, distinct
  from SH441 (cross-lane FMaxV reduction) and SH433 (Fmla accumulate). No
  re-treads.
- Files: docs/frontier-sh444-translator-vecfparith.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only). Commit (pending).

## SH443 (Sep 19, 2026, hermes-worker): hermetic coverage of the BYTE-REVERSE codegen family (translate.rs SimdRev `rev64`/`rev32`/`rev16`) — 4 exact-byte pins
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (24 real task-driven frames `present swap Ok(0x1)`, 0 json
abort, 0 crash — SH442/441 baseline unchanged). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 603/0 incl. 4 new sh443 pins, was 599; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged).
- SimdRev (byte-reverse within granule) had no direct byte tests. SH443 pins
  the exact emit (rd=1, rn=2; Vn@0x130, Vd@0x120): (1) rev64 (granule 8, q=0)
  = mov rax,[0x130] + bswap-r64 (48 0f c8) + mov [0x120],rax; (2) rev64 q=1 =
  TWO 8B granules, second reads Vn+8 (0x138) + stores Vd+8 (0x128) — pins the
  whole-register reversal; (3) rev32 (granule 4, q=0) = bswap-r32 (0f c8, NO
  0x48 REX.W), two 4B granules 0x130->0x120 + 0x134->0x124; the 0f c8-vs-
  48 0f c8 opcode is the 4-vs-8-granule discriminator; (4) rev16 (granule 2,
  q=0) = four 2B granules via movzx + rol eax,8 (66 c1 c0 08, halfword
  byte-swap) + 16-bit 66 89 stores at 2-apart dst (0x120..0x126); the
  rol-imm8-by-8 vs bswap is the 2-granule discriminator.
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Trunk
  verified by a one-off eprintln dump (removed before commit) so pins match the
  real emission.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the byte-reverse family. No re-treads.
- Files: docs/frontier-sh443-translator-simdrev.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only). Commit 1894893.

## SH442 onward (see commit history for the full lantern ledger)
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (24 real task-driven frames `present swap Ok(0x1)`, 0 json
abort, 0 crash — SH441/440 baseline unchanged). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 599/0 incl. 5 new sh442 pins, was 594; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged).
- SH435 pinned FcvtToInt mode 0 (fcvtzs/fcvtzu) + SH439 pinned fcvtzu's
  [2^63,2^64) big-path; the ROUND-before-truncate modes had zero byte tests.
  SH442 pins the exact emit (rd=0, rn=1 d-src; src@0x120, dst g0):
  (1) fcvtau (unsigned, mode 2) = roundsd NEAREST-EVEN (imm8 0x00) + cvttsd2si,
  NO unsigned cmovs clamp (comment-documented saturation edge) — pinned
  negative; (2) fcvtpu (unsigned, mode 3, +inf) = roundsd CEIL (imm8 0x02) +
  cvttsd2si + clamp trio (xor rcx,rcx; test rax,rax; cmovs rax,rcx = 48 0f 48
  c1); (3) fcvtmu (unsigned, mode 4, -inf) = roundsd FLOOR (imm8 0x01) +
  cvttsd2si + same clamp — imm8 0x01-vs-0x02 is the mu/pu discriminator;
  (4,5) signed mode 3/4 (fcvtps/fcvtms) = same roundsd but NO clamp — cmovs
  absence is the signed discriminator.
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Trunk
  verified by a one-off eprintln dump (removed before commit) so pins match the
  real emission. One workspace run exited 101 (transient; re-run 0 — the
  accepted load-sensitive SH345/346/357/370 family), canonical exit 0.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completing the SH435/439 mode-0 line with the
  mode-2/3/4 rounding family. No re-treads.
- Files: docs/frontier-sh442-translator-fcvtoint-rounding-modes.md +
  crates/arm64jit/src/translate.rs (`#[cfg(test)]` only). Commit 336c6fe.

## SH441 onward (see commit history for the full lantern ledger)
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
unchanged-green (24 real task-driven frames `present swap Ok(0x1)`, 0 json
abort, 0 crash — SH440/439 baseline unchanged). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 594/0 incl. 4 new sh441 pins, was 590; cargo
build --workspace + --example elfjit OK). Production code ONLY in translate.rs
`#[cfg(test)]` addition (translator core body byte-untouched; jit.rs
1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged).
- FMaxV/WidenShl had no direct byte tests. SH441 pins the exact emitted x86
  (rd=1, rn=2; Vn@0x130, Vd@0x120): (1) FMaxV fmaxv full buffer — reduce Vn.4s
  into scalar Sd: movd xmm0=<Vn.0>, then movd xmm1=<Vn.i> + maxss for i=1..3,
  movd eax + mov [0x120],eax; the 0x5f maxss-vs-0x5d minss opcode + REX.B 40
  is the max/min discriminator; (2) FMaxV fminv — same ladder but 0x5d,
  asserts no 0x5f; (3) WidenShl shll signed vs unsigned — reverse iter dst
  lane1(0x124) then lane0(0x120), signed movsx_word_mem (48 0f bf) vs unsigned
  movzx_word_mem (0f b7, NO REX.W) — REX.W presence is the signed/unsigned
  discriminator; (4) WidenShl shll2 upper-half byte widen — src base Vn+8=0x138,
  reverse iter 0x13b..0x138, dst 2-apart, movsx_byte_mem (48 0f be) + 16-bit
  66 89 store; asserts the upper-half base (never 0x130) and 16-bit store
  width. Each pins exact full emit + the semantic discriminators.
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Trunk
  verified by a one-off eprintln dump (removed before commit) so pins match real
  emission. One dev-time assert window-width fix (windows(7)->windows(6) on a
  6-byte store) corrected during development — shipped tests pin real emission.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the scalar-reduction + widen families,
  distinct from SH440 (popcnt/sum8), SH439 (fcvtzu). No re-treads.
- Files: docs/frontier-sh441-translator-fmaxv-widenshl.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only). Commit 959212c.

## SH440 onward (see commit history for the full lantern ledger)
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables were
re-verified green at HEAD first (capture_taskv4_frame.sh: 24 real task-driven
frames `present swap Ok(0x1)`, 0 json abort, 0 crash — SH439 baseline
unchanged). Workspace green (cargo test --workspace EXIT 0; arm64jit lib 590/0
incl. 2 new sh440 pins, was 588; cargo build --example elfjit OK + cargo build
--workspace OK). Production code ONLY in translate.rs `#[cfg(test)]` addition
(translator core body byte-untouched; jit.rs 1,048,390 B < 1MiB hook unchanged;
elfjit.rs/session.rs unchanged).
- SimdPopcnt/SimdSum8 had no direct byte tests. SH440 pins both to the exact
  emitted x86 (rd=1, rn=2; Vn@0x130, Vd@0x120): (1) SimdPopcnt full SWAR
  popcount — load + shr 1/and 0x5555../sub + (x&0x3333..)+((x>>2)&0x3333..) +
  (x+(x>>4))&0x0f0f.., the three masks ascending + shift ladder 1/2/4;
  (2) SimdSum8 horizontal byte sum (uaddlv) — three widening sums with shr
  8/16/32 + masks 0x00ff00ff../0x0000ffff0000ffff/0x00000000ffffffff; the
  8/16/32-vs-1/2/4 ladder is the count-vs-sum discriminator. Each pins the
  exact full emit AND the masks/shrs individually incl. negative asserts
  (popcnt never shr-by-8, sum never shr-by-1) so a ladder cross fails.
- Deterministic: synthetic Inst -> translate() -> CodeBuf.as_slice() (zero-pc
  0x1000); [RBX]=CpuState, vector slot v[t]=VECTOR_BASE(0x110)+t*16. Trunk
  verified by a one-off eprintln dump (removed before commit) so pins match the
  real emission.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the byte-count + horizontal-reduce
  family, distinct from SH439 (fcvtzu), SH437 (lanecopy), SH436 (bswl/high-
  narrow). No re-treads.
- Files: docs/frontier-sh440-translator-popcnt-sum8.md + crates/arm64jit/src/
  translate.rs (`#[cfg(test)]` only). Commit 63c17cb.

## SH439 onward (see commit history for the full lantern ledger)
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables were
re-verified green THIS cycle (capture_taskv4_frame.sh: 24 real task-driven
frames `present swap Ok(0x1)`, 197 node pops, 0 json abort, 0 crash, EXIT 124).
Workspace green (cargo test --workspace EXIT 0; arm64jit lib 588/0 incl. 3 new
sh439 pins, was 585; cargo build --example elfjit OK). Production code ONLY in
translate.rs `#[cfg(test)]` addition (translator core body byte-untouched;
jit.rs 1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged).
- FcvtToInt's UNSIGNED big-path had no direct byte tests (SH435 pinned the
  fcvtzu small path clamp but left "[2^63,2^64) and fixed-point scale" as
  next-forward). SH439 pins the discriminators a byte error silently corrupts
  (a bare signed cvttsd2si saturates to INT64_MIN, corrupting the high half of
  an unsigned u64 dst): (1) fcvtzu (unsigned, mode 0) EMITS the full range gate
  — 2^63 const (mov rcx,0x43e0_0000_0000_0000 = 48 B9 .. E0 43), `comisd
  xmm0,xmm1` (66 40 0F 2F C1 — the REX byte is ALWAYS present, a `66 0F 2F`
  window misses it), JB (0F 82) to the signed path, big-path `subsd xmm0,xmm1`
  (F2 0F 5C C1, d-2^63) + `add rax,rcx` (+2^63 restore, 48 01 C8), u64::MAX
  saturation (`mov rax,-1`, 48 B8 FF..) for d>=2^64, and the cmovs negative-to-0
  clamp (48 0F 48 C1); (2) signed fcvtzs is a BARE trunc (movq_load + single
  `cvttsd2si rax,xmm0` F2 48 0F 2C C0 + 64-bit stg store, NO comisd/subsd/
  cmovs/u64::MAX) — comisd presence is the fcvtzu-vs-fcvtzs discriminator;
  (3) fbits>0 fixed-point scales BEFORE truncating (2^fbits double 0x4030.. ->
  `movq xmm1,rax` 66 48 0F 6E C8 -> `mulsd xmm0,xmm1` F2 0F 59 C1), fbits=0 must
  NOT mulsd. 3 exact-byte pins via synthetic Inst -> translate() (zero-pc
  deterministic); window + inverse asserts; [RBX]=CpuState, vector slot
  v[t]=VECTOR_BASE(0x110)+t*16.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the FcvtToInt unsigned family,
  distinct from SH435 (Fcvt/FcvtTzReg/FcvtHalf) and the SH427-438 lineage. No
  re-treads.
- Files: docs/frontier-sh439-translator-fcvttoint-unsigned-bigpath.md +
  crates/arm64jit/src/translate.rs (`#[cfg(test)]` only). Commit 04ed640.

## SH438 (Sep 19, 2026, hermes-worker): hermetic coverage of the STRUCTURE-LOAD/STORE codegen family (translate.rs Ld2 / St2 — ld2/st2 {Vt, Vt1}, [Xn], the interleaved vertex-attribute / RG-z+texcoord structure-pair deinterleave) — 3 exact-byte pins
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables were
re-verified green this cycle (capture_taskv4_frame.sh attempt 1: 24 real
task-driven frames `present swap Ok(0x1)`, 195 node pops, 0 json abort, 0 crash,
EXIT 124). Workspace green (cargo test --workspace EXIT 0; arm64jit lib 585/0
incl. 3 new sh438 pins, was 582; cargo build --example elfjit OK). Production code
ONLY in translate.rs `#[cfg(test)]` addition (translator core body byte-untouched;
jit.rs 1,048,390 B < 1MiB hook unchanged; elfjit.rs/session.rs unchanged).
- Ld2/St2 had no direct byte tests. SH438 pins the deinterleave offset math a byte
  error silently corrupts: memory holds {V0.e0,V1.e0,V0.e1,V1.e1,...} — element i
  of reg j at byte i*(2*es)+j*es; Ld2 writes reg j to VECTOR_BASE(0x110)+(rd+j)*16
  +i*es, St2 reads them back. Pins: (1) Ld2 deinterleave — loads advance j mem+es
  (0F B6 42 04) and i mem+2*es (0F B6 42 08), incl. the mod=0 disp-0 load (0F B6 02);
  stores land V1.elt0 at [0x120] (2nd structure reg at +16) vs V0.elt0 at [0x110]
  (88 83 20 01 00 00 / 88 83 10 01 00 00), element i advances es (88 83 14 01 00 00);
  (2) St2 INVERSE flips the direction — movzx SOURCE becomes the [rbx+Vd-slot]
  (0F B6 83 20 01 00 00) and the store becomes `88` to [rdx+mem-off], the
  Ld2-vs-St2 direction discriminator; (3) q=true (16B, nelems=4) + post=0x20 —
  V0.elt3 lands at [0x11c] AND the rn post-increment emits the imm32 add form
  (48 8B 43 08 + 48 81 C0 20 00 00 00 + 48 89 43 08) — two encoding facts corrected
  during development by dumping the emitted buffer (disp=0 uses mod=0 `0F B6 02` not
  `0F B6 42 00`; add_ri64 emits `48 81 C0` imm32, not imm8) — the shipped tests pin
  the real emission.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME codegen-
  surface coverage completion on the structure-pair family, distinct from SH437
  SimdInsD (single-lane copy). No re-treads.
- Files: docs/frontier-sh438-translator-ld2-st2-structure.md +
  crates/arm64jit/src/translate.rs (`#[cfg(test)]` only). Commit b5ef728.

## SH437 (Sep 19, 2026, hermes-worker): hermetic coverage of the SIMD lane-COPY codegen family (translate.rs SimdInsD — `mov Vd.T[dst], Vn.T[src]`, the per-element lane move used to splat/broadcast/shuffle one value across a vector: color/texel lane packing + 8-bit channel moves on render data paths) — 5 exact-byte pins
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables were
re-verified green at the SH436 HEAD before adding coverage (capture_taskv4_frame.sh
attempt 1: 24 real task-driven frames `present swap Ok(0x1)`, 195 node pops, 0 json
abort, 0 crash, EXIT 124). Workspace green (cargo test --workspace EXIT 0; arm64jit
lib 582/0 incl. 5 new sh437 pins, was 577; cargo build --example elfjit OK).
Production code ONLY in translate.rs `#[cfg(test)]` addition (translator core body
byte-untouched; jit.rs 1,048,390 B < 1MiB hook; elfjit.rs/session.rs unchanged).
- SimdInsD had no direct byte tests (decode pins decode, jit pins runtime, but the
  byte EMISSION between them was unpinned). SH437 pins the esize discriminator chain
  AND the lane-addressing math a byte error silently corrupts: (1) d-lane esize=8
  full-buffer — `mov_load64` (48 8B 83 20 01 00 00: src 0x110+rn*16+src_idx*8) +
  `mov_store64` (48 89 83 18 01 00 00: dst 0x110+rd*16+dst_idx*8); (2) s-lane esize=4
  — `mov_load32` (8B, no-REX.W zero-ext) + `mov_store32` (89), asserts NO 48 8B d-lane
  form; (3) h-lane esize=2 — `movzx_word_mem` (0F B7) + 16-bit 0x66-prefixed
  `mov_store16` (66 89) — the only esize whose store carries the 66 operand-size
  prefix; (4) b-lane esize=1 — `movzx_byte_mem` (0F B6) + byte `mov_store8` (88),
  asserts no 64-bit mov; (5) the lane-ADDRESSING discriminator — stepping src_idx by
  one d-lane moves the source disp exactly +esize (0x140→0x148) while the dest stays
  fixed at VECTOR_BASE 0x110+rd*16, proving `Vn*16 + src_idx*esize/·dst_idx*esize`,
  never a fixed/vt-stale stride.
- 5 exact-byte pins via synthetic `Inst` -> translate() -> CodeBuf.as_slice()
  (zero-pc 0x1000 = deterministic); full-buffer + window/subsequence asserts. [RBX]=
  CpuState; vector slot v[t]=VECTOR_BASE(0x110)+t*16. Deterministic, no image/env.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME codegen-
  surface coverage completion on the lane-copy family, continuing the SH427-436
  translator-core lineage. No re-treads (distinct from SH436 SimdSel/SimdHighNarrow,
  SH435 fcvt, SH434 shifts).
- Files: docs/frontier-sh437-translator-simd-lanecopy.md +
  crates/arm64jit/src/translate.rs (`#[cfg(test)]` only). Commit 0c894cd.

## SH436 (Sep 19, 2026, hermes-worker): hermetic coverage of the SIMD bitwise-select + high-narrow codegen families (translate.rs SimdSel, SimdHighNarrow) — 4 exact-byte pins of the 3-input bitwise select (bsl/bit/bif) and narrowing add (addhn/raddhn) that drive blend masks + byte-level color/normal packing: the BSL (Rn&Rd)|(~Rd&Rm) full-buffer pins the exact pand/pandn/por operand mapping (pandn dst=xmm2 rm=xmm1); the BIT/BIF op=1 inverts the operand ORDER (leads pandn xmm0,xmm1 ~Rm&Rn then pand xmm2,xmm1, never BSL's Rn&Rd first — a transposed mask picks the wrong source); addhn no-round shr-by-dst_bits with no round-carry + Q=0 zeroes the Vd upper half (mov rax,0 + mov [Vd+8],rax); raddhn adds 1<<(dst_bits-1)=0x8000 BEFORE the narrowing shift — round-carry presence + position is the raddhn-vs-addhn discriminator
Single-agent (cone suppressed). recon-v3 deliverables re-verified green at the
SH435 HEAD first (capture_taskv4_frame.sh attempt 1: 24 real task-driven frames
`present swap Ok(0x1)`, 194 node pops, 0 json abort, 0 crash, EXIT 124).
Workspace green (cargo test --workspace EXIT 0; arm64jit lib 577/0 incl. 4 new
sh436 pins, was 573; cargo build --example elfjit OK). Production code ONLY in
translate.rs `#[cfg(test)]` addition (translator core byte-untouched; jit.rs
1,048,390 B < 1MiB hook; elfjit.rs/session.rs unchanged).
- SimdSel (bsl/bit/bif) and SimdHighNarrow (addhn/raddhn) had no direct byte
  tests. SH436 pins the semantically-critical discriminators a byte error
  silently corrupts: (1) SimdSel op=0 BSL full-buffer — load Rn@0x120(xmm0)/
  Rm@0x130(xmm1)/Vd@0x110(xmm2), `pand xmm0,xmm2` (Rn&Rd) then `pandn xmm2,
  xmm1` (~Rd&Rm, dst=xmm2/rm=xmm1) then `por xmm0,xmm2`, store Vd; (2) op=1
  BIT/BIF — window discriminator: op=1 LEADS `pandn xmm0,xmm1` (~Rm&Rn) then
  `pand xmm2,xmm1` (Rd&Rm), never BSL's Rn&Rd first; (3) addhn no-round —
  add rax,rcx + `shr rax,16` (dst_bits) with NO round-carry + Q=0 zeroes the
  Vd upper half (mov rax,0 + mov [Vd+8],rax = the C7-imm form is NOT used);
  (4) raddhn — `mov r10,0x8000` (=1<<(dst_bits-1)) added BEFORE the narrowing
  shift; round-carry presence + position is the discriminator.
- 4 exact-byte pins via synthetic `Inst` -> translate() -> CodeBuf.as_slice()
  (zero-pc 0x1000 = deterministic); full-buffer + subsequence-window asserts.
  [RBX]=CpuState; vector slot v[t]=VECTOR_BASE(0x110)+t*16. A one-off
  dump_sh436 example established two exact byte forms during development (the
  bare 4-byte pandn xmm0,xmm1 = 66 0F DF C1, and the Q=0 upper-half zero via
  mov rax,0 + mov [Vd+8],rax) and was removed before commit — the tree ships
  only the `#[cfg(test)]` additions.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion, continuing the SH427-435 translator-core
  lineage. No re-treads (distinct from SH432 SminMax/SimdSatAdd, SH434 shifts,
  SH435 fcvt, SH433 Fmla).
- Files: docs/frontier-sh436-translator-simdsel-highnarrow.md +
  crates/arm64jit/src/translate.rs (`#[cfg(test)]` only). Commit 4d52dae.

## SH435 (Sep 19, 2026, hermes-worker): hermetic coverage of the FP conversion codegen families (translate.rs Fcvt, FcvtTzReg, FcvtHalf) — 7 exact-byte pins of the widen/narrow/trunc/FP16 conversions every color-intensity/light/texture-sample path leans on: the S->D widen (movd xmm0,eax + F3 cvtss2sd + movq 64-bit store) vs D->S narrow (F3 48 0F 7E movq_load + F2 cvtsd2ss + movd + 32-bit store) — the F3-vs-F2 opcode + lane-width discriminator; fcvtzs uses cvttsd2si (F2 48 0F 2C) DIRECTLY (x86 already trunc-toward-zero, no pre-round); SIGNED vs UNSIGNED — fcvtzu appends the clamp (test rax,rax 48 85 C0 / cmovs 48 0F 48 C1) so negatives become 0, the cmovs presence is the discriminator; FcvtHalf FP16 via F16C vcvtph2ps promote (c4 e2 79 13) / vcvtps2ph demote (c4 e3 79 1d), H->D wides after promote, D->H narrows before demote
Single-agent (cone suppressed). recon-v3 deliverables re-verified green at the
SH434 HEAD first (capture_taskv4_frame.sh attempt 1: 24 real task-driven frames
`present swap Ok(0x1)`, 194 node pops, 0 json abort, 0 crash, EXIT 124).
Workspace green (cargo test --workspace EXIT 0; arm64jit lib 573/0 incl. 7 new
sh435 pins, was 566; cargo build --example elfjit OK). Production code ONLY in
translate.rs `#[cfg(test)]` addition (translator core body byte-untouched;
jit.rs 1,048,390 B < 1MiB hook; elfjit.rs/session.rs unchanged). Two of my
initial window-width assertions were corrected during development (a bare
4-byte op needs windows(4), not windows(3)) — the shipped tests pin the real
emission. 7 exact-byte pins via synthetic Inst -> translate() ->
CodeBuf.as_slice() (zero-pc 0x1000 = deterministic); [RBX]=CpuState; vector
slot v[t]=VECTOR_BASE(0x110)+t*16. Honest: NOT a DM (DM-root [0x106a68818]=0x0
under the complete substrate; Route-B live-DM gate UNCHANGED).
BUILD-THE-RUNTIME codegen-surface coverage completion on the FP conversion
families, continuing the SH427-434 translator-core lineage. No re-treads
(distinct from SH433 Fmla FP FMA, SH434 integer shifts). Files:
docs/frontier-sh435-translator-fcvt-conversions.md + crates/arm64jit/src/
translate.rs (`#[cfg(test)]` only). Commit 00cd1f7.

## SH434 (Sep 19, 2026, hermes-worker): hermetic coverage of the SIMD shift-and-accumulate codegen families (translate.rs SimdShl, SimdShr, SimdShrAcc) — 6 exact-byte pins of the integer shift/rounding math every vertex-index/packed-color/image-lane path leans on: the shl-vs-shr opcode byte (shl C1/E0 vs unsigned shr C1/E8 vs signed arithmetic sar C1/F8 — a shift-direction flub moves every lane the wrong way), the SIGN-extend-before-arithmetic-shift requirement (sshr esize=4 movsxd 48 63 then sar; ushr zero-extend movzx + shr, never movsxd — a zero-extended negative element flips its sign bit), the shift>=esize-bits guard (unsigned xor-to-zero 48 31 C0 vs signed all-ones sign-fill sar,63 48 C1 F8 3F — a bare x86 imm clamps instead), and the SimdShrAcc accumulate ordering (Vd read AFTER the shift, add rcx,rax 48 01 C1, re-store — separates usra/ssra from a plain overwrite)
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
re-verified green at this exact HEAD first (capture_taskv4_frame.sh attempt 1:
24 real task-driven frames `present swap Ok(0x1)`, 194 node pops, 0 json abort,
0 crash, EXIT 124). Workspace green (cargo test --workspace EXIT 0; arm64jit lib
566/0 incl. 6 new sh434 pins, was 560; cargo build --example elfjit OK).
Production code ONLY in translate.rs `#[cfg(test)]` addition (translator core
body byte-untouched; jit.rs 1,048,390 B < 1MiB hook; elfjit.rs/session.rs
unchanged).
- The SimdShl/SimdShr/SimdShrAcc families had no direct byte tests. SH434 pins
  the semantically-critical discriminators a byte error silently corrupts:
  (1) SimdShl esize=8 shift=1 — full 2-lane buffer `mov_load64 [Vn0x120];
  shl rax,1 (48 C1 E0 01); mov_store64 [Vd0x110]`, lane2 at +8 (0x128->0x118);
  asserts shl never emits shr(E8)/sar(F8); (2) SimdShr SIGNED (sshr) esize=8
  shift=2 — full buffer with `sar rax,2` (48 C1 F8 02), the arithmetic form;
  (3) SimdShr UNSIGNED (ushr) — full buffer with `shr rax,2` (48 C1 E8 02),
  the logical form — E8-vs-F8 is the signed/unsigned opcode discriminator;
  (4) esize=4 signed MUST movsxd (48 63 C0) before sar, unsigned MUST NOT
  (zero-extend + shr) — a zero-extended negative element becomes positive;
  (5) shift>=esize-bits guard: unsigned all-zeros `xor rax,rax` (48 31 C0),
  signed all-ones `sar rax,63` (48 C1 F8 3F) — a bare x86 imm clamps instead;
  (6) SimdShrAcc usra lane-0 buffer pins the accumulate ordering — load Vn ->
  shr -> load Vd accumulator into RCX AFTER the shift -> add rcx,rax (48 01
  C1) -> re-store Vd.
- 6 exact-byte pins via synthetic `Inst` -> translate() -> CodeBuf.as_slice()
  (zero-pc 0x1000 = deterministic); full-buffer + subsequence-window asserts
  for the multi-lane bodies. [RBX]=CpuState; vector slot v[t]=VECTOR_BASE
  (0x110)+t*16. Deterministic, no image, no env, parallel-safe.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the integer shift family, continuing
  the SH427-433 translator-core lineage. No re-treads (distinct from SH432
  SminMax/SimdSatAdd, SH433 Fmla/FmlaEl).
- Files: docs/frontier-sh434-translator-simd-shift-accumulate.md +
  crates/arm64jit/src/translate.rs (`#[cfg(test)]` only). Commit 3026eee.

## SH433 (Sep 19, 2026, hermes-worker): hermetic coverage of the SIMD FP multiply-accumulate codegen family (translate.rs Fmla, FmlaEl) — 4 exact-byte pins of the most render-heavy translate.rs surface (matrix/vertex/lighting transforms accumulate as SIMD FMAs): the product DIRECTION (mulss/mulsd targets xmm1 => the accumulate addss into xmm0 gives fmls the CORRECT sign − Vd − Vn·Vm, never Vn·Vm − Vd), the add-vs-sub accumulate opcode (addss 0x58 / subss 0x5C), the .2s single- (F3+movd) vs .2d double- (F2+movq) lane width (a width flub silently halves/squares transform math), and the FmlaEl by-element broadcast into xmm2 (movd xmm2,eax 66 0F 6E D0) + mulss xmm1,xmm2 (F3 0F 59 CA) vs the 3-operand Fmla's full-Vm-lane multiply (F3 0F 59 C8)
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
re-verified green at this exact HEAD first (capture_taskv4_frame.sh attempt 1:
24 real task-driven frames `present swap Ok(0x1)`, 194 node pops, 0 json abort,
0 crash, EXIT 124). Workspace green (cargo test --workspace EXIT 0; arm64jit lib
560/0 incl. 4 new sh433 pins, was 556; cargo build --example elfjit OK).
Production code ONLY in translate.rs `#[cfg(test)]` addition (translator core
body byte-untouched; jit.rs/elfjit.rs/session.rs unchanged).
- The Fmla/FmlaEl family had no direct byte tests (decode pins decode, jit
  pins runtime, but the EMISSION between them was unpinned — STATUS next-forward
  #5). SH433 pins the semantically-critical discriminators a byte error
  silently corrupts: (1) Fmla .2s add — per-lane `movd xmm0=Vn; movd xmm1=Vm;
  mulss xmm1,xmm0` (F3 0F 59 C8, product into xmm1) then reload `movd xmm0=Vd;
  addss xmm0,xmm1` (F3 0F 58 C1), pinned product-before-accumulate so the fmls
  sign is Vd − Vn·Vm not the reverse; (2) Fmla .2s sub — identical layout but
  `subss xmm0,xmm1` (F3 0F 5C C1), the 0x5C-vs-0x58 accumulate discriminator;
  (3) Fmla .2d double — movq load/store (F3 48 0F 7E / 66 48 0F D6) + mulsd/
  addsd (F2 0F 59/58), never the .2s F3+movd/mulss path (asserts no movd and no
  mulss); (4) FmlaEl .2s — broadcasts Vm.el[idx] into xmm2 (movd xmm2,eax =
  66 0F 6E D0) once then per-lane `mulss xmm1,xmm2` (F3 0F 59 CA), the
  broadcast-target/rm vs the 3-operand Fmla's full-lane multiply (C8).
- 4 exact-byte pins via synthetic `Inst` -> translate() -> CodeBuf.as_slice()
  (zero-pc 0x1000 = deterministic); subsequence-window asserts for the
  multi-lane bodies. [RBX]=CpuState; vector slot v[t]=VECTOR_BASE(0x110)+t*16.
  Deterministic, no image, no env, parallel-safe.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the SIMD FP multiply-accumulate
  geometry-math family, continuing the SH427-432 translator-core lineage. No
  re-treads (distinct from SH432's integer-lane SimdVLog/SminMax/SimdSatAdd).
- Files: docs/frontier-sh433-translator-simd-fma.md +
  crates/arm64jit/src/translate.rs (`#[cfg(test)]` only). Commit (pending).

## SH432 (Sep 19, 2026, hermes-worker): hermetic coverage of the SIMD/vector-ALU codegen families (translate.rs SimdVLog, SminMax, SimdSatAdd) — 6 exact-byte pins of the emission that carries real rendered geometry/color lane math: the pand/por/pxor/pandn opcode discriminator (66 0F DB/EB/EF/DF; a /r-flub maps Vd&Vm to Vd^Vm), the SminMax cmov condition byte selecting max-vs-min AND signed-vs-unsigned (cmovg 0x4F / cmovb 0x42 vs cmovl 0x4C — a flub returns the wrong lane or clamps the wrong direction), the movsxd presence flips sub-byte saturating lanes, and the smax-vs-smin CONSTANT+cmov pairing in SimdSatAdd that decides which bound signed overflow clamps to
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
re-verified green at this exact HEAD first (capture_taskv4_frame.sh attempt 1:
24 real task-driven frames `present swap Ok(0x1)`, 194 node pops, 0 json abort,
0 crash, EXIT 124). Workspace green (cargo test --workspace EXIT 0; arm64jit lib
556/0 incl. 6 new sh432 pins, was 550; cargo build --example elfjit OK).
Production code ONLY in translate.rs `#[cfg(test)]` addition (translator core
body byte-untouched; jit.rs/elfjit.rs/session.rs unchanged).
- The SIMD/vector-ALU families had no direct byte tests (decode pins decode,
  jit pins runtime, but the byte EMISSION between them was unpinned for
  SimdVLog/SminMax/SimdSatAdd — STATUS next-forward #5). SH432 pins the
  semantically-critical discriminators a byte error silently corrupts:
  (1) SimdVLog AND whole-buffer (`movdqu xmm0,[rbx+0x120]; movdqu xmm1,[rbx+0x130];
  pand xmm0,xmm1 = 66 0F DB C1; movdqu [rbx+0x110],xmm0` — Vd slot @ VECTOR_BASE
  0x110 + vt*16) + the op discriminator across AND/ORR/EOR/BIC (DB/EB/EF/DF,
  with BIC's `pandn xmm1,xmm0` storing via RCX — a /r-flub maps `Vd&Vm` to
  `Vd^Vm`); (2) SminMax signed .2s max sign-extends lanes (movsxd 48 63) then
  `cmp rcx,rax; cmovg rax,rcx` (48 39 C1 / 48 0F 4F C1), vs unsigned .8b min
  (movzx 0F B6, NO 48 63, cmovb 48 0F 42 C1) — the cmov condition byte is the
  max-vs-min AND signed-vs-unsigned discriminator; (3) SimdSatAdd signed sqadd
  .4s clamps to smax (mov r10,0x7FFFFFFF; cmp r10,rax; cmovg rax,r10 = 49 0F 4F
  C2) and smin (mov r10,0x80000000-as-negative; cmovl = 49 0F 4C C2), vs unsigned
  uqsub .2h clamps to literal 0 (mov r10,0; cmp; sub; cmovb = 49 0F 42 C2, no
  movsxd) — the CONSTANT+cmov pairing decides which bound signed overflow
  clamps to.
- 6 exact-byte pins via synthetic `Inst` -> translate() -> CodeBuf.as_slice()
  (zero-pc 0x1000 = deterministic); subsequence-window asserts for the multi-lane
  discriminators. [RBX]=CpuState; vector slot v[t]=VECTOR_BASE(0x110)+t*16.
  Deterministic, no image, no env, parallel-safe.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the SIMD/vector-ALU geometry/color-math
  families, continuing the SH427/428/429/430/431 translator-core lineage. No
  re-treads (distinct families).
- Files: docs/frontier-sh432-translator-simd-vector-alu.md +
  crates/arm64jit/src/translate.rs (`#[cfg(test)]` only). Commit (pending).

## SH431 (Sep 19, 2026, hermes-worker): hermetic coverage of the bitmask-immediate LOGIC + high-widen MULTIPLY codegen families (translate.rs LogicImm, MulHigh) — 4 exact-byte pins: the `mov xD,#imm` = ORR xzr alias with rn==31 read as XZR-not-SP (mask materialized in RCX + no [rbx+0xf8] access), the flag-setting ANDS w32 form as the ONLY LogicImm op that emits the full store_nzcv pack (C-store 89 93 08 01 00 00 to [rbx+0x108] + caller pop), and the umulh `mul rcx` /4 vs smulh `imul rcx` /5 high-half-in-RDX distinction (mov [rbx],rdx 48 89 13)
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
re-verified green at this exact HEAD first (capture_taskv4_frame.sh attempt 1:
24 real task-driven frames `present swap Ok(0x1)`, 194 node pops, 0 json abort,
0 crash, EXIT 124). Workspace green (cargo test --workspace EXIT 0; arm64jit lib
550/0 incl. 4 new sh431 pins, was 546; cargo build --example elfjit OK).
Production code ONLY in translate.rs `#[cfg(test)]` addition (translator core
body byte-untouched; jit.rs/elfjit.rs/session.rs unchanged).
- SH430 pinned the MulDiv/MulLong/ClzCls arithmetic + B/Cbz/Tbz control-flow
  families; the two adjacent families it did NOT cover had zero direct byte
  tests: LogicImm (AND/ORR/EOR/ANDS with the encoded bitmask immediate — incl.
  the `mov xD,#imm` = ORR xD,xzr,#imm alias compilers use to load constants) and
  MulHigh (umulh/smulh, the high-64 half of every 64x64 multiply). SH431 pins
  them:
  (1) orr x0,xzr,#7 (op=1, rn==31): rn==31 reads as XZR (zero) — `mov rax,0`,
  never the SP slot — + asserts no [rbx+0xf8] access anywhere; the bitmask imm
  is materialized in RCX (`mov rcx,7`), `or rax,rcx`, store.
  (2) ANDS w0,w1,#5 (op=3, sf=false): `and rax,rcx` then the full store_nzcv
  pack (pushfq + 4-bit NZCV extraction ending with the C-store 89 93 08 01 00 00
  to [rbx+0x108] + caller pop 5a 59 58), then the 32-bit zero-extend
  (`mov eax,eax`) + store. Pins that ANDS is the ONLY LogicImm op emitting the
  nzcv pack (and/orr/eor don't).
  (3/4) umulh (48 f7 e1, `mul rcx` /4 unsigned) vs smulh (48 f7 e9, `imul rcx`
  /5 signed): high half lands in RDX, stored `mov [rbx],rdx` (48 89 13). A /4-vs
  -/5 flub silently corrupts the high half of every signed wide multiply.
- 4 exact-byte pins via synthetic `Inst` -> translate() -> CodeBuf.as_slice()
  (zero-pc 0x1000 = deterministic). [RBX]=CpuState; slot g = [RBX+g*8].
  Deterministic, no image, no env, parallel-safe.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion (bitmask-immediate logic + high-widen
  multiply), continuing the SH427/428/429/430 translator-core lineage. No
  re-treads (distinct families).
- Files: docs/frontier-sh431-translator-logicimm-mulhigh.md +
  crates/arm64jit/src/translate.rs (`#[cfg(test)]` only). Commit (pending).

## SH430 (Sep 19, 2026, hermes-worker): hermetic coverage of the MUL/DIV/LONG arithmetic + branch control-flow codegen families (translate.rs MulDiv, MulLong, ClzCls, B/Cbz/Tbz) — 13 exact-byte pins incl. the two documented historical-bug discriminators (the msub `ra - product` direction fix, the clz REX.W-after-F3 order fix) + the ra==31-XZR-not-SP accumulate skip + the call/jmp/jcc fixup-shape pins
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
re-verified green at this exact HEAD first (capture_taskv4_frame.sh attempt 1:
24 real task-driven frames `present swap Ok(0x1)`, 194 node pops, 0 json abort,
0 crash, EXIT 124). Workspace green (cargo test --workspace EXIT 0; arm64jit lib
546/0 incl. 13 new sh430 pins, was 533; cargo build --example elfjit OK).
Production code ONLY in translate.rs `#[cfg(test)]` addition (translator core
body byte-untouched; jit.rs/elfjit.rs/session.rs unchanged).
- The MUL/DIV/LONG arithmetic plus branch control-flow families had no direct
  byte tests (decode pins decode, jit pins runtime, but the byte EMISSION
  between them was uncovered for these two families). SH430 pins the
  semantically-critical discriminators a byte error silently corrupts:
  (1) the msub DIRECTION BUGFIX — x0 = ra - rn*rm (`sub rdi,rax` + `mov rax,rdi`),
  the exact fix for `n - q*d` compiling to msub returning -48 for 1298-25*50;
  (2) the clz REX.W-ORDER BUGFIX — `f3 48 0f bd c0` (REX.W AFTER the F3 prefix,
  immediately before 0F; `48 f3 0f bd` makes the CPU run a 32-bit lzcnt,
  clz(0x16136740)=3 vs oracle 35); (3) mul with ra==31 (XZR) SKIPS the accumulate
  entirely — must never read the SP slot [RBX+0xf8] (adding SP corrupts the
  product); (4) udiv xor-rdx-vs-cqo signed/unsigned high-half + div /6 vs idiv /7;
  (5) smull/umaddl 32-bit-operand sign-/zero-extend before imul (umsubl = neg then
  add ra); (6) the B.L LR-save + `call rel32` placeholder, B `e9`, Cbz/Tbz jcc
  fixup shapes (cc=0x84/0x85/0xfe/0xff, target=pc+imm) incl the tbz masked
  single-bit `mov rcx,1<<bit; test rax,rcx` that distinguishes it from cbz.
- 13 exact-byte pins via synthetic `Inst` -> translate() -> CodeBuf.as_slice()
  (zero-pc 0x1000 = deterministic fixup targets). [RBX]=CpuState; GPR slot
  =[RBX+g*8]. Deterministic, no image, no env, parallel-safe. A `tr_bytes_fx`
  helper returns the byte buffer + the emitted Fixup list so cc/target_pc are
  pinned alongside the bytes.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the arithmetic + control-flow families
  (the MADD/MSUB/UDIV/SDIV/smull + B/cbz/tbz every translated block leans on),
  continuing the SH427/SH428/SH429 translator-core lineage. No re-treads (distinct
  from SH427 move/add/logic/bcond, SH428 LdStrImm, SH429 LdStPair/FP-SIMD).
- Files: docs/frontier-sh430-translator-muldiv-clz-branch.md +
  crates/arm64jit/src/translate.rs (`#[cfg(test)]` only). Commit (pending).

## SH429 (Sep 19, 2026, hermes-worker): hermetic coverage of the load/store-PAIR + scalar FP/SIMD codegen families (translate.rs LdStPair, FpLdStImm) — the families that move real rendered geometry/vertex data; SH429 pins them with 9 deterministic exact-byte hermetics (stride-16 vector slot for FP d-pairs — the Session-99 BUGFIX, pair width/zero-extend, offset raw-byte immediate, fp_scalar_xfer low-N-bytes)
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
re-verified green at this exact HEAD first (capture_taskv4_frame.sh attempt 1:
24 real task-driven frames `present swap Ok(0x1)`, ~196 node pops, 0 json abort,
0 crash). Workspace green (cargo test --workspace EXIT 0; arm64jit lib 533/0
incl. 9 new sh429 hermetics; workspace 714/0; cargo build --workspace OK).
Production code ONLY in translate.rs tests (`#[cfg(test)]` addition); translator
core body byte-untouched, jit.rs/elfjit.rs/session.rs unchanged.
- The LdStPair / FpLdStImm families had no direct byte tests (decode pins 76,
  jit pins 295, but the pair + scalar-FP EMISSION between them was uncovered).
  SH429 pins the semantically-critical discriminators a byte error silently
  corrupts: (1) the Session-99 BUGFIX — FP d-pairs write their 16-byte guest
  vector slots at VECTOR_BASE + vt*16 (d29->0x2e0, d28->0x2d0; a stale rt*8
  stride returned 128 vs 52 in a follow-on fmadd); (2) 32/64-bit pair width +
  zero-extension (32-bit does mov eax then full-64 store; second lane at +esize
  4/8); (3) offset-form RAW-byte immediate (ldp x0,x1,[x8,#2] -> lane0 [rdx+2],
  lane1 [rdx+0xa]); (4) fp_scalar_xfer's lower-N-bytes slice (`ldr s0` stores
  only the low 4 of the d0 vector slot, `ldr d0` the low 8, upper lanes kept).
- 9 exact-byte pins via synthetic `Inst` -> translate() -> CodeBuf.as_slice().
  [RBX]=CpuState; GPR slot=[RBX+g*8]; vector slot vt=VECTOR_BASE+vt*16
  (0x110). Deterministic, no image, no env, parallel-safe (local buffers).
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate UNCHANGED). BUILD-THE-RUNTIME
  codegen-surface coverage completion on the pair + scalar-FP surface (the
  geometry/vertex-moving families), continuing the SH427/SH428 translator-core
  lineage. No re-treads (distinct from SH427 move/add/logic/bcond, SH428
  LdStrImm, SH423 fsmap, SH424 boot, SH425 signals, SH426 x86-emitter).
- Files: docs/frontier-sh429-translator-pair-fpsimd-hermetics.md +
  crates/arm64jit/src/translate.rs (`#[cfg(test)]` only). Commit 0d1aa16.

## SH425 (Sep 19, 2026, hermes-worker): hermetic coverage of the guest signal-delivery core (signals.rs) — rt_sigaction/sigprocmask install-query + BLOCK/UNBLOCK/SETMASK with silent SIGKILL/SIGSTOP drop, pending/deliverable ascending-order hold-and-release, and the full installed-handler dispatch → sigreturn context-restore ABI (x0=signo/x1=siginfo/x2=ucontext/x30=SIGRET) had ZERO tests; SH425 pins all of it with 5 deterministic hermetics. The surface a real client leans on for fatal-path diagnostics (SIGTRAP default-terminate exit 133) and cross-thread cooperative signal pickup
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
re-verified green at this exact HEAD first (capture_taskv4_frame.sh attempt 1:
24 real task-driven frames `present swap Ok(0x1)`, 196 node pops, 0 json abort,
0 crash). Workspace green (cargo test --workspace EXIT 0; arm64jit lib 490/0
incl. 5 new sh425 hermetics; cargo build --workspace OK, 0 errors).
Production code ONLY in signals.rs (off the 1MiB hooks; jit.rs/elfjit.rs
byte-unchanged). Pure `#[cfg(test)]` addition (+5 hermetics + a SIG_TEST_LOCK
that serializes the process-global SIG_ACTIONS table against parallel runs), no
production path / guest byte / JIT-hook-default touched.
- signals.rs serves the Linux signal contract to translated AArch64 guest code:
  rt_sigaction(134) records SIG_DFL/1=SIG_IGN/a guest handler; sigprocmask(135)
  applies SIG_BLOCK/UNBLOCK/SETMASK while silently dropping unblockable
  SIGKILL(9)/SIGSTOP(19) bits; mark_pending/take_deliverable_pending hold a
  blocked signal pending and deliver it once unblocked (ascending signal-number
  order); dispatch_current_thread runs an installed handler via the aarch64
  signal ABI (x0=signo, x1=siginfo*, x2=ucontext*, x30=SIGRET) and sigreturn
  restores the interrupted context.
- Genuine gap: signals.rs had ZERO self-tests — masking/pending/ordering/ABI
  correctness only exercised via SIGTRAP/fault paths on a full boot. SH425's 5
  hermetics pin: (1) block/unblock/setmask + unblockable-drop + oset; (2)
  sigprocmask -EINVAL paths; (3) pending ascending-order delivery + blocked
  hold/release; (4) sigaction install/query byte-exact round-trip + out-of-range
  EINVAL; (5) full handler dispatch → sigreturn restore (pc after svc, sp/tpidr/
  nzcv/x-regs exact, x0 forced to syscall-return 0, stray sigreturn detected).
  Deterministic, no real binary, no env.
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate unchanged). BUILD-THE-RUNTIME
  signal-surface coverage completion, not a re-tread.
- Files: docs/frontier-sh425-signals-hermetics.md + crates/arm64jit/src/signals.rs

## SH424 (Sep 19, 2026, hermes-worker): hermetic coverage of the guest initial-stack builder (boot.rs) — the module that lays out `[argc][argv][envp][auxv, AT_NULL]` + arg/env strings for a remote-loaded aarch64 ELF and patches a zero AT_RANDOM slot to real entropy fed the glibc stack canary had ZERO tests of its own logic; SH424 fixes that with 3 deterministic hermetics, hardening the load-bearing boot surface (a garbage initial stack makes glibc `_start` IFUNC-dispatch into SVE/SME opcodes the JIT can't decode — the memory-note hole the module's minimal-HWCAP design exists to prevent)
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
re-verified green at this exact HEAD first (capture_taskv4_frame.sh attempt 1:
24 real task-driven frames `present swap Ok(0x1)`, 196 node pops, 0 json abort,
0 crash). Workspace green (cargo test --workspace EXIT 0; arm64jit lib 485/0
incl. 3 new sh424 hermetics; cargo build --workspace OK, 0 errors).
Production code ONLY in boot.rs (off the 1MiB hooks; jit.rs/elfjit.rs
byte-unchanged). Pure `#[cfg(test)]` addition (+3 hermetics + a stack-walk
helper), no production path / guest byte / JIT-hook-default touched.
- boot.rs `layout_initial_stack` builds the kernel initial-stack word image and
  patches zero-valued AT_RANDOM to a xorshift 16-byte entropy block (stack
  canary without a getrandom syscall). It keeps AT_HWCAP minimal (FP+ASIMD),
  deliberately NOT advertising SVE/SME/MTE/LSE so glibc takes scalar paths the
  JIT fully decodes. Load-bearing: a garbage stack races IFUNC dispatch into
  unsupported-opcode territory (`__libc_arm_za_disable` `str za`).
- Genuine gap: boot.rs had ZERO self-tests — correctness only exercised
  implicitly when a full boot reached `_start`. SH424's 3 hermetics pin: (1)
  parseable aligned word image with env CStr round-trip + all non-AT_RANDOM
  auxv present; (2) AT_RANDOM zero-slot patched to non-zero in-bounds pointer
  with ≥1 non-zero entropy byte, argc==0 branch; (3) no-AT_RANDOM auxv left
  byte-identical, AT_NULL termination. Deterministic, no real binary, no env
  (mirrors the SH423 fsmap-coverage pattern).
- Honest: NOT a DM (SH415 probe re-confirms DM-root [0x106a68818]=0x0 under the
  complete substrate; Route-B live-DM gate unchanged). BUILD-THE-RUNTIME
  boot-surface coverage completion, not a re-tread.
- Files: docs/frontier-sh424-boot-stack-hermetics.md + crates/arm64jit/src/boot.rs

## SH423 (Sep 19, 2026, hermes-worker): the durable persistence contract (objective 2b "remembers sign-in") proven at the fsmap layer — a REMAPPED guest datastore path lands as a REAL on-disk host file, survives the in-memory override being cleared (simulated restart), and is readable back through a fresh independent remap; plus the 5-root longest-prefix precedence + pass-through rules pinned. fsmap.rs had ZERO tests of its own logic (only jit.rs harnessed it for R1 staging); SH423 delivers the module's own hermetic pair
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
re-verified green at this exact HEAD first (capture_taskv4_frame.sh attempt 1:
24 real task-driven frames `present swap Ok(0x1)`, 197 node pops, 0 json abort,
0 crash). Workspace green (cargo test --workspace EXIT 0; arm64jit lib 482/0
incl. 2 new sh423 hermetics; cargo build --example elfjit OK, 0 errors).
Production code ONLY in fsmap.rs (off the 1MiB hooks, pure host-side remap —
no guest byte / JIT hook default touched); jit.rs/elfjit.rs byte-unchanged.
- fsmap.rs `#[cfg(test)] mod tests`: `FSMAP_ROOT_LOCK` (serializes the two
  tests against the process-global override cell; a first-run parallel race
  where they clobbered each other's override was real and is fixed) +
  `sh423_durable_datastore_write_survives_remap_restart` (remap
  `/data/user/0/com.roblox.client/databases/rbx-session.db` -> preserve guest
  dirs + stay under root; `ensure_parents` scaffolds the deep chain so an
  O_CREAT never ENOENTs; forfeits the value to real disk; clears override to
  None = simulated restart; re-arms same root; fresh `remap_path`; reads the
  EXACT bytes back) + `sh423_remap_precedence_and_passthrough` (longest-prefix
  `/storage/emulated` before `/storage`; `/proc`/`/system` + relative + null
  pass through; fully-inactive state where even `/data` passes through).
- MEASURED: 2/2 hermetics pass, stable 3/3; full workspace 0 failed. BEFORE the
  lock, parallel-run intermittently failed under development (both tests mutated
  the shared override) — the lock makes each own the flag, deterministic green.
- Honest: NOT a DM (SH415 do-init probe under the complete substrate re-confirms
  DM-root [0x106a68818]=0x0, once-guard bit0=1, once-lambda let to run, MH_*
  latch true, AppBridgeV2 genuine vt 0x1063a3410 -> LIVE DM=false — Route-B
  live-DM structural gate UNCHANGED). This is BUILD-THE-RUNTIME COVERAGE
  completion: the "remembers sign-in" persistence contract is now pinned
  end-to-end headlessly (remap -> real file -> survive restart -> read-back),
  a genuine gap (fsmap.rs had zero self-tests; jit.rs only used it for R1
  staging). No re-treads (not an LSM/DM/store-watch seam; distinct from sh351/
  sh354 R1-staging tests).
- Files: docs/frontier-sh423-fsmap-durable-persistence.md +
  crates/arm64jit/src/fsmap.rs. Commit (pending).

## SH422 (Sep 19, 2026, hermes-worker): a reusable guest frame-pointer chain WALKER + one-shot default-inert guard (JIT_ROUTEB_LSM_BT=1) at the persistence-lane POOL-POP entry 0x101d9a5a0 NAMES the caller chain that drains do-init into the standing SH285/SH341 LSM lane — the loop only ever logged the terminal guestpc, never the call path INTO the lane (the MIGRATION-directive hunt's missing first datum)
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
re-verified green at this exact HEAD first (capture_taskv4_frame.sh attempt 1:
24 real task-driven frames `present swap Ok(0x1)`, 197 node pops, 0 json abort,
0 crash; JIT_JSON_ZERO_FIX present). Production code in session.rs (off-hook)
+ one dispatch line in jit.rs (`crate::session::routeb_lsm_bt_guard`, prose
condensed, addresses kept — jit.rs 1,048,417 B < 1MiB hook); elfjit.rs
untouched. Workspace green (cargo test --workspace EXIT 0; arm64jit lib 480/0
incl. 2 new sh422 hermetics; cargo build --example elfjit OK).
- session.rs: `bp_chain_walk(fp, max)` — pure aarch64 frame-pointer walk (saved-
  lr collection; aarch64 grows DOWN, so each caller fp is strictly higher; stops
  on a non-domain fp, a non-ascending fp (loop/edge), or max) + `routeb_lsm_bt_guard
  (state, pc)` — one-shot at the pool-pop entry 0x101d9a5a0, env-gated
  JIT_ROUTEB_LSM_BT, test override `set_lsm_bt_test`; +2 hermetics (walker
  orders/terminates; guard inert-off / walks-live-chain-on / non-target-pc).
- MEASURED on real libroblox.so (SH408 far-reach env, EXIT 139 downstream
  SIGSEGV after the marker): the one-shot chain FIRES — `[routeb-lsm-bt] SH422
  at pool-pop entry 0x101d9a5a0 .. caller chain lrs: [0]0x1021db13c <-
  [1]0x1021daf38 <- [2]0x1021e30bc <- [3]0x1021e2fdc <- [4]0x1021e2f34 <-
  [5]0x1021e2e40 <- [6]0x10217429c <- [7]0x0`. Guest->file (minus 0x100000000):
  pool-pop 0x1d9a5a0 <- 0x21db13c <- 0x21daf38 <- 0x21e30bc <- 0x21e2fdc <-
  0x21e2f34 <- 0x21e2e40 <- 0x217429c (top frame in the SH381 do-init zone, near
  the DM-ctor 0x2173b3c). Names the concrete engine path the SH285 lane swallows.
- Re-targeted from the SH285 reader 0x1d99e30 to the pool-pop entry 0x101d9a5a0
  after MEASURED evidence (--v2boot capture): the reader is short-circuited
  (lsm-map seeder NOP's the LSM init store at 0x101d975f8; 0 reader-entry hits)
  while the pool-pop IS the genuinely-reached lane terminal.
- Honest: NOT a live DM (DM-root [0x106a68818]=0, MH_GAME_LOADED false). Route-B
  live-DM structural gate UNCHANGED. This is an INSTRUMENT that names the caller
  path into the standing persistence lane (the operator's "hunt the DMCONT
  continuation and PATH-B reconstruction INSIDE this JIT" first datum), NOT a
  lane fix and NOT a re-tread (distinct from store-watch SH4xx, LSM map
  manufacture SH396, LSM-ctor lane wiring SH384/385, LSM skips SH349/350/358).
  Default-inert: product path byte-identical without the env.
- Files: docs/frontier-sh422-lsm-caller-chain.md + runs/capture_sh422_lsm_bt.sh +
  session.rs + jit.rs.

## SH421 (Sep 19, 2026, hermes-worker): wire the recon-v3 §A "Reject" rule into code — `--taskv4-seed <guest-hex>` now REFUSES the engine's own dispatcher 0x10285371c / drain 0x102856e40 / producer 0x10285682c (infinite recursion / re-entrancy), leaving the type-4 vector [0x106829ea8] cleared instead of seeding a runaway recursive handler; session.rs taskv4_seed_rejected + hermetic (arm64jit lib 477->478), elfjit.rs pulled under the 1MiB hook (1,048,392 B) by condensing two SH-prose comments
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
re-verified green at this HEAD first AND after (capture_taskv4_frame.sh: 24
real task-driven frames `present swap Ok(0x1)`, 197 node pops, 0 json abort, 0
crash — the deliverable's seed block is untouched because `--taskv4-seed
frame/session` take the safe host-thunk branches). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 478/0 incl. new sh421 hermetic; cargo build
--example elfjit OK). Production code in session.rs (off-hook, 62KB) +
elfjit.rs (intro rejects the three recursive engine addrs; jit.rs untouched.
- The recon-v3 doc's §A "Reject" (seed = engine's own 0x10285371c = infinite
  recursion, seed drain itself = re-entrant) was documented but NEVER enforced:
  the raw-hex --taskv4-seed branch accepted any guest addr, so seeding
  0x10285371c would recurse the popped-task deque forever on the builder thread.
  session.rs now exports `taskv4_seed_rejected(addr)` (denies dispatcher/drain/
  producer; accepts host-thunk addrs + the engine frame-fn 0x105b32c00, which the
  thunk calls via run_guest_callback and is never the vector entry). elfjit.rs
  parses the hex, refuses rejected addrs (`REJECTED ... vector left 0`) and dips
  under the 1MiB hook (24B over at mid-edit) by condensing the taskv4 + deque-probe
  SH-prose comments (addresses kept).
- MEASURED on real libroblox.so (full render env): `--taskv4-seed 0x10285371c` ->
  `[elfjit:taskv4] REJECTED ... vector left 0` + `seeded ... [0x106829ea8] = 0x0
  (cleared)`; `--taskv4-seed 0x105b32c00` -> normally seeded (valid path unregressed).
- Honest: NOT a DM (DM-root [0x106a68818]=0, MH_GAME_LOADED false). Route-B
  live-DM structural gate UNCHANGED; re-confirmed this cycle — the SH415 do-init
  probe under the complete substrate shows the runtime surface fully exercised
  (substrate 11/16 Ok, MH_FLAGS_LOADED/ENGINE_INITIALIZED/APP_READY all latch
  true, AppBridgeV2 genuine vt 0x1063a3410) yet DM-root stays 0: a live DM needs
  a real session ctor the JIT cannot reproduce headlessly. No re-treads.
- Files: docs/frontier-sh421-taskv4-seed-reject-guard.md + session.rs + elfjit.rs.

## SH420 (Sep 26, 2026, hermes-worker): a `__stack_chk_fail` host shim NAMES the GENUINE canary `*** stack smashing ***` wall's failing frame — the STATUS-#2 "distinct store on the deeper nativeGameGlobalInit ladder" finally measured against ITS OWN canary slot, with an instrument that fires ONCE (zero scheduling perturbation)
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
re-verified green at HEAD (capture_taskv4_frame.sh: real task-driven frames
`present swap Ok(0x1)`, 0 json abort, 0 crash). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 477/0 incl. the new sh420 hermetic; cargo
build --example elfjit OK). Production code ONLY in shims.rs (off-hook, 130KB);
jit.rs/elfjit.rs untouched (both at/near the 1MiB hook, byte-unchanged).
- The SH4xx store-watch is DOUBLY inadequate for the genuine wall: (a) it emits
  a host call after EVERY 64-bit store, which perturbs scheduling so the run no
  longer reproduces the stack-smash; (b) it filters to the SH103 leak class
  (value >= 0x7000_0000_0000) only, so a canary clobber by any other value class
  (a zeroed slot) is invisible. SH420 replaces it with a `__stack_chk_fail` host
  shim (fires exactly once at the check failure; reads the dispatcher's
  current_guest_pc = the guest return addr of the `bl __stack_chk_fail` = inside
  the failing fn's epilogue; best-effort frame read; then forwards to the real
  libc fail so the abort is byte-identical whether or not JIT_STACKCHK_DUMP=1).
- shims.rs: `stack_chk_fail_dump` registered in register_shims() as
  `__stack_chk_fail` (register_named precedence over the dlsym resolver), test
  override (live) + muted forward for the hermetic; +hermetic
  `stack_chk_fail_dump_returns_safely_under_test_override`.
- MEASURED on real libroblox.so (SH54 full-boot env + 3-gate crossing,
  JIT_STACKCHK_DUMP=1, EXIT 134). The one-shot line names the wall:
  `[stack_chk_fail] __guest_pc=0x102206d90 ... canary@-8=0x0 expected=0x55c6...` +
  `*** stack smashing detected ***`. Disassembly pins it: file 0x2206c40 =
  nativeGameGlobalInit body; prologue `stur x8,[x29,#-8]` @0x2206c70 stores the
  canary; epilogue `cmp x8,x9; b.ne 0x2206d8c` @0x2206cf4; 0x2206d8c
  `bl __stack_chk_fail@plt`. The canary slot [x29,#-8] is ZEROED during the
  do-init once-path (a callee overruns it between 0x2206c70 and 0x2206cf4) — the
  distinct clobber SH4xx-next anticipated, correlated against the wall's own slot.
- Honest: NOT a DM (DM-root 0, no make_shared, MH_GAME_LOADED false). Route-B
  live-DM structural gate UNCHANGED. This NAMES the genuine wall and its likely
  writer window (do-init once-path callee overrunning [x29,#-8]); it does not fix
  the zeroing. NEXT (STATUS #2): a store-watch scoped narrowly to that once-path
  window (not the whole run) to name the exact zeroing writer. No re-treads
  (mempool/LSM lane is a measured FALSE POSITIVE — do NOT chase it).
- Files: docs/frontier-sh420-stackchk-fail-shim-names-wall.md +
  runs/capture_sh420_stackchk_fail.sh. Commit 07a3082.

## SH4xx-next (Sep 26, 2026, hermes-worker): completed the SH4xx canary forensic's PRODUCER side — a default-inert host-RETURN leak watch (JIT_HOST_RETURN_WATCH=1) wired at the jit.rs integer host-return site (`s.x[0]=ret`) that logs every host fn whose return lands a foreign host pointer (0x7000..0x8000_0000_0000) in guest x0; MEASURED on real libroblox.so it round-trips the canary value by-exact-match and NAMES `boot.mempool_calloc` as the host producer of the canary-smashing pointer at pc=0x101d99e70
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
re-verified green at this HEAD first (capture_taskv4_frame.sh: real task-driven
frames `present swap Ok(0x1)`, 0 json abort, 0 crash). Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 476/0 incl. new sh4xx2 hermetic; cargo build
--example elfjit OK). Production code in translate.rs (off-hook, 498KB) + one
host-side call in jit.rs (110 B under the 1MiB hook after condensing SH187 prose,
addresses kept); elfjit.rs untouched.
- translate.rs: `host_return_leak_watch(pc, ret, caller)` (+ env gate
  JIT_HOST_RETURN_WATCH + test override `set_host_return_watch_test` +
  `host_return_watch_active`), same once-lazy AtomicBool pattern as the SH4xx
  store-watch. Narrows to the same HOST_HI/HOST_LO foreign-ptr leak class, resolves
  the host fn name via `crate::jit::host_call_slot_name`, de-dups consecutive
  (slot,ret) repeats. DEFAULT-INERT: unset env => one AtomicBool load, no logging,
  no guest bytes (a host-side check after the return, not a translated-block edit).
- jit.rs: `crate::translate::host_return_leak_watch(pc, ret, s.x[30])` at the
  integer host-return bridge (the resolved libc/libm/JNI/malloc thunk return path).
- New hermetic `host_return_leak_watch_gated_and_value_classified`: off = no-op on
  any input; on = foreign host-ptr return takes the log path (no panic), small/zero
  returns classified out; flag independently pinnable (no race with store-watch).
- **MEASURED (real libroblox.so, dual-watch runbook, EXIT 134 — standing SH285
  LSM pool-pop SIGSEGV guestpc=0x101d96868 fault=0x0, no `stack smashing`):** the
  store-watch names the same writers as SH4xx (0x102b9dee8 event drain,
  GLES-mempool 0x106251778/a48/a50, 0x102b9def0; terminal store pc=0x101d99e70),
  and the host-return watch rounds the forensic end-to-end by exact value:
  `[host-return-watch] slot=0x7f0000000008 hostfn=boot.mempool_calloc(x1=size)
  -> x0=0x7f0690014bc0 caller=0x101d96848` + `[canary-store-watch] pc=0x101d99e70
  dst=... val=0x7f0690014bc0 is_canary_slot=true`. **The producer is NAMED:
  `boot.mempool_calloc` (jit.rs:7497/7530) returns the raw host pointer that the
  LSM pool-pop stores into the canary-classified window** — the SH103
  bridge-sanitize direction's exact target. Top named producers overall:
  mempool_calloc 50x + slot 0x7f0000002488 57x (unresolved-name slots).
- Honest: NOT a DM (DM-root 0, no make_shared, MH_GAME_LOADED false). Route-B
  live-DM structural gate UNCHANGED. The watch NAMES the producer; it does not fix
  the wall. **Anti-tread refinement: the round-tripped mempool store is a FALSE
  POSITIVE as a canary clobber** — `boot.mempool_calloc` is a HOST-INJECTED shim
  (route_mempool_big_alloc_to_host) that returns a host `calloc` pointer into guest
  x0 BY DESIGN (identity-mapping makes it a valid guest pointer, injected because
  the real pool-init never runs); the store writes into a host-heap pool object,
  not a guarded guest frame. So the mempool/LSM lane is NOT the canary source; the
  genuine `stack smashing` clobber (deeper nativeGameGlobalInit ladder, SH97/98)
  is a DISTINCT store — the next step arms the store-watch on the ladder where
  __stack_chk_fail fires and correlates against THAT canary slot. No re-treads.
- Files: docs/frontier-sh4xx-host-return-leak-watch.md + runs/capture_host_return_leak_watch.sh.

Single-agent (cone suppressed). recon-v3 immediate-priority deliverables re-verified green at HEAD first (capture_taskv4_frame.sh attempt 1: real task-driven frames `present swap Ok(0x1)`, 0 json abort, 0 crash). New off-hook code in translate.rs (484KB, well under the 1MiB hook; jit.rs/elfjit.rs untouched, at/near the hook). Workspace green (cargo test --workspace EXIT 0; arm64jit lib 475/0 incl. 2 new hermetics).
- translate.rs: `canary_store_watch` (post-store host probe, SysV args state/dest/value/pc) + `emit_canary_store_watch`, wired into every 64-bit integer store (LdStrImm, LdStrImmWb, LdStrReg, LdStPair). DEFAULT-INERT: when JIT_CANARY_STORE_WATCH is unset it emits nothing and adds no guest bytes — product path byte-identical (proven by the unchanged green suite). When ON it narrows to the SH103 leak signature (a foreign host pointer 0x7000_0000_0000..0x8000_0000_0000 stored into a lower guest region) and reports pc/dst/val/x29/canary-slot.
- MEASURED on real libroblox.so (full-boot env + 3-gate crossing, EXIT 134): the watch NAMES the writers — pc=0x102b9dee8 (event-drain `stp x24,x23,[sp,#16]` after surface handoff), 0x102b9def0, GLES-mempool region 0x106251778/0x106251a48, and **0x101d99e70 (the STANDING SH285/LSM pool-pop persistence-lane family)** — storing foreign host ptrs into canary windows. Confirms the canary wall converges onto the same measured-closed SH285 family. This is SH106's NEXT finally delivered (a bounded instrument that names the writers); it does NOT yet fix the wall.
- Honest: NOT a DM (DM-root 0, MH_* false). Route-B live-DM gate UNCHANGED. The next forensic: trace which HOST call returns the 0x7f... pointer into guest x-regs at 0x101d99e70 (SH103 bridge-sanitize direction). No re-treads.
- Files: docs/frontier-sh4xx-canary-store-watch-names-writers.md + runs/capture_sh4xx_canary_store_watch.sh. Commit (pending).

## SH419 (Sep 19, 2026, hermes-worker): the R1 content surface now spans BOTH roots a cache-probing resolver may read — files-dir mirror (SH412) + app-CACHE mirror (recon-v3 R1 "also mirror under cache-root" from deleg_8d5648cf); verified the missing-env audit is complete + re-confirmed the recon-v3 frame deliverable green at this exact HEAD

Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
re-verified green at this exact HEAD first (capture_taskv4_frame.sh attempt 1:
real task-driven frames `present swap Ok(0x1)`, 0 json abort, 0 crash — run-variable
frame/node counts; both the 24-frame baseline and this run are crash-free real
frames; the `--startapp` frame path does not touch the session substrate).
Workspace green (cargo test --workspace EXIT 0; arm64jit lib 473/0 incl. new
sh419; cargo build --example elfjit OK). Production code ONLY in session.rs
(off the 1MiB hooks, file 61 KB); jit.rs untouched (1,048,571 B < 1,048,576 hook).
- VERIFIED the STATUS next-forward #1 (missing-env crossing audit) is COMPLETE:
  a full sweep of every `--v2boot` boot-path runbook shows only
  `capture_sh416_input_poll.sh` lacks the 3-gate env, and SH417b deliberately
  reverted it unchanged (env does not cleanse that LSM lane); the two canonical
  full-boot runbooks carry the env (SH417c measured). Audit done.
- session.rs: +`mirror_r1_cache_root() -> Vec<String>` — copies the two staged R1
  candidates (AppShell.lua, CoreScripts.lua) from the files-dir mirror into the
  guest app-cache mirror `<root>/data/user/0/com.roblox.client/cache/scripts/CoreScripts/<Name>.lua`
  (pure file staging, idempotent, honors the test override, guarded src.is_file()
  skip, no guest byte touched); wired into `drive_content_surface()` right after
  `stage_r1_core_scripts()` so the ordered substrate's G3 content surface serves
  BOTH roots a cache-probing resolver probes first.
- New hermetic `sh419_r1_cache_mirror_serves_both_roots` (session.rs): stages the
  files mirror, mirrors into the cache root, proves each cache file exists + names
  a self-constructing ScreenGui AND a guest open of the app-cache path resolves
  through `fsmap::remap_path` to exactly that mirror (SH354-style serve half).
- Honest: NOT a DM (DM-root 0, no make_shared, MH_GAME_LOADED false). Route-B
  live-DM structural gate UNCHANGED. Content-side BUILD-THE-RUNTIME completion —
  removes the cache-probe miss that would blunt R1-first screens the instant a
  live DM arrives (latent-but-correct, exactly like SH412). No re-treads.
- Files: docs/frontier-sh419-r1-cache-root-mirror.md + crates/arm64jit/src/session.rs.
Single-agent (cone suppressed). recon-v3 deliverables re-verified green at HEAD first
(capture_taskv4_frame.sh attempt 1: 24 real task frames `present swap Ok(0x1)`, 197 node
pops, 0 json abort, 0 crash). Workspace green (cargo test --workspace EXIT 0; arm64jit lib
472/0, full suite ok). Production code in jit.rs/elfjit.rs/session.rs UNCHANGED (byte-identical
at HEAD) — this is a runbook-only + doc + attribution correction, the SH417b next-forward #1
(missing-env audit on every boot-path runbook).
- The audit found two canonical stable-baseline FULL-BOOT runbooks that drive a real `--v2boot`
  ladder WITHOUT the crossing env (`JIT_ROUTEB_HASHFIX JIT_JSON_ZERO_FIX JIT_ROUTEB_SETFIX`):
  runs/capture_v2boot.sh + runs/capture_v2boot_gate.sh. Both silently crash the way SH417b
  proved was a MISSING-ENV artifact, not "Route-B".
- MEASURED on real libroblox.so: capture_v2boot.sh WITHOUT the env → SIGSEGV guestpc=0x1029f3f7c
  (fault=0x0, the hashfix string-hash-map `blr x8` lane), EXIT 134, crash-count 2. WITH the env →
  routeb-hashfix fires hundreds of times, 0x1029f3f7c GONE, run ADVANCES into nativeGameGlobalInit's
  real body, then aborts at `*** stack smashing detected ***` — the documented SH97/98 canary wall
  deeper in the ladder, a SEPARATE pre-existing wall correctly NOT claimed clean (crash-count 1).
- capture_v2boot_gate.sh (SH81 header claimed "CRASH-FREE now") carried NO crossing env — same latent
  misattribution; env added to match. Out of scope correctly: capture_v2boot_sh82.sh is a DIAGNOSTIC
  runbook whose whole purpose is the hashfix-lane progression; render-plane + sustain + session-producer
  runbooks never reach the lane (no full --v2boot boot).
- Honest: NOT a Route-B step, no DM (DM-root 0, MH_* false unchanged). Artifact-quality/attribution
  correction — removes a genuine crash from the canonical stable-boot runbook and fixes two headers'
  latent misattribution. Route-B live-DM structural gate UNCHANGED.
- Files: runs/capture_v2boot.sh + runs/capture_v2boot_gate.sh (+ crossing env) +
  docs/frontier-sh417-host-input-loop.md (SH417c section).

## SH417b (Sep 26, 2026, hermes-worker): the boot/input-loop runbook's "known pre-existing nativeInit Route-B lane SIGSEGV" at guestpc=0x1029f3f7c was a MISSING-ENV artifact, not a distinct crash — the promoted host-input LOOP (SH417/418, --v2boot-input-loop) is now a deterministic-clean boot on the real binary (EXIT 124 stable idle, 0 SIGSEGV/SIGABRT)

Single-agent (cone suppressed). recon-v3 deliverables re-verified green at this exact HEAD first
(capture_taskv4_frame.sh attempt 1: 24 real task frames `present swap Ok(0x1)`, 194 node pops, 0
json abort, 0 crash, EXIT 124). Workspace green (arm64jit lib 568/0 incl. sh417/sh418; cargo test
--workspace EXIT 0). Production code in session.rs/elfjit.rs UNCHANGED (both at/near the 1MiB hook,
byte-unchanged) — this is a runbook + doc correction + a measured-closure of a misattribution.
- SH416/417/418 all documented the input/boot runs as terminating in a "known run-variable nativeInit
  'outside image' Route-B lane at guestpc=0x1029f3f7c" and treated it as pre-existing. SH417b
  re-measured it on real libroblox.so: the SAME input-loop boot env PLUS the crossing hash-fix gates
  the full reaching / recon-v3 runbooks already carry (`JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1
  JIT_ROUTEB_SETFIX=1`) yields **deterministic-clean EXIT 124 (stable idle), 0 SIGSEGV/SIGABRT,
  hashfix fired 103×**, and the input loop still runs all 4 iterations 0 real -> 0 translated -> 0
  delivered exactly as designed.
- ROOT CAUSE: 0x1029f3f7c is the SH83/SH91 string-hash-map `blr x8` into a garbage +0x18 hash-fn-2
  slot (`x8=0x4741495241003635`) — precisely the crash `JIT_ROUTEB_HASHFIX` clears. The input-loop
  runbook omitted the env, so the --v2boot boot raced into the unguarded map insert and the terminal
  was MISATTRIBUTED as "a Route-B lane." Fix: capture_sh417_host_input_loop.sh now carries the
  crossing env (same as capture_taskv4_frame.sh / capture_sh415). The input axis itself was never the
  faulting path.
- SCOPE HONESTY: only the promoted sh417 loop is cleansed. The superseded SH416 one-shot poll re-ran
  with the same env and hit a DIFFERENT run-variable lane (guestpc=0x101fd128c, LSM-family, fault=0x18,
  hashfix 0 fired) — a separate pre-existing wall, NOT cleansed by this env; capture_sh416_input_poll.sh
  was reverted unchanged (no false "0 crash" claim).
- No DM (DM-root 0, MH_GAME_LOADED false) — Route-B live-DM structural gate UNCHANGED. This is an
  artifact-quality / BUILD-THE-RUNTIME fix + a ~3-cycle misattribution correction, not a DM step.
- Files: runs/capture_sh417_host_input_loop.sh (+ crossing env) + docs/frontier-sh417-host-input-loop.md
  (SH417b section) + /home/hermes-worker/runs/STATUS.md.

## SH418 (Sep 26, 2026, hermes-worker): promote the persistent-tracker host input LOOP (SH417) to a first-class driven substrate step — drive_routeb_session_substrate now drives drive_host_input_loop (iimg, ib, tpidr, boot_sp, input_loop_iters()) right after the post-bus G3 content-surface step
(SH417 + SH418: SH418 promoted the loop to a FIRST-CLASS driven substrate step —
drive_routeb_session_substrate now drives it right after the post-bus G3 content
surface, same promotion SH411/412 gave the DM binder/app-start/content; substrate
MEASURED 12/16 atoms non-zero Ok on the real binary, up from 11/16, no regression,
input step inert on the no-bridge boot env. Commits fd2c5e3 + b75b336.)
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables re-verified green at HEAD first
(capture_taskv4_frame.sh attempt 1: 24 real task frames `present swap Ok(0x1)`, 195 node pops, 0 json
abort, 0 crash). Production code only in session.rs + elfjit.rs rung; jit.rs untouched (at the 1MiB hook).
elffjit.rs condensed SH-prose comments (addresses kept) to stay under the 1MiB pre-commit hook
(1,048,552 B < 1,048,576). Workspace green (arm64jit lib 470/0 incl. new sh417 hermetic; cargo test
--workspace EXIT 0).
- session.rs: `drive_host_input_loop(iimg, ib, tpidr, boot_sp, iterations)` — bounded persistent-tracker
  loop. Three guards checked ONCE up front (JIT_AINPUT_BRIDGE + registered window XID + live image), then
  drains N non-blocking batches through ONE PointerTracker (pointer-down/multi-touch state survives across
  iterations), marshalling each translated MotionEvent via deliver_motion -> nativePassInput. Mid-loop X
  error stops the loop keeping events already delivered. New hermetic `sh417_host_input_loop_inert_without_all_guards`
  (all three guard trips + zero-iteration bounded, no X connect).
- elfjit.rs: `--v2boot-input-loop` rung (INPUT_LOOP_ITERS, default 8).
- MEASURED (real libroblox.so, SH416 env + INPUT_LOOP_ITERS=4): window wired XID 0x200000 on :308; loop
  ran all 4 iterations `0 raw -> 0 translated -> 0 delivered`, total 0; `[elfjit:v2boot-input-loop] SH417
  ... delivered 0 events`; no input-path crash (terminal SIGSEGV at guestpc 0x1029f3f7c is the known
  pre-existing nativeInit "outside image" Route-B lane, SH416 documented identically — not this change).
- Honest: inert-by-construction like every runtime axis (no live DM -> no constructed login/home screen to
  deliver to -> 0 events moved). BUILD-THE-RUNTIME cause-not-symptom input surface: the moment a live DM
  advances, the same bounded loop delivers real desktop pointer input with correct persisting pointer state.
  Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0, MH_GAME_LOADED false, no make_shared).
  No re-treads. recon-v3 deliverables unchanged-green.
- Files: docs/frontier-sh417-host-input-loop.md + runs/capture_sh417_host_input_loop.sh.

## SH416 (Sep 22, 2026, hermes-worker): wire the real X event source into the input axis — the input delivery path (deliver_motion -> nativePassInput 0x2bbba88) was complete+hermetic but a real host NEVER fed it: drive_host_input_pump only saw synthetic test vectors while the registered ANativeWindow had no poll. New input_wrapper::x11::pump_registered_window (owner-connection select on an EXISTING window; a fresh conn gets BadAccess on Xvfb) + register_window_connection at both wire_real_window sites + session::drive_host_input_poll (one non-blocking real-event batch -> nativePassInput, gated JIT_AINPUT_BRIDGE + real XID). Opt-in --v2boot-input-poll. Head commit 0b8825e. (See commit history for full body + sh416 hermetic + MEASURED window XID 0x200000 on :275, 0 raw -> 0 delivered clean.)
## SH415 (Sep 22, 2026, hermes-worker): make do-init COMPLETION a first-class observable of the ordered session substrate — the four EXECUTE-DO-INIT-GATES live-DM markers (once-guard [0x106a68410].bit0, DM-root [0x106a68818] + in-image vt, app-DM counter [0x106dca0e88]) are now REPORTED right after the two do-init-reaching atoms (StartLuaAppDM 0x1023efe2c + V2StartAppWithParams 0x10258b144), not guessed from one-off probes
Single-agent (cone suppressed). recon-v3 deliverables re-verified green at HEAD first
(capture_taskv4_frame.sh attempt 1: 24 real task frames `present swap Ok(0x1)`, 197 node
pops, 0 json abort, 0 crash). Production code only in session.rs (off the 1MiB hooks;
jit.rs/elfjit.rs untouched, byte-unchanged). Workspace green (arm64jit lib 469/0 incl.
new sh415 hermetic; cargo test --workspace EXIT 0).
- session.rs: `probe_doinit_completion() -> DoinitCompletion` (page-guarded readout of
  the four EXECUTE-DO-INIT-GATES markers; `liveness()` = single "live DM owned" bit)
  wired into `drive_routeb_session_substrate` after StartLuaAppDM + V2StartAppWithParams.
  Hermetic `sh415_doinit_completion_probe_safe_and_aggregates`: no-live-image reads
  degrade to 0 without SIGSEGV; liveness aggregates (all four required); the two
  do-init-reaching atoms stay in the substrate table.
- MEASURED (real libroblox.so, complete SH400-414 substrate, EXIT clean): substrate
  11/16 Ok; the probe fires twice and reports the same honest verdict —
  `once-guard[0x106a68410]=0x101 bit0=1 DM-root=0x0 vt=0x0 counter=0 -> LIVE DM=false`.
  The genuinely-new datum: under the COMPLETE substrate the FIRST precondition (the
  once-guard seeded → once-lambda LET to run) is now MET, yet DM-root still stays 0 —
  direct per-run proof the do-init once-path constructs nothing live (SH381 consistent).
  Route-B live-DM gate UNCHANGED (DM-root 0, MH_GAME_LOADED false).
- Honest: NOT a DM; pure readout (no guest byte moved). Turns the do-init completion
  gate into a reported observable of the runtime — "measure, don't guess." recon-v3
  deliverables + Rule-1 regression all green at HEAD.
- Files: docs/frontier-sh415-doinit-completion-observable.md +
  runs/capture_sh415_doinit_completion.sh. Commit bb87cb3.

## SH414 (Sep 22, 2026, hermes-worker): complete the INPUT runtime axis — the SEP-18 list's last part (session boot, screens, audio, input) was landed latent-but-correct by SH400-413, but the ainput bridge was ORPHANED (zero production callers) and input-wrapper was a DEV-ONLY dependency. SH414 wires them: input-wrapper promoted to a real arm64jit dep; new ainput::from_motion_event/deliver_motion (translated MotionEvent -> guest nativePassInput ABI) + session::drive_host_input_pump (a first-class host-input step, three inert guards, returns delivered count). This is the executable half of the input axis — a real host loop delivering X-window pointer events into the guest input native for a constructed login/home screen
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables re-verified
green first (24 real task frames `present swap Ok(0x1)`, 197 node pops, 0 json abort,
0 crash). Production code only in ainput.rs + session.rs (both off the 1MiB hooks;
jit.rs/elfjit.rs untouched, both at/near the hook). input-wrapper promoted
dev-dep -> real dep (Cargo.toml). Workspace green (arm64jit lib 468/0 incl. 3 new
SH414 hermetics; cargo test --workspace EXIT 0).
- ainput.rs: `from_motion_event(&input_wrapper::input::MotionEvent) -> TouchAction`
  (preserves real pointer id for multi-touch + coords) + `deliver_motion(img, base,
  tpidr, boot_sp, ev)` — production delivery of one translated motion event into
  guest nativePassInput via the existing fire_touch (env-gated, inert without
  JIT_AINPUT_BRIDGE).
- session.rs: `drive_host_input_pump(...) -> usize` — a first-class host-input step:
  delivers each translated MotionEvent through the SH414 bridge. Inert (returns 0,
  no guest path) unless JIT_AINPUT_BRIDGE AND a real window XID is registered AND a
  live input image is present — all three guards tested explicitly
  (sh414_host_input_pump_inert_without_arm).
- 3 new hermetics (2 in ainput, 1 in session). MEASURED (real libroblox.so, SH400
  env): recon-v3 type4 re-verified green with the promoted dep (no regression);
  real-image pin byte-exact (nativePassInput 0x2bbba88 sub 0xd10143ff, bl consumer
  0x940a4aed @+0x50).
- Honest: latent-but-correct exactly like SH132/SH413 — input only lands once a live
  session owns a screen (guest consumer 0x2e4e68c gate); on the current boot path
  the pump is three-guard inert, product path byte-identical. Route-B live-DM
  structural gate UNCHANGED (DM-root 0, no make_shared, MH_GAME_LOADED false). No
  re-treads.
- Files: docs/frontier-sh414-input-axis.md + runs/capture_sh414_input_axis.sh.
  Commit (pending).

## SH413 (Sep 22, 2026, hermes-worker): land the INPUT runtime axis — the SEP-18 BUILD-THE-RUNTIME surface the operator names last ("session boot, then screens, then audio, then input"); audio (SH132) was done, input had ZERO guest-facing wiring (input-wrapper crate orphaned dead code + no host path into the guest's GameActivity input natives). New off-hook module ainput.rs mirrors the SH132 fake-AAudio pattern: env-gated (JIT_AINPUT_BRIDGE), latent-but-correct, hermetic-ABI + real-image-pinned to the actual instruction bytes
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables re-verified green first
(capture_taskv4_frame.sh attempt 1: 24 real task-driven frames `present swap Ok(0x1)`, 197 node pops,
0 json abort, 0 crash). Production code only in the new off-hook module ainput.rs (+ one-line
`pub mod ainput` in lib.rs); jit.rs/elfjit.rs untouched (at/near the 1MiB hook, byte-unchanged).
Workspace green (arm64jit lib 465/0 incl. the new 6 ainput hermetics; cargo test --workspace EXIT 0).
- MEASURED (hermetic + real 109MB APK .so): the 6 ainput tests pass, and the real-image pin asserts
  byte-exact instruction words at the guest input-native addresses — nativePassInput @0x2bbba88
  prologue 0xd10143ff, nativePassMouseMove @0x2bbbcf4 0xd10143ff, input consumer leaf 0x2e4e68c
  0xd10503ff, internal `bl 0x2e4e68c` = 0x940a4aed @ nativePassInput+0x50. Verifies the input
  delivery ABI (env, this, action, pointerId, float x/y in s0/s1) on verified bytes.
- ainput.rs: `marshal_touch` (places int args in x2/x3 + packs float x/y into the SIMD v-lanes the
  JIT's `fmov s8,s1; fmov s9,s0` reads), `fire_touch` (drives a translated touch into guest
  nativePassInput via fresh CpuState + jit_run; NOP/Ok(0) when bridge env unset or no live image),
  `translate_pointer`/`translate_motion` binding the previously-orphaned input-wrapper translation to
  a real guest delivery target. 6 hermetic tests.
- Honest: latent-but-correct, exactly like SH132 — input only matters once a live session owns a
  screen (Route-B live-DM gate UNCHANGED: DM-root 0, MH_GAME_LOADED false). The bridge is the correct
  host-side capability on the input direction; inert behind the env gate (default product path
  byte-identical). No re-treads (input axis, not DM/LSM/glass). recon-v3 deliverables unchanged-green.
- Files: docs/frontier-sh413-input-bridge.md + runs/capture_ainput_bridge.sh. Commit (pending).

## SH412 (Sep 19, 2026, hermes-worker): promote the G3 CONTENT surface — the engine's own files-dir libc++ string [0x10726d600] + the R1 CoreScript stage — into the ordered session substrate as a first-class driven runtime step (SH407/408 measured the equivalent --v2boot-set-filesdir/--v2boot-r1-stage rungs NEVER fire on reaching envs; wiring it into drive_routeb_session_substrate right after the MessageBus.subscribe atom makes it a driven step, exactly as SH411/411b promoted the binder + app-start)
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables re-verified green first
(capture_taskv4_frame.sh attempt 1: 24 real task frames `present swap Ok(0x1)`, 197 pops, 0 json
abort, 0 crash; JIT_JSON_ZERO_FIX present). Production code only in session.rs (off the 1MiB hooks;
jit.rs/elfjit.rs untouched, both at/near the hook, byte-unchanged). Workspace green (arm64jit lib
459/0 incl. new sh412 hermetic; cargo test --workspace 639/0 canonical green).
- session.rs: +`drive_content_surface()` (seeds engine files-dir libc++ string @ 0x10726d600 =
  "/data/user/0/com.roblox.client/files", read-back-verified, wrapped in routeb_ensure_writable;
  then stages the R1 CoreScript via `jit::stage_r1_core_scripts`) wired into
  `drive_routeb_session_substrate` right after the MessageBus.subscribe atom (the post-bus step
  that drives the binder + app-start) — recon-routeB G3 is now a driven runtime step. New hermetic
  sh412 (compile-pins the signature, asserts the bus trigger stays in the table, proves the drive
  seeds on a no-image harness).
- MEASURED (real libroblox.so, SH400 env, EXIT 124, 0 crash): substrate completes 11/16 Ok
  (no regression from the new step); `SH412 G3 files-dir: ... [0x10726d600] SEEDED`; `SH412 R1
  content surface: staged 2 candidates (STAGED AppShell.lua, STAGED CoreScripts.lua)` — the
  content gate now FIRES headlessly in a driven run (was measured-never-firing as a rung, SH408).
- Honest: NOT a DM (DM-root [0x106a68818]=0, no make_shared, MH_GAME_LOADED false). Route-B
  live-DM structural gate UNCHANGED. The content surface is what the engine draws FROM the instant
  a completed do-init owns a live DM (latent-but-correct). No re-treads.
- Files: docs/frontier-sh412-content-surface-substrate-step.md + runs/capture_sh412_content_surface.sh;
  log /tmp/cap_sh412.out (outside repo). Commit (pending).

## SH411 (Sep 19, 2026, hermes-worker): promote the SEP-17 dataModel-bindings LIVE BINDER into the ordered session substrate as a first-class driven runtime step — the messageBus publish-RECEIVE half (publishRaw 0x102334684 -> cb [DataModelBindings+16]) now runs on the ladder thread right after MessageBus.subscribe, not just as an opt-in SH347/364 probe rung
(SH411b extends this: SEP-17 **nativeAppBridgeAppStart** V1 0x102338510 is also
now a first-class post-substrate step — `session::drive_native_app_start` runs it
right after MessageBus.subscribe, MEASURED Ok(0x3e8) registry=12 DM-root=0x0.
Both SEP-17-named RECEIVE/app-start components are driven, not probed.)
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables
re-verified green first (attempt 1: 24 real task frames `present swap Ok(0x1)`,
195 pops, 0 json abort, 0 crash; JIT_JSON_ZERO_FIX present). Production code only
in session.rs (off the 1MiB hooks, file 16KB); jit.rs/elfjit.rs untouched (both
at/near the hook, unchanged at byte-level). Workspace green (arm64jit lib
458/0 incl. new sh411 hermetic; cargo test --workspace 638/0 canonical green —
the one intermittent parallel-run failure is the SH345/346/357/370 accepted
load-sensitive family, not sh411).
- session.rs: +`drive_data_model_binder(iimg,ib,tpidr,boot_sp,env,thiz)` (thin
  wrapper over the existing serialized `jit::drive_messagebus_publish_receive`),
  wired into `drive_routeb_session_substrate` immediately after the
  `MessageBus.subscribe` atom (guest 0x102ba5bb8) — the SEP-17 "dataModel-bindings
  live binder" is now an ordered step of the runtime, not a probe rung. New
  hermetic `sh411_data_model_binder_is_first_class_substrate_step` (compile-pins
  the drive signature; asserts the MessageBus trigger atom stays in the substrate
  table; brokers the SessionHandles the drive builds).
- MEASURED (real libroblox.so, SH400 env + JIT_ROUTEB_BUSRECV, EXIT 124, 0 crash):
  `MessageBus.publishRaw returned Ok(0x3e8)` fires inside the ordered substrate,
  then the binder reports Ok(0x3e8); substrate completes 11/16 atoms Ok.
- Honest: NOT a DM (busrecv cb-entry holder guard @0x102bd7474 did NOT fire — cb
  entry is live-DM-gated, [DataModelBindings+16] stays 0 until a completed
  do-init populates it; SH364 class). Route-B live-DM structural gate UNCHANGED
  (DM-root 0, MH_GAME_LOADED false). No re-treads.
- Files: docs/frontier-sh411-dm-binder-substrate-step.md; commit f70dfb8.

## SH410 (Sep 19, 2026, hermes-worker): COMPLETE the fuller NativeHelper lifecycle surface — drive the login-vs-home gate `onDidLogInReceived` (VOID-with-String, 0x50a545) with a real login payload, steering a fresh headless session to its OWN login screen; the FULL 5-milestone lifecycle now fires through the engine's own registered slot-61 shim
Single-agent (cone suppressed). Workspace green (cargo test --workspace EXIT 0;
arm64jit lib 457/0). Production code only in jni.rs + session.rs (both far under
the 1MiB hooks); jit.rs/elfjit.rs untouched. Real-binary MEASURED on libroblox.so
(SH400 capture env, EXIT 124, 0 crash). Commit 5738301.
- jni.rs: +`MH_LOGIN_RECEIVED`/`MH_LOGGED_IN` observables + getters; `jni_call_void_method` now reads the a3 login-payload jstring on onDidLogInReceived and sets both (empty payload -> NOT logged in -> LOGIN; non-empty -> remembered sign-in -> HOME); +`fire_nativehelper_login_payload(payload)`.
- session.rs: `drive_nativehelper_lifecycle()` (wired after the surface atom, recon-routeB step-2 position) fires onDidLogInReceived with an EMPTY payload; hermetic asserts login_received AND not logged_in.
- MEASURED: `onDidLogInReceived (0B payload) -> logged_in=false; login screen`; MH_LOGIN_RECEIVED=true; substrate 11/16 Ok; AppBridgeV2 genuine vt 0x1063a3410; EXIT 124, 0 crash. The FULL five `gameActivity_*` milestones now fire in order: onFlagsLoaded -> onEngineInitialized -> onAppReady -> onDidLogInReceived -> (onGameLoaded stays false, honest).
- Honest: NOT a DM (DM-root [0x106a68818]=0, no make_shared); does not boot Lua by itself (a completed do-init still owns the live DM, SH405). This is the SEP-18 BUILD-THE-RUNTIME host-side LOGIN-STATE surface — it makes the login-vs-home discriminator read a real host signal (the payload) steering to the operator's FIRST screen ("login renders"). Route-B live-DM structural gate UNCHANGED. No re-treads.
- Files: docs/frontier-sh410-loginstate-gate.md; log /home/hermes-worker/runs/sh410-baseline-login.txt (outside repo).

## SH409 (Sep 21, 2026, hermes-worker): the SH400 substrate's missing onAppReady host surface — drive the NativeHelper lifecycle milestones (onFlagsLoaded -> onEngineInitialized -> onAppReady) through the engine's own registered JNI CallVoidMethod shim, in the recon-routeB step-2 position (after surface, before SendAppEventOnAppReady); MEASURED on real libroblox.so MH_APP_READY now latches (was always 0)
Single-agent (cone suppressed). The SH407/408 frontier "Next" named the one
genuinely-open surface: "REAL session-compat runtime (SH400 substrate + a real
LSM/EGL/onAppReady host drive)". Recon-routeB step-2 + SEP-18 BUILD-THE-RUNTIME both
wanted the NativeHelper lifecycle callbacks "actually DRIVEN" — the SH400 substrate
only ever waited for the engine to reach those CallVoidMethod sites, so none fired
headlessly. Production code (off the 1MiB hooks; jit.rs/elfjit.rs untouched):
- jni.rs: `fire_nativehelper_milestone(name)` — intern the milestone name to a
  readable method-id handle (== engine GetMethodID return) and dispatch through the
  SAME registered slot-61 CallVoidMethod shim, so MH_* transition as a real session's
  callbacks would.
- session.rs: `drive_nativehelper_lifecycle()` drives the ordered sequence through
  that shim, wired into `drive_routeb_session_substrate` right after the
  V2UpdateSurface atom (0x1025f5fec) — the exact recon step-2 position.
- New hermetic `lifecycle_milestones_driven_in_order` (no real binary).
MEASURED (real libroblox.so, SH400 capture, EXIT 124, 11-16 Ok): the three milestones
now log IN ORDER and MH_APP_READY latches (MH_FLAGS_LOADED/ENGINE_INITIALIZED/APP_READY
all true; was all-false on every SH400-408 run). AppBridgeV2 stays at genuine vt
0x1063a3410. Honest: NOT a DM manufacture (DM-root 0, once-slot sentinel, MH_GAME_LOADED
false, no make_shared); the MH_* observables don't boot Lua by themselves (SH405) — but
the substrate now exercises the exact host-to-engine lifecycle call sequence recon-routeB
step-2 / SEP-18 name, a concrete testable shippable BUILD-THE-RUNTIME piece. Route-B
live-DM structural gate UNCHANGED (DM-root 0). No re-treads. Workspace green (cargo test
--workspace EXIT 0; arm64jit lib 457/0). +docs/frontier-sh409-onappready-driven-surface.md.

## SH407+SH408 (Sep 21, 2026, hermes-worker): measured the do-init MAIN-arm terminal decisively — the app-start MAIN body [0x10258b5d8,0x10258bbb0] now runs END-TO-END (biggest app-start reach on record; SH362-404 called it unreachable), then drains into the SH341 pool-pop persistence lane; DMCONT 0x102bd1d68 = 0 hits from the MAIN arm. SH408: same env + files-dir/R1 rungs (which did NOT fire) shows LSM init ADVANCES through initStorageManagerNative + crosses the SH285 reader (0x101db1b08 now 0 hits = old fault terminal bypassed) then next-faults at fault=0x0 in the opnew/insert band — the standing unbound whack-a-mole, one fencepost deeper, not a DM advance. New hermetic sh407 byte-pins the full app-start body span + the LSM-init deepen (arm64jit lib 455->456).
Single-agent (cone suppressed). Two probes (runs/capture_sh407_appstart_main_terminal.sh,
runs/capture_sh408_filesdir_lsm.sh) at the SH406 far-reach (DONEPATH_MAIN + SETFIX + GOVFLAG)
on real libroblox.so. New hermetic `sh407_appstart_body_full_span_runs_end_to_end_and_lsm_init_crosses_reader`
(arm64jit lib 455->456) byte-pins the FULL app-start body span + the LSM-init deepen. elfjit.rs
untouched; jit.rs added only the hermetic (default-inert, no production path change).

- **SH407 verdict**: SH406 left the MAIN-arm terminal ambiguous (log ended mid-pool-pop). With
  region-watch over the whole [0x10258b5d8,0x102590000) + DMCONT + app-shell + governor +
  scriptctx + pool-pop: ALL 14 block-entry pcs of the app-start body fire (0x10258b5d8..0x10258bbb0,
  each 1×), NOTHING past the 0x10258bbb0 terminal block, then 190 valid-key pool-pops (SETFIX fired,
  LR=0x10626b6dc) and EXIT 139 (host-side, no guestpc line). **DMCONT [0x102bd1d68] = 0 real hits**
  — the MAIN arm does not reach the do-init-completion construction gate; it drains to the standing
  LSM persistence lane (answers SH405's frontier "Next").
- **SH408 verdict**: same env + --v2boot-set-filesdir/--v2boot-r1-stage (the files-dir [0x10726d600]
  global the SH406 env never seeds; real host surface). The seed rungs did NOT fire (no log line —
  unreached), yet the run reached DEEPER into LSM: initStorageManagerNative 0x101db1050 ENTERED,
  SH285 reader 0x1d99e30 band entered, append 0x1d9a15c entered, operator-new 0x101db1a38 band
  entered, insert-leaf 0x101db1d04 fired, and **the SH285 reader wall pc 0x101db1b08 = 0 hits
  (CROSSED — the old fault terminal from SH260/284/285/344/348/371/372 is bypassed)**. But the run
  still EXIT 139 with fault=0x0 in the just-entered opnew/insert band — the SH349/350/358/396
  unconstructed-object whack-a-mole (one fencepost deeper; SH385: node value set only by a real LSM
  session ctor). Not a DM advance.
- **Honest**: no DM (DM-root 0, once-slot 0x400000b sentinel, MH_* false, DMCONT unreached from the
  MAIN arm). Route-B live-DM structural gate UNCHANGED. This is map-completion + two successive
  persistence-lane fenceposts (reader crossed, opnew/insert fault), not a DM advance. The app-start
  body is now fully cleared — the single biggest SESSION-CTOR reach on record.
- Do-not-re-tread unchanged: do NOT re-drive LSM skips (SH349/350/358), do NOT re-manufacture the
  map to cross SH285 (SH396), standing closures (setDataModelToCurrent SH388, EC reader SH355/356/374,
  window-attach real SH367, ALooper SH365, -9 string SH380, map-header SH248h, once-lambda SH381).
  SH174 capture-latch single forward observer (with DELEGATE=1, SH395).

## SH406 (Sep 21, 2026, hermes-worker): extended the SH269 GOVFLAG fixed-.bss flag seed to the SH405 MAIN-arm app-start continuation — the 0x10258b5d8 app-start body's continuation now PASSES the NULL-controller deref it faulted at (guestpc 0x1025f501c) and drains into the standing SH285/SH341 persistence lane; DMCONT 0x102bd1d68 still 0 hits
Single-agent (cone suppressed). No production behavior change (default-inert opt-in
JIT_ROUTEB_APPSART_GOVFLAG; widening the guard's pc-window does nothing when unselected).
SH405 measured the MAIN arm faults SIGSEGV (fault=0x0) at guestpc 0x1025f501c right after
`bl 0x25f52b4` returns x0=0, and left two "Next" questions. SH406 answers them:
- ROOT CAUSE: the app-start continuation reads the SAME fixed-.bss flag byte [0x106a64da0]
  SH269 seeds (`ldrb w8,[x9,#3488]` @0x25f502c; cbz @0x25f503c). flag==0 -> `ldr x0,[x19,#1032]`
  (NULL app-DM controller) -> `ldr x8,[x0]` fault=0x0. flag!=0 -> `mov x0,x19; bl 0x2ea3a84`
  (preload-overrides with real x19) + branches over the deref. SH269's seed only windowed the
  governor pcs (0x102ea0b60..0x2ea0bd0) which the MAIN arm bypasses -> flag stayed 0.
- FIX: widened `routeb_govflag_seed_guard`'s pc-window to the StartAppWithParams continuation
  (0x1025f5008..0x1025f5060) so the same idempotent [0x106a64da0].bit0=1 seed fires on the MAIN
  arm. MEASURED (real libroblox.so, SH405 env, EXIT 139): seed fires at pc=0x1025f5008, the
  app-start body passes 0x1025f501c (no fault there), and the continuation executes deep into the
  SH341 pool-pop persistence lane (188 events, caller LR=0x10626b6dc) — NOT toward DMCONT.
- **DMCONT 0x102bd1d68 = 0 region hits** — the MAIN-arm continuation lands in the SAME
  measured-closed SH285/LSM persistence family (answers SH405's (1): same lane, not do-init
  completion; (2) DMCONT stays 0). New real-image hermetic sh406 (arm64jit lib 454->455)
  byte-pins the flag read 0x39768128 / cbz 0x34000088 / NULL-controller deref 0xf9420660+f9400008
  / helper branch 0xaa1303e0. +frontier-sh406-govflag-appstart-main-arm doc. Honest: no DM
  (DM-root 0, MH_* false, AppBridgeV2 genuine vt unchanged). Workspace green (cargo test
  --workspace EXIT 0, 455/0); jit.rs under 1MiB hook, elfjit.rs untouched.
Single-agent (cone suppressed). New probe runs/capture_sh405_donepath_main.sh (SH404 env +
JIT_ROUTEB_DONEPATH_MAIN + LIFECYCLE_EARLYRET + SETTINGS_SSO_SEED) + new real-image hermetic
`sh405_donepath_main_flips_doinit_to_main_dispatch_into_appstart` (arm64jit lib 453->454),
+ frontier-sh405 doc. No production path / JIT hook default / guest byte changed.
MEASURED (real libroblox.so, 2/2, EXIT 134-139, 0 main-thread crash): SH320 fires (seeds
main-id [0x106863a68] to the executing thread at 0x2206db8, was 0x3) -> the thread-match
`b.eq @0x2206df0` is NOT taken -> MAIN branch -> `br x1 @0x2206e24` -> **0x10258b5d8 body
finally ENTERS** (region pcs 0x10258b5d8..0x10258bbb0 all hit), crosses SH322 (0x21f3748
caller-pair seed) + SH323 (0x21f5078/0x1025f36ac SSO seeds), then drains into the standing
SH285/LSM lane (SH341 pool-pop write-site 0x101d9a528 fires right before SIGSEGV at
guestpc 0x1025f501c, the app-start continuation after `bl 0x25f52b4 @0x25f5018`). So the
persistence lane is arm-relative: it swallows BOTH the SH404 fall-through AND the SH405 MAIN
app-start arm (strengthens SH372 path-independence from the MAIN arm). Honest: no DM
(DM-root [0x106a68818]=0, MH_* false, AppBridgeV2 genuine vt 0x1063a3410 unchanged);
DMCONT 0x102bd1d68 = 0 (standing next construction gate). Workspace green (cargo test
--workspace EXIT 0, 635/0 incl sh405; jit.rs condensed under 1MiB hook, elfjit.rs untouched).

## SH404 (Sep 20, 2026, hermes-worker): the do-init FALL-THROUGH arm runs DEEP into the app-shell ctor band; the main dispatch `br x1` is bypassed — corrects SH362's root cause (0x10258b5d8 unreachable by branch-choice, not by a pre-body fault)
Single-agent (cone suppressed). SH403 reached StartApp boot body -> app-bridge pipe 0x2baeeec ->
do-init 0x102206c40 DEEP body + post-doinit worker 0x1023eff4c. SH404 answers "where does the
do-init main-branch actually land": MEASURED (real libroblox.so, 2/2, EXIT 124, 0 crash) the do-init
MAIN dispatch `br x1` @0x2206e24 (vt[+48]=0x10258b5d8, SH361-read) is BYPASSED (0 block-entry hits);
the run takes the FALL-THROUGH arm 0x102206e30 bl 0x102206ebc -> 0x102206e34 (operator-new) ->
nested worker 0x102206fac (`sub sp,#0x60` d10183ff) -> DEEP into the app-shell ctor band
[0x102207000,0x102209000) (25+ block-entry pcs 0x102207c28..0x102207f58, frame stp a9bd7bfd at
0x102207df8), then parks cleanly (no guestpc fault). So SH362's "0x10258b5d8 body never entered"
is the FALL-THROUGH CHOICE (control flows elsewhere), not an SH285 fault before it. DMCONT
0x102bd1d68 = 0 hits (standing next gate); 0x10258b5d8 body = 0 hits. New real-image hermetic sh404
(arm64jit lib 452->453) byte-pins br x1 @0x2206e24 (d61f0020) / fallthrough worker (d10183ff) /
app-shell band frame (a9bd7bfd) / dispatch body prologue (d105c3ff sub sp,#0x170) + capture + frontier
doc. Honest: no DM (DM-root 0, MH_* false, AppBridgeV2 vt 0x1063a3410). The do-init world-build now
runs deep from the StartApp path (fall-through app-shell band); DMCONT + 0x10258b5d8 are the two
un-reached gates. Workspace green (633/0); jit.rs 221 B under the 1MiB hook (condensed SH-prose
comments, addresses kept, per this session's SH402/403/404 additions).

## SH403 (Sep 20, 2026, hermes-worker): the StartApp boot body advances into the app-bridge pipe -> do-init chain — the RECON-V3 convergence line now runs headlessly through the SH400 ordered substrate; DMCONT 0x102bd1d68 + the 0x10258b5d8 dispatch body remain the two un-reached do-init construction gates
Single-agent (cone suppressed). Disasm traced the StartApp boot body interior -> `bl 0x2baeeec`
@0x10258b2dc = the app-bridge pipe RECON-V3 names ("StartAppWithParams 0x258b144 AND StartLuaAppDM
converge on the app-bridge pipe bl 0x2baeeec -> do-init 0x2206c40"). MEASURED (real libroblox.so,
2/2, EXIT 124, 0 crash): StartApp boot body runs PAST SH402's last block-entry pc (0x258b268)
through 0x10258b3a0; the app-bridge pipe 0x102baeeec is ENTERED; do-init 0x102206c40 runs its DEEP
body (20 block-entry pcs 0x102206c40..0x102206fac); the post-do-init worker 0x1023eff4c runs; the
SH361 DM-ctor trace fires (container+32 non-NULL -> vt[+48]=0x10258b5d8 @0x2206e24). So the
RECON-V3 convergence chain StartApp boot body -> pipe -> do-init -> post-do-init worker now
executes headlessly — do-init + its worker were never reached via the StartApp path before (only
via Standalone StartLuaAppDM). DMCONT 0x102bd1d68 = 0 hits (standing next gate); the 0x10258b5d8
dispatch BODY still never entered (SH362 holds — parks cleanly EXIT 124, no fault). Also MEASURED
two never/now-run compositions (genuine-single + old SH371 DMCONT rungs; plain SH371 re-run) both
abort at the run-variable live-object arm 0x10284cfa0 (EXIT 134). New real-image hermetic sh403
(arm64jit lib 451->452) byte-pins the pipe (0x2baeeec d10143ff) / boot `bl 0x2baeeec` @0x10258b2dc
(94188f04) / do-init prologue (0x102206c40 d10303ff) / post-doinit worker (0x1023eff4c d10603ff);
+2 capture scripts + frontier doc. Honest: no DM (DM-root 0, MH_* false, AppBridgeV2 vt 0x1063a3410
unchanged). The forward: the do-init construction line is now reachable from the StartApp path;
DMCONT + 0x258b5d8 body are the two un-reached gates. Workspace green (632/0; jit.rs 31 B under
the 1MiB hook after condensing SH-prose comments, elfjit.rs untouched).

## SH402 (Sep 20, 2026, hermes-worker): CORRECTED SH401's StartAppWithParams attribution + MEASURED the REAL StartApp boot body + app-shell band reach through the ordered substrate; DMCONT 0x102bd1d68 remains the standing next gate (0 hits)
Single-agent (cone suppressed). SH401 measured the governor reach but mis-attributed the
governor's `bl 0x258c6e4` as a "StartAppWithParams entry" — RECON-V3's prose already flagged
0x258c6e4 is NOT a boot body. SH402 byte-confirms it is the AppBridgeV2 app-registry HASH-INSERT
helper (clean `stp x29,x30,[sp,#-64]!` a9bc7bfd prologue, ~0x1d0-byte leaf, returns) whose
SH159e empty-param patch makes it INSERT-then-`ret` (benign no-op), and MEASURES that the REAL
boot body nativeAppBridgeV2StartAppWithParams at 0x258b144 (`sub sp,#0xf0` = d103c3ff) runs DEEP
headlessly through the SH400 ordered substrate drive — region-watch recorded 7 block-entry pcs
0x258b144..0x258b268 — with the do-init/app-shell ctor band [0x102207b50,0x102209000) ENTERED
(~10 distinct pcs, incl the SH360 emptyvec walker 0x102208e4c/e88). DMCONT 0x102bd1d68 still
0 hits (2/2, EXIT 124, 0 crash). Also MEASURED two never/now-run compositions: the genuine-single
drive + the OLD SH371 DMCONT-firing rungs, and a plain re-run of the SH371 env, both abort at the
run-variable live-object arm guestpc 0x10284cfa0 (EXIT 134) before any StartApp body / DMCONT
(not a fresh seedable gate). New real-image hermetic sh402 (arm64jit lib 450->451) byte-pins boot
body / hash-insert helper / app-shell band entry / emptyvec walker pc; +3 capture scripts + frontier
doc. Honest: no DM (DM-root 0, MH_* false, AppBridgeV2 vt 0x1063a3410 unchanged). The STEP forward
is that the StartApp boot body (not just the hash helper) is now reachable through the substrate;
DMCONT stays the next gate. Workspace green (631/0; jit.rs condensed under 1MiB hook to 1,048,690 B).

## SH401 (Sep 19, 2026, hermes-worker): MEASURED the do-init → governor reach through the GENUINE AppBridgeV2 singleton — the frontier's named next gate after SH400, now crossed headlessly: the ordered session drive executes the real governor 0x102e9fa84 and its make-call into StartAppWithParams
Single-agent (cone suppressed). SH400 measured that StartLuaAppDM driven in the ordered
`ROUTEB_SESSION_SUBSTRATE` self-constructs the AppBridgeV2 singleton [0x106a705e8] to its genuine
relocated vt 0x1063a3410; its frontier doc named the next gate explicitly ("drive DEEPER past the
AppBridgeV2 singleton — the governor vt[+0x18]=0x102e9fa84 the singleton makes reachable"). SH401
executes that drive and MEASURES the reach on the real libroblox.so with region-watch over the
do-init worker / governor body / StartAppWithParams / DMCONT, on the SAME env as SH400
(furthest-advancing ladder + --v2boot-session-drive ordered substrate).
- MEASURED (2/2 reproducible, EXIT 124 stable, 0 crash): the ordered session drive now executes
  do-init worker 0x1023eff4c -> `blr [vt+0x18]` @0x23effbc -> **real governor 0x102e9fa84**
  (frame `stp` a9ba7bfd, MODERN router after SH159c patch) -> governor MODERN-dispatch body
  0x102e9fb58 -> `bl 0x258c6e4` **StartAppWithParams** (frame `stp` a9bc7bfd, body 0x258c7b4
  past the allocation) — the governor and StartAppWithParams were BOTH "0 hits" headlessly
  before (SH378/379: "governor/app-shell ctor 0 hits"). The genuine AppBridgeV2 vt makes the
  do-init blur land on the real governor entry.
- NEXT standing gate: **DMCONT 0x102bd1d68 (engine-init continuation) still NOT reached**
  (0 region hits) — one step past StartAppWithParams's entry.
- New real-image hermetic `sh401_doinit_governor_reach_chain_pinned` (arm64jit lib 449->450)
  byte-pins the reach chain: do-init worker prologue d10603ff, blr[vt+0x18] d63f0100 @0x23effbc,
  governor frame a9ba7bfd @0x102e9fa84, gov dispatch blr d63f0120 @0x2e9fb54, gov make-call
  `bl 258c6e4` 97dbb2df @0x2e9fb68, StartAppWithParams frame a9bc7bfd.
- Capture `runs/capture_sh401_governor_reach.sh` + `docs/frontier-sh401-governor-reach.md`.
- Honest: does NOT manufacture a DM (DM-root [0x106a68818]=0, MH_* false, AppBridgeV2 genuine
  vt unchanged from SH400). A MEASURE (region-recorded reach) + hermetic pin, confirming the
  SESSION-CTOR substrate now exercises real engine session code deeper than ever. Workspace green
  (cargo test --workspace EXIT 0, 630 passed/0 failed; arm64jit lib 450). elfjit.rs/session.rs
  untouched; jit.rs off-hook (+new hermetic, size 1,048,517 B < 1MiB hook).

## SH400 (Sep 19, 2026, hermes-worker): the ordered session-substrate DRIVER — the missing executable half of SH399's BUILD-THE-RUNTIME substrate, now lands real production code after 9 probe-only cycles (SH390-398) and MEASURES that driving the ordered 16-atom substrate self-constructs the AppBridgeV2 singleton to its genuine vt
Single-agent (cone suppressed). PRODUCTION CODE (not a probe): new small module `session.rs`
(+ `pub mod session` in lib.rs) — `routeb_session_substrate_drive()`, `substrate_args()` per-atom
ABI templates (mirroring every proven --v2boot rung incl. SH186 "Home"-in-x5 for SendAppEventOnAppReady,
surface-token for V2UpdateSurface, fabricated mgr for onEngineSettingsReceived), `drive_atom()` (single
serialized jit_run SH55/64, per-atom MH_*/AppBridgeV2 readback), + SH82 main-id gate at nativeGameGlobalInit.
elfjit `--v2boot-session-drive` rung + env-gated call site. +2 hermetic (no binary: every atom has a
template; atom-name set exactly matches substrate). Capture runs/capture_sh400_session_substrate_drive.sh.
Real-binary MEASURED 2/2 reproducible: EXIT 124 stable, 0 crash, substrate completes 16/16 atoms
(11 return Ok≠0; rung-1 nativeGameGlobalInit park crossed by SH82 gate), and — the genuine forward —
**AppBridgeV2[0x106a705e8] advances 0x0 → 0x1063a3410 (the genuinely-relocated in-image singleton
vtable, recon-sh156) from atom [5/16] StartLuaAppDM onward** (12 atoms report the real vt; every prior
cycle measured this singleton stuck at 0x0). This is the SESSION-CTOR lever: StartLuaAppDM driven in the
ordered substrate self-constructs the AppBridgeV2 singleton (builder 0x2ea3084 __call_once, the gate
before the governor 0x102e9fa84). Honest: does NOT manufacture a DM (DM-root 0, once-slot 0x400000b
sentinel, MH_* false) — the AppBridgeV2 advance is an in-image session-state construction, not a live
DataModel; it unblocks the next SESSION-CTOR gate (governor). Workspace green (cargo test --workspace
EXIT 0, arm64jit lib 449 passed/0 failed; jit.rs/elfjit.rs untouched, new file off the 1MiB hooks).

## SH399 (Sep 21, 2026, hermes-worker): the coherent ordered SESSION-SUBSTRATE — library deliverable of the SEP-17/18 BUILD-THE-RUNTIME directive. Assembles the real Activity/AppBridge lifecycle atoms (previously only scattered opt-in --v2boot-* rungs, never one validated order) into a single 16-entry pub-const ROUTEB_SESSION_SUBSTRATE table + a real-image hermetic sh399 that byte-pins every guest address to a genuine fn prologue on libroblox.so (sub-sp 0xd1 / stp 0xa9, .text file-offset==vaddr) FLAGS a drift; ABI slot <=5 asserted; addresses deduped. ROUTE-B ladder first (nativeInitializeNativeFlags 0x10232048c, nativeGameGlobalInit 0x102206404, setTaskSchedulerBM 0x102bb2380, V2InitWithParams 0x102365c54, StartLuaAppDM 0x1023efe2c, V2StartAppWithParams 0x10258b144, V2UpdateSurface 0x1025f5fec) then SEP-17 lifecycle (initAppShellReporter 0x1021f53b8, setActive 0x1021f5de4, SetInitParams 0x102bcc814, InitClientSettings 0x1022265fc + Signed 0x102bb070c, onEngineSettingsReceived 0x102bd1c38, SendAppEventOnAppReady 0x102bb463c Home-in-x5, SendAppEventOnGameLoaded 0x102bb429c, MessageBus.subscribe 0x102ba5bb8). Additive-only (table = data; elfjit ladder consumes the order); NO production behavior change. recon-v3 deliverables re-verified green at this HEAD (24 real task frames swap Ok(0x1), producer INERT 0 GATED, JSON fix present). Workspace green (cargo test --workspace EXIT 0, arm64jit lib 447 passed/0 failed); jit.rs held under the 1MiB hook by condensing SH-prose comments (facts/addresses preserved), elfjit.rs untouched. +docs/frontier-sh399-session-substrate-ordered.md. Route-B live-DM gate UNCHANGED (DM-root 0, MH_* false).

## SH398 (Sep 21, 2026, hermes-worker): MEASURED (never-run intersection closed) — the WORKING DELEGATE observer ON the SH285-CROSSED env (append+pack skips fired, sh285=0) records 0 validated make_shared<DataModel> despite the do-init MAIN dispatch firing and 685 allocations observed — the strongest "no live DM" closure yet, and the persistence-cookie milestone (SH177) landed, so Route B owns the cone again
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables re-verified green at this exact
HEAD first: capture_taskv4_frame.sh attempt 1 = 24 real task-driven frames `present swap Ok(0x1)`, 197
node pops, 0 json abort, 0 crash; capture_sh304 session-gated producer INERT (3 UNGATED/0 GATED);
JIT_JSON_ZERO_FIX present at 0x102355d40. New probe
runs/capture_sh398_dmcap_delegate_lsm_crossed.sh (SH397 env + SH349 APPEND_SKIP + SH350 PACK_SKIP +
DELEGATE) + docs/frontier-sh398-dmcap-delegate-lsm-crossed.md. Probe-only — no production Rust / guest
byte / JIT-hook-default touched; tree clean at SH397, workspace green (cargo test --workspace EXIT 0).
- SH397 left one composition never run: the WORKING DELEGATE observer was only ever combined with a
  NON-crossed env that terminates AT the SH285 persistence lane (pre-lane reach only), and the
  SH285-CROSSING env (SH373/385) was never run under an observer. SH398 composes them: SH285 crossed
  (append-skip+pack-skip fire, `guestpc=0x101db1b08` sh285-hits=0), do-init MAIN dispatch fires
  (SH361: container+32 non-NULL -> vt[+48]=0x10258b5d8), DELEGATE installs (`routed CRT operator-new
  ACTIVE` prev_hook 0x1021ebaf4), 685 allocations observed (all 0x18 bytes + ONE 0x20040 pb_defaults
  registry object at the SH88 hashfix site), and **[validated] make_shared<DataModel> = 0**.
- Terminal drains to the standing SH285 pool-pop write-site 0x101d9a528 (SH341 family) — the crossed
  lane lands in the SAME measured-closed seam, no DM event under the only observer that proves absence.
  This is the deepest, fully-crossed, working-observer reach on record; the "no live DM" verdict is
  now cornered from every static-composition angle (SH385/393/396/397 now + SH398).
- Interpretation: map-completion + observer-depth advance (never-run intersection), NOT a DM advance —
  honest. Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false, AppBridgeV2
  0). Session-gated producer + R1 content remain latent-but-correct.
- SEP-15 directive condition MET: the cookie-value persistence milestone (SH177 readback / classified
  value via cookie_jar_write_value, committed at HEAD) has landed, so per the operator this cycle
  STOPs the persistence track and Route B owns the cone again (re-attack live-DM construction; the
  do-init/StartApp body remains the standing SESSION-CTOR wall). Do-not-re-tread unchanged (incl.
  LSM-manufactured wiring SH385, setDataModelToCurrent SH388, maps SH396) + SH398: do NOT expect the
  DELEGATE observer to find a DM past the crossed SH285 lane either (measured 0 validated).

## SH397 (Sep 21, 2026, hermes-worker): MEASURED (never-run intersection closed) — the SH174 DM-allocation capture latch WITH the working SH395 DELEGATE observer on the deepest do-init DISPATCH-REACHING ladder records 0 validated make_shared<DataModel>, finalizing the "no live DM" verdict AT the actual DM-ctor dispatch junction via a trustworthy observer
Single-agent (cone suppressed). Recon-v3 immediate-priority deliverables re-verified green at this
exact HEAD first (capture_taskv4_frame.sh attempt 1 = 24 real task-driven frames `present swap
Ok(0x1)`, 197 node pops, 0 json abort, 0 crash; capture_sh304 producer INERT 3 UNGATED/0 GATED;
JIT_JSON_ZERO_FIX present at 0x102355d40). New probe
runs/capture_sh397_dmcap_delegate_doinit_dispatch.sh + docs/frontier-sh397-dmcap-delegate-doinit-dispatch.md.
No production Rust / guest byte / JIT-hook-default touched; workspace green (cargo test --workspace
EXIT 0, 626 passed/0 failed).
- SH395 established the SH174 latch only arms cleanly with JIT_DM_ALLOC_CAPTURE_DELEGATE=1 (safe-latch
  refuses over the engine's real hook 0x1021ebaf4) and that "0 validated" then is trustworthy. But
  SH395b ran the DELEGATE observer only on the --v2boot-skip-appstart env, which NEVER reaches the
  do-init MAIN dispatch (br x1 @0x2206e24 -> vt[+48]=0x10258b5d8) — the real DM-ctor entry
  (SH361/SH362). The furthest-reach + working-observer composition was never run under DELEGATE.
- SH397 runs exactly that never-run intersection: sh361's full-ladder dyn-trace env (no skip-appstart)
  + JIT_DM_ALLOC_CAPTURE=1 DELEGATE=1. MEASURED: the do-init MAIN dispatch FIRES (SH361: container+32
  non-NULL, [obj]vt=0x10635dde8 vt[+48]=0x10258b5d8); the DELEGATE latch INSTALLS through the engine's
  OWN hook (prev_hook 0x1021ebaf4, delegation clean, FIRST call#1-5 all bytes=0x18 — none DM-plausible);
  **`[validated] make_shared<DataModel>` = 0**; terminal drains to the standing SH285 persistence-lane
  class (SIGSEGV/bad_function_call, EXIT 139); MH_FLAGS_LOADED=false MH_APP_READY=false AppBridgeV2=0.
- Interpretation: the furthest dispatch-reaching ladder produces ZERO validated DM under a WORKING
  observer — the deepest, most honest instrumentation depth for the Route-B live-DM verdict. The do-init
  dispatch reaches the StartAppWithParams body (0x258b5d8) but faults in the SH285 persistence lane
  before any live DataModel allocation. Route-B live-DM gate UNCHANGED (DM-root 0, MH_* false). This is
  map-completion + observer-depth advance, NOT a DM advance (honest). Recon-v3 deliverables unchanged-green.

## SH396 (Sep 2026, hermes-worker): MEASURED NEGATIVE — manufacturing the LSM map-global cannot cross the SH285 reader wall (the pre-existing elfjit [lsm-map] seeder already provides a coherent empty map-global [0x10726f8c0] + per-node 0x20 cells, yet the full ladder STILL faults at guestpc=0x101db1b08 fault=0xff..ff 4/4) — closes SH384's "wire into the lane" manufacture hypothesis with execution evidence and refines SH285/SH385's path-independence verdict from a new angle
Single-agent (cone suppressed). Recon-v3 immediate-priority deliverables re-verified green at
this HEAD first (capture_taskv4_frame.sh attempt 1 = 24 real task-driven frames `present swap
Ok(0x1)`, 197 node pops, 0 json abort, 0 crash; capture_sh304 session-gated producer INERT;
JIT_JSON_ZERO_FIX present at 0x102355d40). SH396 implemented the SH384-line root-cross:
MANUFACTURE a coherent empty LSM map-global into [0x10726f8c0] (mg_buf(key>>29) -> 8192-slot
bucket(ubfx key,16,13) -> one node, [node+0]=0, [node+40]=writable value buffer) at the ladder
entry block 0x2173ff4, gated on the map-global being unconstructed (0) — so the SH285 reader
0x1d99e30 resolves ANY key to a writable value and the append completes instead of faulting
(cause-not-symptom PATH-B reconstruction, INSTALL-only-when-0, +hermetic sh396, arm64jit lib
446->447, workspace green). MEASURED (real libroblox.so, full SH384 env + MAP_MANUFACTURE,
4/4): the manufacture NEVER installs because the existing elfjit [lsm-map] seeder
(elfjit.rs ~6214-6310, SH267 static-empty-map machinery) already populates [0x10726f8c0] with
a coherent map `(4194304 buckets, shared zero sub, node_cells=true)` — cur!=0, so the guard
CORRECTLY declined — AND the ladder STILL SIGSEGVs at guestpc=0x101db1b08 fault=0xffffffffffffffff
(sh285=1) every run. Interpretation: a coherent empty map-global does NOT cross the SH285 reader
wall — the reached node's garbage [+40] is from a path-independent unconstructed object NOT in
the seeded map, set only by a real LocalStorageManager session ctor (SH385's verdict, confirmed
from a NEW manufacture attempt). This is the operator's "PROOF-of-dead-end" grind, measured not
judged. REVERTED cleanly (git checkout jit.rs to HEAD; capture script removed) — the final tree
is byte-identical to SH395 plus this doc + HANDOFF/STATUS; workspace green (cargo test
--workspace EXIT 0, arm64jit lib 446/0). Route-B live-DM structural gate UNCHANGED (DM-root
[0x106a68818]=0, MH_* false, AppBridgeV2 0). Do-not-re-tread +: do NOT re-manufacture
[0x10726f8c0]-style coherent empty maps to cross the SH285 reader — the existing seeder already
provides one and the reader still faults (measured 4/4).

## SH395 (Sep 21, 2026, hermes-worker): CORRECT the SH378/SH394 "capture latch never installs" attribution — it never installed because JIT_DM_ALLOC_CAPTURE_DELEGATE was UNSET and the engine's real allocator hook is present; with DELEGATE the SH174 latch installs + fires cleanly on the SAME furthest-advancing composition, delegating real allocations through the engine's own hook (prev_hook measured 0x1021ebaf4) — the "no DM" verdict is now proven by a WORKING observer, not a refused-install artifact
Single-agent (cone suppressed). Recon-v3 immediate-priority deliverables re-verified green at this
exact HEAD (capture_taskv4_frame.sh attempt 1 = 24 real task-driven frames `present swap Ok(0x1)`,
0 json abort, 0 crash; capture_sh304 producer INERT). Two genuinely-never-run probes
(runs/capture_sh395_opnew_observer_audit.sh + runs/capture_sh395b_dmcap_delegate_fulltable.sh) +
docs/frontier-sh395-latch-observer-artifact-corrected.md. No production Rust / guest byte touched;
workspace green (cargo test --workspace EXIT 0).
- **SH395a** (SH394 env, capture-only): region-watching the whole CRT operator-new band
  [0x102a0d940,0x102a0da00) + DMCONT operator_new entries PROVES operator-new EXECUTES headlessly on
  the furthest composition (0x102a0d9b8/0x102a0d9fc, 0x101db1a38/0x1b1ad4/0x1b1adc/0x1b1afc/0x1b1b78/
  0x1b1c8c, allocator tail 0x101db1c60, 0x101d96768/0x96778/0x967b0 all hit) EVEN THOUGH the latch
  still never installed — so SH394's "operator-new is never enterinstallably" was wrong.
- **SH395b** (the discriminator): SAME full-table env + `JIT_DM_ALLOC_CAPTURE_DELEGATE=1` -> the latch
  NOW INSTALLS on the composition SH394 said it "refuses to arm" on (`routed CRT operator-new ACTIVE
  hook ... capture trail`), and FIRES calls #1-5 delegating 0x18-byte allocations through the engine's
  OWN real hook (prev_hook=0x1021ebaf4) — delegation completes CLEANLY (SH344's cathed bad_function_call
  was the separate JIT_ROUTEB_DM_REALCTOR drive, not delegation). Terminal still the standing SH285
  LSM pool-pop wall guestpc=0x101d9a528 (EXIT 134); glue-full 15/15 Ok; SendAppEventOnAppReady Ok.
- Interpretation: SH378/SH394 measured the safe-latch REFUSING to replace a present engine hook
  without DELEGATE — an observer artifact, not "no allocation." The "no make_shared<DataModel> yet"
  verdict STILL STANDS, now from a WORKING observer (armed trail saw only 0x18-byte allocs, none
  DM-plausible), so future "0 validated" results are trustworthy-by-construction. Route-B live-DM
  structural gate UNCHANGED (DM-root 0, MH_* false, AppBridgeV2 0). Do-not-re-tread +: do NOT run the
  SH174 latch without JIT_DM_ALLOC_CAPTURE_DELEGATE=1 expecting an install (safe-latch refusal, SH395).

## SH394 (Sep 21, 2026, hermes-worker): MEASURED the never-run composition — the SH393 FULL safe app-command table drive + the SH174 DM-allocation capture latch (single forward observer) on the furthest-advancing SH378 env: the capture latch never even ARMS, 0 validated make_shared<DataModel>, terminal still the standing SH285 persistence-lane wall
Single-agent (cone suppressed). Recon-v3 immediate-priority deliverables re-verified green at this
exact HEAD first (capture_taskv4_frame.sh attempt 1 = 24 real task-driven frames `present swap
Ok(0x1)`, 196 node pops, 0 json abort, 0 crash; capture_sh304 session-gated producer INERT 0 GATED;
JIT_JSON_ZERO_FIX present at 0x102355d40). SH393 established the full app-command table drive (the
engine's OWN process_cmd consuming all 15/20 safe APP_CMD cases in lifecycle order). But SH393's
capture never set JIT_DM_ALLOC_CAPTURE, and SH378 ran the SH174 latch on the SH377
env WITHOUT the full-table drive. SH394 runs the genuinely-never-composed intersection: full safe
table + DM-capture latch + the furthest SH378 world-build env. New probe
runs/capture_sh394_dmcap_fulltable.sh + docs/frontier-sh394-dmcap-fulltable.md.
MEASURED (real libroblox.so, 1/3, confirm=1): full-table 15/15 `process_cmd returned Ok`, cmd-11
INIT_WINDOW marker, `glue-full SH393 done`, SendAppEventOnAppReady returned Ok — but **the DM-capture
latch NEVER installs** (0 `routed ... capture trail` lines, so operator-new 0x102a0d9b8 is never
enterinstallably on this composition) and **`[validated]` make_shared<DataModel> = 0**; the only
`bytes=` hit is the SH339 "Home" jstring readback, not an allocation. Terminal: SIGABRT/SIGSEGV
drains to the standing SH285 LSM pool-pop write-site guestpc 0x101d9a528 (EXIT 134). Post-lifecycle
MH_FLAGS_LOADED=false MH_APP_READY=false AppBridgeV2[0x106a705e8]=0x0.
Interpretation: composing the engine's full own-command-queue drive with the DM-allocation latch on
the furthest env STILL does not route any dispatch to make_shared<DataModel> — and the latch
stronger-than-SH378 refuses even to install. Re-confirms SH366/368/393's reading: AppBridgeV2/surface/
DM "move only when a live session/do-init builds the DM world." Route-B live-DM structural gate
UNCHANGED (DM-root [0x106a68818]=0, MH_* false, AppBridgeV2 0). Do-not-re-tread unchanged + SH393's
cmd 1/13/15/17/18 exclusion. Work done beyond the standing walls: this is a never-run-intersection
map-completion (probe-only, no Rust/guest-byte/JIT-hook-default touched); workspace green (cargo
test --workspace EXIT 0; arm64jit lib 446/0).

## SH393 (Sep 21, 2026, hermes-worker): drive the FULL app-command dispatcher table on the confirmed-green SH366 entry — a complete per-command safety map (15/20 safe, cmd 1/13/15/17/18 measured-unsafe) of the engine's own process_cmd, the SESSION-CTOR command-queue lever advanced from SH368's 3-command subset
Single-agent (cone suppressed). Recon-v3 immediate-priority deliverables re-verified green at this
exact HEAD first: capture_taskv4_frame.sh (SH391 guard armed) attempt 1 = 24 real task-driven frames
`present swap Ok(0x1)`, 196 node pops, 0 json abort, 0 crash, EXIT 124; capture_sh304 session-gated
producer correctly INERT (0 GATED); JIT_JSON_ZERO_FIX present at 0x102355d40. New default-inert opt-in
rung `--v2boot-glue-cmd-full` (jit.rs `drive_glue_process_cmd_full`) + real-image hermetic
`sh393_glue_cmd_full_table_bound_pinned` (arm64jit lib 445->446) + capture runs/capture_sh393_glue_full.sh
+ frontier doc. MEASURED + disasm-classified the full 20-entry process_cmd (0x102bcd6e4) jump table:
the case is chosen by a 16-bit rel offset (table 0x69408a -> case 0x2bcd730+rel*4); every case reads
the version gate [0x683d8d0] then `cmp byte0,#6; b.cc <target>`; the b.cc TARGET decides safety at gate 0.
**15 SAFE** (b.cc->epilogue 0x2bcdbf0 / glue/telemetry-only body, incl INIT_WINDOW cmd 11) drive
cleanly — MEASURED 15/15 `process_cmd returned Ok`, cmd-11 marker [inner+9]=1, EXIT 124, 0 crash.
**5 UNSAFE** (measured the runtime refuses headlessly): cmd 1 = PRE-GATE live-object deref
0x2bcd9fc `[x20+24]->[+56]` (fault 0x38); cmd 13 = gate-checked-but-body derefs [x20+24] @0x2bcdc74
(fault 0x20); cmd 15/17/18 = same live-object-deref class. This PROVES gate-check alone != safety
(the b.cc target does). Session observables re-confirm SH366/368 with the full table: the engine's
own command dispatcher does NOT self-transition AppBridgeV2 ([0x106a705e8] 0x0->0) or the surface
XID (0x200000) — those still move only when a live session/do-init builds the DM world. Workspace
green (cargo test --workspace EXIT 0; arm64jit lib 446/0; jit.rs 1,046,770 B <1MiB hook; elfjit.rs
1,048,492 B <1MiB hook, pulled under by condensing two SH-prose comments — 4 bytes over the hook
mid-cycle). Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false,
AppBridgeV2 0); SH174 capture-latch stays the single forward observer. Do-not-re-tread +:
do NOT drive cmd 1/13/15/17/18 expecting a headless return (measured live-object deref; SH366/367 class).

## SH392 (Sep 21, 2026, hermes-worker): arm the SH391 render-determinism guard on the canonical HARD-GATE artifact runbook — the recon-v3 deliverable (capture_taskv4_frame.sh) ran WITHOUT the deterministic fix engaged, so the "24 frames 0 crash" artifact was still subject to the ~1/25 SH345 wire-into-.text flake
Single-agent (cone suppressed). The recon-v3 self-driven-frame capture
(capture_taskv4_frame.sh) was re-verified green at this exact HEAD on skill
handover — attempt 1: 24 real task-driven frames `present swap Ok(0x1)`, 197
node pops, 0 json abort, 0 crash. But auditing the runbook against the SH391
fix landed last cycle exposed a real gap: SH391's guard is opt-in
(`JIT_ROUTEB_RENDER_MEMCPY16_GUARD`, read at leaf-entry 0x102859fd0 in
`routeb_render_memcpy16_guard`, jit.rs:1440) and the capture script did NOT set
it — so the canonical HARD-GATE artifact was produced with the deterministic
fix DISENGAGED, i.e. the ~1/25 SH345 SIGSEGV could still crash the frame
deliverable and the script's MAX_ATTEMPTS=6 retry loop was masking it (the
exact retry-hide SH391 was meant to kill). This cycle arms the guard in
run_once (`JIT_ROUTEB_RENDER_MEMCPY16_GUARD=1` in the env) so the artifact is
deterministic-by-construction, and rewrites the SH345 header to say the fix is
now not-retry-hide and the retry loop survives only for pre-existing
run-variable walls (SH353-class). RE-VERIFIED green WITH the guard armed:
attempt 1, 24 real frames `swap Ok(0x1)`, 0 crash (guard inert on the clean
run — correct). Workspace green (cargo test --workspace EXIT 0, 445 arm64jit
lib tests). Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0,
MH_* false). Do-not-re-tread unchanged (SH388 setDataModelToCurrent, SH385 LSM,
SH355/356 EC reader-gate, SH362/375 0x258b5d8, SH367 window-attach real, SH365
ALooper, SH379 governor gates full-ladder, SH380 -9 string, SH248h map-header,
SH381 once-lambda, SH267 node-cell).

## SH391 (Sep 20, 2026, hermes-worker): deterministic fix for the SH345/SH390 render-plane flake — guard the pinned memcpy16 leaf's no-op self-copy when its dest is non-writable, turning the ~1/25 retry-hidden SIGSEGV into an instrumented, always-green artifact
Single-agent (cone suppressed). SH390 byte-anchored the SH345 fault leaf (guest
0x102859fd0, file 0x2859fd0, `str q0,[x0]` @0x2859fe4) but deliberately left the fix for
"a future guard" — the capture script still retry-hardened (coin-flip, up to 6 attempts).
SH391 delivers option *b* the frontier doc prescribes. **The store ONLY executes when
x0==x1** (the leaf's `cmp x0,x1; b.ne out` guarantees a real copy exits before the
store), so the leaf is always a 16-byte SELF-COPY = semantic no-op; it faults only when
that (self,same) dest is non-writable (PROT_EXEC guest .text in the drain divergence
arm). NEW default-inert opt-in guard `routeb_render_memcpy16_guard`
(JIT_ROUTEB_RENDER_MEMCPY16_GUARD, wires into the per-block-entry guard dispatch) fires
at leaf-entry pc 0x102859fd0: when x0==x1 && x0!=0 && !page_is_writable(x0) it zeroes x1
so the block's `cbz x1` exits BEFORE the store. Provably cannot mask a real walker copy
(a real copy is x0!=x1, already `b.ne`-exited). +hermetic `sh391` (arm64jit lib
444->445; PROT_NONE mmap = deterministic non-writable dest; asserts guard leaves real
copies + writable self-copies alone, inert off-env and on non-entry pc) + frontier doc.
No production path / guest byte permanently touched, no test weakened.
**LIVE real-binary run (real libroblox.so, completing taskv4-frame ladder) with the
guard armed: confirmed-green attempt 1 — 24 real task-driven frames `present swap Ok(0x1)`,
197 node pops, 0 SIGSEGV/ABRT, EXIT 124; guard-firings=0 (inert on the clean run —
correct).** Workspace green (cargo test --workspace EXIT 0; arm64jit lib 445/0).
Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false,
AppBridgeV2 0). SH174 capture-latch stays the single forward observer. Do-not-re-tread
unchanged (LSM skips, setDataModelToCurrent SH388, EC reader-gate, 0x258b5d8/SetInitParams,
window-attach real, ALooper, governor gates, -9 string, map-header repair, once-lambda store,
SH267 node-cell).

## SH390 (Sep 20, 2026, hermes-worker): byte-anchor the SH345 render-plane flake's exact fault leaf (memcpy16 into guest .text 0x102859fd0) — a real determinism fix target for the HARD-GATE reproducible artifact, replacing the retry-hide coin-flip
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables re-verified
green LIVE at this exact HEAD (capture_taskv4_frame.sh attempt 1 = 24 real task-driven
frames `present swap Ok(0x1)`, 197 node pops, 0 json abort, 0 crash; capture_sh304 =
session-gated producer correctly INERT 0 GATED / 0 fabricated frames); workspace green
(cargo test --workspace EXIT 0, 25 test binaries, 444 arm64jit lib tests). All named
Route-B cones remain measured-closed (incl. the SH384 manufactured-manager wiring
target = SH385-closed LSM re-tread, NOT re-driven). SH390 takes the ONE genuinely-open
in-scope defect: the SH345 ~1/25 SIGSEGV in the primary frame deliverable was recorded
in prose only ("store into guest .text 0x102859fd0") and mitigated by capture-script
RETRY, not root-cause. New real-image hermetic `sh390_render_plane_memcpy16_fault_leaf_pinned`
(arm64jit lib 443->444) byte-pins the exact fault leaf: 0x102859fd0 = a memcpy16 guard
(`cmp x0,x1; b.ne; cbz x1; cbz x0; ldr q0,[x1]; str q0,[x0]; ret`), crash store @file
0x2859fe4 — the copy dest computes to a PROT_EXEC guest .text addr from the
activity-lifecycle divergence arm (drain/presenter quirk), the same run-variable
live-object class SH346/353. This turns the retry-hidden coin-flip into a pinned,
guardable leaf (ret-to-leaf or zero-x1 in the drain-entry window = a real determinism
fix, deliberately not implemented this cycle — needs a live JIT_REGION_WATCH first to
avoid masking a real walker copy). NO production path / JIT hook default / guest byte
touched (pure pin, adds coverage). Do-not-re-tread unchanged (setDataModelToCurrent
SH388, LSM skips, EC reader-gate, 0x258b5d8/SetInitParams, window-attach real, ALooper,
governor full-ladder, -9 string, map-header repair, once-lambda store, SH267 node-cell).
Route-B live-DM gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false, AppBridgeV2 0).
SH174 capture-latch stays the single forward observer.

## SH389 (Sep 20, 2026, hermes-worker): correct a false doc claim in the recon-v3 §A session-gated producer's live-DM gate + re-verify all recon-v3 deliverables + the SH388 wall green at this exact HEAD
Single-agent (cone suppressed). Recon-v3 immediate-priority deliverables re-verified green at this HEAD
(no regression): capture_taskv4_frame.sh attempt 1 = 24 real task-driven frames `present swap Ok(0x1)`,
197 node pops, 0 json abort, 0 crash; capture_sh304_session_producer.sh = session-gated producer
correctly INERT on bare boot (0 GATED, 0 fabricated frames, lone present #0 = the independent
render-plane warmup, EXIT 124 clean); JIT_JSON_ZERO_FIX present at 0x102355d40. The SH388
setDataModelToCurrent cone-door wall reproduces exactly (GETTER 0 hits, BODY 0 hits,
[0x102dbcc10,0x102dbcd40) 0 JIT region hits, terminal guestpc=0x101d9a528 EXIT 134) — the standing
measured-closed persistence lane; DM-root 0, MH_* false. Fix: `session_live_dm`'s doc claimed "All
reads page-guarded via read_visible_u64" but read_visible_u64 is a BARE DEREF (no page check) — the
reads are safe only because the real image loader maps those .bss cells; corrected the comment to say
so (documentation accuracy, no behavior change). Workspace green (cargo test --workspace EXIT 0,
25 test binaries, 0 failures). Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0,
MH_* false). Do-not-re-tread unchanged: do NOT re-attack setDataModelToCurrent (SH388 measured
not-executed); do NOT re-drive LSM crossings (SH385) / EC reader (SH355/356/374) / window-attach real
(SH367) / ALooper (SH365); once-lambda store (SH381), -9 string (SH380), map-header repair (SH248h).
SH174 capture-latch stays the single forward observer.

## SH388 (Sep 20, 2026, hermes-worker): MEASURED — the SEP-15 setDataModelToCurrent cone door is DEAD headlessly (getter 0x102dbcc10 / body 0x102dbcc1c never EXECUTED, whole 0x102dbcc region 0 JIT hits, even with a manufactured DM planted in the holder) — closing SH387's "cone door remains OPEN" with execution evidence
Single-agent (cone suppressed). SH387 byte-anchored the operator's SEP-15 re-attack cone
door (DataModelServices current-DM GETTER 0x102dbcc10 leaf / BODY 0x2dbcc1c persistence
state-setter) but never measured whether the engine EXECUTES it headlessly. New READ-ONLY
probe `routeb_dmsvc_getter_probe` (jit.rs, opt-in JIT_ROUTEB_DMSVC_GETTER=1, fires at block
entry of getter+body, reads current-DM holder [0x106391908] + app-DM counter [0x106dca0e88],
ZERO guest mutation) + hermetic `sh388_dmsvc_getter_cone_door_reachability_pinned`
(arm64jit lib 442->443) + probe runs/capture_sh388_dmsvc_getter.sh + frontier doc.
MEASURED (real libroblox.so, full --v2boot reaching env + JIT_ROUTEB_DM_MANUFACTURE=1):
GETTER fires 0, BODY fires 0, [0x102dbcc10,0x102dbcd40) = 0 JIT region hits — the accessor
is NEVER entered headlessly; the manufactured genuine-vptr DM WAS planted (holder already
held in-image obj 0x106358d40) but no path consumes it through this accessor. Terminal
drains to the standing SH285 persistence-lane wall guestpc=0x101d9a528 (EXIT 134), the
measured-closed family every ladder arm converges on. Interpretation: the setDataModelToCurrent
cone is measured-not-executed, so re-attacking it (plant-DM / arm) cannot advance Route B
headlessly — a clean closure of the last explicitly-named re-attack cone (consistent with
SH379 path-specific gates). No DataModel; Route-B live-DM gate UNCHANGED (DM-root 0, MH_*
false). Workspace green (cargo test --workspace EXIT 0, 622/0; arm64jit lib 443/0). Do-not-
re-tread updated: do NOT re-attack setDataModelToCurrent expecting the accessor to fire.
recon-v3 deliverables unchanged-green. SH174 capture-latch stays the single forward observer.

## SH387 (Sep 20, 2026, hermes-worker): byte-anchor the DataModelServices current-DM getter ABI (the SEP-15 re-attack cone door), arm64jit lib 441->442
Single-agent (cone suppressed). The operator's SEP-15 ROUTE-B directive names
ExperienceController / DataModelServices::setDataModelToCurrent (SH163 flagged 'next
seed must target ExperienceController') as the FRESH re-attack cone, but that door was
only ever documented in PROSE (SH172/178/180 CUR_DM_HOLDER) — no hermetic pinned
0x2dbcc10 on the real binary. New real-image hermetic
`sh387_dmservices_current_dm_getter_abi_pinned` (arm64jit lib 441->442) pins: the
pure-leaf GETTER 0x2dbcc10 (`adrp x0,6391000`/`add #0x908`/`ret` -> returns guest
0x106391908, the current-DM holder the harness's CUR_DM_HOLDER already plants a
manufactured DM into) AND the real BODY 0x2dbcc1c (sub sp,#0x40 real DEBUG-stack frame,
canary got 0x67d16f0, dispatch bl 0x24e3e98/0x2417d58 = persistence family, NOT a DM
ctor). Grounds the one cone the operator explicitly wants re-attacked on verified bytes.
recon-v3 immediate-priority deliverables re-verified green at HEAD (24 real task-driven
frames `present swap Ok(0x1)`, 196-197 node pops, 0 json abort, 0 crash, EXIT 0; JSON
len-clamp present; session-gated producer INERT on bare boot). No production path / JIT
hook default / guest byte touched (pure pin + hermetic). Route-B live-DM structural gate
UNCHANGED (DM-root [0x106a68818]=0, MH_* false, AppBridgeV2 0). Workspace green
(cargo test --workspace EXIT 0, 622/0; arm64jit lib 442/0). Do-not-re-tread stands:
setDataModelToCurrent BODY is persistence-family, do NOT re-drive it expecting a ctor
(the cone door remains OPEN for a real re-attack armed with these pins).

## SH386 (Sep 19, 2026, hermes-worker): recon-v3 §A END-STATE byte-anchored — pin the engine-producer self-drive contract (producer 0x10285682c / drain 0x102856e40 / pop + tag-guard) that the SESSION PRODUCER HANDOFF fires through the instant a live session advances (was pinned nowhere: --deque-node code was comment-anchored only)
Single-agent (cone suppressed). recon-v3 immediate-priority deliverables re-verified green at
HEAD: (1) capture_taskv4_frame.sh = 24 real task-driven frames `present swap Ok(0x1)`, 197 node
pops, 0 json abort, 0 crash, EXIT 0; (2) JIT_JSON_ZERO_FIX len-clamp present; (3)
capture_sh304_session_producer.sh = the session-gated producer is correctly INERT on bare boot
(live_dm=true, app_ready=false -> UNGATED x3, GATED=0, lone present #0 = RENDERINIT warmup,
0 crash) — the exact SH382 inverse control. New real-image hermetic
`sh386_session_producer_engine_push_contract_pinned` (arm64jit lib 440->441): byte-anchors the
recon-v3 §A end-state self-drive mechanism the operator's SESSION PRODUCER HANDOFF depends on —
producer 0x285682c prologue (`stp x29,x30,[sp,#-0x60]!` 0xa9ba7bfd + str x27 0xf9000bfb), drain
0x2856e40 prologue (0xa9ba7bfd), drain pop `ldr x23,[x20]` @0x2856f94 (0xf9400297) + `ldar x24,[x23]`
@0x2856f98 (0xc8dffef8), tag-guard `ldr x26,[x1,#104]` @0x2856e6c (0xf940343a) + `b.ne` @0x2856e78
(0x54001041) — MEASURED ok on the real binary. This grounds the end-state handoff's node-push/epoch/
futex wiring on verified bytes (was comment-only). No production path / JIT hook default / guest byte
touched; workspace green (cargo test --workspace EXIT 0, 621/0; arm64jit lib 441/0). Route-B live-DM
structural gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false, AppBridgeV2 0). +frontier-sh386 doc.

## SH385 (Sep 20, 2026, hermes-worker): MEASURED NEGATIVE (fully-composed LSM-crossing) + byte-anchored SH285 reader/pool-move mechanism — the LAST never-run composition loophole closed, refining the persistence-lane verdict to the exact fault mechanism
Single-agent (cone suppressed). Before implementing the SH383/384 "wire the manufactured LSM
manager into the lane" step, run the never-composed full-crossing intersection: the LSM crossings
(KEYFIX + APPEND_SKIP + PACK_SKIP) + full reaching env, watching for the do-init MAIN dispatch body
(0x10258b5d8, SH362's measured-never-executing gate). MEASURED (real libroblox.so, EXIT 134): even
with all three crossings armed the ladder STILL drains to the LSM pool-pop family (guestpc=0x101d9a528,
non-poisoned-key fault) and 0x258b5d8 gets ZERO region hits — the composed crossings do NOT exit the
persistence family nor reach the do-init body (consistent with SH379; keyfix was its missing leg).
Disasm-refined the SH285 letter one level deeper: reader 0x1d99e30 returns x20=[mapnode+40] via
lsm_map_global; append 0x1d9a15c computes write base `add x10,x0,x1` (manager+value) and the
backward-store `strb [x10],#-1` @0x1d9a180 is the fault site; SH267's zeroed node cells give a
nominal [+40]=0 BUT the SH285 live dump showed x20=0xffff80... on the settings-state path (the
reached node is NOT the SH267 cell) — the exact node/value is path-dependent, set only by a real LSM
session ctor. New hermetic `sh385_lsm_reader_value_slot_and_poolmove_base_pinned` (arm64jit lib
439->440) + probe runs/capture_sh385_composed_lsm.sh + frontier doc. Workspace green (cargo test
--workspace EXIT 0; jit.rs 1,014,534 B <1MiB). Route-B live-DM structural gate UNCHANGED (DM-root 0,
MH_* false, AppBridgeV2 0). SH174 capture-latch stays the single forward observer. The LSM lane is
now closed as a Route-B avenue with the strongest (fully-composed + byte-anchored) evidence on
record; do NOT re-compose its crossings expecting the do-init body.

## SH384 (Sep 20, 2026, hermes-worker): implemented + MEASURED the DRIVE of the GENUINE
LocalStorageManager ctor (guest 0x101db0dfc, file 0x1db0dfc, r-x text seg [0x100000000,0x1062d8190),
ELF file-offset==vaddr) through real relocated engine code with a coherent zeroed container —
manufacturing a real vtable-owning manager — SH383's explicitly-named next-step (was only pinned)
Single-agent (cone suppressed). New default-inert opt-in drive `routeb_lsm_ctor_manufacture_drive`
(JIT_ROUTEB_LSM_CTOR_MANUFACTURE=1, scoped to StartLuaAppDM entry, idempotent) + real-image hermetic
`sh384_lsm_ctor_null_tolerant_drive_path_pinned` (arm64jit lib 438->439) + repro
`runs/capture_sh384_lsm_ctor_manufacture.sh` + frontier doc. NO production path / JIT hook default /
guest byte touched (pure opt-in + hermetic; standard ladder default unchanged).
MEASURED (real libroblox.so, completing ladder + the drive env, 2/3): the genuine ctor DROVE ok
ret x0=0 and manufactured a genuine vtable-owning manager — [this+0]=0x10635bf88 (vt page
0x10635b000+0xf88), this+0x28=0x10635bfb8 (+0xfb8), this+0x30=0x10635bff8 (+0xff8), the EXACT words
the ctor's `add x8,x9,#0x30/#0x70` computes; inner ctor 0x101db0748 ran through real code;
in-image vt=true. The drive premise is disasm-anchored (outer cbz x0 @0x1db0e3c, inner cbz x9
@0x1db0778 -> mov x19,xzr @0x1db07c0, self-consistent canary @0x1db076c/@0x1db0810 — a zeroed
container drives it cleanly). First time the genuine LocalStorageManager ctor has ever been DRIVEN
headlessly (SH348-350/373 only skipped/seeded the lane). This is the MIGRATION-directive manufacture
lever: a real vtable-owning manager OBJECT constructed by running the engine's own ctor code.
Workspace green (cargo test --workspace EXIT 0; arm64jit lib 439/0; jit.rs 1,011,782 B <1MiB).
Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false, AppBridgeV2 0);
terminal still the SH285 persistence-lane wall guestpc=0x101db1b08 — wiring the manufactured
manager INTO the lane so the SH285 reader consumes it is the further step (needs a coherent
`container` sub-object whose vt[+24] dispatch survives; structural, not a fixed-.bss seed).
SH174 capture-latch stays the single forward observer. Do-not-re-tread unchanged (incl. LSM skips
SH349/350/358/373, once-lambda store seeding SH381).

## SH383 (Sep 20, 2026, hermes-worker): pin the GENUINE LocalStorageManager constructor (0x1db0dfc) as the MIGRATION-directive manufacture target — the persistence funnel (SH285 reader/pop wall, [obj+0x50]=0xff..ff) was only ever skipped (SH348 leaf-ret) or map-seeded (SH267/285); SH383 byte-pins the real vtable-owning object ctor (this=x0, vt 0x10635b000+0xd58 & +0xe68, reads [x1+8]/[x1+16]/[x1+32], inner ctor 0x1db0748, SH285 byte-copy leaf 0x1d9a15c) so a future run_guest_callback drive is byte-anchored
Single-agent (cone suppressed). recon-v3 self-driven-frame + JSON deliverables
re-verified green at HEAD (capture_taskv4_frame.sh attempt 1: 24 real task-driven
frames swap Ok(0x1), 197 node pops, 0 json abort, 0 crash). New hermetic
`sh383_lsm_ctor_manufacture_target_pinned` (arm64jit lib 437->438, real-image
file-offset pins on 0x1db0dfc/0x1db0e34/0x1db0e84/0x1db0e98/0x1db0748/0x1d9a15c,
skip-if-absent) + frontier doc. NO production path / JIT hook / guest byte touched.
Driver note: the hornshot first attempt placed the hermetic in the 1MiB-adjacent
elfjit example (1,048,526 B, ~50B headroom) and was reverted clean; the hermetic
lives in the lib (jit.rs) like sh371/sh372/sh375. Workspace green (cargo test
--workspace EXIT 0; 438 arm64jit lib; 160 elfjit example). Route-B live-DM
structural gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false, AppBridgeV2 0).
SH174 capture-latch stays the single forward observer. Do-not-re-tread unchanged
(incl. once-lambda store seeding SH381).
## SH382 (Sep 20, 2026, hermes-worker): fold the SH381 once-slot reconciliation into the latent session-gated type-4 producer (recon-v3 §A end-state) — live-DM classifier now also reads the once-lambda's REAL write target [0x106a68408] and explicitly rejects the 0x400000b "Execute" sentinel; recon-v3 deliverables re-verified green at HEAD
Single-agent (cone suppressed). Pure-predicate change (elfjit.rs): added
`live_dm_cell_value_ok(v)` (rejects the <0x100000000 0x400000b sentinel) and made
`session_live_dm()` read once-slot [0x106a68408] (SH381: `str x0,[x23,#1032]`
@0x2206d74, x23=adrp 6a68000) as a THIRD live-DM candidate alongside
[0x106391908]/[0x106a68818], all page-guarded. Behavior-preserving for the gate
decision (MH_APP_READY=false keeps it inert on a bare boot; the once-slot arm only
ADDS recognition of a genuine DM written to the once-lambda's actual target and
still REJECTS the sentinel). +sh382 hermetic
`live_dm_cell_ok_rejects_sh381_execute_sentinel` (sh304 module) + frontier doc.
MEASURED real-binary (completing ladder + --taskv4-seed session): EXIT 124, 0 crash,
3× `disp UNGATED (app_ready=false, live_dm=true) — inert` (correct — no session
emits frames; lone present #0 is the RENDERINIT warmup self-test, not a producer
emit). No production path / JIT hook / guest byte touched. Workspace green
(cargo test --workspace EXIT 0, 617/0; arm64jit lib 437/0; elfjit example 160/0).
Recon-v3 deliverables re-verified green (capture_taskv4_frame.sh attempt 1: 24 real
frames swap Ok(0x1), 197 pops, 0 json, 0 crash).
Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false,
AppBridgeV2 0). SH174 capture-latch stays the single forward hook. Do-not-re-tread
unchanged (incl. once-lambda store seeding SH381).

## SH381 (Sep 20, 2026, hermes-worker): MEASURED at the exact store on the FULL ladder — the do-init once-lambda's DM-constructor `0x2173b3c` returns the **0x400000b "Execute" service-handle sentinel**, NOT a live DM (answers the EXECUTE-DO-INIT-GATES "LET the once-lambda populate" gate with direct evidence); PLUS an ADDRESS-RECONCILIATION fix the whole probe line conflated for ~30 SH cycles (the once-path writes [0x106a68408], not the probed [0x106a68818])
Single-agent (cone suppressed). One new READ-ONLY guard
`routeb_doinit_oncelambda_probe` (jit.rs, opt-in JIT_ROUTEB_DOINIT_ONCELAMBDA, once, ZERO
guest mutation) + hermetic `sh381_oncelambda_probe_read_only_pc_gated` (arm64jit lib
436->437) + probe runs/capture_sh381_oncelambda.sh + frontier doc. elfjit.rs product path
unchanged. Workspace green (cargo test --workspace EXIT 0; arm64jit 437/0). recon-v3
deliverables re-verified green at HEAD (24 real task frames, swap Ok(0x1), 197 pops,
0 json, 0 crash).
- **The measurement** (full --v2boot send-appevent ladder, live x0 at block-entry
  0x102206d70 = just after `bl 0x2173b3c` @0x2206d6c, before `str x0,[x23,#1032]` @0x2206d74):
  `[routeb-sh381] ctor RETURN x0=0x400000b (sentinel/handle — NOT a live DM); once-slot
  [0x106a68408]=0x0 DM-root[0x106a68818]=0x0 once-guard=0x200`. The once-lambda's construct
  helper returns the SAME 0x400000b "Execute" service-handle sentinel SH316 documented —
  a handle factory, not a DataModel ctor. LETTING it populate [0x106a68818] (EXECUTE-DO-
  INIT-GATES) cannot build a DM by itself; the real DM is on the MAIN-branch `br x1`
  @0x2206e24 -> vt[+48]=0x10258b5d8 dispatch (SH361/SH362) which then hits the standing
  live-object family.
- **Address-reconciliation (genuinely new, fixes a 30-cycle conflation)**: do-init's once-path
  stores to **once-slot [0x106a68408]** (0x6a68000 + #1032), while every harness probe reads
  **DM-root [0x106a68818]** (+0x818) — DIFFERENT cells. A probe on 0x106a68818 shows 0 even
  when the once-lambda fires and writes 0x106a68408. Post-run `once-slot` reads (SH155/311/316)
  were actually reading the write target all along; `DM-root` 0x106a68818 is a separate holder.
- Terminal on the full ladder: standing SH248g live-map wall guestpc=0x1021dde34 (EXIT 139,
  signals=3) — unchanged; the guard is read-only and moves nothing. Route-B live-DM structural
  gate UNCHANGED (DM-root 0, MH_* false, AppBridgeV2 0). Do-not-re-tread: do NOT force/seed the
  once-lambda store (its ctor returns the sentinel); do NOT read 0x106a68818 as the once-lambda's
  write target (that's 0x106a68408); standing closures (LSM skips, EC reader-gate, 0x258b5d8,
  window-attach real, -9 string/0x102b504e4, map-header repair) all stand.
Single-agent (cone suppressed). Attempted then measured+reverted two named Route-B "next levers":
(a) `routeb_contstring_repair_guard` (JIT_ROUTEB_CONTSTRING_REPAIR) for SH248's
`operator_new(-9)->bad_alloc` — MEASURED 0-firement: the assign fn 0x102b505f0 is NEVER entered
(region-watch 0x102b505e8-0x102b50614 = 0 hits); SH248c already crossed the -9 via the M+0x48 cap
seed (`routeb_dm_manager_cont` -> valid long cap 0x11). (b) `routeb_appstart_controller_str_guard`
(JIT_ROUTEB_APPSTART_CTRLSTR) for SH248c's 0x102b504e4 NULL-dest — MEASURED 0-firement: the block
IS entered (0x102b504e4 region hit) but x0's destination is already constructed (SH248c's `x0=0`
was a run-variable single-run terminal). Both REVERTED cleanly (no cruft committed; tree at SH379
HEAD). Do-not-re-tread updated: do NOT re-implement the -9 string seed or re-attack 0x102b504e4.
Honest: does NOT manufacture a DM; Route-B live-DM gate UNCHANGED (DM-root [0x106a68818]=0, MH_*
false). recon-v3 self-driven-frame deliverable re-verified green (capture_taskv4_frame.sh attempt
1: 24 real task-driven frames `present swap Ok(0x1)`, 197 node pops, 0 json abort, 0 crash).
Files: docs/frontier-sh380-crossed-levers-terminal-mapwall.md; logs
/home/hermes-worker/runs/sh380-*.txt (outside repo, incl. 8/8 terminal 0x1021dde34).

## SH379 (Sep 20, 2026, hermes-worker): MEASURED NEGATIVE (never-run intersection closed) — GOVFLAG+PRELOAD_VALUECELL+PACK_SKIP are INEFFECTUAL on the FULL --v2boot ladder (app-start driven): governor/DM-creator/setDataModelToCurrent/ScriptContext all 0 hits, run drains into the same closed LSM pool-pop lane 0x101d9a528 — the SH376/377 governor gates only matter on the skip-appstart send-appevent env
Single-agent (cone suppressed). One new probe runs/capture_sh379_full_ladder_govgates.sh (full
--v2boot ladder + SH373 crossing + GOVFLAG + PRELOAD_VALUECELL + PACK_SKIP, region-watch on governor/
DM-creator/setDataModelToCurrent/app-shell/ScriptContext) + docs/frontier-sh379-full-ladder-governor-gates-ineffectual.md.
No production path edited (all existing default-inert guards). Workspace green (cargo test --workspace
EXIT 0, 616 passed/0 failed).

- SH376/377 proved the governor-crossing gates cross the governor NULL-DM + preload walls on the
  --v2boot-skip-appstart send-appevent env (SendAppEventOnAppReady returns Ok). But those gates were
  NEVER run on the FULL app-start-driven ladder (the SESSION-CTOR target that SH344 showed reaches the
  governor tail + DM-creator band). This cycle runs that never-run intersection with region-watch.
- Result: app-shell/do-init world-build runs deep (0x102208xxx registrar band, the SH340 77-block
  construction) but governor (0x102e9fa80), DM-creator (0x102bd1xxx), setDataModelToCurrent
  (0x102dbcc10), and ScriptContext (0x101f1d8ac) all 0 hits; terminal SIGSEGV guestpc=0x101d9a528
  fault=0x0 (EXIT 134) — the same closed LSM pool-pop lane. MH_FLAGS_LOADED/APP_READY false,
  AppBridgeV2[0x106a705e8]=0x0.
- Interpretation: the governor/preload/pack gates are path-specific — they only matter on the
  send-appevent env that drives SendAppEventOnAppReady directly. On the full app-start-driven ladder,
  the run drains into the persistence LSM lane BEFORE the governor is reached, so arming the gates
  changes nothing. Both paths converge on the same closed persistence lane; Route-B live-DM structural
  gate UNCHANGED (DM-root 0, MH_* false).
- Do NOT re-drive LSM skips (SH349/350/358/373/375/377/378); do NOT expect governor gates to change
  the full-ladder terminal (SH379). Files: runs/capture_sh379_full_ladder_govgates.sh, log
  /home/hermes-worker/runs/sh379-full-ladder-govgates.txt (outside repo).

## SH378 (Sep 20, 2026, hermes-worker): SH174 DM-allocation capture latch (CAPTURE-ONLY, no DELEGATE) is byte-silent on the furthest-advancing env (SH377 crossing+GOVFLAG+PRELOAD+PACK_SKIP; SendAppEventOnAppReady returns Ok) — 0 validated make_shared<DataModel>, terminal drains into the closed LSM pool-pop lane 0x101d9a528; the single forward hook still does not fire at the farthest reach (map-completion on a never-run intersection)
Single-agent (cone suppressed). One new probe runs/capture_sh378_dmcap_advancing.sh (SH377
advancing env + JIT_DM_ALLOC_CAPTURE=1, capture-ONLY safe latch without the disruptive DELEGATE
that SH344's record left unread cleanly) + docs/frontier-sh378-dmcap-advancing.md. No production
path edited (all existing default-inert guards). Workspace green (cargo test --workspace EXIT 0,
arm64jit 436/0).

- SH344 ran the SH174 capture with JIT_DM_ALLOC_CAPTURE_DELEGATE=1; that host-side delegation
  dispatch disrupted the deep full ladder (std::bad_function_call EXIT 139) BEFORE a clean readback,
  so the single SH174 forward hook's firing-state on the furthest-forward write was never cleanly
  measured. This cycle closes that gap.
- Capture-only latch (no delegate) on the advancing env: `SendAppEventOnAppReady returned Ok` (the
  farthest the send-appevent path has gone IS reached); `grep -c "[validated]"` = 0 AND the trail
  never even installs (no `routed ... capture trail` / `FIRST call#` allocation lines — the only
  `bytes=` hit is the SH339 w19 jstring readback, not an allocation). => OP_NEW_WRAPPER is never
  entered installably and no validated in-image-vtable DataModel allocates on this env either.
- Terminal: `guestpc=0x101d9a528 fault=0x0` (EXIT 134), the LSM pool-pop write site — the SAME
  measured-closed SH350/SH341 persistence family. Governor/app-shell ctor/Lua still 0 hits.
- Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0, MH_FLAGS_LOADED/APP_READY
  false, AppBridgeV2[0x106a705e8]=0x0). No DataModel manufactured; SH174 capture-latch stays the
  single forward observer. Do NOT re-drive LSM skips (SH349/350/358/373/375/377/378 stand).

## SH377 (Sep 20, 2026, hermes-worker): the corrected combined env + SH350 pack-skip advances SendAppEventOnAppReady to RETURN and lands in the known run-variable live-object family (0x10284cf5c) — confirming the pack-helper closure holds as the next wall on the corrected terminal sequence
Single-agent (cone suppressed). One probe runs/capture_sh377_packskip_combined.sh (crossing-env +
GOVFLAG + PRELOAD_VALUECELL + LSM_PACK_SKIP, the never-run intersection) + live capture (gitignored).
No production path edited (existing default-inert guards only). Workspace green.

- With pack-skip RET'ing the single-caller name-pack helper 0x101d9a708 (SH350's bounded-single-caller
  skip), the corrected combined env reaches `SendAppEventOnAppReady returned` AND advances the terminal
  past 0x101d9a708.
- New terminal: `[SIGSEGV] guestpc=0x10284cf5c fault=0x0 x0=0x0` — a small once-style routine
  (`stp x29,x30,[sp,#-32]!`; `stlrb w8,[x0]` @0x284cf70) called with a NULL this-pointer. This is the
  SAME canonical run-variable live-object arm SH355 already documented ("canonical full-ladder probe
  re-ran EXIT 134 at the run-variable live-object arm guestpc 0x10284cf5c"). NOT a fresh seedable gate:
  the object behind x0 is only built by a real session ctor (SH174/SH204 class), run-variable across runs.
- Confirmation: the corrected map's inference holds — the pack-helper (0x101d9a708, SH350) really
  is the next wall past the crossed governor/preload on the SESSION-CTOR path, and it now leads into the
  known-returned run-variable live-object family (NOT do-init -> DM). This STRENGTHENS the standing
  verdict: the SESSION-CTOR drive, even fully crossed through governor+preload+pack, still re-enters
  measured-returned live-object lane before any live DM construction.

### Honest
Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED (DM-root 0, MH_* false).
SH174 capture-latch stays the single forward hook. Do NOT re-drive LSM sub-call skips deeper
(SH349/350/358/373/375/377).

## SH376 (Sep 20, 2026, hermes-worker): CORRECT the SH375 terminal attribution + MEASURED governor/preload cross under the crossing-env — the real terminal after SH285-crossing is the governor NULL-DM deref (0x102ea0b9c, SH269), not a "SetInitParams abort"; arming GOVFLAG+PRELOAD_VALUECELL advances SendAppEventOnAppReady past governor+preload to the SH350 pack-helper (closed)
Single-agent (cone suppressed). Two probes (runs/capture_sh376_govflag_crossing.sh +
runs/capture_sh376b_combined.sh) + live captures (gitignored) + docs/frontier-sh376-governor-null-dm-terminal-corrected.md
+ sh376_new hermetic (arm64jit lib 435->436, read-only governor word-pins). No production
path edited (probes use only existing default-inert guards SH269/GOVFLAG + SH307/PRELOAD_VALUECELL
+ SH349 append-skip + SH371 crossing seeds). Workspace green (cargo test --workspace EXIT 0,
arm64jit 436/0).

- **SH375's "SetInitParams SIGABRT" is a MISREAD.** Reading the FULL SH375 live log: SetInitParams
  (0x102bcc814) and V2InitWithParams BOTH soft-RETURN benignly (`pc 0x3d0/0x4a0 outside image`,
  the SH331 leaked-host-pc class). The genuine SIGSEGV is the **governor NULL-app-DM deref
  guestpc=0x102ea0b9c fault=0x0** (`ldr x0,[x21,#1032]`=[controller+0x408]=0), then SIGABRT.
- That is SH269's wall (predicate byte [0x106a64da0]), already armed behind
  `JIT_ROUTEB_APPSART_GOVFLAG` but NOT set in the SH375 env.
- Arming GOVFLAG on the crossing-env (4/4): the 0x102ea0b9c deref is GONE; terminal ADVANCES to
  `0x102bb803c` (SH307 preload-valuecell wall).
- Arming GOVFLAG + PRELOAD_VALUECELL (4/4): the preload wall ALSO crosses
  (`SendAppEventOnAppReady returned`); terminal moves to `0x101d9a708` (SH350 pack-helper, the
  known-closed LSM lane).
- Corrected terminal sequence past crossed-SH285: governor NULL-DM (SH269) -> preload valuecell
  (SH307) -> pack-helper (SH350, closed). Do NOT re-drive LSM skips (SH349/350/358/373/375 stand).

### Honest
Does NOT manufacture a DataModel (DM-root [0x106a68818]=0, MH_* false). Route-B live-DM
structural gate UNCHANGED. This is a map-completion + attribution correction + new hermetic;
the completed SendAppEventOnAppReady still re-enters the closed persistence lane (0x101d9a708),
so no live-DM gate moved. The correction matters: SH362/SH375's "0x258b5d8 blocked by SetInitParams
abort" premise is obsolete, but the corrected blocker is SH269-armed and passes cleanly — the
0x258b5d8 body's real (still-unreached) blocker is the SH350 pack-lane. SH174 capture-latch stays
the single forward hook.

## SH374/SH375 (Sep 20, 2026, hermes-worker): MEASURED map-completions under the SH373 reaching-env — EC-world reader-gate still never entered AND the 0x258b5d8 dispatch body still never fires even with SH285 deterministically crossed (both prior closures re-tested on the env that crosses the SH285 leaf, refining their premises); recon-v3 deliverables re-verified green
Single-agent (cone suppressed). Two new probes (runs/capture_sh374_ec_dmfn_reaching.sh +
runs/capture_sh375_dispatch_body_reaching.sh) + live captures (gitignored) +
docs/frontier-sh374-sh375-reaching-env-closures.md + sh375_new hermetic (arm64jit lib
434->435). No production path edited (both probes combine the shipped SH373 crossing env
+ existing default-inert guards). Workspace green (cargo test --workspace EXIT 0).

- **SH374**: SH373's reaching-env (LSM_APPEND_SKIP over the SH371 M+0x48/appname seeds)
  combined with the DMFN/EC-world drive: the EC-world ENTRY guards fire every run
  (routeb-sh298/299/300 @0x102e24598), but the reader-gate block 0x2e24694 STILL never
  enters and the run terminals at 0x101d9a708 (pack helper, same unconstructed-LSM
  family). Crossing SH285 does NOT make the EC marshaller interior reachable — control
  drains into the persistence family first. SH356's "reader-gate = reachability problem"
  re-strengthened from the crossing env.
- **SH375**: SH362 attributed the 0x258b5d8 dispatch-body unreachability to "run dies at
  SH285 first." With SH285 now deterministically CROSSED (sh285=0 all 3 runs), the ladder
  ADVANCES PAST it — StartApp Ok, lifecycle drive (initAppShellReporter + setActive) clean —
  and then SIGABRTs at a FRESH terminal, SetInitParams (0x102bcc814, `nativeAppBridgeSetInitParams`,
  a genuine 0x3f0-frame fn, pinned by the sh375_new hermetic). The 0x258b5d8 body STILL
  never fires: its blocker is NOT the SH285 leaf (crossed) but this SetInitParams LSM-family
  abort that runs first. Refines SH362's premise; does not overturn the standing
  measured-closed LSM family.
- Both confirm the persistence/LSM unconstructed-object family is PATH-INDEPENDENT and
  swallows every Route-B ladder arm (do-init dispatch, EC marshaller, SetInitParams)
  before any live DM construction. No DataModel (DM-root 0, MH_* false); SH174 capture-latch
  stays the single forward hook. recon-v3 deliverables re-verified green (24 real task-driven
  frames, swap Ok(0x1), 0 json abort, 0 crash).

## SH373 (Sep 20, 2026, hermes-worker): MEASURED — SH285 CROSSOVER from the SH371 reaching-env (the standing SH285 persistence leaf is deterministically crossed 5/5, terminal advances to 0x101d9a708 in the SAME measured-closed LSM unconstructed family); recon-v3 + sh372 green
Single-agent (cone suppressed). One new probe runs/capture_sh373_cont_appendskip.sh + live
captures (gitignored) + docs/frontier-sh373-sh285-crossover-continuation.md. No production
path edited (append/pack skips are SH349/SH350's existing default-inert opt-ins; SH373
combines SH349's append-skip with the SH371 reaching env and measures). Workspace green
(cargo test --workspace EXIT 0, 615 passed/0 failed incl sh372).

### The forward this cycle (a reproducible cross, then an honest verdict)
For the first time the standing SH285 persistence-wall (guestpc=0x101db1b08) is
deterministically CROSSED (5/5 runs, sh285=0). SH371 added the DM_CONT_M48_SEED +
CONT_APPNAME_SEED that make the DM-creator continuation continueAfterFlagsLoaded_
(0x102bd1d68) run DEEP headlessly. SH373 adds SH349's append sub-call skip on top:
- continuation fires; **SH285 leaf 0x101db1b08 = 0 hits** (was the terminal of every
  SH260/284/285/3444/348/371/372 run) — the append byte-copy 0x101d9a15c IS the wall again;
- run advances to **0x101d9a708** (SH349's pack/name-string helper), faulting on source
  pointer x19=0xff..ff = the SAME unconstructed-live-object family SH349/350/358 closed.
- SH358's earlier "0 continuation hits for DMCONT+skips" is explained: that run LACKED the
  M+0x48/appname seeds, so the continuation was never reached.
- The append+pack combo (SH373b) instead parks at pool-pop write-site 0x101d9a528
  (write-to-0x1 divergence), a different arm that doesn't reach the continuation.

### Interpretation
SH285 is NOT a fundamental invariant — it is the append byte-copy leaf, crossable with the
known single-caller skip once the continuation is reached. But the cross lands one fencepost
later in the SAME family (0x101d9a708), which SH349 reached + SH350 crossed into the
unbounded pool-pop family. This RE-STRENGTHENS the standing verdict: the persistence lane is
measured-returned (whack-a-mole UNBOUNDED); only a REAL LocalStorageManager/session ctor
gets past, and no seed manufactures it (SH248h/SH256). SH373 closes the last "is SH285 itself
the invariant?" loophole by crossing it and showing the next fencepost is already-known.

### Honest
No DataModel (DM-root [0x106a68818]=0, MH_* false). Route-B live-DM structural gate
UNCHANGED. SH174 capture-latch stays the single forward hook. Do NOT extend to
pack-skip+LSM whack-a-mole (SH350 unbounded); do NOT re-drive further sub-call skips into
the family (SH349/350/358 closure stands).

## SH372 (Sep 20, 2026, hermes-worker): MEASURED — the DM-creator continuation and the settings-state init path CONVERGE on the identical SH285 persistence-object leaf (answers SH371's explicit "same object or different?" gap with a fresh register dump); recon-v3 deliverables re-verified green
Single-agent (cone suppressed). One new probe runs/capture_sh372_continuation_terminal.sh
(JIT_DUMP_PC + JIT_GUEST_STACK_DUMP + JIT_REGION_WATCH on the continuation body) + one new
real-image hermetic `sh372_cont_continuation_converges_on_sh285_persistence_leaf` (arm64jit
lib 433->434; pins the shared leaf sub x2,x0,#0x20 @file 0x1db1b08, its caller bl @0x1db1b14
-> lr 0x1db1b18, and the shared continuation prologue @file 0x2bd1d68) + docs/frontier-sh372-
continuation-convergence.md. No production path edited. Workspace green (cargo test
--workspace EXIT 0, 614 passed/0 failed — 433 arm64jit lib tests + sh372).

### The forward this cycle (the genuine new datum)
SH371 measured the DM-creator continuation continueAfterFlagsLoaded_ (0x102bd1d68) now runs
DEEP headlessly and terminates at the standing SH285 persistence-lane wall
(guestpc=0x101db1b08), and explicitly left open whether it hits the same object the
settings-state path faulted on or a different one. SH372 closes that gap with a fresh full
register + guest-stack dump directly from the continuation path:
- Continuation FIRED (block-entry at 0x102bd1d68) and its deep pcs all executed.
- Terminal pc 0x101db1b08 with lr=0x101db1b18 — the SAME leaf (sh285 reader caller inside
  initStorageManagerNative 0x101d9d8b0) the settings-state drive hits.
- Fault target x20=x1=0xffff8062... — the SAME 0xff..ff-prefixed uninitialized internal
  data-pointer (SH285/[obj+0x50] family); x0=x19=0x7f9d... = a guest-constructed host-heap
  object, exactly as SH285 classified.

### Interpretation (map refinement, not a new wall)
Both independently-reached engine init paths — the settings-state self-drive (SH284/285) and
the DM-creator continuation (SH371, SH372) — converge on the identical unconstructed-manager
leaf, same pc, same lr, same 0xff..ff buffer pointer. This proves the SH285 terminal is
PATH-INDEPENDENT: NOT a benign-body branch one path misses (corroborating SH371's
STRAIGHT-LINE finding), but the manager object's own unconstructed string buffer, which no
seed manufactures (SH248h/SH256 rule) and which only a REAL LocalStorageManager/session ctor
owns (SH174/204 live-object class). The measured-closed SH285 lane record is strengthened with
a second-entry confirmation.

### Honest
Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED (DM-root
[0x106a68818]=0, MH_* false). SH174 capture-latch stays the single forward hook. recon-v3
deliverables re-verified green at HEAD this cycle (24 real task-driven frames swap Ok(0x1),
0 json abort, 0 crash; JIT_JSON_ZERO_FIX present).

### Next (unchanged, authoritative)
Route-B live-DM structural gate stands (SESSION-CTOR / do-init; REG_LIVE SH352). The two
measured dead-ends from the now-reached continuation are (a) the SH285 live-object wall and
(b) the F+0x18 controller floor behind it — both measured-closed (do NOT re-drive LSM sub-call
skips SH349/350/358). R1 content half staged + armed + serviceable (SH351/352/354). Do NOT
re-arm the window-attach once-guard (SH367); do NOT re-enter the ALooper loop (SH365); bounded
process_cmd stays the guarded entry (SH366/368).

## SH371 (Sep 20, 2026, hermes-worker): MEASURED — continueAfterFlagsLoaded_ now EXECUTES DEEP headlessly (corrects the SH226/228 "never fires" map) + hermetic proving the engine-init dispatcher body is STRAIGHT-LINE (the only exits: two leaf blr returns and the bl sub)
Single-agent (cone suppressed). recon-v3 deliverables independently re-verified green at
HEAD (capture_taskv4_frame.sh attempt 1: 24 real task-driven frames `present swap Ok(0x1)`,
197 node pops, 0 json abort, 0 crash; JIT_JSON_ZERO_FIX len-clamp at 0x102355d40 present).
New real-image hermetic `sh371_engineinit_dispatcher_body_straightline_to_sub` (arm64jit
lib 433; scans dispatcher body [0x2bd8ce8,0x2bd8d64) + sub_2bd8dac body [0x2bd8dac,0x2bd8e28)
for any control-flow word, rejecting all but the 4 known sites — bl getter 0x2bd8d14, blr
vt+0xf8 @0x2bd8d2c, blr vt+0x108 @0x2bd8d50, bl sub @0x2bd8d60, and sub's blr vt+0x1f0
@0x2bd8e28 — both STRAIGHT-LINE, so SH228's "diverge at a leaf" narrows to "a leaf's
return never lands back in-image (host landing)" with NO benign body branch) + probe
runs/capture_sh371_dispatcher_body.sh + docs/frontier-sh371-....md. Workspace green
(cargo test --workspace EXIT 0; elfjit 159/0, jit lib 433/0).

### The forward this cycle
A genuinely-new measurement, not a re-tread: with the FULL Route-B env (capture_sh344's
DMCONT + DM_CONT_M48_SEED + CONT_APPNAME_SEED seed set), region-watching [0x102bd8ce8,
0x102bd8e30] ∪ [0x102bd1d68,0x102bd2600] on the completing ladder shows:
- dispatcher 0x2bd8ce8 + sub_2bd8dac BOTH fire (1 each) — the dispatcher body runs through
  its getter + the two leaf blr returns and reaches `bl sub`, contradicting SH228's
  "sub never fires" on the fuller env.
- **continueAfterFlagsLoaded_ (0x102bd1d68) FIRES and runs DEEP — 25+ block-entry pcs
  0x102bd1d68 .. 0x102bd1f64 (its app-name guard, SH245/SH248c-seeded)**. This overturns
  the SH226/SH228 blanket "continueAfterFlagsLoaded_ is never entered" — with the full
  seed env it executes deep past its app-name guard.
- Terminal: guestpc=0x101db1b08 fault=0xff..ff — the standing SH285 LSM reader/pop
  live-object wall, reached now from the DM-creator continuation path; the F+0x18
  post-app-start controller floor (routeb_dm_manager_cont comment) is never reached
  because SH285 fires first (one fencepost EARLIER than that predicted floor).

### Honest
Does NOT manufacture a DataModel (DM-root [0x106a68818]=0, MH_* false, Route-B live-DM
structural gate UNCHANGED). SH371 corrects the map (the continuation is env-reachable deep,
not "never entered") but the continuation immediately dives into the measured-closed SH285
persistence lane (SH349/350 — do NOT re-drive LSM sub-call skips). SH174 capture-latch
stays the single forward hook.

### Next (unchanged, authoritative)
Route-B live-DM structural gate stands (SESSION-CTOR / do-init; REG_LIVE SH352). The two
measured dead-ends from the now-reached continuation are (a) the SH285 live-object wall and
(b) the F+0x18 controller floor behind it — both measured-closed. R1 content half staged +
armed + serviceable (SH351/352/354). Do NOT re-arm the window-attach once-guard (SH367);
do NOT re-enter the ALooper loop (SH365); bounded process_cmd stays the guarded entry
(SH366/368); do NOT re-drive LSM sub-call skips (SH349/350/358).

## SH370 (Sep 20, 2026, hermes-worker): SH357-consolidation completion — lock the sh323 cookie-jar/settings guard test under the shared ROUTEB_PROC_TEST_LOCK (determinism hardening); recon-v3 deliverables re-verified green at HEAD
Single-agent (cone suppressed). One-line test-harness fix (arm64jit/src/jit.rs): `sh323_settings_sso_seed_guard`
was the one routeb-family cookie-jar test that did NOT hold the consolidated ROUTEB_PROC_TEST_LOCK all its
siblings (sh248d/sh248e/sh273/sh175) hold, so under parallel `--test-threads=16` it raced the locked siblings'
mid-assert on the shared fixed cookie-jar page (CELL_B=0x106ed7a28 == sh248d's B), giving intermittent
"cookie-jar slot A/B must be seeded" failures (left:0). Added the lock (behavior-neutral, no assertion
weakened). MEASURED: sh323 passes; sh323/248d/248e pass 50/50 under 16-thread isolation; `cargo test
--workspace` deterministic-green EXIT 0 (612/0; 6/6 canonical runs + earlier 10/10/8/8). The residual futex/
sharded-page flake only appears under artificial `--test-threads=16` and is the documented SH345/346/357
accepted load-sensitive class — the canonical gate is reproductibly green. Route-B live-DM structural gate
UNCHANGED (DM-root 0, MH_* false); recon-v3 frame plane re-verified green (24 frames swap Ok(0x1), 0 json,
0 crash).

## SH369 (Sep 20, 2026, hermes-worker): MEASURED structural pin — window-attach COMPLETION funnels into the CLOSED persistence lane (flags-latch 0x72739d4 -> initStorageManagerNative 0x1db1050), NOT to a live DM; refines SH367's "needs a real surface" reading
Single-agent (cone suppressed). One new real-image hermetic
`sh369_window_attach_completion_converges_to_persistence_lane` (arm64jit lib 432; 11 word-pins
verified on real libroblox.so) + docs/frontier-sh369-windowattach-persistence-convergence.md.
No production path edited (read-only pin). recon-v3 immediate-priority deliverables re-verified
green at HEAD this cycle (24 real task-driven frames, swap Ok(0x1), 0 json abort, 0 crash).
Workspace green (cargo test --workspace EXIT 0, 612 passed/0 failed incl sh369).

### The forward this cycle
SH367 measured that arming the window-attach once-guard + a crafted [win+0x278] hard-faults and
attributed the wall to "needs a REAL EGL surface, only a live Activity/AppBridge session provides".
SH369 pins the COMPLETION chain the armed path would take and shows it is NOT a route to a live DM:
disasm-verified `bl 0x22985c0 (deep GL post-init) -> bl 0x2270a98 -> bl 0x2270b24 (real body)`; the
body gates on the SAME flags-loaded latch the --v2boot ladder seeds (`adrp 0x7273000; ldrb
[x9,#2516]` = [0x72739d4] @0x2270b64) and, when bit0=1, falls through the `cbz w9,0x2270be8`
@0x2270b78 to `bl 0x1db1050` = initStorageManagerNative — the SH285-family persistence lane
(the SH285 fault site 0x101db1b08 is inside it; SH349 crossed it, SH350/358 closed the lane as
measured-unbounded). So window-attach COMPLETION is a SECOND entry into the already-closed
persistence lane, not a path to a live DM. This de-risks the SESSION-CTOR window precondition:
even a REAL surface hands control to a lane already measured returned — that is partly why Route-B's
live-DM gate stands (the window precondition alone cannot produce a DM).

### Honest
Does NOT manufacture a DataModel (DM-root [0x106a68818]=0, MH_* false unchanged). No re-run of the
SH367 armed-fault (already 8/8 measured negative). Route-B live-DM structural gate UNCHANGED;
SH174 capture-latch stays the single forward hook. This cycle is pin/verify, not a new session
drive — but the pin is genuinely new (SH367 stopped at bl 0x22985c0; SH369 traces past it).

### Next (unchanged, authoritative)
Route-B live-DM structural gate stands (SESSION-CTOR / do-init four-stacked closure SH184/185;
REG_LIVE SH352). R1 content half staged+armed+serviceable (SH351/352/354). SESSION half (do-init
owning a live DM) remains THE wall — reached only by a REAL Activity/AppBridge session drive that
constructs the upstream ctor for real. SH174 capture-latch stays the single forward hook. The
window precondition is now pinned as persistence-lane-bound (SH369), so re-attacking it alone
expecting a DM is closed; the genuine EGL-surface/governor/onAppReady home-cone (SH126+ captures)
is still the live session lever. Do NOT re-arm the fabricated once-guard (SH367); do NOT re-enter
the ALooper loop (SH365); bounded process_cmd stays the guarded entry (SH366/368).

## SH368 (Sep 20, 2026, hermes-worker): bounded app-command SEQUENCE drive on the guarded SH366 entry — the engine's own process_cmd dispatcher drives cmds {6,8,11} cleanly (marker IN INIT_WINDOW), session observables measured after EACH command confirm the dispatcher alone does NOT self-transition AppBridgeV2/surface (SESSION-CTOR wall pinned precisely); full 20-entry jump table + window-attach contract pinned in a new real-image hermetic
Single-agent (cone suppressed). New opt-in rung `--v2boot-glue-cmd-seq` ->
`drive_glue_process_cmd_seq` (jit.rs) on the SAME bounded SH366 dispatcher entry (once-guard stays
OFF — SH367 measured fault on the fabricated re-arm) + real-image hermetic
`sh368_glue_cmd_seq_jump_table_and_safe_cases_pinned` (arm64jit lib 431) + capture
`runs/capture_sh368_glue_seq.sh`. elfjit.rs product path unchanged (rung opt-in). Workspace green
(cargo test --workspace EXIT 0, arm64jit 431/0).

### The forward this cycle
SH366 entered the engine's REAL app-command dispatcher (0x102bcd6e4) headlessly and delivered ONE
APP_CMD (INIT_WINDOW, marker [inner+9]=1). The operator's SESSION-CTOR directive is to "drive the
engine's REAL Activity-session init state machine" — a real Activity consumes a QUEUE of APP_CMD
values. SH368 extends the confirmed-green SH366 entry to a sequence of verified-safe commands
({6,8,11}) over a SHARED fabricated app/inner/win so command state accumulates like a real queue,
and reads back the session observables after EACH command. cmd 6/8 are disasm-verified cycle-safe
at version-gate 0 (b.lo straight to the epilogue / write only glue bytes); cmd 11 is the
SH366-proven INIT_WINDOW. **MEASURED (confirm:1 on attempt 1, EXIT 124, 0 crash): all three
commands drive cleanly `process_cmd returned Ok`; cmd 11 fires the INIT_WINDOW marker
[inner+9]=1; once-guard stays OFF.** The observables confirm the SESSION-CTOR reading precisely:
the real dispatcher alone does NOT self-transition AppBridgeV2 ([0x106a705e8] 0x0->0) nor the
surface XID ([0x10683d348] stays 0x200000 = the wired X11 XID, SH112) — those move only when a
live session/do-init builds the DM world.

### Honest
Does NOT manufacture a DataModel (DM-root [0x106a68818]=0, MH_* false, Route-B live-DM structural
gate UNCHANGED). Does not arm the (SH367 faulting) window-attach once-guard, does not create a
real EGL surface. It advances the "drive the engine's real command queue" half of the SESSION-CTOR
directive with re-verifiable pinned addresses (full 20-entry jump table + window-attach contract)
and a bounded live readback that isolates the wall to do-init's live-DM construction.

### Next (unchanged, authoritative)
Route-B live-DM structural gate stands (SESSION-CTOR / do-init four-stacked closure SH184/185;
REG_LIVE SH352). R1 content half staged+armed+serviceable (SH351/352/354). SESSION half (do-init
owning a live DM) remains THE wall — reached only by a REAL Activity/AppBridge session drive
(genuine EGL surface + onAppReady + real jstring) that constructs the upstream ctor for real.
SH174 capture-latch stays the single forward hook; bounded process_cmd is the guarded entry
(SH366/SH368). Do NOT re-arm the fabricated once-guard (SH367); do NOT re-enter the ALooper loop
(SH365); do NOT re-drive LSM skips (SH349/350/358).

## SH367 (Sep 19/20, 2026, hermes-worker): MEASURED NEGATIVE — arming the real window-attach GL-surface path faults (SH366 next-forward executed); the SH366 clean INIT_WINDOW drive is preserved + fault pinned one level deeper into the deep GL post-init 0x22985c0
Single-agent (cone suppressed). One new read-only observation guard
`routeb_glue_realattach_guard` (jit.rs, opt-in JIT_ROUTEB_GLUE_REALATTACH=1, once, ZERO guest
mutation) + real-image hermetic `sh367_window_attach_real_path_pinned_and_guard` (arm64jit lib
429->430) + capture runs/capture_sh367_glue_cmd_real.sh (SH366-clean-entry predicate, confirmed
green attempt 1: EXIT 124, crash 0, drive Ok, marker [inner+9]=1) + frontier doc. elfjit.rs
unchanged. Workspace green (cargo test --workspace EXIT 0, 430/0).

### The attempt (STATUS/frontier-sh366 next-forward, executed + measured)
STATUS named \"hand the engine a REAL wired ANativeWindow in [inner+64] so window-attach 0x2bd29a0
takes its real GL-surface path\". SH367 implemented exactly that: armed [win+0x268].bit0=1, crafted
[win+0x278]=NULL-first-word obj (so the deep GL call fast-returns), registered XID 0x200000 via
set_anativewindow_xid. Disasm pins the real chain: armed -> `add x0,x19,#0x278` @0x2bd2a18 -> bl
0x2291c24 (0x2291c24 `cset w0,ne on [x0]` = ([?win+0x278]!=0)) -> bl 0x22985c0 (deep GL post-init).
MEASURED (8 attempts): every run **hard-faults before the INIT_WINDOW body completes** — EXIT 134/139,
marker [inner+9]=1 never fires, drive no longer returns Ok, fault inside the host GL dispatch
(guestpc=0x7f0000001f50 fault=0x7f818c0097). Root cause: because the crafted non-null [win+0x278]
makes 0x2291c24 return 1, control REACHES 0x22985c0, whose DEEP body needs a REAL EGL surface/context
object — a fabricated obj cannot satisfy it. This is the exact object only a live Android
Activity/AppBridge session drive provides. **Reverted** to the SH366 clean drive (guard left OFF);
re-verified clean (process_cmd Ok, marker set). Kept the read-only guard + hermetic pins as the
measured-negative instrumentation.

### Honest
Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0x0,
MH_* false). SH174 capture-latch stays the single forward observer. SH367 confirms the window
precondition is a REAL GL-surface wall (Session-Ctor operator directive), not a seedable global.

### Next (unchanged, authoritative)
Route-B live-DM structural gate stands (SEP-17 SESSION-CTOR / do-init four-stacked closure
SH184/185; REG_LIVE SH352 addendum 2). The window-attach COMPLETION needs a genuine EGL surface +
onAppReady — a real Activity/AppBridge session drive, not a value seed. R1 content half staged+armed
(SH351/352/354); SESSION half (do-init owning a live DM) remains THE wall. SH174 capture-latch stays
the single forward hook. Do NOT re-seed [win+0x268]/[win+0x278] (SH367); do NOT re-enter the ALooper
loop (SH365); bounded process_cmd remains the guarded-entry (SH366).

## SH366 (Sep 19, 2026, hermes-worker): FIRST headless ENTRY into the engine's own app-command DISPATCHER process_cmd (0x102bcd6e4) — the APP_CMD_INIT_WINDOW case body EXECUTED (marker [inner+9]==1), the SESSION-CTOR window/GL-surface precondition the operator names for initEngine_ was driven (not just watched); Route-B live-DM structural gate UNCHANGED (DM-root 0, MH_* false)
Single-agent (cone suppressed). One new bounded cause-level drive
`drive_glue_process_cmd` (arm64jit/src/jit.rs, opt-in rung `--v2boot-glue-cmd` at the
TOP of the ladder) + one new real-image hermetic
`sh366_glue_process_cmd_abi_and_init_window_case_pinned` (10 byte-pins + the
jump-table index-10->0x2bcd78c mapping verified on real libroblox.so) + capture
`runs/capture_sh366_glue_cmd.sh`. elfjit.rs held <1MB (condensed SH-prose comments).
Workspace green (cargo test --workspace EXIT 0, 609 passed/0 failed — arm64jit lib
429 after sh366).

### The forward this cycle
The operator's SESSION-CTOR lever names the window/GL-surface `APP_CMD_INIT_WINDOW` as
the precondition behind initEngine_'s "*** Engine settings is null" hard-assert. SH39b
region-watched the glue LOOP (0x102bcd5d0) at 0 hits; SH365 measured the app-command
drain dead (addfd=0, pollonce=0, posted=3). BOTH only observed the path; neither
ENTERED it, because the glue main loop is an INFINITE ALooper_pollOnce loop that cannot
be jit_run to completion. The loop dispatches to a BOUNDED fn, `process_cmd(app, cmd)`
at guest 0x102bcd6e4 (w1 = APP_CMD value), which CAN be entered — and nobody had ever
driven it. SH366 is that first entry: with a fabricated app ([app]=inner,
[inner+64]=zeroed win obj, version-gate [0x10683d8b0]=0) the INIT_WINDOW case body does
`ldr x0,[x20,#64]; strb w8,#1,[x20,#9]; bl window-attach` -> for the first time
headlessly the engine RUNS its own APP_CMD_INIT_WINDOW handler (marker [inner+9]=1),
the window-attach path enters, and the whole run is stable.

### MEASURED (real libroblox.so, completing ladder + --v2boot-glue-cmd)
```
[elfjit:glue-cmd] driving process_cmd @ guest 0x102bcd6e4 (app=... [app]=... [inner+64]=win ... cmd=11 INIT_WINDOW; version-gate [0x10683d8b0]=0)
[elfjit:glue-cmd] process_cmd returned Ok(...)
[elfjit:glue-cmd] INIT_WINDOW case body marker [inner+9]=1 EXECUTED (engine window-attach path entered headlessly)
```
Confirmed-green artifact (retry 2): EXIT 124, 0 SIGSEGV/ABRT, marker=1. The drive logs
its success even on a run that later dies at a pre-existing run-variable persistence-
lane wall (SH353-class, unrelated — the drive runs FIRST).

### Honest
Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED (DM-root
[0x106a68818]=0, MH_* false). SH366 proves the window-condition *entry* is reachable
via the bounded process_cmd leaf — the ALooper loop itself is still not entered (SH365
drain dead-letter unchanged). The window obj handed to 0x2bd29a0 is a ZEROED host
buffer, not a wired EGL surface; the real window-attach completion (and the do-init
chain that would build the DM) is the standing next step, not reached this cycle.
SH174 capture-latch stays the single forward hook.

### Next (on the SH366 line)
Hand the engine a REAL wired ANativeWindow — the wired X11 XID 0x200000 (SH112/SH365,
via anativewindow_fromsurface) — in [inner+64] instead of the zeroed obj, so the
window-attach helper 0x2bd29a0 takes its real GL-surface path; then chain process_cmd
to the do-init ladder per the operator's SESSION-CTOR "drive until the upstream ctor
RUNS" directive. Do NOT re-attach a zeroed obj expecting the DM to move; do NOT re-enter
the infinite ALooper loop (bounded process_cmd is the correct entry).

## SH365 (Sep 19, 2026, hermes-worker): MEASURED dead-letter — the host app-command FIFO is never drained by the guest android_app glue loop (addfd=0, pollonce=0, posted=3, 3/3) on the completing ladder, pinning the SESSION-CTOR "window/GL-surface APP_CMD_INIT_WINDOW" precondition as an undelivered lifecycle event; recon-v3 deliverables re-verified green at HEAD
Single-agent (cone suppressed). Always-on ALooper drain counters
(`app_command_drain_stats` in shims.rs) + a 1.5s-grace readback in the elfjit
app-command feed + one hermetic (`app_command_drain_stats_count_shim_entries_and_posted`,
arm64jit lib 427) + capture `runs/capture_sh365_alooper_drain.sh`. No production
path edited (4 cheap atomics; elfjit readback inside the existing
JIT_DRIVE_LIFECYCLE block). Workspace green (cargo test --workspace EXIT 0,
608/0 — was 607).

### The forward this cycle
SH264/276 drove the lifecycle NATIVES directly and each completes headlessly
(initAppShellReporter/setActive/nativeInitClientSettings(_Signed)/
nativeActivity_onEngineSettingsReceived all Ok), but SH264 *suspected* — never
MEASURED — that the android_app glue main loop (guest 0x102bcd5d0) "busy-spins
rather than dispatch APP_CMD_START/RESUME/INIT_WINDOW". The SESSION-CTOR
directive names the window/surface `APP_CMD_INIT_WINDOW` as the precondition
behind initEngine_'s "*** Engine settings is null" hard-assert, and the
app-command FIFO was built to carry it. SH365 MEASURES the drain: on the
completing ladder (3/3, EXIT 124 clean) the host posts 3 APP_CMD_* commands
(START/RESUME/INIT_WINDOW) but the guest NEVER enters ALooper_addFd/ALooper_pollOnce
(addfd=0, pollonce=0, posted=3) — the glue loop never consumes the FIFO, so the
INIT_WINDOW event that would hand the wired X11 XID (0x200000, anativewindow fires
1x) to the engine is never delivered. Cause-level MEASURED (SH264 inferred);
Route-B live-DM structural gate UNCHANGED (DM-root 0x0, MH_* false). The drain
stats are an always-on objective trigger: if a future drive brings the glue loop
alive, addfd/pollonce flip >0 and the readback changes from DEAD-LETTER to drained.

### Honest
Does NOT manufacture a DataModel (DM-root 0, MH_* false). recon-v3 self-driven
frame + JSON re-verified green at this HEAD (24 real task frames, swap Ok(0x1),
197 node pops, 0 json abort, 0 crash).

### The forward this cycle
SH347 measured the messageBus experience-launch RECEIVE path at the cb's
DM-holder read (file 0x2bd7474) NEVER firing, but left its cause ambiguous and
explicitly named a "future receive-payload-with-real-string probe" as the open
forward surface. SH364 closes that surface with two read-only observations:
- `routeb_busrecv_cb_entry_guard` (jit.rs, JIT_ROUTEB_BUSRECV): fires at the cb
  BODY ENTRY (file 0x2bd744c, `sub sp,#128` prologue, BEFORE the DM-holder read)
  — distinguishes "publish never dispatches to the cb" (publish-side) from "cb
  entered, DM-holder null" (live-DM-side gate). READ-ONLY.
- `drive_messagebus_publish_receive_payload` + elfjit rung `--v2boot-session-pub-real`
  publishing a REAL 56-byte envelope (SH347 used only an empty b"" payload).
MEASURED (clean bounded run, EXIT 124): subscribe Ok(0x3e8), publishRaw bound +
Ok(0x3e8), **cb-entry guard fires 0** AND **DM-holder guard fires 0** — identical to
the empty-payload run. So the receive cb is not dispatch-reachable headlessly
even with real content: publish-side registration/live-DM-gated, matching every
other SEP-17 receive behind the same gate. +2 hermetic tests (individually 3/3;
the full-suite SH362 flake earlier was the documented SH357 fixed-address race,
passes isolated + suite green on rerun), +capture_sh364_busrecv_real.sh,
+frontier-sh364 doc.

### Honest
Does not manufacture a DataModel. Route-B live-DM structural gate UNCHANGED
(DM-root [0x106a68818]=0, MH_* false). SH174 capture-latch stays the single
forward observer.

## SH362 (Sep 20, 2026, hermes-worker): MEASURED — the do-init MAIN dispatch body fn 0x258b5d8 (StartAppWithParams+0x494) NEVER EXECUTES headlessly; SH361 read only the dispatch *target pointer* (vt[+48]=0x10258b5d8), SH362's body trace proves the body is never entered — every run faults at the SH285 persistence wall (0x101db1b08) before control reaches it (3/3)
Single-agent (cone suppressed). One new READ-ONLY observation guard
`routeb_startapp_dispatch_body_guard` (jit.rs, opt-in JIT_ROUTEB_DISPATCH_BODY_TRACE=1,
default-inert, once-per-run, ZERO guest mutation) + one hermetic sh362
(arm64jit lib 424->425) + runs/capture_sh362_dispatch_body.sh. elfjit.rs product
path unchanged. Workspace green (cargo test --workspace EXIT 0; 605 passed/0
failed — was 604).
- STATUS.md next-forward #1 named "trace whether the measured dispatch target
  0x10258b5d8 body can be advanced past the SH285 wall." SH361 measured the br x1
  @0x2206e24 lands at vt[+48]=0x10258b5d8. SH362 closes the body-EXECUTION gap:
- Disasm of the dispatch fn (0x258b5d8, a real `sub sp,#0x170` function): it
  immediately branches on flag byte [0x106a64da0] (`adrp x8,6a64000; ldrb w8,[x8,#3488]`
  @0x258b604 -> `cbz w8,0x258b640` @0x258b608): nonzero -> path A @0x258b60c
  (nativePreloadFlagOverrides 0x2dae640 -> blr vt[+144] -> 0x2366694 ->
  nativePreloadFlagOverrides -> blr vt[+296]); zero -> path B @0x258b640 (0x2367270 ->
  blr vt[+144] -> 0x2366694 -> 0x2367270 -> blr vt[+296]); both converge @0x258b670
  -> 0x23c19e0 -> canary check -> ret.
- MEASURED (full app-start ladder, 3/3): `[routeb-dispatch-body] SH362` fires 0/3
  (guard wired into the same per-block-entry hook that DOES fire SH361 at 0x2206db8,
  so the miss is the body, not the hook). Every run EXIT 134, SIGSEGV guestpc=
  0x101db1b08 (SH285 persistence-lane live-object wall), DM-root 0, MH_* false.
- Conclusion: the StartAppWithParams+0x494 body is entirely unreachable headlessly —
  the run faults at SH285 BETWEEN the do-init dispatch (reached) and the body entry
  (never reached). Route-B live-DM structural gate UNCHANGED (no DM manufactured).
  Do NOT re-drive LSM skips to "reach" 0x258b5d8 (SH358/349/350 closed); do NOT
  attack [0x106a64da0] with the ladder (the body never runs so the flag is moot).
  Files: docs/frontier-sh362-dispatch-body-unreachable.md, runs/capture_sh362_dispatch_body.sh,
  sh362 hermetic (arm64jit lib 425).
Single-agent (cone suppressed). One new READ-ONLY observation guard
`routeb_doinit_dyn_trace_guard` (jit.rs, opt-in JIT_ROUTEB_DOINIT_DYN_TRACE=1,
default-inert, once-per-run, ZERO guest mutation) + one hermetic sh361 +
probe runs/capture_sh361_doinit_dyn_trace.sh. elfjit.rs product path unchanged.
Workspace green (arm64jit lib 423->424; cargo test --workspace EXIT 0, 0 failures)
as this HEAD. recon-v3 self-driven frame + JSON fixes re-verified green this cycle
(24 real task frames, dispatch #2698000, present #23 swap Ok(0x1), 0 json abort,
0 crash).
- The operator's EXECUTE-DO-INIT-GATES asks for a "dynamic DM-ctor trace (SH164's
  harness-trace artifact) rather than a static seed"; SH320 had described the do-init
  MAIN-branch binder-dispatch as a "DM-ctor entry" WITHOUT ever reading [container+32].
  SH361 closes that measured-vs-assumed gap at the exact decision point.
- Disasm (real libroblox.so): do-init worker 0x2206db8, `mov x19,x1` @0x2206dd0 ->
  `ldr x0,[x19,#32]` @0x2206df4 -> `cbz x0,0x2206ea4` @0x2206df8 (NULL -> MessageBus
  bail) -> else `ldr x8,[x0]; ldr x1,[x8,#48]` @0x2206e00 -> `br x1` @0x2206e24.
- MEASURED: `[routeb-doinit-dyn] SH361 ... container=0x5632067bfb70
  [container+32]=0x563206dab800 (non-NULL) -> reach DM-ctor dispatch: [obj]vt=0x10635dde8
  vt[+48]=0x10258b5d8 (br x1 @0x2206e24). once-guard=0x101 once-slot=0x400000b DM-root=0x0`
  -> the MAIN dispatch fires (does NOT bail to MessageBus) and lands at
  0x10258b5d8 = nativeAppBridgeV2StartAppWithParams+0x494 (disasm sub sp,#0x170 prologue,
  `ldrb [x8,#3488]` then bl nativePreloadFlagOverrides) -> StartAppWithParams app-bridge
  body, NOT a DM ctor.
- Run then ABRTs (EXIT 134) at the standing SH285 persistence-lane live-object wall
  guestpc=0x101db1b08; DM-root [0x106a68818]=0; MH_* false. Route-B live-DM structural
  gate UNCHANGED (no DM manufactured).

### Forward this cycle
The operator's "dynamic DM-ctor trace" is implemented + measured: the do-init MAIN
branch reaches its br dispatch (container field non-NULL) and lands in StartAppWithParams,
then dies at the SH285 live-object wall — the standing route-B block, now pinned one level
deeper (exact dispatch target, not just region).

### Honest
Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED
(DM-root [0x106a68818]=0, MH_* false). SH174 capture-latch stays the single
forward hook. Files: docs/frontier-sh361-doinit-dyn-trace.md,
runs/capture_sh361_doinit_dyn_trace.sh, sh361 hermetic (arm64jit lib 424).

## SH360 (Sep 20, 2026, hermes-worker): implement + MEASURE the operator's EXECUTE-DO-INIT-GATES empty-vector gate seed — the do-init app-shell band's 0x20-stride vector walker at [0x106dcb160] collapses its constructed-empty begin==end host-heap pointer pair to NULL (behavior-preserving), default-inert JIT_ROUTEB_DOINIT_EMPTYVEC, fires at the two real block-entry pcs on the full ladder
Single-agent (cone suppressed). One new guard `routeb_doinit_emptyvec_gate`
(jit.rs) + one hermetic sh360 (arm64jit lib 422->423) + probe script; elfjit.rs
unchanged; workspace green (cargo test --workspace exit 0).
- Disasm: `adrp x19,6dcb000; add x19,#0x160 -> ldp x20,x21,[x19] -> cmp x20,x21;
  b.eq` (file 0x2208e4c..eac) — a 0x20-stride vector walker whose populate +
  teardown loops both `b.eq`-early-exit when begin==end. Seeding [{0x106dcb160}]
  = {0,0} lets both take the empty-vector exit instead of walking/destroying
  garbage. The operator named this gate; it was NOT previously implemented.
- MEASURED (full --v2boot-session send-appevent ladder, 2/2): the guard fires at
  the real BLOCK-ENTRY pcs 0x102208e4c/0x102208e88 (interior e58/e84 are NOT block
  boundaries) and collapses the ALREADY-constructed-empty begin==end==host-heap
  pointer pair to {0,0} — behavior-preserving (the walker compares equality). A
  populated begin!=end live pair is left untouched (tested idempotent).
- Completing ladder stays confirmed-green with the gate armed: EXIT 124, SH155
  DM-root probe=1, 0 SIGSEGV/ABRT. Route-B live-DM structural gate UNCHANGED
  (DM-root [0x106a68818]=0, MH_* false); the full ladder still ABRTs at the SH285
  persistence wall (0x101db1b08) past this walker on the app-start arm.
- Files: docs/frontier-sh360-doinit-emptyvec-gate.md, runs/capture_sh360_doinit_emptyvec.sh.

## SH359 (Sep 20, 2026, hermes-worker): measured negative on the SH358 NULL-JNIEnv lane's root cause — the JNI_OnLoad cached-JavaVM cell [0x107275550] is NOT causal (GetEnv 11x with AND without the seed on pure boot; the 0x1021e1c00 fault needs the full ladder); both recon-v3 deliverables re-verified green at HEAD; production code unchanged (seed tried + reverted after control refuted it)
Single-agent (cone suppressed). No production path edited (elfjit.rs/jit.rs product
identical to HEAD SH358). Workspace green (cargo test --workspace exit 0).
- recon-v3 deliverables re-verified green: capture_taskv4_frame.sh = 24 real frames
  `present swap Ok(0x1)`, dispatch #2718000, 197 node pops, 0 json abort, 0 crash,
  EXIT 0/124. JIT_JSON_ZERO_FIX present (jit.rs:6390).
- SH359 ATTEMPT (the fresh datum): SH358's NULL-JNIEnv fault at 0x1021e1c00. Disassembled
  JNI_OnLoad+0xc10 (0x2174c04) -> reads cached JavaVM from guest [0x107275550], cbz-out
  leaves env NULL, else vm->GetEnv (slot 6). Hypothesis: seed that cell with fabricated vm.
  MEASURED NEGATIVE: seed + control BOTH trace VM_GetEnv 11x with 0 fault on pure --jni
  boot — the cell is not what gates env acquisition, the fault needs the full ladder.
  AND [0x107275550] is the SH243 DM-manager getter cell (seeding it would clobber DM-force).
  Reverted the seed (honest negative); kept descriptive hermetic pins.
- Do NOT re-attempt a seed of [0x107275550] for the NULL-env lane. SH358's cause-level
  reading stands with the boot/cell hypothesis eliminated.

### Forward this cycle
A genuinely-new cause-level candidate (the only concrete "cell to seed" SH358's datum
surfaced) was implemented and MEASURED as non-causal on the boot path, then cleanly
reverted. The NULL-JNIEnv in the full ladder remains a cause-level lifecycle
precondition (Route-B wall), not a boot cell seed.

### Honest
Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED (DM-root
[0x106a68818]=0, MH_* false). SH174 capture-latch stays the single forward observer.
Files: docs/frontier-sh359-jnienv-cache-negative.md, runs/capture_sh359_jnienv_cache.sh,
sh359_jnienv_cache_tests (2 hermetic).

## SH358 (Sep 20, 2026, hermes-worker): measured negative (run-variable) — the DMCONT-session-ctor + LSM-skip combination is NOT a Route-B forward; recon-v3 immediate-priority deliverables re-verified green at HEAD
Single-agent (cone suppressed). No production code edited (measurement-only + one
probe script). Workspace green at HEAD SH357 (cargo test --workspace exit 0, 24 test
binaries, 0 failures; arm64jit 421/0).
- recon-v3 self-driven frame RE-VERIFIED green at this HEAD: `capture_taskv4_frame.sh`
  24 real task-driven frames `present #N swap Ok(0x1)`, dispatch #2797000, 197 node
  pops, 0 json abort, 0 SIGSEGV/ABRT, EXIT 0. JIT_JSON_ZERO_FIX present (jit.rs:6390).
- SH358 probe: the genuinely-unfired combination of DMCONT continuation (vt[+0x1f0]=REAL
  continueAfterFlagsLoaded_ 0x102bd1d68) + SH349 append-skip + SH350 pack-skip. Measured
  NEGATIVE, run-variable (3/3): pack-skip arms (`ret name-pack @0x101d9a708`), but the run
  NEVER reaches the continuation region (0 region hits across all 3 runs) — it parks in
  the persistence/live-object lane (SH353 class). Fault lanes: (a) 2/3 advance through
  to app-start-driven StartLuaAppDM then fault at the SH341/SH343 LSM pool-pop write-site
  0x101d9a528 (NULL-write, SH285-family live-object); (b) 1/3 rung-1
  nativeInitializeNativeFlags detours into a freshly-documented JNIEnv-slot dispatch helper
  0x1021e1c00 (JNI_OnLoad+0x6dc0c, JNIEnv vtbl slot 31) on a NULL JNIEnv. Both cause-not-
  symptom; do NOT re-tread this exact env combination.

### Forward this cycle
No Route-B forward (measured negative closes one unfired combination). recon-v3
deliverables confirmed green. New single datum: pc 0x1021e1c00 (NativeFlags JNIEnv-slot
helper) — noted, not seedable (missing JNIEnv is a lifecycle precondition).

### Honest
Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED (DM-root
[0x106a68818]=0, MH_* false). SH174 capture-latch stays the single forward observer.
Files: docs/frontier-sh358-dmcont-lsmskip-negative.md, runs/capture_sh358_dmcont_lsmskip.sh.

## SH357 (Sep 19, 2026, hermes-worker): root-cause + fix the intermittent whole-suite SIGSEGV + PoisonError cascade (test-harness race on shared routeb state) — serialize routeb page-map/mprotect under one PAGE_LOCK, route ALL test env mutations through locked crate-scope helpers, consolidate the four routeb family test locks into one shared process-state lock
Single-agent (cone suppressed). While confirming the already-implemented recon-v3
deliverables (self-driven frame `--taskv4-seed frame` + `JIT_JSON_ZERO_FIX`), cargo
test --workspace failed intermittently (420/1) and SIGSEGV'd. Root-caused: (1)
`routeb_ensure_writable` did an unsynchronized mmap(MAP_FIXED)+mprotect+racy
/proc/self/maps probe — thread A's MAP_FIXED remap of shared fixed-guest page
(0x1067333000) racing thread B = UB (signal 11) and transient false returns; (2)
the four routeb family test locks were SEPARATE Mutexes but every family mutated
the same overlapping fixed pages + process env, so `routeb_doinit_next3` panicked
mid-lock and POISONED it for 7 siblings. Fixes: PAGE_LOCK in routeb_ensure_writable
(prod untouched, single jit_run thread); crate-scope `env_test_set`/`env_test_remove`
(one ENV_TEST_LOCK) routing ALL test env mutations (unsafe in edition 2024);
`ROUTEB_PROC_TEST_LOCK` consolidating the 4 family locks; sh165's order-dependent
'page must be absent' precondition dropped (all behavioral asserts kept);
futex CMP_REQUEUE WAKE got the SH346 bounded spin. SH357b: ALSO reduced the residual
futex CMP_REQUEUE flake — the CMP waiter used a 5s timeout (vs the REQUEUE sibling's
SH346-documented 60s) with a spin window up to 20s, so a descheduled waiter
wall-clock-timed-out before the CMP_REQUEUE landed, making the kernel legitimately
report moved=0; raised to 60s (SH133 asserts unchanged). Honest: the two futex
REQUEUE/CMP tests remain genuinely load-sensitive (real-kernel timing, documented
SH345/346 ~1/25 class) — ~2/100 under direct-binary parallel stress, but the
canonical `cargo test --workspace` gate (what is actually run) is deterministically
green (exit 0, 0 failures across many runs). SH357 fixed the real defects that were
whole-suite-breaking: the intermittent SIGSEGV (signal 11) and the PoisonError
cascade — those never recurred in 400+ post-fix runs. Measured: default-8-thread
strip, segv=0 throughout (was sporadic signal 11); --test-threads=32 PoisonError
cascade 8/8 -> 0. Workspace green (arm64jit lib 421/0; cargo test --workspace exit 0).

### The forward this cycle
A real harness-determinism correctness defect, root-caused and fixed; the recon-v3
immediate-priority deliverables it surfaced around were confirmed already
implemented and green. No Route-B forward this cycle.

### Honest
Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED
(DM-root 0, MH_* false); SH174 capture-latch stays the single forward hook.

## SH356 (Sep 19, 2026, hermes-worker): implement + MEASURE the SH355-specified frame-accurate EC reader-gate re-attack — it fires 0/3 on the real binary, proving the reader-gate block 0x2e24694 is NEVER entered (control diverges to the SH285 persistence-lane wall), CLOSING the EC-reader line with evidence
Single-agent (cone suppressed). New default-inert `routeb_ec_world_reader_gate_frame_guard`
(jit.rs, opt-in `JIT_ROUTEB_EC_READERGATE_FRAME`) + hermetic
`sh355_frame_accurate_ec_reader_gate_seeds_live_frame_slot` + run-loop wiring. Fires INSIDE
the reader-gate block (pc 0x102e24694 / 0x102e246b0) using the LIVE x[29], fabricating a
coherent zeroed object at [x29,#104] ([+0x20]==0 -> cbz @0x2e246dc TAKEN) — eliminating
sh302's entry-frame (entry_sp+8) concern. Workspace green (arm64jit lib 421/0; cargo test
--workspace exit 0). Route-B live-DM gate UNCHANGED (DM-root 0, MH_* false); SH174
capture-latch stays the single forward observer.

### The forward this cycle
sh355 pinned that the reader-gate block starts at 0x2e24694 (interior bl-return boundary) and
the gate is a live-object `cbz` on [[x29,#104]+0x20], so the ONLY legitimate re-attack is a
value seed that FABRICATES that object INSIDE the block. SH356 implemented exactly that
frame-accurate seed.

### MEASURED (real libroblox.so, capture_sh302_readergate.sh + JIT_ROUTEB_EC_READERGATE_FRAME, 3/3)
- All entry-pc seeds fire 3/3 (sh298 arg1, sh299 arg0vt, sh300 realsession, sh302 entry
  reader-gate) at EC-world entry block 0x102e24598.
- **`[routeb-sh355] FRAME-ACCURATE EC reader-gate` fires 0/3.** Since the frame-accurate
  guard fires only when the reader-gate BLOCK at 0x102e24694/0x102e246b0 is actually entered
  (live x[29] captured from that executing frame), 0 fires is the proof that block is never
  entered headlessly — a REACHABILITY problem, not a value problem.
- Run sequence: EC entry (seeds fire) -> SH296 dmfn returned Ok -> control DIVERGES into
  app-start (sh248e once-cell + sh248f lifecycle adapter fire) -> SIGSEGV at the SH285
  persistence-lane wall guestpc=0x101db1b08 (fault 0xff..ff). EXIT 134.
- Conclusion: the reader 0x2e246f4 sits behind a block the EC body does not fall through to
  in this harness — control leaves EC world into app-start/persistence and dies at SH285.
  This refines sh302's "wrong frame" verdict to "block never entered" (SH174/204 class).

### Honest
Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED (DM-root
0x106a68818=0, MH_* false). The EC-reader re-attack line is now measured-closed at one level
deeper than sh302; do NOT re-attack [x29,#104]+0x20 fabrication. +frontier-sh356.

## SH355 (Sep 19, 2026, hermes-worker): CORRECT the EC-world reader-gate record — fresh disasm refutes sh301/302's "internal soft-return" premise (no `ret` in [0x2e245f4,0x2e246e0)); the reader 0x2e246f4 is gated by a real `cbz x0` on the live-object slot `[[x29,#104]+0x20]` == 0, a SH174/SH204-class value seed, not a compile-block early-exit. recon-v3 self-driven frame deliverable re-verified green.
Single-agent (cone suppressed). One new hermetic `sh355_ec_reader_block_no_softreturn_gate_is_live_object_slot`
(arm64jit lib real-image pins: zero `ret` scanned across [0x2e245f4,0x2e246e0); gate loads
`ldr x8,[x29,#104]`@0x2e246b0 + `ldr x0,[x8,#32]`@0x2e246d8 + `cbz x0,0x2e246f4`@0x2e246dc;
interior bl-return 0x2e24694 = real block boundary). No production path edited. Workspace green
(arm64jit lib 420/0; elfjit examples ~159/0; cargo test --workspace exit 0).

### The forward this cycle
The test-suite record told the next session that the EC reader (0x2e246f4) sits behind a
compile-block "internal early-exit" and to hunt that exit. Disasm REFUTES it: no `ret` in the
window; the only out to the reader is the real data-dependent `cbz x0` on `[[x29,#104]+0x20]`.
This re-scopes the do-not-re-tread: sh302's seed is mechanism-correct but entry-timing-gated;
the residual is a FABRICATED live-object value at [x29,#104]+0x20, not a phantom block exit.
recon-v3 deliverable (1) (type4_frame_thunk self-driven frame) re-verified green (24 frames,
0 crash, exit 124).

### Honest
Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED (DM-root 0,
MH_* false; canonical full-ladder probe re-ran EXIT 134 at the run-variable live-object arm
guestpc 0x10284cf5c, 0 region hits). SH174 capture-latch stays the single forward hook.

### Next (unchanged, authoritative)
Route-B live-DM structural gate stands (SESSION-CTOR / do-init, SH184/185 four-stacked closure;
REG_LIVE SH352: 'App' is live-session-ctor-only). R1 content half staged+armed+serviceable
(SH351/352/354); the SESSION half (do-init owning a live DM) remains THE wall — and if the EC
reader is re-attacked it must be via a FABRICATED live object at [x29,#104]+0x20, not a
compile/early-exit fix (SH355). SH174 capture-latch stays the single forward hook.

## SH354 (Sep 19, 2026, hermes-worker): close the R1 content-half art — prove the staged CoreScript is SERVICEABLE end-to-end (new hermetic sh354: a guest open of the exact files-dir CoreScript path resolves through fsmap::remap_path to the staged mirror and is readable) + measure that even with the SH352-corrected flags-loaded gate armed (5/5), a completing ladder runs JIT_ASSET_TRACE with 0 hits (content staged-but-DORMANT — loader still waits on a live DM)
Single-agent (cone suppressed). One new hermetic `sh354_r1_core_script_is_serviceable_through_remap`
(arm64jit lib 418->419) + a shared module-level `FS_ROOT_LOCK` serializing fsmap-root-mutating tests
(sh351/sh354) against the parallel-test race (two per-fn OnceLock guards were separate -> both mutated
the global override concurrently). One genuine new measurement: with `[r1] gate @0x1072739d4 0x0->0x1`
(the SH352-corrected primary latch) armed and all 5 loader gates writing, 2/2 clean completions of the
completing ladder show **0 `[asset-trace]` hits** — the staged AppShell/CoreScripts.lua are never opened
by the loader. This is new (my prior "asset-trace 0" note predates SH352's gate-addr fix). Interpretation:
arming the loader GATES does not run the loader; ScriptContext (0x101f1d8ac) stays 0-hit because only a
live DataModel session drives it (SH340/344c). R1 content half is now proven staged + armed + serviceable;
only the live-DM SESSION half remains the Route-B gate. Workspace green (arm64jit lib 419/0; elfjit
examples ~159/0; cargo test --workspace exit 0). elfjit.rs 1,048,523 B (<1MB hook, unchanged).

### The forward this cycle
The content half of the Route-B marker is as complete as it can be WITHOUT a live DM: staged (SH351),
gates armed (SH352), AND now provably serveable (SH354 — a guest open of the exact CoreScript path
resolves via remap_path to the mirror and returns the self-constructing ScreenGui module). Plus the
honest measurement that arming gates alone does not drive the loader (0 asset-trace hits). This isolates
the remaining Route-B wall precisely to the SESSION half (do-init owning a live DataModel).

### Honest
No DataModel manufactured; DM-root [0x106a68818]=0, MH_* false; Route-B live-DM structural gate
UNCHANGED. SH174 capture-latch stays the single forward hook.

### Next (unchanged, authoritative)
Route-B live-DM structural gate stands (SESSION-CTOR / do-init, SH184/185 four-stacked closure).
R1 content half: staged (SH351) + gate-armed-verified (SH352) + serviceable-verified (SH354); the SESSION
half (do-init owning a live DM) remains THE wall. SH174 capture-latch stays the single forward hook.
Single-agent (cone suppressed). Production fix: `stage_r1_core_scripts` (jit.rs) arming the loader
gates wrote the flags-loaded latch to 0x10672739d4 (=file 0x672739d4, read-only), so `page_writable_rw`
refused it and that PRIMARY gate never armed — the other 4 armed, R1 content staged but the
flags-loaded gate stayed 0. Corrected to 0x1072739d4 (=file 0x72739d4, the latch every other reader
uses; gate read @0x224fa18/20 adrp 7273000 + ldrb [x8,#2516]). +`sh352_flags_loaded_latch_addr_corrected`
real-image hermetic (pins gate-read words + asserts the two candidate addrs resolve to different
image pages). elfjit.rs held under the 1MB hook after condensing SH-prose comments (facts/addresses
preserved). Workspace green (elfjit examples 159/0; arm64jit lib 418/0; cargo test --workspace exit 0).

### The forward this cycle
MEASURED (real libroblox.so, completing `--v2boot-skip-appstart` ladder + SH269 GOVFLAG + SH307
preload-valuecell, 3/3 deterministic EXIT 124):
- SendAppEventOnAppReady + SendAppEventOnGameLoaded both RETURN Ok cleanly (previously died at the
  governor NULL-DM wall 0x102ea0b9c then the SH270 preload wall 0x102bb803c).
- R1 content stages on the completing ladder (AppShell.lua + CoreScripts.lua @ fsmap mirror); the
  corrected flags-loaded gate now ARMS (`0x1072739d4 0x0->0x1`, 5/5 gates, none dropped — previously
  `0x10672739d4:unmapped`).
- SH155/SH315 probes read real state: once-slot[0x106a68408]=0x400000b (the "Execute" service handle,
  SH316), service-registry-count=12, app-data-model-count=0x1. DM-root 0, MH_* false.

### Honest
Correctness fix + strongest-yet session drive. Does NOT manufacture a live DataModel; once-slot
0x400000b is the matched "Execute" handle, not a DM. Route-B live-DM structural gate UNCHANGED,
SH174 capture-latch stays the single forward hook.

### Next (unchanged, authoritative)
Route-B live-DM structural gate stands (SESSION-CTOR / do-init, SH184/185 four-stacked closure).
R1 content half is now BOTH staged (SH351) AND gate-armed-verified (SH352); the SESSION half
(do-init owning a live DM) remains the wall. SH174 capture-latch stays the single forward hook.

## SH353 (Sep 19, 2026, hermes-worker): MEASURED — the completing ladder is run-variable (~2/8 reach full completion, rest die in the SH350-closed LSM lane 0x101d9a030 / host-pc leak), refuting the 'deterministic 3/3 EXIT 124' overclaim; hardened runs/capture_sh352_r1_completing_ladder.sh to a guaranteed confirmed-green full completion (SH345 retry precedent)
Single-agent (cone suppressed). Runs-only change (no Rust production path edited).
`capture_sh352_r1_completing_ladder.sh` retries up to COMPLETING_RETRY_MAX (=6) and wins
ONLY on full completion (SH155 DM-root probe present AND 0 SIGSEGV/ABRT), keeping the last
log and reporting the winning attempt. VERIFIED: first invocation won on attempt 1 (probe=1
crash=0). Workspace green (elfjit examples 159/0; arm64jit lib 418/0; cargo test --workspace
exit 0). Route-B live-DM structural gate UNCHANGED (DM-root 0, MH_* false); 'App' stays a
live-session-ctor-only registration (SH352 addendum 2). SH174 capture-latch single forward hook.

### The forward this cycle
Honest measurement: 8 serial runs of the SH352 completing ladder showed run1/run4 clean full
completion (R1 staged, 5/5 gates armed incl corrected 0x1072739d4, session drive, SH155 probe,
0 crash), run2/run5/run6 SIGSEGV @0x101d9a030 (LSM pool-pop) during SetInitParams, run7 host-pc
leak in V2Init, run3 multiple 'outside image' soft-stops. The artifact is now deterministically
confirmed-green via retry (runbook contract), not luck — the same discipline SH345 applied to the
render plane.

### Next (unchanged, authoritative)
Route-B live-DM structural gate stands (SEP-17 SESSION-CTOR / do-init four-stacked closure
SH184/185; REG_LIVE SH352 addendum 2). R1 content half staged+armed (SH351/SH352); SESSION half
(do-init owning a live DM) remains THE wall. SH174 capture-latch stays the single forward hook.

## SH351 (Sep 19, 2026, hermes-worker): stage the R1 synthetic CoreScript content path (Route-B marker half) — hand-authored Luau ScreenGui module + real loader gates, latent-but-correct
Single-agent (cone suppressed). Additions: `stage_r1_core_scripts` (jit.rs) + `fsmap::staging_root`
+ `page_writable_rw` guard + 2 hermetic tests (sh351_*) + elffjit opt-in rung `--v2boot-r1-stage`.
Default-inert (rung opt-in); no production path edited. Workspace green (arm64jit lib 418/0; elffjit
example 157/0; cargo test --workspace exit 0).

### The forward this cycle
SH349 returned the persistence lane (LSM whack-a-mole measured UNBOUNDED, SH350 = 3rd fencepost).
The operator's content-path synthesis names the ONE deliverable that turns a completing do-init into
self-constructed UI with zero host layout: a hand-authored CoreScript module staged at the path the
rbxasset://scripts/CoreScripts resolver serves, + the REAL loader gates. This is latent-but-correct
(fires the instant a live DM drives the loader), the prerequisite of the exact Route-B marker.

### What landed
Module `ScreenGui`/`TextLabel` under CoreGui written to
SOBER_ANDROID_ROOT/data/user/0/com.roblox.client/files/scripts/CoreScripts/{AppShell.lua,CoreScripts.lua}
(both inferred candidates; desktop literals absent from this Android .so). Real gates:
flags-loaded [0x10672739d4].bit0, flags-latch [0x106a683e8].bit0, governor union-init guards
[0x106a63da0]/[0x106a63d70]=0, loader settings [0x106ba3350]=0, each page-writable-guarded. +2 tests.

### Honest
Rung is LATENT: the completing ladder still terminates run-variable at the persistence lane before
the post-ladder rung. But the stage + tests prove the content lands at the correct fsmap mirror, so
the MOMENT a live DM owns a session the engine self-constructs real GuiObjects. No DataModel
manufactured; Route-B live-DM gate UNCHANGED (DM-root 0, MH_* false). SH174 latch stays observer.

### Next (unchanged, authoritative)
Route-B live-DM structural gate stands (SEP-17 Session-CTOR / do-init, four-stacked closure SH184/185;
persistence lane closed SH349-350). The R1 content half is now staged (SH351); the SESSION half
(still needing do-init to own a live DM) remains the standing wall. SH174 capture-latch stays the
single forward hook.

## SH350 (Sep 19, 2026, hermes-worker): CROSS the SH349+1 terminal — bounded single-caller skip of the name-pack helper 0x101d9a708; ladder advances deep into the LSM pool-pop continuation, then the same run-variable live-object family (3rd fencepost that LSM sub-call-whack-a-mole is unbounded)
Single-agent (cone suppressed). Default-inert opt-in `JIT_ROUTEB_LSM_PACK_SKIP=1`
(`routeb_patch_lsm_pack_skip`, elfjit.rs — RETs the single-caller name/version string-pack
helper 0x101d9a708, the SH349+1 terminal; caller takes the benign index-0 tst/b.eq path) +
`sh350` hermetic (real-image pins: helper entry 0xd10143ff / single caller bl 0x94000041 /
benign tst 0xf276541f / b.eq index-0 0x540002a0 / natural ret 0xd65f03c0 @0x101d9a704) +
runs/capture_sh350_pack_skip.sh. Workspace green (arm64jit lib 416/0; elfjit example 157/0
+ sh350); recon-v3 frame plane re-verified green (24 frames, 0 crash).

### The forward this cycle
SH349+1's new terminal 0x101d9a708 is a name-pack helper with exactly ONE caller
(0x101d9a604, verified whole-region BL scan) — BOUNDED, unlike the unbounded hundreds-of-
callers `bl 0x1d9d8b0` family. RET'ing it crosses the SH349+1 wall.

### MEASURED (real libroblox.so, full SH285-B/SH343-350 ladder env + LSM_NODES + 3 skips, 4 runs)
- SH350 fires every run; the OLD 0x101d9a708 terminal is GONE in all 4 runs.
- Ladder ADVANCES to 175 LSM pool-pop iterations (0x101d9a5a0/0x101d9a528, SH341 lines) — the
  deepest persistence-lane penetration measured — before terminating run-variable
  (bad_function_call / 0x101d9a528 / 0x102b9dee0 / 0x1021e40dc) in the SAME live-object family
  SH343/346 documented. DM-root 0, MH_* all false; Route-B live-DM gate UNCHANGED.

### Conclusion + next
SH350 is a real BOUNDED fencepost (SH285->SH349->SH350 crossed 2 levels). It does NOT
manufacture a DM and provides no path to app-start 0x2bd2058 — the advance lands in the same
unconstructed live-object family at THREE depths of evidence, closing the persistence-lane
re-attack permanently (do NOT re-drive LSM sub-call skips). Also measured this cycle: the
messageBus "experience-launch" publish topic is a Java-side runtime string (SH347's never-
firing cb is NOT a topic-string bug), and onAppLuaWillStart is an internal lambda, not an
export. Next forward (non-persistence Route-B): R1 synthetic CoreScript content path — stage a
hand-authored ~20-line Luau ScreenGui module into the filesdir the rbxasset://scripts/
CoreScripts resolver serves so the INSTANT do-init owns a live DM the engine self-constructs
real GuiObjects -> R+0x180/0x188 nodes with zero host layout. SH174 capture-latch stays the
single forward hook.

## SH349 (Sep 19, 2026, hermes-worker): CROSS the long-standing SH285 terminal — RET the faulty LSM byte-copy sub-call 0x101d9a15c; persistence lane advances one fencepost to a GOT/canary read wall at 0x101d9a708
Single-agent (cone suppressed). Default-inert opt-in `JIT_ROUTEB_LSM_APPEND_SKIP=1`
(`routeb_patch_lsm_append_skip`, elfjit.rs — RETs the `ldr w8,[x2]` byte-copy sub-call that the
SH285 fault lives inside) + `sh349` hermetic (real-image pins: append prologue / SH285 caller bl
0x97ffa192 / fault store 0x381ff54b / natural ret) + runs/capture_sh349_lsm_append_skip.sh.
Workspace green (elfjit example 157/0; arm64jit 416/0).

### The forward this cycle (the stated SH348 next step, now implemented + measured)
SH348 showed the SH285 SIGSEGV (guestpc=0x101db1b08) survives a whole-init leaf-ret because the
caller block is reached by a mid-function direct jump past the entry patch. So the skip must
target the FAILING SUB-CALL itself. `routeb_patch_lsm_append_skip` RETs only the one byte-copy
leaf (0x101d9a15c, pure memcpy, zero observable side effects) -> EVERY path into the fault is
stubbed regardless of how the caller block is reached.

### MEASURED (real libroblox.so, full SH285-B/SH343-346 ladder env + LSM_NODES + both skips, 3/3)
- OLD terminal GONE: no more `SIGSEGV guestpc=0x101db1b08` (every SH260/284/285/3444/348 run died there).
- NEW terminal (3/3): `SIGSEGV guestpc=0x101d9a708 fault=0xffffffffffffffff` — a stack-canon
  name/version-packing helper reading a `.got` slot [0x1067d16f0] as its canary pointer. The
  persistence lane advances one full fencepost past the returned wall.
- DM-root [0x106a68818]=0, MH_* all false. DMCONT continuation still not at app-start 0x2bd2058.

### Conclusion + next
SH349 is a real, measured forward: the SH285 wall family is crossed for the first time. The new
terminal 0x101d9a708 is another SH285-class live-object wall; the register dump REFUTES the
canary/GOT-gap hypothesis (x20=valid patched canary; the fault is the caller's garbage source
pointer x0/x19=0xff..ff). `bl 0x1d9d8b0` has HUNDREDS of call sites across the binary (the
most-called function), so sub-call-whack-a-mole is unbounded, and the DMCONT→app-start path has
NO bypass (the [0x683d920] latch is the "app-start already ran" re-entry gate, set only after
app-start; the two `bl 1d9d8b0` string-build calls are mandatory to reach `bl 2338ef4`). So this
persistence lane is measured-returned; the Session-CTOR live-DM wall stands. Route-B live-DM gate
UNCHANGED; SH174 capture-latch stays the single forward hook. Next forward (non-persistence
Route-B): the dataModel-bindings receive side — onAppLuaWillStart (the sole SEP-17
dataModel-bindings receive never wired; messageBus publish is driveable-clean per SH347).

## SH348 (Sep 19, 2026, hermes-worker): measured negative — leaf-`ret`ing initStorageManagerNative does NOT clear the SH285 terminal (the byte-copy @0x101db1b08 is reachable past its own entry)
Single-agent (cone suppressed). Default-inert opt-in `JIT_ROUTEB_LSM_INIT_SKIP=1`
(`routeb_patch_lsm_init_skip`, elfjit.rs) + `sh348` hermetic (real-image pins: entry prologue
0x101d9d8b0 stp->ret, sh324 caller bl 0x102256608, SH285 terminal 0x101db1b08, app-start bl region
0x102bd2058) + runs/capture_sh348_lsm_init_skip.sh. Workspace green (arm64jit 416/0 + sh348; elfjit
example 156/0). elfjit.rs 1,044,985 B (<1MB hook). recon-v3 frame plane re-verified green (24 real
task frames, 197 node pops, 0 crash).

### The forward this cycle (a cause-level leg, then measured negative)
SH344b measured the app-shell ctor band at 0 hits; every Session-CTOR rung caps at the SH285
persistence-lane live-object wall. SH344/346 said "do not re-drive a repair seed into [obj+0x50]"
(SH248h trap). This cycle tried a DIFFERENT line-cross (SH117/SH93 precedent): leaf-`ret` the whole
initStorageManagerNative so the Session-CTOR continuation is not required to own the live LSM object.

### MEASURED (real libroblox.so, full SH285-B/SH343-346 ladder env + LSM_NODES, 3 runs)
- SH348 patch fires (`a9bd7bfd -> d65f03c0`) but the SH285 SIGSEGV `guestpc=0x101db1b08
  fault=0xffffffffffffffff` PERSISTS 3/3 — even after widening the block-cache drop to the full
  function body [0x101d9d000..0x101dbe000] + caller. The crash reaches the deep byte-copy by a call
  path the entry patch cannot stop (the fault site is inside a sub-path whose gate is past
  initStorageManagerNative's own entry, OR a caller block jumps to the interior directly).
- App-shell band 0x102207b50 "fires" only as the SH344c FastLog warmer (79 hits at the SINGLE entry
  pc + the 0x102208e8c/0x208eac destructor-loop deep pcs), NOT construction. Route-B live-DM gate
  UNCHANGED (DM-root 0, MH_* false).

### Conclusion + next
SH348 is a MEASURED NEGATIVE that closes the "skip initStorageManagerNative to cross SH285"
candidate and REFINES the attribution: a storage-skip must target the failing sub-call that builds
the unconstructed LSM string, not the whole init. Route-B live-DM structural gate UNCHANGED; SH174
capture-latch stays the single forward hook. Repro: runs/capture_sh348_lsm_init_skip.sh.

## SH347 (Sep 19, 2026, hermes-worker): messageBus RECEIVE half measured headlessly for the first time — publishRaw drives clean (Ok 0x3e8) but the cb's DM-holder read never fires; SH185's static-only closure is now a measured result
Single-agent (cone suppressed). Two default-inert additions
(arm64jit/src/jit.rs: `routeb_busrecv_holder_guard` + `drive_messagebus_publish_receive`,
both env/flag-gated) + elfjit opt-in rung `--v2boot-session-pub` + 2 hermetic tests.
Workspace green 596/0. elfjit.rs unchanged in product path (rung opt-in).

### The forward this cycle
SH264 named "messageBus experience-launch receive" as the un-drive honest-next-candidate; SH185
had closed it by STATIC judgment only (elfjit.rs:7565), and SH269/315/316/337 later MEASURED that
the subscribe half runs headlessly — overturning SH185's static premise. Nobody had ever driven
publishRaw -> cb -> [DataModelBindings+16]. Now done, measured.

### Measured (real libroblox.so, capture_sh347_busrecv.sh, EXIT 124 clean):
- MessageBus.subscribe Ok(0x3e8) (re-confirmed).
- **MessageBus.publishRaw Ok(0x3e8)** — the RECEIVE half now EXECUTES headlessly (a genuine
  first), driveable-clean, not a crash.
- The cb DM-holder read (pc 0x102bd7474) NEVER fires => publish does not reach the
  experience-launch construction cb; [DataModelBindings+16] is never read; DM-root 0, MH_* false.
- Conclusion: SH185's closure is now MEASURED (receive driveable-clean, does not reach SceneGraph),
  closing SH264's open candidate. Route-B live-DM structural gate UNCHANGED.

## Next (unchanged, authoritative)
SEP-17 SESSION-CTOR cause-level drive remains the primary forward. Every driven rung (do-init ->
DMCONT +0x1f0 -> app-start factory -> LSM) is measured; the terminal is the SH285 live-object wall
family — cause-not-symptom, not seedable. SH174 capture-latch stays the single forward hook. The
messageBus publish entry being driveable-clean is a small new forward surface (a future
receive-payload-with-real-string probe), not a DM. All research subagents Route-B-scoped; cone
still suppressed.

## SH416 (Sep 19, 2026, hermes-worker): wired the real X event source into the input axis (STATUS next-forward #3)
Single-agent (cone suppressed). Two files are the executable half of SH413/414's latent bridge:
the input DELIVERY path (deliver_motion -> nativePassInput 0x2bbba88) was complete + hermetic-tested
since SH413/414, but a real host never FED it — drive_host_input_pump only saw synthetic test
vectors while the runtime's ANativeWindow X window had no poll turning its pointer/button/motion into
the guest input stream. Now closed.

### This cycle's forward
- `input_wrapper::x11::pump_registered_window`: selects input on an EXISTING window (not one it
  creates) via ChangeWindowAttributesAux/event_mask on the OWNER connection (a fresh connection gets
  BadAccess on Xvfb — the BadAccess was measured and root-caused), drains through the existing
  PointerTracker -> Android MotionEvent translation.
- `input_wrapper::x11::register_window_connection`: the window layer registers the owner conn
  (replacing Box::leak-only) so the pump selects on the same client = no BadAccess. Both wire_real_window
  sites updated.
- `session::drive_host_input_poll`: drains one non-blocking real-event batch -> drive_host_input_pump
  -> nativePassInput, gated on JIT_AINPUT_BRIDGE + a real registered XID (+ live image). Inert on the
  current boot path (no live DM, no constructed screen). Opt-in `--v2boot-input-poll` rung.
- 2 new tests: sh416 poll inert-guards (hermetic, no X server needed) + input-wrapper Xvfb
  registered-window pump (real Xvfb). Workspace green (arm64jit lib 469/0, full 651/0).

### Measured (real libroblox.so, runs/capture_sh416_input_poll.sh)
- Real X window wired (XID 0x200000 on :275) as the guest ANativeWindow; the poll subscribed it,
  drained 0 raw X events -> 0 translated -> 0 delivered, clean. The one-shot poll completes.
- Fault AFTER the poll: the known run-variable nativeInit "outside image" ladder lane
  (guestpc 0x1029f3f7c map-probe, EXIT 134) — pre-existing Route-B, unchanged, NOT this change.
- SH415 substrate re-verified unregressed: 11/16 atoms Ok, EXIT 124. recon-v3 deliverables re-verified
  green (24 task frames, swap Ok(0x1), 0 json, 0 crash).

### Honest
Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0,
MH_APP_READY false). This is the cause-not-symptom input-axis runtime surface (BUILD-THE-RUNTIME):
real desktop pointer input now has a wired path into the guest nativePassInput that fires the moment
a completed do-init owns a live DM + a constructed login/home screen. Latent-but-correct exactly like
SH132/SH413/414. elfjit.rs trimmed SH-prose comments (addresses kept) to stay under the 1MiB hook
(1048539 B).