# SH98 — marshal stat/fstat/lstat into the bionic-aarch64 struct stat (clears the guest stack-canary abort)

## Result (wall advance: the `*** stack smashing detected ***` abort is CLEARED; the ladder advances one more gate deeper into nativeGameGlobalInit)

The post-SH97 native fault was a **guest stack-canary abort**, not a string
pointer (`*** stack smashing detected ***: terminated`, `__stack_chk_fail` ->
`__fortify_fail`, guestpc=0x0, host-code abort, gdb-pinned: guest statbuf was at
`sp+0x18`, i.e. a stack-local `struct stat`). Root cause (gdb CpuState dump +
JIT_TRACE hostcall sequence `stat -> strlen -> memmove -> pthread_mutex_* ->
__stack_chk_fail`):

- The deep gameGlobalInit walk calls **`stat(path, &struct stat)` with a guest
  **stack-local bionic `struct stat`**.
- `stat` was **NOT shimmed** -> bound to raw host glibc `stat`, which writes the
  **x86-64 `struct stat` (144 bytes)** into the guest pointer.
- bionic's aarch64 `struct stat` is **128 bytes** -> glibc overruns by **16 bytes**,
  clobbering the guest frame's `__stack_chk_guard` canary stored adjacent to the
  buffer -> the guest's own canary-check `bl __stack_chk_fail` aborts.
- This is the classic bionic-vs-glibc struct-size ABI mismatch (the same class as
  the earlier pthread_mutex_t 44-vs-40 fix).

## Fix

New `bionic_stat`/`bionic_fstat`/`bionic_lstat` host shims (shims.rs, registered
`(b"stat\0", ...)`, `(b"fstat\0", ...)`, `(b"lstat\0", ...)` — `register_named`
takes precedence over the generic `resolve()` dlsym). Each: run the real
host `libc::stat` into a HOST-side `libc::stat`, then marshal the fields into the
**bionic aarch64 128-byte struct stat layout** in the guest buffer:

```
0x00 st_dev u64, 0x08 st_ino u64, 0x10 st_mode u32, 0x14 st_nlink u32,
0x18 st_uid u32, 0x1c st_gid u32, 0x20 st_rdev u64, 0x28 st_size i64,
0x30 st_blksize i32, 0x38 st_blocks i64, 0x40 st_atime i64, 0x48 st_mtime i64,
0x50 st_ctime i64, 0x58..0x80 reserved zeroed
```

Writes are hard-capped at 0x80 bytes (never overruns). Missing/error paths return
the real -1/errno so existence probes behave (ENOENT).

## Verified

- Hermetic regression `bionic_stat_marshals_into_128_byte_aarch64_layout`: real file
  round-trip; bytes beyond 0x80 (a canary sentinel) stay untouched; `st_mode@0x10`
  has S_IFREG (0o100000); `st_size@0x28` == payload length.
- Real libroblox.so `--v2boot`: **`stack smashing detected` count = 0** (was the
  deterministically-aborting site on every run). The ladder now faults ONE gate
  deeper at **guestpc 0x10220847c** (a null-deref, `fault=0x0`, string-build with
  `x2=8`) — the precise next site the SH97 doc predicted. Workspace **518/0**
  (+1 regression). Product path unregressed (exit 124 baseline intact).

## Repro

`runs/capture_v2boot_sh82.sh`. Expect NO `stack smashing detected`; expect the
ladder to fault at guestpc 0x10220847c instead.

## Next (ranked)

1. The 0x10220847c null-deref — a deeper string/pointer build inside
   nativeGameGlobalInit reading a null field (guest x6=0x1029d5f60, x2=8 len).
   Guard/seed the source field so the garbage/null never forms.
2. Goal: gameGlobalInit RETURNS -> rung 2 nativeUpdateAdapterInit (0x10221c3ec) ->
   rungs 2-6 install type-4 vector [0x106829ea8].
3. Wire NativeHelper callbacks -> StartLuaAppDM -> Lua GuiObjects -> login/home.
Standing structural wall (real self-constructed login/home) unchanged.