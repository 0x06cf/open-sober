# Frontier SH406 — GOVFLAG seed extended to the app-start MAIN-arm continuation
# (extends SH269's fixed-.bss flag seed to the SH405 MAIN arm; next-gate after SH405)

Date: 2026-09-21, hermes-worker, single-agent (cone suppressed). Workspace green
before and after (cargo test --workspace EXIT 0, arm64jit lib 455 with the new sh406
hermetic; elfjit.rs untouched; jit.rs added the sh406 hermetic + 1 pc-window widening).

## Why this cycle

SH405 measured that arming JIT_ROUTEB_DONEPATH_MAIN flips the do-init onto its MAIN
dispatch: the never-executed 0x10258b5d8 app-start body RUNS deep, but its continuation
faults SIGSEGV (fault=0x0) at guestpc 0x1025f501c right after `bl 0x25f52b4` returns
x0=0. SH405's "Next" named two open questions: (1) does the MAIN-arm app-start drain head
toward DMCONT/do-init-completion or the same LSM lane, and (2) DMCONT 0x102bd1d68 is 0
hits. SH406 answers (1) with execution evidence and (2) stays closed.

## Root cause of the 0x1025f501c fault (disasm + live)

The app-start continuation reads the SAME fixed-.bss flag byte [0x106a64da0] the governor
reads at SH269 (`ldrb w8,[x9,#3488]` @0x25f502c, x9=adrp 6a64000). When the flag==0 the
`cbz w8,0x25f504c` @0x25f503c falls through to `ldr x0,[x19,#1032]` (NULL app-DM
controller) -> `ldr x8,[x0]` fault=0x0. When flag!=0 it takes `mov x0,x19; bl 0x2ea3a84`
(preload-overrides helper with the REAL app obj x19, non-NULL) then branches over the deref.
SH269's GOVFLAG seed only windows the governor pcs (0x102ea0b60..0x102ea0bd0), which the
SH405 MAIN arm bypasses — so the flag was never seeded on this path and stayed 0.

## The fix (default-inert, idempotent — NO production behavior change when unselected)

Widened `routeb_govflag_seed_guard`'s pc-window to ALSO cover the StartAppWithParams deep
continuation region (0x1025f5008..0x1025f5060), so under JIT_ROUTEB_APPSART_GOVFLAG the
same [0x106a64da0].bit0=1 seed fires when the MAIN arm reads it. This routes the cbz to the
preload-overrides helper (real x19) instead of the NULL [x19,#1032] deref. Same flag, a NEW
reach (SH405's MAIN arm) the original governor-window missed. New real-image hermetic
`sh406_govflag_seed_covers_appstart_main_arm_continuation` (arm64jit lib 454->455)
byte-pins the flag read (0x39768128), the cbz (0x34000088, imm19=+0x10), the NULL-controller
deref (0xf9420660/0xf9400008) and the helper branch (0xaa1303e0) — all verified on real
libroblox.so.

## Live measurement (real libroblox.so, SH405 env + GOVFLAG armed, EXIT 139)

- The extended seed FIRES on the MAIN arm: `[routeb-sh269] seeded governor-predicate flag
  [0x106a64da0] bit0=1 at pc=0x1025f5008` — the app-start continuation, NOT the governor.
- The app-start body still ENTERS (guestpc 0x10258b5d8 region pcs 0x10258b5d8..0x10258bbb0
  all hit) and now PASSES the 0x1025f501c NULL-controller deref (no fault there).
- The continuation then drains into the SH341 pool-pop persistence lane — **188 SH341
  pool-pop events** (keys x0=0x55b3..., caller LR=0x10626b6dc) with the SH123 setfix
  substitution firing once. EXIT 139 (the standing persistence-lane class).
- **DMCONT 0x102bd1d68 = 0 region hits** — the MAIN-arm app-start continuation does NOT
  reach the do-init completion construction gates; it lands in the SAME measured-closed
  SH285/LSM persistence family.

## Interpretation (honest: a fencepost, not a DM)

SH406 proves the GOVFLAG flag byte is reached + seedable from the SH405 MAIN arm (one more
reach the original governor-window seed missed) and that seeding it lets the app-start
continuation execute past the NULL-controller deref. The next wall is the standing
SH285/SH341 persistence lane, which is measured-returned vs a real session ctor
(SH248h/SH256/SH349/350/358/373/385/395-398) — do NOT re-drive it. DMCONT stays 0 hits
(honest: no DM manufactured, DM-root [0x106a68818]=0, MH_* false, AppBridgeV2 genuine vt
0x1063a3410 unchanged). This cycle is map-completion + a corrected seed reach on the
SESSION-CTOR line, not a live-DM advance.

## Files

- jit.rs: `routeb_govflag_seed_guard` pc-window widened (0x1025f5008..0x1025f5060) +
  `sh406_govflag_seed_covers_appstart_main_arm_continuation` hermetic (arm64jit lib
  454->455). elfjit.rs untouched.
- Capture: runs/capture_sh405_donepath_main.sh (re-used; SH405 env already arms
  JIT_ROUTEB_APPSART_GOVFLAG=1). Log (outside repo): /home/hermes-worker/runs/sh405-donepath-main.txt.

## Do-not-re-tread (all standing)

SH285/SH341 LSM family (SH349/350/358/373/385/393/395-398), setDataModelToCurrent SH388,
EC reader-gate SH355/356/374, window-attach real SH367, ALooper SH365, governor-gates
full-ladder SH379, once-lambda store SH381, -9 string SH380, map-header SH248h, app-cmd
1/13/15/17/18 SH393, Sh285-crossover re-drive SH396. SH174 capture-latch single forward
observer (with DELEGATE=1, SH395).