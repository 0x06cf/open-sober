# Frontier SH367 — MEASURED NEGATIVE: arming the real window-attach GL-surface path faults; the SH366 clean INIT_WINDOW drive is preserved

## Session
Sep 19, 2026, hermes-worker. Single-agent (cone suppressed). One new read-only observation
guard `routeb_glue_realattach_guard` (jit.rs, opt-in `JIT_ROUTEB_GLUE_REALATTACH=1`, once,
ZERO guest mutation) + one real-image hermetic `sh367_window_attach_real_path_pinned_and_guard`
(arm64jit lib 430) + capture `runs/capture_sh367_glue_cmd_real.sh` (confirmed-green SH366-entry
predicate). elfjit.rs unchanged. Workspace green (cargo test --workspace EXIT 0).

## The attempt (SH366 next-forward, executed + measured)
STATUS/frontier-sh366 named the next forward: hand the engine a REAL wired ANativeWindow in
[inner+64] so window-attach 0x2bd29a0 takes its real GL-surface path (the SESSION-CTOR
window precondition the operator names for initEngine_). SH367 implemented exactly that:

- Armed the window-attach once-guard: [win+0x268].bit0=1 (window obj in [inner+64]).
- Crafted [win+0x278] = a leaked NULL-first-word obj (so the deep GL call would fast-return).
- Registered the wired X11 XID 0x200000 via set_anativewindow_xid.

### Disasm (real libroblox.so) — the real path
Window-attach 0x2bd29a0: `add x8,x19,#0x268; ldarb w8,[x8]` @0x2bd2a0c/a10, `tbz w8,#0` @0x2bd2a14
(zeroed once-guard -> benign ret = SH366). ARMED -> `add x0,x19,#0x278` @0x2bd2a18 -> `bl 0x2291c24`
(0x2291c24 `ldr x8,[x0]; cset w0,ne` = returns ([x0]!=0) — 1 iff [win+0x278] non-null) -> `tbz w0,#0`
@0x2bd2a20 -> `add x0,x19,#0x278` @0x2bd2a24 -> `bl 0x22985c0` (the REAL GL post-init).

### MEASURED (SH367 capture, 8 attempts, exit=134/139 every try)
```
[elfjit:glue-cmd] armed window-attach real path (once-guard [win+0x268].bit0=1, [win+0x278]=0x7f818c000b70, wired XID=0x200000)
[elfjit:glue-cmd] driving process_cmd @ guest 0x102bcd6e4 ... cmd=11 INIT_WINDOW ...
[SIGSEGV] tid=.. fault=0x7f818c0097 rip=.. guestpc=0x7f0000001f50 rbx_matches_gueststate=true
[SIGABRT] ... guestpc=0x0
```
- **0/8 reach the INIT_WINDOW body completion** (marker [inner+9]=1 never fires); drive no longer
  returns Ok. The arming turns a confirmed-green clean entry into a hard crash.
- Fault is INSIDE the host GL dispatch (guestpc 0x7f0000001f50 = host-thunk region; fault 0x7f818c0097)
  — the deep GL post-init 0x22985c0 runs and needs a REAL EGL surface/context object. Because the
  crafted non-null [win+0x278] makes 0x2291c24 return 1, control REACHES 0x22985c0, whose deep body
  (cbz @0x22985c4 NOT taken on the non-null pointer... its operand is the OBJECT its first word must
  be non-trivial for) cannot be satisfied by a fabricated NULL-first-word obj.

### Reverted
The once-guard arming is a MEASURED NEGATIVE: it faults hard and regresses the confirmed-green SH366
entry. Reverted to the SH366 clean drive (guard left OFF). Verified clean again: `process_cmd returned
Ok(...)`, `INIT_WINDOW case body marker [inner+9]=1 EXECUTED`. The read-only guard + hermetic pins are
kept as the measured-negative instrumentation (default-inert, zero mutation, pass green).

## Interpretation (honest)
- SH367 is a genuine one-level-deeper pin: the real window-attach GL path is a HARD SESSION-CTOR wall,
  not a value seed. The engine's deep GL post-init 0x22985c0 requires a real EGL surface/context — the
  exact object only a live Android Activity/AppBridge session drive (window + GL surface + onAppReady)
  provides. A fabricated window obj cannot satisfy it.
- This CONFIRMS the operator's SESSION-CTOR directive is the right (and only) path: the window
  precondition is real work (a real surface), not a seedable global. The ALooper glue loop still does
  not deliver APP_CMD_INIT_WINDOW (SH365 dead-drain unchanged); SH366's bounded process_cmd entry is
  the correct drive but its window-attach COMPLETION needs a genuine surface.
- Do NOT re-seed [win+0x268]/[win+0x278] to force the real window-attach path — measured fault
  (SH367). Do NOT re-attach a zeroed obj expecting the DM to move (SH366). Route-B live-DM structural
  gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false).

## Do-not-re-tread
- Do NOT arm the window-attach once-guard (SH367 measured fault at the deep GL post-init).
- Do NOT re-enter the infinite ALooper glue LOOP (bounded process_cmd is the entry; SH39b/364/365).
- Do NOT treat the INIT_WINDOW marker as a live DM (SH366/367; DM-root stays 0).

## Verify
- `cargo test --workspace` green (EXIT 0; arm64jit lib 430 including sh367).
- Real-binary clean entry restored: runs/capture_sh367_glue_cmd_real.sh -> SH366-entry-confirm=YES
  (drive Ok + [inner+9] marker). The SH367 armed-fault repro is documented above (8/8 EXIT 134/139).
- Files: arm64jit/src/jit.rs (+routeb_glue_realattach_guard read-only, +sh367 hermetic);
  runs/capture_sh367_glue_cmd_real.sh. Commit: local dev only (operator pushes).