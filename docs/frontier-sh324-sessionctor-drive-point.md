# Frontier SH324 — answer STATUS candidate #1: the terminal 0x102256510 IS a genuine
# SESSION-CTOR drive point, reached from the REAL StartLuaAppDM body; pinned + proved
# the fabricated-object seed is a dead-end on this wall (x8 out-param ABI)

Status: `dev`, single-agent (cone suppressed). Hermetic: `sh324_startluaappdm_this40_sessionctor_drive_point`
(elfjit real-image guard, 1 passed; 8 anchors). elfjit examples 149/0 (148 + sh324);
arm64jit lib 408/0. Workspace green (588/0). elfjit.rs ~0.4 KB under the 1MB hook.
Route-B live-DM gate UNCHANGED (DM-root 0, MH_* false). SH174 latch single forward hook.

## 1. Problem / why this cycle

STATUS listed the SH323 terminal (guestpc=0x102256510, fault=0x0) as next-forward
candidate #1: "determine its caller contract — if the `this` is a real object a real
session would pass, this is the SESSION-CTOR drive point; a benign early-ret may or may
not apply (differs from the SH273 pair-seed pattern)." The operator's SEP-17 SESSION-CTOR
directive makes this the primary lever. This cycle answers the question decisively on the
real libroblox.so and removes guesswork for the next drive.

## 2. Measured / derived (real libroblox.so, JIT_DUMP_REGION + disasm)

1. Determinism confirmed: the SH323 forward arm (SH320 MAIN + SH322 + SH323 seed set) stops
   at guestpc=0x102256510 on 3/3 clean runs (EXIT 139/134/134; first SIGSEGV identical).
   Not run-variable — a stable wall.
2. The wall is inside the REAL `nativeAppBridgeStartLuaAppDM` body (symbol entry 0x1023efe2c;
   the block 0x23f00f8 = StartLuaAppDM+0x2cc does `sub sp,#0x70` = a nested/tail frame of the
   same function, reached by fall-through after `bl 2b9ec9c`). We are executing real
   StartLuaAppDM code — further than the SH156 "benign soft-return" terminal.
3. Fn 0x2256510 (the faulting fn) is a MEMBER method invoked as `this->vt[+32]()`:
   - prologue saves x8 (out-param) to x20, x1 to x21, x0(this) to x19
   - `ldr x9,[x0]; ldr x9,[x9,#32]; blr x9` @0x2256548/4c/50 = dispatch this->vt[+32]
   - `ldr x0,[sp,#8]; mov x1,x21; bl 222a9fc` (real settings/header work), then
   - `... bl 0x1d9d8b0` (LocalStorageManager_initStorageManagerNative) in the tail.
   this=[container+40]==NULL -> `ldr x9,[x0]` fault=0x0.
4. BOTH call sites read the SAME field: 0x23f012c `ldr x9,[x1,#40]` (call1) and
   0x23f01a0 `ldr x0,[x21,#40]` (call2, x21=the fn's saved x1=arg container). So the
   `this` for fn 0x2256510 is the +0x40 sub-member of the `container` object that
   StartLuaAppDM received as its x1 argument. That member is NULL headlessly because the
   upstream session ctor that builds the container (and its +0x40 sub-object) never runs.
5. THIS IS NOT benign-early-ret-able by the SH322 pattern AND NOT crossable by a
   fabricated-object seed: after `blr vt[+32]` the fn does `ldr x0,[sp,#8]` — it CONSUMES
   an out-param written by the dispatched method through the x8 indirect-result register.
   An all-ret1 host leaf (SH299 routeb_ec_arg0_vt_dispatch_obj) receives only x0-x7; it
   cannot write the [sp,#8] (x8) out-slot, so the post-dispatch `ldr x0,[sp,#8]` reads stale
   stack -> drift/fault at 222a9fc. A fabricated vtable therefore cannot satisfy the
   contract. The only cross is a REAL vt[+32] implementation on a REAL session object =
   the container's +0x40 member that the upstream ctor builds.

## 3. Conclusion (why this measured result matters)

The terminal 0x102256510 is confirmed to be the SESSION-CTOR drive point, reached after SIX
consecutive crossings of the MAIN-path engine-settings-init line from SH320 (0x102206df4 ->
0x1021f3748 -> 0x1021f5078 -> 0x1025f370c -> 0x102256510, +2nd lifecycle copy 0x1021f4538).
It is a proof-of-dead-end for the fabricated-object seeding approach ON THIS WALL (the x8
out-param ABI makes fabricated-this unsatisfiable), exactly the "specific gate provably
unprocessable by this JIT's seeds" form the operator's migration directive asks to establish
as a PROOF (not an ROI judgement). The remaining route is the operator's SEP-17 primary lever:
drive the REAL Android Activity/AppBridge session ctor so the container (and its +0x40
sub-object) is constructed for real — do-init/DMCONT continuation or the Activity-session
lifecycle drive — rather than seeding it.

## 4. Verify

- `cargo test -p arm64jit --example elfjit -- sh324` = 1 passed (8 real-image anchors: fn
  0x2256510 prologue `sub sp,#0x60`=0xd10183ff + dispatch `ldr x9,[x0]`=0xf9400009 +
  `blr vt[+32]`=0xd63f0120; call1 `ldr x9,[x1,#40]`=0xf9401429 + `bl 0x2256510`=0x97f998f5;
  call2 `ldr x0,[x21,#40]`=0xf94016a0 + `bl 0x2256510`=0x97f998d8; post-dispatch
  `ldr x0,[sp,#8]`=0xf94007e0 + `bl 0x1d9d8b0`=0x97ed1caa).
- `cargo test --workspace` EXIT 0; elfjit examples 149/0; arm64jit lib 408/0.
- Repro/probe: `runs/probe_sh324_terminal_regs.sh` (JIT_DUMP_REGION dump at 0x102256510 ->
  this=x0=0, x21=container ptr, lr=0x1023f0140).

Single-agent, default-inert. No production seed added (the finding is that seeding is a
dead-end HERE); next work = real session ctor drive per SEP-17.