# Frontier SH369 — MEASURED pin: window-attach COMPLETION funnels into the CLOSED persistence lane (not to a live DM); the SESSION-CTOR window precondition is a *second* entry into the SH285/SH349 wall, not a new path

## Session
Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). One new real-image hermetic
`sh369_window_attach_completion_converges_to_persistence_lane` (arm64jit lib 432). No
production path edited (read-only pin). Workspace gate confirmed green before this commit
(cargo test --workspace WS_EXIT=0, 0 failed; arm64jit 3x parallel 431/0) and re-run after.

## Why
SH366 entered the engine's REAL app-command dispatcher process_cmd headlessly and drove the
INIT_WINDOW case (marker [inner+9]=1). SH368 drove a verified-safe command SEQUENCE {6,8,11}.
SH367 attacked the next-forward: arm window-attach's once-guard + a crafted [win+0x278] so the
engine takes its REAL GL-surface path (bl 0x22985c0 deep GL post-init) — and MEASURED a hard
fault (guestpc=0x7f0000001f50, host GL dispatch), interpreting the wall as "needs a REAL EGL
surface/context, only a live Activity/AppBridge session provides." SH369 corrects that
interpretation with a disasm-verified completion chain.

## The new datum (real libroblox.so)
Window-attach (0x2bd29a0) armed -> bl 0x2291c24 (cset w0,ne on [win+0x278]) -> bl 0x22985c0:
- 0x22985c0: `ldr x8,[x0]; cbz` (fast-return 1 on NULL first word), else -> bl 0x2270a98.
- 0x2270a98: once-guard on 0x6ed70cc; unset -> lock (bl 0x284ce54) + alloc 0x30 (bl 0x1d96768)
  + bl 0x2270b24 (the real body).
- **0x2270b24 body gates on the SAME flags-loaded latch the --v2boot ladder seeds**:
  `adrp x9,7273000; ldrb w9,[x9,#2516]` @0x2270b5c/0x2270b64 reads **0x72739d4**, and when
  bit0=1 falls THROUGH the `cbz w9,0x2270be8` @0x2270b78 to **`bl 0x1db1050`** =
  initStorageManagerNative — the SH285-family persistence lane (guest 0x101db1050; the
  SH285 fault site 0x101db1b08 is inside this function; SH349 RET-crossed it, SH350/358 closed
  the lane as measured-unbounded).

## Interpretation (honest)
So window-attach COMPLETION (the SESSION-CTOR window precondition) is NOT a route to a live DM:
even with flags-loaded seeded (bit0=1, which the ladder already does) the deep body hands control
to initStorageManagerNative — a SECOND entry into the same persistence lane SH349/350/358 already
measured returned. The operator's SEP-17 SESSION-CTOR lever is still correct as *the* live-DM
route (real Activity/AppBridge session constructs the upstream ctor), but this pin de-risks it:
handing the engine a real EGL surface at window-attach does NOT by itself cross Route B — it
re-enters the closed persistence lane. It also explains SH367's fault (deep GL dispatch SFNEGV'd
before the completion body ran) without forcing a "real surface alone unblocks everything" reading.

## Files
- arm64jit/src/jit.rs: +sh369 hermetic (11 word-pins: 0x22985c0/54/d8, 0x2270aac/b4/d0/dc/e4,
  0x2270b5c/64/78/7c, 0x1db1050).
- docs/frontier-sh369-windowattach-persistence-convergence.md (this file).
- HANDOFF.md, STATUS.md updated. Commit: local dev only (operator pushes).

## Verify
- `cargo test -p arm64jit --lib sh369` = 1 passed (real-book pins).
- `cargo test --workspace` green. Route-B live-DM gate UNCHANGED (DM-root 0, MH_* false).

## Do-not-re-tread (refined)
- Window-attach completion re-enters the persistence lane: do NOT re-attack it expecting a DM.
- Persistence lane unchanged: do NOT re-drive LSM sub-call skips (SH349/350/358).
- SH367's "needs real surface" remains the fault *mechanism*; SH369 pins where completion LEADS.