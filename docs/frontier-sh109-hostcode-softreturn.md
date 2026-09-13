# SH109 — V2StartAppWithParams / V2Init soft-return at host-code garbage pc (SH103 leak, non-fatal)

## Where we are
SH106 (guard seed) + SH107 (adapter record) + SH108 (render-thread stack
isolation) made the full --v2boot ladder run CLEAN: EXIT 0, 0 stack-smash, 0
SIGSEGV/SIGABRT, all rungs driven, **StartLuaAppDM returns Ok(0x...)**. The type-4
producer vector [0x106829ea8] stays 0 after every rung.

## The remaining gate: soft `run_loop: pc ... outside image` at host-code bytes
V2InitWithParams (0x102365c54), V2StartAppWithParams (0x10258b144), and the V1
AppStart__ fallback (0x102338510) each STOP (jit_run soft-return) when the guest
indirect-branches to a NON-image pc whose low bytes are HOST x86 machine code:
- 0x48038948028b4810  (48 89 48 02 8b 48 ...  = host `mov`/`add` bytes)
- 0x89480000000106d2  (48 89 ...)
- 0xa08b8b4800

These are the SH103 host-pointer-leak class: a HOST pointer (here: into the JIT's
own x86 code / a host fn) leaked into a guest register that a guest `blr x8` /
`br` then jumps to. The dispatch lands on host-code bytes, jit_run rejects pc
outside image, returns Err, and the ladder moves on (soft gate, not a crash).

Each of these three entries serializes StartAppParams/InitParams through
RBX::json or vtable dispatch; the leak is a getter/bridge returning a host fn-ptr
into a slot the guest treats as a guest call target, OR a vtable slot holding a
host address. The version-gate seed [0x10683d350]=6 (SH109) keeps V2Init/V2Start
on the clean main path (low-byte==6 branch) but did NOT remove these three leaks.

## Next
Pin the exact `blr` site in each of the three (V2Start 0x10258b144, V2Init
0x102365c54, V1 AppStart__ 0x102338510) that jumps to a host-code address. Most
likely: (a) a JNI call-object getter returning a host fn-ptr slot, or (b) an
object vtable whose slot the JIT filled with a host thunk address (0x7f0000000000
region) or that points at translated-host code. Use JIT_DUMP_PC just before the
soft-return to read the branch register, or JIT_REGION_WATCH on each entry. A
bridge sanitize (like SH97 safe_cstr_len) that refuses non-image CALL targets in
the blr position for these three functions, OR seeding the specific vtable slots
with a benign guest leaf, would let V2Start genuinely return and reach the type-4
seed install / StartLuaAppDM session construction (the next real screen plane).
Do NOT blanket-fix the bridge (could break legit PLT/guest calls); scope to the
known leak sites first.

## Repro
timeout 60 env JIT_DRIVE_LIFECYCLE=1 RENDERINIT_WARMUP_MS=5000 V2BOOT_WARMUP_MS=4500
  JIT_ROUTEB_HASHFIX=1 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so
  0x2173ff4 --jni --startapp 0x258b144 --v2boot --renderinit 0x105b3a280 --renderthunk
  --renderframe --persist-roundtrip --kicker 0x106863af8
(grep 'stopped: run_loop: pc' for the leak pcs, 'StartLuaAppDM returned Ok' for the milestone).