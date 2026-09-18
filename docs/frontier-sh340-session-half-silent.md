# Frontier SH340 — measured: the do-init pipe's app-shell ctor world-build runs 77 blocks but the session half (governor/ScriptContext/Lua) is silent — the live-DM gate, quantified

## Session
Sep 19, 2026, hermes-worker. Single-agent (cone suppressed). Measurement-only (region-watch,
no production edit). Workspace green (590/0). Repro `runs/capture_sh340_session_half.sh`.

## What was measured (real libroblox.so, SH307-forward send-appevent deep reach)
JIT_REGION_WATCH across the five frontier bands on the canonical SH307-forward run
(--v2boot-send-appevent, --v2boot-session, PRELOAD_VALUECELL=1). Distinct block-entry pcs
per band:

```
StartAppWithParams 0x10258b144..0x10258b900 : 18 hits  (deepest-driving rung)
do-init            0x102206c40..0x102207c00 : 39 hits  (SH307 pipe entry)
app-shell ctor     0x102207b50..0x102209000 : 77 hits  (world-build runs DEEP)
governor           0x102e9fa80..0x102ea3b40 :  0 hits  (silent)
ScriptContext      0x101f1d8ac..0x101f1da00 :  0 hits  (silent)
```

The app-shell ctor path (0x102207b50) walks 77 distinct block-entry pcs — the deepest clean
world-build on the send-appevent do-init pipe — but control does NOT continue into the
governor 0x102e9fa80 or the ScriptContext/CoreScripts Lua loader 0x101f1d8ac (0 hits both).

## What this confirms (do-not-over-claim)
This is a QUANTIFICATION of the standing Route-B structural gate, not a crossing:
- SH308's honest statement ("app-shell ctor runs but the session half — governor,
  ScriptContext, Lua — is still gated on a live DM") is now precisely measured on this
  path: app-shell 77 blocks deep, governor/Lua ZERO.
- The app-shell ctor world-build is the deepest clean reach on the send-appevent pipe.
  The next hop (governor 0x102e9fa80) is the live-DM gate — its body reads gov+0x408 ->
  blr vt+0x18/0x30 (ROUTE-B RECON V3) and only produces a real session when a live DM exists.
- Consumes no DM; the prior finding stands: the governor tail DOES fire on the FULL --v2boot
  ladder (SH231 measured 0x102e9fa84..0x102ea30dc), but that path dies at the LSM/persistence
  wall (SH260/268) — neither headless route reaches Lua.
- SH174 capture-latch remains the single forward hook.

## Verify / files
- `cargo test --workspace` green (590/0, verified this cycle at HEAD including SH339).
- No production code edited (guard from SH339 is already committed; SH340 is pure region-watch).
- Repro: `runs/capture_sh340_session_half.sh` (+ runs/sh340-sessionhalf.txt, sh340-full.txt
  captured locally, gitignored per CLAUDE.md).
- Commit: local `dev` only (operator pushes). Doc + probe only.