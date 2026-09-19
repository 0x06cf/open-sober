# Frontier SH374/SH375 — two measured map-completions under the SH373 reaching-env

Session: Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). One new probe
runs/capture_sh374_ec_dmfn_reaching.sh (SH373 reaching-env + LSM_APPEND_SKIP combined
with the DMFN/EC-world marshaller drive) + one new probe capture_sh375_dispatch_body_reaching.sh
(SH362's dispatch-body trace re-tested under the reaching env). Live captures
runs/sh374-ec-dmfn-reaching.txt + runs/sh375-dispatch-body-reaching.txt. No production
path edited (both probes use existing default-inert guards + the shipped SH349 append-skip
opt-in). Workspace green (cargo test --workspace EXIT 0, 615/0). recon-v3 deliverables
re-verified green at HEAD this cycle (24 real task-driven frames, swap Ok(0x1), 0 json
abort, 0 crash).

## The genuinely-new combinations (neither pair had ever been run together)

SH373 crossed the standing SH285 persistence wall 5/5 (JIT_ROUTEB_LSM_APPEND_SKIP on top
of the SH371 DM_CONT_M48_SEED + CONT_APPNAME_SEED reaching env). Two prior closure
premises were shaped BY the SH285 wall; re-testing them under the crossing env was the
standing question each left open.

### SH374 — EC-world reader-gate under the reaching env
SH298-302 measured the EC-world marshaller reader-gate block 0x2e24694 as NEVER ENTERED
(0/3) because control diverged to the SH285 persistence wall. SH374 combines the reaching
env + append-skip with the DMFN/EC drive (JIT_ROUTEB_DMFN_FIELDS+REGISTER,
EC_ARG1/ARG0VT/REALSESSION/READERGATE_FRAME):
- EC-world ENTRY guards fire every run (routeb-sh298/sh299/sh300 at 0x102e24598) — the
  DMFN drive still reaches the EC world;
- **reader-gate block 0x2e24694 still never enters (0 hits), and the run terminals at
  0x101d9a708 (SH349's pack/name-helper in the SAME unconstructed-LSM family) — control
  LEAVES the EC world into the persistence lane even though SH285 itself is crossed.**
- Verdict: SH356's closure ("reader-gate is a reachability problem, not a value problem")
  is RE-STRENGTHENED from the reaching env — crossing SH285 does not make the EC
  marshaller interior reachable; the run still drains into the persistence family first.

### SH375 — SH362's dispatch-body closure, premise now obsolete
SH362 measured the do-init MAIN-branch dispatch body fn 0x258b5d8 (StartAppWithParams+0x494)
is never entered, attributing it to "every run faults at SH285 before reaching it." SH375
re-runs the body trace under the crossing env:
- **SH285 is deterministically crossed (sh285=0 all 3 runs; the SH349 append-skip logs
  confirm the leaf is made inert);**
- the ladder ADVANCES PAST SH285 to a fresh, later terminal: StartApp returns Ok, the
  SEP-17 session-lifecycle drive runs initAppShellReporter + setActive cleanly, then
  **SetInitParams (0x102bcc814) SIGABRTs (fault=0x3e90009a4d5, guestpc=0x0)** — the
  LSM-family consumer (SH353 documented SetInitParams faults in this lane);
- the 0x258b5d8 dispatch body STILL never fires (0 hits).
- Verdict: SH362's specific attribution ("blocked by SH285") is obsolete — the body is
  unreachable NOT because of the SH285 leaf (now crossed) but because the run aborts at
  the SetInitParams LSM-family consumer before the do-init reaches its dispatch. This is
  the SAME measured-closed LSM family (SH349/350/358/373); do NOT extend sub-call skips
  into it (unbounded whack-a-mole).

## Interpretation
Neither probe manufactures a DataModel (DM-root [0x106a68818]=0, MH_* false). Both confirm
from the crossing env that the persistence/LSM unconstructed-object family is PATH-
INDEPENDENT and swallows every Route-B ladder arm (do-init dispatch, EC marshaller,
SetInitParams) before any live-DM construction can run. This is consistent with the
standing measured verdict (SH248h/SH256): only a REAL LocalStorageManager/session ctor
owns those objects; no seed manufactures them. The map is now complete at one level deeper
for both the EC reader-gate and the 0x258b5d8 dispatch body under the env that crosses the
SH285 leaf.

## Honest
No DataModel (DM-root [0x106a68818]=0, MH_* false). Route-B live-DM structural gate
UNCHANGED. SH174 capture-latch stays the single forward hook. recon-v3 deliverables
re-verified green. Do NOT re-drive LSM sub-call skips (SH349/350/358/373 closure stands);
do NOT re-attack the EC reader-gate (SH356/SH374); do NOT re-attack 0x258b5d8 expecting
SH285 to be the blocker (SH362/SH375).

## Files
- runs/capture_sh374_ec_dmfn_reaching.sh, runs/sh374-ec-dmfn-reaching.txt
- runs/capture_sh375_dispatch_body_reaching.sh, runs/sh375-dispatch-body-reaching.txt