# SH284 — the LAST never-driven initEngine_ state body (state=9, 0x2bd2668) now SELF-COMPLETES headlessly

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed), default-inert.

## tl;dr

SH273/SH277 pinned the initEngine_ settings-state dispatch at state word `[this+16]`
(==3 → 0x2bd1d68 serializer / ==5 → 0x2bd24b4 / ==9 → 0x2bd2668) and measured that
**states 5 and 9 were NEVER driven** — only state=3 was. SH280-283 then crossed the
state=5 body: SH281 seeded `[config+56]` (the callee drops the value), SH283's
`JIT_ROUTEB_ENG5_QMUTEX_FREE` steal carried state=5 through the GlobalInit-reentry
continuation (5→7). **This cycle drives the LAST remaining body — state=9 (0x2bd2668) —
headlessly for the FIRST time.**

## The measured mechanism

The state=9 body (0x2bd2668) is the same shape as state=5:

- sub sp,#0x80 prologue, reads version word [0x10683d8f8] (both branches)
- sets `[this+16] = 10` (state→10) at 0x2bd26e8
- then `bl 0x2bce0d4` with w2=1 (config dispatch) → `b 275a0c4` (the GlobalInit-reentry
  continuation that SH283's steal now carries through)

It reuses the exact SH279 manager seeds (app-name 'Home' long + [this+0x40] config +
[config+56] buffer) with no new fabrication.

## The gate

New opt-in `--v2boot-session-engine9` (elfjit.rs): sets `[mgr3+16]=9`, drives
`jit_run(0x102bd2668)` directly as its own jit_run on the single ladder thread
(SH55/64 serialized discipline), reusing boot_sp/tpidr + fabricated manager. Default-inert;
repro runs/capture_sh284_repro.sh.

## A/B / repro (real libroblox.so, full SH279 seeds + SH283 steal gate)

3/3 (runs/sh284-repro-{1,2,3}.txt):

```
[elfjit:v2boot] SH284 state=9 body direct returned Ok(0x0)
[elfjit:v2boot] SH284 post state=9 direct: [this+16](state)=10
```

The state=9 body COMPLETES (returns Ok + state→10) — the first headless execution of the
LAST never-driven initEngine_ state body. The run then self-drives into the app-start path
(appstart once-cell seed fires, deeper app-start reach) and terminates at the SH260 LSM wall
guestpc=0x101db1d04 — the parked SEP-15 persistence detour, NOT a regression (identical to
SH283's documented run-1 behavior).

## Honest

- Cause-not-symptom SESSION-CTOR progress: the engine's OWN settings-state machine now
  self-completes ALL THREE variants of its initEngine_ state dispatch bodies that the
  SH280-283 line can drive (state 3 → serializer, state 5 → 7, state 9 → 10). The state→10
  completion is a measured new milestone (was never reached before).
- Does NOT manufacture a DataModel (MH_* still false, DM-root[0x106a68818]=0); Route-B
  live-DM structural gate UNCHANGED; SH174 capture-latch stays the single forward hook.
- The downstream app-start death is the parked LSM persistence wall (objective-2b detour),
  not this gate.

## Verify

- cargo build --workspace EXIT 0; cargo test --workspace green; arm64jit examples 116/0
  (+1 sh284). sh284 filtered = 1 passed.
- Repro: bash runs/capture_sh284_repro.sh (3/3 state=10).

## Discipline

Default-inert (opt-in arg), single-agent, trim-compacted elfjit.rs under the 1MB pre-commit
hook (comments condensed, test pins preserved).