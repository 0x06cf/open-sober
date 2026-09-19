# Frontier SH371 — continueAfterFlagsLoaded_ now EXECUTES DEEP headlessly (corrects the SH226/228 "never fires" map) + pin that the engine-init dispatcher body is STRAIGHT-LINE (diverge can only be a leaf return)

Session: Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). Two artifacts:

1. Hermetic `sh371_engineinit_dispatcher_body_straightline_to_sub` (arm64jit lib +1)
   — sees real libroblox.so, scans the engine-init dispatcher body [0x2bd8ce8, 0x2bd8d64)
   and the sub_2bd8dac body [0x2bd8dac, 0x2bd8e28) for any control-flow word, rejecting
   everything that is not one of the four known sites:
   - 0x2bd8d14 `bl getter 0x2174c04`
   - 0x2bd8d2c `blr vt[+0xf8]` (leaf)
   - 0x2bd8d50 `blr vt[+0x108]` (leaf)
   - 0x2bd8d60 `bl sub 0x2bd8dac`
   and the sub's terminal 0x2bd8e28 `blr vt[+0x1f0]`.
   Both bodies are branch-free STRAIGHT-LINE, so SH226's "the pipeline takes the benign
   path and never reaches the blr at 0x102bd8e28" is mechanism-wrong: there is NO benign
   branch around the blr. The only runtime exits from the dispatcher are the two host-leaf
   blr returns and the bl sub. SH228's "diverge at a leaf" is narrowed to "a leaf's return
   never lands back in-image (host landing) OR a fault before 0x2bd8d2c."

2. Measurement (runs/capture_sh371_dispatcher_body.sh, full Route-B env = the
   capture_sh344_routeb_reach.sh seed set + JIT_REGION_WATCH on the dispatcher body,
   sub, fnB band, and continueAfterFlagsLoaded_ body):
   - sub_2bd8dac FIRES (1 block-entry).
   - **continueAfterFlagsLoaded_ (0x102bd1d68) FIRES and runs deep — 25+ block-entry pcs
     0x102bd1d68 .. 0x102bd1f64 (its app-name guard region, SH245/SH248c-seeded).**
   - Session markers: MH_* all false, AppBridgeV2 0x0, DM-root probe pending (run died
     before the post-ladder probe at the SH285 wall).
   - Terminal: guestpc=0x101db1b08 fault=0xffffffffffffffff (the standing SH285
     LSM reader/pop live-object wall) — NOT the F+0x18 post-app-start floor that
     routeb_dm_manager_cont's comment predicts; the run dies one fencepost EARLIER,
     inside nativeAppBridgeAppStart's persistence lane.

## Interpretation (do-not-over-claim)

This CORRECTS the SH226/SH228 record that "continueAfterFlagsLoaded_ is never entered."
With the full Route-B seed env (DMCONT + DM_CONT_M48_SEED + CONT_APPNAME_SEED) it is
entered and executes deep past its app-name guard. The reason SH228 measured 0 hits was
env/run-variable (the app-name guard at 0x102bd1f64 faults NULL-write without the M+0x48 /
appname seeds, before the continuation can show as block entries past 0x102bd1d68 entry —
SH228's watch was on [0x102bd1d68,0x102bd2600], so even an ENTER-THEN-FAULT-at-f64 would
have logged the entry as a hit; the difference is the seed set). The Route-B live-DM
structural gate is UNCHANGED: the continuation dives into the SH285 persistence lane, which
is the measured-closed LSM lane (SH349/350 — do NOT re-drive LSM sub-call skips). No
DataModel (make_shared<DataModel>) is produced.

The genuinely-new fact to carry forward: the DM-creator continuation (continueAfterFlagsLoaded_)
now demonstrably RUNS headlessly to the persistence lane, so the standing dead-ends are
(a) the SH285 live-object wall reached from this new continuation path, and (b) the F+0x18
post-app-start controller floor (routeb_dm_manager_cont comment) that is never reached
because SH285 fires first. Both are in the measured-closed lane family.

Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED (DM-root 0,
MH_* false). SH174 capture-latch stays the single forward hook.

## Files
- crates/arm64jit/examples/elfjit.rs: `sh371_engineinit_dispatcher_body_straightline_to_sub`.
- runs/capture_sh371_dispatcher_body.sh (probe, new).
- Live capture ~/runs/sh371-dispatcher-body.txt (gitignored).