# SH255 — Marshaler-enclosing fn is INDIRECT-ONLY: mechanical grounding of the EC-world live-state gate

## Session
Sep 17, 2026, hermes-worker. Single-agent (cone suppressed per operator Sep-15 directive). Route-B re-attack.

## What this closes
SH235 pinned the marshaler 0x1023f1210's 3 direct callers `{0x1023f075c, 0x102e15bf0, 0x102e33494}`
and SH236/238 measured StartLuaAppDM soft-returns benignly before ever reaching 0x1023f075c. That left
a static residual: *could some OTHER in-image direct call reach the EC world's DataModel factory without
passing through StartLuaAppDM?* SH255 answers it at the enclosing-fn level with a whole-executable scan.

## Measured (real libroblox.so, full executable .text, decoder over LOAD flags=0x5 seg [file 0x0,0x62d8190))
1. **Marshaler-enclosing fn 0x1023f03b4 is INDIRECT-ONLY**: ZERO direct `bl`/`b` callers target
   0x1023f03b4 across the entire executable .text. The body that holds `bl 0x1023f1210` at 0x1023f075c
   is reachable ONLY via indirect dispatch (blr/br) — the fabricatable-object-graph/live-this class,
   not a static seed.
2. **The marshaler's other two callers (0x102e15bf0, 0x102e33494) are NOT in StartLuaAppDM's own body**
   [0x1023efe2c,0x1023f0800); they sit in the high ExperienceController/game-start region (0x2e00000+)
   — the same live-DataModel-gated area SH231 measured headless-unreached (0 region hits). They are
   NOT inside the narrower EC body window [0x102e1c650,0x102e25200) either (correcting the initial
   draft's overreach); their shared character is the 0x2e00000+ game-start region.
3. **Consequence**: there is no in-image static call site that can reach the EC world except either
   (a) StartLuaAppDM's dispatch switch (block-entry measured inert, SH236/238) or (b) game-start
   internal self-calls (gated on the world already running past the live-DM wall). The
   "no headless seed reaches the genuine DataModel factory" verdict is now mechanized at the
   enclosing-fn level, not just the marshaler/branch level.

## Why this is not a re-tread
Prior closures pinned the marshaler's callers (SH235) and measured the dispatch inert (SH236/238).
This cycle adds the ENCLOSING-fn proof — that the function wrapping the marshaler call has zero direct
callers — which was never scanned. It converts "no direct marshaler call reaches EC except via SLADM"
into "no direct call reaches the ENCLOSING fn at all; EC is unreachable headlessly by any static seed,
full stop."

## Route-B live-DM structural gate
UNCHANGED. SH174 capture-latch (arm *(0x106391908) at a real make_shared<DataModel>) stays the single
forward hook. recon-v3 deliverables (self-driven frames + json-abort fix) stay shipped + verified.

## Code
+1 hermetic `sh255_marshaler_enclosing_fn_indirect_only_ec_selfcallers_in_world` in
crates/arm64jit/examples/elfjit.rs (real-image guard family as sh235, skip-if-absent):
byte-pins enclosing-fn entry 0x1023f03b4 = `str d8,[sp,#-112]!` (0xfc190fe8) + the SLADM `bl marshaler`
word 0x1023f075c = 0x940002ad; whole-text scan asserts ZERO direct bl/b callers of 0x1023f03b4; asserts
the two non-SLADM marshaler callers are outside StartLuaAppDM's body and in the 0x2e00000+ region,
4-aligned. Verified 1 passed on the real libroblox.so.

## Repro
`cargo test -p arm64jit --example elfjit sh255`

## Verification
cargo build --workspace + cargo test --workspace green (checked this session). No production code path
edited; default-inert; single-agent.