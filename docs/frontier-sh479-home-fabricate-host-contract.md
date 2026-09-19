# Frontier SH479 — pin the END-TO-END HOST half of the Route-B G2 'Home' fabricate pipeline (materialize -> GetStringUTFLength -> SSO decode -> operator-pinned event-code 4)

## Session
Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green before
(cargo test --workspace EXIT 0, arm64jit 683/0). New production code: ONE test-only
hermetic in jni.rs (off the 1MiB hooks; jit.rs/elfjit.rs/session.rs untouched; runtime
byte-identical). The recon-v3 immediate-priority deliverables were re-verified green at
HEAD this cycle (capture_taskv4_frame baseline: 24 real task-driven frames `present swap
Ok(0x1)`, 0 json abort, 0 crash) — this is a test-only change, so the runtime deliverable
is byte-identical to that capture.

## The gap closed
SH475 pinned only the PURE SSO-decode fn `routeb_appevent_sso_size_to_event_code` (feeding
it raw `b0` bytes) + the decoder's real-image bytes. It explicitly left OPEN "NOT a fix of
the fabricate path itself" — i.e. whether the HOST's fabricated "Home" jstring -- the handle
the substrate's SendAppEventOnAppReady atom (session.rs `new_string_utf_handle(b"Home")`, put
in x5 per the SH186 ABI) actually hands the engine -- genuinely materializes as a SIZE-4
"Home". SH339 MEASURED the fabricated "Home" jstring materializing as a 6-byte SSO
(b0=0x0c) -> event-code 0, NOT 4, on the deeper send-appevent route. That measured negative
was never pinned as a host regression contract: a silent drift in `str_handle` /
`jni_get_string_utf_length` that makes "Home" materialize as anything-but-4 would silently
reroute the G2 gate (the operator-pinned w19-event=0x4 'Home' path), and NO test would catch
it -- the decode pins only fire on the bytes you FEED them.

SH479 adds `home_fabricate_materializes_size4_routes_event4`, a deterministic pure-host
contract pinning the THREE-STAGE pipeline the G2 'Home' discriminator actually consumes:
1. `new_string_utf_handle(b"Home")` -> a readable guest handle whose bytes read back as
   EXACTLY "Home" (4 bytes, never "Home"+trailing garbage -- the SH339 size-6 collapse).
2. `GetStringUTFLength` (the real JNIEnv fn-table thunk, host_call_at) reads 4.
3. The engine's libc++ SSO header for that size-4 short string (b0=(4<<1)|0=0x08) fed to
   `routeb_appevent_sso_size_to_event_code` -> 4 (operator-pinned w19-event=0x4).
Plus a negative guard: SH339's measured size-6 materialization (b0=0x0c) decodes to 0
NEVER 4, so the exact regression that produced the measured defect cannot silently pass.

## Honest
NOT a live-DM step (Route-B live-DM gate UNCHANGED; DM-root [0x106a68818]=0 structural per
SH462/467), and NOT a fix of the deeper-route ABI where SH339 measured the size-6 (that
route is downstream of the structural live-DM wall; the substrate's SendAppEventOnAppReady
still soft-returns Ok(0x3e8) before the discriminator). This CLOSES the HOST half as a
tested contract: the operator's STEP-2 ABI note ("the existing new_string_utf_handle(b"Home")
suffices") is now a byte-anchored assertion -- the host fabricate half is correct and
deterministic, so any residual size-6 (if it ever recurs on a live-DM session) is provably
NOT a drift in str_handle / GetStringUTFLength / the host handle. No re-treads (SH475 pinned
the decode fn + discriminator bytes; SH186 pinned the ABI x5 placement; SH339 measured the
size-6). Pure host logic, no image/env, parallel-safe.

## Files
- crates/arm64jit/src/jni.rs (test-only hermetic; arm64jit lib 683->684/0)
- docs/frontier-sh479-home-fabricate-host-contract.md (this)
- Commit on local dev only (operator pushes).

## Verify
- `cargo test -p arm64jit --lib home_fabricate_materializes_size4_routes_event4` = 1 passed
  (materialize=exact "Home", GetStringUTFLength=4, SSO decode=4; size-6 negative stays 0).
- `cargo test --workspace` EXIT 0 (arm64jit lib 684/0).