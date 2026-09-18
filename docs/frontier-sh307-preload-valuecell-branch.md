# Frontier SH307 — force the nativePreloadFlagOverrides value branch: SendAppEventOnAppReady crosses its standing preload-overrides terminal

Date: Sep 18, 2026, hermes-worker. Single-agent (cone suppressed). Route-B
SESSION-CTOR forward lever on the current standing terminal — a freshly-tried
combination (default-inert).

## The gap SH270/272 left (why this is new, not a re-tread)

SendAppEventOnAppReady (0x102bb463c, the post-ladder session-ctor rung) had a
triple-pinned terminal at guestpc=0x102bb803c (`ldr x8,[x20]` where x20 =
`nativePreloadFlagOverrides` return = 0). SH270 **wired** the getter's value cell
[0x106a64d78] to the engine's own constructed singleton base and measured the wire
"INERT"; SH272 disassembled both getter branches and concluded both are live-object
walls.

But that conclusion was missing the reason *why* SH270's wire was inert: the getter
0x2dae5f0 is a Meyers lazy once whose **guard helper** (`bl 0x57816f0`, once byte
[0x6d2df30]) routes control to the **CONSTRUCT branch** on every headless run. The
value cell [0x106a64d78] is only read on the **VALUE branch** (guard bit0=1), which
was never taken — so SH270's wire sat in a cell the getter never reached. Nobody had
ever *forced* the value branch + supplied a dispatch object. That is the fresh
combination.

## What SH307 does (default-inert, opt-in JIT_ROUTEB_PRELOAD_VALUECELL)

1. NOP the `tbz w0,#0,0x2dae624` @0x2dae5fc (0x36000140 -> 0xd503201f) so the getter
   **always falls through** to the value-cell path (0x2dae600) instead of branching to
   the construct path.
2. Seed [0x106a64d78] = `routeb_appstart_adapter_object()` (the SH248f fabricated
   all-leaf dispatch object) so the value-cell branch's `ldr x0,[x0]; ldr x2,[x8,#16];
   br x2` dispatches a benign host leaf and returns a **non-NULL** object.
3. block_cache_drop_region + word-guard + mprotect RW/RX (SH245 pattern).

Word-guarded (binary-drift fails loudly), idempotent, self-gated on its own env.

## Measured (real libroblox.so, SH269 canonical ladder, A/B)

- **BASELINE (SH307 off, GOVFLAG=1):** `[SIGSEGV] fault=0x0 guestpc=0x102bb803c`
  EXIT 139 — the standing preload-overrides wall, unchanged. `x20=0x0`.
- **FORWARD (+JIT_ROUTEB_PRELOAD_VALUECELL=1):** `[elfjit:routeB] SH307 preload
  value-branch@0x102dae5fc tbz->nop + value-cell [0x106a64d78]=0x55..d680: getter
  returns obj`; **`SendAppEventOnAppReady returned Ok(0x107273d50)`**, 0 SIGSEGV, and
  the **ladder completes cleanly** (`LADDER_DONE=1`, admission gate cleared,
  EXIT 124). The preload-overrides terminal is crossed.

First time SendAppEventOnAppReady has completed past 0x102bb803c — a real forward
move of the session-ctor terminal, not a re-verify.

## Honest (do-not-over-claim)

- Does NOT manufacture a DataModel: DM-root [0x106a68818] = 0; MH_* stay false
  (post: FLAGS_LOADED/ENGINE_INITIALIZED/APP_READY all false); app-data-model count
  advanced 0x1. Route-B live-DM structural gate UNCHANGED.
- SH270/272's mechanism analysis (both getter branches need a session) remains
  correct; SH307 does not resurrect them as live-object seeds — it forces the *branch*
  and supplies a fabricated dispatch target, which is a distinct (and measured)
  mechanism. The getter still returns a fabricated object, not a real preload map.
- SH174 capture-latch (arm *(0x106391908) at a real make_shared<DataModel>) stays the
  single forward hook.
- recon-v3 deliverables stay shipped + verified.

## Verify / files

- `cargo test -p arm64jit --example elfjit -- sh307` = 1 passed (real-image anchors:
  tbz 0x102dae5fc, value-branch dispatch 0x102dae604/620).
- `cargo test --workspace` EXIT 0 (585 passed / 0 failed: 405 lib + examples + others).
- `cargo build --workspace` EXIT 0. elfjit.rs under the 1MB pre-commit hook (+178 B
  margin). jit.rs: `routeb_appstart_adapter_object` made `pub`.
- Repro `runs/capture_sh307_preload_valuecell.sh` (A/B). Logs kept locally.
- Commit: local `dev` only (operator pushes).