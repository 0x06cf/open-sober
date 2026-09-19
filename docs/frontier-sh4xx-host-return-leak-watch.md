# SH4xx-next — host-RETURN leak watch: the missing producer side of the canary wall

Date: Sep 26, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green.

## What and why
SH4xx's store-watch NAMES the guest *store* sites that write a foreign HOST
pointer (0x700000000000..0x800000000000) into a guest frame canary window during
the standing full-boot wall (`*** stack smashing detected ***` at
nativeGameGlobalInit / app-shell ctor). The measured writers were the event-drain
`stp x24,x23` (0x102b9dee8 / def0), the GLES-mempool region, and — the terminal —
the LSM pool-pop lane **0x101d99e70**. But a store is the *consumer* end; the
frontier explicitly left the *producer* open: "trace which HOST call returns the
0x7f... pointer into guest x-registers."

This cycle delivers that producer side: `host_return_leak_watch`, a sibling probe
(DEFAULT-INERT, env `JIT_HOST_RETURN_WATCH=1`) wired into the integer host-call
return bridge (jit.rs, at the `s.x[0] = ret` site for every resolved libc/libm/JNI/
malloc host thunk). It logs, for every host fn whose return lands a foreign host
pointer in guest x0, the host-thunk slot, the resolved host-fn name
(`host_call_slot_name`), the pointer value, and the guest caller. Run alongside
JIT_CANARY_STORE_WATCH on one boot, the same 0x7f... value in a
`[host-return-watch]` line and a `[canary-store-watch]` line names the host
producer of the canary-smashing pointer by exact value — the SH103 bridge-sanitize
target.

## Code
- translate.rs (off-hook, 498KB < 1MiB): `host_return_leak_watch` (+ env gate +
  test override `set_host_return_watch_test` + `host_return_watch_active`), the
  same once-lazy AtomicBool pattern as `canary_store_watch`. Narrows to the same
  HOST_HI/HOST_LO leak class; de-dups consecutive identical (slot,ret) repeats.
- jit.rs (110 B under the 1MiB hook after condensing SH187 prose; addresses kept):
  one gated call `crate::translate::host_return_leak_watch(pc, ret, s.x[30])` at
  the integer host-return site. Product path byte-identical when the env is unset
  (the probe is a host-side check after the return, not a translated-block edit).
- New hermetic `host_return_leak_watch_gated_and_value_classified` (arm64jit lib
  475->476): off = no-op on any input; on = foreign host ptr return takes the log
  path (no panic), small/zero returns classified out; flag independently pinnable
  (never races the store-watch hermetics).

## Verification
`cargo build -p arm64jit --lib` OK; `cargo build --example elfjit` OK;
`cargo test -p arm64jit --lib` 476/0 (the one transient load-sensitive flake in the
first run is the documented SH343/346/357/370 family, green on rerun); full
workspace run green (see STATUS). **MEASURED on real libroblox.so** (dual-watch
runbook, EXIT 134 — standing LSM pool-pop SIGSEGV, not a canary-smash this run):
the store-watch names the same writers as SH4xx (0x102b9dee8 event drain,
GLES-mempool 0x106251778/a48/a50, 0x102b9def0; terminal store pc=0x101d99e70), and
the host-return watch rounds the forensic end-to-end by exact value:

```
[host-return-watch] slot=0x7f0000000008 hostfn=boot.mempool_calloc(x1=size) -> x0=0x7f0690014bc0 caller=0x101d96848
[canary-store-watch] pc=0x101d99e70 dst=0x5647108c1c40 val=0x7f0690014bc0 ... is_canary_slot=true
```

**The producer is named: `boot.mempool_calloc`** (host mempool alloc shim at
jit.rs:7497/7530) returns the raw host pointer 0x7f0690014bc0 into guest x0
(caller 0x101d96848, the LSM pool-pop), and the very next block of that same
function stores it at 0x101d99e70 into a window the watch classifies as a guest
frame canary slot. This is the SH103 bridge-sanitize direction's exact target —
the host allocator's return is the source of the foreign 0x7f... pointer. Top
named host producers overall: `boot.mempool_calloc` 50x + slot 0x7f0000002488
57x + unresolved slots — all host-thunk returns of 0x7f... pointers.

## Honest
NOT a DM (DM-root [0x106a68818]=0, no make_shared, MH_GAME_LOADED false). Route-B
live-DM structural gate UNCHANGED. This NAMES the host-call producer of the canary
pointer (the SH4xx frontier's exact recorded next); it does NOT fix the wall —
the run still terminates in the standing SH285 pool-pop SIGSEGV (guestpc
0x101d96868, fault=0x0, no `stack smashing` line), and under identity-mapping a
mempool host pointer is a *valid* guest pointer, so whether sanitizing/masking the
return is the right fix (vs. leaving real host allocator pointers intact) is the
follow-up. No re-treads.

## Files
docs/frontier-sh4xx-host-return-leak-watch.md + runs/capture_host_return_leak_watch.sh.