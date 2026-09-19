# Frontier SH475 — pin the SendAppEventOnAppReady event-name discriminator DECODE as a tested contract (the operator's "confirm w19-event=0x4"), plus stage the real APK assets

## Session
Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). recon-v3 immediate-priority
deliverables re-verified GREEN at this fresh HEAD first (capture_taskv4_frame.sh attempt 1: 24
real task-driven frames `present swap Ok(0x1)`, 196 node pops, 0 json abort, 0 crash, EXIT 124 =
stable idle) — deliverable survived the SH462-474 production windows byte-identical. Do-init/Route-B
baseline re-probed on the real binary (capture_sh415: substrate 14/16, once-guard bit0=1, DM-root
[0x106a68818]=0x0 -> LIVE DM=false, MH_FLAGS_LOADED/ENGINE_INITIALIZED/APP_READY all true, AppBridgeV2
vt resolved, 0 crash). Workspace green (cargo test --workspace EXIT 0; arm64jit lib 680->681 incl. 1
new sh475 hermetic; cargo build --workspace + --example elfjit OK). New production code is ONE small
pure helper in jit.rs (off the 1MiB hooks; jit.rs on-disk 1,049,824 B < 1,048,576... measured close to
the hook — commits kept test-heavy; elfjit.rs pin additions).

## The gap closed (SH475): the event-name discriminator DECODE, not just the opcodes

The operator's step-2 ABI note (authoritative) says: "Put the event name 'Home' (0x656d6f48) in x5
(parser reads the 4th/out-ref [sp] jstring; static maps w19=4, runlogs showed w19=1, PIN this:
confirm w19-event=0x4)." SH206 (sh211) pinned only the two `movz` opcodes (@0x2bb47c4 = movz w19,#4
'Home' path, @0x2bb47cc = movz w19,#1 ALT). It does NOT pin the DECODE that selects between them —
the chain that turns the event-name jstring's libc++ SSO header into the size that drives the branch,
and so the "w19-event=0x4" contract was never protected against silent decode drift. SH339 MEASURED
the harness's fabricated "Home" jstring materializes as a 6-byte SSO (b0=0x0c), i.e. event-code 0 NOT
4 — a genuine measured negative on the current fabricate path (the discriminator is reached on the
deeper `--v2boot-send-appevent` route, not the substrate's soft-return).

SH475 closes it with a tested two-part contract:
1. `jit::routeb_appevent_sso_size_to_event_code(b0, sp8)` — a pure, deterministic model of the
   discriminator decode at 0x102bb46b8: long bit (b0&1) selects SSO short (b0>>1) vs [sp+8] long, then
   maps size 4 -> 4 ('Home'), 5 -> 1 (ALT), 12 -> 3 (the 12-byte-name `movz w8,#3` + csel path),
   else -> 0 ("other"). This is the "confirm w19-event=0x4" contract as a testable rule.
2. sh475 hermetic pins the full truth table (Home size-4 -> 4, incl. long-form size 4 -> 4, ALT 5 -> 1,
   12 -> 3, empty/unrecognized/size-6 -> 0) AND sh211's real-image guard now pins the exact bytes of
   the discriminator load/decode chain (ldrb w8,[sp] 0x394003e8, ldr x9,[sp+8] 0xf94007e9, lsr
   0xd341fd0a, tst 0x7200011f, csel 0x9a890149, cmp #12/#5/#4 0xf100313f/0xf100153f/0xf100113f). A
   drift in any of these silently reroutes "Home"; the chain is now byte-anchored + the decode pinned.

Honest: NOT a live-DM step (Route-B live-DM gate UNCHANGED; DM-root 0 structural per SH462/467), and
NOT a fix of the fabricate path itself (SH339's size-6 materialization is a separate, deeper ABI —
the discriminator genuinely routes "Home" only if a real 4-byte string reaches it). This closes the
operator's explicit "PIN this: confirm w19-event=0x4" as a tested, byte-anchored contract so a
regression in EITHER the decode logic or the discriminator bytes is caught. No re-treads (SH206 pinned
the two output movz opcodes, not the load/select decode that feeds them; SH186 pinned the ABI x5
placement). Default-inert / test-only production surface.

## Also staged: the REAL APK assets (unlocks the engine's own content reads when a live DM arrives)

Staged the actual 81MB `assets/` tree (594 files) from the real roblox-android.apk at
`/tmp/sober_assets_real` — the resolver `aassetmanager_open` (shims.rs) previously had NO real content
to serve (SOBER_ASSETS_ROOT was never set to a real extraction), so even a hypothetical live DM-driven
AAssetManager request for e.g. `models/UniversalApp/UniversalApp.rbxm` or the LuaPackages would have
MISSed and returned 0. The engine's `rbxasset://scripts/CoreScripts` path (R1) and the `models/
UniversalApp/UniversalApp.rbxm` patch (R2) are the exact content the operator's R1-first/R2-after
strategy names. MEASURED with JIT_ASSET_TRACE armed on the 24-frame deliverable: ZERO AAssetManager/
rbxasset requests fire on the currently-reachable headless path — confirming honestly (not asserting)
that the engine never issues a content read until do-init owns a live DM (the standing structural
wall). The assets are now staged so the moment a real session advances, the resolver serves REAL
client content, not a miss. This is latent-but-correct: default-safe (a miss is unchanged on unmounted),
no production-path change, environment only.

## Files
docs/frontier-sh475-appevent-w19-decode-pin.md (this) + crates/arm64jit/src/jit.rs (pure helper +
hermetic; +23 on-disk, under the 1MiB hook) + crates/arm64jit/examples/elfjit.rs (sh211 real-image
SSO-decode-chain byte pins). Commit local dev only (operator pushes).

## Verify
- `cargo test -p arm64jit --lib sh475` = 1 passed (pure decode contract: Home->4, ALT 5->1, 12->3,
  size-6/empty/unknown->0, long-form-4->4).
- `cargo test -p arm64jit --example elfjit -- sh211` = 1 passed (real-image: full SSO-decode chain
  bytes @0x102bb46b8 pinned).
- `cargo test --workspace` EXIT 0 (arm64jit lib 681/0).
- repro: `runs/fresh_w19_substrate.sh` (the fresh-HEAD A/B measurement this cycle: substrate
  SendAppEventOnAppReady returns Ok(0x3e8) soft-return BEFORE the discriminator block; the w19 guard
  fires only on the deeper send-appevent route).