# Open-Sober run state (hermes-worker)

## HEAD: `dev` branch, SH307 (force preload-overrides value branch — SendAppEventOnAppReady crosses its standing terminal) — workspace green.

**State**: `cargo test --workspace` EXIT 0 (**585/0**: 405 lib + elfjit examples 132/0 + fsmap + others). `cargo build --workspace` EXIT 0. elfjit.rs under the 1MB pre-commit hook (+178 B margin). Not pushed (operator pushes dev).

## This session (SH307)

1. **Forward lever on the standing SESSION-CTOR terminal (Route-B).** SH270 wired the
   nativePreloadFlagOverrides value cell [0x106a64d78] but measured it INERT; SH272
   concluded both getter branches are live-object walls. Found the missing reason: the
   getter 0x2dae5f0 is a Meyers lazy once whose **guard helper** (bl 0x57816f0, once byte
   [0x6d2df30]) routes control to the CONSTRUCT branch on every headless run — the value
   cell is NEVER read, so SH270's wire was inert. SH307 forces the VALUE branch (NOP
   `tbz w0,#0,0x2dae624` @0x2dae5fc) + seeds [0x106a64d78] with the SH248f fabricated
   all-leaf object, so the getter returns non-NULL.
2. **MEASURED A/B (real libroblox.so, SH269 ladder):** baseline SIGSEGV at the standing
   wall guestpc=0x102bb803c (EXIT 139, x20=0); FORWARD → patch fires, **SendAppEventOnAppReady
   returns Ok(0x107273d50)**, 0 SIGSEGV, ladder completes cleanly (LADDER_DONE=1, EXIT 124).
   First complete past 0x102bb803c.
3. **HONEST:** no DM (DM-root 0, MH_* false) — the getter returns a fabricated object, not a
   real preload map. Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays the
   single forward hook.

## Standing (honest, unchanged across sessions)

- **Route-B live-DM structural gate UNCHANGED**: no make_shared<DataModel> fires headlessly;
  DM-root [0x106a68818] stays 0; MH_APP_READY stays false until a real do-init owns a live DM.
- SH174 capture-latch (arm *(0x106391908) at a real session make_shared) stays the single forward hook.
- **SESSION-CTOR is the primary lever** (operator Sep-17). SH307 (this cycle) forced the
  preload-overrides value branch on SendAppEventOnAppReady — a genuine forward move of that
  rung's terminal.
- Everything achievable headlessly (llvmpipe); GPU host for performance later.

## Next-forward candidates

(a) After SH307 crosses the preload-overrides wall, SendAppEventOnAppReady's body continues —
   region-watch the body (0x2bb8000..0x2bb8300) past the returned-Ok point for the next
   live-object it derefs. (b) R1 content path (synthetic CoreScript module staged+tested,
   latent until a live DM requests rbxasset://). (c) SH304 session-gated producer fires from the
   session side the instant a real session owns a live DM (latent-but-correct).