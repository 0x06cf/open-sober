# SH177 — Cookie READ-BACK wiring: Route B verified executing, emission value-source pinned as env/class-backed

Status: IMPLEMENTED (opt-in `--cookie-readback` driver + `routeb_patch_cookie_readback`
gate patches, env `JIT_ROUTEB_COOKIE_READBACK`; gated off on the product path).
Host: headless VPS. Workspace green (arm64jit 372/0). Doc author: hermes-worker.

## What this is

SH175 made the jar container seedable; SH176 drove the pure-native cookie worker so the
engine **self-constructs** the jar at [0x106ed7a20]. SH177 completes the other half of the
login-persistence tract: the engine's read-back getter `nativeGetCookiesInNetscapeFormat`
(guest 0x1021ff6b0) on its jar-driven **Route B**, and whether a classified value can be
re-emitted headlessly as an RFC6265 line.

The change has two parts, empirically verified on the real libroblox.so:

1. **`routeb_patch_cookie_readback` (elfjit.rs, env `JIT_ROUTEB_COOKIE_READBACK`)**: NOPs
   two read-back-local branch gates that otherwise skip Route B:
   - Gate A @ guest 0x1021ff72c `tbnz w8,#0,0x21ff744` (encoding 0x370000c8) -> the
     getter's Main/WebLogin-store round trip. NOP forces fall-through to `bl 5fee984`
     (Route B). [region-watch verifies Route B runs]
   - Gate B @ guest 0x105fee9c4 `tbz w0,#0,0x5feec00` (encoding 0x360011e0), the per-entry
     re-check of the `1dc7428` always-zero stub — NOP forces fall-through to the emission
     loop instead of the empty epilogue. [byte-recon verified both]
   Both are read-back-local; the 17-caller stub 0x1dc7428 (GL-unsupported-message
   semantics) is never patched.
2. **`cookie_jar_write_value` (jit.rs)**: writes a classified value into the engine's
   libc++ LONG-form jar string (__cap_@[0] bit0=1 long, __size_@[8], __data_@[16]) —
   the exact layout the Route-B getter decodes (0x5fee9e0 / 2203bb8 / 22035c0 all use
   `ldrb w8,[x0]; lsr ; tst w8,#1; csel` bit0=long). +1 hermetic test.

## Empirical result (real binary, llvmpipe; 3 sub-runs exercised)

```
[elfjit:cookie-rb] SH177 gate A patched getter 0x1021ff72c (...) -> nop (force jar-driven Route B)
[elfjit:cookie-rb] SH177 gate B patched Route-B @0x105fee9c4 (...) -> nop (proceed to jar read-back)
[elfjit:cookie-readback] jar[0x106ed7a20] = 0x55... (write N-byte classified value into it)
[elfjit:cookie-readback] cookie_jar_write_value -> 0x55... (jar now holds N-byte classified token)
[elfjit:cookie-readback] config accessor ran, config@0x10683d7e8 vtable = 0x10635fa30
[elfjit:cookie-readback] Route B armed (features[+73].0=0, gates set)
[elfjit:cookie-readback] nativeGetCookiesInNetscapeFormat returned Ok(0x8000000000000002)
[elfjit:cookie-readback] OUT SSO size=0: ""
```
- 0 SIGSEGV/SIGABRT/stack-smash; EXIT 124 (timeout-after-boot); ladder independent
  (standalone driver, single top-level jit_run = SH55/64-safe).
- **JIT_REGION_WATCH on the getter [0x1021ff6a0,0x1021ff860] proves the flow reaches
  Route B** (entries at 0x1021ff6b0 → 6f4 → 720 → then out to bl 5fee984 and back at
  740 → 7ac). Gate A's NOP is doing its job.
- **JIT_REGION_WATCH on Route B [0x105fee984,0x105feebd0] proves Route B reads the jar
  and reaches the emission classifier**: entries 0x105fee984 (Route-B entry) → 9dc (calls
  jar getter 0x21fce24 + decode) → a04 → a44 → a58 → then bails at `tbz w22,#0,0x5feebdc`
  (empty epilogue). w22 = return of `bl 5feeccc` (the keep/domain classifier), = 0.
- OUT stays empty (size 0) with both jar contents tested: `.ROBLESECURITY=<tok>` (full
  cookie line) and `.roblox.com` (pure dotted domain).

## The decisive finding: the emission value-source is env/class-backed (migration-gated)

