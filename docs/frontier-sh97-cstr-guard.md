# SH97 — guard guest C-string pointers in strlen / formatted-output host shims

## Result (wall advance: the strlen-garbage SIGSEGV at guestpc=0x0 is CLEARED; the ladder advances past the formatted-string site deeper into gameGlobalInit)

Read-only recon deleg_190c3f48 root-caused the post-SH96 native fault (guestpc=0x0,
`fault=0xffffff80ffffffc8`, `fault==rdi==rbp==rdx`, raw[] = glibc AVX2 `strlen` body —
`and eax,0xfff; cmp eax,0xfe0; vpcmpeqb; vpmovmskb; tzcnt`): the deep gameGlobalInit
walk formats a log string via a `%s%s%s` literal (guest 0x1025aeb1a) and passes a
**garbage C-string pointer** (0xffffff80ffffffc8, loaded from an unseeded map field) to
a host string fn. The JIT bound `strlen`/`__strlen_chk` to raw glibc strlen with NO
mapped-region check; glibc strlen derefs the unmapped pointer -> SIGSEGV. The guest
also reaches the same garbage pointer through `vsnprintf`/`sprintf` which INTERNALLY
call glibc strlen, so a strlen-only guard is insufficient.

**Fix:**
- `jit.rs safe_cstr_len(ptr)`: refuse the garbage class — 0, sub-image `< 0x100000000`,
  and non-canonical (`ptr>>48 == 0xffff`, sign-extended 48-bit tags like
  0xffffff80ffffffc8) -> return 0; otherwise call plain glibc strlen on the canonical
  pointer (guest image strings AND host-heap guest-malloc strings pass). Plus
  `image_domain_contains` helper.
- `shims.rs bionic_strlen_chk` + new `bionic_strlen`: route through `safe_cstr_len`
  instead of raw glibc strlen; registered `(b"strlen\0", bionic_strlen)` in
  `register_shims` (register_named takes precedence over the generic dlsym binding).
- `shims.rs bionic_vsnprintf`: route the guest's `vsnprintf` (which glibc implements by
  internally strlen()ing `%s` args) through the existing `render_vfprintf` AAPCS64-va_list
  decoder, whose `%s` path now uses `safe_cstr_len` (renders `(bad-ptr)` for garbage,
  `(null)` for NULL) and writes the result into the guest buffer with a NUL terminator.
  Registered `(b"vsnprintf\0", bionic_vsnprintf)`.

**Verified:** the strlen-garbage SIGSEGV (`fault=0xffffff80ffffffc8`, guestpc=0x0) is
GONE on every run — the ladder now advances past the formatted-string site to a REAL
guest pc (0x10220847c, further into nativeGameGlobalInit) / a deeper abort, instead of
the raw-strlen SEGV. SIGSEGV count across 3 --v2boot runs dropped (mostly via the
guard; the residual is a different later site). Workspace **517/0** (+1 regression
`safe_cstr_len_rejects_nonc_canonical_garbage`). Product path unregressed (exit 124,
persist byte-exact, 0 crash; the shims are --v2boot/host-call-gated and the product
main-thread path never feeds a garbage string).

## Repro

`runs/capture_v2boot_sh82.sh`. Expect NO `fault=0xffffff80ffffffc8` raw-strlen SEGV; the
ladder faults at a deeper site (guestpc 0x10220847c or an abort) instead.

## Next (ranked)

1. The ladder now reaches guestpc 0x10220847c (deeper in nativeGameGlobalInit) / a
   host-abort on a guest worker thread (tid 0 vs tid 2) — the next unguarded
   string/pointer deref or the concurrent-thread/block-cache class. Trace the new site:
   guard sibling string ops (strncmp/strcpy/sprintf/__android_log_print) with
   safe_cstr_len as they surface, and/or seed the source map field so the garbage never
   forms.
2. Goal: gameGlobalInit RETURNS -> rung 2 nativeUpdateAdapterInit (0x10221c3ec) ->
   rungs 2-6 install type-4 vector [0x106829ea8].
3. Wire NativeHelper callbacks -> StartLuaAppDM -> Lua GuiObjects -> login/home.
Standing structural wall (real self-constructed login/home) unchanged.