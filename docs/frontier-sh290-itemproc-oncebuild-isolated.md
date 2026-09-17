# Frontier SH290 — item-PROCESSOR once-build measured in isolation (first clean readback)

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Default-inert (opt-in `--v2boot-session-itemproc`).

## tl;dr

SH288 drove the engine's OWN never-run WORKER consume loop (0x10220778c) and its
per-item PROCESSOR 0x102207950 for the first time — but every run SIGSEGV'd at the
downstream SH273 lifecycle-notifier live-object wall (0x1021f3748 fault=0x50)
BEFORE the once-built session cell [0x106a63b00] was ever read back. SH289
corrected the once-guard cell attribution ([0x106a63b08], not 0x106863b08).
This cycle adds the missing isolated measurement: drive item-proc 0x102207950 ALONE
with a fabricated ZEROED queue-item so both indirect blr dispatches cbz-skip, letting
the engine's own ONCE-BODY complete and be read for the first time.

## Why a zeroed item isolates the once-build

item-proc (file 0x2207950, sub sp,#0xe0) after its once-guard/once-body runs the
shared per-item tail:
- `ldr x0,[x19,#32]; cbz x0,skip` — a fabricated item with [item+32]=0 skips the
  vt[+48] dispatch;
- `ldr x0,[x19,#48]; cbz x0,skip` — [item+48]=0 skips the 0x22193a0 helper.
So the ONLY state-changing path exercised is the ONCE path, which is precisely the
object SH288 could never read:

```
once-guard [0x106a63b08] (ldarb 0x2207980) != done
  -> tbz w8,#0, 0x2207a10 (once-body)
       adrp x0,6a63000 #0xb08 ; bl 0x284ce54 (guard-acquire)
       ... once-body builds via 0x2173b3c (string-map insert)
       0x2207a74: str x0,[x8],#8  ; [0x106a63b00] = result
       0x2207a7c: bl 0x284cf5c (guard-release)
  -> 0x2207988 shared tail: clock helper 0x221942c, canary, ret
```

## Measurement (real libroblox.so, runs/capture_sh290_itemproc.sh, 3/3)

Deterministic 3/3 clean readback:
- `SH290 item-proc returned Ok(...)`
- **once-guard [0x106a63b08]: 0 -> 0x101** — the `__call_once` acquire+release ran
  to completion (bit0 done-set + 0x100).
- **once-built [0x106a63b00] = 0x800000c** — the engine's own session-object cell
  now stores a real token headlessly, for the FIRST time read back.
- MH_FLAGS_LOADED=false / MH_APP_READY=false (no DM).
- Terminal after the ladder = SIGSEGV guestpc=0x101db1b08 (SH285-B LSM
  reader/pop live-object wall) — baseline parity, NOT a regression.

## Honest (do-not-over-claim)

- This is the engine's OWN worker item-processor ONCE-BODY completing headlessly:
  cause-not-symptom SESSION-CTOR progress (a real state-construction gate crossed
  + read back), closing the SH288 readback gap. It does NOT manufacture a DataModel:
  MH_* false, DM-root[0x106a68818]=0, Route-B live-DM structural gate UNCHANGED.
- 0x800000c is a small token (likely a per-process intern/type id from the
  string-map insert); it is the once-bodied session object, neither a GuiObject nor
  a DataModel. The value is evidence the once-build RAN, which is the measured claim.
- SH174 capture-latch stays the single forward hook.

## Verify

- `cargo build --workspace` + `cargo test --workspace` EXIT 0.
- `cargo test -p arm64jit --examples` = 119/0 (was 118; +sh290 hermetic).
- recon-v3 frame plane re-verified green earlier this HEAD (24 frames swap Ok(0x1),
  0 json abort, 0 crash) — default-inert change.
- elfjit.rs held under the 1MB pre-commit hook (1,048,50xB; condensed render-plane /
  SH121/SH115/SH201/SH119/SH161 prose to fit).
- +hermetic sh290 (real-image pins: once-body guard-acquire 0x2207a18=0x9419150f,
  guard-release 0x2207a7c=0x94191538, string-map insert bl 0x2207a68=0x97fdb035,
  once-built cell [0x106a63b00] guest, 4-align + in-window).

## Files

- `crates/arm64jit/examples/elfjit.rs` (+ `--v2boot-session-itemproc` rung, default-inert;
  +hermetic sh290).
- `runs/capture_sh290_itemproc.sh` (new repro).
- `runs/sh290-itemproc-{1,2,3}.txt` (gitignored captures).