# SH263 — Armed the single forward hook (SH174 capture-latch) at the DEEPEST headless app-start reach: stays LATENT

Sep 17, 2026 · hermes-worker · single-agent (cone suppressed) · no production code path edited · default-inert

## What this is

A fresh, live measurement of the **single forward hook** — the SH167/169
`routeb_dm_alloc_capture` delegating trial (armed on the engine's real
CRT operator-new wrapper, guest `0x102a0d9b8`, active-hook global
`[0x1067daaf0]`) — during the **deepest headless app-start self-drive at THIS
HEAD**. Prior cycles probed the latch only on the DMCONT ladder, never while the
SH259/260/261 unlocked app-start orchestrator walked to its deepest terminal.
This closes that gap: does any genuine-DM-signature allocation pass through the
trial at the deepest reach?

Repro: `runs/capture_sh263_dmalloc_latch.sh` (SH259 full seed set +
`JIT_ROUTEB_DM_REALCTOR=1` + `JIT_DM_ALLOC_CAPTURE=1` +
`JIT_DM_ALLOC_CAPTURE_DELEGATE=1`). Log: `runs/sh263-dmalloc-latch.txt`
(gitignored, >1MB class dumps).

## Measured (real libroblox.so, one clean bounded run)

- **Terminal unchanged + stable**: `EXIT 139` at **`guestpc=0x101db1d04`** —
  the LocalStorageManager insert-leaf wall SH260 parked (fn `0x1db1cc8`,
  map global `[0x106a6f8c0]`). **93 distinct region pcs** walked (matches
  SH260's 93) — do-init → governor → app-start self-drive at full depth.
- **Trial installed + delegated, free-path-safe**: active hook
  `0x1067daaf0` routed to the trail (`prev_hook 0x1021ebaf4`); the run reached
  the wall with NO SH167 SIGABRT — delegation through the engine's own
  allocator keeps the free-path valid on this path. The instrument itself did
  not perturb or regress the deep app-start run.
- **Trail captured only 9 logged invocations, NONE validated in-image**:
  - `call#1..#8`: all `bytes=0x18` (24 B), tag `0x100340f90`/`0x100323e89` —
    boot-time small allocs, none `[validated]` (re-confirms SH175/SH239's
    "only small 0x18 FIRST allocs, none validated").
  - `call#685`: **`bytes=0x20040` (131 KB)** at tag `a1=0x100442688` →
    base `0x7efe55e00220` **NOT validated** (its first word is not an
    in-image vtable). `a1=0x100442688` = file `0x442688`, **+7 bytes into the
    JNI `Java_com_…_nativeAppBridgeStartLuaAppDM` entry prologue** (objdump:
    the `stp x20,x19`/`add x29,sp,#0x20` at 0x442684/0x442688) — a StartLuaAppDM
    bootstrap / app-registry **buffer**, not a DataModel object (a live DM is a
    lean few-hundred/few-thousand-byte object per SH174's hardening, and would
    validate in-image).
- **Realctor worked as designed**: genuine-vptr DM `0x7efe78035250`
  (vt `0x1067162e8,0x1067163a0,0x1067163f8`, GENUINE MATCH) planted into
  current-DM holder `0x106391908` — yet app-start termination is unchanged,
  identical to SH251.

## Verdict (do-not-re-tread)

The single forward hook **stays latent at the deepest headless reach**: no real
`make_shared<DataModel>` executes, so the delegating trial sees only boot-time
small allocs + one unvalidated StartLuaAppDM bootstrap buffer. This is a fresh
armed-latch measurement at the newest-state reach (not a re-run — the latch was
never previously exercised on the SH259 app-start self-drive), and it
mechanically supports the standing Route-B conclusion: the live-DataModel
construction is a structural gate reachable only by a real session / the
documented GPU-host migration, not by any headless seed. Any in-image-vtable-
validated allocation on the trial would have been the wedge; none occurred.

Route-B live-DM gate **UNCHANGED**. recon-v3 plane **re-verified green at THIS
HEAD** (`runs/capture_taskv4_frame.sh`: 24 task-driven frames, `present #19..#23
swap Ok(0x1)`, **197 node pops**, 0 json abort, 0 crash, EXIT 124). Workspace
green (`cargo test --workspace` exit 0; arm64jit examples **99/0**).

## CODE

None shipped (pure measurement probe). Instrument default-inert
(`JIT_DM_ALLOC_CAPTURE` unset = no seed, no trailing). Tree clean vs HEAD except
the new repro script.