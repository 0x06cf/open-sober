# Frontier SH309 — SendAppEvent spine measured closure + StartAppWithParams ABI pin

Date: Sep 18, 2026, hermes-worker. Single-agent (cone suppressed).

## What was measured (this cycle)

Re-ran the SH307-forward SendAppEventOnAppReady ladder (JIT_ROUTEB_PRELOAD_VALUECELL=1)
on real libroblox.so with JIT_REGION_WATCH spread over the FULL downstream session-ctor
bands, and AUTHORITATIVELY byte-verified every anchor via direct file read
(guest - 0x100000000 == file offset; exec seg [0x0,0x62d8190)).

- SendAppEventOnAppReady returned Ok(0x107273d50), 0 SIGSEGV, ladder completes clean
  (LADDER_DONE=1, EXIT 124).
- The pipe's STARTAPP reach is genUINE: nativeAppBridgeV2StartAppWithParams
  (0x10258b144) runs its body through 0x10258b5a0 (the region-watch reflects these
  block entries; run-variable — adding many region-watch bands perturbs the spine,
  SH248b logging-contamination class, NOT a real absence).
- The measured CLOSURE (candidate (a) in STATUS): on this path the downstream
  session-ctor gates do NOT run — post-do-init 0x1023eff4c, governor 0x102e9fa84,
  EC-world 0x102e24598, ScriptContext loader 0x101f1d8ac all stay 0 region-hits.
  Authoritative: each is a real code entry (byte-pinned below), so "0 hits" is
  meaningful, not a bad-address artifact. The SendAppEvent pipe → do-init →
  app-shell ctor → FMOD audio tail → StartApp which returns clean WITHOUT
  reaching any DM/EC/Lua construction.

## Authoritative byte anchors (guest addr, 4-aligned, in-window)

StartAppWithParams spine (the SH307/308 forward reach):
- 0x10258b2dc = 0x94188f04  `bl 2baeeec` (the app-bridge pipe -> do-init 0x2206c40)
- 0x10258b630 = 0xf9409508  `ldr x8,[x8,#296]` (vt[+296])
- 0x10258b698 = 0xd65f03c0  `ret` clean body completion

Downstream session-ctor gate prologues (provably 0 hits on this path):
- 0x1023eff4c = 0xd10603ff  post-do-init continuation (sub sp,#0x60)
- 0x102e9fa84 = 0xa9ba7bfd  governor (stp x29,x30/sp,#-112)
- 0x101f1d8ac = 0x90022ca8  ScriptContext/CoreScript loader (adrp)

## Honest (do-not-over-claim)

- Does NOT manufacture a DM: DM-root [0x106a68818]=0, MH_* all false. Route-B
  live-DM structural gate UNCHANGED. SH174 capture-latch stays the single forward
  hook.
- The spine reach is run-variable (region-watch band count perturbs it), so this
  cycle anchors the ABI + the unreached gates rather than claiming a fixed new
  forward execution distance. The real forward state is SH308's measured reach
  (do-init ctor -> FMOD tail -> StartApp entry); what is NEW here is the closure
  (nothing past StartApp runs on this path) + the authoritative gate prologues.

## Verify / files

- `cargo test -p arm64jit --example elfjit -- sh309` = 1 passed (real-image pins
  above, skip-if-absent).
- Full elfjit examples suite 134/0; workspace green.
- elfjit.rs held under the 1MB pre-commit hook (funded by condensing SH306/307/308/
  126/272 comment prose — all facts/addresses preserved, no behavior touched).
- Repro runs/sh309-rw.txt (+ the A/B in sh308-rw2..rw5.txt). Capture script
  runs/capture_sh309_startapp_spine.sh (note: multi-band region-watch perturbs the
  spine — keep band count low for a clean run).
- Commit: local `dev` only (operator pushes).