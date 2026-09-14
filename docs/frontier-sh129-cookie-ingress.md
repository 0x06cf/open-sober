# SH129 — Login-cookie ingress: ABI mapped, execution blocked by jar-init wall

## Goal
Prove the "remember sign-in" delivery contract headlessly: deliver a
`.ROBLESECURITY` login cookie into the engine's native HTTP cookie-jar via the
real exporter's pure-native body, then read it back through the engine's own
netscape-formatter. This is the one path toward persistence that is NOT gated on
the Lua app-shell (cookies arrive via native exports, not a self-read file —
see frontier-sh129-... recon deleg_94188fab).

## ABI (read-only disasm, deleg_0e6d339f — mapped exactly)
- `nativeSetMultipleCookies` guest `0x102202ff8` (file `0x2202ff8`) is a 4-arg JNI
  thunk `(JNIEnv* x0, jobject x1, jstring x2=URL, jstring x3=cookieList)`. It
  does NOT take a Set and does NOT iterate; its ONLY JNI calls are
  GetStringUTFChars (slot 169 / byte 1352) + ReleaseStringUTFChars (slot 170 /
  byte 1360) inside helper `0x1021e1fec`. It delegates to the PURE-NATIVE worker:
  - `0x102203148(char* cookies x0, size_t clen x1, char* url x2, size_t ulen x3,
    int a w4=0, int b w5=0)` — parses `name=value` per line, classifies each by
    name against auth constants (`.ROBLESECURITY` @ file 0xd893ab, `.RBXID` @
    0xd893c6, `.RBXIDCHECK` @ 0xd893ba), writes into the HttpCookieProtocol jar.
  - Read-back worker `0x1021ff6b0(char* url x0, size_t ulen x1, int flag w2=0,
    std::string* out x8)` fills `*out` (24-byte libc++ string) with the netscape
    cookie file — line `#HttpOnly_<domain>\tTRUE\t/\t<secure>\t0\t.ROBLESECURITY\t<token>`
    after a successful set.

So the whole ingress is driveable with two char* args (NO Java objects, NO JNI
fabrication) — a much simpler path than a Set<String>.

## Double gate chain (the worker's boot checks)
The worker 0x102203148 checks two boot latches before classifying cookies;
EITHER unset -> an else-path that null-derefs:
1. `[0x72739d4].bit0` ("flags loaded", read at 0x22031b0; the same latch
   nativeInitializeNativeFlags sets) -> else `0x2203208` -> fault `0x220321c`.
2. `[0x6dcfc30].bit0` (guest `0x106dcfc30`, read at 0x22031bc) -> SAME else
   fault `0x220321c`.
Seeding BOTH =1 (implemented in the attempt) advanced the worker past this into
real cookie classification.

## Blocked at jar-construction (structural, observed)
Even with both latches set, the worker's container path calls `0x21fce24` (a
cookie-jar container resize/getter) whose result is deref'd at `0x220331c`; it is
NULL, so the run SIGSEGVs `0x10220331c` (fault=0x0) after progressing two gates.
0x21ff8fc is container cleanup, not a getter — the jar's container is simply NOT
CONSTRUCTED by a bare JNI_OnLoad boot (it is built later during a fuller engine
init that the headless `--jni`-only path never reaches, and StartApp parks before
returning so a post-boot drive on the main thread is unavailable standalone).

This is the SAME structural init-wall class as the nativeInit gate
`0x10232090c`/StartLuaAppDM (SH115): an engine datastructure (here the cookie
jar) requires the engine's own post-boot construction, not a host seed.

## Status
- MAPPED + DOCUMENTED: full cookie-ingress ABI (thunk -> worker -> read-back),
  both boot latches, the jar-construction blocker. Reusable the moment the boot
  reaches jar init.
- The implementation (`--cookie-ingress <token>`) was written and verified to
  drive the worker + clear the two latches (worker progressed 2 gates deeper),
  but executes into the jar-init NULL. REVERTED to keep the tree green; the
  debug log is runs/sh129-cookie-attempt.txt (SIGSEGV 0x10220331c).
- Next for this line: reach a boot state where the cookie jar is constructed
  (clear nativeInit 0x10232090c / the session-advance wall), then re-drive the
  two workers — the plumbing and ABI are now pinned.

No code committed with the SH129 leaf (tree unchanged + green at SH128); commit
is this doc + notes.