# Frontier SH299 — seed the EC arg0 +0x30 virtual-dispatch object, crossing the EC-world app-request dispatch wall

Date: Sep 18, 2026, hermes-worker. Single-agent (cone suppressed). Default-inert
(opt-in `--v2boot-session-dmfn` + `JIT_ROUTEB_DMFN_FIELDS=1` +
`JIT_ROUTEB_DMFN_REGISTER=1` + `JIT_ROUTEB_EC_ARG1=1` + **`JIT_ROUTEB_EC_ARG0VT=1`**).

## tl;dr

SH298b crossed the EC-world arg1+0x48 gate and left a `SIGSEGV fault=0x10`
(guestpc=0x102e245f4, x8=0, x19=dmthis+0x30) in the EC marshaller's app-request
build. SH299 identifies that fault as the EC world's **virtual dispatch through
`[EC arg0 + 0x30]`**: the DM-construction fn forwards arg0 (a fabricated zeroed
box) and the EC world does `0x2e2464c: ldr x8,[x19,#48]` (x19=arg0 →
x8=[arg0+0x30]=0) → `ldr x8,[x8,#16]` → `blr x8` (0x2e24650/0x2e24658). The
dispatch object slot being NULL makes `[0+0x10]` fault at fault=0x10. **Seeding
`[arg0+0x30]` with a coherent dispatch object whose vt[+16] is a benign ret-1
host leaf makes the `cbnz w0` @0x2e2465c take the continue-building branch,
and the ENTIRE DM-construction fn now returns `Ok(...)` 3/3 instead of
faulting.** Measured deterministic on real libroblox.so.

## Why the dispatch object is NULL (the mechanism)

Disassembly of the EC world entry region 0x102e24598:

```
2e245bc: mov x19, x0            ; x19 = EC arg0 = the DM-construction obj received
...
2e245f0: bl 23f1354             ; sub-app-request builder (fn 0x1023f1354)
2e245f4: mov x25, x21           ; (SH298b arg1+0x48 gate, [x25,#72] now in-bounds)
...
2e2464c: ldr x8, [x19, #48]!    ; x8 = [arg0+0x30], x19 += 0x30
2e24650: ldr x8, [x8, #16]      ; dispatch object vt[+16] (fault 0x10 when obj NULL)
2e24654: mov x0, x19
2e24658: blr x8                 ; the dispatch call
2e2465c: cbnz w0, 2e24678       ; nonzero -> continue building
```

`arg0` is the box fn 0x1023f03b4 works on (`dmthis`, all-zeroed). `[dmthis+0x30]`
= 0 → the `ldr x8,[x8,#16]` at 0x2e24650 loads `[0+0x10]` → fault=0x10. The reg
dump (`x19=0x7f..5380 = dmthis(0x..5350)+0x30`, `x8=0`, fault=0x10) and the host
`raw[]` decode (`48 8b 53 40` = `mov rdx,[rbx+0x40]` guest x8; `48 8d 52 10` =
`lea rdx,[rdx+0x10]`; `48 8b 02` = `mov rax,[rdx]`) confirm it precisely.

## SH299 fix

`routeb_ec_world_arg0_vt_guard` (jit.rs, opt-in `JIT_ROUTEB_EC_ARG0VT=1`),
wired right after `routeb_ec_world_arg1_guard` in the block-entry dispatch.
At EC-world entry block pc=0x102e24598, if `arg0 (=state.x[0])` is non-NULL and
`[arg0+0x30]` is empty, it seeds that slot with a coherent dispatch object
(`routeb_ec_arg0_vt_dispatch_obj()`: `[0]=` an all-ret1 leaf vtable, so
`vt[+16]` is a benign host leaf returning 1). The EC marshaller's `blr x8`
returns 1 → the `cbnz w0` @0x2e2465c is taken → control advances to continue
the app-request build instead of faulting at `[0+0x10]`. Idempotent,
default-inert. +1 hermetic `sh299_ec_world_arg0_vt_guard_is_env_pc_gated_and_seeds_dispatch_obj`.

## Measured (real libroblox.so, full SH298b base env + JIT_ROUTEB_EC_ARG0VT=1)

- SH298b baseline (no SH299): 3/3 `SIGSEGV fault=0x10` inside the EC marshaller
  (guestpc 0x102e245f4), DM-construction fn never returns.
- SH299: `[routeb-sh299] seeded EC arg0 +0x30 = coherent dispatch obj ... at
  pc=0x102e24598 arg0=0x...53c0` fires every run + **`SH296 dmfn returned
  Ok(0x...53f0)` 3/3** — the DM-construction fn completes through the EC world's
  app-request dispatch instead of faulting. The new post-run terminal is the
  known separate SH285-B LSM live-object wall at guestpc=0x101db1b08
  (fault=0xffff..), a persistence-detour park already documented; it is NOT in
  the EC world.

## VERDICT (do-not-re-tread)

- The one-next-unsynthesized-object loop advanced one more full gate: the EC
  world's app-request virtual dispatch now resolves through a coherent object
  and the construction fn runs to completion headlessly (was faulting 3/3).
- Route-B live-DM structural gate STILL holds (no make_shared<DataModel>; MH_*
  false, DM-root 0). The new terminal is the SH285-B LSM live-object wall — the
  SH174/SH204 fabricatable-object-graph class, one system away from the EC
  world.
- Cause-not-symptom SESSION-CTOR: another live-object class (a dispatch object
  at the EC arg0's +0x30) is now manufactured and the EC trajectory advanced.
  SH174 capture-latch (arm *(0x106391908) at a real make_shared<DataModel>)
  stays the single forward hook.

## Verify

- `cargo test -p arm64jit --lib -- sh299 sh298` = 2 passed.
- `cargo test --workspace` EXIT 0 (576 tests, 0 failed across all crates;
  25 `test result: ok` groups).
- `cargo test -p arm64jit --example elfjit` = 127 passed (incl. sh298, all
  sh115/29x family).
- Repro `runs/sh299-ecarg0vt-r{1,2,3}.txt`; runner `/tmp/run_sh299.sh`.
- elfjit.rs + HANDOFF prose trimmed (pre-existing) to hold the 1MB pre-commit hook.

## NEXT GATE (for the next frontier session)

With SH299, the EC world's dispatch-object gate is crossed and the DM-construction
fn returns Ok. The standing terminal has moved to the SH285-B LSM live-object wall
(guestpc=0x101db1b08), which is the SEP-17 persistence detour. Per the operator's
ROUTE-B-RETURN directive, the next Route-B step beyond SH299 is to keep driving
the DM-construction / EC world so that a REAL engine-authored session object is
built there (not the harness's fabricated arg0). Concretely the next unsynthesized
object on the EC line is whatever the EC world's post-dispatch app-request build
reads next after 0x2e24678 (the `[x21+...]`/FMOD/AAudio touches are benign; the
governor/`[arg0+0x30]` dispatch set is now coherent). A fresh disasm pass on
[0x2e24678, 0x2e25200) is the cleanest next-forward move, plus re-measuring
whether the EC append path (0x2e24744 `ldr x8,[x19]; ldr x8,[x8,#48]; blr x8`)
needs another coherent sub-object. Keep single-drive discipline (stacking with
the app-start drive suppresses it). Do-not-re-tread broad static seeds per
SH174/204/248h/256 doctrine; keep grinding the DM-construction line inside this
JIT (MIGRATION-IS-NOT-A-STOPPING-POINT).