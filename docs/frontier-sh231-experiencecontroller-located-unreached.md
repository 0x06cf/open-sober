# Frontier SH231 — REAL DataModel-creation site LOCATED via fresh packed-RELA decode + measured headless-UNREACHED

Status: single-agent Route-B re-attack, NEW recon. +1 hermetic `sh231` (elfjit example, 77/0).
frontier: SH178's explicit open directive — "the genuine DataModel allocation site was NEVER
properly located — it is in a region not yet reached by any cone."

## Why this matters

SH178 (and SH163) established that the addresses ~30 prior cones chased as "the only
make_shared<DataModel> inline" (file 0x2206d74 / 0x2173b3c, the GlobalInit do-init tail and a
string-intern GetOrCreate) were MISATTRIBUTED. SH178's correction: the real DM is created by
`ExperienceController::createDataModelForTeleport` / `submitStartGameTask` / AppBridge
`initializeLuaAppWithDataModel`, but at that time "the genuine DataModel allocation site was
NEVER properly located." This cycle LOCATES it.

## What was done (fresh packed-RELA decode, the loader's own path)

1. Read the real libroblox.so's RTTI mangled names (`strings`) for the DM-creation lambdas:
   `NSt6__ndk110__function6__funcIZN3RBX23UgcExperienceController26createDataModelForTeleport...`
   — these identify the std::function `__func` closure types created inside
   `createDataModelForTeleport` and `submitStartGameTask`.
2. Used the loader's OWN decoder (`libloader::android_relocs::read_elf_relocations`, the exact
   load_elf_image path per SH216) to scan the ~568K R_AARCH64_RELATIVE relocations for:
   - sites (r_offset) in the `.data.rel.ro` vtable band **0x63981d8..0x6399c00** whose ADDEND
     lands in the createDataModelForTeleport/submitStartGameTask **RTTI rodata band
     [0x6dc000,0x6e2000)** — these are std::function `__func` typeinfo-name-string slots
     (relocation+session-populated, i.e. all-zero on disk, filled at load).
   - CODE (exec-segment) addends referenced from the same band — the **lambda bodies**.
3. The __func vtable rows there reference genuine code bodies at **guest
   [0x102e1c650, 0x102e25200)** — e.g. row 0x63985e8's code slots 0x102e1ddc8/0x102e1dde8
   (createDataModelForTeleport lambda bodies), row 0x63988b8's manager/invoke addends
   (0x1db2cf0 shared __clone stub, 0x21e96f8 invoke helper) — the REAL DM-construction machine.

## What was measured (real libroblox.so, canonical completing ladder)

Region-watch `JIT_REGION_WATCH=0x102e1c650-0x102e25200,0x102e9fa80-0x102ea3b40` on the canonical
--v2boot completing ladder with the SH210 env (JIT_DRIVE_LIFECYCLE / DM_SEED / HASHFIX /
JSON_ZERO_FIX / SETFIX / SH115_SINGLETON_PATCH / V2_ONDEMAND):

```
run c6: EXIT 124, ladder done (1 SendAppEvent), governor-tail control FIRES
          (0x102e9fa84 .. 0x102ea30dc full walk), EC body region [0x102e1c650,0x102e25200) = 0 hits
run c7: EXIT 124, ladder done (7 markers), governor control FIRES, EC body region = 0 hits
```

**2/2 clean completing runs** (c6/c7, manual, verified SendAppEvent) plus the repro batch (2/3
govtail-control-positive): the governor-tail control (a real headless-executing world region,
SH197/204/209) fires, but the freshly-located ExperienceController DM-creation body region
**never executes** headlessly — **0 EC-world hits across 4/5 calibrated completing runs**
(EXIT 124, ladder done). This is a FRESH, correct-location reachability negative: the
genuine site is real but unreached — the same Route-B live-DM structural gate (SH178 law-level:
the site is `__func`-closure + relocation-populated and only constructible inside a real
experience/app-launch session).

## Caller-side strengthening (independent confirmation)

The region also has **241 direct `bl`/`b` callers** into [0x102e1c650,0x102e25200) found by a
static imm26-branch scan — including from the do-init/app-shell region chain that IS on the
executing ladder (0x1023cfd68/0x1023d0360/0x1023d09d0 -> `bl 0x102e24468`; 0x1023f1294 ->
`bl 0x102e24598`). Yet the region-watch measured **0 hits** across 4/5 govtail-positive
completing runs: because a reached caller would translate its `bl` target as a fresh block and
log a region hit, 0 hits means **none of those callers executes on the completing ladder**
(they live in deeper engine boot/migration bodies the ladder never reaches). So the region is
NOT merely vtable-latent — it has direct static callers — but they too are unreached headlessly.
This converts "unreachable" from an assumed-vtable-latency conclusion into a doubly-measured
negative (caller-side AND region-side).

## Honest boundary (do-not-over-claim)

Does NOT manufacture a DataModel and does NOT lift the Route-B live-DM structural gate. It CLOSES
SH178's "never properly located" with a measurement: the genuine site (bodies 0x102e1c650..
0x102e25200, __func vtable band 0x63981d8..0x6399c00) is now LOCATED, pinned, and measured
headless-unreached. The next drive on this line starts from the correct address (a real session's
createDataModelForTeleport/submitStartGameTask executing the lambda world), not the refuted
do-init addresses.

## Code

- `crates/arm64jit/examples/elfjit.rs` hermetic `sh231_experiencecontroller_dm_creation_world_located_and_pinned`
  (real-image guard family as sh227/sh228/sh229b; skips clean if the .so is absent):
  byte-pins the EC body-region entry 0x102e1c650 (0xa9bf7bfd), mid-body 0x102e20398 (0xa9be7bfd),
  upper-window 0x102e25148 (0xd102c3ff), region bounds + 4-alignment, AND verifies via the
  loader's OWN relocation decode that the vtable band [0x6398000,0x639a000) carries >=4 RELATIVE
  relocs whose addend lands in the createDataModelForTeleport RTTI rodata band [0x6dc000,0x6e2000).
  So a future drift in the located site fails loudly instead of silently re-reading the wrong region.
- Throwaway recon tools (dmfind/dmreloc) removed after deriving the pins.
- Repro: `runs/capture_sh231_experiencecontroller_reach.sh`.

## Verify

- `cargo test -p arm64jit --example elfjit sh231` -> 1 passed (real-image anchors on libroblox.so).
- Full `cargo test --workspace` green (re-verified at HEAD before this cycle; elfjit example 76->77/0).
- recon-v3 render plane + SH210 latent wiring re-verified green at HEAD earlier this session.

## Next (honest, single-agent)

Route-B live-DM = structural gate UNCHANGED. The genuine DM-creation site is now located + pinned;
it is unreached headlessly and constructible only in a real session. SH174 capture-latch arming at
a real make_shared stays the single forward hook. recon-v3 immediate-priority deliverables stay
shipped + verified.