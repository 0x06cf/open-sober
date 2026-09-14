# How the real Roblox Android client performs network HTTP, and the host capability to add next

Binary under test: `/home/hermes-worker/.cache/open-sober/robbox/libroblox.so` (109 MB, ARM
aarch64, NDK r28c, for Android 26, stripped; SONAME `libroblox.so`). All evidence is from
`readelf -d/-Ws` on that file, `grep -a` over its bytes, and read-only inspection of
`runs/open-sober` (`crates/arm64jit/src/{resolver,jit,fsmap,jni}.rs`, `docs/frontier-sh129-*`).

## Verdict

The client does **NOT** use JNI-to-Java HTTP. It performs login/auth HTTP with **raw native
sockets (bionic libc imports) + a natively statically-linked TLS stack (OpenSSL 3.5.0 fork)
bundled inside the `.so` itself** — option (a)+(c) combined. No `java/net`, no
`HttpURLConnection`/`OkHttp`, no `android.webkit.WebView` is ever referenced. Consequently the
JNI-HttpURLConnection-emulation path (option b) is a dead end for this client.

## Evidence

**(i) No external TLS dependency — NEEDED libs.** `readelf -d` shows only:
`libOpenMAXAL, libmediandk, libandroid, libm, libOpenSLES, libGLESv2, libEGL, liblog,
libdl, libc`. There is **no `libssl.so`/`libcrypto.so`** and no Java-framework lib. So TLS is
not imported from the platform or any bundled `.so`.

**(ii) TLS is statically linked into the guest.** `readelf -Ws`: **zero** undefined
`SSL_*`/`OPENSSL_*` symbols (count = 0). But `grep -a` over the file's string tables finds the
complete OpenSSL surface as in-binary text, i.e. guest-internal code the JIT executes natively:
- `"OpenSSL 3.5.0 8 Apr 2025\0ossl_cmp"` — the version banner string.
- Function names `SSL_connect`(7), `SSL_read`(3), `SSL_write`(9), `SSL_CTX_new`(3),
  `SSL_new`(2), `SSL_do_handshake`(1), `BIO_new_socket`, `SSL_set_fd`, `SSL_set_tlsext`,
  `libcrypto`(1), and `OpenSSL`(95).
- The full OpenSSL cipher-name tables: ~twenty `TLS_AES_`, `TLS_DHE_*_WITH_*`,
  `TLS_CHACHA`, `TLS_CIPHER`, etc.

These are defined-symbol strings from a static OpenSSL 3.5.0 (the standard Roblox "felix"
BoringSSL-adjacent / OpenSSL fork pattern), not imports.

**(iii) DNS + raw sockets are native libc imports.** `readelf -Ws` includes many file-bound
bionic symbols resolved by the JIT's import table (line numbers = dynsym index):
`socket`(173), `connect`(175), `sendmsg`(205), `recvmsg`(206), `sendto`(244), `recvfrom`(245),
`sendmmsg`(507), `recvmmsg`(474), `getaddrinfo`(172), `getnameinfo`(209), `gethostbyname`(505),
`inet_ntop`(258), `inet_pton`(259), `bind`(262), `listen`(332), `accept`(333), `accept4`(468),
plus `epoll_*`, `poll`, `select`. So DNS resolution and raw TCP/UDP all come from the guest's
libc import table — a native path, not JNI.

**(iv) JNI never touches networking classes.** `grep -a -c` for `android/webkit`, `java/net`,
`HttpURLConnection`, `OkHttp`, `javax/net`, `java/nio/channels`, `java/socket` → **0**. All
Java-class strings in the file are non-network platform interfaces:
`com/roblox/engine/jni/...`, `com/google/androidgamesdk/gametextinput/...`,
`android/app/Activity(Thread)`, `com/roblox/client/startup/NativeHelper`. These ride the
`gameActivity_*` / `NativeHelper` JNI callbacks that `jni.rs` registers set-only today.

**(v) "WebView" strings are not Android WebView.** All 121 `WebView` hits are the client's
**own C++ classes**, gated by feature flags, e.g. `StratusWebViewClientBridge`(10),
`StratusWebView`(8), `WebViewProtocolCore`/`C`, `IWebViewProtocol`,
`roblox_protocols_webview_WebViewProtocol_getFFlagWebViewH`,
`WebViewService_{EFvN...}`. No `android/webkit/WebView`/`WebViewClient` class string exists.
This is an in-engine overlay/feature abstraction, not the HTTP transport. The client also
carries its own native TCP/`quic`/`raknet`/`enet`/`resp` transport strings.

## Transport completeness on the host (READ-ONLY from repo)

The reachable socket/TLS path is already fully plumbed, so the transport itself is NOT the
gap:
- `jit.rs` forwards socket syscalls 200 `bind`, 203 `connect`, 206 `sendto`, 207 `recvfrom`,
  202 `accept`, 213 `accept4`, timerfd 85–87, epoll/poll/select to real host libc; a passing
  test (`jit.rs` ~7545) runs guest `connect→sendto("SESSDATA")→recvfrom("PONG")` to a real
  TCP peer.
- `clock_gettime`(113) and `getrandom`(278) are forwarded — the guest OpenSSL gets entropy.
- `resolver.rs` binds libc imports to host glibc (`dlopen`/`dlsym`), incl. `getaddrinfo`
  (line 3151) and `gethostbyname` (3266), each with a resolve test.
- No `/dev/urandom` fsmap needed: open/read syscalls hit real host paths directly.

So host sockets + DNS + clock + entropy + guest-bundled TLS all reach the real network today.

## Engineering recommendation

**Do not build a JNI `HttpURLConnection`/`OkHttp`/`WebView` emulation layer** — the evidence
above proves the client never calls Java networking, so that would be effort with no consumer.

The single most valuable implementable host capability is to **complete the reachable
credential/config leg of the JNI platform-interface path so the already-working native
socket+TLS stack can win a real authenticated exchange**. Concretely: in `jni.rs`, replace the
canned/no-op answers on the registered `gameActivity_*` / `NativeHelper` callbacks and the
AppBridge getters with live values — a real `getBaseURL` (api.roblox.com / login.roblox.com),
real device/network params — and implement the already fully-mapped (SH129) JNI cookie-ingress
thunk `nativeSetMultipleCookies`, which accepts `(URL, cookieList)` as two plain `jstring`s and
delegates to a pure-native worker that classifies `.ROBLESECURITY`/`.RBXID`/`.RBXIDCHECK` and
writes the engine's native cookie jar. That is implementable on the reachable boot path where
the JIT already dispatches JNI, needs no Java side at all, and is the one link that turns a
transport-complete native TLS channel into a completing login handshake against the real
endpoints. The known caveat (kept from SH129): the worker's cookie jar is constructed only in a
fuller engine init that the headless boot currently stops short of — the same structural
session/jar wall (`docs/frontier-sh131d-*`). So pair this with the highest-leverage push toward
reaching the jar-init boot state; the SH129 ingress ABI is already pinned and reusable
verbatim the moment that clears.