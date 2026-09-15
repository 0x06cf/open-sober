# SH169 — Route-B NEXT-3 do-init seed spec: RECON-CORRECTED (dead, wrong-address)

Author: hermes-worker (autonomous loop), Sep 15 2026.
Status: **recon (READ-ONLY), no code change** — verifies the operator's
"deleg_8d5648cf NEXT-3 do-init seeds … implement today" spec is STALE and that
implementing it would add wrong-address / anti-advance litter, not progress.
Supersedes nothing shipped (nothing shipped these 3). Reconciles with the
SH163–167 ledger: **do-init path is seed-complete; the live-DM wall is the
migration gate.**

## Method

Independent 3-agent Route-B recon cone (all READ-ONLY, no harness run,
disasm dumps kept small / cleaned):
- deleg_task-0: empirical disassembly of the real 109MB `libroblox.so` for the
  three fault sites + addresses.
- deleg_task-1: cross-checked the full do-init ctor callee chain and the current
  elfjit.rs ladder wiring (which guards are live vs dead).
- deleg_task-2: disassembled 0x1f1d8ac / the CoreScripts loader + the filesdir
  read sites to settle the R1 synthetic-CoreScript question.

Each verified by direct `aarch64-linux-gnu-objdump` on
`~/.cache/open-sober/robbox/libroblox.so` (guest = file_vaddr + 0x100000000).

## Verdict 1 — the three NEXT-3 seeds are all DEAD

### Seed #1 — thread-init singleton `[0x1067333aa0]` (spec: clears SEGV @0x102207ef0)
Fault site real: `0x2207ee8 adrp x8,7333000; 0x2207eec ldr x8,[x8,#2720]` loads
`[0x7333aa0]`; `0x2207ef0 ldr x0,[x8,#16]` would SIGSEGV if the cell were NULL.
**But the address is WRONG**: `adrp 7333000` → guest base **0x107333000**
(`0x100000000 + 0x7333000`), so the real cell is guest **0x107333aa0** — the spec's
`0x1067...` prefix is **off by 16MB** (that's a non-image address). It IS read by
ctor callee `0x2207df8` (via `0x2207edc`), and its page is **already mapped** by
elfjit.rs line 6703 (`0x107333aac`). On the current seeded EXIT-0 runs the init
body completes without fault — **satisfied, not an unblock.**

### Seed #2 — telemetry once-cell `[0x106dcd380]=-1` (spec: clears cond_wait @0x2b4cd1c)
Address is correct and genuinely on-path (callee `0x22082d8` does `ldar`/`cmn #-1`),
**but it is an ANTI-ADVANCE**: the spec premise "clears a cond_wait park at 0x2b4cd1c"
is wrong — `0x2b4cd1c` is a generic single-flight/std::call_once primitive whose
`pthread_cond_wait` parks ONLY when a second thread is in-flight (state==1).
Headless single-thread: state starts 0, callback runs immediately, never parks.
`[0x6dcd380]` is not that state cell, and the async path is bypassed anyway by
SH126's pipe sync-gate `[0x10683d010]=-1`. Setting it to `-1` makes `b.eq` SKIP the
already-clean non-allocating registrar. **Zero contribution to DM construction.**

### Seed #3 — map page `0x10673336000` + `[0x10673336d8].bit0=1` (spec: clears SEGV @0x102212838)
**Bogus address**: guest `0x10673336000 − 0x100000000 = 0x67336000` ≈ 27.7GB, far
beyond the 109MB file — not a valid image address. The real .bss thread page is
`0x107333000` (prefix 0x1073, not 0x1067), already mapped; and the once-flag at
offset 0x6d8 is NOT consumed by the do-init ctor chain (it reads 0xaa0–0xaf0).
The `0x2212838 ldarb w8,[0x73336d8]` fault premise doesn't hold — that page is
already mapped by the loader (seg2 RW `[0x62dc1c0,0x7333c3c)`); the skip only stops
a benign clock-double computation. **Dead/wrong.**

## Verdict 2 — do-init's true current stopping point

Not a `.bss` seed fault. `nativeGameGlobalInit` entry 0x2206404 → `bl 0x2206c40`
do-init → once-guard `[0x6a68410]` → main path reads `[0x6a68818]` (guest
0x106a68818, offset 0x408) and dispatches vt+0x30 at 0x2206e24. SH156 empirically
seeded `[0x106a68818]`, do-init crossed to real engine-boot (0x1023eff4c →
AppBridgeV2 governor 0x102e9fa84 → nativeAppBridgeStartAppWithParams; app-data-model
counter advances). **The concrete wall is structural**: a live `RBX::DataModel` is
created only by `ExperienceController::createDataModelForTeleport` /
`initializeLuaAppWithDataModel` → `DataModelServices::setDataModelToCurrent` during a
**real app-launch session** — the GPU-host / real-input **migration gate**. The three
NEXT-3 sites are upstream helper blocks the current ladder is already past.

## Verdict 3 — R1 synthetic CoreScript is NOT viable (restores SH163 on correct grounds)

- `rbxasset://scripts/CoreScripts` IS code-referenced (file 0x232ed4; SH163's
  "zero code refs / 0x232f34" was a wrong measurement) — but the filesdir seeded at
  guest 0x10726d600 is read at only 4 sites (2 stores: nativeSetFilesDirectory +
  crashpad; 2 reads: settings/crashpad logs). **It is NEVER joined to a
  CoreScripts Lua path.** No filesdir→loose-file CoreScripts resolver exists.
- CoreScripts content resolves through the `rbxasset` content manager (URI carried
  as std::string at 0x258850c/0x2588534/0x4345f8c, combined against runtime base
  slot `0x6c22000+0x9a0`, byte source via aasset/AssetManager + patch-model —
  `LoadCoreScriptsFromPatchOnly` file 0x53087d). The loader is **downstream of a
  live DataModel**.
- Host-seeding `files/scripts/CoreScripts/<name>.lua` will NEVER be read. Do not
  build the R1 lever.

## Reconciliation

0 of 3 NEXT-3 seeds are implementable-now unblocks (two rest on a 16MB
address-prefix error; the third suppresses an already-clean telemetry call).
Independently confirms SH163/SH166/167: no live seed remains on the do-init
continuation; the live-DM wall is the migration gate. Do NOT re-tread this line.
Keep the 3-wide Route-B cone armed but Route-B-scoped: re-derive on the
ExperienceController / live-DM synthesis / DataModelPatcher path, not do-init seeds.

## Traps recorded (footguns for any future agent)

- **Thread-page prefix**: real guest thread-init page is `0x107333000`. Trust
  `adrp x8,7333000` disasm; treat any `0x1067...` addressing a thread page as
  suspect — verify against the `adrp`. Off-by-16MB.
- Do-init ctor already-complete: SH156/163 crossed all four callees
  (0x2207d6c / 0x2207df8 / 0x22082d8 / 0x2208354); they complete cleanly.

## Standalone repro (for the record; the running ladder already reaches this)

`timeout 220 env JIT_DRIVE_LIFECYCLE=1 JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DM_SEED=1
JIT_ROUTEB_HASHFIX=1 JIT_JSON_ZERO_FIX=1 JIT_ROUTEB_SETFIX=1
JIT_SH115_SINGLETON_PATCH=1 JIT_DM_ALLOC_CAPTURE=1
./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4
--jni --startapp 0x258b144 --v2boot --v2boot-surface-handoff --v2boot-send-appevent`
(exits 0/124 clean, governor tail + SendAppEventOnAppReady Ok, no SIGSEGV/SIGABRT).