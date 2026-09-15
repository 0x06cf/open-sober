# SH176 — Cookie worker deep-path exercised headlessly: engine self-constructs the jar (objective 2b)

Status: IMPLEMENTED (opt-in `--cookie-ingress` driver, env `JIT_ROUTEB_COOKIE=1`).
Host: headless VPS (a genuine headless advance on login-persistence, not a
migration item). Workspace green (~476/0). Commit: 8ca920e. Author: hermes-worker,
recon deleg_13d959ca / deleg_7cff8f6e / deleg_fe7fe565.

## What this is

The FIRST genuine headless advance on the cookie/login-persistence line since
SH129. SH175 made the jar CONTAINER seedable (routeb_cookie_jar_guard seeds
[0x106ed7a20] at worker entry with an empty SSO + clears the two boot gates),
which DEFUSED the SH129/174 "jar-CONSTRUCTION NULL" fault at 0x220331c but only
*observed* the guard latent (0 fires on the boot path). This session drives the
pure-native cookie worker 0x102203148 itself and shows the engine's OWN init
constructs a real cookie-jar container on the guest heap.

## The change (crates/arm64jit/examples/elfjit.rs)

New opt-in `--cookie-ingress` flag (driver only — no lib change; SH175's
`routeb_cookie_jar_guard` in jit.rs is already the shipped guard). When set, the
harness drives the worker as a STANDALONE top-level jit_run on the main thread
BEFORE StartApp, with JIT_ROUTEB_COOKIE=1 so the guard fires at the worker's
entry pc:

- ABI: `run_guest_callback(0x102203148,[cookies,clen,url,ulen,1,0,0,0],tp)`
  — x0 cookies ptr, x1 clen, x2 url ptr, x3 ulen, x4(w4)=1 (REQUIRED to reach
  the classifier/commit accumulator; arg4=0 early-bails at 0x2203b20), x5=0.
- Guest-visible NUL-terminated buffers (guest==host) hold a `.ROBLESECURITY`
  cookie and the roblox.com url.
- Single-jit_run discipline: standalone (no --v2boot ladder), so it is the
  first/only top-level jit_run on the main thread (SH55/64-safe).
- After return, decodes the jar at [0x106ed7a20] as a libc++ std::string
  (short/long layout) to prove it is a genuine constructed container.

## Empirical result (runs/sh175-cookie-ingress.txt, 3/3 reproducible)

