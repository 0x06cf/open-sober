# SH201 — precise v2 singleton-dispatch family scanner (characterization lever)

Worker: hermes-worker · date 2026-09-16 · workspace green (561/0)

## 1. What this is

SH200 patched 4 *located* V2 singleton-dispatch sites (fn 0x6251e0c @+0x118,
0x62523ac @+0x130, 0x6258e88 @+0x2f0, 0x6258ffc @+0x2f8) and observed V2Start
completion *run-variably*. It also named the real fix — a **precise discriminator
that verifies a site really is an objB-getter dispatch before patching** — as a
"genuine future lever", and warned that a naive data-driven scan over-patches
genuine in-band `ldr x8,[x8,#N]; blr x8` calls (N<0xf0) → SIGABRT.

This cycle: (a) derives that precise discriminator as a pure, hermetic-tested
scanner over the whole image; (b) fixes a real sign-extension bug found while
building it; (c) **empirically proves the family-wide runtime patch crashes the
run** (the exact over-patch class SH200 warned about) — so the patch is
characterized + reverted, the scanner stays as the tested lever.

## 2. FINDING: the family is 384 sites, isolated by a getter-prefix gate

The objB singleton-getter (`bl 0x6249eb8`) has **463 call sites** in the image.
Of those, **365-384** follow the past-leaf dispatch shape:
`bl 0x6249eb8` (objB getter) -> `ldr x8,[x0]` -> args -> `ldr x8,[x8,#N]`
(N*8 >= 0x60, a vtable slot PAST the harness-seeded 0x60 leaf) -> `blr x8`
(0xd63f0100) → jumps into host box-alloc bytes → "outside image" soft-return.

`sh201_v2_family_scan(image) -> Vec<(ldr_x0_link, blr_link)>` gates on:
- a `bl 0x6249eb8` within the preceding 16 slots,
- a `ldr x8,[x0]` (0xf9400008) AFTER the getter,
- a `ldr x8,[x8,#N]` with N*8 >= 0x60 within the 4 slots before the `blr`.

The getter prefix + N>=0x60 is exactly the discriminator that excludes the
genuine in-band N<0xf0 calls that broke SH200's naive scan. **On the real
libroblox.so the scanner finds 384 sites** (hermetic-guarded: the test asserts
>=100 on the real image so a silent 0-scan regression is caught).

**BUG FIXED en route:** the imm26 sign-extension subtracts 2^26 (**0x400_0000**),
not 2^30 (0x4000_0000). The 2^30 constant silently broke every *backward* `bl`
(fwd branches passed the naive test, masking it) → the scan returned 0 on the
real image until corrected.

## 3. EMPIRICAL: the family-wide patch is NOT shippable (over-patch crash)

Wired `routeb_patch_v2_family` (patch every scanner-located site with the SH200
movz/movk window) behind `JIT_ROUTEB_V2FAMILY=1` and ran the ladder vs baseline:

- **Baseline** (SH200 only, no family patch), 4 runs: `SH200 patched` 4/4,
  5-6 rungs `returned Ok`, **0 SIGSEGV/SIGABRT, EXIT 124** on all 4.
- **Family patch on**: the run reaches 0x106258000 (inside the patched family
  region) and **SIGSEGVs / aborts** — the over-patch crash class SH200 predicted
  for a naive family scribble.

Verdict: the 384-site family is real and locatable, but blindly patching all of
it destabilizes the run past the SH55/64 concurrency wall. **The runtime patch is
reverted** (kept only as the characterization note in elfjit.rs); the pure
scanner + 2 hermetic tests ship as the measured lever. Do NOT re-enable
`routeb_patch_v2_family` at runtime (the crash repro is runs/sh201-family-crash.txt).

## 4. CODE (crates/arm64jit/examples/elfjit.rs)

- `sh201_v2_family_scan(&[u8]) -> Vec<(u64,u64)>` — pure scanner, no patch.
- +1 hermetic `sh201_v2_family_scan_precise_discriminator` (synthetic image:
  getter+past-0x60+blr site matches; in-band N<0x60 decoys and no-getter blrs
  do NOT — the SH200 false-positive guard).
- +1 hermetic `sh201_v2_family_scan_real_image_nonempty` (real libroblox.so
  must find >=100 sites; skips when the artifact is absent).

## 5. Reproduce

```
cd /home/hermes-worker/runs/open-sober
cargo test -p arm64jit --example elfjit sh201      # 2 hermetic tests
cargo build --example elfjit
./runs/capture_sh200_v2dispatch.sh                  # baseline ladder stable (EXIT 124, 0 crash)
```

## 6. NEXT (honest)

The family is located + characterized, but its deterministic clearing is NOT
headless-shippable (over-patch crash). Route-B live-DM world-build remains the
standing structural gate (unchanged). Next genuine lever for V2Init reaching the
SH199 world-build gate 0x102368100 would need a *runtime on-demand* patch of only
the exact site the run actually blr's through (not a pre-scan) — a larger design
(intercept at the outside-image stop with the loaded-vtable check), which SH200
correctly de-prioritized as non-Route-B-critical. Keep the do-init→app-shell→
governor continuation + live-DM line as the primary front.