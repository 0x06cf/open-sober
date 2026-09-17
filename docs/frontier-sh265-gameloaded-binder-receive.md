# SH265 — SEP-17 session-drive extension: drive the dataModel-bindings live-binder receive
# nativeAppBridgeV2SendAppEventOnGameLoaded (guest 0x102bb429c)

Sep 17, 2026 · hermes-worker · single-agent (cone suppressed) · opt-in `--v2boot-send-game-loaded` (default-inert)

## What this is

SH264's honest note names the **dataModel-bindings receive entries** as the next un-driven
candidates on the SEP-17 session-drive line. `OnAppReady` (0x102bb463c) is already driven as a
post-ladder rung; its sibling **`nativeAppBridgeV2SendAppEventOnGameLoaded` (0x102bb429c)** was
not. This cycle adds it as an opt-in post-ladder rung so a real headless execution of the
dataModel-bindings LiveObject-binder receive can be measured — the exact surface the SEP-17
'session-ctor is the primary lever' directive names.

## What landed (elfjit.rs, default-inert)

`--v2boot-send-game-loaded` post-ladder rung (after OnAppReady): drives 0x102bb429c with
env/thiz + 3 fabricated jstrings (x2/x3/x4), reusing the proven SH186 identity shim, seeds the
pipe sync-gate [0x10683d010]=-1 so the body's `bl 0x2baeeec` takes the SYNCHRONOUS do-init path
(`cmn x8,#-1` 0x2baef24 -> `tbz w20,#0` w20=0 -> `bl 0x2206c40` GlobalInit do-init), and
measures MH_FLAGS_LOADED/MH_APP_READY + dumps the rung. Hermetic `sh265_gameloaded_binder_receive_pinned`
(real-image guard family as sh264, skip-if-absent) byte-pins 8 anchors (prologue, jstring
marshal bl 0x21e1fec, vtable-base add+store, do-init pipe bl 0x2baeeec, terminal teardown ldr/blr).

## MEASURED CORRECTION (the valuable finding)

I initially assumed OnGameLoaded's event vtable guest 0x10635dfe8 was ANOTHER all-zero
`.data.rel.ro` vtable (the SH126 soft-return class solved for OnAppReady) and built a
materializer (`routeb_patch_gameloaded_appevent_vtable`) on that premise. **Falsified at the
load level**: on-disk bytes are all-zero, but `load_elf_image` applies packed-RELA RELATIVE
addends at load time (readelf -rW can't decode packed relocs — the same class SH231/233
flagged), so the LOADED slots are real in-image code:
- `[0x10635dfe8+0x20]` = 0x102bb782c = `add x0,x0,#8; b 0x26f43d0` (delete/tail leaf)
- `[0x10635dfe8+0x28]` = 0x102bb7834 = `stp x29,x30` (string-teardown)
i.e. OnGameLoaded's terminal `ldr x9,[x0]; ldr x8,[x9,x8]; blr x8` (0x102bb44a0) dispatches
**REAL teardown code**, NOT `blr 0`. The wrong-premise materializer was REMOVED (not shipped);
the rung now only seeds the sync gate. The hermetic guard pins the REAL loaded slots so a
future silent drift into the empty-SH126 class fails loudly. Corrected in-tree.

## Honest measurement (real libroblox.so, full SH259 seed set)

The app-start ladder remains the gating constraint: it self-terminates at run-variable
live-object walls (LocalStorageManager insert-leaf 0x101db1d04 OR `std::bad_function_call` in
the app-start drain, SH260/261/262-class) BEFORE the post-ladder stages run. Batch of 3 runs:
RUN2 = EXACT baseline parity (guestpc=0x101db1d04, 93 region pcs, settings-once fired) — no
regression from this change; RUN1 = the known run-variable early SIGABRT; RUN3 = same LSM wall.
In NO run did the ladder complete far enough to reach the post-ladder OnGameLoaded rung
(`game_loaded_printed=0`), so the rung itself remains latent-but-correct.

## Honest verdict (do-not-over-claim)

Adds the dataModel-bindings live-binder receive as a real, opt-in session-drive rung and
corrects the false "all-zero vtable" premise with a measured load-level finding (packed-RELA
populates OnGameLoaded's event vtable with real teardown). Does NOT manufacture a DataModel;
Route-B live-DM structural gate UNCHANGED (SH174 capture-latch stays the single forward hook).
The receive dispatches into the same do-init pipe and MH_*/AppBridgeV2 still gate on a live-DM
session. Single-agent, no default-config production path edited, default-inert. recon-v3 plane
re-verified at SH264 (24 frames, 197 pops, 0 json abort). Workspace green (100/0 examples).