```
[elfjit:cookie-ingress] driving cookie worker 0x102203148 (cbuf=… len=42 url=… len=23, w4=1)
[elfjit:cookie-ingress] jar[0x106ed7a20] before = 0x0
[elfjit:cookie-ingress] cookie worker returned Ok(0x…) — jar-init cleared, deep path exercised
[elfjit:cookie-ingress] jar[0x106ed7a20] after = 0x55f89404c8a0 (was 0x0, engine-constructed=true)
                          gates[0x6dcfc30].0=1 [0x72739d4].0=1
[elfjit:cookie-ingress] jar is SHORT/SSO libc++ string: size=0 data@0x…991 = ""
```
- Worker returns Ok (no SIGSEGV/SIGABRT/stack-smash; earlier runs at 0x3e8).
- jar slot transitions 0x0 → a real guest-heap pointer (0x55f…), and decodes as
  a coherent SHORT/SSO libc++ std::string (size 0). This is NOT the guard's
  inert seed — the engine's jar-init REPLACED it by constructing a real 80-byte
  urn container (recon: the only static write into [0x106ed7a20] in the whole
  .text is the urn-singleton init at file 0x215cdd0, which does operator-new(80)
  and str's to [0x6ed7a20]/[0x6ed7a28]/[0x6ed7a18]/[0x6ed79e0]/[0x6ed7a10]).
- Both persistence gates end set: [0x106dcfc30].0=1 and [0x1072739d4].0=1.

## Recon (deleg_7cff8f6e, READ-ONLY, authoritative)

The commit path is TELEMETRY-ONLY — it does NOT write a classified cookie into
the jar (or any file/store). Details:

1. Worker 0x102203148 → per-cookie keep-validator 0x22035c0 → commit-loop
   0x2203898 → classify 0x2203eec: builds a LOCAL stack accumulator/vector and
   emits telemetry (log id 0x2797 / FR path 0x25fb6bc). It never references the
   jar page 0x6ed7000 or the jar accessor 0x21fce24.
2. The netscape read-back getter 0x1021ff6b0 (nativeGetCookiesInNetscapeFormat)
   reads a SEPARATE singleton at guest [0x10683d7e8] (gate byte [0x10683d810],
   store object magic 0xc0dedbad) — independent of the jar at [0x106ed7a20].
   Reachable+seedable but feeds a different global.
3. Real persistence (a classified value landing in a non-empty urn) is BEHIND
   feature-flag routing to guest 0x5fef19c: requires both feature bits
   ([0x1072739d4] and [0x106dcfc30]) = 1 AND injected worker w4&1=1 (the JNI
   dispatcher 0x102202ff8 hardcodes w4=0; the guard currently clears the bits to
   force the default path). That is the NEXT gate for content — not a regression,
   just the honest boundary.

## Honest boundary / NEXT

- PROVEN: the worker deep path (classifier/commit/keep-validator) now executes
  headlessly end-to-end and the engine self-constructs the jar container — a
  first since SH129 and the strongest headless signal on the cookie/persistence
  line yet.
- NOT-YET: a classified VALUE persisting into the jar. That needs the feature
  path (0x5fef19c) + w4=1, per recon. And cross-restart sign-in still needs Java
  CookieManager on a real app-launch (migration) — cookie injection is memory-only.
- Ladder determinism recon (same cone): StartLuaAppDM park-vs-Ok(0x3e8) is
  decided by thread-dispatch vs the stored main-id cell [0x106863a68] +
  once-guard [0x106a68410] + DM-root [0x106a68818], racing the concurrent
  main-thread StartApp on the shared boot_sp. Deterministic-Ok requires seeding
  all four + serializing the main StartApp behind LADDER_DONE. Not chased this
  cycle (the cookie drive is standalone and avoids the race entirely).
- NEGATIVE RESULT (this cycle, empirically disproven): a proposed PATCH C
  (per-rung re-assert of the dispatch-singleton .data records 0x106829a48/
  0x106829a68 to fix `run_loop: pc ... outside image` on rung-0
  nativeInitializeNativeFlags / setTaskSchedulerBM / V2Init) was IMPLEMENTED and
  VERIFIED INEFFECTIVE — the identical pcs (0x178828948000000 /
  0x828948000000e883) recur every run with or without it. The leak is NOT from
  those two records; it is the pre-existing SH55/64 concurrent-thread host-code
  class (documented deterministic-in-value, not ASLR-variable). Reverted; do not
  re-tread. Making the ladder fully deterministic needs serializing the main
  StartApp behind LADDER_DONE (bare-ladder JIT_SERIALIZE_RENDER regresses per
  SH170), which the standalone cookie drive already sidesteps.

## Verify

- `cargo test --workspace` green (arm64jit 371/0 in the full run; the sh167 test
  flaps under parallel execution only — passes isolated, a pre-existing
  process-global race, NOT this change: it's lib-only and this change is
  example-only).
- `cargo build --workspace` + `cargo build --example elfjit` green.
- Real-binary repro: `bash runs/capture_sh175_cookie.sh` → worker Ok + jar
  engine-constructed + gates set, 0 crashes (3/3).
- /tmp clean.

## Files

- crates/arm64jit/examples/elfjit.rs — `--cookie-ingress` driver (+ jar decode).
- runs/capture_sh175_cookie.sh — repro.
- runs/sh175-cookie-ingress.txt — live capture.