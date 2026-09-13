# SH108 — layer-2 stack-smash fixed: render thread needs its own guest stack

## The layer-2 canary-slot writer, finally root-caused and fixed

SH106 fixed layer-1 (the pervasive `__stack_chk_guard` GOT slot 0x67d16f0 was
never seeded). But with `--renderinit`/`--renderthunk` active the ladder STILL
stack-smashed: the canary slot `[x29-16]` of a ladder canary function was clobbered
with a saved `stp x29,x30` frame pair (`x29+0x30` + a return addr into an object-init
routine at file 0x1d99ff0) — BEFORE any nested call in the canary function itself
(JIT_DUMP_REGION probe proved it was wrong immediately after the prologue).

## Root cause: shared guest stack across two jit_run threads

Both the `--renderinit` thread AND the `--v2boot` ladder thread started their
jit_run with the SAME guest SP: `renderinit` seeded `s3.x[31] = isp` where
`isp = st.x[31]` (line 6039), and ladder seeded `boot_sp = st.x[31]` (line 4549).
When both drive guest code concurrently, the render thread's GUEST frames grow
down into the ladder thread's LIVE frame, and a nested guest `stp x29,x30`
(object-init prologue) writes over the ladder frame's canary slot -> the
`__stack_chk_fail` "stack smashing detected" SIGABRT. This is the harness
reusing one boot stack across threads (the SH55/64 desync class, layer-2 of
SH104/105). NOT the guard slot (that was fixed in SH106) and NOT a guest self-store.

## Fix (elfjit.rs renderinit thread)

Give the `--renderinit` thread its OWN dedicated leaked 1 MiB guest stack
(guest==host identity map, so a leaked buffer is guest-addressable) instead of
reusing the boot SP:
```
const RSTACK: usize = 1 << 20;
let rstack = Box::leak(vec![0u8; RSTACK].into_boxed_slice());
s3.x[31] = rstack.as_mut_ptr() as u64 + RSTACK as u64 - 0x100;
```
Mirrors how `run_guest_callback` gives each guest re-entry its own stack. `isp`
is still passed to the render walker paths (they use it independently), so it
remains a real binding.

## Verified (real libroblox.so, full ladder + render flags)
EXIT 0, **0 stack-smash** (was 1 every render run), 0 SIGSEGV/SIGABRT, and the
ladder now drives: nativeGameGlobalInit Ok -> nativeUpdateAdapterInit Ok(0x1) ->
setTaskSchedulerBM -> V2InitWithParams -> **StartLuaAppDM Ok** -> V2StartAppWithParams
-> V1 AppStart__. StartLuaAppDM never executed before this cycle (it was behind the
rung-1 park). Workspace 518/0. Product path unregressed (exit 124, 0 crash, persist
byte-exact, present swap Ok(0x1)).

## Next
V2StartAppWithParams / V2InitWithParams soft-return at garbage pc (0x194/0x21,
an indirect blr to a non-image address) rather than hard-fault — those are the
soft gates to clear next. Type-4 vector [0x106829ea8] still 0 after each rung.