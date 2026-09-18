# Open Sober — Agent Handoff
## SH340 (Sep 19, 2026, hermes-worker): quantified the Route-B live-DM gate on the do-init pipe; SH339 closed SH308's open Step-2 ABI question (measured negative).

Single-agent (cone suppressed), measurement-focused cycle (no production edit beyond the SH339
default-inert guard), workspace green (590/0).

SH339 (committed 4b79b17): `routeb_appevent_w19_guard` (jit.rs, JIT_ROUTEB_APPEVENT_W19 —
default-inert) captures SendAppEventOnAppReady's event-discriminator INPUT at block boundary
0x102bb46b8 (region-watch-verified). MEASURED 4/4 real-libroblox.so runs: [sp].b0=0x0c
(libc++ SSO size 6) → the `cmp #0xc/#5/#4` chain falls through to event-code **0, not 4**; the
fabricated handle bytes ARE `486f6d65` ("Home") but the RBX string materialized at [sp] is not.
=> the harness 'Home' event does NOT reach the router as 'Home'. REFINES SH307/308: the do-init
pipe forward-reach (app-shell ctor/FMOD/StartAppWithParams, app-data-model 0→1) still fires with
event-code 0 — driven by the SH126 sync-gate, NOT 'Home' routing — so Route-B's session advance
is robust to the fragile fabricated-jstring round-trip. CRITICAL do-not-judge: post-return
se.x[19] is ALWAYS 0 (x19 callee-saved & restored) — it cannot answer event-routing questions.

SH340 (committed 1d761b4): region-watch across 5 frontier bands on the SH307-forward deep
reach. Distinct block-entry pcs: StartAppWithParams 18 / do-init 39 / **app-shell ctor 77** /
**governor 0 (SILENT)** / **ScriptContext loader 0 (SILENT)**. The app-shell ctor world-build
runs deep (deepest pc 0x102208eac, the SH308 FMOD audio-iterate tail) but does NOT continue
into the governor 0x102e9fa80 or ScriptContext/CoreScripts Lua loader — the exact live-DM
gate, now quantified. No crossing (full-ladder governor tail still dies at the LSM wall
SH260/268). SH174 capture-latch stays the single forward hook. Docs: fronti-
er-sh339-appevent-w19-negative.md, frontier-sh340-session-half-silent.md. Repro probes under
runs/ (capture_sh339_appevent_w19.sh, capture_sh340_session_half.sh; run captures gitignored).

HONEST: measurement & precision, not a session advance — Route-B live-DM structural gate
UNCHANGED (DM-root 0, MH_* false). NEXT (unchanged): the real Activity/AppBridge session drive
that registers the "App" service (SEP-17 SESSION-CTOR lever); V1-AppStart-on-populated-registry
is measured-negative (SH337).