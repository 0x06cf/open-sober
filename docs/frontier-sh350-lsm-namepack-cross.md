# Frontier SH350 — CROSS the SH349+1 terminal (0x101d9a708) with a BOUNDED single-caller skip: the completing ladder advances deep into the LSM pool-pop continuation, then terminates in the same run-variable live-object family — confirming SH349's "unbounded whack-a-mole" verdict at one deeper fencepost

## Session
Sep 19, 2026, hermes-worker. Single-agent (cone suppressed). Additions: default-inert
opt-in `JIT_ROUTEB_LSM_PACK_SKIP=1` (`routeb_patch_lsm_pack_skip`, elfjit.rs — RETs the
single-caller name/version string-pack helper 0x101d9a708) + `sh350` hermetic (real-image
pins: helper entry prologue 0xd10143ff / single caller bl 0x94000041 / benign tst 0xf276541f /
b.eq index-0 0x540002a0 / natural ret 0xd65f03c0) + runs/capture_sh350_pack_skip.sh.
No production code path edited — new patch env-gated. Workspace green (arm64jit lib 416/0;
elfjit example 157/0 + sh350). recon-v3 frame plane re-verified green (24 real task frames,
0 crash, no json abort).

## Why (genuinely new, not a re-run)
SH349 crossed the long-standing SH285 terminal (0x101db1b08) by RETing the LSM byte-copy
sub-call 0x101d9a15c, and the completeness ladder (SH344 routeB reach + SH350 measurement)
confirmed the new terminal is `SIGSEGV guestpc=0x101d9a708`. Prior cycles believed the LSM
lane was "unbounded" because `bl 0x1d9d8b0` (initStorageManagerNative) has HUNDREDS of call
sites, but the new terminal helper 0x101d9a708 has exactly ONE caller (0x101d9a604, verified
by whole-region BL scan) — so skipping it is BOUNDED. This is the first genuinely-bounded
continuation past SH349, worth one direct measurement.

## The patch
`routeb_patch_lsm_pack_skip` RETs 0x101d9a708 (`sub sp,#0x50` -> `ret`), a pure name-version
string-pack helper with no observable side effects. Its single caller (0x1d9a604) does
`mov x0,x19; mov x1,x20; bl 0x1d9a708; tst x0,#0xfffffc00; stp x0,x1,[sp,#16]; b.eq index-0`
— a RET (x0=0, x1 preserved) drives the benign index-0 table path (0x1d9a664 -> 0x1d9a680
continuation -> stack-canary pop -> 0x1d9a528 loop), never the faulty backward byte-copy that
read the caller's garbage source string (x19=0xff..ff, unconstructed LSM field). CAUSE-level
line-cross (SH117 class), NOT a live-object repair (SH248h trap). Idempotent; block-cache
dropped for the helper body + its single caller block.

## MEASURED (real libroblox.so, full SH285-B/SH343-350 ladder env + LSM_NODES + all 3 skips, 4 runs)
- **SH350 fires every run** (`[elfjit:routeb] SH350 `ret` name-pack helper @0x101d9a708
  (d10143ff->d65f03c0)`); **the OLD SH349+1 terminal 0x101d9a708 is GONE in all 4 runs** (the
  ladder no longer SIGSEGVs there).
- **The completing ladder ADVANCES past it into the LSM pool-pop continuation**: 175 pool-pop
  iterations (0x101d9a5a0/0x101d9a528, the LSM pool-pop/pop-write, SH341 lines) before
  termination — the deepest persistence-lane penetration yet measured.
- **Termination is run-variable in the SAME live-object family** (SH343/346 documented):
  `bad_function_call` (host-side std::function, SIGABRT 134), 0x101d9a528 pool-pop site,
  0x102b9dee0, 0x1021e40dc — one run each varying.
- DM-root [0x106a68818]=0, MH_* all false, AppBridgeV2[0x106a705e8]=0; Route-B live-DM gate
  UNCHANGED.

## Honest (do-not-over-claim)
- SH350 is a real, measured, BOUNDED fencepost: the SH285->SH349->SH350 terminal chain
  (0x101db1b08 -> 0x101d9a15c-crossed -> 0x101d9a708) is now crossed two levels deep.
- It does NOT manufacture a DataModel and provides NO path to app-start 0x2bd2058: the
  advance lands in the same unconstructed-live-object family, and SH349+1 already measured
  that family is UNBOUNDED (hundreds of `bl 1d9d8b0` call sites; the two LSM name-string
  builds at 0x2bd2008/0x2bd2030 are mandatory to reach app-start and there is no bypass). This
  cycle's 4-run data set confirms the family persists at every depth — one more concrete
  datum for the "persistence lane is a measured dead-end" verdict (SH349), now at 3 fenceposts
  of evidence.
- Recon-v3 self-driven frame plane unchanged (re-verified green this cycle).

## Verify
Workspace green (cargo build + cargo test --workspace exit 0; arm64jit lib 416/0; elfjit
example 157/0 incl. sh350). Repro: runs/capture_sh350_pack_skip.sh -> EXIT 134, SH350 fired,
0 hits on 0x101d9a708, LSM pool-pop depth measured, MH_* false.