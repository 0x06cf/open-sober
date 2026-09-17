# SH266 — SEP-17 session-drive extension: drive the LAST genuinely-NEVER-driven Route-B
# candidate, the messageBus receive Java_com_roblox_universalapp_messagebus_MessageBus_subscribe
# (guest 0x102ba5bb8)

Sep 17, 2026 · hermes-worker · single-agent (cone suppressed) · opt-in `--v2boot-session-bus` (default-inert) · workspace green

## Why this is the one worth measuring

SH185 closed the messageBus "experience-launch" subscribe by **STATIC judgment alone** — its doc
reads "subscribe registered only inside the migration-gated initializeLuaApp_... only the gated
bootstrap produces [a DM]" and the loop has re-treaded the app-start wall 15+ times since. But SH185
never **drove** the subscribe export as a real guest entry. Its body is a JNI-RECEIVE export: it
(i) dispatches a JNI table slot near the top (`lrd x8,[x0]; ldr x8,[x8,#248]; blr x8` = a vtable/
telemetry slot on the fabricated env's function table, SH186 identity-shim family), (ii) telemetry-logs
and allocates subscription boxes via `operator_new 0x1d96768` (mksize 0x28/0x20), and (iii) **directly
bl's nativeAppBridgeAppStart 0x2343c10** (the `nativeAppBridgeAppStart__String,..,String,Z,..,Z`
marshaller) — a REAL app-start route the ladder drives via the fabricated StartLuaAppDM frame instead.
If subscribe completed, it would register a `experience-launch` subscription AND drive a genuine
app-start. This is exactly the SEP-17 "messageBus experience-launch listen" live candidate the
operator's session-drive directive names.

## What landed (elfjit.rs, default-inert)

`--v2boot-session-bus` post-ladder rung (after OnAppReady + OnGameLoaded): drives 0x102ba5bb8 with
env_ptr + fabricated thiz + 4 fabricated jstrings (`b1="experience-launch"`, b2/b3/b4 empty), on the
SAME single ladder thread (SH55/64), reusing boot_sp/tpidr — the established SH186 identity-shim
pattern proven for the other lifecycle/binder receives. Measures MH_FLAGS_LOADED/MH_APP_READY +
dumps [0x106829ea8]. Hermetic `sh266_messagebus_subscribe_jni_receive_pinned` (real-image guard,
skip-if-absent) byte-pins 6 anchors (entry 0x102ba5bb8=0xd10643ff, JNI-table dispatch
ldr/ldr/blr @0x2ba5bf4/0x2ba5bf8/0x2ba5bfc, jstring marshal bl 0x21e1fec @0x2ba5c7c=0x97d8f0dc,
subscription vtable adrp @0x2ba5cfc, **REAL app-start bl 0x2343c10 @0x2ba5e14=0x97de777f**,
operator_new 0x28 box @0x2ba5d34=0x97c7c28d) — verified passing against the real libroblox.so.

## Honest measurement (real libroblox.so, full SH259 seed set)

4 runs total (1 from the repro script + a 3-run batch): **every run self-terminates at the standing
app-start terminal `SIGSEGV guestpc=0x101db1d04` (LocalStorageManager insert-leaf, SH260) BEFORE the
post-ladder rung runs** — `driving MessageBus.subscribe` never prints in any run (grep count 0×4).
The rung is therefore **latent-but-correct**: the app-start ladder still dies at the run-variable
live-object wall before OnAppReady/OnGameLoaded/subscribe can execute, exactly the constraint SH265
measured for the OnGameLoaded rung. No regression: the 93 distinct region pcs / same terminal match
SH260/263/264/265 baseline parity.

## Honest verdict (do-not-over-claim)

This converts SH185's *static* "migration-gated / never-driven" judgment into a **wired + measured
latent-but-correct** state: the last genuinely-NEVER-driven Route-B candidate now has a real opt-in
session-drive rung + a passing real-image ABI pin, and a live measurement showing it (like OnAppReady
and OnGameLoaded before it) gates on the same standing app-start live-object wall — NOT on the
subscribe path itself. It does NOT manufacture a DataModel; Route-B live-DM structural gate UNCHANGED
(SH174 capture-latch stays the single forward hook). Single-agent, no default-config production path
edited, default-inert. recon-v3 plane re-verified green at this HEAD (24 task frames swap Ok(0x1),
191 node pops, 0 json abort, 0 crash), workspace green (examples 102/0, was 101/0).

Repro: runs/capture_sh266_bus_subscribe.sh.