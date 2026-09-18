# Frontier SH325 — SH160's init3-gate NOP was dead-code on the fault path; hoisted it
# into upfront --v2boot setup and CLEARED the SH324 terminal (0x102256510 -> 0x1025f5300), 3/3

Status: `dev`, single-agent (cone suppressed). Hermetic: `sh325` (elfjit real-image guard, 1
passed; 6 pins) alongside `sh324` (unchanged). elfjit examples 150/0 (149 + sh325); arm64jit lib
408/0; workspace green (588/0). elfjit.rs held ~90 B under the 1MB pre-commit hook after
condensing SH-prose. Route-B live-DM gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false).

## 1. Problem / why this cycle

SH324 proved the fabricated-object seed is a dead-end at the SESSION-CTOR wall 0x102256510
(fn 0x2256510 `this->vt[+32]()` with this=[container+40]=NULL consumes the x8 out-param
post-dispatch, which an AArch64 host leaf cannot write). The operator's SEP-17 lever is a REAL
session ctor. But the codebase ALSO carries SH160 (`routeb_patch_startapp_init3_gates`), a
`.text`-NOP that replaces both `bl 0x2256510` call sites (0x1023f013c/0x1023f01b0) with
`stp xzr,xzr,[x8]` — zeroing the out-buffer so the downstream csel/cbz skips the copies. That is
a legitimate line-cross WITHOUT fabricating the object (distinct from the SH324 dead-end). It
never helped because it was gated behind the StartLuaAppDM rung.

## 2. Root cause (measured, real libroblox.so)

SH160's call sat inside `if *guest == 0x1023efe2c && JIT_SH115_SINGLETON_PATCH==1` — the ladder
rung that runs `nativeAppBridgeStartLuaAppDM`. Two failures compound:

1. `--v2boot-skip-appstart` SKIPS that rung entirely (SH269: skip because the app-start rungs
   terminate the process) — so SH160 never fires on that route.
2. WITHOUT skip-appstart, the MAIN thread's `--startapp 0x258b144` (V2StartAppWithParams) reaches
   fn 0x23f00f8 -> `bl 0x2256510` BEFORE the ladder thread gets through warmup + the earlier
   rungs, so the process faults at 0x102256510 with SH160 still unpatched.

Net: SH160 was dead code on the very path it was built to clear, so the SH324 terminal persisted
3/3 for many cycles even though the "fix" existed.

## 3. The fix (hoist to upfront --v2boot setup, gated the same)

Moved the `routeb_patch_startapp_init3_gates()` call OUT of the StartLuaAppDM-rung block into
the upfront `--v2boot` setup (beside the other routeb_patch_* calls), gated on the same
JIT_ROUTEB_DM_SEED / JIT_ROUTEB_DMFORCE flags. It is self-guarding + idempotent (checks the
original `bl` bytes before writing `stp xzr,xzr`), so firing on the MAIN thread's StartApp drive
is safe and idempotent with the later rung-level call.

## 4. Measured (real libroblox.so, runs/probe_sh324_terminal_regs.sh, 3/3)

- BEFORE (SH160 dead): first SIGSEGV guestpc=0x102256510 every run (3/3).
- AFTER (hoisted): `[elfjit:routeB] SH160 patched init3 dispatch-gate call @0x1023f013c ... @0x1023f01b0`
  both fire; first SIGSEGV advances to GUESTPC=0x1025f5300 (3/3, EXIT 134/139/134 — stable wall).
- New terminal 0x1025f5300: inside nativeAppBridgeV2StartAppWithParams (fn 0x25f5270), after
  `bl 0x25f54e8` (a field-copy helper) the caller faults at fault=0x140 reading
  [x0+0x140] where x0 = [appstart_obj+24] (= a genuinely session-constructed AppStarted field,
  still 0 headlessly). This is the NEXT live-object construct, one gate further down the same
  line — again a REAL-session object, not a seedable cell.
- The skip-appstart session-drive ladder still completes cleanly (EXIT 0 when not perturbed by
  the flaky ASLR printf; the observed 134/139/0 spread is the known run-variable base, SH320).

## 5. Honest

Does NOT manufacture a DM (DM-root 0, MH_* false). Route-B live-DM structural gate UNCHANGED.
What is new + measured: the SH324 wall — a PROVEN fabricated-seed dead-end — is now CLEARED by
SH160's existing NOP, which was simply reachable-code-misplaced. The line advances one more
fencepost (0x102256510 -> 0x1025f5300) into V2StartAppWithParams' real field-copy. Next work: the
new live-object construct at [appstart_obj+24] (fn 0x25f54e8's source) is again a genuine
session-built AppStarted object (SEP-17 territory); a host-seed there is the same dead-end class
SH324 proved, so the forward remains the real session-ctor drive.

## 6. Verify

- `cargo test -p arm64jit --example elfjit -- sh325` = 1 passed (6 real-image pins: both bl
  0x2256510 SH160-NOP sites; new-terminal 0x1025f5300 `adrp x8,0x6a70000` + `ldr x0,[x8,#456]`;
  fn 0x25f54e8 `sub sp,#0x50`; caller `ldr x0,[x19,#24]`).
- `cargo test --workspace` EXIT 0; elfjit examples 150/0; arm64jit lib 408/0; `cargo build` EXIT 0.
- Repro: `runs/probe_sh324_terminal_regs.sh` (first_sigsegv now 0x1025f5300, SH160 lines both fire).
- elfjit.rs held under the 1MB hook (1023.90 KB).

Single-agent, default-inert (gated on DM_SEED/DMFORCE, same as before — only the timing/location
of an existing patch changed).