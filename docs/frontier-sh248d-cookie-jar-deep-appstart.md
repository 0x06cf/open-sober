# SH248d — seed the cookie-jar container globals; the continuation advances deep into nativeAppBridgeAppStart (NULL-range string-array wall)

Date: Sep 17, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green.

## Context (SH248c, this session)
SH248c got continueAfterFlagsLoaded_ to enter nativeAppBridgeAppStart (0x2338510) for the
first time headlessly. The immediate fencepost was a SIGSEGV at 0x102b504e4 (string copy,
fault=0x0) with a NULL destination. This cycle resolves it.

## What was done (one default-inert, opt-in guard)
The crash lr=0x1021f4834 named the caller: at 0x21f4824-30 inside the app-start
string-dispatch the engine does `adrp x8,6ed7000; ldr x0,[x8,#2592]; mov x0,x8; bl 2b504e4`
— assigning into the **cookie-jar container global [0x106ed7a20]** (the SH175 lane). That
.bss global is NULL headlessly -> NULL-dest string copy. The fn then (0x21f4840-48) reads a
SECOND adjacent global [0x106ed7a28] and byte-derefs it -> also NULL. New guard
`routeb_appstart_jar_seed_guard` (opt-in `JIT_ROUTEB_APPSART_JAR_SEED=1`) fires on any
block-entry pc in the enclosing fn [0x1021f47f0,0x1021f4840) and seeds BOTH
[0x106ed7a20] and [0x106ed7a28] with a leaked valid empty SSO std::string
(`routeb_empty_sso_string()`, zeroed 0x20) when NULL, idempotently.

## Measured (real libroblox.so, canonical --v2boot ladder, DMFORCE+DMCONT+SH245 env + the
## SH248c M48/APPNAME seed + JIT_ROUTEB_APPSART_JAR_SEED=1; 6-run batch)
- The two crash sites (0x102b504e4 and 0x1021f4848) are GONE. The jar seed fired 12×
  (6 runs × 2 slots). The continuation reached 6/6 runs (was ~2/6 pre-SH248c; timing
  stabilized by the extra guards).
- The continuation advances DEEPER into nativeAppBridgeAppStart before the next fault:
  `SIGSEGV guestpc=0x102339208 (lr 0x102339018) fault=0x0, x0=0x0, x1=0x0` — a string-vector
  helper (prologue 0x33901d0 computes a range's element count `sub x8,x1,x0; asr/mul`) called
  with a NULL pointer-range (x0=x1=0). This is the app-start's live session/container array —
  the SH174/SH204 live-object-lifetime wall, now reached from inside the real app-start path
  (NOT a harness-driven call).

## Honest (do-not-over-claim)
- Does NOT manufacture a DataModel; Route-B live-DM structural gate UNCHANGED; SH174
  capture-latch stays the single forward hook.
- What IS new + measured: the continuation (real continueAfterFlagsLoaded_ + the engine's own
  nativeAppBridgeAppStart) now survives the -9 bad_alloc, the app-name NULL-store, AND both
  NULL cookie-jar string writes, reaching a DEEP app-start string-array op on a NULL range.
  Four distinct headless execution boundaries advanced in this session, all from the real
  app-start path.
- The 0x339208 NULL-range wall is the live-string-array/container class: to pass it the
  engine must be handed a live session-stack object (or the array the helper ranges over
  must be non-NULL), which is the fabricatable-object-graph class SH174/SH204 maps, NOT a
  simple scalar seed.

## Next (honest, single-agent)
The NULL range (x0/x1=0 into the string-vector helper at 0x33901d0) is the app-start
session-array construction. Options: locate which container pointer is NULL there and
whether it is a seedable global vs a live-object member; if global, seed a valid empty
string-vector object; if a live member, it is the SH174/SH204 wall. Standing forward hook
unchanged.

## Code / files
- crates/arm64jit/src/jit.rs: `routeb_empty_sso_string()` (leaked 0x20 empty SSO string);
  `routeb_appstart_jar_seed_guard` (opt-in JIT_ROUTEB_APPSART_JAR_SEED=1, fires on fn range
  0x1021f47f0..0x1021f4840, seeds [0x106ed7a20]+[0x106ed7a28] when NULL, idempotent);
  wired into the block-entry dispatch after `routeb_cookie_jar_guard`. Default-inert.
- repro runs/batch_sh248d_jar_seed.sh.
- cargo build --workspace / cargo test --workspace green.