# Frontier SH134 — AppBridge base URL is now the real production web root

## Status
SH134 (reachable-path hardening, direction a per SH131d). Workspace green
(534/0, elfjit 41/0). Re-verified on the real binary (combined capture EXIT 0 /
24 frames / 0 crash). Commit: SH134.

## Why (recon deleg_a26ee3a7, docs/frontier-http-transport-recon.md)
A read-only recon determined HOW the real client performs its login/auth HTTP:
**raw native sockets (bionic libc imports) + a statically-bundled OpenSSL 3.5.0**
inside the `.so` — NOT JNI-to-Java. There are zero `java/net`/`HttpURLConnection`/
`OkHttp`/`android.webkit` references, and no external TLS import (NEEDED =
libc/libm/libdl/liblog/libandroid/GLES/EGL/audio only). The transport is already
fully plumbed host-side: `jit.rs` forwards socket/connect/sendto/recvfrom/bind/
epoll and `clock_gettime`/`getrandom` (so guest OpenSSL gets entropy);
`resolver.rs` binds getaddrinfo/gethostbyname to host glibc. So host
sockets + DNS + clock + entropy + guest-TLS reach the real network today.

The one reachable gap on that path was a config value: the engine's AppBridge
`getBaseURL` (the web-request root it prefixes onto its API calls) returned
**""**. With the transport complete, that "" meant any TLS/API exchange had no
real host to target. The binary itself carries `https://www.roblox.com` as its
canonical base (strings).

## Fix
`jni.rs::auto_value_string_getter` — `getBaseURL` now returns
`b"https://www.roblox.com"` instead of `b""`. Still a readable jstring handle
(the json writer reads its byte length), just a real production base. No other
AppBridge/DeviceParams string defaults changed. Reachable-path, low-risk,
on-boot; the bytes reach the engine's json init params exactly as before, only
the base-URL field now names a real host.

## Verification
- New hermetic `sh134_base_url_and_appbridge_string_getters_resolve_to_readable_jstring`
  (jni.rs): `getBaseURL` maps to the real production base and CallObjectMethod
  returns a readable jstring of exactly that length; the other string params
  still resolve to readable zero-length jstrings (no NULL).
- Updated the pre-existing `jni_auto_value_params_getters_resolve_via_fn_table`
  match to assert the new getBaseURL length (24) — a strengthening, not a
  weakening (the change is intentional; the test now pins the real value).
- `cargo test --workspace`: 534/0 (was 533). elfjit example 41/0.
- Real libroblox.so combined capture: EXIT 0, 24 real task-driven frames, 0
  crashes, ladder done + joined cleanly.

## Note (unchanged structural wall)
Per SH131d, the engine still cannot self-construct its session/UI headlessly
(lua app-shell + Java Activity init are out of reach), so this base-URL change
is latent-but-correct: it makes the moment the client DOES reach a real HTTP/
TLS exchange target a real endpoint rather than empty. It is the correct value
for the transport-complete channel, ready for when the session wall clears.

## Next
Per the recon, do NOT build a JNI HttpURLConnection/OkHttp emulation layer (the
client never calls Java networking — no consumer). The remaining reachable
credential/config items are the SH129-mapped `nativeSetMultipleCookies`
`.ROBLESECURITY` cookie-ingress thunk (pure-native worker, two jstrings) and
live device/network params, both behind the structural session/jar-init wall.