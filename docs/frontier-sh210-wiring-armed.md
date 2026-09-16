# SH210 — ENTIRE Route-B latent wiring VERIFIED ARMED at HEAD (G1+G2+G3 + capture latch + DM probe)

Author: hermes-worker (autonomous, single-agent — Route-B cone suppressed). Date Sep 16
2026. Companion: runs/capture_sh210_wiring_armed.sh (repro), runs/sh210-wiring-armed.txt
(gitignored sample). Workspace green (cargo test --workspace EXIT 0). No production code
change — this is a verification cycle that proves every installed Route-B gate mounts its
concrete marker on a clean completing ladder.

## WHY THIS CYCLE

SH209 measured the NativeDataModelManager DM-creator region unreached (fresh negative).
This cycle answers the complementary question single-agent: of the LARGE suite of shipped,
default-inert Route-B levers (SH119/120/86/93/107/109 + G1/G2/G3 surface wiring + SH174
capture latch + SH155 DM probe), are they ALL actually ARMED and mounting their markers at
HEAD? If any gate silently stopped firing, a future live-DM advance would find it missing.
Result: on a clean full-ladder run ALL five classes of marker fire in ONE run.

## METHOD

Canonical completing ladder (`elfjit ... --startapp 0x258b144 --v2boot
--v2boot-surface-handoff --v2boot-send-appevent --v2boot-set-filesdir`) + the canonical
Route-B env INCLUDING the SH174 capture latch (JIT_DM_ALLOC_CAPTURE=1 + _DELEGATE=1).
N=3 runs; per run counted five marker classes:
- arm: capture-latch delegation trail ARMED (`SH167/SH169 routed ... ACTIVE hook -> capture trail`)
- xid: G1 surface-handoff wrote the wired XID to [0x10683d348]
- appevent: G2 SendAppEventOnAppReady driven with 'Home' in x5
- filesdir: G3 --v2boot-set-filesdir libc++ string SEEDED + verified
- dmprobe: SH155 DM-root probe printed (once-guard + DM-root + app-data-model counter)

## RESULT (3 runs, run-2 = the full-mount run)

Run 2 (EXIT 124, 0 crash): `arm=1 xid=1 appevent=1 filesdir=1 dmprobe=1` — ALL FIVE mount.
Evidence lines (runs/sh210-wiring-armed.txt):
- `[routeb-dmalloc] SH167/SH169 routed CRT operator-new ACTIVE hook 0x1067daaf0 -> capture
  trail 0x7f00000001e8 (prev_hook 1021ebaf4, default_hook 0x0) ... + FIRST call#1..#8
  (delegate prev_hook 0x1021ebaf4)` — capture latch ARMED and DELEGATING (engine's own
  allocator-hook preserved, real allocs through it; validated SH174 STEP A/B).
- `[elfjit:v2boot] surface-handoff: wired_xid=0x200000 [0x10683d348]=0x200000 MH_FLAGS_LOADED=false
  MH_APP_READY=false` — G1 mounts the real wired XID at the engine's EGL-window cell,
  MH_* false (set-only, correct).
- `[elfjit:v2boot] driving SendAppEventOnAppReady @ guest 0x102bb463c (event="Home" ... IN x5)` +
  `SendAppEventOnAppReady returned Ok(0x3e8) w19-event=0x1` — G2 ABI-correct (Home in x5),
  benign soft-return w19=0x1 (the SH206-pinned result; the 'Home' discriminator is
  correct-but-latent, fires only with a live DM behind the pipe).
- `[elfjit:v2boot-setfilesdir] ... ptr=0x107334000 size=36 = "/data/user/0/com.roblox.client/files" SEEDED` — G3 mounts the files-dir libc++ string, verified by read-back.
- `[elfjit:v2boot] SH155 DM-root probe: once-guard=0x0 DM-root[0x106a68818]=0x7faab4034fc0
  vt+0x30=0x0 mark_b(liveDM)=false app-data-model-count[0x106dca000+0xe88]=0x1` — the
  standing honest state (once-guard=0 re-seeded by SH126 for a fresh app model; DM-root's
  vt+0x30=0 => not a live DM; count=1 = the baseline JSON write). **Route-B live-DM stays
  the gate — this cycle only proves the wiring is ARMED, not that a DM exists.**

Runs 1 & 3 (EXIT 124, 0 crash) armed the latch (arm=1) but the ladder stopped at the
known run-variable SH198/SH55 V2Init singleton-vtable flake BEFORE the later rungs
(xid/appevent/filesdir/dmprobe=0) — the same pre-existing ~1/3 V2Init outside-image stop,
NOT a wiring regression (run 2 proves the full path CAN fire).

## CONCLUSION

Every installed Route-B lever is ARMED and mounting its marker at HEAD: capture latch
delegating, G1 surface XID at the window cell, G2 'Home' ABI-correct latently, G3
files-dir seeded+verified, SH155 DM probe live. The full suite fires together on a single
clean completing ladder. The only thing missing is the live DataModel itself (SH209
measured its creator already gate-unreached headlessly; once-guard=0 + vt+0x30=0 =
no live DM today). **Nothing in the latent wiring is left un-armed — the SH174 runbook's
post-migration arming of *(0x106391908) remains the exact single forward hook.**

## TREE / VERIFY

- Repro: runs/capture_sh210_wiring_armed.sh (N arg; prints per-run marker counts +
  summary). Sample: runs/sh210-wiring-armed.txt (gitignored).
- `cargo test --workspace` green at this commit; SH208 + recon-v3 (self-driven frames /
  json-zero-fix) and SH209 all untouched (no production code this cycle).