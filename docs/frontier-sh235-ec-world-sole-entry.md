# SH235 — EC world 0x102e24598: SOLE direct-entry pinned (9-arg marshaler 0x1023f1210) + full-.text caller proof

Session: Sep 17, 2026 (hermes-worker). Real libroblox.so (109,193,800 B, robbox copy).
+1 hermetic `sh235_ec_world_sole_direct_entry_via_9arg_marshaler_pinned` (real-image guard family
as sh232; skip-if-absent). Workspace green, examples 80/0. No production code path edited.

## Why (a genuinely-open angle, not a re-tread)

The operator's standing Route-B re-attack directive names the ExperienceController /
initializeLuaAppWithDataModel line and demands the next drive start at the CORRECT address. SH231
located the genuine DM-creation lambda world (code bodies guest [0x102e1c650, 0x102e25200); vtable
band 0x63981d8..0x6399c00) + measured it headless-UNREACHED. SH232 pinned the two in-rung `bl`s
into that world and measured the enclosing caller bodies also never translate (rungs benign-complete
upstream). What SH232 did NOT establish was the exact entry instruction of the world or the exact
caller function/frontier — it mislabeled the caller as "EC-arg helper 0x1023f11f4", which is actually
a different tiny cleanup fn. This cycle closes that with precise, full-.text-backed pins.

## Measured (fresh, authoritative)

### The world entry
Guest **0x102e24598** (`stp x29,x30,[sp,#-96]!` = 0xa9ba7bfd) inside the EC body region
[0x102e1c650, 0x102e25200). Fresh disasm: it is a big **start-app-params marshaller** — reads a live
`this` from x0 (=x19), dispatches `blr [this+0x30] -> vt+0x10`, derefs ~24 params fields on a second
object (x25=x1 at +8/+72/+145/+156/+164(float)), reads flags globals (0x6a69000+0x358, 0x6d31000+0xe28),
then `bl 0x23c5538` and `bl 0x23f1654`. So even if forced, it immediately dereferences live objects
+ network-populated flags = the fabricatable-object-graph class, NOT a static seed.

### The sole marshaler
Guest **0x1023f1210** (`sub sp,#0xb0` = 0xd102c3ff). Its ONLY `bl` (0x1023f1294=0x9428ccc1, the SH232
pin) targets 0x102e24598. It is a lean 9-arg marshaler (saves x1/x2/x3/x4/x5/x6/x7/this, builds a
stack std::string, stacks [sp]=x19/[sp+8]=x28, calls the world, then dispatches a vtable callback).
SH232's "EC-arg helper 0x1023f11f4" is a DIFFERENT fn (tiny `stp x29,x30,[sp,#-16]!` clean-up that
just calls 0x2b9e950 — it does NOT call the EC world).

### Full-.text direct-caller proof (authoritative — objdump's per-symbol grep under-reports)
The test scans every word of the executable .text for direct `bl`/`b` and asserts the exact sets:
- marshaler 0x1023f1210 callers = {0x1023f075c (StartLuaAppDM), 0x102e15bf0, 0x102e33494 (EC self-sites)}
- EC world 0x102e24598 callers = {0x1023f1294 (marshaler bl), 0x102e18408 (=0x94003064, EC self-call)}

## Verdict (do-not-re-tread)
The genuine DM-creation world 0x102e24598 is reachable headlessly ONLY through StartLuaAppDM's
marshaller call at 0x1023f075c, itself gated by StartLuaAppDM's `JNICallProtocol_receiveCall`
message-dispatch switch on live controller state. The next drive on this line starts from the exact
gate: **make StartLuaAppDM's receiveCall dispatch fall through to the `bl 0x1023f1210` at
0x1023f075c with a real ExperienceController `this` + full StartAppParams** — a live-state /
fabricatable-graph requirement, not a static seed. Route-B live-DM structural gate UNCHANGED.
It does not manufacture a DM and does not lift the gate.

## Re-verify
`cargo test -p arm64jit --example elfjit sh235_ec_world` = 1 passed (real-image guard).
`cargo test -p arm64jit --examples` = 80/0. `cargo build --workspace` EXIT 0.