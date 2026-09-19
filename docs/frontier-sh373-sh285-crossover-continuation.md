# Frontier SH373 — SH285 CROSSOVER from the SH371 reaching-env: the append sub-call skip deterministically clears the standing SH285 persistence leaf; the continuation lands in the SAME measured-closed LSM unconstructed family (0x101d9a708)

Session: Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). One new probe
runs/capture_sh373_cont_appendskip.sh + live captures runs/sh373-cont-appendskip.txt +
runs/sh373-e*.txt (gitignored). No production path edited (append/pack skips are
SH349/SH350's existing default-inert opt-ins; SH373 only combines them with the SH371
reaching env and measures). Workspace green (cargo test --workspace EXIT 0).

## The genuinely-new, reproducible measurement

For the first time, the **standing SH285 persistence-wall (guestpc=0x101db1b08) is
deterministically CROSSED from the continuation/reaching env** (5 runs, sh285=0 every
time). Origin of the combination:

- SH371 added the DM_CONT_M48_SEED + CONT_APPNAME_SEED that make the DM-creator
  continuation continueAfterFlagsLoaded_ (0x102bd1d68) RUN DEEP headlessly (it previously
  faulted at its app-name guard 0x102bd1f64 before showing deep pcs).
- SH358 had measured DMCONT + {append,pack}skips producing **0 continuation-REGION hits**
  — but that run LACKED the M+0x48/appname seeds, so the continuation was never reached in
  it. SH373 adds SH349's append-skip ON TOP of the SH371 reaching env.

MEASURED:
- continuation fires (block-entry at 0x102bd1d68 in the reaching runs);
- **SH285 leaf 0x101db1b08: 0 hits** (was the terminal of EVERY SH260/284/285/3444/348/
  SH371/SH372 run) — the append byte-copy sub-call 0x101d9a15c that SH349 RETs IS the wall,
  confirmed again from this path;
- run advances to **0x101d9a708** — SH349's pack/name-string helper, faulting on source
  pointer x19=0xff..ff (x0=x19=0xffffffffffffffff, x1=0xffff80...) = the SAME
  unconstructed-live-object family SH349/350/358 measured-closed.

The append+pack combo (SH373b) instead parks at the pool-pop write-site 0x101d9a528
(fault=0x0, x1=0x1 write-to-address-1 divergence) — a DIFFERENT arm that does not reach
the continuation, consistent with run-variable divergence (SH353 class).

## Interpretation (do-not-over-claim)

SH373 proves the SH285 "wall" is NOT a fundamental invariant — it is the append byte-copy
leaf, crossable with the known single-caller skip once the continuation is actually reached.
BUT the cross lands one fencepost later in the SAME unconstructed-family lane (0x101d9a708),
which SH349 already reached and SH350 crossed to the un-bounded LSM pool-pop family
(0x101d9a528, hundreds of call sites). This RE-STRENGTHENS, not overturns, the standing
verdict: the persistence lane is measured-returned (whack-a-mole UNBOUNDED); the genuine
session Object (a real LocalStorageManager/session ctor) is the only way past, and no seed
manufactures it (SH248h/SH256). SH373's value is map-completion: it closes the last "is
SH285 itself the invariant?" loophole by crossing it deterministically and showing the next
fencepost is the already-known family.

## Honest
No DataModel (DM-root [0x106a68818]=0, MH_* false). Route-B live-DM structural gate
UNCHANGED. SH174 capture-latch stays the single forward hook. Do NOT extend this into
pack-skip+LSM whack-a-mole (SH350 measured that unbounded); do NOT re-drive further
sub-call skips into the family (SH349/350/358 closure stands).
