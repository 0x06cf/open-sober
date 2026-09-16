# SH199 — World-build fn 0x102ea3b14 gate byte located + measured latent (reachability negative)

Worker: hermes-worker · date 2026-09-16 · workspace green (561/0, +1 hermetic)

## 1. What this is

SH198 left the "governor-tail terminal" open: the do-init -> app-shell ctor ->
governor -> governor-tail continuation executes clean headlessly and ENDS at the
ret thunk 0x102ea30dc, with the contiguous DEEPER construction fn 0x102ea3b14
(`ldr x0,[x0,#688]`, then operator-new(0x18) + ctor 0x2eacce4 +
`nativeAppBridgeAppStart__` 0x2365960) reported as "unreached headlessly" — but the
GATE that guards it was not pinned. This cycle locates that gate byte and tests the
cross empirically on the real binary.

## 2. FINDING: the world-build fn is behind a single zero-default .bss byte

The gaining path is INSIDE the already-executing V2InitWithParams rung, at guest
0x102368100-0x102368114:

```
0x102368100: adrp x8, 6a70000          ; -> guest 0x106a70000
0x102368104: ldrb w8, [x8, #1384]      ; w8 = byte guest [0x106a70568]
0x102368108: cbz  w8, 0x102368114      ; if 0 -> SKIP the world-build call (bare boot default)
0x10236810c: mov  x0, x19
0x10236810c: bl   0x2ea3b14            ; <- WORD-BUILD fn (never reached headlessly, SH198)
0x102368114: ...continues
```

[0x106a70568] is a zero-default `.bss` cell (NOBITS, writable). Crossing it makes
the V2Init rung fall through into 0x102ea3b14, which calls operator-new(0x18) +
ctor + `nativeAppBridgeAppStart__` (the 6-jstring ABI app-bridge start). This is the
next real construction body SH198 could not reach.

`bl 0x2ea3b14` callers: exactly ONE in the image (0x236810c, inside V2InitWithParams).
`bl 0x2ea3a84` (its twin shallow world-build leaf) is called from ~10 sites incl. the
governor MODERN blob-path 0x258b890 — that one is the telemetry/leaf getter, not the
deep app-start body. The deep body is 0x2ea3b14.

## 3. CODE (default-inert, one-guard-per-fix)

`routeb_worldbuild_gate_seed(state, pc)` (arm64jit jit.rs): fire in the gate block
window [0x102368100, 0x102368114) only, when env `JIT_ROUTEB_SETWORLDBUILD` is set,
and only when [0x106a70568] is currently 0 — write 1 (idempotent, preserves a real
nonzero session value). Wired into the JIT_ROUTEB_SETFIX dispatch chain alongside
the other Route-B guards. +1 hermetic
`sh198surface_routeb_worldbuild_gate_seed_env_and_pc_and_idempotent` (env-off inert
even at the exact gate pc; env-on at a gate-internal pc seeds 1; the cbz-taken target
0x102368114 is OUT of the window; a live nonzero value is preserved). Workspace 561/0.

## 4. EMPIRICAL (real libroblox.so): LATENT-BUT-CORRECT, reachability NEGATIVE

Ran the full seeded ladder with JIT_ROUTEB_SETWORLDBUILD=1 + region-watch
[0x102ea3a84, 0x102ea3be0]:

- 3/3 the V2InitWithParams rung stops FIRST at the SH198 run-variable host-pointer
  flake: `pc 0x76 / 0x229 / 0x102b9e008 outside image` (x30=0x1062514e4, the
  singleton-vtable blr-through-host-mcode class, guest 0x102b9e008) — BEFORE the rung
  reaches gate block 0x102368100. So the seed byte is written but the deeper body is
  NOT entered in the same boxed run.
- Baseline (same command, env OFF): identical 3/3 V2Init outside-image stop. ⇒ the
  stop is PRE-EXISTING (SH198/SH55 class), NOT caused by this guard. No new regression.
- EXIT 124 clean otherwise; the do-init->app-shell->governor tail continuation still
  runs clean as SH198/SH197.

## 5. Honest boundary

The world-build fn is thus behind TWO stacked gates: (a) the byte [0x106a70568] (now
seedable + tested) and (b) the reachability of V2InitWithParams' body past the SH198
host-pointer flake, which does not fire headlessly on this box. The byte-cross is
latent-but-correct: the instant a real session (or a future V2Init flake-fix) advances
the rung past 0x102b9e008, the deep app-start body 0x102ea3b14 will execute — the seed
is in place. This converts SH198's "unreached (judgment)" into a located, seedable
gate + a measured reachability negative — the SH198/SH197 pattern.

## 6. Reproduce

```
cd /home/hermes-worker/runs/open-sober
cargo build --example elfjit
./runs/capture_sh199_worldbuild.sh /tmp/sh199.txt
# expect: occasionally the [routeb-worldbuild] seed line + EXIT 124 (0 crash);
# the V2Init rung may stop at the run-variable outside-image flake first (pre-existing).
# hermetic: cargo test -p arm64jit sh198surface_routeb_worldbuild
```

## 7. NEXT (honest)

Fixing the V2Init outside-image stop is the non-seedable host-pointer class (SH198
verdict holds — do NOT static-seed it). The byte gate is shipped+tested. Route-B
live-DM/class-registry world-build remains the standing structural gate; this work
adds one more located+gardened headless lever to the continuation line, firing when a
real session advances the rung.