# SH277 — the SEP-17 initEngine_/getFlagsFromEngine_state-dispatch gate measured at the new feed state

Sep 17, 2026 · hermes-worker · single-agent (cone suppressed) · hermetic `sh277` (real-image pins) · workspace green

## What this is

SH275 (client-settings signed) + SH276 (engine-settings receive) latched the two named **feeds** that the SEP-17
directive says `initEngine_`'s "Engine settings is null" hard-assert needs, alongside the window/GL-surface
(APP_CMD_INIT_WINDOW + `wire_real_window`, both already armed). The genuinely-open question this cycle closes:
**now that both settings feeds execute headlessly, does the engine's own `initEngine_`/`getFlagsFromEngine_`
region [0x102bd1a30,0x102bd1d08) fire — and does the app-start line shift under the combined feed?** Both were
measured (fresh region-watch, real libroblox.so); neither advances, and the mechanism WHY is now byte-pinned.

## Measurement A — clean session path (--v2boot-skip-appstart, settings feeds ON)

Region-watch `JIT_REGION_WATCH=0x102bd1a30-0x102bd1d08,0x102bd1d68-0x102bd2128,0x102207b50-0x102209000`:

- The do-init **app-shell ctor 0x102207b50 runs 78 distinct block-entry pcs** to 0x102208eac — deeper than
  SH239's 61-block measurement, and `nativeGameGlobalInit` returns Ok clean, EXIT 124, no crash. This is a
  new measured **deepest clean world-build reach** with the session feed present.
- The `initEngine_` dispatch region gets **0 hits for the entry/resume pcs** (0x102bd1a30/0xa3c/0xd08).
  The only in-window fire is 0x102bd1c38/0xcac/0xccc = the SH276 engine-settings **receive** rung itself.
  So even with client-settings + engine-settings signed both latched, the engine never dispatches into a
  settings body headlessly.

## Measurement B — full ladder (no skip-appstart, settings feeds + LSM_NODES ON)

`JIT_REGION_WATCH=0x101d9a400-0x101d9a580` across 3 runs: **deterministic 3/3** `SIGSEGV guestpc=0x101d9a528`
(the SH268 LSM free-list/pop write-off wall, R-E exec segment write:off). The client/engine-settings feeds are
**consumers** — they do NOT shift the app-start terminal. This A/B (full-session-drive + both new settings
feeds + LSM crossing) had never been measured; it is now closed: same parked wall, no advance.

## The mechanism (why a fabricated manager can't reach a settings body) — pinned

Fresh disasm of `initEngine_` dispatch at 0x2bd1d08 (the "Engine settings is null" gate is downstream of this):

```
0x2bd1d08  ldr  w8,[x19,#16]        ; state word [this+16]
0x2bd1d10  cmp  w8,#0x9 ; b.eq -> 0x2bd2668
0x2bd1d1c  cmp  w8,#0x3 / b.ne      ; ==3 -> 0x2bd1d68 (settings-serializer)
0x2bd1d34  bl   0x2bd24b4           ; ==5 body
0x2bd1d40  bl   0x2bd2668           ; ==9 body
0x2bd1d44  mov  x0,x19,#0x14; b pthread_mutex_unlock  ; ELSE (incl state 0) -> benign tail
```

A fabricated/zeroed manager has `[this+16]=0` → the **benign tail** every time. SH276's latch sets
`[this+648]=1` and leaves `[this+16]=0`, so the engine-settings-received flag is set but the state word never
becomes 3/5/9 — initEngine_ mono-tails. This is the mechanical version of SH164c's antiverdict and the reason
feeding the two settings inputs can't manufacture a DM: the settings path needs `[this+16]==3`, which only a
real session's manager state-transition (not a fabricated-manager receive) produces.

## Honest framing

Both feeds are consumers, not causes — measured, not judged, at the new combined state. No DM manufactured;
route-B live-DM structural gate UNCHANGED (DM-root 0, MH_* false); SH174 capture-latch stays the single forward
hook. The value: (1) the deepest-clean app-shell world-build reach (78 blocks, GlobalInit returns Ok) is
recorded, (2) the full-ladder app-start terminal is re-verified UNCHANGED under the new feeds (3/3), (3) the
initEngine_ state-dispatch gate that blocks the settings path is byte-pinned so a future drive knows the exact
gate: it must make the real session set `[this+16]==3`, not re-feed inputs.

## Verify

`cargo test -p arm64jit --example elfjit sh277` = 1 passed with real-image pins; examples **110/0** (was 109);
`cargo build --workspace` + `cargo test --workspace` EXIT 0. Repro: `runs/capture_sh277_init_engine_feed.sh`.
recon-v3 render plane stays verified green at HEAD.