The read-back does **not** emit its value from the native jar alone. Route B's emission is
gated by the classifier 0x22035c0 (reached via 5feeccc), which validates the requested
HOST/DOMAIN relationship and, for the name/value cookies themselves, feeds through
**env/class JNI backing** (5feee7c → JNIEnv cookie reads; domain via 21ff8fc). The jar at
[0x106ed7a20] is read (proven by region-watch) but its content is lowercased and used to
*gate* the domain (classifier 0x22035c0 validates a dotted host: `cmp w9,#0x2e`('.')),
not as the emitted value source. On a real device the name/value pairs come from the Java
CookieManager; headlessly there is no env/class to serve them, so the classifier returns 0
and the out-string stays empty. This converges with SH174/SH176's recon: cross-restart
sign-in needs the Java CookieManager — a migration (GPU-host) item, not a headless seed.

Recon cone (deleg_66d4cead, deleg_b11c2a89) both ran READ-ONLY and both agree Route B
executes and reads the jar; they split on the value-source (jar vs JNI) — the empirical
region-watch + empty-OUT decisively resolves it: **jar-only gating + env/class value
source; empty headlessly.**

## Honest boundary / NEXT

- PROVEN: (a) the getter's Route B can be made to execute headlessly (gate-A/B NOPs),
  (b) the engine reads its own jar (jar getter 0x21fce24 fires on Route B), (c) the jar
  write-side is layout-correct (libc++ long form, hermetic-tested).
- NOT-YET / migration-gated: emitting a real RFC6265 `.ROBLESECURITY=<token>` line
  headlessly needs the env/class(JNI CookieManager) name/value source; the classifier
  0x22035c0's 0-return is the honest next gate and it cannot be satisfied by a jar/dot-seed
  alone. The value-persistence write path remains the SH176 feature-path (0x5fef19c +
  w4=1) for a classified value *into* the jar — itself still behind the live-DM/feature
  wall.
- The write-side (`cookie_jar_write_value`) + both gate-NOPs are CORRECT, env-gated
  (latent-but-correct): they make the read-back path reachable the instant a real session
  (migration) provides env/class cookies. This is the objective-2b equivalent of SH169's
  "latent migration readiness."

## Verify

- `cargo build --workspace` + `cargo build --example elfjit` green.
- `cargo test --workspace` green (arm64jit 372/0 incl. `sh177_cookie_jar_write_value_layouts_long_string`).
- Real-binary repro: `bash runs/capture_sh177_cookie_readback.sh` -> EXIT 124, Route B
  executes (region-watch), 0 crashes, OUT empty (env/class value-source, honest).
- `/tmp` cleaned.

## Addendum — JIT_LADDER_SERIALIZE gate (also landed in SH177): benign, verified non-regression

The `JIT_LADDER_SERIALIZE=1` opt-in (elfjit.rs:11140-11159) extends SH162's
main-start_app serialization to a bare --v2boot ladder WITHOUT arming WORKER_ADMISSION_GATE
(so it avoids the SH170 EXIT-139 pitfall that combining the bare ladder with
JIT_SERIALIZE_RENDER causes). Empirically confirmed this cycle:
- Cone (deleg_a21155ee task-0) verified statically: the gate only waits (bounded 300s) for
  LADDER_DONE before the main start_app jit_run; it does NOT park the ladder thread, so it
  cannot deadlock; no new untested ladder-determinism seed is exposed.
- Real-binary bare-ladder runs (5x with the gate + 3x control without): the rung-0
  `nativeInitializeNativeFlags: run_loop pc 0x21 outside image` crash occurs in BOTH,
  at identical frequency — it is the PRE-EXISTING SH55/64 ladder-thread-vs-clone-worker
  race (host x86-pointer leak through unseeded singleton vtables, SH103/SH109 class,
  documented in SH176/HANDOFF), NOT a regression from this gate. The gate serializes only
  the MAIN thread's start_app; it cannot fix the ladder thread's own concurrent-thread
  flake. Confirmed: the canonical SH130 combined run (which uses JIT_SERIALIZE_RENDER +
  WORKER_ADMISSION_GATE via the SH130 recipe) still EXITS 0 with 24 real task frames and 0
  crashes at HEAD — the product path is unregressed.
- The DM-capture migration latch (capture_sh167_dm_alloc_capture.sh) also reconfirmed at
  HEAD: EXIT clean, 0 captures (latent-correct — no live make_shared<DataModel> headlessly),
  ladder done, SendAppEventOnAppReady Ok(0x3e8), 0 crashes. Migration-readiness intact.

## Files

- `crates/arm64jit/examples/elfjit.rs` — `--cookie-readback` driver + `routeb_patch_cookie_readback` (+ `--cookie-ingress`).
- `crates/arm64jit/src/jit.rs` — `cookie_jar_write_value` (+ hermetic test).
- `runs/capture_sh177_cookie_readback.sh` — repro.
- `runs/sh177-cookie-readback.txt` — live capture.