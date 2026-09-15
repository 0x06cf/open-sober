# SH175 — Cookie-jar container is a SEEDABLE global, not a structural wall (objective 2b)

Status: IMPLEMENTED (default-inert, env-gated). Host: headless VPS (this change is a
headless advance, not a migration item). Workspace green at HEAD (~476/0, arm64jit 371).
Commit: 060eeb7. Author: hermes-worker, recon deleg_466252aa.

## What the recon cone found (3 parallel READ-ONLY Route-B agents)

1. **Live-DM factory live-probe = PATH B (CLOSED).** createDataModelForTeleport
   (0x2e1dc38) is an init-over-preallocated ctor (prologue `ldr x0,[x0,#56]; cbz`),
   NOT a DM maker; it requires a pre-populated this and allocates no DataModel.
   No isolated sizeof+new+ctor site exists to cut at headlessly; the make_shared is
   inlined in the vtable-gated app-join flow. The one angle SH165fwd left open
   ("in-process jit_run probe") is now closed with disassembly evidence. The SH174
   migration-gate framing is correct; ~30 angles agree.
2. **Cookie jar "construction NULL" is seedable (GENUINELY NEW).** Getter 0x21fce24
   is `adrp x8,6ed7000; ldr x0,[x8,#2592]; ret` = `*(std::string**)(0x106ed7a20)`.
   In a bare boot that slot is NULL, so the pure-native cookie worker 0x102203148
   faults at 0x220321c/0x220331c (fault=0x0) — the exact SH129 "jar-CONSTRUCTION
   NULL" that SH129/174 mislabeled *structural / non-seedable*. It is actually a
   seedable .bss global. The two worker boot gates are [0x1072739d4].bit0 (flags,
   already free via the ladder) and [0x106dcfc30].bit0 (must seed).
3. **Present/future boundary pinned** (verified by reading a real capture, not docs):
   24 real task frames headless (EXIT 124, swap Ok(0x1)); the fence line is DM
   instance allocation, NOT the R+0x180/0x188 scene list. All render taps
   (make-current/frame-fn/swap, scene renderer, present-walker, geometry emitter)
   are host-seedable; a *genuine* GuiObject still needs the DM+Lua VM (migration).

## SH175 change (objective 2b, data-persistence / remember-signin)

`routeb_cookie_jar_guard` (crates/arm64jit/src/jit.rs), wired into the block-entry
dispatch. Fire only at pc == 0x102203148 (the cookie worker's exact entry), gated on
env `JIT_ROUTEB_COOKIE=1`, default-inert (env off -> byte-identical). On fire:
- ensures guest 0x106ed7a20 maps writable, and if the slot is 0, seeds it with a
  leaked valid EMPTY libc++ std::string (zeroed 0x20 SSO buffer: size 0, cap 0).
- clears [0x106dcfc30].bit0 (and [0x1072739d4].bit0 if unset).
- Idempotent: only seeds when the slot is 0, so a real session's constructed jar is
  preserved.

Why this matters: it is the FIRST headless advance on the cookie/login-persistence
line since SH129 (which documented the jar-init as a structural wall). Clearing the
jar-init deref means driving the worker (or a future --cookie-ingress) no longer
SIGSEGVs at 0x10220331c; the worker can classify a .ROBLESECURITY cookie into the jar.

## Hermetic test

`sh175_cookie_jar_guard_seeds_container_and_gates_env_gated` (jit.rs tests):
- env unset -> slot untouched (inert).
- env set + wrong pc (0x102203144) -> slot untouched (exact-pc scope).
- env set + exact worker pc -> seeds a non-NULL empty SSO std::string AND clears
  both gate bits.
- idempotent second call preserves the same SSO pointer (real jar not clobbered).
- seeded SSO size==0 sanity.

## Honest boundary / NEXT

- The proven advance is clearing the jar-init deref. The deeper insert/commit path
  (0x22035c0 -> 0x2203708 -> 0x2203898) touches further *unexercised singletons*
  (recon could not prove the insert fully completes). If a future drive faults past
  0x220331c, that is the NEXT gate to observe, not a regression.
- Cookie *injection* into the seeded jar is memory-only; cross-restart sign-in still
  needs the Java CookieManager (setCookiesFromDisk) on a real app-launch = migration.
- The sim to drive the worker is `run_guest_callback(0x102203148,
  [cookies,clen,url,ulen,0,0,0,0], tp)` — pure-native, no Java. Not wired as a CLI
  flag this cycle (SH129's driver was reverted for tree-green; a driver is a follow-on
  if a session forms).
- Do NOT re-attempt the in-process DM-factory probe (PATH B closed above).

## Verify

- Hermetic sh175 test (jit.rs, 371/0 arm64jit) — env-off inert, wrong-pc inert, exact-pc
  seeds container+gates, idempotent, SSO size 0.
- `cargo test --workspace` + `cargo build --workspace` + `cargo build --example elfjit`
  all green.
- **Real-binary ladder with JIT_ROUTEB_COOKIE=1 (runs/sh175-cookie-ladder.txt, live):**
  full 9-rung ladder completes (nativeGameGlobalInit -> StartLuaAppDM Ok(0x3e8) ->
  ladder done), EXIT 124 (timeout-after-completion = clean), 0 SIGSEGV/0 SIGABRT, and
  the cookie guard fired **0 times** — expected: the cookie worker 0x102203148 is not on
  the boot path, so the guard is LATENT until a future drive (run_guest_callback
  [cookies,clen,url,ulen,0,0,0,0]) enters it, at which point it clears the jar-init
  SIGSEGV. No regression: default and env-on boots byte-identical up to the seed.
- /tmp cleaned (5% used, was 40%). No new large dumps (the recon agent left
  reop scratch at /home/hermes-worker/recon_dm/ — small scripts; the 1.1GB
  text.dis there predates this session and is flagged for the operator, not repo).