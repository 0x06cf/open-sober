# Frontier SH360 — implement the operator's do-init empty-vector gate seed: the app-shell band's 0x20-stride vector walker at [0x106dcb160] now collapses a constructed-empty begin==end host-heap pair to NULL (default-inert, measured)

## Session
Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). One new hermetic
(sh360 doinit_emptyvec_gate_inert_and_fires, arm64jit lib 422->423) + one probe
script. Production elfjit.rs unchanged; jit.rs adds ONE default-inert opt-in
guard `routeb_doinit_emptyvec_gate` (JIT_ROUTEB_DOINIT_EMPTYVEC) wired into the
block-entry loop. Workspace green (cargo test --workspace exit 0).

## Why this cycle
The operating directives' "EXECUTE DO-INIT GATES" names a specific, NOT-yet-
implemented gate seed: empty-vector shape (begin==end==NULL, count 0) at
[0x106dcb160], in the do-init app-shell band (file 0x2208e4c..0x2208eac, the
band SH340 measured running 77 blocks deep on the send-appevent pipe). The
disasm confirmed the cell: `adrp x19,6dcb000; add x19,x19,#0x160` (x19=
0x106dcb160), `ldp x20,x21,[x19]` (begin=x20,end=x21), `cmp x20,x21; b.eq` —
a 0x20-stride vector walker whose populate loop (dispatch `blr [obj+0x18]` per
entry) and teardown loop (`bl 0x21c7948` per entry) both `b.eq`-early-exit when
begin==end. Seeding the pair to begin==end==NULL lets both loops take the
empty-vector early-exit instead of walking/destroying garbage.

## Implemented + MEASURED (real libroblox.so, full --v2boot-session send-appevent ladder)
- Guard fires at the two real BLOCK-ENTRY pcs (verified via JIT_REGION_WATCH —
  the interior pcs e58/e84 are NOT block boundaries; the whole walker is one
  translated block entered at 0x102208e4c and re-entered at 0x102208e88 for the
  teardown re-read). Wrong-pc / env-off are inert (tested).
- MEASURED (2/2 runs): `[routeb-doinit] SH360 empty-vector gate @0x106dcb160 =
  {0,0} (begin=0x7f51c401c9f0 end=0x7f51c401c9f0)` — the vector is ALREADY a
  constructed-empty (begin==end==host-heap pointer pair). The guard collapses it
  to {0,0} (NULL), which is behavior-preserving (the walker compares equality
  either way; a populated begin!=end live pair is left untouched, tested).
- Completing ladder stays confirmed-green with the gate armed: EXIT 124,
  SH155 DM-root probe=1, 0 SIGSEGV/SIGABRT.

## Honest
Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED
(DM-root [0x106a68818]=0, MH_* false). The gate smooths one app-shell band
walker that the full app-start-driving ladder enters, but that ladder still
ABRTs at the SH285 persistence-lane live-object wall (0x101db1b08) before the
walk continues — the same run-variable class SH344/344c/346 documented. On the
completing skip-appstart ladder the band is not reached (0 gate hits), so the
gate is fully latent there. This is the operator-named do-init gate made real
+ measured; it does not cross the live-DM gate.

## Files
- crates/arm64jit/src/jit.rs: +routeb_doinit_emptyvec_gate (default-inert,
  JIT_ROUTEB_DOINIT_EMPTYVEC), wired at block-entry; +sh360 hermetic test.
- runs/capture_sh360_doinit_emptyvec.sh (probe; live captures gitignored).
- HANDOFF.md / STATUS.md updated. Workspace green (arm64jit lib 423/0).