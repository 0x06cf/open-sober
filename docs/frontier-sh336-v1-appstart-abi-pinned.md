# Frontier SH336 — pin the REAL V1 nativeAppBridgeAppStart__ ABI (the SEP-17 named, never-independently-driven lifecycle primitive)

Date: Sep 19, 2026, hermes-worker. Single-agent (cone suppressed). Default-inert
(no production path edited; only a new hermetic byte-pin test). Workspace was green
at session start (590 passed workspace, arm64jit lib 410/0, elfjit examples 154/0).

## Why this cycle

Every cycle since SH264 has verified the SEP-17 SESSION-CTOR session rungs
(initAppShellReporter, setActive, SetInitParams, nativeInitClientSettings,
MessageBus.subscribe) and characterized the Route-B live-DM gate as UNCHANGED. This
cycle's recon confirmed (via reading elfjit.rs + the real binary) that one primitive
the operator lists is verifiably reachable ONLY via the `--v2boot` V1 fallback block,
which `--v2boot-skip-appstart` removes — so it has never independently executed as a
session rung, and its ABI was never byte-pinned. That primitive is V1
`nativeAppBridgeAppStart__` (0x102338510, Java_...NativeAppBridgeInterface_
nativeAppBridgeAppStart__Ljava_lang_String_2...ZLjava_lang_String_2...).

Also re-verified at the newest HEAD (no regression):
- recon-v3 SELF-DRIVED FRAMES deliverable: 24 task-driven frames, present #19..#23
  swap Ok(0x1), 197 node pops, 0 json abort, EXIT 124 (runs/capture_taskv4_frame.sh).
- SH307 A/B (runs/capture_sh307_preload_valuecell.sh): arm-B SendAppEventOnAppReady
  returns Ok, 0 SIGSEGV, EXIT 124; the do-init/app-shell-ctor/StartAppWithParams pipe
  executes real code (app-data-model count 0->1).
- SH335 closure reproduces (runs/probe_sh335_bus_reglive.sh): the DM-ctor name->service
  lookup sees registry count 0..12 with ONLY the task-scheduler family; "App" never an
  entry; once-slot 0->0x400000b ('Execute'), DM-root 0. EXIT 124.

## What landed (SH336, hermetic byte-pin)

`sh336_v1_appstart_abi_pinned` (elfjit.rs): pins the V1 AppStart__ ABI:
- prologue `sub sp,#0x1a0` (0x102338510 = 0xd10683ff), `add x29,sp,#0x150` (0x528),
  `mov x19,x7` (6th arg), version-gate adrp 0x6a64000 / ldr [0x10683d350]
  (0x102338550/0x338554), x0=env (mov x21,x0 @0x33859c), and the first jstring
  marshal `bl 0x21e1fec` (0x1023385a4 = 0x97faa692, the SH186 identity shim).

ABI summary (for a future drive): x0=JNIEnv*, x1=thiz, x2..x7 = 5 fabri-catable
jstring handles + jboolean Z (like the existing --startapp-v1 fallback at elfjit.rs
~7026 but through the SEP-17 rung slot). Version-gates on [0x10683d350] low-byte 6.

## Measured: V1 AppStart__ drives clean headlessly

`cargo test` already green; then drove it via the existing `--startapp-v1` path on the real
binary: EXIT 124 (stable idle), 0 SIGSEGV/SIGABRT. It posts APP_CMD_0/1/2 and wires the real
X11 window XID (0x200000 on the JIT's ANativeWindow shim) — the ABI is driveable clean. It does
NOT independently reach the do-init/app-shell-ctor render markers or manufacture a DM (no
present/#frame/log; consistent with SH184's lifecycle map: the engine builds the DM world only
via a session, never from a single AppStart entry). So: driveable-clean, session-null.

## HONEST (do-not-over-claim)

- SH336 is a byte-pin + ABI record, NOT a session advance. It does NOT manufacture a
  DataModel (DM-root [0x106a68818]=0, MH_* stay false); Route-B live-DM structural gate
  UNCHANGED. Driving V1 AppStart__ is not attempted this cycle (the V1 path walks the
  same LSM/app-start live-object wall that --v2boot-skip-appstart exists to bypass).
- The value is (a) locking the one genuinely-unpinned SEP-17 primitive ABI so a future
  session drive can hit it, and (b) reconfirming at HEAD that all standing closures
  (recon-v3, SH307, SH335-reglive) still reproduce (no regression since SH335).

## Verify

- `cargo test -p arm64jit --example elfjit -- sh336_v1_appstart` = 1 passed.
- `cargo build --workspace` EXIT 0; `cargo test --workspace` EXIT 0.
- elfjit.rs = 1048566 B < 1048576 (1MB pre-commit hook; prose condensed from earlier
  SH blocks to hold it, facts/addresses preserved).
- Single-agent, default-inert, no production path altered.