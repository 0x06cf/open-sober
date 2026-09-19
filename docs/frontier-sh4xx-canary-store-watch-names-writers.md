# SH4xx — store-watch NAMES the canary stack-smash writers (SH106 NEXT delivered)

Date: Sep 19, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green.

## What and why
The standing #1 full-boot blocker (STATUS next-forward) is the canary
`*** stack smashing detected ***` wall at nativeGameGlobalInit / app-shell ctor.
SH103/SH106 established the CLASS: a host pointer leaks into the guest frame's
canary slot. But the EXACT guest `str`/`stp` writer was never named — SH106's
documented NEXT step was a translator store-watch, which no cycle delivered.

This cycle implements it: a bounded, env-gated (**JIT_CANARY_STORE_WATCH=1**,
DEFAULT-INERT — emits nothing / adds no guest bytes when off, so the product path
is byte-identical), post-store host probe in translate.rs (`canary_store_watch`,
wired into every 64-bit integer store: `LdStrImm`, `LdStrImmWb`, `LdStrReg`,
`LdStPair`). It fires after the store (only observes, never perturbs), narrows to
the SH103 leak signature (a FOREIGN host pointer `0x7000_0000_0000..0x8000_0000_0000`
stored into a LOWER guest region), and reports the guest `pc`, `dst`, `val`,
current `x29`, the canary-slot `[x29-16]`, and whether `dst` lands in the frame's
canary window `[x29-0x60, x29]` (`is_canary_slot`).

## Result (real libroblox.so, full-boot env + 3-gate crossing, EXIT 134)
JIT_CANARY_STORE_WATCH=1 run: the watch fires and NAMES the exact guest store
sites that write foreign host pointers into canary windows on the live path:

```
[canary-store-watch] pc=0x102b9dee8 dst=0x55d4a051db50 val=0x7fd178488880
                     x29=0x55d4a051db90 canary_slot=0x55d4a051db80 is_canary_slot=true
[canary-store-watch] pc=0x102206a54 dst=0x106a684d0 val=0x7fd1757086c0 (is_canary_slot=false)
[canary-store-watch] pc=0x101d99e70 dst=0x55d4a051dc30 val=0x7fd16c014bc0 is_canary_slot=true
```

distinct writer pcs (top): `0x102b9dee8` (event-drain `stp x24,x23,[sp,#16]`
after the surface-handoff), `0x102b9def0`, `0x106251778`/`0x106251a48` (GLES
mempool thunk region), `0x101d99e70` (**the LSM pool-pop persistence lane — the
exact standing SH285-terminal family**). Combined with the earlier REGIONDUMP
tail (four foreign ptrs `0x7f91621c...` in guest locals at `0x102206ce8`), the
SH285/LSM pool-pop lane is now confirmed as carrying host pointers into guest
frames at `0x101d99e70`.

## Interpretation (honest)
This NAMES the writers with a working observed instrument — a genuine, bounded
advance on the #1 wall (SH106's NEXT, previously never built). It does NOT yet
fix the wall: `is_canary_slot=true` identifies stores whose `dst` is in a frame
window, and the LSM pool-pop site `0x101d99e70` matches the standing persistence
lane. The next forensic (future cycle) is: at the exact store `0x101d99e70`,
trace which HOST call returns the `0x7f...` pointer into guest x-registers — the
upstream host-return leak (SH103 bridge sanitize direction). Route-B live-DM gate
UNCHANGED (DM-root 0, MH_* false).

## Files / code
- translate.rs: `canary_store_watch` (host probe) + `emit_canary_store_watch`
  wired into the four 64-bit integer store sites. Off-hook file (484KB < 1MiB).
- 2 hermetics: `canary_store_watch_emit_is_inert_off_and_emits_on` (byte-inert
  when off, emits call when on, host fn observes nothing when off) +
  `canary_store_watch_never_removes_the_store`. arm64jit lib 473 -> 475/0.
- runs/capture_sh4xx_canary_store_watch.sh (repro).

## Verification
cargo build --example elfjit OK; cargo test -p arm64jit --lib 475/0 (default
env-off store bytes unchanged = product path byte-identical). recon-v3
deliverables re-verified green at HEAD (24 real task frames swap Ok(0x1), 0
json, 0 crash).