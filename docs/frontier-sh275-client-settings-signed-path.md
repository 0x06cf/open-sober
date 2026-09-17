# SH275 — drive the Signed client-settings receive on the REAL readLocalFlags path

Sep 17, 2026 · hermes-worker · single-agent (cone suppressed) · opt-in `--v2boot-session-signed` (default-inert) · workspace green

## Why this is a genuinely-new lever on the SEP-17 SESSION-CTOR line

The SEP-17 directive names initEngine_'s "*** Engine settings is null" hard-assert (SH184) as a real
session primitive, and the client-settings feed as the missing input. SH264 drove ONLY the plain
`nativeInitClientSettings` (0x22265fc) at version word `[0x10683cff8]` = 0 — which takes the
**empty-early** branch (the decode at 0x2bb075c `cmp w10,#0x6` / ccmp → readLocalFlags path requires
low byte == 6 && byte1 == 3). The delivery variants the REAL Android client actually uses —
`nativeInitClientSettingsSigned` (0x2bb070c), `Cached` (0x2bb0a34), `CachedCompressed` (0x2bb0c5c) —
were never driven, and the readLocalFlags parse path was never exercised. That converts SH264's
'pure consumer' judgment into a 'judged, not measured' residual on the exact SEP-17 named feed.

## What landed

1. New opt-in session rung `--v2boot-session-signed` (elfjit.rs, default-inert): sets the REAL version
   word `[0x10683cff8] = 0x0306`, then drives `nativeInitClientSettingsSigned` (0x102bb070c) as a real
   guest `jit_run` on the single ladder thread (SH55/64 serialized discipline), reusing
   `boot_sp`/`tpidr`/`env_ptr`/`thiz` + 4 fabricatable CLEAR-XML jstrings (SH186 identity shim resolves
   them via the jstring→RBX-string helper 0x21e1fec). Reports the parse result word (w0) + MH_* state
   after, and dumps `[0x106829ea8]`.
2. Hermetic `sh275_client_settings_signed_version_gate_pinned` (real-image guard, skip-if-absent):
   byte-pins the Signed entry prologue (`0x102bb070c`=0xd102c3ff, `+4`=0xa9077bfd), the version-gate
   decode (`0x102bb074c`=0x9001e468 adrp 683c000, `0x102bb0750`=0xf947fd08 ldr [x8,#4088]→[0x10683cff8],
   `0x102bb075c`=0x7100195f cmp w10,#0x6), and 4-aligned/in-window for Signed/Cached/CachedCompressed.

Repro: `runs/capture_sh275_session_signed.sh` (SH269 full seed set + `--v2boot-session-signed`).

## Honest framing

Client-settings is a **consumer**, NOT a DataModel ctor — no forward hook expects a milestone from it.
The value is cause-not-symptom: it converts SH264's 'judged consumer' into a measured drive of the
REAL readLocalFlags parse path the app uses (byte-pinned version gate), so a future public-version
client-settings payload is known to land on the right branch. Route-B live-DM structural gate is
unaffected by design (SH174 capture-latch stays the single forward hook).

## Verify

`cargo test -p arm64jit --example elfjit sh275` = 1 passed with real-image pins; workspace green
(`cargo build --workspace` + `cargo test --workspace` EXIT 0). recon-v3 render plane re-verified green
at this HEAD (`runs/capture_taskv4_frame.sh`: 24 task-driven frames, present #19..#23 swap Ok(0x1),
195 pops, 0 json abort, EXIT 124).