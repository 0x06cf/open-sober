# Frontier SH395 — CORRECT the SH378/SH394 "capture latch never installs" attribution: it never installed because JIT_DM_ALLOC_CAPTURE_DELEGATE was UNSET and the engine's real allocator hook is present — with DELEGATE the latch installs + fires cleanly on the SAME furthest-advancing composition, delegating real allocations through the engine's own hook. The "no DM" verdict is now proven by a WORKING observer, not a refused-install artifact.

Date: 2026-09-21 (this cycle), hermes-worker, single-agent (cone suppressed). Two new probe
scripts `runs/capture_sh395_opnew_observer_audit.sh` + `runs/capture_sh395b_dmcap_delegate_fulltable.sh`
(never-run compositions). No production Rust / guest byte / JIT-hook-default touched. Workspace
green at start and end (cargo test --workspace EXIT 0). All recon-v3 immediate-priority deliverables
re-verified green at this exact HEAD (capture_taskv4_frame.sh attempt 1: 24 real task-driven frames
`present swap Ok(0x1)`, 0 json abort, 0 crash; capture_sh304: session-gated producer INERT; JSON
len-clamp at 0x102355d40 present).

## Why this cycle

SH378 then SH394 both reported the single forward observer (SH174 `routeb_dm_alloc_capture` latch)
"NEVER installs" on the furthest-advancing composition — SH394 read it as "operator-new is never
enterinstallably on this composition," i.e. an apparent strengthening that the engine does not even
allocate headlessly. But the latch guard `routeb_dm_alloc_capture_guard` (jit.rs) has a documented
SAFE-LATCH branch: it only replaces a nonzero ACTIVE allocator-hook global under
`JIT_DM_ALLOC_CAPTURE_DELEGATE=1`, and returns without installing otherwise. The engine ships its
OWN real allocator hook (measured prev_hook=0x1021ebaf4 below), and both SH378 and SH394 ran WITHOUT
DELEGATE. So "never installs" was plausibly the safe-latch refusing to replace a live engine hook —
an OBSERVER ARTIFACT, not a measurement of "no allocation." SH395 audits this and runs the
never-armed-with-DELEGATE A/B.

## SH395a probe (capture-only, same env as SH394) — operator-new IS reached

Region-watched the whole CRT operator-new band [0x102a0d940,0x102a0da00) + DMCONT operator_new
entries on the exact SH394 full-table env (no DELEGATE). Even though the latch STILL never installed
(the safe-latch refused), the region-watch PROVES operator-new executes headlessly:
- 0x102a0d9b8 (CRT wrapper) 1 hit, 0x102a0d9fc 1 hit;
- DMCONT operator_new 0x101db1a38/0x101db1ad4/0x101db1adc/0x101db1afc/0x101db1b78/0x101db1c8c and
  the free-list allocator tail 0x101db1c60 all hit; 0x101d96768/0x101d96778/0x101d967b0 hit.
Terminal unchanged: SIGSEGV/SIGABRT at the standing SH285 LSM pool-pop wall guestpc 0x101d9a528
(EXIT 134); glue-full 15/15 Ok; SendAppEventOnAppReady returned Ok.

## SH395b A/B (the discriminator) — with DELEGATE the latch INSTALLS and FIRES cleanly

Same full-table env + `JIT_DM_ALLOC_CAPTURE_DELEGATE=1`:
```
[routeb-dmalloc] SH167/SH169 routed CRT operator-new ACTIVE hook 0x1067daaf0 -> capture trail 0x7f00000001e8 (prev_hook 1021ebaf4, default_hook 0x0) at pc=0x102a0d9b8
[routeb-dmalloc] FIRST call#1: bytes=0x18 ... -> base 0x7fe050571890 (delegate prev_hook 0x1021ebaf4)
[routeb-dmalloc] FIRST call#2..5: bytes=0x18 ... (delegate prev_hook 0x1021ebaf4)
```
- The latch INSTALLS on the same composition SH394 said it "refuses to arm" on. Prev_hook=0x1021ebaf4
  (the engine's real allocator hook — captured, not guessed).
- It FIRES: calls 1-5 delegate real allocations (0x18-byte objects) through the engine's own hook,
  and delegation completes CLEANLY (no std::bad_function_call — SH344's `bad_function_call` was from
  the separate JIT_ROUTEB_DM_REALCTOR drive in that env, not from delegation itself).

## Interpretation — a genuine attribution correction

1. **SH378/SH394's "latch never installs ⇒ operator-new unreachable" is an artifact, now refuted.**
   The latch installs whenever DELEGATE is set, on the SAME composition. What both cycles measured was
   the safe-latch refusing to replace a PRESENT engine hook with DELEGATE unset — that is the code's
   intended guard behavior, not evidence the engine does not allocate.
2. **The "no DM yet" verdict STILL STANDS, now from a WORKING observer.** The armed trail captured
   calls #1-5 all as 0x18-byte allocations — none DM-plausible — so the engine has genuinely not made
   a make_shared<DataModel> on this reach, and that is now a verified-negative-by-working-observer
   rather than an ambiguous refused-install. The forward observer is functional, just silent on real
   DM sizes because the DM is not constructed on this further reach.
3. **The engine's real allocator hook is 0x1021ebaf4** (measured for the first time cleanly), and
   DELEGATE-delegation itself is clean — correcting SH263/SH344's conflation of delegation with the
   DM_REALCTOR crash.

Route-B live-DM structural gate UNCHANGED (DM-root 0, MH_* false, AppBridgeV2 0). This is an
attribution correction + observer-validity proof (map-completion), NOT a DM advance — but it matters:
the single forward hook that every Route-B verdict rests on is now proven capable of arming and
capturing, so future "0 validated" results are trustworthy-by-construction.

## Do-not-re-tread (all prior closures stand, unchanged)

SH393's cmd 1/13/15/17/18, LSM skips (SH349/350/358/373), EC reader-gate (SH355/356/374),
0x258b5d8/SetInitParams (SH362/375), window-attach real (SH367), ALooper (SH365), governor gates
full-ladder (SH379), -9 string (SH380), map-header (SH248h), once-lambda store (SH381), SH267
node-cell (SH385), setDataModelToCurrent (SH388), LSM-manufactured wiring (SH385), and do NOT run
the SH174 latch WITHOUT DELEGATE expecting an install (safe-latch refusal — SH395; use
JIT_DM_ALLOC_CAPTURE_DELEGATE=1 to arm it).

## Files

- runs/capture_sh395_opnew_observer_audit.sh (new, committed)
- runs/capture_sh395b_dmcap_delegate_fulltable.sh (new, committed)
- Logs /home/hermes-worker/runs/sh395-opnew-observer-audit.txt + sh395b-dmcap-delegate-fulltable.txt (outside repo)
- Commit: local dev only (operator pushes).