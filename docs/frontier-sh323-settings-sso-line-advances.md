# Frontier SH323 — cross the SH322 NEXT fenceposts (settings SSO string + second lifecycle copy),
# advancing the engine-settings-init line deeper (STATUS candidate (a), continued)

Status: `dev`, single-agent (cone suppressed). Hermetic: `sh323` (jit.rs unit, 1 passed) +
`sh322` (extended to the second lifecycle-notify copy) + `sh323_settings_sso_fencepost_pinned`
(elfjit real-image guard). Workspace green; elfjit examples 148/0; arm64jit lib 408/0;
elfjit.rs 48 B under the 1MB hook (condensed SH-prose, facts preserved). Route-B live-DM gate
UNCHANGED (DM-root 0, MH_* false). SH174 latch single forward hook.

## 1. Problem

SH322 crossed the SH273 lifecycle wall (fn 0x21f3748) on the SH320/321 MAIN-path, but the
engine-settings-init body then faulted at the NEXT fencepost guestpc=0x1021f5078: fn 0x21f5078
(a whitespace-check) reads global std::string [0x106ed7a18] (`adrp x8,6ed7000; ldr x8,[x8,#2584]`,
ldrb [x8] faults fault=0x0 when [0x106ed7a18]==0). That cell sits 8 B directly BELOW the SH248d
cookie-jar globals [0x106ed7a20]/[0x106ed7a28].

## 2. Guard (default-inert, JIT_ROUTEB_SETTINGS_SSO_SEED=1)

`routeb_settings_sso_seed_guard` seeds a NULL cookie/string global with `routeb_empty_sso_string()`
(a valid empty SSO libc++ string) at two pcs:
- CELL_A [0x106ed7a18] at pc=0x1021f5078 (the whitespace-check fn entry).
- CELL_B [0x106ed7a28] at pc=0x1025f36ac (a) — after CELL_A clears, StartAppWithParams path
  `bl 0x221364c` @0x25f3708 (which returns x0=[0x106ed7a28]) then `ldrb [x0]` @0x1025f370c.
  The bl executes in an EARLIER block than the ldrb, so the seed must fire at the confirmed
  block entry 0x1025f36ac (not at the fault pc 0x1025f370c).

Additionally SH322's `routeb_lifecycle_wall_earlyret_guard` now fires at BOTH lifecycle-notify
copies: 0x1021f3748 (SH273 path) and 0x1021f4538 (a SECOND identical copy on the StartAppWithParams
line: sub sp,#0x90, ldr [x1], ldrb [x8,#80], tbnz->early canary-check+ret @0x21f4638).

## 3. Measured (real libroblox.so, runs/ab_sh323_settings_sso_seed.sh; both arms stack
## SH320 DONEPATH_MAIN + SH322 LIFECYCLE_EARLYRET)

- BASELINE (SH320+322 only): EXIT 134, first SIGSEGV guestpc=0x1021f5078 (the SH322 terminal).
- FORWARD (+SETTINGS_SSO_SEED): seed323=2 (both cells fire), the 0x1021f5078 wall is GONE and the
  terminal advances TWO more fenceposts: 0x1025f370c (cleared by CELL_B) -> 0x102256510
  (fault=0x0). 0x102256510 is a JNI-receive-style fn (sub sp,#0x60) whose prologue reads [x0] with
  x0=0 (a live-object/callback class, not a seedable string). FIVE consecutive crossings on the
  same engine-settings-init line: 0x1021f3748 -> 0x1021f5078 -> 0x1025f370c -> 0x102256510.

## 4. Honest

Three net-new crosses this milestone (0x1021f5078 + 0x1025f370c + the second lifecycle copy),
advancing the SH320/321 MAIN-path engine-settings-init line deeper into StartAppWithParams. Does
NOT manufacture a DataModel (DM-root 0, MH_* false); the new terminal 0x102256510 is the
live-object class (a JNI-receive fn entered with NULL this — the same SESSION-CTOR cave). Route-B
live-DM structural gate UNCHANGED; SH174 capture-latch stays single forward hook.

## 5. Verify

- `cargo test -p arm64jit --lib sh322` + `--lib sh323` = 2 passed (env/pc-gated; both lifecycle
  copies seed the [x1] pair; both SSO cells seed when NULL; live refs untouched).
- `cargo test -p arm64jit --example elfjit -- sh323` = 1 passed (real-image pins: fn 0x21f5078
  prologue 0x1021f5078=0xa9bf7bfd, adrp 0x1021f5080=0xd0026708, ldr 0x1021f5084=0xf9450d08,
  ldrb 0x1021f5088=0x3940010a).
- `cargo test --workspace` EXIT 0 (arm64jit lib 408/0, elfjit examples 148/0); `cargo build` EXIT 0.
- Repro: `runs/ab_sh323_settings_sso_seed.sh` (bounds 0x1021f5078 baseline vs 0x102256510 forward).

Single-agent, default-inert.