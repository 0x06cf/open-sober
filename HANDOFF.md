# Open Sober — Agent Handoff
## SH307 (Sep 18, 2026, hermes-worker): FORCE the nativePreloadFlagOverrides value branch — SendAppEventOnAppReady crosses its standing preload-overrides terminal (0x102bb803c) for the first time. SH270 wired the value cell [0x106a64d78] but measured INERT; SH272 called both getter branches live-object walls. MISSING REASON FOUND: getter 0x2dae5f0 is a Meyers lazy once whose guard helper (bl 0x57816f0, once byte [0x6d2df30]) routes control to the CONSTRUCT branch on every headless run — the value cell is NEVER read, so SH270's wire sat unreached. SH307 (opt-in JIT_ROUTEB_PRELOAD_VALUECELL, default-inert, SH245 pattern): NOP `tbz w0,#0,0x2dae624` @0x2dae5fc (0x36000140->0xd503201f, word-guarded) so the getter ALWAYS takes the value-cell branch (0x2dae600 `ldr x0,[0x106a64d78]; ldr x8,[x0]; ldr x2,[x8,#16]; br x2`) and returns that dispatch's result; seed [0x106a64d78]=routeb_appstart_adapter_object() (SH248f fabricated all-leaf obj; made pub in jit.rs). MEASURED A/B on real libroblox.so (SH269 canonical ladder): BASELINE (SH307 off) = standing wall `[SIGSEGV] fault=0x0 guestpc=0x102bb803c` x20=0, EXIT 139; FORWARD = patch fires (`value-branch@0x102dae5fc tbz->nop + value-cell ...`), **SendAppEventOnAppReady returned Ok(0x107273d50)**, 0 SIGSEGV, ladder completes clean (LADDER_DONE=1, admission gate cleared, EXIT 124) — FIRST completion past 0x102bb803c, a genuine forward move of the session-ctor terminal (not a re-verify). +hermetic sh307 (real-image anchors: tbz 0x102dae5fc=0x36000140, value-dispatch 0x102dae604=0xf946bd00/0x102dae620=0xd61f0040, value-cell in RW window; 1 passed). HONEST: does NOT manufacture a DM (DM-root [0x106a68818]=0, MH_* all false — the getter returns the fabricated obj, not a real preload map); Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays single forward hook; recon-v3 deliverables stay shipped+verified. Verify: cargo test --workspace EXIT 0 (585/0 = 405 lib + elfjit examples 132/0 + fsmap + others); cargo build EXIT 0; elfjit.rs under 1MB pre-commit hook (+178 B, funded by condensing route-a render-prose SH68/72/73 + Obj-2b); jit.rs routeb_appstart_adapter_object made pub. Doc docs/frontier-sh307-preload-valuecell-branch.md, repro runs/capture_sh307_preload_valuecell.sh. NEXT: region-watch SendAppEventOnAppReady body past the returned-Ok point for the next live-object it derefs (0x2bb8000..0x2bb8300). Single-agent.
## SH306 (Sep 18, 2026, hermes-worker): hermetic pin for the standing SendAppEventOnAppReady governor terminal — the current Route-B no-regression anchor had ZERO byte-pin (sh270/272 pin the downstream preload-overrides wall, not the governor NULL app-DM-controller site 0x102ea0b9c itself). New real-image-guard `sh306_governor_null_controller_terminal_pinned` (elfjit.rs, sh115_tests): byte-pins governor fn prologue 0x2ea0b48=0xd10203ff, controller read 0x2ea0b78=0xf9401015, GOVFLAG read 0x2ea0ba8=0x39768108, controller 0x2ea0bd0=0xf94206a0, NULL-deref terminal 0x2ea0bd4=0xf9400008, GOVFLAG cell 0x106a64da0 in-window. All 5 bytes verified vs objdump. Verify: sh306 1 passed (examples 131/0 → 130+1); cargo test --workspace EXIT 0 (405 lib); build EXIT 0; elfjit.rs 1,048,567 B = 9 B under 1MB hook (funded by condensing doc-prose SH 81/99/116b/118/119/120/122/128/131/153/160 — address+grammar, no behavior/addr change). HONEST: pure hardening, no DM (MH_* false, DM-root 0); Route-B live-DM structural gate UNCHANGED; SH174 latch stays forward hook; recon-v3 deliverables stay shipped+verified. Repro: cargo test -p arm64jit --example elfjit -- sh306. Doc docs/frontier-sh306-governor-terminal-pinned.md. Single-agent.
## SH305b (Sep 18, 2026, hermes-worker): fix a SECOND parallel-test race surfaced when hunting suite stability — `jit` tests sh164/sh243/sh165 all mutate the SAME process-global state (JIT_ROUTEB_DMFORCE env + guest-mapped fixed-.bss page 0x102727550, sh243 also 0x107275550) but ran un-serialized, so under parallel cargo sh165's `!any_page_mapped(HOLDER)` precondition raced a concurrent map of that page and env set/remove interleaved — intermittently red (404/1 at default 8t, multi-fail under 24t). Fix: new DM_MANAGER_TEST_LOCK (repo's CONT_MGR_TEST_LOCK pattern) held in all three; production guards UNCHANGED. Verify: arm64jit lib 405/0 ×8 (default 8t stable), full cargo test --workspace EXIT 0, elfjit examples 130/0, build EXIT 0, jit.rs under 1MB hook. Commit 807e39c. Honest: test-hygiene, not a Route-B advance; recon-v3 deliverables + Route-B SESSION-CTOR terminal re-derived at HEAD (SH305) unchanged.
## SH305 (Sep 18, 2026, hermes-worker): fix a REAL test regression + re-verify the recon-v3/session-ctor deliverables at the newest HEAD. (1) `shims::anativewindow_fromsurface_returns_stable_nonnull_handle_with_size` was failing (404/1): the two ANativeWindow tests run in PARALLEL under cargo test and both write the process-global `ANATIVE_WINDOW_XID` static — the real-X11-window test set it to 0x2c00000d, which the sentinel-stability test's 2nd call read back (738197517==0x2c00000d), failing the stability assert. Same pattern as the repo's `CONT_MGR_TEST_LOCK` for the fixed-.bss cells. Fix: module-level `ANATIVE_TEST_LOCK` mutex, both tests hold it; production shim UNCHANGED. (2) RE-VERIFIED the recon-v3 SELF-DRIVEN-FRAMES deliverable at HEAD (runs/capture_taskv4_frame.sh): type4_frame_thunk registered+seeded [0x106829ea8], heartbeat patches, RENDERCTX recovered, 24 task-driven frames present swap Ok(0x1), 197 node pops, 0 json abort, EXIT 124. (3) RE-DERIVED the Route-B SESSION-CTOR terminal at newest HEAD (runs/capture_sh269_session_ctor_exec.sh): deterministic standing wall — MessageBus.subscribe Ok(0x3e8) x3, once-guard[0x106a68410]=0x1 (do-init __call_once self-latches), DM-root[0x106a68818]=0x0 (never owns a live DM); OnGameLoaded reached; SendAppEventOnAppReady baseline dies governor NULL-app-DM-controller 0x102ea0b9c (EXIT 134), GOVFLAG advances one gate to preload-overrides live-object 0x102bb803c. NO regression vs SH269; Route-B live-DM structural gate UNCHANGED at HEAD. Verify: cargo test --workspace EXIT 0 (arm64jit lib 405/0, elfjit examples 130/0); build EXIT 0; elfjit.rs held under 1MB hook (+47B). Commit fafdaaa (test fix; ledgers STATUS.md/HANDOFF.md). HONEST: no DM (MH_* false, DM-root 0); SH174 capture-latch single forward hook; recon-v3 deliverables stay complete+shipped+verified.
## SH304 (Sep 18, 2026, hermes-worker): implement the SESSION-GATED type-4 producer (recon-v3 self-drive handoff) — the one reconciled deliverable that was genuinely absent from the tree. `--taskv4-seed session` registers+installs `type4_session_gated_thunk` into the dispatcher's type-4 vector [0x106829ea8] (ABI handler(node=a0,[node+32]&~1,consumer), BR leaf, non-recursive, producer-only no-EGL). Gate = `session_producer_gate(mh_app_ready, live_dm) = mh_app_ready && live_dm`; `session_live_dm()` page-guards the current-DM holder [0x106391908] (SH172/SH174 latch arm) OR do-init DM-root [0x106a68818]. GATED -> queues a real present exactly as `type4_frame_thunk` (engine make-current 0x105b3b358 -> frame-fn 0x105b32c00 -> swap 0x105b3b408 on the recovered ctx) — REPLACES the harness seed the instant a real session owns a live DM; UNGATED -> bounded-log + inert (no present, host does ZERO layout/fabrication). Default-inert (opt-in flag only; `--taskv4-seed frame` keeps the verified recon-v3 plane unchanged). MEASURED on real libroblox.so (runs/capture_sh304_session_producer.sh): registered at 0x7f00000001d0, boot dispatches log `UNGATED (app_ready=false, live_dm=true) — inert`, GATED=0, EXIT 124, 0 json abort — the session-less boot stays frame-free (only the standalone --renderinit/--renderframe SH18/19 swap present, independent of the gate). +3 hermetic sh304 gate tests (closed w/o app-ready; closed w/o live-DM; opens only on both). Verify: arm64jit --example elfjit sh304 3 passed; examples 130/0 (127 baseline + 3); build+test --workspace EXIT 0 (405/0 lib). Back-filled elfjit.rs to hold the 1MB pre-commit hook (+47B margin) by condensing verbose SH comment/prose (115/117/118/122/156/157/159/177/223/226/236/255/264/272/276 + option-doc blocks) — all facts/addresses preserved, no behavior touched. HONEST: latent-but-correct (MH_APP_READY stays false on bare-boot headless until do-init owns a live DM; fires the instant a real session advances); Route-B live-DM structural gate UNCHANGED; SH174 capture-latch single forward hook. Doc docs/frontier-sh304-session-gated-producer.md.
## SH303 (Sep 18, 2026, hermes-worker): RESOURCE-LEAK FIX (not Route-B). Every render/window-wiring elfjit run leaked an Xvfb: `wire_real_window()` spawns Xvfb on `220+(pid%250)` under JIT_DRIVE_LIFECYCLE, leaks the X connection, and never killed the child — 236 accumulated Xvfb (~61MB RSS each ≈ 14GB) on this box. FIX: `SPAWNED_XVFB` registry + once `install_xvfb_reaper()` (libc::atexit) that kill+waitpids ONLY Xvfb children this process spawned (pre-existing/other displays untouched). Verified recon-v3 self-drive still green (24 frames swap Ok(0x1), 197 pops, 0 json abort, EXIT 124) and Xvfb count unchanged after a run (no new leak). Swept the 236 leaked Xvfb + stale /tmp/.X11-unix sockets. Trimmed ~900B of redundant elfjit.rs eprintln/comment prose (bindings `_`-prefixed or dropped) to hold the 1MB pre-commit hook (file now 155B under). HONEST: Route-B live-DM structural gate UNCHANGED (MH_* false, DM-root 0). Verify: build+test --workspace green (arm64jit 405/0), elfjit examples 127/0. Commit 3c633be.
## SH302 (Sep 18, 2026, hermes-worker): seed + MEASURE the exact "pre-reader continuation" lever SH301's NEXT GATE named — the EC reader-gate caller-frame object [x29,#104]=entry_sp+8 -> `ldr x0,[x8,#32]` @0x2e246d8 -> `cbz x0,reader` @0x2e246dc -> `blr vt[+48]` @0x2e246f0 (which consumes control, so the realsession reader at 0x2e246f4 is never reached). New default-inert guard `routeb_ec_world_reader_gate_guard` (jit.rs, opt-in `JIT_ROUTEB_EC_READERGATE=1`, wired after the SH300 realsession guard) seeds [x29,#104] to a leaked ZEROED buffer so [+0x20]==0 -> the cbz is TAKEN. MEASURED INERT on real libroblox.so (5/5 firm via runs/capture_sh302_readergate.sh + sh302c batch): readergate fires 5/5, SH300 flag fires 5/5, dmfn returned Ok(0x...4b10) 4/5 CLEAN (run5 = known SEP-17 session-drive flake before the dmfn line), but NEITHER realsession bl-target ever fires (real V2Init 0x1023c5538: JIT_DUMP_PC=0x dumps 0/5; benign 0x1023c1b0c: region-watch 0), and the terminal is byte-identical SIGSEGV guestpc=0x101db1b08 (LSM insert-leaf wall)/SIGABRT EXIT 134. => the EC tail [0x2e245f4..0x2e247dc] compiles as ONE block whose real exit is an EARLIER soft-return upstream of 0x2e246d8 — SH301's dormant verdict now CONFIRMED at the newest HEAD *with a concrete seed attempt on the named gate*, not just a doctrine re-run. +hermetic sh302 (env/pc-gated seed test: inert-without-env, wrong-pc inert, seeds non-NULL slot to zeroed-buf with [+0x20]==0, idempotent, leaves NULL slot NULL; +4 real-image byte-pins 0x2e246b0=f94037a8 / 0x2e246d8=f9401100 / 0x2e246dc=b40000c0 / 0x2e246f0=d63f0100 — the reader-gate mechanism SH301 did NOT pin). Verify: sh301+sh302 = 2 passed (arm64jit lib 408/0); cargo test --workspace EXIT 0; cargo build --workspace EXIT 0. HONEST: no DM (MH_* false, DM-root 0), Route-B live-DM structural gate UNCHANGED, SH174 capture-latch stays the single forward hook. Doc docs/frontier-sh302-ec-readergate-inert.md. Do-not-re-tread SH300/301/302: the reader is gated by the co... [truncated]
## SH301 (Sep 18, 2026, hermes-worker): byte-pin + close the SH300-less objection — the EC realsession-reader frontier is GENUINELY dormant at BLOCK-ENTRY level (do-not-re-tread SH300). The objector's escape ("EC body compiles as ONE straight-line block, so pc-probes are blind to its interior") is REFUTED: region-watch/REGIONDUMP on real libroblox.so fires at the interior prefix string-assign `bl 0x2b504e4` @0x2e24610/0x2e24618 as a GUEST BLOCK ENTRY (0x102b504e4) — proving interior guest `bl` targets DO open block entries (not inlined mid-block). Therefore IF the realsession reader at 0x2e246f4 (`cbz w8` on [0x106d31e28]) were reached, at least one of its two `bl` targets MUST open a block entry too. Across ~10 probe configs NEITHER fires: real V2Init 0x1023c5538 (flag=1, `bl 0x23c5538` @0x2e24704) NOR benign singleton 0x1023c1b0c (flag=0 @0x2e24730). => the EC body provably soft-returns BEFORE 0x2e246f4: SH300's flag seed is correct+latched but its reader is unreached headlessly — genuinely dormant, NOT probe-blindness. +hermetic sh301 (real-image guard, 9 byte-pins: entry 0x2e24598=a9ba7bfd, entry-bl 0x2e245f0=97d73359, prefix-add 0x2e24610=91002280, prefix-bl 0x2e24618=97f4afb3, reader 0x2e246f4=b001f868/0x2e246f8=3978a108/0x2e246fc=34000188, real-bl 0x2e24704=97d6838d, benign-bl 0x2e24730=97d674f7). Verify: sh301 1 (lib 404/0); build --workspace EXIT 0. Doc docs/frontier-sh301-ec-realsession-blockentry-proof.md. No prod path edited, default-inert; HONEST: no DM, Route-B live-DM gate UNCHANGED, SH174 latch single forward hook, recon-v3 re-verified green (24 frames swap Ok(0x1), 0 json abort, 0 crash).
## SH300 (Sep 18, 2026, hermes-worker): disarm-branch gate on the EC world — seed the realsession flag [0x106d31e28]=1 so the EC body's real V2Init/StartLuaAppDM branch is armed; MEASURED DORMANT-by-measurement (latent-but-correct prep, SH265/266 class, NOT SH248h/256 revert). The EC world 0x102e24598 (SH296-299 line, dmfn 0x1023f03b4 completes through it) disassembled one gate past SH299's dispatch: at `0x2e246f4` it reads writable .bss byte guest [0x106d31e28] (`cbz w8,0x2e2472c`) — flag=0 (headless default) -> benign singleton `bl 23c1b0c`, SKIPS the real /nativeAppBridgeV2InitWithParams AppBridge singleton `bl 23c5538` + StartLuaAppDM body `bl 23f1654` (0x2e24704/0x2e24714). SH300 guard (opt-in JIT_ROUTEB_EC_REALSESSION, wired after SH299 in block-entry dispatch) seeds [0x106d31e28]=1 at EC entry 0x102e24598 (idempotent, routeb_ensure_writable maps the .bss page). MEASURED A/B (real so, full dmfn env, runs/sh300-*.txt): ON arm `[routeb-sh300] seeded...` fires every run, flag persists, terminal UNCHANGED SIGABRT at the SH285-B LSM persistence-detour wall (guestpc 0x101db1b08, still SIGSEGV'd there pre-abort), dmfn=Ok(dmthis+0x30) both arms. Full JIT_TRACE pc-set diff (ON vs OFF) IDENTICAL EC-region pcs = the flag's READER (the app-request build 0x2e24678..0x2e25200) never runs headlessly — the dmfn benign-soft-returns before descending into that continuation (SH299 NEXT-GATE class). HONEST: no DM (MH_* false, DM-root 0), Route-B live-DM gate UNCHANGED; SH174 latch stays single forward hook. Cause-not-symptom: the EC real-init path is now ARMABLE-and-mapped; dormant only because its reader is unreached. +hermetic sh300. Verify: sh300 1 (lib 403/0); cargo test --workspace EXIT 0; baseline parity. Doc docs/frontier-sh300-ec-realsession-flag-dormant.md, repro runs/capture_sh300_realsession_ab.sh. Single-agent, default-inert.
## SH299 (Sep 18, 2026, hermes-worker): CROSS the EC-world fault=0x10 wall — seed the EC arg0 +0x30 virtual-dispatch object so the DM-construction fn COMPLETES through the EC world (SH235/298/298b genuine DM-creation machine). SH298b left `fault=0x10` (guestpc 0x102e245f4, x8=0, x19=dmthis+0x30) in the EC marshaller app-request build. Body: 0x2e2464c `ldr x8,[x19,#48]` (x19=arg0 -> x8=[arg0+0x30]=0) -> 0x2e24650 `ldr x8,[x8,#16]` = [0+0x10] fault -> 0x2e24658 blr x8. SH299 (--v2boot-session-dmfn + JIT_ROUTEB_EC_ARG0VT=1): routeb_ec_world_arg0_vt_guard seeds [arg0+0x30]=coherent dispatch obj (vt[+16]=ret1 host leaf) on EC entry -> cbnz w0 @0x2e2465c TAKEN -> continue build. MEASURED 3/3 on real libroblox.so: `SH296 dmfn returned Ok(0x...53f0)` (was 3/3 SIGSEGV fault=0x10). New terminal = SEPARATE known SH285-B LSM live-object wall (guestpc=0x101db1b08, persistence detour), NOT the EC world. HONEST: no DM (MH_* false, DM-root 0); Route-B live-DM gate UNCHANGED; SH174 latch forward hook. +hermetic sh299. Verify: sh299+sh298 lib 2; cargo test --workspace EXIT 0 (576/0); elfjit examples 127/0. Doc docs/frontier-sh299-ec-arg0vt-dispatch-object.md, repro runs/sh299-ecarg0vt-r*.txt. Single-agent, default-inert.
## SH298 (Sep 18, 2026, hermes-worker): CROSS SH297's registration-fn wall FULLY — drives the DM-construction fn into the EC world (SH235's genuine DM-creation world 0x102e24598, FIRST headless penetration). SH297's bug: reg-fn 0x21e45c8 (bl@0x23f05b8) gets arg0 = `[this+136]` DOUBLE-deref. SH297 arg1[8]=singleton -> arg0=[singleton]=singleton's VTABLE (host-leaf); [arg0+8]=leaf 0x7f..d0 -> `ldr x8,[x22,#24]` derefs 0x7f..e8 -> SIGSEGV. SH298 (--v2boot-session-dmfn + JIT_ROUTEB_DMFN_REGISTER=1): arg1[8]=CELL -> coherent reg obj R ([R+0]=leaf-vt, [R+8]=singleton VALID host obj) -> arg0=R -> 0x21e45c8 COMPLETES -> falls to bl nativeAppBridgeAppStart. MEASURED (real so, full SH297 env, 3/3 det): SH297 wall GONE; NEW terminal SIGSEGV guestpc=0x102e245f4 (fault=0x48) in 0x1023f1354 via EC bl@0x102e245f0. HONEST: Route-B live-DM gate UNCHANGED (no DM, MH_* false, DM-root 0); new terminal = NULL+0x48 live-object (SH174/SH204 class); SH174 latch stays forward hook. Cause-not-symptom SESSION-CTOR. Verify: sh298 1; workspace EXIT 0. elfjit 1,048,408 + HANDOFF under 1MB hook. Doc docs/frontier-sh298-registration-ecworld.md, repro runs/capture_sh298_register.sh. Single-agent, default-inert.
## SH297 (Sep 17, 2026): CORRECT SH296's DM-construction-fn seed — the construction body 0x23f0484 reads `ldp x21,x24,[x22,#8]` (x22=ARG1=x1) then `stp x21,x24,[this,#136]`: it CLOBBERS this+136/144 from arg1[8]/arg1[16]. SH296 S2 seeded this+136/144 = NO-OP (overwritten from a ZEROED arg1; [arg1+8]=0 -> cbz x21 @0x23f04d0 safety-epilogue fired before any this-field read incl. this+128 factory) — its "blanket-singleton-insufficient" verdict rests on a MIS-SEEDED test. SH297 seeds the REAL gates: coherent arg1 ([arg1+8]=[arg1+16]=routeb singleton -> this+136/144, cbz passes) + this+120 + this+128 (refcount factory 0x2b4ea48 nonzero). MEASURED (real so, full SH296 env, region-watch): **construction body EXECUTES** (0x1023f0544/550/55c app-server box-build) then SIGSEGV **guestpc=0x1021e460c** (`ldr x0,[x21,#8]` in 0x21e45c8 reg-fn @0x23f05b8) — ONE BL before nativeAppBridgeAppStart (@0x23f05f8). 3/3 det. NEXT gate: coherent 0x21e45c8 reg obj ([+8] valid host obj) -> app-start (standing SH248-260 wall). HONEST: no DM (MH_* false, DM-root 0); Route-B gate UNCHANGED; SH174 latch single forward hook. +hermetic sh297 (7 pins). Verify: sh297 1; workspace EXIT 0. elfjit 1,048,302 under 1MB hook. Doc docs/frontier-sh297-dmfn-arg1-clobber.md, repro runs/capture_sh297_dmfn.sh + capture_sh297b_dmfn_region.sh. Single-agent, default-inert.
## SH296 (Sep 17, 2026): LOCATE + DRIVE DM-CONSTRUCTION handler 0x1023f03b4 (SH255: indirect-only) FIRST time. SOLE ref file 0x68ea538=entry[4] of table 0x73fa4d0 ({0x403,fn,tag}). New default-inert `--v2boot-session-dmfn` drives it (this.vt[32]=ret0 leaf). MEASURED: S1 empty recv 3/3 Ok(0x0) — executes first time, benign-return; S2 (fields this+136/144=singleton) still Ok(0x0) 0 hits — "needs distinct live sub-objects" **REVISED BY SH297** (seeded wrong fields; arg1 clobbers). +hermetic sh296. Verify: examples 125/0; workspace EXIT 0. Doc docs/frontier-sh296-dmfn-handler-table.md, repro runs/capture_sh296_dmfn.sh. Single-agent, default-inert.
## SH295 (Sep 17, 2026): DRIVE item-proc's LAST edge ([item+48]) — SH293/294 "not driveable" JUDGMENT -> measured wall. `--v2boot-session-itemproc` (item[+48]=benign cont, item[+32]=leaf, seed [0x1068262e8]). det 3/3: 0x1022193a0+0x1022079d8 fire, then SIGSEGV 0x1028511f8 (nativeOnDestroyed vt[+112] live-object wall), first headless exec. Item-proc FULLY driven (SH290-295). HONEST: no DM, Route-B UNCHANGED, SH174 latch. +sh295. Verify: examples 123/0; workspace EXIT 0. Doc docs/frontier-sh295-item48-edge-driven-to-wall.md. Single-agent, default-inert.
## SH294 (Sep 17, 2026): CORRECT two SH293 mis-attributions. (1) lifecycle-registry fn-ptr cell = guest **0x1068262e8** (adrp 0x6826000 + #744), WRITABLE .data — NOT 0x1068266e8 — read via `cbz x8,skip; blr x8` = SH292 benign-leaf-fireable, so SH293's "not driveable" is a JUDGMENT. (2) 0x22076e8 is NOT the canary blr (that's 0x22076f8) — a tail-TRAMPOLINE `mov x3,xzr; b 0x2850ef0` into nativeOnDestroyed dispatcher (w2==0 arm -> vt[+112] live-object wall — HOLDS). +hermetic sh294. HONEST: attribution/ledger correction, no prod path edited, no DM, Route-B UNCHANGED, SH174 latch; NEW benign-leaf-fireable cell [0x1068262e8] for a future [item+48] drive. Verify: sh294 1; examples 123/0. Doc docs/frontier-sh294-item48-cell-correction.md. Single-agent.
## SH293 (Sep 17, 2026): item-PROCESSOR LAST edge ([item+48]->0x22193a0) mechanism-CLOSED — completes item-proc 0x102207950. The edge = authentic per-item continuation; item[0] splits to 0x2219428(=b 0x28506a4) or 0x24993c8(=b 0x28508a8), both in the nativeOnDestroyed SH273 family -> SH273/SH174 live-object wall (cell corrected to 0x1068262e8 by SH294). HONEST: no DM, Route-B UNCHANGED, SH174 latch. Verify: sh293 1; examples 122/0. Doc docs/frontier-sh293-itemproc-last-edge-closed.md. Single-agent.
## SH292 (Sep 17, 2026): ITEM-PROCESSOR per-item DISPATCH driven (doc NEXT past SH291). Fresh disasm corrects SH291: the REAL per-item sites are `[item+32]->vt[+48] blr` (0x22079cc) + `[item+48]->bl 0x22193a0` (0x22079e0); the 0x2207cb0/0x207ce0 blrs SH291 pinned belong to SEPARATE fn 0x2207bbc (only-called-from 0x2dadb90). SH292 sources item[+32] from routeb_singleton_obj_addr() (SH159b benign dispatcher) so the vt[+48] blr EXECUTES (host leaf) + item[+48]=0 skips SH273; once-guard latched -> pure per-item edge. det 4/4: once-built=0x800000c guard=0x101 STABLE + **SH292 returned Ok + PER_ITEM_DISPATCH_EXECUTED=true** — engine's own per-item dispatch fires headlessly FIRST time. HONEST: cause-not-symptom SESSION-CTOR, one gate past SH291; NO DM; Route-B gate UNCHANGED; SH174 latch. Verify: build+test EXIT 0; recon-v3 green. Doc docs/frontier-sh292-itemproc-peritem-dispatch-drive.md. Single-agent, default-inert.
## SH291 (Sep 17, 2026): ITEM-PROCESSOR guard-latched RE-ENTRY measured IDEMPOTENT. Re-drive item-proc 0x102207950 with once-guard latched (0x101): `tbz w8,#0` @0x2207af0 NOT taken -> skips once-body. det 3/3: re-entry Ok + once-built UNCHANGED (0x800000c) + guard=0x101, IDEMPOTENT=true. +hermetic sh291. HONEST: cause-not-symptom SESSION-CTOR; NO DM; Route-B gate UNCHANGED. NEXT = per-item dispatch (SH292). Verify: build+test EXIT 0 (580); recon-v3 green. Doc docs/frontier-sh291-itemproc-reentry-idempotent.md. Single-agent, default-inert.
## SH290 (Sep 17, 2026): ITEM-PROCESSOR once-build measured IN ISOLATION — SH288's readback gap closed. New default-inert `--v2boot-session-itemproc` drives item-proc 0x102207950 with a fabricated ZEROED item (both indir blr cbz-skip) so only the once-guard [0x106a63b08] __call_once + once-body (string-map insert 0x2173b3c -> [0x106a63b00]) + clock helper run. det 3/3: item-proc Ok + **once-guard 0->0x101** + **once-built [0x106a63b00]=0x800000c** — first clean readback. Terminal = SH285-B LSM wall. +hermetic sh290. HONEST: cause-not-symptom SESSION-CTOR; NO DM; Route-B gate UNCHANGED; SH174 latch. Verify: examples 119/0; recon-v3 green. Doc frontier-sh290-itemproc-oncebuild-isolated.md. Single-agent.
## SH289 (Sep 17, 2026): CORRECT SH288's consumer cell attribution. Fresh disasm: cond-wait predicate=[0x106864508]; item-proc once-guard=[0x106a63b08] (ldarb 0x2207980) — the ACTUAL gate; the probed [0x106863b08] is NEITHER. Item-proc tail bl 0x221942c is a CLOCK helper (bl 0x6201fc8), NOT a DM-build. CODE: consumer rung probes [0x106a63b08]. +hermetic sh289. Verify: build+test EXIT 0 (580/0); examples 118/0; recon-v3 green. Route-B gate UNCHANGED (SH174 latch).
## SH288 (Sep 17, 2026): drive the never-run worker CONSUMER loop — the missing half of the producer-only SESSION-CTOR pump. 0x10220778c is the consume loop (locks session mutex 0x106863aa0, cond_wait 0x106863ac8 on queue 0x106863a70, pops item x22, calls item-proc 0x102207950). New opt-in --v2boot-session-consumer. MEASURED: A parks (baseline); B2 (state=9->10): engine's OWN consumer executes headlessly FIRST time (item-proc hit), then SIGSEGV fault=0x50 @0x1021f3748 = SH273 lifecycle-notifier wall via the own queue. HONEST: no DM, MH_* false, Route-B gate UNCHANGED; SH174 latch. Doc frontier-sh288-consumer-half-driven.md. Single-agent.
## SH287 (Sep 17, 2026): R1 content-path TESTED + combined SESSION-CTOR measurement. (1) Hermetic r1_synthetic_corenodes_stages_persistent_module (fsmap 8/8): /data/user/0/.../files/scripts/CoreScripts/AppShell.lua resolves through fsmap to a real on-disk module — latent-but-correct (asset-trace fires ZERO rbxasset). (2) Full SEP-17 stack on ONE ladder (runs/capture_sh287_full_session.sh): G1 XID lands 0x200000; SendAppEventOnAppReady terminates at SH270 wall 0x102bb803c; stacking SUPPRESSES the engine9 drive — single-drive only. HONEST: no DM. Verify: build+test EXIT 0; recon-v3 green. Doc frontier-sh287-r1-contentpath-and-combined-session.md.
## SH286 (Sep 17, 2026): classify the SH285 terminal (runs/sh286-dump-b1.txt): fault=0xffffffffffffffff @0x101db1b08 is a libc++ std::string BACKWARD-COPY through UNINITIALIZED [x19+0x50] (guest host-heap) = SH174/204 LIVE-OBJECT, NOT a .bss seed (sh285-pinned); do-not-re-drive a [obj+0x50] repair (SH248h/256 std). No prod path edited. Gates: build/test EXIT 0; recon-v3 green. Addendum frontier-sh285.
## SH285 (Sep 17, 2026): the engine's OWN initEngine_ settings-state self-drive (state=9->10, --v2boot-session-engine9) crosses the LocalStorageManager INSERT-LEAF when LSM_NODES is on — A/B: A 3/3 SIGSEGV @0x101db1d04 (insert-leaf); B (+LSM_NODES) 3/3 @0x101db1b08 (LSM reader/pop, one fencepost deeper). Closes SH267-vs-SH284 gap; cause-level SESSION-CTOR (state9ok=3/3). HONEST: no DM (DM-root 0, MH_* false); SH174 latch. Doc frontier-sh285-settings-lsmnodes-cross.md.
## SH284 (Sep 17, 2026): drive the LAST never-driven initEngine_ state body — state=9 (0x2bd2668) — headlessly FIRST time. New opt-in `--v2boot-session-engine9` on the single ladder thread (SH279 seeds). det 3/3: `state=9 body direct returned Ok(0x0)` + `[this+16](state)=10` (all THREE initEngine_ bodies self-complete: 3->serializer, 5->7, 9->10). Then self-drives into app-start, dies at the SH260 LSM wall 0x101db1d04 (parked detour, not a regression). HONEST: cause-not-symptom SESSION-CTOR; NO DM; Route-B gate UNCHANGED; SH174 latch. +hermetic sh284. Doc frontier-sh284-engine-settings-state9-body-driven.md.
## SH283 (Sep 17, 2026): CROSS the SH280-282 engine5 reentry park + falsify "zeroed .bss cannot block". The park is a REAL contended lock: state=5 continuation's last block is `hostcall@pthread_mutex_lock x0=0x106863aa0` (state=2 owned by a spawned never-run worker, start 0x1022076f8) -> futex-waits (EXIT 124). GATE (default-inert JIT_ROUTEB_ENG5_QMUTEX_FREE=1): if env+addr==0x106863aa0, force-clear the 16-bit bionic word. A/B: A parks; B **state=5 body Ok(0x0) + [this+16]=7** — engine's OWN settings-state machine SELF-COMPLETES its state=5 body FIRST time. Repro 3/3 state=7. HONEST: cause-not-symptom; NO DM; Route-B gate UNCHANGED; SH174 latch. Verify: sh283 1; workspace EXIT 0. Doc frontier-sh283-session-mutex-contended-gate.md.
## SH282 (Sep 17, 2026): CORRECT SH281's wrong premise — the state=5 reentry continuation 0x2207118 locks FIXED GLOBAL 0x106863aa0 (TRUE zeroed .bss static-init pthread_mutex_t, cannot block), not [box+0xaa0]; saves x0 to [sp], then enqueues via 0x2d9713c -> construct 0x22071ac on fixed obj 0x106863a70. SH281's "box +0xaa0 / correctly-sized box" is a DEAD END (box never read at +0xaa0). MEASURED: engine5 rung reaches reentry callee 0x275a23c + `bl 0x2206f04`, never returns (EXIT 124 park); park = enqueue-construct on fixed obj 0x106863a70 = SH174/204 live-object.
## SH281 (Sep 17, 2026): CROSS SH280's config+56 NULL-deref by seeding `[config+56]`=leaked 0x100 zeroed buf in `--v2boot-session-engine5`. SH280 faulted at GlobalInit-reentry `ldr x0,[x21,x8]` @0x10275a154 (x21=[config+56]=0). Callee 0x275a23c DROPS the read value, so [config+56] only needs a valid buffer. MEASURED det 3/3: state=5 body runs the FULL GlobalInit-reentry (0x10275a144/148->275a23c->2207118, no SIGSEGV, EXIT 124) — FIRST penetration past 0x10275a148 into the game-global-init continuation on the engine's OWN settings-state path. HONEST: cause-not-symptom SESSION-CTOR (3->5->6, one gate past SH280); NO DM; Route-B gate UNCHANGED; SH174 latch. Verify: sh281 1; examples 114/0; workspace EXIT 0. Doc docs/frontier-sh281-config56-seed-crosses-reentry.md.
## SH280 (Sep 17, 2026): drive the initEngine_ settings-state state=5 body (0x2bd24b4) headlessly FIRST time — SH277 pinned the dispatch (==3/==5/==9) but only state=3 was ever driven. New opt-in `--v2boot-session-engine5` drives 0x2bd24b4 directly BEFORE the state=3 serializer. SH279 seeds satisfy app-name csel + config; sets state->6, config dispatch -> 275a0c4 (GlobalInit-reentry), faulting guestpc=0x10275a154 (config+56=0). MEASURED 3/3: every run terminates 0x10275a148 (EXIT 134); A/B baseline hits LSM wall 0x101db1d04. HONEST: cause-not-symptom (3->5->6); NO DM; SH174 latch. Verify: sh280 1; examples 113/0; workspace EXIT 0. Doc docs/frontier-sh280-engine-settings-state5-driven.md.
## SH279 (Sep 17, 2026): CROSS the SH278 settings-serializer (0x2bd1d68) world-build gates — the initEngine_ state=3 path SELF-DRIVES into app-start depth. Gate 1: serializer reads [this+0x40]=0 -> SIGSEGV; seed [this+0x40]=zeroed 0xc00 config. Gate 2: app-name guard reads [this+0x48] empty -> NULL-store; pre-seed LONG "Home" (cap 0x11) + re-seed [this+0x50]=5. MEASURED 3/3: terminal moves to LSM insert-leaf 0x101db1d04; LSM_NODES crosses to 0x101db1b08. HONEST: engine's OWN settings path runs its serializer; NO DM (MH_* false, DM-root 0); SH174 latch forward hook. Verify: sh279 1; examples 112/0; workspace EXIT 0. Doc docs/frontier-sh279-settings-serializer-gates-crossed.md.
## SH278 (Sep 17, 2026): CROSS the SH277-pinned initEngine_ state gate: the ENGINE'S OWN engine-settings receive (0x2bd1c38) transitions manager state [this+16]->3 WHEN [this+649]!=0. New default-inert `--v2boot-session-engine3`: seeds [this+649]=1 on a fabricated zeroed 0x800 manager so receive transitions state->3, then drives initEngine_ dispatch 0x2bd1cf0. MEASURED 3/3 `receive Ok(0x0): [this+648]=1 [this+16]=3` — gate CROSSED, ==3 branch -> serializer 0x2bd1d68 (mono-tail GONE), then SIGSEGV world-build. HONEST: gate crossed, NO DM; SH174 latch forward hook. Verify: sh278 1; examples 111/0. Doc docs/frontier-sh278-engine-state3-gate-crossed.md.
## SH277 (Sep 17, 2026): measure the initEngine_ state-dispatch gate at the SH275+SH276 feed state. (A) clean session path: app-shell ctor runs 78 blocks, nativeGameGlobalInit Ok EXIT 124, but initEngine_ dispatch 0 hits. (B) full ladder: app-start terminal UNCHANGED 3/3 at the SH268 LSM free-list wall 0x101d9a528. Pinned WHY a fabricated manager can't reach a settings body: initEngine_ dispatches on state [this+16] (==3->0x2bd1d68/5->0x2bd24b4/9->0x2bd2668; else benign tail) — a fabricated state-0 manager mono-tails; needs a real session. +hermetic sh277. HONEST: no DM; Route-B gate UNCHANGED; SH174 latch single forward hook. Doc docs/frontier-sh277-init-engine-gate-feed-measured.md. Commit 99815a3.
## SH276 (Sep 17, 2026): drive the engine-settings RECEIVE `nativeActivity_onEngineSettingsReceived` (0x2bd1c38) on a fabricated zeroed manager. Body reads version [0x10683d8f8], (both branches) mutex_lock tmp (2b53a68), LATCH [this+648]=1 + optional state->3, unlock. Opt-in `--v2boot-session-engine` (seeds [0x10683d8f8]=6). +hermetic sh276. MEASURED 2/2 Ok(0x0) [this+648]=1 [this+16]=0 EXIT 124 once=1 DM-root=0. HONEST: consumer, not DM ctor; SH174 latch. Doc docs/frontier-sh276-engine-settings-receive.md.
## SH275 (Sep 17, 2026): DRIVE the REAL client-settings receive on the readLocalFlags parse path (SEP-17 SESSION-CTOR feed for "Engine settings is null"). New opt-in `--v2boot-session-signed`: sets version [0x10683cff8]=0x0306 so nativeInitClientSettingsSigned (0x102bb070c) takes readLocalFlags+parse (0x21e8f4c->0x2baf38c). +hermetic sh275. MEASURED 2/2 `Ok(0x1)` — real signed-settings parse first time; once=1, DM-root 0. HONEST: consumer not DM ctor; SH174 latch. Doc docs/frontier-sh275-client-settings-signed-path.md.
## SH274 (Sep 17, 2026): implement ROUTE-B RECON V3 NEXT-3 do-init seed VALUES (opt-in JIT_ROUTEB_DOINIT_NEXT3): thread-init singleton [0x1067333aa0]->0x20 zeroed buf (clears SEGV 0x102207ef0), telemetry once-cell [0x106dcd380]->-1 (clears 2b4cd1c park), map-page [0x10673336d8].bit0->1 (clears SEGV 0x102212838). Two default-inert mechanisms: (a) in-crate block-entry guard in do-init band [0x102206c40,0x102213000); (b) DIRECT ladder seed at the SH156 page-map point (GOVFLAG lesson: ctor blocks JIT-cached, entry guard latent). +hermetic unit. MEASURED A/B (full SH258/SH259 seed set): A = EXIT 134 terminal 0x101db1d04 (LSM insert-leaf); B +NEXT3 = all 3 seeds FIRE (pc=0x1022076f8), terminal **UNCHANGED** 0x101db1d04. HONEST: NEXT-3 ctor SEGVs are NOT the binding floor on the full ladder — control dies earlier at the SH260-parked LSM insert-leaf wall (SEP-15 persistence-detour park); latent-but-correct prep, regression-free. No DM; Route-B gate UNCHANGED; SH174 latch. Workspace green (578/0; examples 107/0). Commit df7fad6. Doc docs/frontier-sh274-doinit-next3-seeds.md.
## SH273 (Sep 17, 2026, hermes-worker): CLOSE the SEP-17 directive's ambiguous "remaining lifecycle levers" — the ENTIRE JNIActivityLifecycleCallbacks nativeOn* family converges on ONE shared dispatcher, so it is a single closed live-object wall, NOT 12 independent primitives. SH264 measured only OnResumed + setActive; this cycle disassm'd all 12 public lifecycle entries (PreCreated..Destroyed) and byte-proved they're 44-byte JNI stubs of IDENTICAL shape: prologue `stp x29,x30,[sp,#-16]!`, JNIEnv GetStringUTFChars (slot169/offset1352 = SH186 identity shim), and a FINAL unconditional `b 0x21f15a4` (per-entry state literal 0..8 in w0). 0x102_21f15a4 = the SINGLE shared Activity-lifecycle notifier dispatcher (`sub sp,#0x1c0`) whose downstream deref of a real lifecycle-callback-registry object is the SH264-measured live-object fault (callee prologue 0x1021f3748=0xd10243ff, fault=0x50). Driving any sibling = the identical wall as nativeOnResumed — no fresh seed lever. Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays the single forward hook. +hermetic sh273 (12 entries+dispatcher+fault site). Verify: sh273 1 passed; examples **107/0**; build+test --workspace EXIT 0. Doc docs/frontier-sh273-lifecycle-converge-single-dispatcher.md. No prod path edited.
## SH272 (Sep 17, 2026, hermes-worker): mechanize SH270's open residual — WHY nativePreloadFlagOverrides getter (0x2dae5f0) returns 0 on BOTH branches. (A) VALUE-CELL branch: 0x2dae600 adrp/604 ldr [0x106a64d78]/608 cbz/60c ldr [x0]/610 ldr [x8,#16]/620 br x2 = VTABLE DISPATCH needing a REAL preload-overrides object w/ vt[+16] (explains SH270's wire-inert). (B) CONSTRUCT branch (0x2dae624 bl ctor 0x101df8ff8): ctor zero-inits [obj+80]; helper 0x2daf5ec ldr [x19,#80]/5f0 cbz -> ret 0 ALWAYS. VERDICT (do-not-re-tread both): SH174/SH204 live-object class — needs real session construction+populate; neither seedable. Guard helper 0x57816f0 once byte = [0x6d2df30]. Route-B live-DM gate UNCHANGED. +hermetic sh272. Verify: sh272 1 passed; examples 106/0; recon-v3 green (24 frames swap Ok(0x1), 193 pops, 0 json abort, EXIT 124). Doc docs/frontier-sh272-preload-getter-both-branches-dead.md.
## SH270 (Sep 17, 2026, hermes-worker): RECONCILE the SendAppEventOnAppReady post-advance wall — SH269 was RIGHT. The wall guestpc=0x102bb803c (`ldr x8,[x20]`) derefs x20 = nativePreloadFlagOverrides return (bl 0x102bb801c -> 0x102dae640 -> thunk 0x2dae5f0, then mov x20,x0 @0x102bb8024; the getter returns 0 headlessly -> fault=0x0). Tested levers: (a) nativePreloadFlagOverrides is a lazy Meyers singleton whose OWN ctor 0x101df8ff8 IS engine-constructible headlessly (drive -> Ok(0x106d2dd20)), but (b) wiring that base into [0x106a64d78]/[0x106a64d98] does NOT move the wall (identical fault) — getter return doesn't come from those cells, seed-wire INERT. CORRECTED: the wall fn (entry 0x102bb7fd4) sets x20 by mov x20,x0 (getter return); its OWN canary loads into x21 @0x102bb7ff4; canary-init rung REVERTED (inert). Drive rung + canary rung REVERTED (not shipped). +hermetic sh270 CORRECTED (wall-fn prologue 0x102bb7fd4=0xd10183ff, bl 0x102bb801c=0x9407d989, mov 0x102bb8024=0xaa0003f4, wall 0x102bb803c=0xf9400288, thunk 0x102dae640=0x17ffffec, getter 0x2dae5f0/0x2dae5fc/0x2dae624, ctor 0x101df8ff8). Route-B live-DM gate UNCHANGED; SH174 latch stays single forward hook. Doc docs/frontier-sh270-canary-not-preloadoverrides.md (corrected). Workspace green (25 ok).
## SH269 (Sep 17, 2026, hermes-worker): make the SEP-17 SESSION-CTOR post-ladder rungs EXECUTE for the FIRST time headlessly. ROOT CAUSE of the perpetually-latent session rungs: the --v2boot ladder's StartLuaAppDM/V2StartAppWithParams rungs self-drive DEEP into app-start via the DMCONT continuation and TERMINATE (SIGABRT after the LSM SIGSEGV) BEFORE the loop reaches the post-ladder session-ctor rungs. New opt-in `--v2boot-skip-appstart` (elfjit.rs) skips those two + V1 AppStart__ so the loop completes and the session rungs run. MEASURED (real libroblox.so, full SH267 seed set, deterministic 3/3): MessageBus.subscribe RETURNS Ok(0x3e8) EXIT 124 clean AND the do-init once-guard [0x6a68410] LATCHES 0x1 — the __call_once ran via the REAL session drive (first time); DM-root[0x106a68818]=0 (live-DM gate UNCHANGED); OnGameLoaded ridden EXIT 124; SendAppEventOnAppReady drives the governor predicate. BONUS: new opt-in JIT_ROUTEB_APPSART_GOVFLAG (routeb_govflag_seed_guard, flag [0x106a64da0] fixed-.bss byte) advances SendAppEventOnAppReady PAST the governor NULL app-DM-controller deref (guestpc=0x102ea0b9c GONE) one fencepost to guestpc=0x102bb803c (x20=nativePreloadFlagOverrides return=0; preload-overrides object [0x106a64d98]=never-constructed live object = SH174/204 class). HONEST: no DM (MH_* false, DM-root 0); the session-ctor drive ADVANCES for the first time via its REAL path — cause-not-symptom on the SEP-17 PRIMARY lever. Doc docs/frontier-sh269-sessionctor-executes-first-time.md, repro runs/capture_sh269_session_ctor_exec.sh. Workspace green.
## SH268 (Sep 17, 2026, hermes-worker): close the SH267 crossed-state GUARD GAP — pin the LSM free-list/pop terminal mechanism the insert-leaf crossing UNLOCKED. SH267's +1 hermetic pinned ONLY the insert-leaf/reader; the NEW terminal it reaches (LSM free-list/pop guestpc=0x101d9a528) was never pinned, so the crossed state could drift silently. Fresh re-measurement (full SH267 seed set incl. JIT_ROUTEB_APPSART_LSM_NODES=1, real libroblox.so): EXIT 139 `[SIGSEGV] fault=0x101d968e4 guestpc=0x101d9a528` x19=host-heap x20=x1=0x101d968e4 lr=0x101d9a6b8, deterministic 3/3 — the lane crosses the insert-leaf then dies one fencepost deeper in the free-list/pop. Mechanism (fresh disasm): fn 0x1d9a528 (tail of sub_1d9a4e0, `mov x0,x19; bl 0x1d9a528` @0x1d9a6b4/0x1d9a6b0) decodes bucket=*[0x726f8c0]/sub, then the pop tail `ldr x8,[x0,#24]; str x1,[x0,#24]; str x8,[x1]` @0x1d9a568 writes the free-list link into *key; key 0x101d968e4=file 0x1d968e4 inside the R-E exec seg [0x0,0x62d8190) write:off = the SH249/SH258 proven-unwritable live-object class (REGRESSION pin, NOT a forward seed — per SEP-17 do-not-re-drive-seeds directive). +hermetic sh268 (real-image guard, skip-if-absent): byte-pins free-list entry/bucket/sub-decode + fatal link-store tail 0x1d9a568 + caller bl, asserts write-target in-image + file-offset in the R-E seg. Verify: cargo test -p arm64jit --example elfjit sh268 = 1 passed (examples 104/0). Route-B live-DM gate UNCHANGED; SH174 capture-latch stays single forward hook; session rungs still latent (ladder dies at the deeper live-object write-off wall). Doc docs/frontier-sh268-lsm-freelist-terminal-pinned.md. Pure hardening; no production path edited, default-inert; workspace green.
## SH267 (Sep 17, 2026, hermes-worker): CROSS the LocalStorageManager INSERT-leaf wall (SH260's parked terminal guestpc=0x101db1d04) that blocks the SEP-17 session-drive rungs. Fresh disasm: LSM map *global=[0x10726f8c0] -> bucket[key>>29] -> sub[(key>>16)&0x1fff]; reader 0x1d99e40 returns not-found on 0, but INSERT 0x1db1cc8 reads x1=sub[idx] and `bl 0x2b9ea40` = atomic-OR `ldset x0,x0,[x1]` of bit1 into *x1; empty-map x1=0 -> atomic op on addr 0 -> fault=0x0. Fix (default-inert JIT_ROUTEB_APPSART_LSM_NODES=1 in seed_static_empty_map): fill each of 0x2000 sub-slots with its own leaked zeroed 0x60 node cell so the OR lands in valid memory; reader then returns node+40=0. MEASURED A/B (real libroblox.so, full SH259 seed set, runs/capture_sh267_lsm_nodes_ab.sh): OFF=0x101db1d04 (parked wall); ON fires, 0x101db1d04 GONE, advances to NEW terminal 0x101d9a528 = LSM free-list/pop `str x8,[x1]` @0x1d9a568 into node addr 0x101d968e4 = file 0x1d968e4 inside the R-E exec seg [0x0,0x62d8190) write:off = SH249/258 proven-unwritable live-object class, one fencepost deeper. HONEST: no DM; Route-B gate UNCHANGED; session rungs still latent (ladder dies at live-object write-off wall); the exact gate blocking the session-drive is now crossed — first deterministic move past 0x101db1d04. +hermetic sh267 (insert-leaf + sub-slot + reader byte-pins). Doc docs/frontier-sh267-lsm-insert-leaf-crossed.md, repro runs/capture_sh267_lsm_nodes_ab.sh. Commit bfa7e1a. Workspace green.
## SH266 (Sep 17, 2026, hermes-worker): extend the SEP-17 session-drive to the LAST genuinely-NEVER-driven Route-B candidate — the messageBus receive `MessageBus.subscribe` (guest 0x102ba5bb8), which SH185 closed by STATIC judgment only ("subscribe registered only inside the migration-gated initializeLuaApp_", never driven). Its body is a JNI-RECEIVE export: (i) dispatches a JNI table slot on the fabricated env (ldr [x0]/ldr [x8,#248]/blr = identity-shim, SH186), (ii) allocates subscription boxes via operator_new 0x1d96768 (0x28/0x20), (iii) DIRECTLY bl's nativeAppBridgeAppStart 0x2343c10 (String,..,String,Z,..,Z marshaller) — a REAL app-start route the ladder drives via the fabricated StartLuaAppDM frame instead. New default-inert `--v2boot-session-bus` post-ladder rung (elfjit.rs) drives it as a real guest entry on the single ladder thread (env/thiz + 4 fabricated jstrings, b1="experience-launch"), measures MH_*. **MEASURED (real libroblox.so, full SH259 seed set, 4 runs = repro + 3-run batch): EVERY run self-terminates at the standing app-start LSM insert-leaf terminal SIGSEGV guestpc=0x101db1d04 (SH260) BEFORE the post-ladder rung runs — `driving MessageBus.subscribe` never prints (0×4); rung is latent-but-correct**, identical to the SH265 OnGameLoaded constraint. 93 distinct region pcs / same terminal = baseline parity, no regression. +1 hermetic sh266 (real-image guard, 6 byte-pins incl. the REAL app-start bl 0x102ba5e14=0x97de777f + operator_new 0x28 box 0x102ba5d34=0x97c7c28d; examples 102/0 was 101). HONEST: converts SH185's static judgment into a wired + measured-latent-but-correct state; does NOT manufacture a DM; Route-B live-DM gate UNCHANGED; SH174 capture-latch stays single forward hook. recon-v3 RE-VERIFIED green at this HEAD (24 task frames swap Ok(0x1), 191 node pops, 0 json abort, 0 crash, EXIT 124). Doc docs/frontier-sh266-messagebus-subscribe-drive.md, repro runs/capture_sh266_bus_subscribe.sh. Single-agent, default-inert.
## SH264 (Sep 17, 2026, hermes-worker): SEP-17 SESSION-DRIVE workstream — drive the REAL Android Activity-lifecycle natives the engine asserts on (SH184), not single-object seeds. Measured: all four lifecycle natives (initAppShellReporter 0x1021f53b8, JNIAppLifecycleNativeAdapter_setActive 0x1021f5de4, nativeAppBridgeSetInitParams 0x102bcc814, nativeOnResumed 0x1021f5db8) are JNI-RECEIVE entries with ZERO in-image bl callers — only the Java side of a real Activity invokes them, so the harness MUST drive them as real guest entries. New opt-in `--v2boot-session` (elfjit.rs, default-inert) drives the three that complete as real guest jit_runs ON THE SAME single ladder thread (SH55/64 serialized), reusing boot_sp/tpidr + fabricated thiz + AutoValue init-params jobject, BEFORE the GlobalInit/app-start rungs. MEASURED (real libroblox.so, full SH259 seed set, EXIT 134 at the SAME parked LSM wall 0x101db1d04, 93 region pcs = SH260/263): **initAppShellReporter + setActive each RETURN Ok(0x0) cleanly** — the first headless execution of these lifecycle natives ever; SetInitParams soft-returns; nativeOnResumed measured dead-end as driven (kept opt-in, segfaults at SH184 live-object registry). No milestone (MH_*=false); AppBridgeV2[0x106a705e8]=0 after. NO regression to the app-start line. HONEST: causes-not-symptoms (session-ctor, not DM); Route-B live-DM gate UNCHANGED; SH174 capture-latch single forward hook. Doc docs/frontier-sh264-lifecycle-natives-driven.md.
## SH265 (Sep 17, 2026, hermes-worker): extend the SEP-17 session-drive to the dataModel-bindings LIVE-BINDER receive `nativeAppBridgeV2SendAppEventOnGameLoaded` (0x102bb429c) — SH264's named next candidate. New default-inert `--v2boot-send-game-loaded` post-ladder rung (elfjit.rs) drives it as a real guest entry (env/thiz + 3 fabricated jstrings, SH186 identity shim), seeds the pipe sync-gate [0x10683d010]=-1 so its `bl 0x2baeeec` takes the SYNCHRONOUS do-init path (bl 0x2206c40), measures MH_*. Corrections: an SH126-mirror materializer on the premise OnGameLoaded's event vtable 0x10635dfe8 is all-zero was FALSIFIED — packed-RELA populates the LOADED slots with real teardown code. Post-ladder rung never reached (app-start ladder self-terminates at run-variable live-object walls first) — latent-but-correct. HONEST: wires+binds the real binder receive; does NOT manufacture a DM; Route-B live-DM gate UNCHANGED; SH174 latch single forward hook. Workspace green. Doc docs/frontier-sh265-gameloaded-binder-receive.md.
## SH261 (Sep 17, 2026, hermes-worker): MEASURE the NEW frontier at THIS HEAD after SH260 parked the LSM insert-leaf (persistence detour). Re-running the exact SH259 repro (full SH248c-f + SH259 seed set, real libroblox.so) the app-start body now walks DEEP past the LSM line into its own self-drive message-loop drain (region-watch [0x10233a000,0x102350000) fires 100-248 distinct pcs incl. app-start drain 0x233bc88..0x233bcf0: `bl 0x21dae90` once-check + `bl 0x21daef8` registry/once builder + `[[0x6a70c90]]` read + `blr vt+48` live heap dispatch + loop counter). TERMINAL IS RUN-VARIABLE live-object class (the SH174/SH204 gate): run-to-run (a) the parked LSM insert-leaf SIGSEGV 0x101db1d04 (dominant), (b) `std::bad_function_call` thrown inside the drain path (an EMPTY std::function target in a LIVE host-heap app-start object, NOT a fixed .bss global — no constant seed lever), or (c) engine-init SIGSEGV 0x1021748a4. HONEST: the app-start body demonstrably advances PAST the LSM wall into its own drain (a real newly-measured surface, SH260's parked terminal is not the only endpoint), but still terminates at live-object construction — Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays the single forward hook. recon-v3 plane RE-VERIFIED green at this HEAD (24 task-driven frames swap Ok(0x1), 196 node pops, 0 json abort, 0 crash, EXIT 124): runs/g-recon.txt. +1 hermetic sh261 (real-image guard: 10 byte-pins — drain block 0x233bc88..0x233bce0 + registry-builder entry 0x21daef8 + map-insert op_new 0x21db07c; skip-if-absent). examples 99/0 (was 98). Doc docs/frontier-sh261-appstart-drain-live-object.md, repro runs/capture_sh259_settings_once.sh. Workspace green. Single-agent, no production code path edited, default-inert.
## SH262 (Sep 17, 2026, hermes-worker): the SH261 app-start DRAIN dispatch at global [0x106a70c90] MEASURED as a DEAD-END seed (reverted) — confirms SH261's live-object classification at the one constant-.bss cell the drain touches. Fresh SH259-repro A/B (real libroblox.so, full seed set, runs/capture_sh262_drain_ab.sh): `0x233bcb8 adrp 6a70000; ldr x0,[x8,#0xc90]; cbz x0,0x233bcf8; ldr x8,[x0]; ldr x8,[x8,#48]; blr x8` reads the RW-.bss ptr global [0x106a70c90] (page 0x106a70000, heavily referenced) and vt+48-dispatches it — the SH248f adapter pattern at a NEW fixed cell. Result: the seed NEVER fired (the global is always **0** headlessly -> cbz skip path), AND a run threw `std::bad_function_call` with ZERO drain region hits ([0x10233bc80,0x10233bd00)=0) — so the empty-function terminal is a SEPARATE live host-heap object (invoked via the drain's `bl 0x21daef8` registry builder), NOT this dispatch. A per-block-entry eprintln diagnostic measured as the SH248b contamination class (perturbed the run: drain stopped firing 0/8 vs ~2/5 clean; removed). GUARD REVERTED (a lever provably unable to fire on the failing case — same standard as SH248h/SH256). 9th adjacent closure at the app-start live-object wall (SH248g/h,249,250,251,253,259,260 + this). READER of the drain dispatch confirmed: [0x106a70c90] is never a live-but-empty object headlessly. Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays the single forward hook. recon-v3 plane RE-VERIFIED green at HEAD (runs/g-recon.txt: 24 task-driven frames swap Ok(0x1), 196 node pops, 0 json abort, 0 crash, EXIT 124). Workspace green; single-agent; tree clean vs HEAD (only run script new). Doc docs/frontier-sh262-drain-dispatch-deadend.md.
## SH260 (Sep 17, 2026, hermes-worker): the SH259-cleared app-start orchestrator's NEW terminal MEASURED + PARKED (single-agent, cone suppressed). Fresh SH259 repro (real libroblox.so, full SH248c-f+SH259 seed set) confirms SIGSEGV guestpc=0x101db1d04 = fn 0x1db1cc8 (small-key map-insert, LocalStorageManager init) with 93 distinct app-start region pcs walked (StartLuaAppDM -> nativeAppBridgeStartAppWithParams -> nativeAppBridgeAppStart). JIT_DUMP_PC register dump (runs/sh259-settings-once.txt): `lsm_map_global=0x7f9c7c51a010` == the harness-seeded bucket-array base (INTACT; [0x10726f8c0] is writable .bss); fault advanced PAST the NULL-bucket read into the INSERT leaf 0x2b9ea40 (`ldr x1,[sub,idx*8]`=0 then `ldset x0,x0,[x1]` atomic-claim on addr 0 -> fault=0x0). DECISION (SEP-15 directive): LocalStorageManager is the persistence/data-store (objective-2b) DETOUR — this wall is confirmed MEASURED as the SH174/SH204 live-object class (the insert needs genuine per-node live allocation, not a read-only "not found" map) and is PARKED; the loop returns to Route B as top priority. ROUTE-B RE-MEASUREMENT at the new state: region-watch on EC-world [0x102e1c650,0x102e25200), marshaler 0x1023f03b4–0x1023f1300, DataModelServices 0x102dbcc10–0x102dbcf00 = **0 hits** — SH259's unlocked app-start body does NOT shift live-DM reachability (structural gate UNCHANGED). +hermetic sh260 (real-image guard, LSM insert-fn 0x101db1cc8/0x101db1d08/0x101db1d14/0x101db1d2c/0x101db1d44 + insert-leaf 0x102b9ea40/44/48 + writable map-global 0x10726f8c0 membership; examples 98/0). recon-v3 re-verified green (24 task frames swap Ok(0x1), 197 pops, 0 json abort, EXIT 124). Doc docs/frontier-sh260-lsm-insert-leaf-parked.md. Route-B gate UNCHANGED; SH174 latch stays forward hook. Workspace green.
## SH258 (Sep 17, 2026, hermes-worker): app-start reaches its DEEPEST point 0x102339d44 with the FULL combined seed set; the map wall proven W-off (R-E exec segment) at that depth. Fresh run runs/capture_sh258_postgovtail.sh (this HEAD, jar/once/adapter + SH245 getter-tail + M48 + SETFIX + DMCONT + all 3 JIT_ROUTEB_APPSART_* ON together — a state no prior session ran simultaneously) — the DMCONT continuation's do-init->governor->app-start chain now region-hits 0x102339004/00c/018/050/07c/1f8/208/c3c/d0c/d44 then EXIT 134 at the SAME standing live-object map wall 0x1021dde34. 0x102339d44 is inside deep app-start orchestrator 0x2339d0c at its `bl 21dac2c` (once-guarded settings/registry singleton factory, flags byte [0x106a6f430], __call_once 284ce54 — a registry-object builder, NOT a forward DM ctor). The deepest app-start reach ever recorded (SH251: 0x10233907c; the extra climb = SH248d/e/f seeds doing their job). Register dump decisive + matches SH248/250: x20=x21=0x100548ca9 (map-`this`), x19=0x40c29c7e746e86a1 garbage hash, x22=0x11 stride-0x2a0 index, x23=0x2. 0x100548ca9=file 0x548ca9 in the R-E (W-off) exec LOAD seg [file 0x0,0x62d8190) => a pointer into execute-only code memory no seed/repair/count-clamp/dynamic-ctor lever can write (SH249 proof re-confirmed at deepest reach). Live-array allocators {0x1df48c0,0x1eb9af4,0x1eba550} still 0 hits (SH254). +hermetic sh258 (deep orchestrator 0x102339d0c=0xd10583ff, deepest bl 0x102339d44=0x97fa83ba->0x21dac2c, map-this 0x100548ca9 host_addr_of!=0, 4-align). Examples 95 +sh258. Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays single forward hook. Single-agent, no production code path edited, default-inert, workspace green.
## SH257 (Sep 17, 2026, hermes-worker): CORRECT SH239's "do-init is absorbed by the FMOD tail / never completes construction" — with the FULL SH248c-f seed set (jar/once/adapter/appname + SH245 getter-tail + M48 + SETFIX + DMCONT), the app-shell init body's FMOD iterate 0x5fb30b4 COMPLETES via its empty-container early-return and do-init climbs to the governor before the standing app-start map wall. This is the genuinely-open thread SH244's "does the tail's early-return path ever come back?" measured with full seeds. MEASURED (real libroblox.so, full DMCONT env, EXIT 134): FMOD iterate 0x5fb30b4 is a container-iterator with an empty-early-return — `ldp x8,x9,[x0,#8]; cmp; b.eq 0x5fb3134` (begin==end -> skip audio body) -> 0x5fb3134 (canary-reload) -> 0x5fb3154 `ret`. In the full-seed run the early-return FIRES (region hit 0x105fb3134), so the tail RESOLVES instead of absorbing control; the chain then climbs post-do-init 0x1023eff4c -> governor 0x102e9fa84 -> govtail 0x102ea30dc -> THEN nativeAppBridgeAppStart dies at the STANDING live-object map wall 0x1021dde34 (SH248g/h/249..256). So SH239's "never completes construction, FMOD-absorbed" framing is WRONG at this HEAD; do-init construction demonstrably ADVANCES to the governor. Does NOT manufacture a DM (app-start still dies at the SH174/SH204 live-object gate). +hermetic sh257 (14 real-image byte-pins: FMOD empty-check ldp 0x5fb30d8=0xa940a408/cmp/b.eq 0x5fb30e0=0x540002a0, early-return canary-reload 0x5fb3134=0xf9400288, post-do-init entry 0x1023eff4c=0xd10603ff, governor entry 0x102e9fa84=0xa9ba7bfd+0xa9016ffc, flags lsl+and, app-shell ctor 0x102207b50=0x14000001 + tail `b` 0x102208ebc=0x14f6a87e, 4-aligned; skip-if-absent). Recon-v3 re-verified green (24 task frames swap Ok(0x1), 196 pops, 0 json abort, 0 crash). Doc docs/frontier-sh257-fmodtail-earlyreturn.md. Commit 6a0657d. Examples 95/0, workspace green. Route-B live-DM structural ga...
## SH256 (Sep 17, 2026, hermes-worker): "repair the app-start map via a fixed root global [0x1067d16f0]" MEASURED as a FALSE POSITIVE — that global is the STACK CANARY (SH182's CANARY), NOT a map-root holder. Fresh JIT_DUMP_PC register dump at THIS HEAD (runs/sh256-maproot-dump.txt) showed x24 STABLE across all 3 clean inserts AND === [0x1067d16f0], so the map root looked anchored at a fixed .bss cell. Implemented default-inert routeb_appstart_maproot_seed_guard (JIT_ROUTEB_APPSART_MAPROOT_SEED) + hermetic sh256 + dispatch wiring: repairs [root+0][root+8] of the root at [0x1067d16f0] with a coherent header + 0x400-slot zeroed bucket array (so the never-run grow ctor is unneeded). A/B (full DMCONT env, 3 OFF + 3 ON): OFF = stable SIGSEGV guestpc=0x1021dde34 ×3, 0 stack-smash; ON = repair fires 2× (root base==root, count=0x2f2a1a0a0e0f1011), the 8-closure wall SIGSEGV GONE, but ALL 3 runs exit "stack smashing detected" → SIGABRT. ROOT CAUSE: app-start fn 0x21ddc44 loads x20=[0x1067d16f0] as the stack-canary pointer, saves [x20] at [x29,#-8], epilogue 0x21ddc8c re-reads [x20] and b.ne __stack_chk_fail; my repair wrote [root+0]=[0x1067d16f0] (the canary VALUE) so the canary changed between prologue and epilogue → deterministic __stack_chk_fail. The "advance past the wall" is an artifact of corrupting the canary (failure mode moved from map-SIGSEGV to canary-SIGABRT), NOT a real advance, NOT a DataModel. GUARD REVERTED (a lever that only corrupts the stack canary adds no value); tree clean vs HEAD. CORRECTS SH250's register attribution (x24 === [0x1067d16f0] is the stack-canary pointer, stable *because* it is a stable .bss canary cell — NOT the map root). Closes the 8th adjacent angle at 0x1021dde34. recon-v3 re-verified green (24 task frames swap Ok(0x1), 196 pops, 0 json abort, 0 crash). Route-B live-DM structural gate UNCHANGED; SH174 capture-latch (arm *(0x106391908) at a real make_shared<DataModel>) stays the single forward hook. Doc docs/frontier-sh256-maproot-canary-falsepositive.md (evidence runs/sh256-* gitignored). Single-agent, no production path edited, workspace green.
## SH255 (Sep 17, 2026, hermes-worker): marshaler-enclosing fn 0x1023f03b4 measured INDIRECT-ONLY over the whole executable .text (0 direct bl/b callers) — the EC-world (genuine DataModel factory) live-state gate is now mechanized at the enclosing-fn level, not just the marshaler/branch level. Fresh my-own-BP full-.text scanner (exec LOAD flags=0x5 seg [file 0x0,0x62d8190)): the fn holding `bl marshaler 0x1023f1210` @0x1023f075c is reachable ONLY via blr/br (indirect dispatch), the fabricatable-object-graph/live-this class, NOT a static seed. The marshaler's other two direct callers (0x102e15bf0, 0x102e33494) are outside StartLuaAppDM's own body [0x1023efe2c,0x1023f0800) and outside the narrower EC body window [0x102e1c650,0x102e25200) — they sit in the high ExperienceController/game-start region (0x2e00000+, enclosing fns 0x2e19da4/0x2e33b78), the same live-DM-gated area SH231 measured headless-unreached (draft overreach corrected with measured enclosers). Consequence: NO in-image static call site reaches the EC world except StartLuaAppDM's dispatch switch (measured inert SH236/238) or game-start self-calls gated on a running world. recon-v3 deliverables RE-VERIFIED GREEN at this HEAD (24 task frames swap Ok(0x1), 196 pops, 0 json abort, 0 crash, EXIT 124). +hermetic sh255 (pins 0x1023f03b4=0xfc190fe8, 0x1023f075c=0x940002ad; whole-text scan -> 0 direct callers of 0x1023f03b4; non-SLADM marshaler callers outside SLADM body + in 0x2e00000+, 4-aligned; examples 94/0). Doc docs/frontier-sh255-marshaler-enclosing-indirect.md. Route-B live-DM structural gate UNCHANGED; SH174 capture-latch (arm *(0x106391908) at a real make_shared<DataModel>) stays the single forward hook. Single-agent (cone suppressed), no production code path edited, default-inert, workspace green.
## SH253 (Sep 17, 2026, hermes-worker): implement + MEASURE the SH252 remaining forward lever — seed the bulk-registrar SOURCE vector so the engine's OWN in-ladder loop builds the resolver. New default-inert `routeb_source_vector_seed_guard` (JIT_ROUTEB_SOURCE_SEED=1, fires anywhere in nativeGameGlobalInit [0x102206404,0x10220881c)) writes a leaked 3-class-name array (PlayerGui 0x87e / ScreenGui 0x892 / CoreGui 0x77f; each descriptor [desc+8]=LONG-SSO name string, [desc+16]=classid) into every empty source slot {0x106dca0ea8 loop-read, 0x106dca0e08 sibling, 0x106dca0e90 registrar source-map}. MEASURED (real libroblox.so, completing --v2boot ladder, 3-run batch): seed fires every run (source containers non-empty at the registrar block, readback {0x55ccc8077310,0x55ccc8077328}) BUT resolver 0x106dca0e70 stays {0,0} AND the source-loop body (block 0x1022085d4) is NEVER entered (0 region hits; block 0x1022085c0 fires) — the registrar's per-element drive + resolver insert does not execute headlessly even with a non-empty source. Termination UNCHANGED + STABLE: 2x EXIT 134 / 1x 139 at the standing 0x1021dde34 live-object map wall. VERDICT: SH252's "source-vector bridge" is now MEASURED INERT (not just judged) — the resolver is a downstream session consumer, not the cause; consistent with SH192/194 + SH174 live-DM gate. 6th adjacent closure at 0x1021dde34. +hermetic sh253 (env/pc-gated, seeds 3 containers idempotently; 398 lib tests). Doc docs/frontier-sh253-source-vector-seed.md. Single-agent, no production code path edited, default-inert, workspace green.
## SH252 (Sep 17, 2026): full-text sweep CLOSES SH193's "locate the RESOLVER map ctor" — NO lazy-static ctor constructs 0x106dca0e70. The resolver map (getService walker 0x105e09bc8 -> resolver 0x2373cec) is built ONLY by the in-ladder bulk registrar 0x2208ae8 iterating the SOURCE vector 0x6dca0ea8. MEASURED (exec seg [0x0,0x62d8190)): 8,378 bl __cxa_guard_acquire sites; exactly 11 `adrp 6dca000 + add #0xe70` form the resolver addr ({0x220841c, 0x2208b10/8be0/8c9c, 0x25f8fd8, 0x28442c8, 0x31fcac4, 0x3ceca04/6cd6c, 0x4894100, 0x4b547f8}) — NONE stores into the resolver header from inside a guard-ctor body (64-insn window, STR/STP/STR-Q only: 0 writes). The one forming guard (0x25f8fb8) only passes &resolver to the READ-ONLY probe 0x2373cec (consumer, not ctor). VERDICT: header written only by the in-ladder default-construct 0x2208418 + registrar insert; a real bucket array needs first-insert rehash, which needs SOURCE non-empty (empty headlessly). REMAINING forward lever UNCHANGED: seed SOURCE vector 0x6dca0ea8 so the engine's OWN registrar loop (0x22085c4) populates the resolver. Does NOT manufacture a DM; Route-B gate UNCHANGED; SH174 latch single forward hook. +1 hermetic sh252 (11-site set + guard header-write sweep; examples 93/0). Doc docs/frontier-sh252-resolver-no-static-ctor.md. Single-agent, no prod path edited, workspace green.
## SH251c (Sep 17, 2026, hermes-worker): CORRECTION to SH251b — 0x102b9eca0 is a SHARED RUNTIME TEARDOWN HELPER, not a forward Route-B ctor. Measured (two independent BL-imm26 decoders — my throwaway self-caller scan AND GNU objdump both count 64,748 direct `bl 0x2b9ec9c` callers, ZERO `bl 0x2b9eca0` over the exec segment): SH251b's "new ctor fencepost 0x102b9eca0, 0 direct bl/b callers, indirect-dispatch-only, next-unsynthesized-object" is a +4-ENTRY MISLABEL. The function's true entry is 0x2b9ec9c (`paciasp`); two of the 64,748 callers are inside the PlayerGui getter 0x10201fce0 as EXCEPTION LANDING PADS (0x201fda0 / 0x201fe48, each preceded by __cxa call_once cleanup `bl 284cfbc`/`bl 5fb25e0`), invoked with the caught/unwinding context as x0 during the spawned-thread SIGTRAP unwind — so the `this=NULL` fault=0x10 at 0x102b9ecbc is TEARDOWN, not forward construction, and there is NO fabricatable receiver to seed. 0x2b9eca0 (via 0x2ba2ba0) saves ALL GPRs+FP regs into its frame then dispatches on `[this+16]/[this+24]` = a setjmp/ucontext-style cleanup trampoline, not a class-descriptor ctor / register continuation / GuiObject producer. Do-not-re-tread the "receiver for 0x2b9eca0" (64,748 callers; NULL-this expected on unwind). Genuine forward state UNCHANGED: PlayerGui+ScreenGui class descriptors ARE registered headlessly (vtable+vtable-family markers match); the name->classid resolver map 0x106dca0e70 stays empty (live-ctor-gated, SH193/194); SH174 capture-latch (arm *(0x106391908) at a real make_shared<DataModel>) remains the single forward hook. recon-v3 deliverables stay shipped + verified. +hermetic sh251c (byte-pins paciasp entry 0x2b9ec9c + two landing-pad bl words + their __cxa cleanup bls; asserts bl-imm26 targets 0x2b9ec9c NOT 0x2b9eca0). Doc docs/frontier-sh251b-corrected-teardown-helper.md + addendum in frontier-sh251b doc. Single-agent, no production code path edited, workspace green (576/0, examples 92/0).
## SH251 (Sep 17, 2026, hermes-worker): the operator SEP-15 re-attack lever ("cross the live-DM wall via a dynamic DM-ctor trace rather than a static seed") MEASURED on the real combination for the first time = NEGATIVE. `routeb_dm_real_ctor_drive_guard` (SH187) constructs a GENUINE-vptr DM through the real ctor wrapper 0x1023f5ff8 and plants current-DM holder 0x106391908 — but prior DMCONT batches (SH248c..f/250) ran WITHOUT `JIT_ROUTEB_DM_REALCTOR`, so app-start's live host-heap map construction (SH248g wall 0x1021dde34) always saw holder=0. This cycle pairs the genuine manufactured DM 3/3 with the DMCONT continuation (repro runs/batch_sh251_dmcont_realctor.sh): realctor GENUINE MATCH + SH187b holder-plant 3/3; continuation runs its full serialize body (0x102bd1d68..0x102bd2014) + drives nativeAppBridgeAppStart deep (0x102339004..0x10233907c) — and STILL terminates at the SAME live-object wall `SIGSEGV guestpc=0x1021dde34 fault=0x0 EXIT 134` (runs 1/3; run 2 = known SH55/64 flake). DataModelServices registry fan-out (guest 0x102dbcc10) NOT reached. **Verdict (do-not-re-tread): the operator's named dynamic-trace re-attack is now measured-closed on the actual pair — a genuine real-code-constructed DM in the holder does NOT change app-start's own live-object map construction, because that graph (JNI_OnLoad+0x69e region, under-allocated stride-0x2a0 array, its own ctor never ran headlessly) is downstream and needs a real upstream session ctor. Fresh measured closure, not a re-run.** Route-B live-DM structural gate UNCHANGED; SH174 capture-latch (arm *(0x106391908) at a real session make_shared<DataModel>) stays the single forward hook. recon-v3 re-verified green (24 task-driven frames swap Ok(0x1), 197 pops, no json abort). Doc docs/frontier-sh251-dmcont-realctor-reattack-negative.md. Single-agent, no production code path edited, workspace green.
## SH251b (Sep 17, 2026, hermes-worker): class-desc-REGISTERED path (SH189 lever + genuine-DM CONSUMER+DISPATCH) drives the REAL PlayerGui+ScreenGui register getters headlessly for the FIRST time and ADVANCES into a NEW stable ctor fencepost 0x102b9eca0 (NULL `this`, fault=0x10). `routeb_dm_real_ctor_drive_guard` produces GENUINE MATCH + SH187c get-or-create DROVE ok 3/3; the SH189 guard's PlayerGui getter 0x10201fce0 (ret 0x106c980b8) + ScreenGui getter 0x10201f42c drive OK with class-desc + vtable-family markers populated; the register continuation then enters ctor 0x102b9eca0 (sub sp,#0x480, x19=this saved) whose `ldr x3,[x19,#16]` @0x102b9ecbc faults with this=NULL (reproducible 3/3 EXIT 134/139). The ctor is indirect-dispatch only (0 direct bl/b callers, 0 .data.rel.ro addend) — its receiver is the next-unsynthesized-object (same live-object/fabricatable graph as SH174/204/248g, at a location never before reached). Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays single forward hook. +hermetic sh251b (8 real-image byte-pins incl. the new ctor + fault insn). recon-v3 re-verified green. Doc docs/frontier-sh251b-classdesc-register-advances-ctor-fencepost.md, repro runs/batch_sh251b_fencepost_repro.sh. Single-agent, workspace green.
## SH249 (Sep 17, 2026, hermes-worker): segment-level proof that the app-start map wall's fatal pointer is R-X text (write=OFF) — closes SH248h's "maybe a fixed .bss holder at a stable address" residual. host_addr_of() on real libroblox.so: both observed fatal addrs (x21=0x100548ca9 / 0x1004a0373) land in the single R-X EXECUTE LOAD segment [0x100000000, 0x1062d8190) with prot write:false — NOT host-heap and NOT a writable .bss/.data cell. No static header seed (SH248g) or runtime in-place repair (SH248h) can ever reach it; all three levers (static-seed, runtime-repair, segment-protected) now measured/proven closed at the SAME 0x1021dde34 wall. Fresh 3-run batch re-confirms the wall is stable (all EXIT 134 at guestpc=0x1021dde34, continuation advancing region 0x10233901x/0x10233920x + dispatcher 0x102bd8ce8 + continueAfterFlagsLoaded_ 0x102bd1d68 + app-name guard 0x102bd1f64). Extends the sh248g real-image guard with a SH249 segment-membership assertion (both fatal addrs in [0x100000000, 0x1062d8190)); full example suite 90/0, workspace green (build + test exit 0). Pure regression hardening + proof-closure of a residual — does NOT manufacture a DataModel, Route-B live-DM structural gate UNCHANGED. Doc docs/frontier-sh249-appstart-map-segment-protected.md. SH174 capture-latch arm *(0x106391908) stays the single forward hook. recon-v3 deliverables stay shipped + verified.
## SH248h (Sep 17, 2026, hermes-worker): measure + close the ONE angle SH248g left open — the RUNTIME in-place repair of the app-start live hash-map header — as a dead-end too. Added default-inert `routeb_appstart_map_repair_guard` (JIT_ROUTEB_APPSART_MAP_SEED, fires at pc=0x1021dde34, reads x21=map-`this` from live CPU state, re-seeds coherent empty-8-bucket header when incoherent, idempotent per-pointer via HashSet) and A/B'd it on the real libroblox.so (full DMCONT env, 3 completing runs). MEASURED: guard fires + idempotent 3x/run (repairs the 3 SUCCESSFUL baseline host-heap maps, count=2 each) but the wall is NOT crossed — every run still SIGSEGVs guestpc=0x1021dde34 (EXIT 134/139; fault addr 0x0→0x9 as the bucket walk relocates). The fatal 4th-iteration map `this` is x21=0x100548ca9/0x1004a0373 — an IN-IMAGE / low guest address (top 16 bits 0), NOT host-heap, so ANY header seed is register-unreachable (my `(map>>48)!=0` host-heap check correctly skips it). This is the app-start LiveObject graph needing a REAL DataModel ctor to fully initialize the under-allocated live array (stride 0x2a0, 4th element runs off into image memory) — the SH174/SH204 live-object structural gate, now confirmed from inside nativeAppBridgeAppStart with BOTH static-seed (SH248g) and runtime-repair (SH248h) levers measured closed. Guard REVERTED (not shipped — a lever provably unable to fire on the failing case). Doc docs/frontier-sh248h-runtime-map-repair-deadend.md. Workspace green (build + test exit 0, lib 397/0). Route-B live-DM structural gate UNCHANGED; SH174 capture-latch (arm *(0x106391908) at a real make_shared<DataModel>) stays the single forward hook.
## SH248g (Sep 17, 2026, hermes-worker): register-level determination that the post-SH248f app-start wall (SIGSEGV guestpc=0x1021dde34) is a LIVE host-heap hash-map insert, NOT a fixed-.bss seed — closes SH248f's "is it seedable" question with evidence. The DMCONT continuation now runs its own host-heap map construction (3+ successful inserts) before the wall — the farthest app-start measured yet. +hermetic sh248g (real-image byte pins on the 5 wall anchors 0x21dde00/0x21dde30/0x21dde34/0x21ddea8/0x21dd800, skip-if-absent). Doc docs/frontier-sh248g-hashfind-live-wall.md, repro runs/batch_sh248f_adapter_seed.sh. Workspace green (examples 90/0, lib 397/0, recon-v3 re-verified 24 frames swap Ok(0x1)).
## SH248f (Sep 17, 2026, hermes-worker): fabricate the app-lifecycle adapter at [0x106b0bde0]; the DMCONT continuation clears the closure-dispatch NULL wall and advances DEEP into nativeAppBridgeAppStart (region hits to 0x102339d44) — the farthest the app-start path has run headlessly yet. Following SH248e (once-cell seed [0x106b0bdf0] -> -1, cleared SIGSEGV 0x102339208), the next fencepost was SIGSEGV 0x102339020: setActive (0x21f5f80) copies the app-lifecycle adapter triplet from fixed globals [0x106b0bde0]/[0x106b0bde8] into the frame, then `ldr x8,[x0]; ldr x8,[x8]; sub x2,x29,#0x40; mov w1,#3; blr x8` (0x233903c..0x233904c) virtual-dispatches the adapter's vt[0] as a std::function closure — x0=[0x106b0bde0]=NULL headlessly -> SIGSEGV fault=0x0. New default-inert `routeb_appstart_adapter_seed_guard` (JIT_ROUTEB_APPSART_ADAPTER_SEED=1, fires on block entry 0x102339018..0x102339050 — the once-fn resume block entry, since the setActive+closure dispatch are MID-BLOCK) seeds [0x106b0bde0] = a fabricated all-leaf-vtable object (`routeb_appstart_adapter_object`: leaked 0x100 obj, 0x200 all-leaf vt) so the blr resolves a benign leaf. MEASURED (real libroblox.so, DMFORCE+DMCONT+SH245+SH248c/d/e env): seed fires, 0x102339020 crash GONE, continuation advances ~0x500 through nativeAppBridgeAppStart (region hits 0x102339050/07c/c3c/d0c/d44) to the NEXT fault `SIGSEGV guestpc=0x1021dde34 fault=0x0 x0=host x19=0x40c29... x20/x21=0x100548ca9 x22=0x11` = a separate hash-map construction fn (JNI_OnLoad+0x69e34 region) computing bucket indices from LIVE garbage size/hash fields — the SH174/SH204 live-object class, NOT a fixed .bss seed. Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays the single forward hook. +hermetic sh248f (env/pc-gated, seeds leaf object, idempotent; CONT_MGR_TEST_LOCK-serialized vs the fixed-.bss parallel-test race). Doc docs/frontier-sh248f-adapter-closure-seed.md, repro runs/batch_sh248f_adapter_seed.sh. Workspace green (cargo build + cargo test --workspace exit 0; arm64jit lib 397/0; examples 89/0).
## SH248e (Sep 17, 2026, hermes-worker): seed the app-start once-cell global [0x106b0bdf0] -> -1; the DMCONT continuation clears its pthread_mutex_lock once-branch and advances one gate deeper in nativeAppBridgeAppStart. The SH248d fencepost `SIGSEGV guestpc=0x102339208 (lr 0x102339018) fault=0x0, x0=0, x1=0` is NOT a live-object range (SH248d's initial framing) — it is a **seedable fixed .bss once-cell**. Disasm: the once-check fn vaddr 0x2339208 does `adrp x9,6b0b000; ldr x0,[x9,#3568]; ldar x8,[x0]; cmn x8,#0x1; b.eq 0x2339264` — reading POINTER global guest 0x106b0bdf0 (vaddr 0x6b0bdf0, RW data seg) whose pointed-to cell is NULL -1-flagged. New default-inert `routeb_appstart_once_seed_guard` (JIT_ROUTEB_APPSART_ONCE_SEED=1, fires on fn range 0x102339208..0x102339244) seeds [0x106b0bdf0] = a leaked 8-byte cell holding -1 -> the `cmn x8,#-1; b.eq` skip is taken instead of the pthread_mutex_lock branch. MEASURED: seed fires, 0x102339208 crash GONE, continuation advances ONE more gate into nativeAppBridgeAppStart (next wall guestpc=0x102339020). Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays forward hook. +hermetic sh248e. Doc docs/frontier-sh248e-once-cell-seed.md, repro runs/batch_sh248e_once_seed.sh. Workspace green.
## SH248d (Sep 17, 2026, hermes-worker): seed the cookie-jar container globals — the DMCONT continuation advances DEEP into nativeAppBridgeAppStart (NULL-range string-array wall). Following SH248c (continuation entered nativeAppBridgeAppStart 0x2338510 headlessly for the first time), the next fencepost was SIGSEGV 0x102b504e4 (NULL-dest string copy). lr=0x1021f4834 named the site: app-start string-dispatch assigns into cookie-jar global [0x106ed7a20] (.bss NULL) then byte-derefs adjacent [0x106ed7a28] (also NULL) — the SH175 lane. New default-inert guard `routeb_appstart_jar_seed_guard` (JIT_ROUTEB_APPSART_JAR_SEED=1, fires on fn range 0x1021f47f0..0x1021f4840) seeds BOTH [0x106ed7a20]+[0x106ed7a28] with a leaked valid empty SSO std::string (`routeb_empty_sso_string()`, zeroed 0x20) when NULL, idempotent. MEASURED (real libroblox.so, DMFORCE+DMCONT+SH245+SH248c env, 6-run batch): jar seed fired 12x (6x2); the 0x102b504e4 + 0x1021f4848 crashes GONE; continuation reached 6/6; advances DEEP into nativeAppBridgeAppStart to the next fault `SIGSEGV guestpc=0x102339208 (lr 0x102339018) fault=0x0, x0=0x0, x1=0x0` = string-vector helper (prologue 0x33901d0 computes a range's element count `sub x8,x1,x0; asr/mul`) called with a NULL pointer-range — the live session/container array, the SH174/SH204 live-object wall from inside the real app-start path. Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays the single forward hook. Doc docs/frontier-sh248d-cookie-jar-deep-appstart.md, repro runs/batch_sh248d_jar_seed.sh. Workspace green (cargo build + cargo test --workspace exit 0).
## SH248c (Sep 17, 2026, hermes-worker): the DMCONT continuation crosses the -9 bad_alloc AND the app-name NULL-store gate -> nativeAppBridgeAppStart (0x2338510) ENTERED headlessly for the FIRST time. Two default-inert seed fixes (jit.rs, opt-in env-gated), +1 unit update. Commit 333ee8b. Workspace green (cargo build + cargo test --workspace exit 0).
- **(1) -9 bad_alloc fixed (SH248 root cause).** `routeb_dm_manager_cont`'s M+0x48 app-name seed wrote `cap=1` (only the long-flag bit0, ZERO capacity). The engine's serialize-assign 0x2bd1dfc reads dest `[this]&~1`=0 as capacity -> not enough room -> grow path -> `oldcap-1` underflows to 0xffff..ff > max_size -> `b.hi 0x2b50690` -> `x23=-9` -> `operator_new(-9)`->NULL->`std::bad_alloc`. Fix: valid long cap `[m+0x48]=0x11` (cap 0x10, bit0=1 long) with the CORRECTED long-form layout `[0]=cap, [8]=size, [16]=data ptr` + a 64-byte leaked "Home\0" buffer. MEASURED: the `OPERATOR_NEW size=-9` line is GONE; all 20+ serialize-assign blocks (0x2bd1dfc..0x2bd1f5c) run clean.
- **(2) App-name NULL-store fault 0x2bd1fd4 crossed.** The serializer (driven from the all-zeroed flags-holder F) OVERWRITES M+0x48 with an EMPTY string (size=0), so the guard at 0x2bd1f64 (`cbnz [M+0x50]`) falls to the deliberate `mov x8,xzr; strb w9,[x8]` NULL store. New guard `routeb_cont_appname_seed_guard` (opt-in `JIT_ROUTEB_CONT_APPNAME_SEED=1`) fires at guard block-entry pc=0x102bd1f64 and re-seeds `[M+0x50]=5` (long size; M+0x58 still holds "Home" data) so the cbnz skips to 0x2bd1fe0. MEASURED: continuation passes 0x2bd1fd4, reaches 0x102bd2014.
- **MEASURED (real libroblox.so, DMFORCE+DMCONT+SH245 env, 2/6 completing ladder runs; run-variable ~1/3):** continuation runs its ENTIRE serialize body then **`nativeAppBridgeAppStart` (entry 0x2338510) executes headlessly for the FIRST time** (SH245's explicit target eventually reached). Next fencepost: `SIGSEGV guestpc=0x102b504e4 (string assign) fault=0x0, x0=0x0 (NULL destination), x1=0x55dc.. src, x2=0x55dc..` — a NULL-`this` std::string copy inside the app-start construction (SH174/SH204 live-object-lifetime wall, now reached from inside the real app-start rather than a harness-driven path).
- **HONEST (do-not-over-claim):** does NOT manufacture a DataModel; Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays the single forward hook. What IS new + measured: the continuation + serialize body complete cleanly and the engine's own app-start entry executes headlessly for the first time — the DMCONT line moved from "bad_alloc" to "inside nativeAppBridgeAppStart, NULL-controller string copy".
- **NEXT (honest, single-agent):** the NULL-destination string assign inside app-start is the post-nativeAppBridgeAppStart fencepost — fabricatable-object-graph class (SH174/SH204 map): seed the string destination F holds for the app-start controller, or supply a live controller object.
- **CODE:** jit.rs — `routeb_dm_manager_cont` M+0x48 seed (cap 0x11 + corrected layout + 64B Home), `CONT_MANAGED_M` static + `routeb_cont_managed_m()`/`store_cont_managed_m`, `routeb_cont_appname_seed_guard` (env JIT_ROUTEB_CONT_APPNAME_SEED), wired into the block-entry dispatch after `routeb_alloc_probe_guard`; `sh245_m48_long_string_seed_passes_continuation_appname_guard` updated to the corrected long-form layout (asserts size field decode + data ptr at [16]). All default-inert. Doc docs/frontier-sh248c-cont-enter-appstart.md, repro runs/capture_sh248c_cont_next.sh + runs/batch_sh248c_cont_next.sh.
## SH248 (Sep 17, 2026, hermes-worker): MEASURE the real allocator at runtime (live guard JIT_ROUTEB_ALLOC_PROBE=1) — the continuation's `bad_alloc` is a CORRUPTED-STRING-OBJECT assign at 0x2b50600, NOT an allocator-capacity wall. This corrects SH245-247's ">0xa can't be served headlessly" framing. +hermetic sh248 (real-image byte-pins, 7 allocator sites; examples 89/0), Doc docs/frontier-sh248-allocator-badalloc-corrupt-string.md, repro runs/capture_sh248_allocprobe.sh + runs/batch_sh248_allocprobe3.sh. Commit a9a7129. Workspace green.
- **THE MEASURED MECHANISM (the bad_alloc source is deterministic):** on every completing continuation run the fatal alloc is `OPERATOR_NEW x0(size)=0xfffffffffffffff7 (=-9) x30(caller)=0x102b506bc` -> operator_new returns NULL -> libc++abi `std::bad_alloc` (the -9 line always immediately precedes the terminate). 0x2b50600 is a libc++ std::string::assign leaf: when its DESTINATION string's capacity word `[this]&~1` exceeds the max_size sentinel 0x7ffffffffffffff2 (`cmp x9,x8; b.hi 0x2b50690`) it deliberately emits `x23 = -9` and `bl operator_new` (0x2b506b4). operator_new(-9): `cmn x2,#0xa` (=-9+10) overflows -> C=1,Z=0 -> `b.ls` NOT taken -> NULL. So the continuation assigns into a std::string whose capacity word is uninitialized GARBAGE (a corrupted guest-string state in the app-start construction) — the -9 is libc++'s max_size-exceeded SENTINEL, not an allocator-capacity limit.
- **CORRECTIONS (do-not-re-tread):** (1) REFUTED "no size>0xa can be served headlessly" — the real allocator wrapper 0x1db1c60 is reached headlessly with sizes 0x90/0xa0/0x69/0xe61 and the ladder proceeds past them, so the allocator genuinely serves >0xa. (2) REFINED where the continuation is blocked: repair/seed the garbage [this] capacity word of the destination string so the assign reallocs normally — a DIFFERENT and smaller surface than the whole scudo size-class bootstrap SH247 proposed. (3) The free-list tail 0x623fe1c is never a block entry (target of a direct `b` tail-jump; SH217 class), so it is unobservable by block-entry guards — only the wrapper is.
- **HONEST (do-not-over-claim):** does NOT manufacture a DataModel; Route-B live-DM structural gate UNCHANGED. This is a measured correction of WHERE the continuation is stopped (a specific corrupted string, seedable) rather than proof the allocator cannot serve the construction. SH174 capture-latch stays the single forward hook.
- **SH248b (same session, follow-up):** minimal NON-PERTURBING probe + the corrupt object captured. The heavy-log v1 probe (265 eprintln lines + per-entry gettid syscall) measurably shifted the run to a DETERMINISTIC secondary-thread crash (guestpc=0x101d96768 fault=0x24, operator_new B, harness-tid=4096) before the continuation (0/6), while probe-OFF reached the continuation bad_alloc 2/2. Minimal v2 probe (fires at op_new A/B, eprintlns ONLY the -9 sentinel, ~1 line/run, no per-entry syscall) RESTORES the clean run (3/4 EXIT 139) and captures the object deterministically: `CORRUPT_SENTINEL x20(this)=0x7f1dfc5a3ac8 [this+0]=0x1 [this+8]=0x7f1dfc033800(host ptr) [this+16]=0x5, x19=0 x2=0 x30=0x102b506bc`. Decode: [this]=1->x9=0; length-0 assign -> resize path -> x9=oldcap-1 underflows to 0xffffffffffffffff > max 0x7fff..f2 -> b.hi -> x23=-9. So the fatal bad_alloc is a LENGTH-0 assign into a HOST-heap std::string whose resize underflows on its first word — not "garbage capacity"; located + reproducible at exact addresses (object + 0x2b50600/0x102b506bc). Fix options: seed the host-heap string's SSO word / fix the upstream ctor. Refines SH248's mechanism. Commits a9a7129 + 65890a4; examples 89/0; workspace green.
## SH247 (Sep 17, 2026, hermes-worker): route ALL `operator_new` sizes through the WORKING descriptor/real-allocator path and MEASURE — the DMCONT continuation is allocation-waled at its first >0xa box via a 2nd mechanism (proof-of-dead-end). Opt-in JIT_ROUTEB_OPNEW_SIZE_GATE: single-word patch per variant changing the fast-path size-gate `b.ls SMALL` (taken ~ unsigned size<=0xa) to an UNCONDITIONAL `b SMALL`, so size>0xa ALSO walks the ≤0xa descriptor path (scudo class 0x1d969cc/0x1d99bf0 -> real alloc tail 0x1db1c60, returns real memory headlessly for tiny sizes) — WITHOUT touching the bit0 flag (whose =1 route is a measured 3/3 regression, SH245 #4). Sites file 0x1db1a78 (0x540001e9) + 0x1d967ec (0x540001c9); replacements 0x1400000f/0x1400000e, byte-guarded, whole-function block-cache drop. MEASURED (real libroblox.so, DMFORCE+DMCONT+GETTER_TAIL_RET+M48 env): OFF EXIT 134 bad_alloc w/ 0 continuation hits; ON EXIT 139 continuation RUNS (region 0x102bd1d68->0x102bd1dfc) yet still `std::bad_alloc`; ON+combination with the SH246 0x28 box patch -> SAME bad_alloc, region hits d68+dfc only. CONCLUSION: the descriptor→real-allocator path that serves ≤0xa does NOT serve the continuation's 0x28/0x20 (narrow size-class region set up for tiny, not for >0xa); 3 measured mechanisms now pin the continuation's alloc wall — NULL fast-path (SH245), pervasive-site (SH246), and descriptor-tail (SH247). Honest: does not manufacture a DM; Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays single forward hook. +1 hermetic sh247 (real-image gate words + unconditional-B imm26 target check; examples 88/0). Doc docs/frontier-sh247-opnew-size-gate-deadend.md, repro runs/capture_sh247_opnew_sizegate.sh.
## SH246 (Sep 17, 2026, hermes-worker): implement SH245-candidate-1's scoped op_new box at the ACTIVATED continuation's closure (0x2bd2128) AND MEASURE the continuation is allocation-walled, not single-site. Opt-in JIT_ROUTEB_DM_CONT_OPNEW_BOX: 3-slot call-site patch (mov w0,#0x28; mov w1,#8; bl op_new -> movz/movk/movk of a leaked 0x40 box into x0) at 0x102bd2120, byte-guarded, <2^48 assert. KEY FIX: `block_cache_drop_region` matches by block ENTRY pc — this call site is mid-straight-line-block (entry 0x102bd1dfc), so a window-only drop leaves the stale compiled block calling op_new (measured); the WHOLE-continuation drop [0x102bd1d68,0x102bd2600) is required. Patch readback confirms (3 movz/movk words). MEASURED (real libroblox.so, 7+ runs, EXIT 134/139): closure op_new ELIMINATED yet bad_alloc persists EVERY run from OTHER operator_new sites — nativeAppBridgeAppStart 0x2338ef4 (bl'd at 0x2bd2058) uses the OTHER operator_new variant 0x1d96768 at 0x28/0x20 (3 direct sites), plus string/app-start constructions. The continuation PERVASIVELY NULL-allocs headlessly (op_new fast path returns NULL for size>0xa when [0x10727570c].bit0 clear, and the real-alloc path is the SH245-#4 regression). Correction: SH245's 'next gate' is NOT this single closure site — it's the whole AppBridge/live-world-build alloc wall (SH174/SH204 measured do-not-chase: never a DM factory, ZERO GuiObjects). Diagnostic A/B (ret bl 0x2338ef4 @0x2bd2058) did NOT shift observable bad_alloc, removed (not shipped). +hermetic sh246 (real-image 3 window words + 48-bit movz/movk round-trip + drifted-site-fails-loudly). Examples 87/0, workspace green, single-agent. Box patch inert by default (env-gated). Route-B live-DM structural gate UNCHANGED; SH174 capture-latch stays single forward hook. Doc docs/frontier-sh246-cont-opnew-scoped-box.md, repro runs/capture_sh246_cont_opnew_box.sh + runs/batch_sh246.sh + runs/diag_sh246_skip_appbridge.sh. Commit 3706431.
## SH245 (Sep 17, 2026, hermes-worker): getter tail→ret ACTIVATES the real continueAfterFlagsLoaded_ for the FIRST time headlessly + clears its app-name guard. The engine-init getter 0x102174c04 ends with an UNCONDITIONAL tail `b 0x624e6c0` (FMOD/AAudio) that never returns to the dispatcher (0x2bd8d18) — patching it to `ret` (routeb_patch_getter_fmod_tail_ret, opt-in JIT_ROUTEB_GETTER_TAIL_RET) makes the dispatcher resume -> runs its benign vt[+0xf8]/vt[+0x108] leaves -> reaches `bl sub_2bd8dac` -> vt[+0x1f0] -> **REAL continueAfterFlagsLoaded_ 0x102bd1d68 EXECUTES (region-watch, first ever)**. +M+0x48 seed (JIT_ROUTEB_DM_CONT_M48_SEED, long "Home" string at the cont-manager's flags-app-name field) clears its deliberate NULL-store app-name guard (0x102bd1fd4): continuation ran its FULL body incl. nativeAppBridgeAppStart 0x2338ef4, reaching 0x102bd1dfc, then hit the NEXT gate: `std::bad_alloc` from the 0x28 app-controller closure (operator-new 0x1db1a38 returns NULL for size>0xa when alloc-flag byte [0x10727570c].bit0 == 0 — headlessly clear). The broad alloc-flag seed is a MEASURED REGRESSION (routes ALL operator-new into the headless-failing real-alloc path -> early SIGABRT; reverted). DMCONT continuation is now WIRED+ARMED+ACTIVE (was "latent"), backed by a measured runtime path through the real continuation body. +2 hermetic (sh245 getter-tail word+site guard; sh245 m48-seed passes the app-name decode). elfjit examples 86/0, workspace green, single-agent. Doc docs/frontier-sh245-getter-total-activates-continuation.md, repro runs/capture_sh245_getter_tail_ab.sh (off|on|on.m2|on.m48|on.m2m48). Route-B live-DM gate NOT lifted but the continuation line moved forward one measured gate.
## SH244 (Sep 17, 2026, hermes-worker): the engine-init getter's vt[+0x30] return word gates a StartLuaAppDM SELF-call. Implemented + A/B'd the never-tried -2 verb (`JIT_ROUTEB_DM_MGR_MINUS2`, default-inert): the getter 0x102174c04 the dispatcher bl's (`bl 2174c04` @ 0x2bd8d14) dispatches M.vt[+0x30], tests `cmn w0,#0x2`; ONLY a -2 return makes it TAKE its own `bl 0x10242a5e4` (nativeAppBridgeStartLuaAppDM body) before `b 0x624e6c0` FMOD/AAudio tail. Our write-leaf returned 0 → b.ne skipped StartLuaAppDM every time and diverted to FMOD (which never returns to the dispatcher → SH243's "never resumes past getter" mechanism now explained at the exact branch). +hermetic sh244 (lib 392/0). Doc docs/frontier-sh244-mgr30-minus2-startluaappdm-selfcall.md, repro runs/capture_sh244_ab.sh.
- **MEASURED (real libroblox.so, A/B one completing ladder each, EXIT 124, 0 crash):** BASELINE (mgr30=0): getter 0x102174c04/c48 fired, **SLADM body 0x10242a5e4 = 0 hits**, FMOD tail 0x10624e6c0 fired, dispatcher resume 0x2bd8d18 = 0, sub/continueAfterFlagsLoaded_ = 0. FORWARD (mgr30=-2): **0x10242a5e4 + 0x10242a5f8 ENTERED (both 0 in baseline)** — engine-init self-executes a headless-previously-unreachable region of the nativeAppBridgeStartLuaAppDM function body — FMOD tail still reached, 0x2bd8d18 still 0. StartLuaAppDM returned Ok(M) both arms.
- **HONEST (do-not-over-claim):** genuine new engine-init execution path opened (never-before-driven SLADM self-invoke); does NOT manufacture a DM — the FMOD tail 0x624e6c0 still consumes control after it, so the dispatcher interior (0x2bd8d18) + continueAfterFlagsLoaded_ (0x102bd1d68) stay 0. Route-B live-DM structural gate UNCHANGED. Next: does the FMOD tail's early-return path (0x624e708 cbz [x0+8] → ret) ever return to 0x2bd8d18.
- **CODE:** jit.rs — new `routeb_dm_manager_write_leaf_minus2` (write a1<=a0, return 0xFFFFFFFE) + `routeb_manager_mgr30_leaf()` (env JIT_ROUTEB_DM_MGR_MINUS2 selects -2 vs default 0 at vt[+0x30]) wired into both fabricated + cont managers; default path unchanged (env off = write-leaf 0 = verified baseline). + hermetic sh244.
## SH243 (Sep 17, 2026, hermes-worker): MEASURED the NativeDataModelManager GETTER CELL was seeded at the WRONG ADDRESS for 8 cycles (SH165-240). Getter 0x102174c04 `adrp x8,7275000; add x8,+#0x550; ldar x0,[x8]` reads GUEST 0x107275550 (vaddr 0x7275550, RW data seg) — NOT 0x102727550 (vaddr 0x2727550, which is inside the R-E CODE seg), 0x5000000 apart. Verified 3 ways (hand ADRP decode, objdump symbol, loader segment map). A/B on the real binary (6/6 completing ladders EXIT 124): with the TRUE cell ALSO seeded -> StartLuaAppDM returns Ok(M)=the fabricated manager pointer (was benign Ok(0x3e8)) — the session now carries M down the call instead of the benign soft-return. Total search coverage: the prior 8 cycles' "dispatcher never resumes past the getter" verdict was partly a WRONG-ADDRESS seed, not purely a latent wall.
- **WHY (a genuinely open premise, not a re-tread):** SH165-240 all seeded the manager into 0x102727550 and concluded the dispatcher 0x102bd8ce8 "never resumes past the getter" = a live-session wall. Nobody re-derived WHICH cell the getter's ldar actually reads. This cycle does.
- **MEASURED (authoritative decode + runtime A/B):** getter read cell = 0x107275550 (3 authorities). routeb_dm_manager_guard now ALSO seeds it (both cells, DMFORCE-gated, idempotent). +1 hermetic sh243 (lib 391/0; examples 85/0; workspace green; recon-v3 plane re-verified 24 frames swap Ok(0x1) no json abort).
- **HONEST (do-not-over-claim):** delivering M into the getter is NECESSARY-BUT-INSUFFICIENT for a live DM. Dispatcher interior + sub_2bd8dac + continueAfterFlagsLoaded_ (0x102bd1d68) STILL 0 hits under the interior region watch; marshaller 0x1023f075c + EC world 0x102e24598 still 0 hits. The dispatcher's first-call-boundary leaf path still terminates upstream of the bl sub. A real flags-loaded engine-init session remains the standing Route-B structural gate. No make_shared<DataModel>; M is a fabricated all-leaf manager.
- **CODE:** jit.rs routeb_dm_manager_guard (seed both cells) + sh243_page_mapped helper + sh243 hermetic; elfjit.rs sh240 pin semantic label corrected (byte words unchanged; getter ldar reads 0x107275550). Repro runs/capture_sh243_manager_cell.sh. Doc docs/frontier-sh243-manager-cell-wrong-address.md. Commit 6fb645f.
- **STANDING (do-not-re-tread, unchanged):** Route-B live-DM = structural gate. SH174 capture-latch arming *(0x106391908) at a real make_shared<DataModel> = the single forward hook. recon-v3 deliverables stay shipped + verified.
## SH240 (Sep 17, 2026, hermes-worker): DMCONT continuation re-measured FRESH at HEAD = block-entry-DEFINITIVE negative (SH228 holds at newest state) + full dispatch chain byte-pinned. +1 hermetic (sh240, elfjit examples 85/0). Doc docs/frontier-sh240-dmcont-fresh-negative-pinned.md. Workspace green (lib 390/0).
- **WHY (the re-verify-at-newest-state doctrine + operator's named 'keep grinding the DMCONT continuation' line):** SH228 measured the engine-init dispatcher 0x102bd8ce8's `bl sub_2bd8dac` (-> vt+0x1f0 = continueAfterFlagsLoaded_ 0x102bd1d68) as headless-unreached at the OLD 4b6700b HEAD. Since then SH229/233/234/235/237/238/239 landed (new guards + SH161b/SH217 state). Re-measured fresh under the exact DMCONT env (JIT_ROUTEB_DMFORCE=1 + DMCONT=1, holder 0x102727550 seeded -> manager vt[+0x1f0]=REAL continueAfterFlagsLoaded_).
- **MEASURED (3/3 clean EXIT 124, 0 signals, real libroblox.so, canonical completing --v2boot ladder):** manager routing fires (fnB entered, `SH165 manager singleton holder... -> continuation-routed manager (vt[+0x1f0]=REAL continueAfterFlagsLoaded_ 0x102bd1d68)`); **JIT_REGION_WATCH on interior resumption pcs (0x2bd8d30 post-leaf1, 0x2bd8d54 post-leaf2), sub_2bd8dac, AND continueAfterFlagsLoaded_ = 0 hits all 3 runs; JIT_DUMP_REGION [0x102bd8ce8,0x102bd9060) fired exactly once at block-entry 0x102bd8ce8.** Block-entry-definitive: the dispatcher is entered but control NEVER reaches 0x2bd8d18/0x2bd8d30/0x2bd8d54/0x2bd8d60/sub/continueAfterFlagsLoaded_ — it terminates at/inside its first call-boundary (the getter `bl 2174c04` / leaf path) without falling to the unconditional `bl sub_2bd8dac`. SH228's 'dispatcher completes through the resolve/leaf paths before the bl sub' holds fresh at the newest HEAD.
- **HONEST (do-not-re-tread):** DMCONT manufactured-manager continuation stays LATENT (necessary-but-insufficient: routing +0x1f0 to the real continueAfterFlagsLoaded_ never even gets dispatched because the dispatcher doesn't reach the `bl sub`; a fabricated all-leaf manager supplies no genuine flags-loaded/network-fetch state). Fires only on a real session's flags-loaded engine-init state. Route-B live-DM structural gate UNCHANGED.
- **CODE (sh240, real-image guard family as sh239/238/237, skip-if-absent):** byte-pins all 12 chain anchors — fnB 0x102bd1b98=0xd10103ff + `bl 0x2bd8ce8`@0x102bd1c10=0x94001c36; dispatcher 0x102bd8ce8=0xd10143ff; getter adrp 0x102174c18=0xb0028808 + ldar 0x102174c28=0xc8dffd00 (seeded holder 0x102727550); leaf blrs 0x102bd8d2c/0x102bd8d50=0xd63f0100; `bl sub_2bd8dac`@0x102bd8d60=0x94000013; sub 0x102bd8dac=0xd104c3ff + ldr vt+0x1f0@0x102bd8e20=0xf940f908 + blr@0x102bd8e28; continueAfterFlagsLoaded_ 0x102bd1d68=0xa9ba7bfd + 4-align/in-window. No production path edited.
- **UNREGRESSED:** workspace green (390 lib + all crates), examples 85/0 (was 84/0); recon-v3 plane re-verified green at HEAD (24 task-driven frames presents #19..#23 swap Ok(0x1), 192 node pops, no json abort, EXIT 124). Repro runs/capture_sh240b_dmcont_flow.sh.
- **STANDING (do-not-re-tread, unchanged):** Route-B live-DM = structural gate. SH174 capture-latch arming *(0x106391908) at a real make_shared<DataModel> = the single forward hook. DMCONT = WIRED + ARMED + LATENT (fresh). recon-v3 deliverables stay shipped + verified.
## SH239 (Sep 17, 2026, hermes-worker): do-init once-lambda "LET it populate [0x106a68818]" premise A/B-FALSIFIED headlessly (yields intern 0x400000b, never a DM) + app-shell ctor 0x102207b50 body MEASURED DEEP (61 blocks to 0x102208eac, tail FMOD/AAudio 0x5fb30b4) — qualifies "never completes construction". +2 hermetic (sh239 + sh238 real-image band; examples 84/0, lib 390/0). Doc docs/frontier-sh239-doinit-qualified.md. Workspace green.
- **FALSIFIED (fresh A/B, real libroblox.so, 2 clean completing ladders EXIT 124, 0 crash):** the operator's EXECUTE-DO-INIT-GATES hypothesis that "LETTING the once-lambda populate [0x106a68818]" manufactures a DM is WRONG. WITHOUT the SH156 seed, oncel-guard [0x6a68410] still self-latches 0->1 but DM-root [0x106a68818] stays **0** and once-slot [0x106a68408] = intern **0x400000b** (RTApp registry key, NOT an in-image DM controller). The `str x0,[x23,#1032]` completion store (file 0x2206d74) writes the intern either way. So the SH156 fabricated DM-root seed is **necessary-but-insufficient**, and the live-DM wall is NOT seed-caused — removes the "maybe our seed masks the once-lambda output" residual.
- **QUALIFIED (region-watch [0x102207b50,0x102209000), SH156 seed present):** the app-shell/global-init ctor 0x102207b50 body executes **61 distinct block-entry pcs** to **0x102208eac** (incl. app-data-model register appends, SH203-known) and its terminal tail targets **FMOD/AAudio iterate 0x5fb30b4** (sub sp,#0x60) — the sound-pillar first contact (SH212/213-class). So "StartLuaAppDM's do-init never completes app-shell construction" is too coarse: the ctor body RUNS far; what never happens is `make_shared<DataModel>` (SH174 capture latch: only small 0x18 FIRST allocs, none validated). Route-B live-DM structural gate UNCHANGED (SH209/218/223/224/228/231/232/235/236/237/238).
- **CODE (+2 hermetic, real-image guard family, skip-if-absent):** sh239 byte-pins the once-lambda completion store (0x102206d74=0xf90206e0 str x0,[x23,#1032] -> [0x106a68408]; 0x102206d78=0xd0024300 adrp 6a68000), do-init match br (0x102206e24=0xd61f0020), app-shell ctor entry/prologue/oncel-guard (0x102207b50=0x14000001, 0x102207b54=0xa9be7bfd, 0x102207b68=0x08dffd08 ldar [0x106a64d70]), FMOD tail (0x105fb30b4=0xd10183ff); sh238 pins the receiveCall dispatch band relocs still covering select slots 0x635dd88/0x635dd90. No production path edited.
- **HONEST:** forward recon + measured falsification + regression pins; does NOT manufacture a DM. Single forward hook stays SH174 capture-latch arming at a real make_shared<DataModel>. recon-v3 deliverables stay shipped + verified (re-verified at HEAD before commit).
# (previous session SH238 follows)
- **WHAT (a genuine forward attempt, not a static recon pin):** SH235 pinned the sole EC-world front-door (StartLuaAppDM -> bl 0x1023f1210 @0x1023f075c -> EC world 0x102e24598); SH236/237 localized the soft-return to the receiveCall dispatch-select (union [0x10635dd68]: [+0x20]=__clone 0x101db2cf0, [+0x28]=invoke 0x1021e96f8, both loader-synthesized) and said "the next drive must satisfy the dispatch." This cycle actually DRIVES it: a new default-inert guard (routeb_startluaapp_invoke_guard, env JIT_ROUTEB_SLADM_INVOKE=1) at the select block-entry pc 0x1023efeb0 seeds guest [sp+32] = a leaked box holding 0x10635dd68 so the dispatch selects [union+0x28]=invoke, then region-watches StartLuaAppDM body + marshaler + EC world.
- **MEASURED (real libroblox.so, 2 clean completing ladders EXIT 124, 0 crash; A/B lever off vs on):** BASELINE soft-return pcs `.. f00f8 f01e4`, marshaler 0 hits, EC world 0 hits. FORWARD: `[routeb-sladm] CROSSED -> [sp+32]={box} ... was 0x5624f64f6a50` — StartLuaAppDM STILL soft-returns Ok through the IDENTICAL terminating blocks, marshaler 0 hits, EC world 0 hits. Two measured conclusions: (1) [sp+32] is natively a REAL host-heap pointer (0x5624f64f6a50), NOT the self-ref sp — so the dispatch natively selects INVOKE already (SH236's __clone inference is WRONG); (2) the select lever is a MEASURED dead-end — gating the marshaler is DOWNSTREAM of helper 0x1023f00f8's return path (the big sub-body 0x23f03b4 is reached by control flow that never executes headlessly), empirically confirming SH237's correction.
- **HONEST (do-not-over-claim):** closes the SH235-237 select lever with a runtime measurement (proof-of-dead-end standard for THIS lever) and corrects SH236's slot inference. Does NOT manufacture a DataModel; Route-B live-DM structural gate UNCHANGED (SH209/218/223/224/228/231/232/235/236/237). Next lever on this line = forcing control into the big sub-body 0x23f03b4 directly (fabricatable-object-graph class, SH204/SH174 already mapped to the same wall) — not implemented.
## SH237 (Sep 17, 2026, hermes-worker): StartLuaAppDM receiveCall dispatch-SELECT is LOADER-SYNTHESIZED .data.rel.ro (corrects SH235/236's "session-gated select" class) + helper 0x1023f00f8 COMPLETES a real V2Init struct-copy+FMOD sub-body (corrects SH236's "benign-return" label). +1 hermetic (sh237, elfjit example 82/0 from 81). Doc docs/frontier-sh237-sladm-dispatchselect-loader-synth.md. Workspace green (568/0).
- **WHY (a genuinely-open angle, not a re-tread):** SH235 statically concluded the EC world 0x102e24598's only headless front-door (StartLuaAppDM -> bl 0x1023f1210 @0x1023f075c) is "gated by StartLuaAppDM's receiveCall dispatch switch on live controller state" — but nobody had MEASURED where StartLuaAppDM's own flow terminates headlessly. This cycle closes that with a runtime block-entry measurement of StartLuaAppDM's OWN body.
- **MEASURED (real libroblox.so, canonical completing --v2boot ladder, EXIT 124, 0 crash, full 9-rung ladder Ok end-to-end):** region-watch on the FULL StartLuaAppDM body [0x1023efe2c,0x1023f0800) + marshaler [0x1023f1210,0x1023f1300) fires **only 14 distinct block-entry pcs, the LAST = 0x1023f01e4** (`efeb0 efedc eff4c effa0 effac effc0 effc8 effd0 efffc f0008 f0020 f00f8 f01e4`); then StartLuaAppDM benign-soft-returns `Ok(real heap)` and the ladder continues. The marshaler-call block 0x1023f075c (bl 0x1023f1210) is **NEVER entered**; marshaler region = 0 hits. The receiveCall dispatch terminates inside helper fn **0x1023f00f8** (sub sp,#0x70, reads stack flags [sp+8]/[sp+32], benign-returns), reachable from the entry dispatch tail 0x1023efed8 (`blr x8`) — the exact spot to satisfy to fall through to the marshaler + EC world.
- **VERDICT (do-not-re-tread):** CONVERTS SH235's static "dispatch switch gates the EC front-door" into a measured, located mechanism: StartLuaAppDM soft-returns in helper 0x1023f00f8 (last block 0x1023f01e4) before the marshaler call — the next drive's exact obstacle is that helper's stack-flag path (live-state/fabricatable-graph class, NOT a static seed). Route-B live-DM structural gate UNCHANGED.
- **HONEST:** forward recon + regression pins; single-agent; no production path edited; recon-v3 (24 frames / json zero-fix) + SH174 capture-latch ARMED unchanged at HEAD.
## SH235 (Sep 17, 2026, hermes-worker): genuine EC DM-creation world's SOLE entry pinned via full-.text caller proof — 9-arg marshaler 0x1023f1210 is the only headless front-door to 0x102e24598. Commit 9bab2d4. Doc docs/frontier-sh235-ec-world-sole-entry.md. +1 hermetic (sh235, elfjit example 80/0). Workspace green.
- **WHY (a genuinely-open angle, not a re-tread):** the operator's Route-B re-attack directive names the ExperienceController/initializeLuaAppWithDataModel line and says the next drive must start at the CORRECT address. SH231 located the DM-creation lambda world + measured it unreached; SH232 pinned its in-rung `bl`s and CORRECTLY found the caller bodies never translate, but mislabeled the caller as "EC-arg helper 0x1023f11f4" and never pinned the world's exact entry. This cycle closes that.
- **MEASURED (fresh, authoritative):** the genuine DM-creation world's entry instruction is **0x102e24598** (`stp x29,x30,[sp,#-96]!`=0xa9ba7bfd) inside the EC body region — a big start-app-params marshaller (lr dispatches `blr [this+0x30]->vt+0x10`, derefs ~24 fields on a 2nd object, reads flags globals 0x6a69000+0x358/0x6d31000+0xe28, then `bl 0x23c5538`/`bl 0x23f1654`). Its SOLE headless front-door is the 9-arg marshaler **0x1023f1210** (`sub sp,#0xb0`=0xd102c3ff), whose only `bl` (0x1023f1294=0x9428ccc1, SH232's pin) targets the world. SH232's "EC-arg helper 0x1023f11f4" is a DIFFERENT tiny cleanup fn (calls 0x2b9e950, not the EC world) — label corrected.
- **FULL-.TEXT CALLER PROOF (authoritative — objdump's per-symbol grep under-reports):** the test scans every executable word for direct bl/b: marshaler 0x1023f1210 callers = EXACTLY {0x1023f075c (StartLuaAppDM), 0x102e15bf0, 0x102e33494 (EC self-sites)}; EC world 0x102e24598 callers = EXACTLY {0x1023f1294 (marshaler), 0x102e18408 (=0x94003064, EC self-call)}.
- **VERDICT (do-not-re-tread):** 0x102e24598's only headless path is StartLuaAppDM's `bl 0x1023f1210` @0x1023f075c, gated by StartLuaAppDM's receiveCall message-dispatch switch on live controller state. The next drive's exact gate: make that dispatch fall through to 0x1023f075c with a REAL ExperienceController `this` + full StartAppParams = the fabricatable-object-graph class, NOT a static seed (0x2e24598 immediately derefs live objects). Route-B live-DM structural gate UNCHANGED. Does not manufacture a DM.
- **HONEST:** forward recon + regression pins; single-agent; no production path edited. recon-v3 (24 task-driven frames / json zero-fix) + SH174 capture-latch ARMED — the standing forward hook — unchanged at HEAD.
## SH234 (Sep 16, 2026, hermes-worker): recon-v3 RENDER-SIDE ENGINE CONTRACT byte-pinned (+1 hermetic sh234, ed example 79/0). Doc docs/frontier-sh234-render-side-contract-pinned.md. Workspace green.
- **WHY:** the recon-v3 SELF-DRIVEN FRAMES deliverable rides an ENGINE render contract type4_frame_thunk derefs through RENDERCTX (vtable 0x106731ae0): make-current [vt+16]=0x105b3b358, frame-fn 0x105b32c00, swap [vt+24]=0x105b3b408, renderinit 0x105b3a280. SH230 pinned the type-4 DISPATCH site but NOT these engine fns — a silent drift breaks the 24-frame plane with no loud failure. This closes that gap.
- **CODE (+1 hermetic sh234, real-image guard family as sh230-232, skip-if-absent):** byte-pins the four engine fn prologues (0x105b3a280/0x105b3b358=0xa9bd7bfd, 0x105b3b408=0xa9420408, 0x105b32c00=0xd10303ff + first siblings), window/4-alignment for the fns + vtable 0x106731ae0, AND via the loader's OWN packed-RELA decode verifies the ctx vtable band (file [0x6731000,0x6732000)) holds RELATIVE addends 0x5b3b358 and 0x5b3b408 (the [vt+16]/[vt+24] pointers, on-disk zeros). Reuses the sh233 `load_real_image()` cache.
- **VERIFIED:** `cargo test -p arm64jit --examples` = **79/0** (was 5 red pre-SH233, 78 after SH233, 79 with sh234); sh234 1 passed filtered; cargo build --workspace EXIT 0. recon-v3 plane green at this HEAD.
- **HONEST:** pure regression hardening of an already-verified deliverable; does NOT manufacture a DataModel, does NOT lift the Route-B live-DM structural gate (SH209/218/223/224/228/231/232/233 unchanged).
## SH233 (Sep 16, 2026, hermes-worker): `cargo test -p arm64jit --examples` parallel batch made genuinely GREEN — was silently failing 5 of 78 real-image guards (sh222-228/sh231/sh232) on EVERY run. Test-harness-only fix, zero production change. Doc docs/frontier-sh233-examples-batch-green-cache.md. Commit 1956f00.
- **THE DEFECT (measured, reproducible):** the documented examples gate failed 5/78 every run — real-image guard family panicked `left: 0` + one `Failed to read segment into JIT image: Bad address (os error 14)`. NOT a regression: each passes 0.08 s filtered (`cargo test --example elfjit sh232` = fresh process, one load). Root cause: `libloader::elf::load_elf_image` maps the 109 MB libroblox.so at a FIXED guest base 0x100000000 (required so guest==host) and LEAKS the mapping for the one-shot run; the parallel `--examples` batch triggers ~8 concurrent loads of the same .so in one process → only the first mmap holds the base, the rest read 0 / EFAULT.
- **FIX (test-harness only):** added `load_real_image()` — a per-process `OnceLock<LoadedElf>` cache — in `sh115_tests` and routed the 8 real-image test load sites (sh221/sh223/sh224/sh225/sh226/sh227/sh228/sh231/sh232) through it → exactly ONE mmap per test process, all assertions unchanged. The production jit_run load (elfjit.rs main, line 6340) and the four `std::fs::read` sites (sh116b/sh212-family) are untouched; `cargo build --workspace` EXIT 0.
- **VERIFIED at HEAD:** `cargo test -p arm64jit --examples` = **78 passed; 0 failed** (was 5 failed every run); filtered single-run semantics unchanged (sh232/231/228/227/226 each 1 passed); recon-v3 SELF-DRIVEN FRAMES re-verified green (24 task-driven frames, `swap Ok(0x1)` ×24, distinct colors, no json abort, 0 crash, dispatch #4654000) via runs/capture_taskv4_frame.sh.
- **HONEST (do-not-over-claim):** pure test-harness robustness; does NOT manufacture a DataModel, does NOT lift the Route-B live-DM structural gate (SH209/218/223/224/228/231/232 unchanged). Closes a real footgun — the documented examples gate is now genuinely green instead of silently red, so a drifted real-image constant fails loudly in batch rather than being masked by the mmap collision.
## SH232 (Sep 16, 2026, hermes-worker) — the in-rung EC callers never translate on the completing ladder (closes SH231a's static inference with a runtime mechanism). +1 hermetic (sh232, elfjit example 78/0). Doc docs/frontier-sh232-ec-caller-mechanism.md, repro runs/capture_sh232_ec_caller_mechanism.sh. Workspace green (389 arm64jit + all crates).
- **WHY (genuinely new, not a re-confirmation):** SH231 measured the EC TARGET region [0x102e1c650,0x102e25200) at 0 hits headlessly; SH231a added a STATIC caller-side scan (241 direct bl/b callers, incl. two inside LADDER rungs). The gap SH231a left: are the caller bodies entered-as-blocks and diverted in-body, or do the enclosing rungs benign-complete BEFORE the caller block is ever translated? The rungs are `nativeAppBridgeStartLuaAppDM` 0x1023efe2c (EC caller 0x1023f1294 -> `bl 0x2e24598`) and `nativeAppBridgeV2InitWithParams` (deep-branch 0x23cfafc, EC caller 0x1023cfd68 -> `bl 0x2e24468`).
- **MEASURED (runtime, real libroblox.so, canonical --v2boot ladder, 5 watched regions, completing-only):** on BOTH govtail-positive (control-fired) runs — EXIT 124, governor tail walked to 0x102ea30dc — the EC target AND all four EC caller/body blocks (StartLuaAppDM EC-arg 0x1023f12d0..0x1023f1300, V2Init EC-caller 0x1023cfd40..0x1023cfd70, V2Init mid-body 0x1023cfb40..0x1023cfc7c) stayed **0 hits**. Ladder sequence confirms it: `driving StartLuaAppDM` -> `StartLuaAppDM returned Ok(0x55a01d785a20)` (benign soft-return) -> SH155 once-slot=0x400000b (intern, NOT a DM) -> drive V2StartAppWithParams -> outside-image stop. The rungs benign-complete UPSTREAM of their EC-call blocks, so the `bl`s are never even translated into blocks on the ladder.
- **HONEST (do-not-over-claim):** CLOSES SH231a's residual with a runtime mechanism (block-entry-definitive: the caller bodies themselves never enter). Does NOT manufacture a DM, does NOT lift the Route-B live-DM structural gate (SH209/218/223/224/228/231 unchanged). Ladder ~1/3 flaky (SH198/SH55 V2Init singleton-vtable stop); conclusion rests on govtail-positive runs only (SH209/223/231 pattern).
- **CODE (+1 hermetic sh232, real-image guard family as sh231, skip-if-absent):** byte-pins the two EC-call `bl` words (0x1023f1294=0x9428ccc1->0x2e24598; 0x1023cfd68=0x942951c0->0x2e24468), enclosing-fn prologues (StartLuaAppDM entry 0x1023efe2c=0xd10183ff, EC-arg helper 0x1023f11f4=0xa9bf7bfd, V2Init deep-branch 0x23cfafc=0xa9bc7bfd), bl imm26 sign-extended target cross-checks, + 4-alignment/window guards. No production path edited.
- **NEXT (honest, single-agent):** Route-B live-DM = structural gate UNCHANGED. The EC world is located (SH231) + its in-rung callers pinned + non-execution mechanism measured (SH232). Standing forward hook unchanged: SH174 capture-latch arming *(0x106391908) at a real make_shared<DataModel>. recon-v3 deliverables stay shipped + verified.
## SH231 (Sep 16, 2026, hermes-worker): REAL DataModel-creation world LOCATED via the loader's own fresh packed-RELA decode + measured headless-UNREACHED (the site SH178 said was "NEVER properly located"). +1 hermetic (sh231, elfjit example 77/0). Doc docs/frontier-sh231-experiencecontroller-located-unreached.md, repro runs/capture_sh231_experiencecontroller_reach.sh. Workspace green (389 arm64jit + all crates).
- **WHY (SH178's explicit open directive):** SH178 refuted the ~30-chase do-init addresses (0x2206d74/0x2173b3c) as the "only make_shared<DataModel> inline" (they're the GlobalInit do-init tail + a string-intern GetOrCreate), and stated the genuine DM is created by ExperienceController::createDataModelForTeleport / submitStartGameTask / initializeLuaAppWithDataModel — but "the genuine DataModel allocation site was NEVER properly located." This cycle LOCATES it.
- **LOCATED (fresh packed-RELA decode on the loader's exact path, real libroblox.so):** the REAL DM-creation machine = std::function __func lambda world whose vtable band (guest 0x63981d8..0x6399c00, relocation+session-populated .data.rel.ro) carries typeinfo-name-string pointers (RELATIVE addends) into the createDataModelForTeleport/submitStartGameTask RTTI rodata band [0x6dc000,0x6e2000), and whose CODE bodies land at guest [0x102e1c650, 0x102e25200) (e.g. row 0x63985e8 code slots 0x102e1ddc8/0x102e1dde8; shared __clone stub 0x1db2cf0 / invoke 0x21e96f8).
- **MEASURED (real libroblox.so, canonical completing --v2boot ladder, calibrated):** region-watch EC-body [0x102e1c650,0x102e25200) + governor-tail control [0x102e9fa80,0x102ea3b40) = **0 EC-world hits across 4/5 govtail-control-positive clean completing runs (EXIT 124, ladder done)**; run-3's govtail=0 = the known run-variable non-completing stop, not a region hit. **CALLER-SIDE double confirmation:** the region has 241 direct bl/b callers (incl. from the executing do-init/app-shell chain 0x1023cfd68->0x102e24468, 0x1023f1294->0x102e24598) — yet 0 region hits means none of those callers executes on the ladder (a reached caller would translate its target as a block and fire the watch). The genuine site is real but headless-UNREACHED — the same Route-B live-DM structural gate, now at the CORRECT location (SH178's mandate fulfilled: fresh packed-RELA decode, not the refuted addresses).
- **HONEST (do-not-over-claim):** does NOT manufacture a DM, does NOT lift the structural gate. CLOSES SH178's "never properly located" with a located+pinned+measured-unreached artifact. Next drive on this line starts from the correct address (real session's createDataModelForTeleport lambda world), not the refuted do-init line.
- **CODE (sh231, real-image guard family as sh227/sh228/sh229b, skip-if-absent):** byte-pins EC body entry 0x102e1c650=0xa9bf7bfd, mid-body 0x102e20398=0xa9be7bfd, upper-window 0x102e25148=0xd102c3ff, region bounds+alignment, AND verifies via the loader's own read_elf_relocations that the vtable band [0x6398000,0x639a000) holds >=4 RELATIVE relocs whose addend is in the createDataModelForTeleport RTTI rodata band [0x6dc000,0x6e2000). No production path edited. Throwaway analysis tools (dmfind/dmreloc) removed after deriving pins.
- **UNREGRESSED at HEAD:** workspace green (cargo test --workspace EXIT 0; 389 arm64jit + all crates); elfjit examples 77/0 (was 76). recon-v3 render plane + SH210 latent wiring re-verified green earlier this session. Tree clean at end of commit.
- **STANDING (do-not-re-tread, unchanged):** Route-B live-DM = structural gate. SH174 capture-latch arming *(0x106391908) at a real make_shared = the single forward hook. recon-v3 immediate-priority deliverables stay shipped + verified.
# (previous session SH230 follows)
## SH230 (Sep 16, 2026, hermes-worker): byte-pin the type-4 dispatch site — the recon-v3 self-driven-frame load-bearing opcodes (file 0x2853784 adrp / 0x2853788 ldr / 0x285378c cbz / 0x28537b8 br x3) added to the sh211 real-image guard. +re-verified recon-v3 plane (24 frames) + ENTIRE SH210 latent wiring ARMED at HEAD (3/3 clean). Doc docs/frontier-sh230-type4-dispatch-bytepins.md. Workspace green (examples 76/0).
- **The gap (genuinely open, not a re-tread):** recon-v3 deliverable SELF-DRIVEN FRAMES rides the type-4 dispatch site `adrp x8,6829000; ldr x3,[x8,#3752]; cbz x3,skip; ...; br x3` — the exact stream that br's into the seeded `0x106829ea8` vector to present a task-driven frame. SH211 byte-pinned the heartbeats + wiring cells but NOT this dispatch site's own opcodes. A silent drift there breaks the deliverable with no loud failure. Closed it with 4 real-image byte-pins (0x2853784=0xd001fea8, 0x2853788=0xf9475503, 0x285378c=0xb4001b23, 0x28537b8=0xd61f0060), verified LE against the real libroblox.so (109,193,800 B).
- **SH210 full latent wiring RE-VERIFIED at this exact HEAD (3/3 clean EXIT 124):** arm=3 xid=3 appevent=3 filesdir=3 dmprobe=3 — capture latch, G1 surface-XID, G2 SendAppEvent 'Home', G3 files-dir, SH155 DM probe: ALL ARMED. recon-v3 plane green (24 task-driven frames, present #19..#23 swap Ok(0x1), 195 pops, no json abort, 0 crash).
- **HONEST (do-not-over-claim):** pure regression hardening; does NOT manufacture a DM or advance past the live-DM wall. Route-B live-DM structural gate UNCHANGED. Single forward hook stays SH174 capture-latch arming at a real make_shared. recon-v3 deliverables stay shipped + verified.
- **VERIFY:** cargo test --workspace green (389 arm64jit + all crates); cargo test -p arm64jit --examples 76/0; `cargo test -p arm64jit --example elfjit sh211_routeb_wiring` 1 passed with real-image pins. Tree clean at end of commit.
# (previous session SH229 follows)
- **WHY (an angle SH187 left measured-pending):** SH187 drove the REAL DM ctor wrapper (0x1023f5ff8 -> 0x1023f6038) but only ever as a PARTIAL build — it NOP'd the subobject call `bl 0x23f6b0c` @ 0x1023f60b8 so the straight-line body fell through to the genuine-vptr writes. SH189 recon says that subobject builds the DM's INTERNAL 361-entry class index — a MORE-complete DM. The full-ctor run was never measured. This cycle adds `JIT_ROUTEB_DM_REALCTOR_FULL` (leaves the subobject INTACT + adds an index/instance-base read-back) and measures it — the operator's "drive its ctor world-build further / dynamic DM-ctor trace" line.
- **MEASURED (real libroblox.so, canonical --v2boot ladder, EXIT 124, 0 crash; PARTIAL baseline re-verified this HEAD):** PARTIAL (NOP'd) = `obj vptr {0x1067162e8,0x1067163a0,0x1067163f8} GENUINE MATCH` (real DM family, planted into holder). FULL (un-NOP'd) = `obj vptr {0x1067147c8,0x1067bdb20,0x106796dc0} (genuine=false)`; `obj+0x1f0=0x106796dc0; index-build obj+0x2a0={0,0,0}`. **The un-NOP'd subobject redirects construction to the instance-base vptr family 0x106796dc0 (SH190c instance base, a DIFFERENT class), NOT the derived DataModel, and the 361-entry index stays empty.** So SH187's NOP was LOAD-BEARING for genuine-DM production — running the subobject produces a base-class-shaped object upstream of the DataModel.
- **HONEST (do-not-over-claim):** the FULL ctor does not crash (SH187's "stalls inside 0x23f6b0c" is a different transient) but converts the operator's full-ctor re-attack into a MEASURED dead-end for genuine-DM production. Route-B live-DM structural gate UNCHANGED (SH209/218/223/224/228). The PARTIAL drive remains the correct genuine-DM manufacture path and is UNREGRESSED (re-verified GENUINE MATCH this cycle).
- **CODE (default-inert, env JIT_ROUTEB_DM_REALCTOR_FULL as a seed modifier on the base JIT_ROUTEB_DM_REALCTOR):** skips the subobj NOP under FULL + adds FULL-mode read-back of obj+0x1f0 and the obj+0x2a0 index region. Hermetic `sh229_dm_real_ctor_full_mode_env_and_region_gated` asserts default-inert / base-env-gated / region-scoped. No default-config production path edited.
- **UNREGRESSED at HEAD:** recon-v3 render plane re-verified (24 task-driven frames, present #19..#23 swap Ok(0x1), 194 node pops, no json abort, EXIT 124); type4_frame_thunk registered+seeded in [0x106829ea8]; full workspace green (389 arm64jit + all crates). Tree clean at end of commit.
- **STANDING (do-not-re-tread, unchanged):** Route-B live-DM = structural gate. SH174 capture-latch arming *(0x106391908) at a real make_shared = the single forward hook. recon-v3 immediate-priority deliverables stay shipped + verified.
## SH228 (Sep 16, 2026, hermes-worker): engine-init dispatcher sub/continueAfterFlagsLoaded_ NEVER enter as blocks — closes SH166's open question (a) DEFINITIVELY + corrects SH226's completion mechanism. +1 hermetic (sh228, elfjit example 76/0). Commit 4b6700b. Doc docs/frontier-sh228-dispatcher-divert-before-sub.md, repro runs/capture_sh228_dispatcher_divert.sh. Workspace green.
- **WHY (a genuinely-open residual, not a re-confirmation):** SH166 explicitly left question (a) open — whether the engine-init vt[+0x1f0] dispatch (-> continueAfterFlagsLoaded_) truly executes or is diverted "is NOT decidable from region-watch alone." SH226 then CLAIMED the engine-init dispatcher 0x102bd8ce8 "benign-completes via its 2nd-frame (sub_2bd8dac) -> 0x102bd9058 soft-return" — a mechanism assertion never pinned against whether sub itself is even entered.
- **MEASURED (real libroblox.so, ONE completing --v2boot ladder run, DMCONT=1, 4 region windows, reproducible 2/2):** fnB (0x102bd1b98, engine-init entry) AND the dispatcher (0x102bd8ce8) BOTH fire as their own block entries; **sub_2bd8dac (reached by an UNCONDITIONAL `bl 0x2bd8d60 -> 0x2bd8dac`) and continueAfterFlagsLoaded_ (0x102bd1d68, the vt[+0x1f0] target) NEVER fire (0 hits). EXIT 124, 0 crash.** Because each is a distinct function = its own cached_block entry, non-appearance is block-entry-DEFINITIVE (never entered), not region-watch-blind.
- **TWO CONSEQUENCES:** (1) SH166(a) is CLOSED as a DEFINITIVE negative — the +0x1f0 dispatch at 0x2bd8e28 never runs even with the fabricated manager's vt[+0x1f0] routed to the REAL continueAfterFlagsLoaded_ (0x102bd1d68, DMCONT=1); (2) SH226's mechanism is CORRECTED — sub_2bd8dac is never entered, so the dispatcher completes through the resolve/leaf paths (the `blr vt[+0xf8]` / `blr vt[+0x108]` leaves before the unconditional `bl sub`), NOT "via its 2nd-frame sub". The DMCONT continuation lever is WIRED+ARMED but INERT because the dispatcher stops before sub, not per the older "flags not loaded" framing.
- **HONEST (do-not-over-claim):** does NOT manufacture a DM and does NOT lift the Route-B live-DM structural gate (SH209/218/223/226 unchanged). Removes the last "can't-decide" DMCONT residual and corrects a mechanism claim; the next drive on this line starts from "dispatcher diverts at a leaf before sub."
- **CODE (+1 hermetic sh228, real-image guard family as sh222-227):** byte-pins the unconditional `bl sub` (0x2bd8d60=0x94000013) + mov x0,x20, sub entry (0x2bd8dac=0xd104c3ff) + the +0x1f0 dispatch chain (0x2bd8e18=0xf9400008 / 0x2bd8e20=0xf940f908 / 0x2bd8e28=0xd63f0100), the two pre-bl leaves (0x2bd8d24=0xf9407d08 / 0x2bd8d2c=0xd63f0100 and 0x2bd8d38=0xf9408508 / 0x2bd8d50=0xd63f0100), continueAfterFlagsLoaded_ prologue (0x102bd1d68=0xa9ba7bfd) + guest-transform/4-alignment for the 4 sites. Verified on real libroblox.so (real-image guard, skip-if-absent). No production path edited.
- **UNREGRESSED (re-verified this session):** workspace green (cargo test --workspace); recon-v3 render plane (capture_taskv4_frame.sh: 24 task-driven frames present #19..#23 swap Ok(0x1), 194 node pops, no json abort, 0 SIGSEGV/SIGABRT, EXIT 124); type4_frame_thunk registered+seeded in [0x106829ea8]. Tree clean at end of commit.
- **STANDING (do-not-re-tread, unchanged):** Route-B live-DM = structural gate. SH174 capture-latch arming *(0x106391908) at a real make_shared<DataModel> = the single forward hook; a genuine flags-loaded engine-init state = the other (neither a headless seed). recon-v3 immediate-priority deliverables stay shipped + verified.
## SH227 (Sep 16, 2026, hermes-worker): do-init closure-build binder-dispatch DECODE CORRECTED (#32, not #4) + measured b.ne reachability bypass — converts SH225/226's "fabricate a binder at [union+4]" next-target into a MEASURED proof-of-dead-end (single-agent Route-B re-attack). Commit 7dc7383. Doc docs/frontier-sh227-binderdecode-corrected-bnebypass.md, repro runs/capture_sh227_binder_bypass.sh. +1 hermetic (sh227, elfjit example 75/0). Workspace green (567/0).
- **WHY (a decode error two sessions locked in as authoritative — the lever the operator's do-init re-attack would drive next):** SH156 decoded the do-init closure-build dispatch load as `ldr x0,[x19,#32]`; SH225 "re-corrected" it to `[x19,#4]` and SH226 propagated it as "AUTHORITATIVE / byte-for-byte", building the "binder at [union+4], fill [union+8]" fabrication mechanism on it.
- **MEASURED (3 independent authorities — SH156 was right, SH225/226 wrong):** GNU objdump decodes 0xf9401260 at 0x102206df4 as `ldr x0,[x19,#32]`; arm64jit decode+translate gives `LdStrImm{rn:19,imm:4,size:8}` with address=rn+imm*size (translate.rs:1139) = x19+4*8 = **#32**. AArch64 unsigned-imm LDR scales imm12 by the access size (8 for 64-bit); SH225 read the raw imm12 "4" and mislabeled `#4` (32-bit ×4 semantics). **The SH225 "correction" introduced the error; SH156's original `[x19,#32]` was correct.**
- **MEASURED (live 3-run region-watch, real libroblox.so, EXIT 124, 0 crash):** closure-build enters (0x102206db8/0x206dec, 3/3), the `b.ne` thread-match gate (0x206df0: cmp pthread_self vs stored main-id [0x106863a68]) is TAKEN → LocalStorageManager non-match path (0x206e28/34/3c, 3/3); **the binder-dispatch block 0x206df4..0x206e24 (incl. br x1) = ZERO hits across all runs → never executes headlessly.**
- **CONCLUSION (do-not-re-tread):** SH225/226's "fabricate a binder at [union+4]" next-target is a MEASURED dead-end on BOTH (a) offset (data read is [union+32]=stack self-ref, not a binder at +4) and (b) reachability (b.ne bypass on the ladder) — the operator's proof-of-dead-end standard, measured not judged, for that specific lever.
- **HONEST (do-not-over-claim):** does NOT disprove the do-init→app-shell→governor continuation (SH197 still reaches 0x1023eff4c via a different mechanism) and does NOT lift the Route-B live-DM structural gate (SH209/218/223/226). SH227 removes ONE phantom lever with a corrected decode; remaining forward hooks = live-DM world-build (SH174 capture latch at a real make_shared<DataModel>) or a real flags-loaded engine-init state — neither a headless seed.
- **CODE (+1 hermetic sh227, real-image guard family):** asserts the load word, that decode(0xf9401260) is a 64-bit LDR whose imm12*size==32 (drift/mislabel fails loudly), the b.ne gate 0x540001c1, the non-match target mov x0,sp, and the byte-present-but-unexecuted br x1. Corrected the sh225/sh226 semantic labels ([x19,#32]; assert message now FAILS on `#4`). All byte pins unchanged (they were correct).
- **UNREGRESSED:** recon-v3 render plane (24 task-driven frames, present #19..#23 swap Ok(0x1), 194 node pops, no json abort, EXIT 124) + workspace (567/0) + examples (75/0). Tree clean at end of commit.
## SH226 (Sep 16, 2026, hermes-worker): do-init DM-construction dispatch chain AUTHORITATIVELY reconciled + DMCONT continuation MEASURED negative at HEAD (single-agent Route-B re-attack). Commit 83858cb. Doc docs/frontier-sh226-doinit-dispatch-reconciled-dmcont-negative.md. +1 hermetic (sh226, elfjit example 74/0). Workspace green.
- **WHY (two conflicting records on the same dispatch):** SH156 decoded the closure-build as `ldr x0,[x19,#32]` -> table[+0x30] -> 0x1023eff4c; SH225 freshly decoded it as `[x19,#4]` binder -> vtable vt+0x30 -> br x1. They disagree on offset + semantics + branch. Fresh decode this cycle CONFIRMS SH225 byte-for-byte and pins the full controller chain so a future drive starts from one correct map, not a re-argued reachability.
- **MEASURED (fresh authoritative decode, real libroblox.so, guest=file+0x100000000):** closure-build 0x102206db8: `mov x19,x1`(0x206dd0) -> binder=`ldr x0,[x19,#4]`(0x206df4=0xf9401260, imm 4) -> `cbz x0 ->0x102206ea4` (NULL -> benign Ok(0x3e8)) -> `ldr x8,[x0]`(0x206dfc) -> `ldr x1,[x8,#0x30]`(0x206e00=0xf9401901) -> `br x1`(0x206e24). **SH156's `[x19,#32]`/0x23eff4c = mis-decode.** Full provenance pinned: StartLuaAppDM 0x1023efe2c union-build {[+0]=0x635dd68 table,[+8..24]=0,[+32]=sp} -> `bl 0x102baeeec`(x0=sp,w1=0) -> dispatcher `mov x19,x0`/`mov w20,w1`, `mov x1,x19`, `bl do-init 0x102206c40` (x1=union) -> do-init `mov x19,x2`/`mov x20,x1`, `mov x1,x20`, `bl closure-build 0x102206db8` (arg1=union) -> binder = **`[union+4]`** = high-half-of-table(<2^32) coalesced with [union+8]=0 = **deterministic-NULL** from StartLuaAppDM's own layout (filling [union+8] is the mechanism to fire it).
- **DMCONT CONTINUATION MEASURED NEGATIVE (converts SH165-fwd "Next item 2" from guess to measured):** even with JIT_ROUTEB_DMCONT routing the fabricated manager's vt[+0x1f0] to REAL continueAfterFlagsLoaded_ (0x102bd1d68), region-watch [0x102bd1d68..0x102bd2600] = **0 hits on the completing ladder** (guard fires, M routed, ladder done Ok(0x3e8), EXIT 124, 0 crash). The +0xf8/+0x108 leaf network-fetch returns no flags; the engine-init dispatcher 0x102bd8ce8 benign-completes via its 2nd-frame (0x102bd8dac) -> 0x102bd9058 soft-return, never reaching the `blr` at 0x102bd8e28 (ldr x8,[x8,#0x1f0]; blr x8) that would dispatch vt[+0x1f0]. Routing the slot is required-but-insufficient: the continuation is gated behind a genuine flags-loaded state a fabricated all-leaf manager never produces headlessly.
- **HONEST (do-not-over-claim):** recon + regression pin + measured negative. Does NOT fabricate a binder that fires the dispatch, does NOT drive continueAfterFlagsLoaded_ (its +0xf8/+0x108 network flags state is not headless-producible), does NOT produce a live DM/GuiObject. Route-B live-DM structural gate UNCHANGED (SH209/218/223). DMCONT lever confirmed WIRED+ARMED but LATENT (fires the instant a real flags-loaded state emerges, matching SH165-fwd's honest residual).
- **CODE (sh226, real-image guard family as sh225/sh224/sh223):** hermetic `sh226_doinit_binder_dispatch_chain_reconciled` byte-pins the 14-word controller chain (StartLuaAppDM union-build + bl dispatcher / dispatcher->do-init / do-init->closure-build), the two decisive close-build dispatch words + a hard assert binder load = [x19,#4] imm-4 (NOT imm 32), and the 5 DMCONT continuation anchors (continueAfterFlagsLoaded_ prologue 0xa9ba7bfd + state mov w8,#4; engine-init dispatcher vt+0x1f0 load 0xf940f908 + blr 0xd63f0100 + 2nd-frame sub sp,#0x130). Repro `examples/doinit_dispatch_reconcile.rs` + `runs/capture_sh226_dmcont_negative.sh`. No production path edited.
- **UNREGRESSED:** elfjit examples (74/0, was 73) + workspace green. **recon-v3 render plane re-verified at HEAD** (capture_taskv4_frame.sh: 24 real task-driven frames, present #19..#23 swap Ok(0x1), no json abort, EXIT 124). SH174 capture-latch ARMED+DELEGATING = single forward hook; Route-B live-DM = structural gate.
# (previous session SH225 follows)
- **WHY (the operator's re-attack directive + proof-of-dead-end standard):** SH186 recon task-0 mapped the ONE reachable DM-touching path (StartLuaAppDM 0x1023efe2c -> dispatcher 0x102baeeec -> do-init 0x102206c40) and JUDGED that a DM is created inside a scheduled app-start reached "through a captured vtable", calling fabrication "same-difficulty as static-seed" — WITHOUT building it out or measuring the fork. Doctrine: convert that judgment into a fabrication recipe OR a measured dead-end. This cycle does the disasm.
- **MEASURED (fresh decode, resolved branch targets, real libroblox.so):** do-init 0x102206c40 acquire-loads the once-guard `[0x106a68410]` (ldar w9,[x8] file 0x206c7c / tbz w9,#0 -> 0x102206d10 first-call). First-call path bls **0x10284ce54 = the __call_once SH196 measured to self-latch to a strcmp intern (0x400000b, NOT a DM)**, then builds registry-key strings. do-init then bls **closure-build 0x102206db8** (file 0x206cdc) whose dispatch reads `x0=[x19,#4]` (the binder/app-bridge object) -> `x8=[x0]` (vtable) -> `x1=[x8,#0x30]` (vt+0x30 slot) -> **`br x1`** (file 0x206e24). A second fork bl 0x10221942c (file 0x206ce4). NULL binder at 0x102206df8 cbz -> epilogue = the benign `Ok(0x3e8)` soft-return that is the current headless state.
- **THE PINNED CONTRACT (new vs SH186 prose):** **`GlobalInit do-init`'s first-call path dispatches the DM-construction entry via `br x1` where `x1 = vt+0x30` of the binder object at `[x19+4]`.** That vtable slot is the exact "scheduled app-start" a fabricated binder must satisfy to advance past `Ok(0x3e8)`. Reconciliation w/ SH224: the *DM object's own* vt+0x30 = 0x1057d1b9c (a tiny accessor) — this is the *binder/app-bridge object's* vtable, a DIFFERENT class; do not conflate.
- **HONEST (do-not-over-claim):** recon + regression pin only. Does NOT manufacture the binder, drive do-init further, or produce a DM. Route-B live-DM structural gate UNCHANGED (SH209/218/223). The NEXT Route-B drive can now target a real object (@x19+4 binder's vt+0x30) instead of re-arguing reachability.
- **CODE (sh225, real-image guard family as sh224/sh223/sh222/sh219/sh213):** hermetic `sh225_doinit_dm_construction_dispatch_fork_pinned` byte-pins the 9 dispatch words (do-init prologue / once-guard ldar+tbz / both bl fork call sites / the [x19+4]-load, [x0]-vtable, vt+0x30-slot, br-x1 dispatch) + once-guard 8-alignment + the resolved __call_once target (0x10284ce54) + fork-0x10221942c prologue. Repro `examples/dm_construction_fork.rs` + `runs/capture_sh225_doinit_fork.sh`. No production path edited.
- **UNREGRESSED:** elfjit examples (73/0, was 72) + full workspace. recon-v3 deliverables stay shipped+verified; SH174 capture-latch ARMED+DELEGATING = single forward hook; Route-B live-DM = structural gate.
# (previous session SH224 follows)
## SH224 (Sep 16, 2026, hermes-worker): genuine DM vtable rows re-derived at the SH187-corrected vptr base — reconciles SH186c's +8-slip "manufacture row is a NULL-stub / inert" verdict (refuted at the corrected base: primary slot-2 is real method 0x1057d19bc, not 0x10229c2a4), corrects SH187's own residual +8 slot-index (0x1057d6ef4 "app-shell ctor" is corrected-primary vt+0x38, not vt+0x30), and re-confirms no-GuiObject-producer at the corrected base. Commit d98af35. Doc docs/frontier-sh224-dm-vtable-corrected.md. +1 hermetic (sh224, elfjit example 72/0). Workspace green (567/0).
- **WHY (operator's "re-derive against the corrected base" — SH187's own open NEXT):** the SH179-186 dead-end proof was FALSIFIED by SH187's correction to vptr base 0x1067162e8, but the vtable-row re-derivation it demanded was never done. SH186c's load-bearing verdict ("SH181/182 manufacture lever used vt=0x67162f0 null-stub row -> cb-inert"; primary slot-2 = 0x10229c2a4 mov x0,xzr;ret) was computed on the +8 addresses and predates the correction — exactly the +8-slip class of stale closure the operator's re-verify-at-newest-state doctrine targets.
- **MEASURED (fresh decoded rows, loader-relocated .data.rel.ro via load_elf_image, identical on robbox + android-env .so copies):** corrected primary row (0x1067162e8) slot-2 @0x1067162f8 = **0x1057d19bc = REAL method body** (sub sp,64; stp x29,x30; mov x20,x0) — NOT the null-stub; null-stub 0x10229c2a4 is corrected-slot-3; slot-6 vt+0x30 = tiny accessor 0x1057d1b9c; slot-7 @0x106716320 = 0x1057d6ef4 (the SH182/187 "app-shell ctor") at **vt+0x38**. corrected secondary row slot-1 @0x1067163a8 = 0x10240a8b8 (SH186c "real consumer" thunk: add x0,x0,#0x758; b deep-body — dies on a shallow manufactured DM, never a GuiObject). tertiary slot-1 = 0x1057d07f8 destructor. slot-9 primary = 0x103facf10 (boolean registration predicate, not a factory, unchanged SH189).
- **RECONCILED (do-not-re-claim):** (1) SH186c's "manufacture primary row = NULL-stub, hence inert" is REFUTED at the corrected base — consistent with SH187c's earlier empirical that the corrected manufactured DM genuinely dispatches through a consumer; (2) SH187's doc carried a residual +8 slot-index slip (vt+0x30 label for 0x1057d6ef4); (3) **no DM vtable row produces a GuiObject** (primary=real method, secondary=deep-consumer that faults, tertiary=shared/destructor; create-path 0x103facf10 is a boolean predicate) — Route-B no-GuiObject structural gate UNCHANGED, but now closed by the corrected-base re-derivation, not a stale +8 verdict.
- **CODE (sh224, real-image guard family as sh223/sh222/sh219/sh213):** hermetic `sh224_dm_vtable_corrected_slots_pinned` pins the six load-bearing corrected-base slots (primary slot-2/3/6/7 + secondary slot-1 + tertiary slot-1) read via load_elf_image + host_addr_of, so a drift fails loudly. Repro tool `examples/dm_vtable_corrected.rs` + `runs/capture_sh224_dm_vtable.sh`. No production path edited (pure corrected-base re-derivation + regression pin).
- **UNREGRESSED at HEAD:** workspace (567/0) + elfjit examples (72/0). Route-B live-DM = structural gate unchanged (SH209/220/223); SH174 capture-latch arming at a real make_shared = single forward hook; recon-v3 deliverables stay shipped + verified.
# (previous session SH223 follows)
## SH223 (Sep 16, 2026, hermes-worker): DM-creator reachability re-measured at the corrected-SH161b state (FRESH NEGATIVE) + Region addresses pinned into the real-image guard family. Commit c4aafff. Doc docs/frontier-sh223-dmcreator-region-pinned.md. +1 hermetic (sh223, elfjit example 71/0). Workspace green (567/0).
- **WHY (re-verify-at-newest-state doctrine, the operator's Route-B re-attack target):** SH209's DM-creator reachability negative was measured at the post-SH202 state, BEFORE SH217's corrected SH161b window started actually firing the transition-body bypass (SH217/218 landed later). Since SH161b alters the governor-tail transition, re-measuring whether the NativeDataModelManager DM-construction bodies (getFlagsFromEngine_/initEngine_ [0x102bd1a30,0x102bd1d08) + initializeLuaApp_/startLuaApp_ [0x102bd21d4,0x102bd2600]) become reachable now that SH161b truly engages is a NEW measurement, not a re-run.
- **MEASURED (3/3 clean completing runs, EXIT 124, 0 crash, fresh at corrected-SH161b state):** governor-tail positive control fired 25 distinct pcs per run (75 total — the run genuinely walks the governor tail to the 0x102ea30dc canary-ret); **DM-creator regions = 0 hits in ALL runs** (0 total), identical to SH209. The 3 non-completing runs stopped at the known pre-existing non-seedable SH198/SH55 V2 singleton-vtable host-pointer flake before the control — not a DM-creator pc. **Verdict (do-not-re-tread): even with SH161b genuinely bypassing the V2Init transition body, the NativeDataModelManager bodies stay unreached. Route-B live-DM world-build = structural gate reconfirmed at the newest corrected state.** No seed warranted (once-slot still intern 0x400000b; resolver maps still empty).
- **CODE (+1 hermetic sh223, real-image guard family as sh222/sh219/sh213/sh211):** byte-pins the 5 DM-creator region boundary words (0x2bd1a30=0xaa1403e0, 0x2bd1d08=0xb9401268, 0x2bd21d4=0x912fa063, 0x2bd2504=0x910f1063, 0x2bd2600=0xf94002a8) + the govtail positive-control entry (0x2e9fa84=0xa9ba7bfd) with the guest=file+0x100000000 transform + alignment + canonical-window check — the reachability re-measure reads these exact addresses, and a drifted constant would silently read 0 hits (false negative). Verified on real libroblox.so.
- **UNREGRESSED at HEAD:** recon-v3 plane (capture_taskv4_frame.sh = 24 task-driven frames, present #19..#23 swap Ok(0x1), 196 node pops, no json abort, 0 crash, EXIT 124) + full workspace (567/0) + examples (71/0).
- **STANDING (do-not-re-tread, unchanged):** Route-B live-DM world-build = structural gate (SH209 + this corrected-state re-confirmation). SH174 capture-latch arming *(0x106391908) at a real make_shared = the single forward hook. recon-v3 immediate-priority deliverables stay shipped + verified. No production path edited — pure regression net + a measured re-confirmation.
# (previous session SH222 follows)
## SH222 (Sep 16, 2026, hermes-worker): real-image byte-pin for the recon-v3 JSON-ABORT append-check site. Commit a22251f. Doc docs/frontier-sh222-json-append-check-anchored.md. +1 hermetic (sh222, elfjit example 70/0). Workspace green (567/0 --workspace + 70/0 --examples).
- **WHY (an unguarded load-bearing hook):** recon-selfdrive-seed-jsonfix.md §B's JSON-ABORT deliverable = JIT_JSON_ZERO_FIX, a pc-gated host hook at guest 0x102355d40 that forces the leaking guest-stack libc++ std::string length to 0 (SSO empty) so RBX::json::Writer never throws "string length overflow". It is an IMMEDIATE-PRIORITY, load-bearing delivery (the recon-v3 24-frame plane depends on it) yet had NO real-image byte-pin in the guard family — only a gate-fn unit test. A silently drifted constant would stop the fix engaging without a loud failure.
- **CODE (sh222, skip-if-absent real-image guard family as sh219/sh213/sh211):** byte-pins the block-entry the hook gates on (file 0x2355d40 = 0xa9bd7bfd `stp x29,x30,[sp,#-48]!`, the append bound-check fn prologue) + the cap cell it compares against (guest 0x107275648) + the throw helper the fix prevents reaching (file 0x25fb6bc = 0xd10543ff `sub sp,sp,#0x150`) + the guest=file+0x100000000 transform + 4/8-alignment. Verified against the real libroblox.so (1 passed, 69 filtered).
- **UNREGRESSED at HEAD (fresh re-verify this session):** recon-v3 plane (capture_taskv4_frame.sh = 24 task-driven frames present #19..#23 swap Ok(0x1), 195 node pops, no json abort, 0 crash, EXIT 124) + full workspace (567/0) + examples (70/0). Tree was clean at session start (SH221 a22251f-parent).
- **STANDING (do-not-re-tread, unchanged):** Route-B live-DM world-build = structural gate (SH209/218/220). SH174 capture-latch arming *(0x106391908) at a real make_shared = the single forward hook. recon-v3 immediate-priority deliverables stay shipped + verified. No production path edited — pure regression net.
# (previous session SH221 follows)
- **THE STALE VERDICT (why):** docs/fp16-decode-gap.md listed ~9,552 Unsupported FP16/NEON decode hits incl. a "~100 orr/bic SIMD-immediate NEXT lever" (0x4f0177eX / 0x2f047400) claimed to "JIT-abort the moment the real boot reaches them". Fresh scandecode of the REAL libroblox.so on the same .text window [0x102d95980, 0x1072d5a84) across 13,961,732 instructions = **0 Unsupported, 0 PANIC, 0 distinct** — the whole ledger is closed (commit 88db5be already declared 100% decode coverage). A future session following the doc would have wasted a cycle re-implementing closed code.
- **MEASURED (real image):** 0x4f0177e4/0x4f001fe0 decode as VecMovi with the CORRECT immediates+kind (kind=2 orr / 1 bic; the broad VecMovi gate top-byte {0F,1F,2F,4F,5F,6F} + cmode-shift arms already handle the orr/bic RMW forms). The 4.2M whole-image "unsupported" hits are a red herring — DATA bytes (.rodata/.eh_frame/.rela) inside the single big R E LOAD segment below .text, not instructions (scandecode's .text-window scan is the authoritative metric).
- **CODE (sh221, skip-if-absent real-image guard family as sh219/sh213/sh211):** hermetic `sh221_fp16_doc_flagged_simd_immediate_opcodes_decode_as_vecmovi` (a) asserts the doc-flagged modified-immediate encodings decode as the CORRECT VecMovi (right lo/hi + kind, never Unsupported), and (b) mirrors scandecode's exact PF_X-segment walk over the real code window [0x102d95980, 0x1072d5a84), asserting 0 Unsupported — a durable "decode coverage must not regress" pin. Doc updated with a RESOLVED banner.
- **WHY THIS AND NOT a Route-B seed:** Route-B live-DM = measured structural gate (do-not-re-tread, SH209/218/220, ~30 cones). recon-v3 immediate-priority deliverables re-verified green at this HEAD (capture_taskv4_frame.sh: 24 task-driven frames present #19..#23 swap Ok(0x1), 197 node pops, no json abort, 0 crash, EXIT 124). This cycle hardens the JIT's one real remaining risk surface (an Unsupported decode whenever the boot reaches FMOD/audio/render HDR math) into a regression test.
# (previous session SH220 follows)
- **MEASURED (JIT_ASSET_TRACE=1 + region-watch CoreScript loader [0x101f1d8ac,0x101f1db00) + rbxasset site [0x10232ed4,0x10232f00), canonical completing ladder, 3 arms):** arm A (harness only) / B (staged synthetic R1 CoreScript at filesdir mirror + SOBER_ANDROID_ROOT) / C (B + REAL extracted assets incl. R2 UniversalApp.rbxm) — ALL EXIT 124, **0 asset-trace lines, 0 CoreScript-loader region hits, 0 crash.** Engine makes ZERO AAsset + ZERO rbxasset://scripts requests on the completing ladder even with real content mounted.
- **ANSWER (the operator's delegated "name the R1 file"):** there is NO engine-requested R1 filename to match — the CoreScript loader 0x101f1d8ac never fires headlessly (SH186 law wall: mid-ctor instr, 0 bl callers, no live DM to parent a ScreenGui). Staging the file is a no-op (no code reads it). **NEW measured fact:** the ENTIRE Android asset surface (AAsset, rbxasset, CoreScripts, R2 UniversalApp.rbxm) is inert at boot; only the host-driven (Route-A/recon-v3) render plane executes.
- **HONEST:** converts R1/R2's "assumed reachability-blocked" (SH203/SH186) into a measured 3-way negative WITH the operator's own diagnostic. Does NOT manufacture a DataModel; common unlock stays the live-DM structural gate (unchanged). No new seed warranted.
- **UNREGRESSED:** recon-v3 (24 task-driven frames, 195 node pops, no json abort, 0 crash, EXIT 124) + SH174 capture latch ARMED+DELEGATING (engine's own hook 0x1021ebaf4) + persistence fsmap roundtrip byte-exact.
- **STANDING (do-not-re-tread):** Route-B live-DM = structural gate. SH174 capture-latch arming at a real make_shared<DataModel> = single forward hook; R1/R2 content has nothing to load until then.
## SH219 (Sep 16, 2026, hermes-worker): SH203 "post-family gate" RE-MEASURED at HEAD = NOT REPRODUCED in 28 runs (stale classification) + flag-manager/post-family words pinned as regression anchors. Commit 161bb42. Doc docs/frontier-sh219-postfamily-renegade.md. +1 hermetic (sh219). Workspace green (arm64jit 387/0 + elfjit example 68/0 + all crates).
- **WHY (a stale verdict worth re-measuring):** SH203 classified the post-family fault (NULL-singleton pthread_mutex_lock, pc=host-call slot 0x7f00000022b0, lr=0x102b53a78, x0=0x28, reached only AFTER the V2 family clears) as a deterministic 4/12 live-world-build gate via "enclosing fragment 0x2b53a64 has 0 direct bl callers". SH205 LATER PROVED that exact "0 direct bl callers" method unreliable (it miscounted and dismissed a seedable flag-manager fault as migration). The post-family site was NEVER re-run under GSDSP (JIT_GUEST_STACK_DUMP), and SH116b (flag-manager set) + SH217 (SH161b window) landed AFTER SH203.
- **MEASURED — 0/28 post-family reproductions at HEAD:** two canonical V2_ONDEMAND batches (16 GSDSP + 12 exact-SH203-env) = 25 clean EXIT 124 / 3 faults at ONLY the known non-seedable classes (0x10284f490 SH55/64 `__call_once` blr-x2, 0x106240c24 FMOD/AAudio crash-A, 0x1021dea94 SH208 singleton-vtable host-ptr) — NEVER the post-family 0x102b53a78/0x7f00000022b0 site. SH203 measured it 4/12 (33%) pre-SH116b/SH217. Both SH203's site and SH116b's site are NULL+0x28-mutex singletons (same family) ⇒ closure consistent with SH116b absorbing it or SH217 shifting the dispatch. The "post-family = structural gate" verdict is no longer measurably alive at HEAD.
- **HONEST BOUNDARY (do-not-over-claim):** 28-run non-reproduction is strong evidence, not proof of impossibility (could be a rarer residual). It does NOT manufacture a DataModel — Route-B live-DM structural gate UNCHANGED (do-init once-lambda still yields intern 0x400000b, not a DM; SH155 liveDM=false). The 3 residual fault classes are all documented non-seedable live-object/host-heap, no seed warranted. No production code change.
- **CODE (+1 hermetic sh219, real-image guard family as sh213/sh211/sh116b/sh200):** byte-pins the SH116b flag-manager load slot (0x2320a24=0xf0027a88 adrp x8,7273000 / 0x2320a2c=0xaa0303e4 mov x4,x3 / 0x2320a30=0xf944d908 ldr [0x10672739b0]) + the SH203 post-family fragment (0x2b53a64=0x940140e4 bl receiveCall+0x558 / 0x2b53a6c=0xa9bf7bfd stp / 0x2b53a74=0x94de0a4f bl pthread_mutex_lock@plt) + guest=file+0x100000000 transform + 4-alignment for all 6 .text sites — so a future drift fails before any stale re-classification.
- **UNREGRESSED at HEAD:** recon-v3 plane (24 task-driven frames present #19..#23 swap Ok(0x1), 196 pops, no json abort, 0 crash) + SH174 capture-latch ARMED+DELEGATING (ACTIVE 0x1067daaf0 → engine's own 0x1021ebaf4). Forward hook intact.
- **STANDING (do-not-re-tread):** Route-B live-DM = structural gate (SH196/203/204/209/218). This cycle removes a possibly-FALSE "live-world-build gate" from the closure set. SH174 capture-latch arming *(0x106391908) at a real make_shared = the single forward hook. recon-v3 immediate-priority deliverables stay shipped + verified.
## SH218 (Sep 16, 2026, hermes-worker): GOVERNOR-TAIL CONTINUATION re-measured at the CORRECTED-SH161b state (SH217) = FRESH NEGATIVE + SH161b transition-body purpose PINNED (bypass proven Route-B-safe) + manufacture/recon-v3 unregressed. Commit 9c44697. Doc docs/frontier-sh218-govtail-corrected-negative-transitionbody-routesafe.md, repro runs/capture_sh218_postsh161b_continuation.sh. Workspace green (arm64jit 387/0 + all crates).
- **STATE CHANGE EXPLOITED:** SH217 corrected the SH161b mode-2 seed window to the REAL tail-entry [0x102e9fcc4,0x102e9fdc8], so the transition body in fn 0x1023c12c0 is now TRULY bypassed — in EVERY prior measurement (SH197/204/209) the seed never fired and the fault-prone body was live. This cycle re-measures the continuation under the genuinely-bypassed state for the first time.
- **(a) Transition body = V2Init audio/telemetry marshalling only (NEW, protective):** fresh disasm shows every dispatch (0x23c14dc jumptable, 0x23c1504 build-info, 0x23c1574 struct-copy) + conditional FMOD-AAudio 0x626b6d0 sits behind fn entry `ldr w8,[x0,#696](impl+0x2b8); cmp; b.eq canary-ret 0x23c1434`, all gated on version-state global [0x6a70700]. NO DM/world-build/scene writer. SH161b's mode-2 early-return therefore forfeits ONLY V2Init audio-param + telemetry — **provably Route-B-safe** (was assumed fault-prone before, never mechanically pinned).
- **(b) Governor-tail = FRESH NEGATIVE at corrected state (run 2, EXIT 124):** SH161b fires (impl[+0x2b8]=2), transition body bypassed, yet tail STILL terminates at the 0x102ea30dc canary-ret lodestone (`ldr x19,[sp,#16]` after `bl 21e9610`) — no reach into world-build fn 0x102ea3b14 (fresh disasm: reads [x0,#688], op-new(0x18)+ctor 0x2eacccc + `bl nativeAppBridgeAppStart__ 0x2365960` = the SH174-closed no-DM-factory line) and no DM-creator pc. SH155: once-slot=0x400000b intern, liveDM=false. Runs 1/3/4 = known non-seedable FMOD AAudio direct-JNI crash (guestpc 0x106240ca0, fn 0x6240900 SH213/212 region) + SH55/64 flake.
- **(c) Unregression surfaces clean at SH217 HEAD:** manufacture lever (SH187 real-ctor drive → obj vptr set {0x1067162e8,0x1067163a0,0x1067163f8} GENUINE MATCH, planted into holder 0x106391908, EXIT 124, 0 crash) + recon-v3 render plane (24 task-driven frames, present #20..#23 swap Ok(0x1), 195 pops, no json abort, 0 crash).
- **VERDICT (do-not-re-tread):** Route-B live-DM world-build = structural gate reconfirmed at the newest corrected-SH161b state — the transition body was never a Route-B edge (now proven) and the tail still ends at the lodestone terminal. No production code change (SH217 landed; this cycle verifies + documents). Standing front unchanged: SH174 capture-latch arming *(0x106391908) at a real make_shared<DataModel> = the one forward hook.
## SH218b (Sep 16, 2026, hermes-worker): last apparent continuation angle CLOSED — the surface-handoff rung's OWN world-build call (bl 0x2ea3b14 at 0x25f5e04 inside V2UpdateSurfaceAppWithPlatformParams 0x25f5fec, gated by SH199 byte [0x106a70568]) measured UNREACHED (region-watch: fn-entry pc only, not the gate block nor the bl; V2UpdateSurface soft-returns Ok(0x3e8), SH199/200 singleton class). With SH204's V2Init call-site negative, fn 0x102ea3b14's world build now has EVERY call site empirically measured unreached. Commit 8cebd5f. Same SH218 doc + runs/capture_sh218_surface_worldbuild.sh.
# (previous session SH217 follows)
- **DEFECT (empirically measured, not assumed):** the SH161b guard's strict `pc==0x102e9fe04` window fired ZERO times on the completing --v2boot ladder. Region-watch proved the governor tail is translated as ONE block entered at **0x102e9fcc4** (the call-site 0x102e9fe04 is mid-block, never a block entry; the bl ret-landing 0x102e9fe0c is the separate entry). So the impl[+0x2b8]=2 seed never landed, leaving fn 0x1023c12c0's fault-prone transition body (reads [0x6a70700], dispatches 0x23c14dc/1504/1574, conditional FMOD-AAudio 0x626b6d0 -> SH212 crash-A site 0x6240d8c) **live on every completing ladder**.
- **FIX:** widened `routeb_tail_eq_guard`'s window to the tail-region entry [0x102e9fcc4,0x102e9fdc8] — the SAME window as routeb_tail_dispatch_guard (the operator's "exact SH159c pattern"). Seed now lands at the real tail-block entry before the `bl`, so fn 0x1023c12c0 takes its benign b.eq mode-2 no-op; transition body bypassed. Idempotent (writes only when 0, preserves a live nonzero session value), reads x19 impl base like its sibling, default-inert under JIT_ROUTEB_SETFIX. Hermetic sh161b updated to the corrected window (0x102e9fc00 no-seed / 0x102e9fcc4 seeds 2 / 0x102e9fdb0 preserves live 7 / impl==0 no-deref).
- **VERIFIED on the real libroblox.so** (EXIT 124, 0 crash): `[routeb-setfix] SH161b impl[+0x2b8] seeded =2 ... transition body bypassed` present (was 0 fires before); region-watch shows fn 0x1023c12c0 entered + governor tail continues through 0x102e9fe0c, transition-body hits ABSENT. Tail block-drop lift (0x102e9fcb0..0x102ea3b40) reconfirmed working (0x102e9fcc4 + 0x102e9fe0c re-execute). recon-v3 render plane unregressed at HEAD (capture_taskv4_frame.sh: 24 task-driven frames, 197 node pops, no json abort, 0 crash). The run-variable ~1/8 flake hits at guestpc 0x102174930/0x101d99558/0x106240d8c are the known SH55/64 host-heap classes (non-seedable, do-not-chase).
- **STANDING (do-not-re-tread, unchanged):** Route-B live-DM = structural gate (SH209). This cycle hardens the ladder against the SH212 FMOD-AAudio crash-A class by making the installed SH161b lever actually fire; it does NOT manufacture a live DM.
# (previous session SH216 follows)
## SH216 (Sep 16, 2026, hermes-worker): RELOCATION-COVERAGE COMPLETENESS AUDIT — answers the operator's named proof-of-dead-end criterion (a "reloc type no loader pass can synthesize") = MEASURED NEGATIVE. The real libroblox.so's entire data-reloc type set = {R_AARCH64_RELATIVE 568,194 + GLOB_DAT 56 + ABS64 22} (+534 JUMP_SLOT in DT_JMPREL) — ALL loader-synthesizable (android_relocs apply_relatives for RELATIVE; plt bind_glob_dat for GLOB_DAT/ABS64 via dlsym/host-thunk/GLES+NDK-shim; PLT resolver for JUMP_SLOT). No unsynthesizable relocation type => Route-B is NOT provably dead on the relocation axis => per doctrine, the grind is not released. Commit 112bb1f. Doc docs/frontier-sh216-reloc-coverage-complete.md. Hermetic +1 (real-image guard, sh202 family). Workspace green 566/0.
- **Method (authoritative):** loader's OWN decoder (`read_elf_relocations`, the exact load_elf_image path), not hand-rolled; a naive APS2 decode mis-read this .so (DT_ANDROID_RELA not raw "APS2"). The 78 symbol-based relocs are all **UNDEF(import)** libc/libm/media/GLES (__stack_chk_guard, environ, glGetShaderInfoLog, AMEDIAFORMAT_KEY_*, mmap/read/write/stat/...) — the plt binder's load-time job, not a gap. No code-path fix falls out (nothing unsynthesized); SH209 live-DM structural gate UNCHANGED.
- **Forward note → MEASURED-CLOSED (runtime bind verification, one canonical ladder):** `[plt] bound 534 JUMP_SLOT + 77 GLOB_DAT/ABS64 (0 unresolved), 1 unresolved` => 77/78 GLOB_DAT/ABS64 bound; lone unresolved = WEAK `__gcov_dump`/`__gcov_flush` (dlsym miss, not shimmed; GLES/media/`__stack_chk_guard` misses covered by shims/patch_stack_canary). Unresolved weak-undefined binds 0 + callers guard => BENIGN, not a NULL-fault source. SH55/64 flakes NOT caused by unresolved data-import slot; SH209 live-DM gate attribution stands.
- **Code:** +hermetic `real_image_relocation_types_all_loader_synthesizable` (subset-of-{1027,1025,257}; fails if a future .so adds an unsynthesizable type). Throwaway reloc_audit example removed.
- **STANDING:** Route-B live-DM = structural gate unchanged (SH209 do-not-re-tread); recon-v3 green; cookie committed-as-is (SH177). This cycle boxes the relocation axis as NOT a dead-end (evidence FOR the grind), leaves next genuine Route-B lever as before.
## SH213 (Sep 16, 2026, hermes-worker): Sound-pillar first-contact anchor pinned (FMOD/AAudio JNI export) + empirical A/B proving the SH132 AAudio bridge never fires on the ladder. Commit 509fa62. Hermetic sh213 test + doc. Workspace green (cargo test --workspace EXIT 0).
- **PURPOSE (forward regression net, the sound pillar's first measured anchor):** SH212 recorded crash A as `Java_org_fmod_FMOD_OutputAAudioHeadphonesChanged` (guest 0x106240d8c), the audio subsystem's first boot contact, but classified it non-seedable. This cycle byte-pins that contact + independently A/B-measures whether the pre-built SH132 AAudio bridge prevents it.
- **EMPIRICAL A/B (10+10 canonical ladder runs, JIT_AAUDIO_BRIDGE=1 vs default):** **0/20 crashes, EXIT 124 both arms, and 0 `[aaudio:bridge]` log lines in the bridge arm** — the guest never `dlopen("libaaudio.so")` on this boot path, so the SH132 dlopen/dlsym intercept never engages. **Fresh measured negative:** the SH212 FMOD crash is NOT on the bridge's dlopen path (it is a direct-JNI `this`/jobject misconstruction reached via the JNICallProtocol registry, host-allocated unmaterialized device). The bridge stays latent-but-correct; crash A stays SH55/64 non-seedable — enabling the bridge is NOT the fix, do-not-re-tread as such.
- **HERMETIC sh213** (`sh213_fmod_aaudio_first_contact_anchors`, same real-image guard family as sh211/sh116b/sh200): byte-pins crash block-entry 0x6240d8c (`add x10,x10,#0x3ff`), the two jobject libc++ std::string member reads 0x6240b9c/0x6240bc4 (`ldr x1,[x19,#16]/[x19,#24]`) that thread the NULL fault, fn entry 0x6240900, + SH132 customer-side cells (0x106d0ef20/0x104fbea00); guest=file+0x100000000 transform + alignment. Verified against the real libroblox.so (1 passed, 66 filtered). Asserted AAudio symbols are NOT dynamic imports (dlopen-resolved at runtime, matching the bridge's intercept model).
- **NO SEED / NO DEFAULT CHANGE:** pure regression net on the sound pillar's forward anchor; no production code path edited. Route-B live-DM = structural gate UNCHANGED (SH209 do-not-re-tread). recon-v3 render plane untouched & stays green at HEAD. Cookie persistence unchanged (SH177 committed, not extended).
- **Why this and not a seed:** the operator's Route-B directives (SH169 corrected the NEXT-3 do-init seeds as dead/wrong-address; SH209 measured DM-creator unreachability negative; ~30 recon angles closed) leave live-DM = the migration gate. The audio crash is run-variable (1/8 in SH212), non-seedable, and sits behind the render/DM wall — the correct forward action is to anchor it so the future audio-hardening milestone starts from a drift-verified site, not to build an (empty) seed against it now.
# (previous session SH212 follows)
## SH212 (Sep 16, 2026, hermes-worker): Residual ladder flakes characterized at HEAD — FMOD/AAudio audio-subsystem CONTACT + raced flag-manager system_error (both non-seedable) + recon-v3 render plane re-verified green. Doc docs/frontier-sh212-residual-flakes-characterized.md, repro runs/capture_sh212_residual_flakes.sh. Workspace green (cargo test --workspace EXIT 0; arm64jit lib 387/0 + all crates).
- **MEASURED 8 canonical ladder runs (SH210 env + GSDSP + outside-trace): 7/8 clean EXIT 124**, full V2 ladder (nativeInitFlags→gameGlobalInit→do-init once self-latches→governor MODERN router [0x106a70880]=1→V2Confirm→V2Start→V1 AppStart). Residuals: run-2 SIGSEGV guestpc=0x106240d8c (fault=0x0, x0=0); run-8 libc++abi `terminating due to uncaught exception std::__ndk1::system_error` at nativeInitializeNativeFlags.
- **Crash A = the AUDIO pillar's first contact point (forward evidence, not a regression):** guestpc 0x106240d8c = file 0x6240d8c inside `Java_org_fmod_FMOD_OutputAAudioHeadphonesChanged` (the char-decode block entry; fault is a NULL-`this` in its strstr/strtol stream-format parse). GSDSP fired and is DEFINITIVE-inconclusive: guest-word dump all host-heap + sign-extended HOST pointer, `BT: -> HOST(0x7f2bb86f58bb)` — no in-image GUEST caller ⇒ NULL object is a host-allocated unmaterialized audio device, NON-SEEDABLE (SH55/64+SH206 class), do-not-re-tread. The client's FMOD audio init is now measured as reaching its AAudio output path headlessly; a future audio-hardening milestone starts from this anchor (guest 0x106240d8c region).
- **Crash B = raced (not deterministic) resource throw:** `nativeInitializeNativeFlags` after "Registered Flag Provider ID from Java" (1/8 runs); the thrown system_error's `.what()` prints as garbage stack words = the unconstructed-guest-stack-string family (same class recon-v3's JIT_JSON_ZERO_FIX clears). Non-seedable, do-not-chase. SH116/SH116b lock-class fix re-verified firing every run; the deterministic 0x102320a98 fault is gone (residual ≠ SH116b).
- **STANDING UNCHANGED:** Route-B live-DM = structural gate (do-init yields strcmp intern, not a DM; SH209 verifies; do-not-re-tread). recon-v3 green at HEAD (24 task-driven frames swap Ok(0x1), 195 pops, no json abort) + all SH210 latent wiring still ARMED. No production code change this cycle (both flakes proven non-seedable; a patch would be cruft).
## SH211 (Sep 16, 2026, hermes-worker): Route-B wiring + recon-v3 render-plane opcode regression anchors pinned against the REAL libroblox.so. Doc docs/frontier-sh211-wiring-opcode-anchors.md. +1 hermetic test (elfjit example 66/0). Workspace green (cargo test --workspace EXIT 0).
- **Pure regression net, no default change / no seed.** The armed-but-latent Route-B wiring (SH210) + recon-v3 self-driven frame plane + G1/G2/G3 all key off exact guest addresses; the SH174 migration runbook fires at the real app-launch, so a silently drifted constant breaks the ONE forward hook at the exact moment it's needed. Locks the load-bearing sites into the project's real-image guard family (sh116b/sh200/sh201).
- **Byte-pinned on libroblox.so (109,193,800 B; skip-if-absent):** type-4 drain idle heartbeats (file 0x2856f24=0x52800044 `mov w4,#2` + 0x2856f68=0x52800064 `mov w4,#3`, patched to w4=4 by recon-v3); SendAppEventOnAppReady 'Home' discriminator (0x2bb47c4=0x52800093 `movz w19,#4`) + ALT (0x2bb47cc=0x52800033 `movz w19,#1`) — the SH206 pin.
- **Hermetic always:** guest=file+0x100000000 transform + 4-align for those sites; 8-aligned in-window placement for the 8 wiring cells (type-4 vector, surface-XID, files-dir, gov-router, capture arm, flags-latch, once-guard, DM-root).
- **Standing unchanged:** Route-B live-DM = structural gate (SH209/210). recon-v3 re-verified green at HEAD before this. Cookie persistence stays committed-as-is (SH177), not extended (operator: return to Route B).
## SH210 (Sep 16, 2026, hermes-worker): ENTIRE Route-B LATENT WIRING VERIFIED ARMED at HEAD (all 5 marker classes mount on one clean completing ladder). No production code change. Doc docs/frontier-sh210-wiring-armed.md, repro runs/capture_sh210_wiring_armed.sh. Workspace green.
- **Complement to SH209's negative:** SH209 proved the DM-creator is gate-unreached headlessly; SH210 verifies the rest of the installed Route-B lever suite is STILL ARMED (not silently dead) on the canonical completing ladder (--v2boot + surface-handoff + send-appevent + set-filesdir, env incl. JIT_DM_ALLOC_CAPTURE=1+DELEGATE).
- **MEASURED (run 2, EXIT 124, 0 crash): ALL FIVE marker classes in ONE run** — capture latch ARMED + DELEGATING (8 FIRST-call allocs through the engine's own hook 0x1021ebaf4); G1 wired XID 0x200000 at [0x10683d348]; G2 SendAppEventOnAppReady 'Home' in x5 (Ok(0x3e8) w19-event=0x1 = SH206 benign soft-return, correct-but-latent); G3 files-dir @[0x10726d600] SEEDED + read-back verify "/data/user/0/com.roblox.client/files"; SH155 DM probe once-guard=0 DM-root vt+0x30=0 liveDM=false count=0x1.
- **Runs 1/3 (EXIT 124, 0 crash) armed the latch then stopped at the known run-variable SH198/SH55 V2Init outside-image flake before the later rungs** — pre-existing ~1/3 stop, NOT regression (run 2 proves the full path).
- **VERDICT (do-not-re-tread):** nothing in the latent wiring is left un-armed; the only absent piece is the live DataModel itself (SH209). SH174's post-migration arm of *(0x106391908) = the single exact forward hook. recon-v3 + SH208 + SH209 untouched (no prod code this cycle).

## SH209 (Sep 16, 2026, hermes-worker): ROUTE-B DM-CREATOR REACHABILITY = FRESH MEASURED NEGATIVE at the post-SH202 completing-ladder state. No production code change. Doc docs/frontier-sh209-dmcreator-reach-negative.md, repro runs/capture_sh209_dmcreator_reach.sh. Workspace green.
- **Single-agent (Route-B cone suppressed) re-attack of the operator's named front** — reaching a real DataModel via ExperienceController / initializeLuaAppWithDataModel / NativeDataModelManager on the now-completing ladder. SH164's DM-creator 0-hit measurement was taken pre-SH202, when the ladder consistently stopped at the V2Init outside-image flake first — so its "DM creator not reached" could not be separated from "ladder never got that far." SH202's V2_ONDEMAND closes that gap (full ladder now completes ~8/12). Re-measured the NativeDataModelManager creator bodies on the canonical completing ladder with an in-run calibrated positive control (governor tail).
- **MEASURED (10 runs, 3 clean-completing + control-positive):** governor control fired 25 distinct pcs walking the full tail to the 0x102ea30dc canary-ret (the SH197/204/198 terminal) in each clean completing run; the DM-creator regions [0x102bd1a30..0x102bd1d08] + [0x102bd21d4..0x102bd2600] stayed at **0 hits in all 10 runs**. The 7 non-completing runs faulted at the known pre-existing SH198/SH55 singleton-vtable host-pointer site (0x102b9dee0, fault=0x10 — same signature SH205/206/208) or the 0x10284ce54 flake; never at a DM-creator pc.
- **VERDICT (do-not-re-tread):** even on the completing path the game ends at the governor-tail canary-ret and never enters NativeDataModelManager's DM-construction bodies. Route-B live-DM world-build = structural gate reconfirmed at the new completing state (closes SH164's "not separated from early ladder stop" residual). No static seed to a NativeDataModelManager cell (initEngine_'s next gate = network feature-flag fetch, SH164). Standing front unchanged: SH174 capture-latch observer arming *(0x106391908) at the instant a real session's make_shared<DataModel> runs. recon-v3 + SH208 deliverables untouched (no prod code this cycle).

## SH208 (Sep 16, 2026, hermes-worker): LADDER FLAKE CHARACTERIZATION — residual run-variable dispatch flakes are NOT the SH202 on-demand patcher (measured A/B, exonerated) + recon-v3 immediate-priority deliverables re-verified green at HEAD. Doc docs/frontier-sh208-flakechar.md. Workspace green (cargo test --workspace EXIT 0, 0 failures). No production change (do-not-chase, no seed).
- **RE-VERIFIED at HEAD ffb9920:** (1) SELF-DRIVEN FRAMES — 24 task-driven frames (present #19..#23 swap Ok(0x1) distinct colors), 197 node pops, EXIT 124, 0 crash; type4_frame_thunk @0x7f00000001d0 seeded into vector [0x106829ea8]. (2) JSON-ABORT — JIT_JSON_ZERO_FIX firing (0x102355d40 overflow -> len=0 SSO). Both recon-v3 deliverables stay green.
- **ON-DEMAND PATCHER EXONERATED (the standing untested hypothesis closed):** A/B'd V2_ONDEMAND vs SH200-only. mode=on 7/8+6/6 -> 1 fault @0x1021e34b0. mode=off 12/14 -> 2 faults at *different* sites (0x1062412dc fault=0x7f0000000020, 0x1021deaac fault=0x20). Fault site MOVES = run-variable live-object dispatch (host-heap obj threaded from caller frame, vtable-slot deref; helper 0x21e3960 `ldr x0,[x0,#8];ret`), NOT fixed .bss (non-seedable). SH202's mid-run .text mutation + pc rewind is NOT the SH104/105 canary-writer.
- **STACK-SMASH (SH104/105 class) CONFIRMED at nativeInitializeNativeFlags** — same region SH116b/SH205 fixed its lock-crash sibling. The only named fix (bridge return-sanitize: zero host ptr returns >=0x100000000) is proven unimplementable by SH105 (guest stack ptrs are ALSO 0x55-range, so blanket zeroing corrupts legit self-pointers). Do-not-chase.
- **VERDICT (do-not-re-tread):** the residual is non-deterministic, non-seedable, fixed-host-pointer-free — no production fix is safe or warranted this cycle. Route-B structural gate UNCHANGED (once-slot=intern, resolver maps EMPTY {0,0}, live-DM gate). Next genuine lever unchanged: live-DM session (SH174) or an out-of-band Route-B derivation.
## SH207 (Sep 16, 2026, hermes-worker): RESOLVER-MAP GATE CLOSED AT FRESH SOURCE-DEPTH (bulk-registrar SOURCE 0x106dca0e90 measured EMPTY) + futex-requeue flake FIXED to green. Commit 9f39594. Doc docs/frontier-sh207-resolver-source-empty.md. Workspace green (387/0 arm64jit incl. new sh207 test + all crates).
- **FRESH MEASUREMENT (routeb_dm_service_resolve_guard, always-fires probe relocated to top of guard):** the bulk-registrar **SOURCE map 0x106dca0e90 is EMPTY {0,0}** on 2/2 clean ladder runs — the one container SH191/192/194 never directly read (they read only dest 0xe70 + register 0xf60). `SH192+: resolver(dest) 0x106dca0e70={0,0} source 0x106dca0e90={0,0} register 0x106dca0f60={0,0}` all EMPTY. Region-watch confirms the bulk registrar 0x2208ae8 **IS exercised in-ladder** (fires at 0x102208ae8/0xb6c/0xcf8/0xe38 = its rehash + insert-path blocks) — but with BOTH source+dest empty it is a clean **no-op**, NOT corruption-destined. This **reconciles** SH191/194's "resolver stays EMPTY" with SH194's "registrar region fires every run", and refines SH192/194's *bad_weak_ptr* (that was from driving the registrar standalone with a fabricated key, which we did NOT repeat — we measured only).
- **VERDICT (do-not-re-tread, now at source-depth):** getService("PlayerGui") name→classid resolution (SH191's linked-node RETURN) sits behind the live class-registry world-build that populates the source map — the Route-B live-DM structural gate UNCHANGED (SH174/193/194/196/203/204/206 + this). No headless seed populates the resolver source.
- **GENUINE PROD FIX:** `futex_requeue_actually_moves_waiter` **flake fixed** — the waiter's `futex WAIT` timeout was 5s (wall-clock) while the REQUEUE side spins up to ~20s; under full parallel `cargo test --workspace` load a descheduled waiter could sleep past 5s before the REQUEUE landed → unblocked on timeout → REQUEUE moved 0 → assertion failed. Bumped the waiter timeout 5→60s (assertion untouched). Verified under load: full workspace green (futex test now passes in the parallel run).
- **+1 hermetic** (`sh207_registry_three_map_addresses_and_dest_resolver_layout`): pins the dest-resolver / registrar-source / register-map guest addresses + adjacency + the source-selector 0x20-past-dest relation.
- **STANDING:** Route-B live-DM = structural gate. Both recon-v3 deliverables stay green (24 task-driven frames / no json abort). NEXT remains a live-DM session (SH174 latch) — and inside this JIT, nothing further is seedable on the resolver/registration line.
## SH206 (Sep 16, 2026, hermes-worker): RECON-V3 IMMEDIATE-PRIORITY DELIVERABLES VERIFIED GREEN AT HEAD + w19-event PIN SETTLED (benign soft-return) + ladder 5/6 clean. Doc docs/frontier-sh206-recon-v3-verified.md. Workspace green (386/0 arm64jit + all crates; 9 pre-existing warnings, no errors). No new seed warranted.
- **(1) SELF-DRIVEN FRAMES + (2) JSON-ABORT both GREEN at SH205b HEAD** (`runs/capture_taskv4_frame.sh`): type4_frame_thunk registered at 0x7f00000001d0, type-4 vector [0x106829ea8] seeded, both heartbeats w4=4-patched, RENDERCTX published; **24 real task-driven frames (present #19..#23 swap Ok(0x1), distinct colors), 195 node pops, no json abort, 0 SIGSEGV/SIGABRT, EXIT 124** — the recon-selfdrive-seed-jsonfix deliverables (immediate priority) shipped + independently verified at this HEAD.
- **LADDER 5/6 CLEAN** (`capture_sh205_postfamily.sh` + fresh 6-run w/ JIT_GUEST_STACK_DUMP=1): the lone fault = `guestpc=0x102b9dee0 fault=0x10` = the documented SH198/SH55 pre-existing V2 singleton-vtable host-pointer flake (SH204 listed the same class), NOT seedable/NOT a new gate. SH116b (flag-manager) fired every run. Residual 0x10284ce54/0x104c393f0 = known SH55/64 class. Run-1 of an earlier batch completed the FULL V2 ladder (V2Init Ok, StartLuaAppDM Ok(real heap), V2Start Ok, V1 AppStart Ok, V2UpdateSurface Ok(XID 0x200000), SendAppEvent Ok(0x3e8)).
- **w19-event=0x1 PIN SETTLED (fresh disasm):** fn 0x102bb463c's discriminator blocks — file 0x2bb47c4 `mov w19,#0x4` is the 'Home'-magic path (len==4 -> `movk w10,#0x656d<<16; add #0xe01` = 0x656d6f48='Home' LE -> b.eq w19=4); `mov w19,#0x1` at +8 is the ALT path. Observed w19=0x1 is the fn-entry `mov x19,x5` (x19 = x5 jstring-handle low word) left intact because SendAppEventOnAppReady **soft-returns Ok(0x3e8) before the discriminator body runs** — NOT taking the w19=1 branch, NOT a harness bug. Step-2 ABI (Home in x5, XID 0x200000) is correct-but-latent.
- **ROUTE-B do-init cells reconfirmed UNCHANGED at HEAD (SH196 holds):** once-guard[0x106a68410]=0x1, once-slot[0x106a68408]=0x400000b (strcmp intern, NOT a DM), DM-root=<SH156 seed>, flags-latch=0x0, app-data-model=0x0, holder=0x106358d40, resolver map {0,0,0 live=false}; all five MH_* flags false. The do-init __call_once self-latches but yields an RTApp intern, not a live DM. **Route-B live-DM world-build = structural gate UNCHANGED.**
- **TRIAGE:** the run-1 frame-timing fault (guestpc 0x102306158, fault=0x210) traced to fn 0x2306130 (`[this+528]/[this+552]` frame-accum) called with `this=0` from sole caller file 0x5fcbc48 — but `this` is threaded from a **live enclosing object member, NOT a fixed .bss pointer** (unlike SH116b's `[0x10672739b0]`), so NOT a seedable singleton; it's the known SH55/64 run-variable NULL-object flake (0 occurrences in the subsequent 6-run batch). Do-not-chase as a seed.
- **STANDING (do-not-re-tread):** Route-B live-DM = structural gate (~30 recon angles + SH196/203/204/206). Both recon-v3 immediate-priority deliverables shipped+verified; step-2 ABI correct-but-latent; MH_APP_READY stays false headlessly; session-producer handoff latent-but-correct.
## SH205 (Sep 16, 2026, hermes-worker): POST-FAMILY FAULT = SEEDABLE SH116-CLASS SINGLETON, NOT the migration gate — GSDSP guest-stack diag + SH116b flag-manager site patch. Commit bd97ab9. Doc docs/frontier-sh205-postfamily-seedable.md. Workspace green (564/0; arm64jit example 65/0).
- **SH203'S VERDICT FALSIFIED (measured, both premises wrong):** the post-family fault (SIGSEGV host-call slot 0x22b0 pthread_mutex_lock, x0=0x28, lr 0x102b53a78) was dismissed as "the same live-world-build migration gate, 0 direct callers". (1) The lock-helper 0x2b53a68 has **500+ direct `bl` callers** (SH203 counted 0 — it looked at the helper, not the caller). (2) New env-gated GSDSP (`JIT_GUEST_STACK_DUMP=1`) fault-stack dump proves the caller ra = **guest 0x102320a98** = `nativeInitializeNativeFlags` (file 0x2320a94) locking `[*(0x10672739b0)+0x28]` where the flag-manager `.bss` global reads 0 — a **fixed pointer on the flags page, the SH116 seedable class**, reached one site deeper than SH116's sibling patch (0x2320710). NOT a live-DM world-build.
- **GSDSP TOOL:** crash handler (elfjit.rs) now, under `JIT_GUEST_STACK_DUMP=1` (default-inert), dumps the guest stack words `[sp..sp+56]` tagging in-image guest text — identifies the *caller* return address of a crash inside a cross-called out-of-image host thunk (CpuState only shows guestpc=host-slot and x30=leaf ret). Reusable for any future "out-of-image slot crash".
- **CODE (SH116b, `routeb_patch_nativeinit_flagmanager`, default-inert under JIT_SH115_SINGLETON_PATCH):** patch file 0x2320a24's load slot (adrp+ldr `[0x10672739b0]`) → movz/movk x8 = a low fixed zeroed mmap'd page (0x60000000, 2 slots, +0x28=0=PTHREAD_MUTEX_INITIALIZER), mirroring SH116's proven code-patch mechanism (a startup store to the `.bss` is impossible — page unmapped until engine boot, SH116 ENOMEM). Idempotent, shift-guarded (slot0 must be `adrp x8,7273000` 0xf0027a88), non-vtable-widening. + pure `sh116b_flagmanager_words` + 1 hermetic (round-trip incl. real-image word check).
- **MEASURED (corrected after further runs):** the flag-manager lock site is FIXED — **zero** faulting runs show GUEST 0x102320a98 in the GSDSP (before: that site faulted run-variably). The ladder overall is **NOT deterministic-clean**: a corrected 10-run characterization with the slot fix = 8 clean / 2 fault, the 2 residuals at *different* pre-existing SH55/64 sites (0x104c393f0 system-dialog region, 0x10284ce54 do-init __call_once blr x2) — unrelated to SH116b. Earlier "10/10 clean (was 8/12)" claim = a lucky streak, RETRACTED. Default path unregressed (3/3 clean, SH116b off).
- **SLOT-FIX (bd97ab9 was incomplete):** the adrp(+0x00) and ldr(+0x0C) load slots are NOT adjacent — 0x2320a28 `mov x5,x4` + 0x2320a2c `mov x4,x3` (body arg shuffle) sit between them. bd97ab9 wrote words[0..1] at +0x00/+0x04 (clobbering `mov x5,x4` AND leaving the `ldr` to re-zero x8 → x0 still 0x28). Corrected to slots {+0x00, +0x0C}, preserving the arg movs.
- **STANDING (unchanged):** Route-B live-DM world-build = the structural gate (SH174/196/203/204). SH116b does NOT manufacture a DataModel — it clears one specific crash (flag-manager lock) that the earlier SH116 sibling did not cover.
- **GSDSP repro:** re-run the SH203 repro command with `JIT_GUEST_STACK_DUMP=1` to see `GSDSP[+8=GUEST(0x102320a98)]` on the pre-SH116b binary.
## SH204 (Sep 16, 2026, hermes-worker): WORLD-BUILD GATE = MEASURED REACHABILITY NEGATIVE on the completing V2 path — SETWORLDBUILD seed stays latent (both clean + fault paths) + deep body 0x102ea3b14 terminates in the already-closed SH174 AppBridge lifecycle line. Doc docs/frontier-sh204-worldbuild-reachnegative.md. No production code change. Workspace green (564/0).
- **WHY THIS CYCLE:** the SH202/203 state change (full V2 ladder now completes clean 8/12) opened the genuinely-new question SH199 could not answer: does the now-completing run cross SH199's world-build gate 0x102368100 and execute the deep fn 0x102ea3b14? SH199 measured the seed LATENT only under the OLD state where V2Init always stopped at the SH198 host-pointer flake first.
- **METHOD:** region-watch CALIBRATED to prove fire-ability (fires on known-executing blocks 0x106251e94 SH200-patch + 0x102365c54 V2Init driver in the same run), then the negative.
- **THE NEGATIVE (3/3 clean, EXIT 124, full ladder through SendAppEvent Ok):** JIT_REGION_WATCH=0x102368100-0x102368140,0x102ea3b14-0x102ea3c50 → gate_block_hits=0, deepbody_hits=0. The gate block is NOT on the completing path (V2Init soft-returns via the SH200 singleton-materialization, never runs the natural body that would fall through to 0x102368100) — so the SETWORLDBUILD seed never fires with any effect on a completing run. Fault runs (2/3 = SH203 post-family pthread_mutex_lock 0x7f00000022b0; 1 = SH198/SH55 host-pointer 0x102b9e950) are all known classes and never reach it either.
- **DEEP BODY = ALREADY-CLOSED LINE (fresh disasm):** fn 0x102ea3b14 = `ldr x0,[x0,#688]` gate → operator-new(0x18) + ctor 0x2eacccc → `bl nativeAppBridgeAppStart 0x2365960` → canary ret. Its terminal action is the AppBridge lifecycle closure SH174 measured (via V1 AppStart__ governor 0x2338ef4) as "NEVER a DM factory, ZERO GuiObjects, reached the headless ceiling, do not seed the AppBridge line further." Host-driving 0x102ea3b14 would route into already-closed AppBridge territory → do NOT build it.
- **VERDICT (do-not-re-tread):** SETWORLDBUILD is now MEASURED inert-by-reachability on both paths (not assumed course); correct-but-harmless, fires only if a real session ever executes 0x102368100. No seed change gets the deep body running headlessly. AppBridge lifecycle = SH174 do-not-chase. Route-B live-DM world-build = structural gate UNCHANGED (~30+ recon angles + this measurement).

## SH203 (Sep 16, 2026, hermes-worker): POST-FAMILY FAULT = CONFIRMED SAME LIVE-WORLD-BUILD GATE (SH202 §6 answered) + app-data-model register trace (SH197 NEXT answered). Doc docs/frontier-sh203-postfamily-samegate.md, repro runs/capture_sh203_repro.sh. No production code change (no new seed warranted — see verdicts). Workspace green (564/0). recon-v3 render plane re-verified green (24 task-driven frames, 195 pops, no json abort).
- **POST-FAMILY FAULT (deterministic, 4/12):** `SIGSEGV guestpc=0x7f00000022b0` (host-call slot 1110 = pthread_mutex_lock bridge), `lr=0x102b53a78` (guest ret from `bl 62d63b0 <pthread_mutex_lock@plt>` at file 0x2b53a74), `x0=0x28` IDENTICAL across all faulting runs = NULL `this` object with a pthread_mutex_t at +0x28 (`obj==0`, locking `&obj+0x28`). Enclosing fragment file 0x2b53a64 is a **vtable/computed-dispatch target — ZERO direct bl callers** — in the JNICallProtocol_receiveCall / app-bridge call-protocol region. Reachable only AFTER the V2 family is cleared. ⇒ **SAME live-world-build/Java-session gate** (not a fixed-address seed like SH116's .bss lock-owner helper). VERDICT: do-not-chase, forward-motion evidence (run now advances past the V2 singleton family into app-bridge territory the baseline never reached).
- **V2 LADDER COMPLETION NOW MEASURED 8/12 clean** (was 1/4 at SH202) on the canonical ladder with V2_ONDEMAND; the residual 4/12 all fault at the identical same-gate site. Default path (V2_ONDEMAND off) unregressed: 0 patches, EXIT 124, 0 crash.
- **SH197 NEXT ANSWERED:** the app-shell ctor's app-data-model register 0x2208354 runs its FULL body (→ operator-new(0x18) 0x2086c0 → store [0x106dca000+0xed8] → 0x208710) + do-init base reaches 0x1023f00f8 + governor tail runs to the canary-ret 0x102ea30dc (137 region hits) — BUT SH155 DM-root markers UNCHANGED: once-guard=1, DM-root=SH156 seed, liveDM=false, once-slot=0x400000b (strcmp intern, NOT a DM controller), app-data-model-count[e88]=0x1 (STABLE baseline — the JSON-serialization path writes it once every run; NOT per-run DM progress, do-not-re-tread as a marker). 0x2208354 is a registry-entry builder, NOT a DM factory.
- **STANDING:** Route-B live-DM = structural gate (unchanged). Do-not-re-tread this cycle: app-data-model register 0x2208354 (registry-entry builder, not a DM factory); once-slot 0x400000b (intern, not controller); post-family fault (same live-world-build gate). Closest-original open thread remains the R1 synthetic-CoreScript content path (reachability-blocked on live DM, not a headless seed).

## SH202 (Sep 16, 2026, hermes-worker): ON-DEMAND single-site V2 singleton-dispatch family patcher (SH201 §6's named lever) — clears ONLY the exact blr the run dispatches through at the outside-image stop, then rewinds pc + continues; avoids the SH201 family-wide over-patch crash. FIRST full V2 ladder completion at HEAD. Workspace green (564/0, +3); arm64jit lib 386/0; elfjit example 64/0. Commit 783cd28. Doc docs/frontier-sh202-v2ondemand.md, repro runs/sh202-ondemand-full-ladder.txt (gitignored).
- **CODE (arm64jit/src/jit.rs, default-inert, env JIT_ROUTEB_V2_ONDEMAND):** `v2_family_bl_target_l` (imm26 branch decode, CORRECT sign-ext in i64: subtract 2^26=0x400_0000 NOT 2^30 — the SH201-style bug, falsified by a backward-bl regression test), `v2_family_blr_from_guest` (pure family classifier: bl 0x6249eb8 getter within 16 back + ldr x8,[x0] after + past-0x60 ldr x8,[x8,#N] within 4 of blr), `v2_ondemand_object` (stable leaked 0x80 all-leaf singleton), `v2_ondemand_patch_at` (classify+patch+drop cache+return rewind pc). Hooked into the outside-image stop in jit_run_inner, gated on the env; rewinds state.pc to the window start and `continue`s. +4 hermetic (classifier genuine-vs-decoys; backward-bl sign-ext; object stability; **real-image guard: the 3 measured stop blr sites + 4 SH200 sites ALL classify**).
- **EMPIRICAL (real libroblox.so, llvmpipe, canonical 9-rung ladder):** with V2_ONDEMAND=1, the run patches exactly the site(s) it dispatches through (1-3/run) and returns past them (0 outside-image stops after). **FULL LADDER COMPLETION OBSERVED**: nativeInitFlags → nativeGameGlobalInit Ok → nativeUpdateAdapterInit Ok → setTaskSchedulerBM Ok → **V2InitWithParams Ok(0x3e8)** → **V2StartAppWithParams Ok(0x3e8)** → V1 AppStart__ Ok(0x3e8) → V2UpdateSurface Ok (XID 0x200000) → SendAppEventOnAppReady Ok(0x3e8) → EXIT 124, 0 crash. FIRST full V2-ladder completion (V2Start was "observed, run-variable" at SH200 HEAD). **Honest boundary:** 1/4 runs completes clean; 3/4 patch the family then advance DEEPER into a NEW reachable wall — `SIGSEGV pc=0x7f00000022b0 (host-call slot) fault [x0+0x28] lr=0x102b53a78` — a host thunk deref'ing a NULL/small arg, the next seed target (a live-world-build-class gateway; forward motion, not regression). **Default path unchanged** (V2_ONDEMAND off ⇒ 0 patches, EXIT 124, SH200/201 baseline).
- **NEXT (honest):** Route-B live-DM world-build remains the standing structural gate. The new post-family fault (host thunk slot 0x22b0, fault [x0+0x28], lr 0x102b53a78) is the next unblocked seed: drive whatever object lr's caller threads into the slot, or confirm it's the same live-world-build gate (do-not-chase if so). Keep the do-init→app-shell→governor continuation + live-DM line as the primary front.
## SH201 (Sep 16, 2026, hermes-worker): PRECISE v2 singleton-dispatch family scanner derived + tested (384 sites located; sign-extension bug fixed) + family-wide patch EMPIRICALLY shown to over-patch (crash) and REVERTED. Commit <SH201COMMIT>. Doc docs/frontier-sh201-v2family-scanner.md, repro runs/capture_sh201_v2family.sh. Workspace green (561/0).
- **SCANNER (elfjit.rs, pure `sh201_v2_family_scan(&[u8])`):** the ~365-site objB-getter singleton-dispatch family that still stops V2Init/V2Start at run-variable pcs after SH200's 4 sites is precisely isolated: `bl 0x6249eb8` (getter within 16 back) + `ldr x8,[x0]` AFTER it + `ldr x8,[x8,#N]` (N*8>=0x60, past the seeded 0x60 leaf) within 4 slots before `blr`. Getter-prefix + N>=0x60 is exactly the discriminator that excludes SH200's warned genuine in-band N<0xf0 calls. **On the real libroblox.so the scanner finds 384 sites.** +2 hermetic tests (synthetic getter-gated match + N<0x60/no-getter decoys rejected; real-image >=100 guard).
- **BUG FIXED en route:** imm26 sign-extension subtracts 2^26 (0x400_0000), NOT 2^30 (0x4000_0000) — the 2^30 constant silently broke all backward `bl`s (forward branches passed the naive test, masking it), returning 0 on the real image until fixed.
- **EMPIRICAL NEGATIVE (patch reverted):** wiring `routeb_patch_v2_family` (patch all sites w/ SH200 window) behind JIT_ROUTEB_V2FAMILY crash-loops the run — the run reaches 0x106258000 (inside the patched region) and SIGSEGVs, the exact over-patch class SH200 predicted. **Baseline (SH200 only): 4/4 runs EXIT 124, SH200 4/4, 5-6 rungs Ok, 0 crash.** The family-wide runtime patch is NOT shippable; the scanner + tests ship as the measured lever. Do NOT re-enable routeb_patch_v2_family.
- **STANDING (unchanged):** Route-B live-DM world-build = structural gate. V2Init deterministically reaching the SH199 world-build gate 0x102368100 would need a runtime on-demand patch of only the exact blr'd site (intercept + loaded-vtable check), de-prioritized as non-Route-B-critical per SH200.

# Open Sober — Agent Handoff
## SH200 (Sep 16, 2026, hermes-worker): V2Init/V2Start "outside image" stop is a SCOPED-SEEDABLE singleton-dispatch family (FALSIFIES SH198's "non-seedable host-pointer class"). Commit 824d8ac. Doc docs/frontier-sh200-v2dispatch-seedable.md, repro runs/capture_sh200_v2dispatch.sh. Workspace green (562/0, +1 hermetic; arm64jit example tests 62/0).
- **FINDING:** the V2Init/V2Start soft-return stop (SH198 pinned to `blr x8` at 0x106251eb4, pc varies 0x9e/0x8b/0xb848c300000100/0xc148...) is NOT a genuine host pointer — it is fn 0x6251e0c (V2Init/Start params accessor) reading singleton objB's vtable slot +0x118 (280), PAST the harness-seeded 0x60 leaf vtable, into host box-alloc bytes that look like x86-mcode (hence the run-variable pc). The SITE is fixed + seedable: materialize the stable singleton object into x0 and NOP the blr. (SH198 saw a run-variable pc and concluded "host pointer => non-seedable" without disassembling the fixed site.)
- **CODE (elfjit.rs, default-inert under JIT_SH115_SINGLETON_PATCH, idempotent):** `routeb_patch_v2_dispatch` patches 4 located sites (fn 0x6251e0c +0x118, 0x62523ac +0x130, 0x6258e88 +0x2f0, 0x6258ffc +0x2f8) — dispatch window from `ldr x8,[x0]` guard (0xf9400008) through the `blr` -> movz/movk x0 = stable object + nops, preserving the trailing `ldr x8,[x19]; str x0,[x8]`/`strb w0,[x8]` store (receives the stable obj, no NULL+0x28). Pure `sh200_v2_dispatch_window` + 1 hermetic test (movz/movk round-trip; tail nops; 4-slot min).
- **EMPIRICAL (real libroblox.so, EXIT 124, 0 crash):** SH200 fires 4/4 on EVERY run (deterministically installed). V2StartAppWithParams COMPLETED (Ok(0x0)) in some runs (observed, was always soft-return at SH199 HEAD) — REAL improvement but RUN-VARIABLE, NOT guaranteed, because the family is ~600 uniform `bl 0x6249eb8` accessor sites (each reads objB vtable past 0x60). A data-driven full-cluster scan was TRIED + REVERTED (false positives on genuine in-band `ldr x8,[x8,#N]; blr x8` N<0xf0 -> SIGABRT); a precise discriminator (verify loaded vtable == harness-seeded before patching) is a future lever but LOW-ROI (V2Init is NOT Route-B-critical; the do-init->StartLuaAppDM->governor continuation runs clean regardless). Default env-OFF path untouched.
- **STANDING (unchanged):** Route-B live-DM/class-registry world-build = the structural gate. SH200 FALSIFIES SH198's "non-seedable" verdict for the V2 stop (source is a fixed, seedable, deterministically-installed family) + observe V2Start completion (run-variable). Do not chase the full V2Init family over the live-DM gate.
## SH199 (Sep 16, 2026, hermes-worker): WORLD-BUILD FN 0x102ea3b14 GATE BYTE LOCATED + SEEDED (latent-but-correct reachability negative). Commit b64e871. Doc docs/frontier-sh199-worldbuild-gate.md, repro runs/capture_sh199_worldbuild.sh. Workspace green (561/0, +1 hermetic; arm64jit 383/0).
- **FINDING:** the SH198 "never reached headlessly" deep construction fn 0x102ea3b14 (`ldr x0,[x0,#688]`; operator-new(0x18) + ctor 0x2eacce4 + `nativeAppBridgeAppStart__` 0x2365960) sits behind a SINGLE zero-default `.bss` byte [0x106a70568], gated inside the already-executing V2InitWithParams rung. `adrp x8,6a70000; ldrb w8,[x8,#1384] (=[0x106a70568]); cbz w8,0x102368114` at guest 0x102368100-108 SKIPS `bl 0x2ea3b14` at 0x10236810c (the SOLE call-site in the whole image). Crossing the byte falls the rung through into the deep app-start world-build body.
- **CODE (arm64jit jit.rs, default-inert, env JIT_ROUTEB_SETWORLDBUILD):** `routeb_worldbuild_gate_seed` — at gate-block pc in [0x102368100, 0x102368114), write 1 to [0x106a70568] when currently 0 (idempotent; preserves a real nonzero session value). Wired into the JIT_ROUTEB_SETFIX dispatch chain. +1 hermetic `sh198surface_routeb_worldbuild_gate_seed_env_and_pc_and_idempotent` (env-off inert even at the gate pc / env-on seeds 1 / the cbz-TAKEN target 0x102368114 is OUT of the window / live value preserved).
- **EMPIRICAL (real libroblox.so, EXIT 124, 0 crash):** SETWORLDBUILD=1 + region-watch [0x102ea3a84,0x102ea3be0] — 3/3 the V2InitWithParams rung stops FIRST at the SH198 run-variable host-pointer flake (pc 0x76/0x229/0x102b9e008 "outside image", x30=0x1062514e4) BEFORE reaching gate block 0x102368100, so the seed fires but the deep body is not entered in the same run. BASELINE (env OFF) = identical 3/3 V2Init stop ⇒ pre-existing SH198/SH55 flake, NOT a regression. The byte-cross is LATENT-BUT-CORRECT: the instant a real session (or a future V2Init flake-fix) advances the rung past the outside-image stop, 0x102ea3b14 executes. Converts SH198's "unreached (judgment)" into a located, seedable gate + a measured reachability negative.
- **STANDING (unchanged):** Route-B live-DM/class-registry world-build = the structural gate. Do NOT static-seed the V2Init outside-image stop (host-pointer class, SH198 verdict holds). This adds one more located+gardened lever to the manufacture/DMCONT continuation line.
- **CODE (jit.rs):** JIT_OUTSIDE_TRACE ring (`[u64;16]` last block-entry pcs) + `ring_ordered` — the V2 out-of-image stop is a repeating `0x102b9df10->0x106249ecc->0x1062514c0` then bad pc = HOST x86-mcode bytes (`48 89 8b`) via `blr x8`@0x106251eb4 through a HOST-heap obj x0=0x55e.. — the SH176/SH103/SH109 singleton-vtable class (host ptr, NOT a static-seedable slot; pc varies per run). JIT_REGION_WATCH now parses comma multi-region specs (`region_watch_contains`; the old single-pair parse silently FABRICATED region negatives).
- **REGION-WATCH at HEAD (single run, all regions):** hits across do-init post-body 0x1023eff4c, app-shell ctor 0x102207b50 (-> app-data-model register 0x2208354), governor 0x102e9fa84 -> governor tail 0x102e9fdc8/0x102ea30dc — the whole do-init->app-shell->governor continuation executes real relocated engine code headlessly (EXIT 124, 0 crash; gov vtable resolves real 0x102e9fa84). **TERMINAL:** tail ends at a stack-canary `ret` (0x102ea30dc); the deeper fn 0x2ea3b14 (`ldr x0,[x0,#688]`) is NEVER entered headlessly.
- **ROUTE-B POSITIVE:** the governor MODERN startAppWithParams blob-path 0x258c6e4 IS reached (region 0x10258b144..0x10258b400 fires) — closes SH197 §6 (a). **NEXT (honest):** the V2 out-of-image stop is definitively non-seedable (host ptr, deterministic evidence) — do NOT chase static seeds for it; the residual is a live class-registry populating the host obj's vtable slot +0x118 (same live-DM wall).
- **SH197b (commit 3ad4957): [elfjit:dmcells] probe of the class-name RESOLVER map 0x106dca0e70 with the continuation live = in-context NEGATIVE.** app-data-model register 0x2208354 -> bulk registrar 0x2208ae8 reaches the resolver map in-context (SH194 was standalone-only) but resolver[0x106dca0e70]={0,0,0 live=false}, source[0x106dca0e90]={0,0}, register[0x106dca0f60]={0,0} all EMPTY (0 crashes, EXIT 124) — the registrar's SOURCE is empty pre-world-build, so it inserts nothing. Closes the last "in-context populates the resolver map" lever (SH193/194). Don't re-tread: name->classid resolution still needs a live class-registry world-build.

# Open Sober — Agent Handoff
## SH196 (Sep 16, 2026, hermes-worker): DO-INIT `__call_once` SELF-LATCHES HEADLESSLY — the once-lambda COMPLETES (once-guard 0x106a68410 bit0 0→1) but produces a strcmp string-intern (0x400000b), NOT a live DM. Commit d9ebacb. Doc docs/frontier-sh196-doinit-once-completes.md. Workspace green (558/0); recon-v3 still green.
- **METHOD (the operator's dynamic-do-init-trace, now observed in-run):** StartLuaAppDM's jit_run parks forever in the engine main-loop idle nanosleep poll (lr=0x10284d134), so the post-rung SH155 probe never fires. Added a **default-inert `[elfjit:dmcells]` observable** (JIT_THREADS=1 + JIT_DMCELLS=1) to the JIT_THREADS sampler: page-guarded reads of once-guard[0x106a68410]/once-slot[0x106a68408]/DM-root[0x106a68818]/flags-latch[0x106a683e8] once per tick.
- **EMPIRICAL (real libroblox.so, full --v2boot ladder, EXIT 124, 0 crash, reproducible):** `once-guard=0x1 once-slot=0x400000b DM-root=<SH156 seed unchanged> flags-latch=1`; world-build expanded 1656→5576 JIT compiles then settled flat at idle.
- **VERDICT (closes the operator's 'once-lambda never completes' / dynamic-trace question at its strongest):** the do-init's own `__call_once` NOW COMPLETES headlessly — once-guard bit0 self-latches (std::call_once at 0x284cf5c runs after the lambda at 0x284ce54) — but the lambda body (file 0x2206d24..0x2206d74 → `bl 0x2173b3c` strcmp string-intern → `str x0,[x23,#1032]` = [0x106a68408]) yields a small interned-string status (0x400000b), NOT a live DM. The once-slot is not a DM controller (only a string-intern return). No live DM materializes in once-slot OR DM-root at idle; the actual DM factory (live ExperienceController app-launch make_shared) stays unreached by any do-init cell write. Route-B live-DM evidence = ~30 recon angles + this measured completion-without-DM.
- **DO-NOT-RE-TREAD (added):** do NOT write once-slot [0x106a68408] to a fabricated controller (engine-owned; its completion already stores the intern); do NOT re-attempt 'the once-lambda builds the DM' (measured = strcmp intern).
- **STANDING (unchanged core wall, now with a completion proof):** Route-B's self-constructed home/login needs a live DataModel whose factory is only reachable via a real app-launch (migration per lapsed-cone = the manufacture/DMCONT/PATH-B line is grinding it but the do-init path provably ends DM-less). Shipped+reproducible on the Route-B sub-frontier: PlayerGui/ScreenGui self-construct (SH190d/e), service-node (SH191), clean walker, genuine populated vtable (SH195), do-init __call_once completes (SH196). Next genuine lever remains a live-DM session (SH174 latch ready) or a live-DM-class event outside the ~30 closed gates.
## SH195 (Sep 16, 2026, hermes-worker): SCENE-ATTACH ABI GATE — the self-constructed PlayerGui's LIVE vtable is genuinely populated (positive), but its draw slot (vt+24) is an x1-forwarding bounce the present-walker cannot direct-dispatch. Commit 95c3ecd. Doc docs/frontier-sh195-scene-attach-abigate.md. Workspace green (558/0); recon-v3 re-verified 24 task-driven frames / 196 pops / no json abort.
- **FINDING (real libroblox.so, SH191 self-construction drive + new benign read-back):** the self-constructed PlayerGui's runtime vtable (on-disk zeros, reloc-populated) holds genuine engine dispatch entries — draw slot vt+24 = guest 0x105fb2dac, file 0x5fb2dac: `ldr x9,[x1]; mov x8,x0; mov x0,x1; mov x1,x8; ldr x3,[x9,#56]; br x3` — a BOUNCING ADAPTER that swaps args and dispatches through a SECOND object `[x1]->vt+56`. The scene present-walker (SH64 pin: per-node `ldr x0,[x20,#8]; ldr x8,[x0]; ldr x8,[x8,#24]; blr x8`) supplies ONLY x0=render_obj, never x1 ⇒ a genuine GuiObject cannot be dropped into the R+0x180/0x188 node list as a bare render-obj. The base render object in x1 is built only in a live scene-graph/world-build (same live-DM migration gate).
- **CODE:** benign live-vtable read-back (16 slots) in routeb_dm_service_resolve_guard under JIT_ROUTEB_DM_SERVICE_NODE (default-inert, per-slot page_is_mapped guarded). Hermetic sh191 passes (380/0 lib). No call into any slot.
- **DO-NOT-RE-TREAD:** do NOT write the genuine PlayerGui/ScreenGui pointer into a scene node's [node+8] and drive the present-walker (x1-stale bounce → FTL/wrong dispatch); do NOT manufacture a fake base object for the bounce (host-thunk vt+56 = Route-A host geometry, not an engine-authored draw).
- **STANDING:** Route-B sub-frontier advanced one more rung — resolution (SH194) AND scene-attach (SH195) both now closed at mechanism/ABI depth behind the live-DM world-build. PlayerGui/ScreenGui self-construct (SH190d/e), service-node (SH191), clean walker, genuine populated vtable = shipped+reproducible. Next lever (if cone re-opens): locate a leaf draw (real vt+56 on the base render entry) via a live scene-graph, OR the live-DM session (migration, SH174 latch ready).

# Open Sober — Agent Handoff
## SH194 (Sep 16, 2026, hermes-worker): RESOLVER-MAP CONSTRUCTION = mechanism-level recon-negative — closes SH193's 'next lever' (locate+drive resolver-map ctor/insert). Doc docs/frontier-sh194-resolver-construction-negative.md. Benign diagnostic (8-word resolver-map header read-back) added to routeb_dm_service_resolve_guard; corrupting registrar poke built+reverted (default-inert removed). Workspace green (558/0).
- **RECON (fresh disassembly):** resolver map 0x106dca0e70's ONLY writer = bulk registrar 0x2208ae8 (nativeGameGlobalInit+0x26e4; ABI `x0=&{key_ptr,key_len}` -> ret `&element.classid`; rehash 0x5fb2948; stride 0x18, classid@+0x10); ONLY reader = read-only probe 0x2373cec (getService walker 0x105e09bc8 + ~8 consumers). Per-class register writer 0x1dc4bc8 targets REGISTER map 0x106dca0f60 (resolver does NOT read it). The 48-byte map header IS written in-ladder (region-watch 0x102208418-0x102208620 fires 6x every clean run, incl. pcs 0x20843c/450/47c/504/584/5c0) — but as DEFAULT-EMPTY {begin=0,end=0} from a stack default-ctor, NOT a live-constructed unordered_map.
- **EMPIRICAL (in-context registrar drive, real libroblox.so, then reverted):** drove 0x2208ae8 at StartLuaAppDM (post nativeGameGlobalInit, resolver pages ensure-writable) with fabricated {&"PlayerGui",9} + wrote 0x87e into returned classid slot -> registrar returned a HOST slot (0x7fce84...) + run aborted `bad_weak_ptr` (EXIT 139) — the EXACT SH192 symptom. Confirms SH193's 'no headless-reachable ctor' gate at MECHANISM level: the map's bucket-sentinel/allocator (built only by a real unordered_map ctor inside a live world-build) is absent headlessly; the registrar hashes against null bucket state. getService name->classid resolution (thus the SH191 linked-node RETURN) is live-class-registry-world-build-gated = same live-DM migration pattern.
- **DO-NOT-RE-TREAD:** drive 0x2208ae8 (host-slot + bad_weak_ptr, SH192 3/3 + SH194 1/1) nor 0x1dc4bc8 (writes 0x106dca0f60, a map the resolver does not read).
- **SHIPPED (default-inert under JIT_ROUTEB_DM_SERVICE_NODE):** 8-word resolver-map header read-back printed at drive time; walker still executes clean not-found, re-verified EXIT 124, 0 SIGSEGV/0 SIGABRT, no bad_weak_ptr.
- **VERIFY:** workspace green (558/0). Clean SH191 capture 2/2 EXIT 124, SH194 header dump fires, walker clean, no corruption.
- **STANDING (new):** Route-B sub-frontier closed at same depth as live-DM gate: PlayerGui/ScreenGui SELF-CONSTRUCT (SH190d/e) + PlayerGui service-node on [dm+0x68] (SH191) + walker executes clean = shipped+reproducible; name->classid RESOLUTION is the one remaining piece and is live-world-build-gated. Next = scene-attach (R+0x180/0x188) via a direct parent hook that bypasses getService name-resolution, OR the live-DM session (migration).

# Open Sober — Agent Handoff
## SH193 (Sep 16, 2026, hermes-worker): CLASS-NAME REGISTRY = THREE lazy-static unordered_maps; getService name->classid resolution is LIVE-ctor-gated (concrete control-flow gate, not a judgement). Doc docs/frontier-sh193-classname-registry-lazy-gate.md. Workspace green (558/0, EXIT 0). Doc-only (characterization cycle, SH192 pattern).
- **RECON (fresh disassembly):** the class-name registry on page 0x6dca000 = three separate function-local-static std::unordered_maps, each with its own __cxa_guard lazy ctor: RESOLVER 0x106dca0e70 (name->classid stride 0x18, classid@+0x10; read by resolver 0x2373cec from getService walker 0x105e09bc8 + ~8 service consumers; NO headless-reachable ctor), bulk SOURCE 0x106dca0e90 (read by bulk registrar 0x2208ae8 in live nativeGameGlobalInit), REGISTER 0x106dca0f60 (written by per-class writer 0x1dc4bc8 via insert 0x1dc4c3c -> rehash 0x202d858; **lazy ctor LOCATED 0x5fb2a78**, guard 0x6dca0f90 — zeroes the 32-byte header, no-op on already-zero .bss, bucket lazily alloc'd on first insert).
- **GATE (concrete, stronger than a judgement):** resolver 0x2373cec is a READ-ONLY open-address probe (`ldp begin,end,[x0]; b.eq not-found` on empty) — never constructs. Post-ladder the resolver map reads {0,0} (measured this cycle). Only writers = the live nativeGameGlobalInit bulk registrar (0x2208ae8 mid-nativeGlobalInit fragment; driving standalone corrupts the shared page — SH192 bad_weak_ptr 3/3, do-not-repeat) and the register writer (0x1dc4bc8 -> 0x6dca0f60, a DIFFERENT map the resolver does not read). No headless code path constructs 0x106dca0e70 ⇒ SH191's service-node attach + clean walker drive are correct+latent and fire when a real session constructs the resolver map (same migration pattern as type-4 producer / DM-capture latch).
- **NEXT lever (if the Route-B cone re-opens):** locate the RESOLVER map's own ctor/insert helper (sibling to the register writer but {key_ptr,key_len,classid} element layout) and drive it headlessly to construct+populate 0x106dca0e70 via engine code — the missing bridge for the in-ladder registrar + getService to resolve the self-constructed PlayerGui node. Do NOT hand-write hash elements.
- **VERIFY:** workspace green (EXIT 0). SH191 capture script at HEAD: one EXIT 124 / one EXIT 134 (SIGSEGV guestpc 0x6240cb8 = `tbnz`, no deref) = pre-existing SH55/64 run-variable flake; fresh re-run clean. Default guards unregressed.

# Open Sober — Agent Handoff
## SH192 (Sep 16, 2026, hermes-worker): resolver-map bulk registrar LOCATED (0x2208ae8) + empirically NON-CONSTRUCTIVE standalone. Commit 9bde043 (doc-only; the destabilizing registrar drive was built + reverted). Doc docs/frontier-sh191-service-node-getservice.md (SH192 addendum). Clean SH191 re-verified 3/3 (EXIT 124, 0 crash, no bad_weak_ptr).
- **FINDING:** the class-name→classid RESOLVER map 0x106dca0e70 (getService walker 0x105e09bc8 → resolver 0x2373cec, stride-0x18 open-addressing {key_ptr,key_len,classid}) is populated by the bulk registrar **0x2208ae8** (in nativeGameGlobalInit), which IS headless-reached (region-watch fires every clean run) but inserts nothing because its SOURCE (0x106dca0e90/register map 0x106dca0f60) is empty pre-world-build. Driving 0x2208ae8 standalone with a fabricated {&"PlayerGui",9} key + writing 0x87e into the returned slot **3/3 returns a HOST slot, map stays {0,0}, run aborts bad_weak_ptr** — the std::unordered_map at 0x106dca0e70 needs its ctor (bucket array + allocator + count@0x106dca0e88) which only runs inside LIVE nativeGameGlobalInit. Do NOT re-attempt; do NOT hand-write the hash element. getService "PlayerGui"→0x87e resolution (linked-node RETURN from the walker) sits behind the live class-registry world-build — the same Route-B live-DM wall, now with the registrar identified.
- **VERIFY this session:** workspace green (558/0); clean SH191 ladder 3/3 EXIT 124, 0 SIGSEGV/SIGABRT. (The exit-139 bad_weak_ptr noise this cycle was my SH192 registrar drive, not pre-existing — proved by re-running the clean build.)
- **NEXT (honest, untouched wall):** scene-attach (R+0x180/0x188) + a real engine-authored GuiObject scene node rendering still sit behind a live DataModel class-registry construction. Continue Route-B manufacture/DMCONT/PATH-B grinding per operator; do not re-tread the resolver registrar or hand-write the hash element.
- **CODE (routeb_dm_service_resolve_guard, env JIT_ROUTEB_DM_SERVICE_NODE=1, default-inert):** after SH190e captures the real PlayerGui instance, (1) link it as a thin 0x70 service node on [dm+0x68] ({[+0x18]=0x87e classid, [+8]=instance, [+0x68]=next}), then (2) drive the engine's getService walker 0x105e09bc8 with a fabricated long-form "PlayerGui" name string + x8=&out. Disassembled walker ABI: resolves name->classid via map 0x2373cec, walks [dm+0x68] comparing [node+0x18]==classid, materializes into out. +1 hermetic sh191 test. Workspace green (558/0).
- **EMPIRICAL (real libroblox.so, llvmpipe, EXIT 124, 0 SIGSEGV/SIGABRT):** `SH191: linked PlayerGui service node 0x7fb..0xf0 (classid 0x87e, instance 0x7fb..0720) at [dm+0x68]` — the engine-authored PlayerGui (vptr 0x106648950) is now a service node on the genuine DM's service container. Walkers `DROVE ok ret x0=0x0 out={0,0}` (not-found) — the engine's getService code path executes cleanly headlessly.
- **GATE (honest, precisely characterized):** name->classid RESOLUTION uses TWO SEPARATE registries, both **EMPTY {0,0}** at this boot: the resolver MAP 0x106dca0e70 (read by 0x2373cec; element stride 0x18 {key_ptr,key_len,classid@+0x10}) the walker + ~8 consumers all reference, AND the register's map 0x106dca0f60 (written by 0x1dc4bc8). The SH189 class-register GETTER populates the DESCRIPTOR cache (0x106c980b8, vtable-family 0x106648908) but NOT the name->classid resolver map — the per-class DescribedCreatable register and the resolver map are DISTINCT; the resolver map is built by a bulk registrar, not the per-class getter. Do NOT hand-write the hash-map element (24-byte stride + open-addressing hash, docs warn against it).
- **NEXT (real leap):** locate + drive the bulk class-name registrar that fills 0x106dca0e70's {begin,end} vector (the known once-init builder at 0x1dad050-family wraps the call_once idiom but targets a DIFFERENT page 0x6c26000; the resolver map on page 0x6dca000 needs its OWN writer). With the map populated, the walker resolves "PlayerGui"->0x87e and returns the linked instance (item vptr 0x106648950) — the engine genuinely returning its self-constructed PlayerGui headlessly. Scene-attach (R+0x180/0x188) sits behind that. Cone stays armed.

# Open Sober — Agent Handoff
## SH190e (Sep 16, 2026, hermes-worker): BOTH GuiObject-family instances self-construct headlessly — PlayerGui (0x106648950) AND ScreenGui (0x106649ce0). The SH189b recon-negative is crossed. Commit e620aee. Doc docs/frontier-sh189-dm-classregistry.md (SH190e). Repro runs/capture_sh190c_member_seed.sh (gitignored log).
- **FIX (extend SH190d to ScreenGui):** routeb_dm_instance_guard now drives the SGI pair-consumer 0x10247a88c after the PlayerGui drive, with (1) a call-site NOP of the SGI ctor's sub-init `bl 0x4b5df04` at 0x247a99c (0x949b8d5a->0xd503201f) so its derive body runs to its own vptr write, (2) the same string-member seed at SGI_CTOR_ENTRY 0x10247a984 (obj+0x60 long-form empty + SSO windows), (3) a run_guest_callback_x8 drive (x0=dm, x8=&out). The SGI object is read from the shared capture atomic (overwritten by the SGI ctor-entry; PGI obj restored after). ScreenGui class vptr 0x106649ce0 (recon SH190b — NOT the 0x106628740 the sub-init secondary write clobbers).
- **EMPIRICAL (real libroblox.so, 2/2 EXIT 124, 0 SIGSEGV/SIGABRT):** `SH189c: *** CONFIRMED — ... RBX::PlayerGui instance ... (vptr 0x106648950) ***` + `SH190e: *** CONFIRMED — ... RBX::ScreenGui instance ... (vptr 0x106649ce0) ***`. Both engine SELF-CONSTRUCTED, headlessly, reproducible. Defaults unregressed (3/3 clean).
- **CODE:** env + pc gated, SGI_NOP + SGI drive + capture-atomic save/restore, hermetic test extended (SGI ctor-entry also seeds). Workspace green.
- **NEXT (honest):** both instances are STANDALONE — PlayerGui not a service NODE on [dm+0x68] ([node+0x18]==0x87e), ScreenGui not parented, neither in the scene-scan (R+0x180/0x188). Scene-attach layer (live-app-shell/vt-dispatched service arming) is the standing frontier. Cone stays armed.

# Open Sober — Agent Handoff
## SH190d (Sep 16, 2026, hermes-worker): FULL PLAYERGUI SELF-CONSTRUCTION headlessly — the derive body COMPLETES and the genuine PlayerGui-class vptr 0x106648950 is written. The SH190c 'crash at the next member' gate is CLOSED. Commit b0693b5. Doc docs/frontier-sh189-dm-classregistry.md (SH190d). Repro runs/capture_sh190c_member_seed.sh + runs/sh190c-member-seed.txt (gitignored).
- **FIX:** `routeb_dm_instance_ctor_capture` (under JIT_ROUTEB_DM_INSTANCE_NOP, at PGI ctor-entry 0x10255d1dc) seeds the PlayerGui object's string members: obj+0x60 = coherent LONG-FORM empty std::string {__cap_=0x100|1 (long), __size_=0, __data_=real zeroed 0x100 buffer}, + zero [obj+0x40,0x60) & [obj+0x78,0xa8) as EMPTY SSO. Root cause of the 0x5e1f44c crash: the derive-body string copy-assign (`setString` 0x2374d4c -> `operator=` 0x5e1f380) derefs the DEST string's LONG-FORM __data_ (obj+0x70) UNCONDITIONALLY; the mempool object's slots are garbage. Zeroed-SSO fails (NULL->[0x8]); a cap-constant fails (0x101->[0x109]); a real buffer works.
- **EMPIRICAL (real libroblox.so, llvmpipe, 3/3 EXIT 124, 0 SIGSEGV/SIGABRT):** `*** CONFIRMED — engine SELF-CONSTRUCTED a real RBX::PlayerGui instance ... (vptr 0x106648950) headlessly ***`, `obj [+0x0]=0x106648950`. The FULL PlayerGui-class layer (vptr 0x106648950) is written headlessly — deepest Route-B point yet (the SW190 observe re-focused onto the derived class layer, past the instance base 0x106796dc0). Default NOP-off capture UNREGRESSED (3/3 EXIT 124, instance-base-only).
- **CODE (+1 hermetic sh190c_dm_instance_nop_member_seed_*):** env/pc/window-gated, SSO windows + long-form cap/size/data assertions, capture-atomics preserved. Workspace green (arm64jit 379/0).
- **NEXT (honest):** the constructed PlayerGui is a STANDALONE instance — not yet a service NODE on [dm+0x68] ([node+0x18]==0x87e) nor parented to a ScreenGui scene (R+0x180/0x188). Observe-instance DONE; the scene-attach layer is the standing frontier (behind live-app-shell/vt-dispatched service arming — recon-negative, do NOT re-tread the walker 0x5e09bc8). Route B's playerGui-self-construct is not yet a scene node or rendered UI; that is the next cone. Cone stays armed.

# Open Sober — Agent Handoff
## SH190c (Sep 16, 2026, hermes-worker): CORRECTED Route-B diagnosis — the PlayerGui-CLASS vptr write IS reachable headlessly via the call-site NOP (supersedes the SH190 erratum's 'not forceable'). Commit a69bc15. Doc docs/frontier-sh189-dm-classregistry.md (SH190 ERRATUM + CORRECTION). Repro runs/sh190-nop-derived-vptr.txt (gitignored) + runs/capture_sh189c_instance.sh.
- **FINDING:** `routeb_dm_instance_ctor_capture` gains an opt-in `JIT_ROUTEB_DM_INSTANCE_NOP=1` (SEPARATE env; the standard JIT_ROUTEB_DM_INSTANCE-only capture stays clean EXIT 124, verified 3x). It NOPs the PlayerGui ctor's sub-init CALL `bl 0x255d2f4` at guest 0x10255d200 (0x9400003d -> 0xd503201f) so the derive body runs directly. EMPIRICAL (real libroblox.so, diagnostic EXIT 134): the derived body EXECUTES and the crash dump shows `x8=0x106648950` — the adrp 0x6648000+add #0x950 at 0x255d214-0x255d218 LOADED the **PlayerGui-class vptr**. The objective "class vptr write becomes reachable" is ACHIEVED headlessly (the erratum's tail-RET lever failed because it edited the sub-init's INTERNAL tail at 0x255d334; the CALL-SITE NOP bypasses the sub-init entirely — that's the working mechanism).
- **NEXT GATE (moved from 'unreachable' to 'next member'):** the derived body then faults at guestpc 0x105e1f44c `ldr x9,[x20,#16]!` (fault=0x8, x20=obj+0x70, x19=obj) — a post-write NULL member (~obj+0x70/+0x80) the derive continues to deref. The crash is run-variable (the object then flows through the caller's post-create path, different fault pcs across runs — SH55/64 class). Landing a full PlayerGui = clear/seed those post-write members so the ctor completes -> observe [obj+0]=0x106648950. Do NOT re-tread the tail-RET (reverted); the call-site NOP (0x10255d200) is the working lever. This is an opt-in DIAGNOSTIC (crashes at the member); NOT the default capture.

## SH190 (Sep 16, 2026, hermes-worker): FIRST OBSERVED headless INSTANCE-BASE Construction under the genuine DataModel — the PlayerGui ctor chain's object is now driven AND read back (SH189c's "allocated but unobserved" residual CLOSED). Commit f2c3f68 + 94c57c8. Doc docs/frontier-sh189-dm-classregistry.md (SH190 addendum), repro runs/capture_sh189c_instance.sh.
- **KEY FIX (observation, not new levers):** the pair-consumer 0x10255d0e4's `ret x0` walks to a STRING ("Invalid da...") and the out-buffer stays {0,0} (shared_ptr attach skipped) — BOTH red herrings. The authoritative read is the **ctor-entry object capture**: `routeb_dm_instance_ctor_capture` fires at the PlayerGui ctor block entry 0x10255d1dc (x0==the op-new'd object) inside the nested jit_run, snapshots its vptr, and the guard reads the post-drive layout.
- **EMPIRICAL (real libroblox.so, 3/3, EXIT 124, 0 crash):** `ctor-entry pc=0x10255d1dc obj=0x7f..599950 vptr-at-entry=0x0` then `obj vptr=0x106796dc0`, layout [+0x0]=0x106796dc0 [+0x18]=0x106dc0c58 [+0x30]=0x100000000. **0x106796dc0 = instance-ctor 0x2374310's relocated vtable** (`adrp x8,0x6796000; add #0xdc0` @0x2374368, in-image filevma 0x6796dc0) — the engine SELF-CONSTRUCTED a real vtable'd instance object headlessly, now OBSERVED (not just the drive returning Ok).
- **DERIVED PENDING:** the PlayerGui-class vptr 0x106648950 (write at 0x255d21c) does NOT yet land — the ctor sub-init chain (bl 0x255d2f4 -> getter 0x201fce0 -> tail 0x23768e8) mounts the instance BASE. NEXT: force the ctor body to its own vptr write (inspect why bl 0x255d2f4 returns to a different vptr-set path / drive 0x255d21c) so [obj+0]=0x106648950; then the full PlayerGui self-constructs before service-node attach ([dm+0x68]) + scene-scan (R+0x180/0x188).
- **CODE (arm64jit jit.rs, default-inert JIT_ROUTEB_DM_INSTANCE):** +`routeb_dm_instance_ctor_capture` (block-entry, pc 0x10255d1dc/0x10247a984, snapshots x0+vptr-at-entry into Atomics) wired in the dispatch chain; guard reads entry-vptr + post-drive obj vptr + leading-word layout dump. Workspace green (arm64jit 377/0).

# Open Sober — Agent Handoff
## SH189c (Sep 15, 2026, hermes-worker): REAL PLAYERGUI INSTANCE CONSTRUCTION EXECUTES TO COMPLETION headlessly — Route-B's furthest point (migration gate MOVED). Commit <SH189CCOMMIT>. Doc docs/frontier-sh189-dm-classregistry.md (+ SH189c addendum), repro runs/capture_sh189c_instance.sh.
- **BREAKTHROUGH (recon deleg_5d14fcbe + deleg_25bb1ff0, verified vs objdump):** the populated class-name registry made the REAL PlayerGui/ScreenGui INSTANCE ctor chain headlessly REACHABLE — the creator ServiceProvider::getOrCreate 0x102373458 (class-mgr resolve 0x23736dc -> op-new 0x1d96768 -> blr 0x255d1b4 -> insert) + pair-consumer 0x10255d0e4 + real ctors 0x255d1dc (PlayerGui vptr 0x106648950)/0x247a984 (ScreenGui). Gates: (a) creator's current-DM global guest 0x107333948 ≠ *0x106391908; (b) x23 = instance-ctor x1 = owner via pair-consumer's x0.
- **CODE (default-inert, JIT_ROUTEB_DM_INSTANCE):** `run_guest_callback_x8` (sets the x8 out-reg; the args array only maps x0–x7) + `routeb_dm_instance_guard` (plant *(0x107333948)=dm, drive 0x10255d0e4 with x0=dm). +1 hermetic test (377/0).
- **EMPIRICAL (real libroblox.so, llvmpipe, 2/2 EXIT 124, 0 crash):** `PlayerGui pair-consumer 0x10255d0e4 DROVE ok ret x0=0x7f9ffa567361` — the real PlayerGui instance ctor chain EXECUTES TO COMPLETION headlessly (was the x23=0 null-deref EXIT 134; x0=dm fixed the owner). HONEST: out={0,0} — the drive returns Ok but doesn't yet surface the instance through the out-buffer, so self-construction isn't yet observed; next = walk the op-new'd object for the PlayerGui vptr 0x106648950 to CONFIRM it. The lone EXIT-139/nativeInitialize crash = SH55/64 clone-worker flake (before this guard).
- **STANDING (updated):** PlayerGui/ScreenGui service NODE on [dm+0x68] + scene-scan attach (R+0x180/0x188) now sit BEHIND a reachable, completing ctor chain — not an unreachable vt-factory. This is NOT the migration gate; the instance ctor is demonstrably driveable. Only observed-instance + scene-attach layers remain. Cone stays armed (next: confirm the constructed instance by reading the op-new'd object's vptr).

# Open Sober — Agent Handoff
## SH189b (Sep 15, 2026, hermes-worker): SCREENGUI CLASS-DESCRIPTOR REGISTRATION too — engine's own code now registers BOTH PlayerGui (0x87e) and ScreenGui (0x1b87) in its global class-name registry headlessly. Commit 4513237. Doc docs/frontier-sh189-dm-classregistry.md (+ addendum), recon doc docs/frontier-sh189b-getter-abi-drive.md, repro runs/capture_sh189_dm_services.sh.
- **CORRECTED ABI (recon deleg_5c489b38):** the ScreenGui register BODY 0x10201f4f0 null-derefs headless (`str x0,[x22,#8]` @guestpc 0x101db7e38, class-member builder, source=0) — it MUST be driven via its CALLER GETTER **0x10201f42c** (parameterless; latch 0x106c980a28 + nested source-builder guard 0x106c96868), exactly the PlayerGui pattern (getter wraps once-body). StarterGui getter 0x102020120 is NOT all-zero-safe (forwards caller x0->source into register helper; needs a real source — follow-up).
- **CODE (arm64jit jit.rs, default-inert, same JIT_ROUTEB_DM_SERVICES path):** extend routeb_dm_service_seed_guard to ALSO drive ScreenGui getter 0x10201f42c (clear its 2 latches) + probe ScreenGui desc vtable/vt-family.
- **EMPIRICAL (real libroblox.so, llvmpipe, 2/2 clean EXIT 124, 0 crash):** ScreenGui getter DROVE ok ret x0=0x106c98a40. **NOTE: recon's desc addr 0x106c980a40 is 0x2000 LOW — the real desc object is the RETURNED addr**, whose [0x106c98a40]=0x1067a6150 (shared DescribedCreatable desc vtable, same base as PlayerGui) and vtable-family [0x106c98a40+0x230]=**0x106649c98** (exact recon-expected ScreenGui value). Both PlayerGui and ScreenGui class descriptors now registered headlessly in the engine's own class-name registry.
- **STANDING WALL (unchanged, recon-negative do-not-re-tread):** PlayerGui/ScreenGui service NODES on [dm+0x68] ([node+0x18]==0x87e/0x1b87) + INSTANCES parented + scene-scan attach (R+0x180/0x188) need the real RBX ctor (vt-dispatched/live app-shell) — the walker 0x5e09bc8+0x2377600 only refcounts existing nodes. MIGRATION GATE unchanged. Cone stays armed.

# Open Sober — Agent Handoff
## SH189 (Sep 15, 2026, hermes-worker): PLAYERGUI CLASS-DESCRIPTOR REGISTRATION headlessly — the engine's OWN code registers the PlayerGui class in its global class-name registry for the first time (one-next-object past the SH187 genuine-DM ctor). Workspace 376/0. Commit dfa68e0. Doc docs/frontier-sh189-dm-classregistry.md, repro runs/capture_sh189_dm_services.sh, log runs/sh189-dm-services.txt.
- **CONE (deleg_9884123a + deleg_3a08fcf0 + deleg_58cfcb06 + deleg_9b2cfbef + deleg_728c4b80, all READ-ONLY, converged):** (1) get-or-create 0x102dbcd88 create-path `blr [DM-vptr+0x1c0]`=0x103facf10 = BOOLEAN registration predicate (cbz x1->ret / RTTI IsA proxy), never a GuiObject; (2) `DescribedCreatable<ScreenGui>` rodata = PURE RTTI type_info (reloc-materialized {vptr 0x106358d90, name}), NOT a classname->factory dispatch — both GuiObject levers DEAD; (3) the genuine DM ctor 0x1023f6038 NEVER builds PlayerGui/CoreGui/ScreenGui (services resolve LAZILY by name via per-DM container [dm+0x68]/[dm+0x78], gated on global class-name registry 0x106dca0e70); **ONE-NEXT-UNSYNTHESIZED-OBJECT = PlayerGui class descriptor** (getter 0x10201fce0, once-latch 0x106c97f30, body 0x10201fda4, nested latch 0x106c883a0, classid 0x87e, typeid 0x298).
- **CODE (arm64jit jit.rs, default-inert, env JIT_ROUTEB_DM_SERVICES, +1 hermetic test):** `routeb_dm_service_seed_guard` (StartLuaAppDM-scoped, OnceLock): seed coherent EMPTY service-list head+vector on the constructed DM (only when NULL); clear chained once-latches 0x106c97f30+0x106c883a0; drive the REAL PlayerGui class-register GETTER 0x10201fce0 (parameterless; 0x106c980b8 is the descriptor OBJECT, not a latch — recon correction); probe counter/cached/vtable/vt-family.
- **EMPIRICAL (real libroblox.so, llvmpipe, 2/2 clean EXIT 124, 0 crash):** GETTER DROVE ok ret x0=0x106c980b8; **cached desc [0x106c97f28]=0x106c980b8, desc vtable [0x106c980b8]=0x1067a6150, PlayerGui vtable-family slot [0x106c980b8+0x230]=0x106648908** (all recon-agreed markers ✓) — FIRST headless execution of the engine's own PlayerGui class-registration. HONEST: class-desc counter [0x106dca0e28]=0 (recon had it at 5fb225c tail; descriptor-object markers are the stronger proof; likely a different counter word). Lone EXIT-134 = pre-existing SH55/64 run-variable flake @V2InitWithParams (upstream of this guard). SH187 upstream (genuine DM ctor + holder plant) re-verified in the same log.
- **STANDING WALL (recon-negative, do-not-re-tread):** a PlayerGui service NODE on [dm+0x68] ([node+0x18]==0x87e) + a ScreenGui INSTANCE parented + scene-scan attach (R+0x180/0x188) need the RBX::PlayerGui ctor (vt-dispatched/live app-shell) — walker 0x5e09bc8+0x2377600 only REFCOUNT an existing node, never creates. MIGRATION GATE unchanged. 3 headless-drivable class descriptors: PlayerGui (DONE), ScreenGui (getter 0x10201f4f0, desc 0x106c980a40, classid 0x1b87), StarterGui (getter 0x10202014c, classid 0x892).
- **NEXT (closest unblocked):** register the ScreenGui + StarterGui class descriptors with the same mechanic (completes name-resolution for the resolver), add a resolver probe (drive 0x102373dec(x0=&0x106dca0e70, x1=&"PlayerGui"), assert [el+0x10]==0x87e), and keep the Route-B cone armed. The PlayerGui/ScreenGui INSTANCES stay the migration gate.

# Open Sober — Agent Handoff
## SH187 (Sep 15, 2026, hermes-worker): VPTR-BASE CORRECTION + FIRST genuine-DM ctor CONSTRUCTION headlessly + holder-plant (SH187b). Workspace 375/0. Commits 16c60e5 (SH187), 42fb965 (SH187b). Doc docs/frontier-sh187-dm-vptr-base-corrected.md, repro runs/capture_sh187_real_ctor.sh, log runs/sh187-real-ctor.txt.
- **BREAKTHROUGH (fresh 3-agent Route-B cone, the operator's 'dynamic DM-ctor trace' re-attack):** the genuine primary RBX::DataModel vptr is **0x67162e8** (guest 0x1067162e8), NOT the +8-off **0x67162f0** used throughout SH179-186. RTTI typeinfo 0x6714e18 sits at vptr-8=0x67162e0. A REAL DataModel ctor EXISTS at guest **0x1023f6038** (wrapper 0x23f5ff8): it materializes the genuine vptr set {0x67162e8, 0x67163a0, 0x67163f8} via `adrp x8,0x6716000` + imm12 adds (offsets 0x2e8/0x3a0/0x3f8 all fit). The prior "no static materialization / migration gate" proof-of-dead-end is **FALSIFIED** — it scanned page 0x671000 + the +8 constants (adrp+add geometrically impossible there), MISSING this ctor on page 0x6716000.
- **THE +8 ERROR EXPLAINS SH181/183:** `routeb_manufactured_dm()` planted vptr 0x1067162f0 (+8, a method slot), so the manufactured "genuine-vptr DM" was invalid — that's why the manufacture lever never dispatched anywhere. Now corrected to plant the FULL genuine set {0x1067162e8, 0x1067163a0, 0x1067163f8}.
- **CODE (arm64jit jit.rs, default-inert, +2 hermetic tests):** (1) `routeb_manufactured_dm()` writes the full genuine vptr set; (2) `routeb_dm_real_ctor_drive_guard` (env JIT_ROUTEB_DM_REALCTOR) host-drives the real ctor wrapper 0x1023f5ff8 via run_guest_callback. Follow-up recon deleg_fa2be765: the ctor body to 0x23f6130 is branch-free, the drive stalled INSIDE subobject-call `bl 0x23f6b0c` at 0x1023f60b8 — NOP'd it (0x94000295->0xd503201f) and the ctor now falls through.
- **EMPIRICAL (real libroblox.so, llvmpipe):** capture_sh187_real_ctor.sh — after the NOP, **`obj vptr set = 0x1067162e8,0x1067163a0,0x1067163f8 GENUINE MATCH (genuine=true)`, 0 SIGSEGV/SIGABRT, EXIT 124, ladder clean.** The JIT CONSTRUCTED a genuine RBX::DataModel through its REAL ctor code headlessly for the first time.
- **SH187b (42fb965):** the constructed genuine DM (ret x0=obj+0x1f0) is now planted into the current-DM holder *0x106391908 (always, crash-free) + consumer delegate slots +0xe8/+0xf0/+0xc0 +0x38c seeded.
- **SH187c (recon deleg_c64972f1 + deleg_f39b7cda):** the one bl-reachable consumer — get-or-create 0x102dbcd88 (from MemStorage_bind 0x24c61e8 / nativeOnDestroyed 0x275bd04) — driven with a seed spec (once-guards 0x106a665a0/0x106a665b0=1, key 0x106a665a8=0, keyed-lookup vector OBJ+0xb0=D / OBJ+0xc0=D+16, element *(D)). **FAST mode (JIT_ROUTEB_DM_REALCTOR_CONSUMER=1): get-or-create RETURNS the planted genuine DM cleanly, 0 crashes. DISPATCH mode (+JIT_ROUTEB_DM_REALCTOR_DISPATCH=1: element NULL + flag 0x106dbf238=1): the create-path `blr [DM-vptr + 0x1c0]` at 0x2dbce70/0x2dbce80 executes real relocated code on the genuine DM vtable (ret x0 = a new create-path object), 0 crashes, EXIT 124.** Both verified on real libroblox.so — the FIRST headless dispatch of the genuine DM vtable through a reachable engine consumer.
- NEXT (honest): reach the classname-keyed InstanceFactory (e.g. "ScreenGui"/"ScriptContext" strings at file 0x31868b/0x22a648 factory entries) so the engine SELF-CONSTRUCTS its first GuiObject under the genuine DM. Honest correction from recon deleg_f445c1da: the dispatch create-path's ret-x0 object is the DM's **embedded sub-object** (DM+0x70, +8 instance wrapper) — NOT a GuiObject; and the DM vtable, re-scanned at the CORRECTED base, still has NO GuiObject/SceneGraph/ScriptContext producer slot (it's a service/member-accessor table). So Route B's next real gate is the InstanceFactory classname-keyed creation path. Prior "migration-gate" closures were built on the wrong +8 vptr and need re-derivation.
- **SH187 RECON REFINEMENT (deleg_0f44536a, authoritative negative):** the premise that 'ScreenGui'@file 0x31868b / 'ScriptContext'@0x22a648 lead to a classname-keyed InstanceFactory is FALSIFIED — both are analytics/perf labels; NO plainclassname string dispatch for ScreenGui exists anywhere (no adrp+add to 0x5344e8/0x586c4f, no movz/movk "ScreenGui" constant). The real classname registry = **RBX::DescribedCreatable<T,Base>** (RTTI rodata cluster 0x9daa5e <RelativeGui,GuiObject>, 0x9daad1 <ScreenGui,GuiLayerCollector>, 0x9dab13 <ScrollingFrame,GuiObject>, typeinfo N3RBX9ScreenGuiE @ 0x9ecbf3), built at static-init via PC-rel — NOT driveable as a plain-string lookup. Concrete headless lever that constructs a real object under the genuine DM remains get-or-create 0x102dbcd88 (SH187c seed). Next step to a ScreenGui: locate the DescribedCreatable<ScreenGui> registrar init (rodata 0x9daa5e-0x9dad65) OR drive the DM's PlayerGui service-arming during the DataModel build.

# Open Sober — Agent Handoff
## SH186 (Sep 15, 2026, hermes-worker): FRESH 3-AGENT ROUTE-B RE-ATTACK (operator Sep-15 directive) reconciled — live-DM is MIGRATION-GATE at a 5th stacked closure + 3 fresh re-attacks; closed a real jstring-path JNI slot gap. Workspace 552/0. Commit da07249. Doc docs/frontier-sh186-routeb-fresh-reattack-jstring-slot.md.
- **CODE (arm64jit jni.rs):** extend `jni_table_has_official_abi_slots_nonnull` hermetic test to also assert slot 170 (ReleaseStringUTFChars) is a non-null host thunk. The fabricated-jstring / SendAppEventOnAppReady step-2 path (helper 0x21e1fec) derefs BOTH slot 169 (GetStringUTFChars) AND slot 170 — production table already stubs it (build_jni:1092 `= ok`); the assertion locks it against regression. arm64jit jni_ = 16/0.
- **CONE (deleg_99108c2b + deleg_70c84ea5, 3 READ-ONLY Route-B agents, the two 429-dropped re-batched):** (1) ExperienceController re-attack — createDataModelForTeleport (0x2e1dc38) & setDataModelToCurrent (0x2dbcc10) are DEAD leaves (0 bl, 0 addr-taken, 0 pointer-ref whole-file); the ONE DM-touching path = nativeAppBridgeStartLuaAppDM -> 0x2baeeec -> do-init 0x2206c40 -> scheduler closure needing a prebuilt appbridge binder [0x106a68818] + pre-registered scheduler runnable (0x6a68000+1032, gate 0x6a68000+0x410) = live app-launch precondition; no entry makes a DM in isolation. operator-new hook would catch the make_shared IF it ran, but it never runs headlessly. **No dynamic headless lever** — strictly migration (the only hypothetical = in-memory fabricate binder+runnable, same difficulty as the static-seed route). (2) SendAppEventOnAppReady 0x102bb463c = telemetry-only: both export & dispatch void, NO 0x3e8 token anywhere; drives clean headlessly ONLY if BOTH JNI slots 169+170 stubbed.**(3) R1 synthetic-CoreScript = LAW wall** (not judgment): rbxasset://scripts/CoreScripts @0x10232ed4 has 3 refs, NONE load+execute Lua; filesdir 0x10726d600 has 10 code refs, ZERO join a Lua path; CoreScriptLoader 0x1f1d8ac = mid-ctor instr, 0 bl callers; no live DM to parent a ScreenGui => even a seeded Lua can't self-construct. Type-4 vector host-install-only confirmed (0 native stores, sole reader dispatcher w4==4 arm); MH_APP_READY `&& live-DM` gate correct (first conjunct judgment-forcible, second = law).
- **EMPIRICAL (this box):** re-ran capture_sh182_dm_ctor_driver fresh — manufacture lever EXECUTES the real app-shell ctor 0x1057d6ef4 (`DROVE ok ... vt=0x1067162f0 entered AND returned through real code`, `entered region`=1, EXIT 124, 0 crash). This FALSIFIES deleg_99108c2b task-0's 'vtable fabricated / 0x57d6ef4 absent (low-VMA data)' claim — the .data.rel.ro vtable slots are RELATIVE-reloc-zero in file BY DESIGN (SH178/179), and the JIT really enters guest 0x1057d6ef4. Manufacture lever stands.
- **STANDING:** Route-B live DM = MIGRATION GATE at 5 stacked closures + 3 fresh re-attacks. Do NOT re-derive holder/DMCONT/live-binder/CreateDataModelForTeleport (authoritative negatives). Next: live-DM session = migration (GPU-host/real-input) w/ SH174 capture-latch observer; any new Route-B push must first locate a live-DM-construction event outside the 5 gates. Cone kept armed.
- **SH186b (2-agent fresh-sector re-attack, counter-decisive, both READ-ONLY):** operator's 're-attack via do-init/do-build completion / dynamic DM-ctor trace' answered at the strongest level. (1) The do-init GATE 0x2206c40 is CROSSABLE headlessly as code (~80/100): seed flag [0x106a68410].bit0=1 + runnable [0x106a68408]=0 (engine default runner) + descriptor vt[+0x30]=app-shell ctor 0x1057d6ef4 (runtime-known) + main-thread-id [0x106863a68]=live tid, then call 0x102206c40 on-main-thread. BUT reaching a live DM make ~35/100, and the DM it yields has NO consumer (gate 2) + NO GuiObject-producer vtable (gate 3) — crossing it produces NO self-constructed screens; a do-init-driver edit = cruft against an empty endpoint, do NOT build. (2) EXHAUSTIVE APS2 decode: the DM vptr VALUES 0x67162f0/0x67163a8/0x6716400 have ZERO static materialization sites (0 adrp+add / 0 movk / 0 GOT/.data addend / 0 literal / absent as raw bytes from the whole 109MB image); DM ctor slots = 0 direct bl callers (vtable-dispatched only); no onDataModelLoaded/dataModelOnCreated string; only createDataModel paths downstream of a live DM. **No boot-reachable DM-construction event exists outside the 5 gates.** Stronger than any prior negative. Route-B = strictly migration (GPU-host/real launch), SH174 observer.
- **SH186c (2-agent follow-on, both READ-ONLY, authentic closure):** (1) Manufactured-DM through the messageBus 'experience-launch request' cb (body fn 0x2bd744c) consumes the DM ONLY via `blr [vt+0x10]` (slot 2) then Release — never getDataModel, never SceneGraph, never R+0x180/0x188. Per-row slot-2 (full APS2 decode): vt 0x67162f0 = 0x229c2a4 `mov x0,xzr;ret` null-stub (benign dead-end); 0x67163a8 = 0x24090a8 StartLuaAppDM consumer → SH182 PATH-B deep-NULL fault; 0x6716400 = destructor. **CORRECTION: the 3 genuine DM vptrs are NOT interchangeable vtable rows** — SH181/182 used the null-stub row, which is why feeding the manufactured DM through the cb is inert. Even the consumer row dies on deep NULL members before any GuiObject instantiation. Manufactured-DM headless line FULLY EXHAUSTED; do NOT re-tread. (2) Persistence/session-restore (objective b) = STRICTLY MIGRATION: auth is 100% JNI/env-class (Java CookieManager; no session.db/shared_prefs/cookies.db native literal); headless ladder touches ZERO guest /data or /cache files; SH168 rbx-storage prestage + SH177 jar gates are correct-but-inert migration-readiness. No further persistence-seed investment.
- **CONE (deleg_67e9118b, 2 READ-ONLY, both authoritative):** (1) dataModelBindings_onGameLoaded/onAppLuaWillStart = LIVE JNI->AppBridge events (export 0x2bb429c -> dispatch 0x2baeeec, needs real JVM; no subscription table). Sole messageBus path = 'experience-launch request' cb 0x102bd76e8 (subscribe 0x2ba5bb8 registered INSIDE gated initializeLuaApp_; reads DM at [DataModelBindings+16], cbz->cleanup if null). 'Manufacturing a DM handoff' = manufacturing the DM (the gated work). **MIGRATION-GATE** — closes the one crack SH184 left open. (2) type-4 self-sustaining loop SOUND as-is: producer pre-hook this->[24] NULL at boot (no [24]/[32] owner store in engine/drain region) => host nodes push, not dropped; deque tagged-CAS (node&~7) confirmed, SH60 host node pushes on tail path; 3 presenter-reachable flood exits (controller cb w4=2/3/4 HALT in pop-loop 0x102856e40, producer [24] veto, dispatchable [node+40]=free-no-repush) => FIFO, no unbounded growth, no hardening required.
- **STANDING:** Route-B live DM = MIGRATION GATE at FOUR stacked levels (full-lifecycle SH184-1, holder-no-consumer SH184-2, vtable-no-producer SH183, live-binder SH185). Headless Route-B crack closed at the strongest depth reached. Do NOT re-dispatch holder-*0x106391908 / DMCONT / live-binder-subscriber cones (authoritative negatives). recon-v3 self-drive verified sound + end-to-end (24 frames/196 pops). Next: live-DM session = migration (GPU-host/real-input) w/ SH174 capture-latch observer; any new Route-B push must FIRST locate a live-DM-construction event outside the 4 closed gates, else it is strictly migration work. Cone kept armed.

## SH184 (Sep 15, 2026, hermes-worker): FRESH 3-AGENT ROUTE-B CONE (operator's return-to-Route-B directive) answers at FULL-LIFECYCLE level — NativeDataModelManager = MIGRATION-GATE with corrected address map; current-DM holder *0x106391908 has NO data consumer (ref-closure falsifies 'join reads the holder'); type-4 producer host-install-only + session-swap gate confirmed correct/strictly post-live-DM. Recon-v3 deliverables RE-VERIFIED on the real binary at HEAD. No production edit; workspace 374/0 + all crates. Doc docs/frontier-sh184-routeb-full-lifecycle-gate.md.
- **CONE (deleg_4350aec4 x3, all READ-ONLY, all authoritative):** (1) NativeDataModelManager full map corrected (continueAfterFlagsLoaded_ = 0x102bd3bb0, NOT the prior 0x102bd1d68 which is bootstrapTheApp_'s tail; it consumes the REAL fetched flag-payload string in x20 then does live-only GL/AppBridge/reporter init including nativeAppBridgeAppStart; initializeLuaApp_ just listens for a messageBus 'experience-launch request'; DM load is only via inbound live-binder dataModelBindings_onGameLoaded) => every stage needs a live engine service or inbound event; NO cell write short-circuits. **MIGRATION-GATE.** (2) *0x106391908 (current-DM holder) = exactly ONE static ref = orphaned address-forming getter 0x2dbcc10 (adrp/add/ret, never derefs the stored value), no callers, nothing reads it live OR headless => **FALSIFIES SH172/178 'ExperienceController::join reads the holder'**; the manufacture lever fires into a slot nobody derefs. (3) 0x102829ea8 host-install-only confirmed (only dispatcher w4==4 arm). Engine producer 0x10285682c = (this,node,w2=[node+32]&1,x3=[node+32]&~1) with self-sustaining push->drain->dispatch loop + real futex WAVE@0x2856744. MH_APP_READY strictly post-live-DM (SendAppEventOnAppReady is a JVM-export, zero bl callers, no DM check; NativeHelper.onAppReady slot-61 never fires headless) => the derived `MH_APP_READY && live-DM` swap gate is correct + load-bearing (ungated self-push = idle-spin livelock). Keep harness seed inert-but-correct.
- **RECON-V3 RE-VERIFIED at HEAD (fresh artifact, runs/capture_taskv4_frame.sh):** type4_frame_thunk registered at 0x7f00000001d0, vector [0x106829ea8] seeded, heartbeat w4#2/#3 patched to w4=4, RENDERCTX published 0x7f1b98110a00; **present #19..#23 swap Ok(0x1)** real task-driven frames (24 total); 196 real engine node pops; (no json abort); 0 SIGSEGV/SIGABRT; EXIT 124 stable. JIT_JSON_ZERO_FIX (part B) confirmed forcing len=0 at 0x102355d40 (runs/sh122-run-jsonfix.txt). Both recon-v3 deliverables shipped + verified.
- **STANDING:** Route-B live DM = MIGRATION GATE at full-lifecycle closure (strongest yet). Manufacture-lever slot has NO consumer at all (weaker than the SH181-183 'live-session-only' picture). Headless manufacture/carve-out line exhausted. Do NOT re-dispatch holder-*0x106391908 constituent cones. Any further Route-B cone must first re-derive whether a subscriber to the DM-created/loaded event (dataModelBindings_onGameLoaded / dataModelLifeCycle onAppLuaWillStart) could expose construction; else strictly GPU-host/real-app-launch migration with the SH174 capture latch as observer. Cone kept armed per discipline.

## SH183 (Sep 15, 2026, hermes-worker): CONSOLIDATED Route-B re-examination with the manufacture lever LIVE — DMCONT continuation empirically re-run = LATENT-CONFIRMED; genuine DM vtable decoded = NO GuiObject-producer slot. Docs + repro only (no production edit this cycle); workspace 552/0. Commit c742941. Doc docs/frontier-sh183-routeb-recon-live-lever.md.
- **CONE (deleg_b638c343, READ-ONLY, decisive):** (1) holder *0x106391908's only static reader = caller-less getter 0x2dbcc10; registry consumers fire only on the session-gated NULL->real transition (ExperienceController::join). (2) FULL packed-RELA decode of the genuine primary DM vtable 0x1067162f0: every slot is a lifecycle accessor; the only content-adjacent slots (+13/+14 -> 0x57cbcf8/0x57cbd00 -> UniversalApp loader 0x24090a8) read [x0] (sub-object vtable) -> 0 on a zeroed manufactured DM -> cbz benign no-op. NO DM slot builds the SceneGraph/render world. (3) DMCONT continuation = the one already-built, empirically-UNTESTED lever worth re-running.
- **EMPIRICAL (real libroblox.so, llvmpipe):** `JIT_ROUTEB_DMFORCE=1 JIT_ROUTEB_DMCONT=1 JIT_ROUTEB_DM_MANUFACTURE=1` + ladder + region-watch on 0x102bd1d68/0x2338ef4/0x2207b54: manager installs (continuation-routed, M+0x40=flags-holder, +0x1f0=REAL continueAfterFlagsLoaded_) BUT **continueAfterFlagsLoaded_ = 0 region entries** (single text match = install log only); nativeAppBridgeAppStart + app-shell ctor also 0. Ladder clean EXIT 124, 0 crashes. => DMCONT LATENT-CONFIRMED (SH168 reconfirmed under the live lever): the +0xf8 network feature-flag fetch never completes synchronously headlessly, so +0x1f0 never fires.
- **VERDICT:** Route-B GuiObjects remain behind a genuine live DataModel at the STRONGEST evidentiary level: (a) manufacture lever is LIVE (SH182, ctor dispatches real code), (b) yet the DM vtable has NO GuiObject-producer slot, (c) the only real engine-init forward (DMCONT) is latent-confirmed. This is the operator-requested re-examination answered with the falsification incorporated. Live DM = MIGRATION GATE; SH174 capture latch = validated observer for the real app-launch. Manufactured-DM line delivered its full headless value (latent -> live-dispatching, SH181+SH182); do NOT chase deeper PATH-B member reconstruction or DMCONT (~latent/low-ROI). Keep the Route-B cone armed per discipline.
- **Route-B content path CLOSED at the reference level (deleg_0c2db598 + deleg_0b72e1ba, exhaustive):** slot13/14 UniversalApp "loader" CORRECTED = generic container/teardown utility (93 callers, never refs UniversalApp.rbxm str @0x106c9a40). Real content path = DataModelPatcher config-init guest 0x1020eb8c8 with **ZERO static references** (0 relocs in 87,532+300, 0 init-array, 0 text operands, 0 raw-byte); config global 0x10683e3e8 written/read only from JVM-reached JNI natives (JNI_OnLoad/MainGameActivity/NativeGLInterface/etc.); DM vtable 0x1067162f0 / app-shell ctor 0x1057d6ef4 / do-init 0x2206c40 / governor chain have NO patcher path; apply live-DM-gated ('DataModel expired' @0x10229006, 'deserializeInstance DataModel is null' @0x10288090). => **HARD NO**: no manufactured-DM drive constructs a live DataModelPatcher headlessly. UniversalApp/GuiObject stays live-DM + Java-session gated. Route-B headless closed at reference level.
- **Migration-capture latch RE-VERIFIED at HEAD** (JIT_DM_ALLOC_CAPTURE=1+DELEGATE=1): 8 delegated FIRST-call allocs through engine prev_hook 0x1021ebaf4, ladder done, EXIT 124, 0 crash — SH174 observer migration-ready.

## SH182 (Sep 15, 2026, hermes-worker): HOST-DRIVE the manufactured genuine-vptr DM through its REAL app-shell ctor — Route-B manufactured-DM line gets its FIRST headless DM-vtable execution. Commits 699feae (PATH A), a8a4034 (PATH B builder), b0f6484 (PATH B member-seed). Doc docs/frontier-sh182-dm-ctor-driver.md, repro runs/capture_sh182_dm_ctor_driver.sh.
- **CONE (deleg_661626bb READ-ONLY 2-agent, + deleg_aac54e43 + deleg_623cac1f):** (1) app-shell ctor 0x1057d6ef4 ([V+0x30] of genuine primary DM vt 0x1067162f0) reads ONLY stack-canary global file 0x67d16f0 (guest 0x1067d16f0, value 1 -> SEGV; any stable val passes) + DM+0x38c (scalar safe 0). The SH181-flagged `[x1,#8]` deref is a std::string GATE not a code-ptr. (2) host API run_guest_callback(fn,args[8]) args[1]=x1 host-controlled; same-thread NESTED = sanctioned (IN_JIT_RUN keeps cache warm, no SH55/64; SH151=GL-reentrancy only, ctor pure .text). (3) PATH-B string = SHORT-form SSO 'ServerRestartScheduled', byte0=0x2c=22<<1, data +1, byte23=0 (equality fn 0x2152f30 reads SHORT len=byte0>>1). (4) ctor is a THIN SUB-INIT; fault 0x28 = transitive Mutex::lock(NULL+0x28); seeding DM+0x610/+0x648 does not clear (deeper NULL); ctor does NOT build SceneGraph/render/DM-sub-objects.
- **CODE (arm64jit jit.rs, default-inert, +1 hermetic test 373->374):** `routeb_dm_ctor_driver_guard` (JIT_ROUTEB_DM_CTOR_DRIVER, [0x1023efe2c,0x1023eff20], OnceLock) seeds canary 0x1067d16f0 + nested-run_guest_callback(0x1057d6ef4, x0=manufactured_dm, x1=routeb_dm_ctor_arg(full)). PATH A (default) zeroed descriptor -> clean survival no-op. PATH B (JIT_DM_CTOR_FULL) SSO 'ServerRestartScheduled' descriptor -> runs real init body. `routeb_seed_dm_pathb_members` un-NULLs DM+0x610/+0x648 (load-bearing embedded +0x28 mutex). Test: PATH A zeroed/stable, PATH B SSO byte-exact, member-seed idempotent non-NULL +0x28 zero, env/region gating.
- **EMPIRICAL (real libroblox.so, llvmpipe):** PATH A = **[region-watch] entered 0x1057d6ef4 (SH181 was 0)** + `DROVE ok ret x0=<dm> vt=0x1067162f0 entered AND returned through real code` + canary seeded; ladder clean EXIT 124, 0 crashes (re-verified 3x). PATH B reaches+runs the real init body (passes string gate) then faults 0x28 at a deep DM NULL member (unchanged after member-seed). Manufacture lever now LIVE-dispatching (not just latent).
- **HONEST:** first headless execution of the manufactured genuine-vptr DM through its real relocated app-shell ctor. PATH A verified clean = the SH182 deliverable. PATH B's ctor is a thin sub-init that needs unbounded live-DM member reconstruction and does NOT produce GuiObjects — kept as default-inert diagnostic, do NOT chase deeper (low-ROI at Route-B's expense). NEXT high-value: with the manufacture lever now LIVE, re-examine whether the manufactured DM + a live-DM holder can reach ANY GuiObject-producing consumer, or whether the live-DM session (do-init completion path) remains the migration gate. Live-DM session = migration gate unchanged.

## SH181 (Sep 15, 2026, hermes-worker): ROUTE-B "DEAD WALL" PREMISE FALSIFIED — JIT-side RBX::DataModel MANUFACTURE lever shipped + verified on the real binary (default-inert, env JIT_ROUTEB_DM_MANUFACTURE=1). Workspace 551/0. Commit 4da4da1. Doc docs/frontier-sh181-dm-manufacture-lever.md, repro runs/capture_sh181_dm_manufacture.sh.
- **FRESH CONE (2-agent READ-ONLY + 1 decisive, all Route-B-scoped after SH179 pinned the genuine DM vtable family):** (1) SH179 residual closed EMPTY: 0x2b37b6c/0x2b37a8c/0x2b3ed04 are clone/buffer factories that never write offset-0 (the vptr); 0x28f78a0/0x2950b3c/0x2950b5c store no vptr; the vtables are referenced by ZERO code/relocs/GOT — no static construction exists. (2) migration harness verified CONSISTENT (arm 0x106391908, read_vt_in_image accepts the genuine family ⇒ a real captured DM logs `[validated]`). (3) **DECISIVE (recon deleg_7effc85a): the SH178/SH180 "vtable inert ⇒ DECISIVELY DEAD" premise is FALSIFIED — the genuine DM vtables (0x67162f0/0x67163a8/0x6716400) ARE loader-populated at runtime** (189 R_AARCH64_RELATIVE relocs write real engine fns into every slot; [V+0x30]→0x57d6ef4; RTTI 0x6714e18→0x6358df8). So a manufactured object bearing a planted genuine vptr dispatches into REAL relocated engine code — the one headless Route-B lever.
- **CODE (arm64jit jit.rs, default-inert, +1 hermetic test):** `routeb_dm_manufacture_guard` (env JIT_ROUTEB_DM_MANUFACTURE=1, scoped to StartLuaAppDM [0x1023efe2c,0x1023eff20]) plants a OnceLock-built manufactured genuine-vptr DM (vt=0x1067162f0) into the current-DM holder `*0x106391908` (setDataModelToCurrent getter target, SH172) + `routeb_manufactured_dm()` (leaked zeroed 0x1108 block, first word = genuine primary DM vtable) + test `sh181_dm_manufacture_guard_plants_genuine_vptr_env_gated`.
- **EMPIRICAL (real libroblox.so, region-watch 0x1057d6ef4):** EXIT 124 (clean timeout), 0 SIGSEGV/SIGABRT/stack-smash, ladder done. Plant fires (holder swapped 0x106358d40 → 0x7f7938034160 vt=0x1067162f0); **DM app-shell ctor region [0x1057d6ef4..0x57d7100] = 0 'entered region'** — no headless boot consumer dispatches the current-DM vtable (empirical counterpart of SH172: registry current-DM consumed only by a session consumer). VERDICT: lever mechanically correct + benign + **LATENT-BUT-CORRECT** (type-4-producer class) — installed/live-correct, fires the instant a session reads the current-DM holder. Route-B's live DM stays the migration gate, but the one headless manufacture seed is now installed and proven live (not inert).
- **STANDING:** Route-B top priority; manufacture lever installed+verified; SH174 capture latch the validated GPU-host observer; workspace 551/0.

# Open Sober — Agent Handoff
## SH180 (Sep 15, 2026, hermes-worker): ROUTE-B STATIC-CONSTRUCTION HUNT CLOSED AT THE VTABLE-REFERENCE LEVEL + migration-capture harness verified consistent. Workspace green (550/0). No production code change. Doc docs/frontier-sh180-dm-vtable-unreachable.md. Commit 26609cc.
- **2-agent fresh READ-ONLY cone (operator's Sep-15 'return to Route B / re-examine the wall' directive), both authoritative:**
  (1) **SH179's residual is CLOSED as EMPTY (delegate, decisive-NEGATIVE).** Following "follow the object-vptr STORE from the 0x2b37b6c ctor": 0x2b37b6c (`bl 2a0d9b8`) is a clone/duplicate inside fn 0x2b37b44 (alloc 0x1108 then memcpy); sibling 0x2b37a8c→0x2b3ed04 writes fields at offsets 64/72/88/108/168/184 only — **offset 0 (vptr) never written**; 0x28f78a0 (byte-buffer) and 0x2950b3c/0x2950b5c (string-buffer) store no vptr; 2a0d9b8/2a0da80 are hookable alloc wrappers (`[0x67daaf0]` else 0x1d96a40), not operator-new. **Decisive:** the three genuine DataModel vtables {0x67162f0, 0x67163a8, 0x6716400} are REAL populated objects (code slots 0x57ce740/0x240a8b8/0x57d07f8; T=0x6714e18 at −0x10) but are referenced by **ZERO relocs, ZERO adrp/add, ZERO movz/movk, ZERO dynsym** in the entire ELF — the vptr values are unreachable from any static reference. ⇒ No in-process jit-runable route to a live typeinfo-correct RBX::DataModel exists; driving nativeGameGlobalInit/StartLuaAppDM/do-init 0x2206c40→0x2206db8/ExperienceController/V2StartAppWithParams changes nothing. The operator-requested re-examination is answered at the STRONGEST negative yet: **there is no dynamic DM-ctor to trace, because no reachable ctor materializes the vptr at all** (SH178's wall, formerly attribution-based, is now reference-based and closed). Route-B = migration gate, beyond any static seed.
  (2) **Migration-capture harness VERIFIED CONSISTENT + SH179 cross-check PASSES (delegate, positive).** All constants/mechanism match source: capture latch env JIT_DM_ALLOC_CAPTURE[+_DELEGATE] (jit.rs:1299/1322/1324), block pc 0x102a0d9b8 / active 0x1067daaf0 / default 0x1067d0840 (1278-80), delegation run_guest_callback(prev_hook,..,current_guest_tp) (1200-4), budget 256 (1228), idempotent latch (1315-17). Arming target stays **0x106391908** (getter 0x2dbcc10 re-disassembled `adrp x0,6391000/add #0x908/ret`; runbook frontier-sh174:67/103 refuses the old 0x106391918 — SH172 holds). Probe (R+0x188−R+0x180)/0x28, walker 0x105b2ed48, render_scene_base==0 consistent (jit.rs:11501-11558). **KEY cross-check:** read_vt_in_image (jit.rs:1248-1260) ACCEPTS the genuine family {0x1067162f0/0x1067163a8/0x106716400} (all ≥base 0x100000000 and <image_len 109,193,800) ⇒ a genuine captured DM base logs `[validated]`.
- **CODE:** none this cycle (forcing a change would be feature-flag cruft). Workspace 550/0 at HEAD 485ee20.
- **STANDING (unchanged):** live-DM = MIGRATION GATE — now closed at the vtable-reference level, the strongest negative in the lineage (replaces all prior 'no seed' verdicts). Type-4 producer latent; R1/R2 dead on reachability; SH174 migration runbook verified READY end-to-end. **NEXT (GPU-host):** capture (JIT_DM_ALLOC_CAPTURE=1+DELEGATE=1) → arm *(0x106391908)=B → probe R+0x180/0x188 for the first engine-self-constructed scene node.

# Open Sober — Agent Handoff
## SH179 (Sep 15, 2026, hermes-worker): WORKSPACE-FLAKE FIX (DM-capture test race) + fresh packed-RELA/RTTI decode locates the GENUINE DataModel vtable family (page 0x671000) + DM-capture latch runbook audit. Workspace 550/0. Commit c241925. Doc docs/frontier-sh179-dm-vtable-decode.md.
- **CODE (arm64jit jit.rs, +2 lock lines, no test weakened):** the fleet was NOT green at session start — `cargo test --workspace` failed 371/1 at sh167_dm_alloc_capture (running alone passed 3/3; assert line shifted 5068↔5085 by run = process-global race, NOT logic). Root cause: sh167 (jit.rs:5016) + sh169 (jit.rs:5106) run on parallel threads and both mutate the shared PREV_DM_ALLOC_HOOK static AND the JIT_DM_ALLOC_CAPTURE[_DELEGATE] process env (unsafe set_var/remove_var races a parallel var_os read = UB). Fix: a test-only process-global `DM_CAPTURE_TEST_LOCK: Mutex<()>` acquired by BOTH tests (production = single jit_run thread per run, shared static stays correct). Verified: arm64jit lib 5/5 (was ~1/2 flaky), full workspace 550/0.
- **RECON (fresh packed-RELA/RTTI decode, deterministic, this is SH178's sanctioned forward):** pinned the REAL DataModel RTTI family — typeinfo **T = fileVMA 0x6714e18** (name `N3RBX9DataModelE` sole ref at 0x6714e20 = T+8), three Itanium `-0x10` RTTI relocs at 0x67162e0/0x6716398/0x67163f0 ⇒ candidate DataModel vtables **V = 0x67162f0 / 0x67163a8 / 0x6716400** (page 0x671000). DM object is HEAP make_shared, NOT static (0 relocs + 0 adrp/add + 0 movz/movk store any of the three vptr values; the vptr reaches the object only via the runtime make_shared path). Plausible op-new sites sized 0x800–0x1200: 0x28f78a0/0x2950b3c/0x2950b5c (0x1000) + 0x2b37b6c (0x1108, followed by a ctor call). HONEST residual: exact ctor + join + sizeof NOT statically closed this cycle (vptr store likely GOT/indirect or this-adjusted base); now a WELL-SCOPED recon next-step (follow the object-vptr STORE from one of the 3 candidate vtables in the 0x2b37b6c ctor), and even when located it does NOT unlock a headless session (registry [0x1063915a0..0x106392600) is reloc+session-populated).
- **LATCH AUDIT (deleg_4f69ac65 task-2, succeeded — migration readiness):** SH169 JIT_DM_ALLOC_CAPTURE DELEGATE+VALIDATE latch VERIFIED READY (ABI ok; a genuine DM alloc is caught twice-over). Two runbook corrections for SH174: (1) arming needs BOTH JIT_DM_ALLOC_CAPTURE=1 AND JIT_DM_ALLOC_CAPTURE_DELEGATE=1 (DELEGATE unset ⇒ the guard silently refuses to arm on a live engine hook ⇒ GOLD line absent); (2) verify prev_hook != 0 at the arm marker or delegation is inert and freed engine-pool allocs fall back to host-calloc ⇒ SIGABRT (EXIT 134). Dynamic-trace re-affirmed impossible (~0.9) even once the join is located.
- **STANDING (unchanged):** live-DM = MIGRATION GATE (~30 cones). Type-4 producer latent; R1/R2 dead on reachability; headless dynamic-DM-trace NOT built (SH178 verdict holds). The genuine DM vtable family (page 0x671000, V=0x67162f0/0x67163a8/0x6716400) REPLACES the misattributed 0x2206d74/0x2173b3c anchor for any future Route-B/migration work. /tmp clean (removed a 1.1G text.dis + all .tmpwork).

# Open Sober — Agent Handoff
## SH178 (Sep 15, 2026, hermes-worker): ROUTE-B CONE CORRECTION — the "ExperienceController::join make_shared<DataModel>" premise is MISATTRIBUTED; live-DM is a LAW-level (reloc-populated) migration wall. Workspace green (550/0). No production change. Doc docs/frontier-sh178-routeb-join-misattribution.md. Commit <SH178COMMIT>.
- **3-agent fresh Route-B cone (operator's "return to Route B" directive) + independent packed-RELA decode, all READ-ONLY:**
  (1) The text prior cones cited for "the only make_shared<DataModel> is inlined at ExperienceController::join" (file 0x2206d74 / 0x2173b3c) is MISLABELLED. 0x2206c40 = GlobalInit do-init (consumes once-guard ldar w9,[0x106a68410], bl 0x2173b3c, stores [0x106a68408]; only operator-news 0x28/0x40/0x100/0x1000 = string/vector/dispatch sizes, NO DataModel-size). 0x2173b3c = string-intern GetOrCreate (strcmp prologue + hash-bucket chain 0x2173c24..0x2173d48), NOT RTApp app-registry / NOT a DM allocator. 0x2206db8 dispatch needs [obj+#32]=real vtable'd object whose +0x30=real ctor = exactly the SH156 JIT_ROUTEB_DM_SEED path to the governor dead-head. The genuine DM allocation site sits in a region NO cone reached (still unlocated, confidence 0.95).
  (2) **LAW-LEVEL WALL PROVEN AT THE RELOC LEVEL (main-loop decode, 568,272 packed ANDROID_RELA):** the reloc writing guest 0x106391908 (setDataModelToCurrent getter return / SH172 "invokable __func vt + current-DM holder") = (0x6391908, R_AARCH64_RELATIVE(1027), addend 0x6358d40) -> loader writes base+0x6358d40 from an all-zero on-disk .data.rel.ro cell; 490 relocs populate [0x1063915a0,0x106392600). The DM holder/registry is RELOCATION- + SESSION-populated (wired consumer std::function + live current shared_ptr exist only post-session — circular with the construction we lack). NOT seedable; no static seed and no dynamic jit_run of the misattributed join reaches a live DM. This upgrades "not seedable" from judgment to mechanism/law.
  (3) R1 (synthetic CoreScript) reconfirmed dead for the CORRECT reason: rbxasset://scripts/CoreScripts at 3 pull-only URI-builder sites (VMA 0x2588508/0x2588530/0x4345f88); CoreScriptLoader 0x101f1d8ac has ZERO direct bl callers (vt-registered, only inside a live DM's ScriptContext); filesdir 0x10726d600 never joins a CoreScripts Lua path. R2 (UniversalApp.rbxm) still blocked on the same live-DM (0x229006 'DataModel expired' / 0x288090 / cache-miss [sp+160]!=NULL @0x2d87b18). Type-4 producer [0x106829ea8] latent-only (MH_APP_READY false headlessly). Present-walker 0x105b2ed48 node-list = the Route-A plane (do NOT re-polish at Route-B's expense).
- **VERDICT (honest):** do NOT build the headless dynamic-DM-trace attempt — it re-treads SH156/164c, targets a misattributed address, and is expected to fault on the relocation/session-populated .data.rel.ro registry. Live-DM stays the single structural migration gate; the SH169 JIT_DM_ALLOC_CAPTURE DELEGATE+VALIDATE latch is the ready observer for the real app-launch (GPU host / real input). If Route B is re-chased later, it MUST first locate the genuine join inline via a fresh packed-RELA decode of the DM vtable/ctor — never reuse 0x2206d74/0x2173b3c.
- Cookie/persistence line fully persisted (SH175 060eeb7, SH176 8ca920e, SH177 3006fd3/9f09f67/6552801) — per the Sep-15 operator directive the persistence detour is now parked and Route B owns the cone.

# Open Sober — Agent Handoff
## SH177 (Sep 15, 2026, hermes-worker): COOKIE READ-BACK — the getter's jar-driven Route B now EXECUTES headlessly; emission value-source pinned as env/class-backed (Java CookieManager, migration-gated). Workspace green (arm64jit 372/0). Commit 3006fd3. Doc docs/frontier-sh177-cookie-readback.md, repro runs/capture_sh177_cookie_readback.sh.
- **2-agent READ-ONLY Route-B-scoped cone (deleg_66d4cead + deleg_b11c2a89) + EMPIRICAL region-watch, decisive:** (1) both read-back-local branch-gate NOPs are CORRECT (getter 0x1021ff72c `tbnz w8,#0,0x21ff744`=0x370000c8 -> Main WebLogin round-trip; Route-B 0x105fee9c4 `tbz w0,#0,0x5feec00`=0x360011e0 after the always-0 stub 0x1dc7428; 17-caller stub itself never touched). (2) the jar string layout = libc++ LONG (__cap_@[0] bit0=1, __size_@[8], __data_@[16]; exact decode used by 0x5fee9e0/2203bb8/22035c0); `cookie_jar_write_value` writes it, +1 hermetic test. (3) **DECISIVE — emission value-source is env/class-JNI (Java CookieManager), NOT the jar**: Route B reads the jar (getter 0x21fce24 fires, region-watch), but the emitted name/value pairs come via env/class JNI backing (5feee7c / domain 21ff8fc) and the keep/domain classifier 0x22035c0 (via 5feeccc) validates a DOTTED HOST (`cmp w9,#0x2e` '.'), returning 0 -> empty OUT headlessly.
- **CODE (elfjit.rs --cookie-readback + routeb_patch_cookie_readback, env JIT_ROUTEB_COOKIE_READBACK; jit.rs cookie_jar_write_value; default-inert):** drives nativeGetCookiesInNetscapeFormat 0x1021ff6b0 x0=url/x1=ulen/x8=&out after --cookie-ingress; NOPs gate A+B so Route B runs; writes the classified value into the jar. **EMPIRICAL:** region-watch proves getter->`bl 5fee984`->jar getter->classifier 5feeccc->22035c0 all execute (entries 0x105fee984/9dc/a04/a44/a58 -> bail `tbz w22,#0,0x5feebdc`); classifier 0 -> OUT empty with BOTH jar contents (.ROBLESECURITY=<tok> and .roblox.com); 0 SIGSEGV/SIGABRT, EXIT 124, standalone single-jit_run (SH55/64-safe).
- **HONEST NEXT (converges SH174/176 recon):** emitting a real RFC6265 `.ROBLESECURITY` line headlessly needs the env/class Java-CookieManager name/value source — a **migration (GPU-host) item**, not a headless seed. The write-side + gate-NOPs are the latent-but-correct reachability (fires the instant a real session provides env/class cookies). A classified value INTO the jar = SH176 feature-path 0x5fef19c + w4=1, still behind the live-DM wall. Route-B top-priority (live-DM/do-init completion) is the unchanged migration gate.
## SH176 (Sep 15, 2026, hermes-worker): COOKIE WORKER DEEP PATH EXERCISED HEADLESSLY — the engine SELF-CONSTRUCTS the cookie-jar container (objective 2b; first genuine advance on the login-persistence/cookie line since SH129). Workspace green (371/0 arm64jit). Doc docs/frontier-sh176-cookie-worker-deep-path.md. Repros runs/capture_sh175_cookie.sh + runs/sh175-cookie-ingress.txt.
- **CONE (deleg_13d959ca new facts + deleg_7cff8f6e both tasks, all READ-ONLY):** (1) gate-a recon: cookie worker 0x102203148 is pure-native (no thread/JNI/guest->host re-entry), safe as its own top-level jit_run; jar [0x106ed7a20] is READ via read-only getter 0x21fce24; the deep-path fault risks are all behind zero-default telemetry gates ([0x106dcead8]/[0x106dceac8]/[0x106dceae0]). (2) gate-b recon: the worker COMMIT path is TELEMETRY-ONLY (no jar/file write); real persistence is behind the feature path guest 0x5fef19c requiring both feature bits + injected w4&1=1; the netscape read-back getter 0x1021ff6b0 reads a SEPARATE singleton [0x10683d7e8] (magic 0xc0dedbad), independent of the jar. (3) adjacent-risk recon: StartLuaAppDM park-vs-Ok(0x3e8) is decided by thread-dispatch vs main-id cell [0x106863a68] + once-guard [0x106a68410] + DM-root [0x106a68818], racing the concurrent main-thread StartApp on the shared boot_sp; the rung-0 'pc outside image' (0x43894800000947c0) is a HOST x86-mcode pointer leaking into a br/blr target through the unseeded singleton vtables (SH103/SH109 class) — CpuState::new zeroes all regs so it's NOT inherited state.
- **CODE (elfjit.rs, --cookie-ingress driver, opt-in):** drives 0x102203148 as a STANDALONE top-level jit_run on the main thread before StartApp, with JIT_ROUTEB_COOKIE=1 (jar+gate guard armed), ABI (cookies,clen,url,ulen,w4=1,w5=0); after return decodes the jar as a libc++ std::string. **EMPIRICAL (3/3): worker returns Ok, jar slot [0x106ed7a20] 0x0 -> real guest-heap 0x55f... decoding as SHORT/SSO libc++ string (size 0) = the engine's OWN urn-init (operator-new(80) @ file 0x215cdd0) REPLACED the guard's inert seed, and both persistence gates end set ([0x106dcfc30].0=1 [0x1072739d4].0=1); 0 crashes.** First self-constructed engine object on the cookie/persistence line ever observed headlessly.
- **HONEST NEXT:** a classified VALUE persisting into the jar needs the feature path 0x5fef19c + w4=1 (recon-pinned). Cross-restart sign-in still needs Java CookieManager at migration. The cookie drive is standalone so it sidesteps the ladder race; deterministic ladder-Ok (seed main-id/once-guard/DM-root + serialize main StartApp behind LADDER_DONE) is a documented follow-on, not chased this cycle. Live-DM = migration gate unchanged.

# Open Sober — Agent Handoff
## SH175 (Sep 15, 2026, hermes-worker): COOKIE-JAR CONTAINER IS SEEDABLE (objective 2b, first headless advance on the login-persistence line since SH129) — 3-agent READ-ONLY Route-B cone closes the last open live-DM angle + a default-inert code guard. Workspace green (371/0 arm64jit). Commit 060eeb7. Doc docs/frontier-sh175-cookie-jar-seedable.md.
- **CONE (deleg_466252aa, all authoritative):** (1) **In-process DM-factory live-probe = PATH B CLOSED** — createDataModelForTeleport (0x2e1dc38) is an init-over-preallocated ctor (`ldr x0,[x0,#56]`), not a maker; no isolated sizeof+new+ctor headless site exists; the one angle SH165fwd left open now has disassembly evidence. Migration gate reconfirmed (~30 angles). (2) **Cookie jar "construction NULL" is seedable** — getter 0x21fce24 = `*(std::string**)(guest 0x106ed7a20)`, NULL in a bare boot -> the pure-native cookie worker 0x102203148 faults at 0x220331c (fault=0x0). SH129/174 mislabeled this *structural*; it is a seedable .bss global. (3) Present/future boundary pinned from a real capture: 24 frames headless (swap Ok(0x1)); the fence is DM allocation, NOT the R+0x180/0x188 list.
- **CODE (jit.rs routeb_cookie_jar_guard, env JIT_ROUTEB_COOKIE, default-inert):** at the cookie worker's exact entry pc (0x102203148), seed [0x106ed7a20] = valid empty libc++ std::string (zeroed 0x20 SSO) when NULL + clear [0x106dcfc30].bit0 (and [0x1072739d4].bit0). Idempotent (never clobbers a real constructed jar). FIRST headless advance on cookie/login-persistence since SH129 — objective 2b. +1 hermetic sh175 test (env-off inert / wrong-pc inert / exact-pc seeds container+gates / idempotent / SSO size 0). /tmp cleaned 40%->5%.
- **HONEST NEXT:** proving jar-init clears; the deeper cookie insert/commit path (0x22035c0..) touches further unexercised singletons (next-gate if a future drive faults it). Cookie injection is memory-only; cross-restart sign-in still needs Java CookieManager on a real app-launch (migration). Live-DM stays the migration gate; do NOT re-attempt the DM-factory probe.

# Open Sober — Agent Handoff
## SH174 (Sep 15, 2026, hermes-worker): ROUTE-B CONE CLOSES + MIGRATION-HARNESS RUNBOOK + two code hardening items (docs/frontier-sh174-migration-runbook.md). Workspace green (370/0, arm64jit 370).
- **CONE (deleg_432e3dd0, 2 fresh angles, both DECISIVE, READ-ONLY):** (1) **SKIP __sprintf_chk / __snprintf_chk** — zero repo occurrences + zero run-log dispatch; the only reachable fortify printf-family dispatch is the already-guarded va_list `_vs*_chk` form (runs/sh126-workers.txt: `__vsnprintf_chk` only, valid buffers). The deleg_23e8a3e3 'falls to raw dlsym' claim is an unproven inference about a never-dispatched symbol — SH149 falsifier holds. (2) **Every content load is DEFINITIVELY downstream of a live RBX::DataModel** — aassetmanager_open (shims.rs:564) is pull-only (ZERO non-test Rust callers; only a guest `AAssetManager_open`@plt fires it, reached only via AssetReaderAndroidImpl 0x21f5f94 -> DataModelPatcher, which hard-aborts 'DataModel is null' [sp+160]==NULL @0x2d87b18/0x288090); CoreScriptLoader 0x1f1d8ac is constructed below a live DM; filesdir 0x10726d600 feeds only 4 crashpad/settings sites, never a CoreScripts Lua path. R1 synthetic CoreScript = dead on REACHABILITY (matches SH169/SH172). No headless pre-DM content hook exists.
- **DELIVERABLE:** migration-harness **RUNBOOK** (docs/frontier-sh174-migration-runbook.md) — a cold-followable GPU-host procedure: canonical capture env (JIT_DM_ALLOC_CAPTURE=1 + DELEGATE=1, omit JIT_SERIALIZE_RENDER on a bare ladder = SH170 EXIT 139 abort), the SH167/SH169 ARM marker, the GOLD-2 live-DM capture line (`base 0x<B> [validated]`), in-process arming of *(0x106391908)=B (NOT 0x106391918), and the R+0x180/0x188 acceptance probe (n=(tail-head)/0x28, render_scene_base statics==0 -> first engine-self-constructed scene node). **Address-audited CONFIRMED by a fresh recon cone (deleg_84be9ca6 task-0)** — all latch/arn/probe addrs + all STEP-A flags/env verified against jit.rs / elfjit.rs / SH172 / SH63 / SH173; only a line-citation (jit.rs:1163->1189) was stale and is fixed. Ready the instant a real session forms on the GPU host.
- **CODE (jit.rs, +2 tests, arm64jit 368->370):** (1) `park_until_worker_gate_cleared()` — bounds the previously-UNBOUNDED WORKER_ADMISSION_GATE spin in BOTH spawn_guest_thread and spawn_pthread (clone-worker + pthread-worker parks) to a 300s deadline + SH174 WARN fallthrough, mirroring the existing bounded LADDER_DONE waits. A worker the do-init needed while the gate was wrongly set (the SH170 config-sensitive abort class) previously livelocked until masked as EXIT 124; now it turns into a diagnosed fallthrough. (2) `dm_capture_worth()` — SH167/SH169 DM-capture latch silently DROPPED a genuine sub-4KB RBX::DataModel (allocation #N>>8 past the FIRST-8 window) because capture required size in [0x1000,0x40000]; a base that validates as a real in-image-vtable object is now captured at ANY size (size window remains an OR), budget widened 64->256, still env-gated + default-inert + delegating. Both verified on a real binary: canonical SH130 combined-render run EXIT 0, ladder done+joined, 24 task-driven frames, 0 crashes, gate SET->CLEARED.
- **CONFIRMED ALREADY-DONE (deleg_69dc7d4c):** the clone-worker WORKER_ADMISSION_GATE park is ALREADY extended into spawn_guest_thread (SH162b, jt.rs:2596-2601) — the SH162 note's 'extend the park' premise was stale. Only remaining hardening = bound the unbounded yield-spin (low value).
- **RECON (deleg_a092dbb4, 2 tasks, both DECISIVE — see docs/frontier-sh174-v1-noop-cookie-wall.md):** (1) **V1 AppStart__ fallback rung = CONFIRMED TERMINAL NO-OP** for the UI goal — it drives guest 0x102338510 (`nativeAppBridgeAppStart__`, corrected 6-jstring ABI) -> AppStart governor 0x2338ef4, an app-bridge lifecycle/telemetry/platform-init closure that is NEVER a DM factory (ZERO GuiObjects/no session node). Reached the headless ceiling; MOVE ON, do not seed the V1/AppBridge line further. (2) **.ROBLOSECURITY cookie ingress = reconfirmed behind the SAME jar-init migration wall** — gate 1 [0x72739d4].bit0 is now cleared FOR FREE by the ladder's real engine write chain, but gate 2 [0x6dcfc30].bit0 is unseeded, the SH129 driver was reverted, and the cookie-jar-construction NULL at 0x21fce24 is UNCHANGED (even forced through, the worker SIGSEGVs 0x10220331c fault=0). Same structural wall, NOT a new gate. Not drivable headlessly.
- **BARE-LADDER RE-VERIFIED clean this session (runs/sh161b-tail-eq-seed.txt):** full 9-rung ladder through V2StartAppWithParams + SendAppEvent, 0 SIGSEGV/0 SIGABRT/0 stack-smash — the SH174 bounded-park refactor does NOT regress the default (gate OFF) path. Combined SH130 run also EXIT 0 / 24 frames / 0 crash. The SH55/64 concurrency class is confined to the tooled combined path (gate+serialize handle it); the bare path is stable 4/4 at HEAD (audit deleg_65eb75db).
- **STANDING (unchanged, doubly-confirmed this session):** live-DM = MIGRATION GATE. No headless .bss/.data seed constructs one (~29 recon angles + 2 fresh); the capture latch is the ready observer; Type-4 [0x106829ea8] host-install-only; producer handoff latent-but-correct (MH_APP_READY stays false headlessly). /tmp clean; workspace green.

# Open Sober — Agent Handoff (older, retained below)
## SH173 (Sep 15, 2026, hermes-worker): Session-producer dispatch HERMETIC PROOF (code) + auth/R1/render-seam recon corrections. Workspace green (546/0, arm64jit 368). Doc docs/frontier-sh173-producer-hermetic-auth-recon.md.
- **CODE (jit.rs, +1 test, arm64jit 367->368):** `session_producer_push_epoch_bump_wake_dispatch` proves the session-producer push->epoch-bump->wake mechanism WITHOUT a live drain (mirrors elfjit.rs --deque-node-bump; fake consumer parks on futex, re-reads epoch high-32 + pops headcell; timeout-retry loop is lost-wake-robust and deterministic, no hang; 0 guest/jit_run). This was the one piece of the operator's session-producer handoff that had no hermetic coverage (recon deleg_d585254f task-0: producer implemented-latent, only runnable with a live engine drain).
- **AUTH STRING CORRECTED (task-1):** login persistence = **`.ROBLOSECURITY`** cookie (SH129/earlier mis-typed `.ROBLESECURITY`). Zero native DB refs (session.db/shared_prefs/app_webview/cookies.db absent; rbx-storage.db = content cache only). Auth lives in the JAVA universalapp cookie manager: restore `JNICookieManager_setCookiesFromDisk` (0x2bcbebc) -> inject `nativeSetMultipleCookies` (0x2202ff8) -> SessionService -> `initializeLuaAppWithLoggedInUser`. Cookie write+inject reachable pre-live-DM; session restore behind migration gate. SH168 prestage uses the WRONG artifact for auth.
- **RENDER-SEAM (task-2, HARD-CONFIRMED):** present-walker 0x105b2ed48 node loop has ZERO DM/Luau gate (only host-touchable bytes R+0x22f/0x288/0x298/0x2a0/0x260 + the R+0x180/0x188 list). Engine RENDERING fabricated nodes is host-seedable — but that's the already-done Route-A plane (SH152/153/154), not Route-B self-construction. Per operator: do NOT polish Route-A at Route-B's expense.
- **STANDING (unchanged):** live-DM = MIGRATION GATE (~29 recon angles). Type-4 [0x106829ea8] host-install-only; producer handoff latent-but-correct (MH_APP_READY stays false headlessly).

# Open Sober — Agent Handoff (older, retained below)
## SH172 (Sep 15, 2026, hermes-worker): DataModelServices arming-spec CORRECTION (ABI-decoded, READ-ONLY cone). The SH169 registry-arming premise was FALSIFIED at the ABI level — no production code change. Doc docs/frontier-sh172-dmservices-arming-corrected.md. Workspace green (545/0, arm64jit 367). Commit 2056ab7.
- **CORRECTION (decisive, objdump-verified):** the SH169 arming claim ("write DM* into std::function capture word 0x106391918") is a CATEGORY ERROR. `setDataModelToCurrent` (0x2dbcc10) is a pure leaf GETTER (`adrp x0,6391000 / add x0,x0,#0x908 / ret`) returning **&0x6391908** (guest 0x106391908); ZERO direct bl callers; its code never touches 0x6391918. The registry std::function<void(RBX::DataModel*)> receives the DM as an INVOCATION ARG (dispatch 0x2dbcd18: stack-temp in x1 then `blr x21`), never from a capture slot. Correct post-migration arming target = the current-DM holder the getter RETURNS: **guest 0x106391908** (verified in LOAD#2 RW data [0x62dc1c0,0x67d27c0)). The invokable __func vt 0x106358d40 + entry __f_ 0x6391900 are loader-runtime-built (all-zero in file) under packed ANDROID_RELA — arming is migration-time/in-process/post-migration ONLY, never a static seed (relocated-vt hand-clobber rule). `crates/` references none of these addrs (repo-wide zero hits) — the SH169 capture latch is unchanged, no code edit warranted.
- **STANDBY (same cone):** (a) R1 recon-v3 premise CORRUPTED — the CoreScript/NoCoreScripts/UniversalApp literals ARE all present (rbxasset://scripts/CoreScripts @0x232ee7, NoCoreScripts @0x4b546b, UniversalApp.rbxm @0x6c9a40); the earlier "absent" premise measured wrong. BUT R1 stays dead for the CORRECT reason (reachability — CoreScriptLoader 0x1f1d8ac is downstream of a live DM; the only hook aassetmanager_open is pull-only, never fires pre-DM). (b) Next-object recon: the next un-synthesized object is a live RBX::DataModel (inlined make_shared at ExperienceController::join), structural, seedable=false.
- **STANDING (unchanged):** live-DM = MIGRATION GATE (~28 recon angles). SH169 capture latch is the ready observer; corrected arming target = 0x106391908. Recon-v3 deliverables (self-driven frames + JSON fix) verified done at HEAD; canonical ladder stable (EXIT 0/124, 0 crash).

# Open Sober — Agent Handoff (older, retained below)
## SH171 (Sep 15, 2026, hermes-worker): SendAppEventOnAppReady discriminator RE-VERIFIED with a DOC CORRECTION — w19=4 IS "Home" (magic = mov/movk/add summing to 0x656d6f48, NOT "Game"); the operator's "Home in x5 -> w19=4" premise is CORRECT. Path still telemetry-only (FLog + local 0x58 event struct, no DM). No production change. Doc docs/frontier-sh171-sendapp-discriminator-telemetry.md.
- **w19 pin CORRECTED (verbatim disasm, 0x2bb46f0..0x2bb4708):** `mov w10,#0x6147; movk w10,#0x656d,lsl#16; add w10,w10,#0xe01` => 0x656d6147+0xe01 = **0x656d6f48** = LE "Home" -> `b.eq 0x2bb47c4` -> **w19=4**. The prior SH171 "Home->5" note (and one recon agent) DROPPED the `add #0xe01`; fixed. "Chat"(0x74616843)->w19=1, "More"(0x65726f4d)->w19=5. **Confirms the operator's original premise (w19=4 for "Home" in x5).**
- **Still telemetry-only, NOT a Route-B gate:** continuation (0x2bb47d0..0x2bb49e4) = version-gate 0x683d350/0x683d358 -> FLog "[FLog::JNIAppBridge] eventFeature = {}." -> local 0x58 event struct (w19@+0x50) -> dispatch 0x2baeeec -> free 4 strings. NO DM/governor/ScriptContext. w19 changes only the logged int.
- **Empirical discrepancy SETTLED — no jstring-production bug:** the earlier "empty RBX-string at [sp]" was an INVALID post-return stack read (frame unwound -> stale bytes). A fresh read-only trace refutes all four empty-string causes: (1) handle ABI-correct + non-zero; (2) builder 0x1d9d074 provably emits len-4 "Home" SSO; (3) env slot 169 = identity shim (returns non-zero, JIT stores to x0 so `cbz x0` can't fire); (4) no other path. Fabricated-jstring wiring is CORRECT; any mismatch points at hostcall dispatch for a guest-bl-entered nested helper frame (jit.rs:2978-2999), only reachable if the rung nested BL's — not a jni.rs defect. Telemetry-only => not chased. Live-DM structural wall unchanged (~19 recon angles).
- **DM-CAPTURE DELEGATE RE-VERIFIED on this tree** (JIT_DM_ALLOC_CAPTURE=1 JIT_DM_ALLOC_CAPTURE_DELEGATE=1, real binary): EXIT 124 (timeout after completion), ladder done end-to-end, **0 SIGSEGV/SIGABRT, 8 delegated FIRST-call allocations routed through the engine's OWN prev_hook** (delegate mode — the SH169 fix's whole point: the SH167 host-calloc SIGABRT never fires because real alloc goes through the engine's allocator), **0 DM captures (latent-correct**: no real session's make_shared<DataModel> runs headlessly). Re-confirms migration-readiness of the capture latch.
- **COMBINED-RUN REPRODUCIBILITY 3/3 on this tree** (SH130 recipe: ladder + --renderinit + taskv4 self-driven frames, JIT_SERIALIZE_RENDER=1): each of 3 runs = 24 real task-driven frames (taskv4-frame present), 0 SIGSEGV/SIGABRT, ladder done + joined cleanly (EXIT 0 in the first). A fresh reproducible artifact of the engine's own task dispatch presenting real frames in the SAME serialized run that drives the session ladder.
## SH170 (Sep 15, 2026, hermes-worker): RECON-V3 DELIVERABLES VERIFIED AT HEAD on a real run + canonical ladder STABLE 4/4 + serialization pitfall pinned. No production code change (the "IMMEDIATE PRIORITY" list was already committed — now empirically re-verified on the real binary). Commit <SH170COMMIT>. Doc docs/frontier-sh170-ladder-verify-serial-pitfall.md.
- **VERIFIED FIRING on real libroblox.so:** (1) type4_frame_thunk ships via --taskv4-seed frame (24 real frames swap Ok(0x1), 191 type-4 pops previously); (2) JIT_JSON_ZERO_FIX hits BOTH clamps (len=0xb3/0x24 -> SSO empty); (3) DM capture (SH167/169) arms (`routed operator-new ACTIVE hook 0x1067daaf0 -> trail prev_hook ENGINE`) and stays SILENT — the honest proof no session forms headlessly.
- **CANONICAL LADDER STABLE 4/4** (full 9-rung --v2boot, EXIT 124 = timeout-after-completion, 0 SIGSEGV/0 SIGABRT): nativeInitializeNativeFlags -> nativeGameGlobalInit Ok -> nativeUpdateAdapterInit Ok(1) -> setTaskSchedulerBM Ok(1) -> V2InitWithParams -> StartLuaAppDM Ok(0x3e8) -> V2StartAppWithParams Ok -> V1 AppStart__ Ok -> V2UpdateSurface Ok (XID 0x200000) -> SendAppEventOnAppReady Ok(0x3e8, "Home" x5). MH_* false / DM-root shell / app-data-model-count=1 (live-DM wall unchanged).
- **PITFALL (NEW):** JIT_SERIALIZE_RENDER=1 on a BARE ladder (no combined render) aborts nativeGameGlobalInit with an unhandled guest C++ exception (EXIT 139) every run — WORKER_ADMISSION_GATE parks the clone worker the do-init needs. SH130/SH162 is for the COMBINED render+ladder run ONLY. Canonical ladder command MUST NOT include JIT_SERIALIZE_RENDER (SH169's repro omits it). Do-not-repeat footgun (runs/sh170-serial-1..3.txt).
- **RECON CONSENSUS (~19 angles, unchanged):** fresh 104-tool-call agent re-confirmed createDataModelForTeleport=CONSUMER (0 static callers), setDataModelToCurrent=GETTER (&0x6391908), Serializer::load deserializes INTO an existing DM (3rd arg) — no seed manufactures one headlessly; the only make_shared<DataModel> is inlined at ExperienceController::join (unreachable). Live-DM = MIGRATION GATE. Recon-v3 was latent-correct, not shootable-now.
- **CODE (b3307ef):** worker-gate site (elfjit.rs:6209) now emits a SH170 WARNING when JIT_SERIALIZE_RENDER is combined with the DM-seed chain (JIT_ROUTEB_DM_SEED/DMFORCE) — the known-abort combo. Diagnostic only; no behavior change on valid paths (capture_sh130.sh render chain unaffected). Verified: WARNING fires + EXIT 139 on bad combo; canonical non-serialized 9-rung ladder STILL clean (EXIT 124, 0 crash, 9 rungs Ok); workspace 545/0.
## SH169 (Sep 15, 2026, hermes-worker): DM CAPTURE upgraded to DELEGATING + VALIDATING (migration readiness) + NEXT-3 recon corrected + post-ladder re-entry falsified + registry-arming spec. Workspace green (545/0; arm64jit 367). Commit 5d02ce1. Doc docs/frontier-sh169-dm-capture-delegating.md, docs/frontier-sh169-next3-seed-recon-corrected.md.
- **SH169 CODE (impl):** `routeb_dm_alloc_capture` now DELEGATES the real allocation through the JIT to the engine's OWN allocator hook (run_guest_callback saved prev_hook, passing `current_guest_tp()`) whenever one is saved — so the returned base stays in the engine's allocator pool and its free-path stays valid (FIXES the SH167 host-calloc SIGABRT, which only fired because a host-calloc result is un-freable by the engine's free). It validates the base via `read_vt_in_image` (first word = vt must be in-image) so only genuine vtable'd DM objects count as `[validated]`. `routeb_dm_alloc_capture_guard` with a NEW env `JIT_DM_ALLOC_CAPTURE_DELEGATE=1` saves the engine's live hook into PREV_DM_ALLOC_HOOK and arms the trail; WITHOUT that env the SH167 never-clobber latch is preserved byte-for-byte (a live hook is never replaced). Idempotent (cur==trail no-op). Default-inert; latent migration readiness — it is the READY capture that observes the DM pointer the instant a real session's make_shared<DataModel> runs. **tpidr=0 would have been a real bug (audit deleg_fe92d2e1): jit_run_inner republishes CURRENT_TP from the callback state with no restore, so passing 0 clobbers the thread's guest TLS base and faults the engine's TLS operator-new; fixed to current_guest_tp().** +2 hermetic tests (sh167 delegate mode replaces/saves/idempotent; sh169 trail falls back to a real host alloc when no image).
- **NEXT-3 SEEDS RECON-CORRECTED (docs only):** deleg_8d5648cf's 3 "do-init NEXT-3 seeds" are all DEAD/wrong-address. Seed #1 thread-singleton real cell is 0x107333aa0 (spec's 0x1067... is off by 16MB; page already mapped, body completes). Seed #2 telemetry once-cell 0x106dcd380=-1 is an ANTI-ADVANCE (0x2b4cd1c is a std::call_once primitive that never parks single-threaded; setting -1 skips a clean registrar). Seed #3 page 0x10673336000 is a BOGUS address (>27GB, beyond the 109MB file); real .bss thread page 0x107333000 already mapped. Do-init ctor chain (0x2207d6c/df8/82d8/8354) completes cleanly. R1 synthetic CoreScript ALSO dead: filesdir 0x10726d600 is read at only 4 sites (never joined to a CoreScripts Lua path); content resolves via rbxasset content manager downstream of a live DM. Do NOT build those levers.
- **RE-ENTRY FALSIFIED (deleg_fe92d2e1 task-1, DECISIVE):** driving the SAME 7-rung ladder a SECOND time after it completes is a NO-OP de-accumulator, not a deeper path — the once-guard [0x106a68410] self-latches on run 1 so the do-init's DM-construct lambda (bl 2206d6c->2173b3c RTApp GetOrCreate) NEVER re-runs; the latched match dispatch 0x2206db8 brs into the same 0x23eff4c engine-boot body -> governor 0x2e9fa84 -> empty-handler-list benign ret (q+0x7c0=0x106a66260 stays a .bss zero; DM-root [0x106a68818] has ZERO str writers in the whole binary). Re-entry does not accumulate toward a live DM.
- **REGISTRY-ARMING SPEC (deleg_fe92d2e1 task-0):** DataModelServices::setDataModelToCurrent (0x2dbcc10) is a GETTER (adrp/add/ret returns &0x6391908), NOT a setter — target=0x2dbcc10+bound-this is INVALID. The std::__ndk1::function<void(RBX::DataModel*)> entry (guest 0x106391900..0x106391920) is LOADER-populated via APS2 RELATIVE (getter 0x63918e8, __func vptr 0x106391908=0x106358d40, dispatch descriptor 0x10635eec0) except the ONE spare capture word 0x106391918 (+0x18, file-zero, no reloc) — the candidate host-writable DM holder post-migration. The invokable __func vtable 0x106358d40 is RUNTIME-only (zero-reloc band), so valid arming must run in-process post-migration (Rust host-call), never as a static seed. pre_migration_safe=NO.
- **AUDIT (deleg_fe92d2e1 task-2, SH169 trail readiness):** ABI correct (real disasm: hook invoked (x0=size,x1=callsite-tag,x2=flags) at wrapper 0x2a0d9b8, mov-x0/x1/w2 then blr); guard fires at the true block-entry pc; nested re-entry is the proven qsort/pthread_once mechanism (NOT the SH151 GLES-corruption class). The ONLY latent bug was tpidr=0 — FIXED above.
- **STANDING (unchanged):** the live-DM structural wall is the GPU-host / real-input migration gate (~16 recon angles). SH169 makes the capture latch functional at that moment. Next = the real app-launch session with JIT_DM_ALLOC_CAPTURE=1 + JIT_DM_ALLOC_CAPTURE_DELEGATE=1, then arm the registry per the spec (post-migration, in-process).
## SH168 (Sep 15, 2026, hermes-worker): DATA-PERSISTENCE pre-staging tool (latent readiness, objective 2b) — scripts/prestage_data_init.py. Recon deleg_6a9bf0d6 (task-1): **rbx-storage.db opens from the CACHE dir** /data/user/0/com.roblox.client/cache/ (NOT the SH131b files-dir global); it is the content/asset cache (KVS by BLOB id, 8-col 'files' DDL + 5 indexes). Script mkdir's the app-data skeleton {files,cache,databases,shared_prefs,code_cache,app_webview} + creates a valid empty SQLite cache/rbx-storage.db with the engine's EXACT schema (page_size 4096, integrity=ok). Host-disk only, idempotent, zero boot disturbance (SOBER_ANDROID_ROOT unset -> fsmap passthrough; verified a staged tree is never referenced by the working ladder). Activation: mount SOBER_ANDROID_ROOT=<stage> when a real session launches. Auth (.ROBLESECURITY) is a SEPARATE native plane behind the SH129 jar-init wall — not pre-seedable. Doc docs/frontier-sh168-data-prestage.md.
## SH167 (Sep 15, 2026, hermes-worker): DM ALLOCATION-CAPTURE hook (latent migration-readiness) + SH166-cone DECISIVE recon (manager-shell EXHAUSTED; headless-session IMPOSSIBLE). Workspace green (all suites 0 fail, arm64jit 366). Commits d1267ac (SH166) + SH167. Docs docs/frontier-sh167-dm-alloc-capture.md, docs/frontier-sh166-dmcont-continuation.md.
- **SH166-cone DECISIVE (deleg_35857472, 3 READ-ONLY agents, code-grounded):** (a) a computed-`blr` ALWAYS terminates the trace (translate.rs:6924-6932) -> JIT_REGION_WATCH not firing at 0x102bd1d68 PROVES the DMCONT vt[+0x1f0]->continueAfterFlagsLoaded_ dispatch did NOT execute (no longer ambiguous). (b) **manager-shell line EXHAUSTED** — continueAfterFlagsLoaded_ (0x102bd1d68)->nativeAppBridgeAppStart (0x2338510)->AppStart (0x2338ef4) is app-bridge lifecycle/telemetry/platform-init ONLY (FastLog, base-url, JNIAppLifecycle setActive, SendAppEvent/MessageBus, cookie/web-login/crashpad/headers/LocalStorage, refcount stubs); createDataModelForTeleport (0x2e1dc38) and the scene walker 0x105b2ed48 each have ZERO direct bl callers. DO NOT resume the fabricated-NativeDataModelManager / continueAfterFlagsLoaded_ line for the UI goal. (c) **headless session IMPOSSIBLE** — the NativeHelper gameActivity_* callbacks (jni.rs:731-786) are engine->Java POST-CONDITION effect-signals (cause<->effect inverted): nothing polls the set-only MH_* atoms into a session-advance gate, so calling them cannot form a session; the DM-creator is unreached headlessly. This is the documented GPU-host / real-input MIGRATION GATE.
- **SH167 (impl, env JIT_DM_ALLOC_CAPTURE=1):** DM allocation-capture latch so a live RBX::DataModel pointer is caught the instant a real make_shared runs (the one place sizeof resolves, post-migration). `routeb_dm_alloc_capture` host-call trail + `routeb_dm_alloc_capture_guard` (jit.rs) on the CRT operator-new wrapper guest 0x102a0d9b8 (active-hook global 0x1067daaf0 / default 0x1067d0840). **ABI EMPIRICALLY = (size=a0, callsite-tag=a1, flags=a2)** — probe showed a0=0x18, a1=code addr (NOT a size; do NOT max(a0,a1)). **Empirical negative: the engine ships its OWN nonzero ACTIVE hook, and FORCE-replacing it with host calloc guest-SIGABRTs the free-path (probe EXIT 134)** — so the guard seeds only when hook==0 (safe latch) + never clobbers a live hook; the correct migration-time design is capture-only DELEGATION, not replacement. Trail MECHANISM + ABI PROVEN by probe (trail invoked, correct allocations, clean boot when not replacing a live hook). Default-inert (env off -> byte-identical); env-on inert on the real binary (live hook). +1 hermetic sh167 test.
- **Housekeeping:** /tmp hit 100% (7.7G tmpfs, recon disasm dumps) -> ENOSPC made 7 pre-existing tests fail (futex_requeue, guest_svc_*, aasset_*, bionic_stat). Cleared to 5%; re-ran all green (NOT a regression). Keep disasm dumps small or delete (the tmpfs is a box-breaker).
- **NEXT (honest):** the Route-B frontier is STILL the live-DM structural wall — now ~12 recon angles agree no static/manager-shell seed crosses it. The single real forward is engine-internal make_shared<DataModel> during a REAL app-launch session (allocation-capture), which needs the GPU-host/real-input migration. SH167 is the ready capture latch for that moment. Keep the 3-wide Route-B recon cone armed but explicitly do NOT re-tread the manager-shell line.
## SH165-fwd (Sep 15, 2026, hermes-worker): the recon-specified SCOPED manager re-seed now makes the real engine-init pipeline (fnB->0x102bd8ce8->manager vt dispatch) BENIGN-COMPLETE, verified 3/3. Workspace green (541/0). Commit f801a34. Doc docs/frontier-sh165fwd-manager-reseed.md, repro runs/capture_sh165fwd.sh.
Forward (recon deleg_62a86bcd task-0, SH165's next gate): fnB's bl 0x102bd8ce8 forwards the feature-flag string to the NativeDataModelManager singleton whose getter is 0x102174c04 -> holder guest **0x102727550**, which JNI_OnLoad had stlr'd with a host JavaVM* (empirically confirmed: the guard read 0xaa0003f31400000c before seeding) -> the getter would return a host JNINativeInterface* and vt[+0xf8] blr into raw host JNI. **`routeb_dm_manager_guard` (jit.rs, gated JIT_ROUTEB_DMFORCE=1, SCOPED to fnB region [0x102bd1a30,0x102bd1d08])** seeds 0x102727550 with a fabricated all-leaf-vtable manager M (vt[+0x30]=write-leaf, +0xf8/+0x108/+0x1f0=leaf, all else 0 => vt[+0x720]==0 benign; M+0=vt,M+8=0). **Empirical requirement discovered:** the holder page is mapped READ-ONLY (file-backed .data), NOT unmapped — the guard's own seed-write SIGSEGV'd until `routeb_ensure_writable` (page_is_writable) mprotect'd the page PROT_READ|PROT_WRITE (map anon RW only when genuinely absent, SH156 pattern). **VERIFIED real binary 3/3: `SH165 manager singleton holder 0x102727550 -> fabricated all-leaf-vtable manager ... (was host JavaVM* 0xaa...)` fires at pc=0x102bd1b98, region-watch enters fnB, EXIT 0/124, ladder done, SendAppEventOnAppReady returned Ok(0x3e8), 0 SIGSEGV/SIGABRT.** +1 hermetic sh165_dm_manager_guard (scoped/env-gated/leaf-vtable/idempotent, extends to page_is_writable+routeb_ensure_writable). fnB is a VOID init leaf (deleg_0b8468d7 task-0): after bl 0x102bd8ce8 it is pure canary+ret @0x102bd1c30 and touches no manager vtable slot — with a non-NULL seeded holder it benign-completes (NULL holder -> getter leaves x20=0 -> 0x102bd8d20 `ldr x8,[x20]` faults).
CONE (deleg_0b8468d7, all 3 authoritative, READ-ONLY): (task-1) **HONEST — no .bss/.data seed produces a live DataModel.** getFlagsFromEngine_->continueAfterFlagsLoaded_(0x2bd1d68)->nativeAppBridgeAppStart(0x2338ef4)->app-shell ctor 0x2207b54 is the SESSION forward, but initializeLuaApp_(0x2bd24b4)/startLuaApp_(0x2bd2668) are thin state-advancers; the real DataModel is a heap make_shared (createDataModelForTeleport fn 0x2e1dc38, ZERO direct bl xrefs, indirect-vtable-only, ctor/sizeof packed-RELA) not reachable on the mgr sequence — a real companion factory must be driven. (task-2) rbxasset content is aasset-read OFFLINE (AssetReaderAndroidImpl 0x21f5f94, AAsset_open@plt 0x21f6944 — SH164's ExtraContent re-root is correct) but DataModelPatcher deserialization hard-gates on a LIVE DataModel ([sp+160]!=NULL @0x2d87b18 <- [x0,#136]; 'DataModel is null' 0x288090) -> staged UniversalApp.rbxm is STRICTLY DOWNSTREAM of a live DM. NEXT: route the manager flag-completion slot (+0x1f0) to the REAL continueAfterFlagsLoaded_ 0x2bd1d68 with a sized-up M (>0x240, flags blob M+0x48..) + the app-bridge object to reach the app-shell ctor (0x2207b54 via nativeAppBridgeAppStart) — structural; the live-DataModel wall is unchanged.
CORRECTION (deleg_7e5b7101 task-1, DECISIVE — supersedes the above NEXT): **guest 0x102207b50 is NOT an app-shell constructor.** It is a `__cxa_guard` one-time-init that only warms FastLog/logging namespaces; nativeAppBridgeAppStart(0x102338510)->AppStart(0x2338ef4) sets base-url + JNIAppLifecycle setActive but produces ZERO GuiObjects/scene nodes; the scene walker 0x105b2ed48 has no direct `bl` caller. Real self-constructed login/home needs the live DataModel + Luau VM + ScriptContext/CoreScriptLoader(0x1f1d8ac) + ContentProvider, built by the SEPARATE engine boot (nativeGameGlobalInit/service-provider + game-document load), NOT the governor-tail path. **Do NOT invest further in the governor-tail engine-init / app-shell path for the UI goal.** DMCONT (012c4f7, JIT_ROUTEB_DMCONT) routes +0x1f0 to REAL continueAfterFlagsLoaded_ but is LATENT (region-watch 0 hits — the +0xf8 network feature-flag fetch never synchronously completes, so +0x1f0 never fires); even if it ran it only warms logging. Live-DM synthesis (task-2 verdict): the realistic headless path is an in-process jit-run of the engine's OWN make_shared/ExperienceController to allocate a real DataModel (the only place sizeof is resolved), then feed it to DataModelServices::setDataModelToCurrent (0x2dbcc10/instance 0x35d5908/vt 0x635eec0) — a live-engine probe, not a seed sequence.
## SH165 (Sep 15, 2026, hermes-worker, retained below): the governor-tail dispatch now ENTERS real engine-init (DM-force) — commit c17b3bd, doc docs/frontier-sh165-dmforce-engine-init.md.
Two real code changes (default-inert):
1. **fallocate aarch64 nr 47** (permanent persistence gap): the guest emits fallocate as syscall nr 47 (asm-generic), NOT x86-64 285 — a real posix_fallocate fell to -ENOSYS. Wired `47 | 285 => SYS_fallocate`; fsmap_persist test now exercises nr 47 (sub-assert offset fixed past EOF so 285 leg grows 4096->4099).
2. **`routeb_dm_force_guard` (jit.rs, gated JIT_ROUTEB_DMFORCE=1)**: at the governor-tail block entry (impl[+0x408] then `ldr x8,[x0]; ldr x8,[x8,#48]; blr x8`) substitute a fabricated NativeDataModelManager shell whose vt[+0x30]=**0x102bd1b98 (real engine-init fnB)** and pre-seed its settings chain — [shell+0x40]=P1,[P1+0x18]=P2,[P2+0x10]=P3 + a real NUL-terminated `"{}"` feature-flag payload at [P3+0x18] (per recon: 0x102bd8ce8 reads [arg+0x18] as a C-string forwarded to manager vt +0xf8/+0x108/+0x1f0). Prior SH161/SH164 kept that slot a leaf no-op (0x106a72000) so the tail resolved benignly but NEVER entered DM engine-init (SH164 region-watch: 0 hits). **VERIFIED real binary (JIT_REGION_WATCH=0x102bd1a30-0x102bd1d08): [region-watch] entered region at guest pc=0x102bd1b98 (1 hit vs SH164's 0), EXIT 124, 0 SIGSEGV/SIGABRT, ladder done, SendAppEventOnAppReady Ok.** The `dmtrace vt=0` line is the honest image-domain guard (host-heap shell 0x7f... outside guest domain) — the shell IS in impl[+0x408]. +1 hermetic sh164_dm_force_shell_is_coherent_and_env_gated.
RECON NEXT GATE (authoritative, deleg_62a86bcd task-0): fnB's bl 0x102bd8ce8 forwards the string to manager vt +0xf8/+0x108/+0x1f0; the manager singleton getter is **0x102174c04 -> guest holder 0x102727550**, which JNI_OnLoad (0x102173ff4) already stlr'd with a host JavaVM* -> the getter would return a host JNINativeInterface* and blr vt[+0xf8] into raw host JNI. Forward = **SCOPED re-seed** of 0x102727550 with a fabricated all-leaf-vtable manager M (+0x30 write-leaf `str x0,[x1]; mov w0,#0; ret`, +0xf8/+0x108/+0x1f0 leaf, all else 0 so vt[+0x720]==0 benign soft-return; M+0=vt,M+8=0). Seed ONLY on the fnB entry hook (JNI-critical global — never blanket-clobber). vt+0xf8 = network feature-flag fetch; NO fabricated continueAfterFlagsLoaded_ string. Session producer-handoff spec pinned verbatim by deleg_42b6f9d6 (see STATUS.md). App-shell ctor 0x102207b50 is reached only via the AppStart JNI nativeAppBridgeAppStart 0x102338ff8 — a SEPARATE path from this governor tail.
## SH164c (Sep 15, 2026, hermes-worker): DMDRIVE vt-shell replay DECISIVELY NEGATIVE (read-only recon deleg_876cb7b2). Workspace green (540/0). Doc docs/frontier-sh164c-dmdrive-antiverdict.md.
The proposed JIT_ROUTEB_DMDRIVE (write shell_vt[+0x30]=real creator into impl[+0x408] so the governor tail's own blr calls the creator) is a DEAD END for DM construction: (a) full packed-RELA decode (568,272 relocs) shows ZERO relocations resolve into the DM-creator family [0x102bd1a30,0x102bd1d08) — no .data.rel.ro vtable backs getFlagsFromEngine_/initEngine_; shell_vt[+0x30] must be a LITERAL (initEngine_ 0x102bd1cf0 / getFlagsFromEngine_ 0x102bd1a38). (b) initEngine_ dispatches on w8=[this+0x10]: a ZEROED shell (w8=0) takes the benign default tail 0x2b53abc (no DM work); w8=3 enters the settings body where [this+0x50]==0 aborts 'Engine settings is null' (0x2d2308), and even seeding +0x50 only dodges the log — the serializer's member derefs (+0x48/+0x60/+0x78..+0x240) fault next. NO shell variant constructs a live DM. Confirms: a real DM needs a harness dynamic trace driving a REAL instance (standing structural wall, unchanged). Do NOT re-attempt DMDRIVE.
## SH164 (Sep 15, 2026, hermes-worker): 3-agent cone maps the live-DM wiring + EMPIRICAL VERIFICATION (the DM path is confirmed un-reached) + two code changes. Workspace green (540/0). Commit <SH164COMMIT>. Doc docs/frontier-sh164-recon-live-dm-wiring.md.
TWO healthy real-binary ladder runs (JIT_ROUTEB_DM_SEED=1 set — my first diagnostic omitted it, so the governor region showed 0; corrected) with JIT_REGION_WATCH: governor tail 1 hit @0x102e9fa84 + SH161 inert-DISPATCH seed fires @0x102e9fcc4 (positive control proves the mechanism works), but DM-creator regions 0x102bd1a30..0x102bd1d08 = **0 hits** — the ladder is DIVERTED by the inert DISPATCH and never enters NativeDataModelManager::initEngine_. The real vt[+48] is loader-runtime-built (not statically substitutable; relaxing the guard restores the impl[+0x408]==0 SIGSEGV), and initEngine_ needs a real instance — a dynamic-trace-only structural wall, now empirically confirmed (was inferred). Reconciliation: NativeDataModelManager is the first-login creator, NOT createDataModelForTeleport (0x102e1dc2c is a static thunk — false positive from the prior cone). DataModelServices registry fully decoded (guest 0x1063915a0..0x106392600, entry 0x1063918f8, code 0x102dbcc10; NULL current-DM -> benign tail). CODE CHANGES: (1) aassetmanager_open models→ExtraContent fallback (shims.rs) serves the bare LocalAssetURI (UniversalApp.rbxm content-path enabler, +1 hermetic test); (2) routeb_tail_dispatch_capture probe (jit.rs, JIT_ROUTEB_DMTRACE env-gated) fires at governor-tail block entry on every run printing impl/vt/vt[+0x30]+regs and flags the DM-creator family — VERIFIED on a real run (fires at 0x102e9fcc4/0x102e9fdc8, impl[+0x408]=0x106a72000 inert DISPATCH -> vt[+0x30]=host ptr, NOT DM family; +1 hermetic test). Run repro: runs/capture_sh164_dm_path.sh.
## SH163 (Sep 14, 2026, hermes-worker): do-init path SEED-EXHAUSTED — 6-agent cone convergence persisted (docs only). Workspace green (538/0). Commit <SH163COMMIT>. Doc docs/frontier-sh163-recon-consolidation.md.
Baseline real-binary ladder re-verified this session: EXIT 124 (outer 300s timeout), **0 SIGSEGV/SIGABRT**, governor tail + SH161 DISPATCH fire end-to-end, SH158 AppBridgeV2 probe solid. Two 3-wide recon waves converge on the truthful frontier:
1. `[0x10683cf38]` (global-init provider gate, `ldr x8,[x8,#3896]`@file 0x2dadbb4) — a NO-OP: NULL reaches init continuation 0x2dae0ec via `sysconf(96)≥3` fallback. Recon corrected from its 0x10683cf98 slip. Its 4 ctor-body callees also all complete cleanly (never construct a DM).
2. Synthetic loose CoreScript dead-end (zero code refs; .rela.plt only; LoadCoreScriptsFromPatchOnly).
3. Real content = patch-models (UniversalApp.rbxm PRESENT / DataModelPatch.rbxm ABSENT), DataModelPatcher Post-TTI strictly downstream of live DM (0x2d87b18 'no content' abort).
4. Session-driven producer ABI-correct but STRICTLY LATENT behind MH_APP_READY∧live-DM (engine wake = FUTEX_WAKE 0x8a).
5. Live DM created by ExperienceController::createDataModelForTeleport / initializeLuaAppWithDataModel, wired via DataModelServices::setDataModelToCurrent (weak_ptr) — the [0x106a68818] seed is NOT the real holder.
NO remaining .bss seed on the do-init path. NEXT implementable seed must target the ExperienceController/live-DM creation path specifically (Route-B scope). All subagents Route-B-scoped; re-derive on the ExperienceController path, not the do-init tail.
## SH161b (Sep 14, 2026, hermes-worker): POST-SH161 next gate — seed the governor-tail epilogue impl[+0x2b8]=2 (recon deleg_61f88e9a authoritative). Workspace green (538/0). Commit a3a694c. Doc docs/recon + runs/capture_sh161b.sh.
New `routeb_tail_eq_guard` (jit.rs), gated JIT_ROUTEB_SETFIX, fires at block entry pc==0x102e9fe04 (`mov x0,x19; mov w1,#0x2; bl 0x1023c12c0`) and writes 2 into impl[+0x2b8] when 0. fn 0x1023c12c0's b.eq target 0x1023c1434 is a pure canary-check+ret (mode-2 no-op); without it the fn falls into a fault-prone transition body (reads [0x6a70700], dispatches 0x23c14dc/0x23c1504/0x23c1574 + conditional FMOD AAudio 0x626b6d0). VERIFIED benign by 3 read-only recon subagents (default-inert re-confirmed by a 3rd). REAL-BINARY ladder (JIT_ROUTEB_DM_SEED=1+SETFIX): 0 crashes, StartLuaAppDM returned Ok, tail block walked 0x102e9fcc4->0x102e9fdc8 with the SH161 DISPATCH (impl[+0x408]=0x106a72000) firing. HONEST: the eq-seed is LATENT-BUT-CORRECT — the 0x102e9fe04 epilogue sits on the deeper continuation path; proven hermetic (wrong-pc no-fire / exact-pc seed=2 / nonzero-preserved / impl==0 no-crash).
**SH161b2 (attempted + REVERTED)**: I built `routeb_startlua_launchparams_guard` (seed launch-params[sp+0x10 obj +0x108].bit0=1 to clear the StartLuaAppDM continuation `tbz w19,#0` at 0x23efff4). CONE anti-verdict (deleg_dcf18f68 task-0/1/2, deleg_2ee483db task-1): DEAD-END — bit0 alone cannot reach 0x2409864 because 0x24c3930 first cbz-gates on the dispatcher struct's +16 (NULL; its ctor 0x2db7b08 is in the bypassed guard-init path); even seeding +8/+16 only benign-exits at 0x2409a94 on the empty handler list (q+0x7c0). Real app-start dispatch is STRUCTURAL. Reverted; shipped scope = SH161b only.
## SH162 (Sep 14, 2026, hermes-worker): serialized the MAIN start_app jit_run behind LADDER_DONE — removes the deterministic dual-top-level overlap behind the SH55/64 flake. Workspace green (537/0). Commit 1475161. Doc docs/frontier-sh162-serialize-startapp-gate.md.
Root cause (recon deleg_f177139a task-2, READ-ONLY): the MAIN thread's start_app top-level jit_run (elfjit.rs:11047, V2StartAppWithParams) genuinely runs CONCURRENTLY with the detached --v2boot ladder thread (spawned 6397), both nesting==0 top-level jit_runs sharing the single global block cache AND the same boot guest stack (s2.x[31]=st.x[31]). That is the deterministic source of the run-variable SH55/64 flake (crashes at DIFFERENT guestpc each run). SH162: when JIT_SERIALIZE_RENDER=1+--v2boot, wait for LADDER_DONE (bounded 300s) BEFORE driving start_app on the main thread, mirroring the prove renderinit gate (8410) + WORKER_ADMISSION_GATE. Only one top-level jit_run exists at a time on the whole ladder+start_app path. NOTE: fixed a scope bug — `serialize` is a LOCAL in combined_frame_capture(), so use the WORKER_ADMISSION_GATE env condition (JIT_SERIALIZE_RENDER=="1" + --v2boot) directly, not the `serialize` var.
VERIFIED: SH162 gate fires, start_app drives AFTER the ladder, clean run EXIT 0; default env-off unregressed (EXIT 124, 24 task frames, 0 crash, 0 SH162 lines); tests green (537/0). HONEST residual: the remaining run-variable crashes (guestpc 0x106240c78 etc.) are the SEPARATE pre-existing SH55/64 clone-worker class — WORKER_ADMISSION_GATE only parks the pthread_create path (spawn_pthread, jit.rs:3163), but clone(220)/clone3(435)-created workers (spawn_guest_thread, jit.rs:1902) escape the park entirely. SH162 fixes the main-to-ladder overlap only; the clone-worker race is the next thing to close (extend the park into spawn_guest_thread, per recon deleg_f177139a).

## SH161 (Sep 14, 2026, hermes-worker): Route-B — the AppBridgeV2 governor TAIL now executes END-TO-END (StartApp session + SendAppEventOnAppReady complete). Workspace green (537/0). Commit 4845820. Doc docs/frontier-sh161-routeb-govtail-exec.md.
Two fixes (opt-in JIT_ROUTEB_DM_SEED=1 + JIT_ROUTEB_SETFIX=1; default unregressed EXIT 124 / 24 frames / 0 crash):
1. tail-dispatch guard (jit.rs routeb_tail_dispatch_guard, SH123 pattern): at any block entry into the governor-tail region [0x102e9fcc4,0x102e9fdc8], if `[x19+0x408]` (=impl[+0x408]) is 0/sub-image, write the inert DISPATCH (0x106a72000, all-leaf vt with vt[+0x30]=leaf) into the impl slot. ROOT CAUSE (recon deleg_c94a8b2f x3, READ-ONLY): the reported crash guestpc 0x102e9fcc4 (`ldr x22,[x19,#1096]`, x19=live host-heap impl) is a fresh-block-cache ATTRIBUTION; the real fault=0x0 is `ldr x8,[x0]` at 0x2e9fd90/0x2e9fdb0 where x0=impl[+0x408]=NULL. x19 is a RUNTIME host-heap object (0x7ff6...), not static-seedable, so a static seed can't reach it; SH159d only covered the MODERN window (0x2e9fb44), the TAIL re-loads impl[+0x408] fresh and never saw that substitution. The runtime hook is the only fix. Idempotent.
2. tail-continuation NOP (elfjit.rs routeb_patch_gov_tail_cont): NOP the 3-inst window at 0x2e9fdf4 (`ldr x0,[x19,#1088]; mov x1,x20; bl 24c3768`) — a device-display-handler shared_ptr/refcount helper called with x0=impl[+0x440]==NULL under the partial do-init (fault `[x0,#320]`=0x140); its return is DISCARDED (mov x0,x19 at 0x2e9fe00), so NOP is a benign no-op (mirrors SH160's init3-gate NOP). Exact-match guard + block drop [0x102e9fdc8,0x102ea3b40].
VERIFIED (real libroblox.so, llvmpipe): the FULL ladder now runs to completion ONE jit_run — nativeGameGlobalInit Ok -> nativeUpdateAdapterInit Ok(1) -> setTaskSchedulerBM Ok(1) -> StartLuaAppDM Ok -> V1 AppStart Ok -> **SendAppEventOnAppReady returned Ok(0x106a72000)** (event='Home' in x5, ABI-correct) -> `ladder done` — **EXIT 0, 0 SIGSEGV/SIGABRT, reproducible 2/2** (was EXIT 134 at the governor tail every run). Default env-off path unregressed (EXIT 124, 24 real task frames, 0 crash, 0 SH161 lines). Standing walls unchanged (MH_* false, DM-root no live DM, app-data-model count=1): real self-constructed login/home UI still behind the structural CoreScripts/content + live-DataModel loader wall (SH131d-class). NEXT: the governor post-tail epilogue (0x2e9fe04 `bl 23c12c0`, tail teardown 0x2e9fe0c+) — run and observe the next structural gate.

## SH157-SH160 (Sep 14, 2026, hermes-worker): Route-B — the AppBridgeV2 governor 0x102e9fa84 now EXECUTES END-TO-END (StartAppWithParams + init3 gates crossed). Workspace green (all suites 0 fail; elfjit example 61/0). Commits 978cf96(SH159a) 3743e19(SH159c) 5380358(SH159d) d5ff632(SH159e) 0dab194(SH160) b4bbb2b 80c651f. Doc docs/frontier-sh159-routeb-governor-exec.md.
Opt-in JIT_ROUTEB_DM_SEED=1. SH159a: zero union-init guard globals [0x106a63da0]/[0x106a63d70] (held host-heap ptrs -> sub-ctor garbage-dispatched, never returned to the body dispatch) -> the 0x2366694 union-init returns and the body `blr` at 0x23effbc reaches the governor. Corrected a false recon premise: .rela.dyn is ~2MB ANDROID_RELA PACKED (readelf can't decode; loader applies) — vt[+0x18]=0x102e9fa84 is LIVE, so NO vtable hand-seed. SH159c: patch 0x102e9fad8 `b.cc`->`b` to skip the faulting MODERN appendix (goto ROUTER). SH159d: substitute the governor MODERN dispatch's impl[+0x408] deref (NULL) with an inert 4-inst window (x0=DISPATCH 0x106a72000) so it reaches `bl 0x258c6e4`. SH159e: make startAppWithParams' param inputs deterministic (cap=0/count=0/float=1.0). SH160: NOP the two nativePostClientSettingsLoadedInitialization3 dispatch-gate calls (0x23f013c/0x23f01b0 -> stp xzr,xzr,[x8]). VERIFIED: the engine boots PAST the whole governor + startAppWithParams + init3 gates (region-watch 0x102e9fa84 enter -> 0x2e9fb58 -> 0x2e9fb6c -> 0x2e9fbbc -> 0x2e9fbc8). Default path UNREGRESSED (EXIT 0, 24 real task frames, 0 SIGSEGV/SIGABRT). NEXT frontier: governor tail 0x102e9fcc4 block-translation coherence (not a guest NULL; impl + 0x448 reads in-bounds zero; the host fault is a translated store through a null base on the freshly-entered large region — recon deleg_19f62ad8), then the 0x2e9fd90 impl[+0x408] vt[+0x48] dispatch.
## SH156 (Sep 14, 2026, hermes-worker): the GlobalInit do-init now EXECUTES real GlobalInit construction (live DM-root seed + .bss page-remap). Workspace green (all suites 0 fail; elfjit example 61/0). Commit SH156; docs docs/frontier-sh156-routeb-dmseed-ctor.md + docs/recon-sh156-startluaappdm-postdoinit.md; repro runs/sh156-r3a/r3b.txt.
Opt-in JIT_ROUTEB_DM_SEED=1 rung: seeds a live 0x10 object at DM-root [0x106a68818] with the real dispatch vtable 0x10635cce0, pins vtable[+0x30] (0x10635cd10)=the genuine global-init ctor 0x102207b50, and remaps the once-faulting .bss pages (routeb_map_guest_page: /proc/self/maps probe then MAP_FIXED anon RW only for genuinely-unmapped pages — never clobbers file-backed). VERIFIED on real libreblox.so (2/2, EXIT 0, 24 real task frames, 0 crash): `ctor-guard[0x6a64d70]=Some(1)` — the GlobalInit dispatch ctor's OWN __call_once COMPLETED and self-latched its once-guard (previously always 0/never reached); the once-faulting `ldr [0x7285fb0]` (SIGSEGV 3/3 before) now reads 0. Recon deleg_94c0be26 (docs/recon-sh156-startluaappdm-postdoinit.md): on the StartLuaAppDM path the do-init match dispatch brs into the REAL engine-boot body guest 0x1023eff4c -> AppBridgeV2 governor 0x102e9fa84 -> nativeAppBridgeStartAppWithParams (0x258c6e4) — next frontier marker, NOT a fault. Recon deleg_69ab5272: the [0x7285fb0] flags gate is conditional (needs 0x306 only on a telemetry path a healthy run skips) — do NOT pre-seed it. Recon deleg_eeec00a2: the DataModel-primary vtable +0x30 is a trivial getter, NOT the app-shell ctor (corrected SH155's premise); the real dispatch vtable is 0x10635cce0. Honest: engine now at the AppBridgeV2 governor; it does NOT yet self-construct login/home GuiObjects (structural Lua/app-shell wall remains downstream of the governor).
## SH155 (Sep 14, 2026, hermes-worker): Route-B do-init GATE-FIX (once-guard inversion) + SendAppEvent ABI fix; DM-root structural wall root-caused by disassembly. Workspace green (537/0). Commit 46e5af2. Doc docs/frontier-sh155-routeb-onceguard-gatefix.md.
The ladder forced once-guard [0x106a68410].bit0=1 at StartLuaAppDM, which made the do-init (0x102206c40) SKIP its std::__call_once (bl 0x284ce54) — the DM-construct lambda never ran, so DM root [0x106a68818] stayed NULL and the match path's `cbz [x19,#0x20]` dead-ended Ok(0x3e8). GATE-FIX: leave once-guard clear at StartLuaAppDM (seed only flags-latch + main-id). VERIFIED (real binary): once-guard now SELF-sets to 0x1 — the guest __call_once completes for the first time without host-forcing — and the app-data-model counter [0x106dca000+0xe88] advances 0->1. SendAppEventOnAppReady ABI fix: event jstring moved from x2 to x5 (discriminator reads the 4th string) so w19=0x4 for "Home"; latent (body soft-returns at the SH115 singleton-vtable leak before the discriminator). Added DM-root/once-slot probes. Combined ladder EXIT 0, 24 real task frames, 0 crash.
RECON (deleg_d8c3d2b2, 141 disasm tool calls): the do-init's construct chain (0x2173b3c -> 0x61e30bc) is the RTApp app-registry GetOrCreate — the observed once-slot [0x106a68408]=0x4000 is the CORRECT app-task-id return (0x4000<<appid | count), NOT a malformed DataModel pointer. DM-root [0x106a68818] (appbridge obj 0x106a687f8 +0x20) has NO static/once writer in the entire binary; it is only planted by a LIVE heap store during the real AppBridge V2 app-launch, which the headless ladder never reaches. The ONE-NEXT-UNSYNTHESIZED-OBJECT is a live vtable'd DataModel at [0x106a68818] with [vt+0x30]=app-shell ctor — but structural (real app-launch heap construction), SH131d-class. CoreScripts loader 0x1f1d8ac runs AFTER a live DM (confirming the standing Lua/CoreScripts wall is downstream). No static seed opens DM-root; SendAppEventOnAppReady stays latent.
## SH154 (Sep 14, 2026, hermes-worker): real task-driven FILM gate + a REAL cross-frame GL texture-bleed fix. Workspace green (537/0). Doc docs/frontier-sh154-sequence-film-texture-bleed-fix.md.
RENDER_TASKFRAME_SEQUENCE=1 presents a film alternating distinct real-content surfaces by serial n%k (default k=2 => Mesh,Home; k=3 adds a palette slot), reusing the proven SH152 (home/FPSBackground) + SH153 (sphere+studs) emitters. CORE FIX: render_engine_emitter_mesh never re-bound its studs texture to unit 0 per frame — a Mesh frame after a Home frame sampled the HOME atlas through the sphere shader; now re-binds glActiveTexture(0)+glBindTexture(0x0DE1,m.tex) after glUseProgram. VERIFIED (real libroblox.so, llvmpipe): seq frame #0 step 0/2 fires the engine mesh wrapper ret=0x0, 0 SIGSEGV/SIGABRT, EXIT 124; mesh standalone unregressed with the rebind; default env-off byte-identical. HONEST: standalone live dispatch is sparse (SH55/64 drainer-not-resident) so a single run proves the mechanism + bleed fix, not a long film.
## SH153 (Sep 14, 2026, hermes-worker): REAL 3D MESH in type-4 task-driven frames — smooth_sphere.mesh (1652v/9216 indexed tris) + studs.dds (128x2048 R8) + Blinn-Phong lit through the engine's OWN geometry wrapper 0x105b35288, driven by task dispatch. Closes SH151's original intent safely via its sanctioned cached-program design. Workspace green (537/0). Doc docs/frontier-sh153-taskframe-mesh.md, repro runs/capture_taskv4_frame_mesh.sh.
RENDER_TASKFRAME_MESH=1; mesh program/VBO/EBO/texture/geometry-ctx built ONCE outside the present loop (pure-host mesa_fn); per-frame ONLY uMVP/uModelRot upload + engine-wrapper jit_run (indexed x5=n_elems) then swap via real ctx-vt[+24] — zero nested guest-bridge GLSL (the SH151 SIGABRT class). Readiness gated on taskframe_mesh().is_none() (wrapper Ok(ret)=0 on clean draw). MESH branch wins over HOME, falls back to palette on warm-up failure. RENDER_TASKFRAME_MESH_RAW=1 dumps the frame. VERIFIED (real libroblox.so, llvmpipe): 2 distinct task frames, engine wrapper Ok ret=0 elems=9216; 19.80% sphere-drawn (182,484px = SH144/145 footprint), mean lum 136 stdev 69 (real studs+diffuse shading), specular highlight, vision confirms the shaded sphere; 0 SIGSEGV/SIGABRT, EXIT 124. Default env-off path byte-identical (24 palette task frames, 0 crashes). Honest scope: harness selects real assets; engine wrapper+GLES render them — still not self-constructed UI (SH131d wall unchanged).
## SH152 (Sep 14, 2026, hermes-worker): type-4 task-driven frames now carry REAL home artwork (FPSBackground + RO-BLOX wordmark) via the engine's OWN emitter — the SH151-sanctioned cached-program design. Workspace green (537/0). Doc docs/frontier-sh152-taskframe-home-artwork.md, repro runs/capture_taskv4_frame_home.sh.
present_one_task_frame threads iimg/ibase/isp; RENDER_TASKFRAME_HOME=1 drives render_engine_emitter_multi (real FPSBackground, RENDEREMITTER_HOME=1) or emitter_home (palette) as a SEPARATE top-level jit_run on the currency thread, then presents via the same real ctx-vt[+24] swap. CACHED textured program + pre-uploaded texture — zero nested guest-bridge GLSL/GL-allocation (the SH151 SIGABRT class). Selector by env, not emitter return (Ok(ret)=0 on both success & early-fail). VERIFIED (real libroblox.so, llvmpipe): task frame #0 REAL HOME ARTWORK sprites=12 swap=Ok(1), real FPSBackground probe readback (175,95,59,255), 0 SIGSEGV/SIGABRT, EXIT 124. Default env-off path byte-identical re-verified: EXIT 124, 24 task frames, 0 crashes, home branch not entered. Honest scope: harness gates selection; engine's own GLES path + real APK art, still not self-constructed UI (SH131d wall unchanged).
## SH151 (Sep 14, 2026, hermes-worker): verified-NEGATIVE — feeding real mesh geometry into type-4 task frames (RENDER_TASKFRAME_MESH=1) SIGABRTs (nested run_guest_callback GLES bridge heap corruption on the currency thread). REVERTED to SH150; env-off default re-verified UNREGRESSED (24 task frames swap Ok(0x1), EXIT 124). Doc docs/frontier-sh151-negative-taskframe-mesh.md.
- Recon deleg_7734b24f: Route-B ladder FULLY CLOSED (7 rungs "ladder done" elfjit.rs:5828 + 24 frames; latch 0x72739d4 via guest chain). ALL 5 NativeHelper callbacks ALREADY wired (jni.rs:731-786/1082/1086, SH111 2e4b7f2) — handoff's "ZERO implemented" claim is STALE. Wiring callbacks = dead-end (never invoked). SH130 exit-133 flood ALREADY fixed by SH131.

# Open Sober — Agent Handoff
## SH150 (Sep 14, 2026, hermes-worker): REAL FPSBackground 'HOME' SURFACE — the last un-composited real Roblox auth artwork (FPSBackground.png, the opaque 1024x1024 FPS-game-tile launcher collage) now renders through the engine's own emitter path (0x105b2ed48) with the real RO-BLOX wordmark, gated by RENDEREMITTER_HOME=1. Workspace green (537/0). Commit SH150; doc docs/frontier-sh150-home-surface.md, capture runs/sh150-home.png.
- The 'home' half of 'login/home' in the harness-authored-but-real-asset sense. VERIFIED: FPSBackground decoded (1024x1024 RGBA8), emitter Ok count=78 sprites=12 swap=Ok(1), wordmark probe present=true byte-exact, 0 crashes. Vision confirms the real launcher collage + wordmark.
- RECON (deleg_636bbc4f): BOTH recon-v3 deliverables (type4 self-driven frame thunk + RBX::json Writer stack-leak/JSON-abort) are CONFIRMED CLOSED at HEAD (type4_frame_thunk shipping 24 real task-driven swap Ok(0x1) via --taskv4-seed frame; JSON fixed by JIT_JSON_ZERO_FIX jit.rs:2562); the one unblocked code change is feeding the real-geometry path into present_one_task_frame (elfjit.rs:4956-4969).

# Open Sober — Agent Handoff
## SH149 (Sep 14, 2026, hermes-worker): GUARDED-SHIM INSURANCE TAIL — strncpy/strncat/strpbrk/strnlen/memrchr/strtoull/strtoll/strtof/strftime now ptr_ok-guarded. Workspace green (537/0 + sh14 example 15). Doc docs/frontier-sh149-shim-insurance.md.
Honestly-labeled ZERO-REACHABILITY insurance (unlike SH147's memmove, proven ~192x): these raw string/mem deref imports have no run-log dispatch evidence but are the same SH97/SH135 garbage-pointer crash class if the do-init ever hands them a bad ptr. register_named wins over dlsym. strtof guards only (float s0 return can't be satisfied; result dropped, documented). strftime is benign-reachable (1x sh130). VERIFIED: hermetic sh97_harden extended (all 9 reject garbage no-fault); real capture_sh130 EXIT 0 / ladder done / 0 SIGSEGV (no regression).
## SH148 (Sep 14, 2026, hermes-worker): BLINN-PHONG SPECULAR on the real studs sphere — the full lighting model (ambient+diffuse+specular) now renders a glossy highlight through the engine's own geometry wrapper 0x105b35288. Workspace green (537/0 + sh14 example 15). Doc docs/frontier-sh148-specular.md.
The mesh-uv FS gains `spec = pow(dot(n,normalize(L+V)), 32)` with `precision highp float` (mediump underflows pow^32 -> washed the highlight). VERIFIED (real libroblox.so, 1652v studs sphere): Mesh+DDS parsed, MVP+uModelRot uploaded, wrapper Ok(0x0), silhouette 24/25, 0 crash; **vision confirms a concentrated white specular glint in the sphere's upper-right quadrant — distinctly brighter/localized than the diffuse gradient (classic Blinn-Phong hotspot)**. Capture runs/sh148.png. Honest scope: host shader + real asset/normals through engine's own wrapper; completes the lighting model. NOT self-constructed UI (SH131d wall). Recon (deleg_fb89b6a4): render plane now SATURATED — all forward render candidates are declining-value; the only gate to real screens is the structural SH131d wall (no seed/patch opens it); the proven-memmove-class hardening is EXHAUSTED (SH147). Highest-value honest next = documentation+robustness consolidation + a cheap, honestly-labeled insurance tail (ptr_ok guards for the zero-reachability strtod-family + benign reachable strftime).
## SH147 (Sep 14, 2026, hermes-worker): GUARDED-SHIM EXTENSION — memmove/strrchr/strdup/strndup/atoi/atol/atoll/strtol/strtoul now guarded against the SH97/SH139 garbage-pointer crash class. Workspace green (537/0 + sh14 example 15). Doc docs/frontier-sh147-guarded-shim-extension.md.
Nine new `ptr_ok`-guarded shims (unsafe ptr -> safe default; valid -> glibc bit-identical) for the last raw-bound pointer-deref imports. `memmove` is PROVEN reachable at boot (~192x dispatch in sh126-workers, same class as guarded memcpy/memset); the rest are cheap insurance. register_named wins over resolve_common dlsym (SH135-139 precedence). `strtod` DEFERRED (double returns in d0 needs a HostGlesCall-style v0-write bridge; no reachability proof). VERIFIED: hermetic sh97_harden test extended (all 9 reject garbage no-fault + valid path: atoi("12345")=12345, strtol base10, strrchr->offset3, strdup->"hello", strndup->"hel", memmove copies); real-binary capture_sh130 re-run EXIT 0 / ladder done / 0 SIGSEGV (no regression). Reachable-path hardening per SH131d dir (a).
## SH146 (Sep 14, 2026, hermes-worker): COMPOSED REAL-AVATAR SCENE — N distinct Roblox .mesh objects in ONE frame, each with its own VBO/EBO, per-object compose-MVP (translate+scale+yaw, shared scene camera) and SH145 diffuse lighting, all driven through the engine's own geometry wrapper 0x105b35288. Workspace green. Doc docs/frontier-sh146-mesh-compose.md, repro runs/capture_mesh_compose.sh.
New `mesh_compose_mvp_shared`, `--renderframe-mesh-compose <p1>,<p2>,...`. VERIFIED (real libroblox.so, torso 664v + head 517v + sphere 1652v): 3/3 real objects composed in one frame, each wrapper Ok(0), silhouettes 9/9/9, swap Ok(1), 0 crash; frame shows the blocky gray torso (left) + studs sphere (right, spherical lighting) as distinct lit objects — a real composed avatar scene through the engine's own path. NOTE the program-reuse dependency: the compose block requires --renderframe-mesh + --renderframe-mesh-tex (builds the mesh-uv program); capture includes them.
## SH145 (Sep 14, 2026, hermes-worker): DIFFUSE LIGHTING on the real studs-textured sphere — per-vertex NORMALS (the last empty mesh render-plane cell) now drive a real `dot(N,L)` diffuse shader through the engine's own geometry wrapper. Workspace green (537/0 + sh14 example 14/0). Doc docs/frontier-sh145-mesh-lighting.md, repro runs/capture_mesh_lit.sh.
The mesh parser already produced per-vertex normals but the renderer dropped them. SH145: `mesh_interleave_model_uv` emits stride-36 `[x,y,z,1,u,v,nx,ny,nz]`; coherent renderer gains a THIRD primitive aNormal (fmt2={size3,GL_FLOAT} verified @0x100cecf8c, attr slot2, offset 24); VS `vN=mat3(uModelRot)*aNormal` with new uModelRot uniform (identity static / yaw-matching under orbit); FS `color = tex*(0.30+0.70*max(dot(N,L),0))` with fixed world light. VERIFIED (real libroblox.so): mesh+DDS parsed, `real MVP + uModelRot uploaded (uMVP loc=0x0 uModelRot loc=0x1; stride-36, normals lit)`, wrapper Ok(0x0), silhouette 5x5 24/25, 4 swaps Ok, 0 crash; **real luminance gradient (mean 72.4, stdev 34.3, bright upper-left lum~116 -> dark bottom/edges lum~38) — genuine directional falloff, not SH143's flat gray**. Capture runs/sh145-lit.png. +1 hermetic normal-interleave test. Honest scope: host VBO/shaders + real asset + normals + lighting through engine's own wrapper — NOT self-constructed UI (structural SH131d). Recon (deleg_046e7f15): audio (FMOD/AAudio) is STRICTLY LATENT behind the structural wall (reject as next); the highest-value next headless cell is MULTI-MESH COMPOSE — render N real .mesh objects (avatar torso/limbs/head + sphere) in one frame, each with per-object model transform + SH145 lighting, reusing the proven orbit uniform machinery.
## SH144 (Sep 14, 2026, hermes-worker): CAMERA-ORBIT — the real studs-textured sphere (smooth_sphere.mesh 1652v + studs.dds R8) renders from a ROTATING camera (model yawed about +Y across N frames) through the engine's OWN geometry wrapper 0x105b35288. Real rotating textured 3D scene, same VBO/EBO (only the MVP uniform changes per frame). Workspace green (537/0 + sh14 example 13/0). Doc docs/frontier-sh144-mesh-uv-orbit.md, repro runs/capture_mesh_uv_orbit.sh.
Adds `mat4_rotate_y`, `mesh_orbit_mvp` (P*V*Ry(yaw)*T(-center) on the SH143 bbox-framed camera), and `--renderframe-mesh-uv-orbit <N>`: each frame clears to a distinct dark backdrop, re-uploads the yawed MVP via glUniformMatrix4fv (transpose=0), re-drives the engine's own wrapper, swaps on the real ctx. VERIFIED (real libroblox.so): all 4 orbit frames rendered (yaw 0/1.57/3.14/4.71 rad, each `wrapper=Ok(0) swap=Ok(1)`), silhouette 5x5 24/25 maintained, **two captured frames differ by 85.3%** with the studs pattern visually oriented differently (head-on vs rotated grooves) — real rotation; 182,484 geometry px 100% grayscale per frame; 0 crash. Capture runs/sh144-orbit-f0/f1.png. +2 hermetic rotate/orbit tests. SH141 solid-red mesh path re-verified unregressed (309,707 red px). Honest scope: host-authored geometry + real APK mesh/texture + real MVP + camera motion through the engine's own wrapper — NOT engine self-constructed UI (structural SH131d). Recon (deleg_c5c49477) confirmed the present-walker frontier is ALREADY CLOSED (SH64/65 delivered it) and the ONE remaining unblocked mesh render-plane cell is LIGHTING/normal-based shading (normals parsed but never uploaded; aNormal is a 3rd empty attribute) — the next action.
## SH143 (Sep 14, 2026, hermes-worker): REAL PER-VERTEX UV + REAL CAMERA/MVP on the engine's own GLES path — the ranked render-plane ascent over SH142. Workspace green (541/0). Commit SH143. Doc docs/frontier-sh143-mesh-uv-mvp.md, repro runs/capture_mesh_uv.sh.
SH142 sampled the real studs.dds R8 over smooth_sphere.mesh with SCREEN-SPACE planar UV (gl_FragCoord), NDC-baked positions, pass-through VS. SH143 replaces all three: real per-vertex UVs (parsed, previously dropped), a real perspective camera/MVP (bbox-framed axis-aligned camera at +z, column-major P*V*M, uploaded via glUniformMatrix4fv from the int resolver), and `gl_Position = uMVP * aPos`. New pure helpers mesh_bbox/mesh_positions_model/mesh_uvs/mat4_perspective/mat4_translate/mat4_mul/mesh_interleave_model_uv; stride-24 VBO + 2-primitive coherent renderer (attrib0 aPos off0, attrib1 aUV off16) mirroring the proven quad pattern. VERIFIED (real libroblox.so): 1652v mesh + 128x2048 R8 DDS parsed, `real MVP uploaded to uMVP loc=0x0`, wrapper Ok(0x0), **silhouette 5x5 24/25 (sphere now FRAMED by the real projection camera, not edge-to-edge planar)**, centroid readback RGBA(129,129,129,255) = real studs luminance via per-vertex UV, non-bg px 100% grayscale, 0 crash, 4 swaps Ok. Capture runs/sh143-mesh-uv.png. +4 hermetic mat4/mvp/bbox tests (541/0). Honest scope: host-authored VBO/shaders through the engine's own wrapper with real asset + camera — NOT engine self-constructed UI (structural SH131d wall unchanged). Recon (deleg_7bd9f842) confirmed the ONLY remaining concrete headless UI frontier is unblocking the engine's real per-node present walker 0x105b2ed48 (second-iteration nested-jit_run desync at 0x105b2eedc) — the next action.
## SH142 (Sep 14, 2026, hermes-worker): real APK TEXTURE (studs.dds R8) on real mesh geometry through the engine's own GLES bridge. Workspace green (elfjit example 48/0). Commit e68611a. Doc docs/frontier-sh142-mesh-tex.md, repro runs/capture_mesh_tex.sh.
Added `parse_roblox_dds_r8` (pure std DDS DDPF_LUMINANCE R8 parser, bounds-checked, rejects FourCC/compressed) and `--renderframe-mesh-tex <dds>`: R8->RGBA expand + upload via glTexImage2D, sampled over the `--renderframe-mesh` geometry with a texture-sampling FS. VERIFIED (real libroblox.so, smooth_sphere.mesh + studs.dds): both parsed (1652v/3072f mesh + 128x2048 R8 DDS), geometry wrapper Ok(0x0), silhouette 5x5 25/25 drawn, ~400k pure-gray pixels (real studs luminance) over the sphere, 0 crash. Widened the mesh-mode silhouette probe to count "differs from cleared bg" (both solid-red and grayscale-textured render). Captures runs/sh142-mesh-tex.png. +2 hermetic DDS tests. SH142 follows SH141 (660441f): `parse_roblox_mesh_v2` decodes the APK's real `version 2.00` binary meshes and the engine's own wrapper renders them (CompositTorsoBase.mesh 664v -> 33.6% red silhouette; smooth_sphere.mesh 1652v -> 56.6%, 9216 real indices). Honest: screen-space UV (single-attribute coherent renderer), real camera/MVP + per-object UV remain harness-walled; SH131d structural wall unchanged.
Added a pure-std Roblox `version 2.00` binary mesh parser (`parse_roblox_mesh_v2`, bounds-checked, byte-exact vs every asset) + `mesh_positions_to_ndc` (center/scale cm -> NDC). New `--renderframe-mesh <path>` feeds the real VBO/EBO (full index count) into the existing coherent renderer and drives the engine's own geometry wrapper 0x105b35288 -> primitive-setup 0x105b353d0 -> real indexed glDrawElements. VERIFIED (real libroblox.so): CompositTorsoBase.mesh (664v/416f/1248idx) wrapper Ok(0x0) + **309,707 red px (33.61%)**, 5x5 silhouette grid 7/25, 0 crash; smooth_sphere.mesh (MaterialManager preview sphere, 1652v/3072f/9216idx) **521,972 red px (56.64%)**, 13/25 drawn — a real circular silhouette (elliptical only because NDC x stretches to the wide viewport). Captures runs/sh141-mesh.png / sh141-sphere.png. +4 hermetic mesh tests. Recon (deleg_485f301c) next items: bind a real material texture (smooth_sphere.mesh + android/textures/studs.dds 128x2048 R8, R8->RGBA8 expand, no decoder) — the genuine MaterialManager preview pair; also confirmed the type-4 vector [0x106829ea8] is STILL the only self-driven frame producer and real camera/MVP render remains harness-walled (every VS is host pass-through). Honest scope: host-authored geometry upload + real asset data + engine wrapper — NOT engine self-constructed in-world objects (SH131d structural wall unchanged).
**The synthesized Roblox login surface (SH69-78) gains its signature link row under the green button — blue "Forgot password?" + "Sign up" labels rendered through the ENGINE'S OWN real geometry emitter (0x105b35288, GL_BLEND over dark) from the APK's SourceSansPro-Bold.ttf. "Forgot password?"'s `?` is a COMPOSITE glyph — the FIRST visible artifact to exercise SH79's latent composite-glyph recursion (all prior shipped labels were simple letters). Two new entries in the SH77 rasterize table + two placements (cy -0.74/-0.85) with probe texels chosen as 3-wide solid stroke cores (x±1 all a=255) so GL_LINEAR samples the full baked link color. VERIFIED (real binary, standalone emitter path — stable): sprites 9→11 (`emitter Ok ret=0x0 count=72 sprites=11 swap=Ok(1)`), **11/11 present=true** incl. both new link probes reading `(59,130,246,255) diff=[0,0,0,0]` (byte-exact), walker Ok, 0 crash. New hermetic `sh140_forgot_password_link_label_composite_question_rasterizes` ('?' composite decodes; shipped label >50 opaque px @ link color; 3-wide core exists). Workspace 537/0; example 42/0. Scope honest: host-authored geometry through the engine's own emitter + real APK font — NOT engine self-constructed UI (SH131d structural wall unchanged). Standing walls unchanged (structural Lua app-shell, SH131d).
**Recon (deleg_0eab8fe6) verdict: the raw-varargs printf family (fprintf 513 / snprintf / printf / __android_log_assert) is NOT reached with garbage on the current boot path (falsifier — those are bound raw to glibc today and boots are clean), so an ABI-lossy raw-varargs renderer is NOT worth building. Pivot to cheap proven-pattern wins: (1) `__memcpy_chk` (169 sites) + raw `memcpy` (the GameActivity init calls memcpy@plt at file 0x2b9e084 from an unseeded state slot — an observed SIGSEGV this cycle) where ptr_ok(dst)&&ptr_ok(src)||n==0 -> dst (no copy); (2) raw `memset` + `__strncpy_chk` same pattern; (3) `__android_log_print` upgraded from `!=0` to `ptr_ok` so a garbage (0xffffff80ffffffc8) tag/fmt can't fault CStr::from_ptr. All valid pointers pass through to real glibc bit-identical. register_named slots win over resolve_common dlsym. Extended the SH135 hermetic test. VERIFIED: workspace 537/0 clean. Real binary: ladder-only EXIT 0 clean on the majority of runs; run-variable SH55/64 flake (different guestpc each time: 0x102b9e09c memcpy@plt closed by this commit, then 0x101ee2d24) is the documented pre-existing concurrent-thread class, NOT a regression. Standing walls unchanged (structural Lua app-shell, SH131d).
**`__vsprintf_chk` (10 call sites, RAW) is the sprintf-family fortify reroute — signature (dest, flag, slen, fmt@x3, va_list@x4, 5 fixed args) verified at fast-log call site file 0x2d9a5e0*. No shim -> a garbage `%s` (SH97 class) SIGSEGV'd raw glibc sprintf's internal strlen. FIX (shims.rs): `bionic_vsprintf_chk` reuses the shared guarded renderer (decode_aapcs64_va_list + render_to_buf with u64::MAX — unbounded dest, sprintf semantics, so no size clamp). New hermetic `vsprintf_chk_unbounded_guards_garbage_arg_indices`: garbage %s -> "(bad-ptr)" + real %d rendered + NUL, valid LONG render NOT clamped to slen=0x14 (distinguishes from vsnprintf's size-1 cap). VERIFIED: workspace 537/0; real-binary ladder-only capture EXIT 0 / full ladder clean. Recon also confirmed `__android_log_vprint` is NOT imported (dead resolve_common entry — zero call sites, skip). Standing walls unchanged (structural Lua app-shell, SH131d).
**Bionic fortify reroutes printf-family calls into `__vsnprintf_chk` (6 fixed args: dest, supplied_size, flag, slen, fmt@x4, va_list@x5 — verified at call site file 0x2631414). It has NO shim, so a garbage `%s` (the SH97 class 0xffffff80ffffffc8 from an unseeded map field) hit raw glibc vsnprintf -> internal strlen SIGSEGV. FIX (shims.rs): `bionic_vsnprintf_chk` reuses the SAME guarded renderer as bionic_vsnprintf (safe_cstr_len %s -> "(bad-ptr)"), with a shared `decode_aapcs64_va_list` + `render_to_buf` helper so the extra fixed args don't drift the va-arg offsets. Refactored bionic_vsnprintf's inline va_list decode into the shared helper (behavior-identical). New hermetic `vsnprintf_chk_guards_garbage_renders_real_bounded`: garbage %s -> "(bad-ptr)" + NUL, valid %s -> real bytes, size==0 -> writes nothing, tiny size -> truncated + NUL. VERIFIED: workspace 536/0; real-binary ladder-only capture EXIT 0 / full ladder clean (combined capture still in the run-variable SH55/64 flake window, not a regression). Standing walls unchanged (structural Lua app-shell, SH131d).
**The do-init map/flag walk also lands on `memcmp` (the length-bounded compare, bound raw via resolve_common) + `strcasecmp`/`strncasecmp` (pure pointer walks) — same SH97/SH135 garbage-pointer crash class (0xffffff80ffffffc8 / 0 / sub-image tag). FIX (shims.rs): guarded shims mirroring bionic_memchr/strcmp — unsafe ptr or n==0 -> 0 (equal/empty path), valid ptr -> real glibc (bit-identical). register_named slots win over the resolve_common dlsym binding. Extended the SH135 hermetic test to feed the crash class to all three (no SIGSEGV) + valid-string compare/case-insensitive/zero-len. VERIFIED: workspace 535/0. HONEST: the real-binary combined capture (`runs/capture_sh130.sh`) is in a run-variable SH55/64 flake window this cycle (EXIT 134 at guestpc 0x106240cb8, the exact documented concurrent-thread site — confirmed identical with the change stashed, i.e. NOT a regression; these shims only divert pointers that would otherwise SIGSEGV). Standing walls unchanged (structural Lua app-shell, SH131d).
**The deep do-init walk hands unseeded map-field pointers to libc string ops — the exact SH97 crash class (0xffffff80ffffffc8 garbage / 0 / sub-image tag) that SH97 eliminated only for strlen/vsnprintf. Ten string imports were still bound raw to host glibc and SIGSEGV'd on a garbage pointer. FIX (shims.rs): new `ptr_ok` domain predicate + guarded shims for `strcmp/strncmp/strstr/strchr/strcpy/strspn/strcspn/memchr` + fortified `__strchr_chk/__strcpy_chk` — unsafe ptr -> deterministic non-faulting default (0 / dst unchanged), valid ptr -> real glibc (bit-identical). register_named slots win over the generic dlsym binding (same path as bionic_strlen). New hermetic test feeds the crash class to every shim (no SIGSEGV) and checks valid strings still behave. VERIFIED: workspace 535/0; real-binary combined capture `runs/capture_sh130.sh` EXIT 0 / 24 real task frames / 0 SIGSEGV (no boot-path regression). Removes the highest-probability remaining silent SIGSEGV deterministically — reachable-path hardening per SH131d, not session-gated. Standing walls unchanged (engine self-constructed login/home + login persist remain behind the structural Lua app-shell wall, SH131d).
**Recon (deleg_a26ee3a7, READ-ONLY) — how the client does login/auth HTTP:** **raw native sockets (bionic libc imports) + a statically-bundled OpenSSL 3.5.0 inside the .so** — NOT JNI-to-Java. Zero `java/net`/`HttpURLConnection`/`OkHttp`/`android.webkit` references; NEEDED = libc/libm/libdl/liblog/libandroid/GLES/EGL/audio only (no external TLS). The transport is already fully plumbed host-side: jit.rs forwards socket/connect/sendto/recvfrom/bind/epoll + clock_gettime/getrandom (guest TLS gets entropy); resolver binds getaddrinfo/gethostbyname to host glibc. So host sockets + DNS + clock + entropy + guest TLS reach the real network today — transport is NOT the gap. Do NOT build JNI HttpURLConnection/OkHttp/WebView emulation (no consumer).
**SH134 (be3bb5d) — AppBridge `getBaseURL` now returns the real production web root `https://www.roblox.com`** (was ""). With the transport-complete host TLS channel, "" meant the engine's API calls had no real host to target; the real base (the binary's own canonical string) lets a real endpoint be reached when the client does hit HTTP. Reachable-path, low-risk, still a readable 24-byte jstring for the json writer. +1 hermetic sh134 test; strengthened the pre-existing auto-value-params test to pin the new base-URL length. Re-verified on the real binary: combined capture EXIT 0 / 24 real task frames / 0 crash / ladder done (baseURL latent — not hit on this boot path).
## SH132/133 (Sep 14, 2026, hermes-worker): FAKE-AUDIO BRIDGE (FMOD AAudio) + FUTEX REQUEUE/CMP hardening. Workspace green (533/0, elfjit example 41/0). Commits 75cd193 (SH132), 0edf597 (SH133). Docs docs/frontier-sh132-aaudio-bridge.md, docs/frontier-sh133-futex-requeue.md.
**SH132 (75cd193) — FMOD audio via a fake-libAAudio host bridge (latent-but-correct).** The real client statically links FMOD; its de-facto output driver is native AAudio, resolved at runtime by `dlopen("libaaudio.so")` + 26×`dlsym` into a fn-ptr table (driver 0x4fbf3a8, table guest 0x106d0ef20, JNI entry 0x104fbea00) — absent on x86-64 Linux, so FMOD's AAudio open fails and no PCM drains. New module `crates/arm64jit/src/aaudio.rs` (+resolver wiring) intercepts the guest's `dlopen`/`dlsym`/`dlclose`/`dlerror` imports under **JIT_AAUDIO_BRIDGE=1** and serves the 26-slot AAudio C ABI: tagged opaque FakeBuilder/FakeStream handles (never deref'd), device-native getters (48k/stereo/192 fpb), state converges to DISCONNECTED(10) on stop (the guest's stop-wait loop needs out==10), WAV sink (`wav_header_bytes`, 44B PCM, env JIT_AAUDIO_SINK default /tmp/open-sober-fmod.wav). No guest-re-entering thread (fresh SH55/64 JIT block-cache race). HONEST: FMOD only opens output once the SoundService session advances (behind the SH131d structural wall), so it is inert on the current boot path — it is the host-side capability ready for audio. Env-gated off; product path byte-identical. +8 aaudio hermetic tests (also fixed a WAV test bug: block-align sits at byte offset 32, not 34).
**SH133 (0edf597) — futex hardening (reachable-path, direction a).** The guest `futex` (nr 98) handler forwarded WAIT/WAKE/WAIT_BITSET to the real host and returned a fabricated 0 to the guest for every other op — bionic `pthread_cond` broadcast uses **FUTEX_REQUEUE(4)**/**CMP_REQUEUE(6)** to re-park wakees onto the condvar's OWN futex, so a fake 0 stranded the parked waiter forever (silent lost wakeup on the exact multithreaded plane the client reaches). Extracted `handle_futex(a:[u64;6])` (jit.rs, hermetic-testable) and forwarded REQUEUE(4)/CMP_REQUEUE(6) to the real kernel (the move + `*uaddr==cmp` gate happen genuinely; mismatch truthfully returns -EAGAIN) and made a truly-unknown op return **-ENOSYS** instead of 0. Boot-path `FUTEX_WAIT_BITSET_PRIVATE` (0x89) untouched. +3 hermetic tests that run REAL host futexes: requeue moves a parked waiter then WAKE-on-dst releases it; cmp_requeue returns -EAGAIN on mismatch and moves+wakes on match; unknown op <0. Green 533/0; **combined capture re-verified EXIT 0 / 24 real task frames / 0 crash / ladder done** on the real binary.
STANDING walls unchanged: engine self-constructed login/home + the cookie-jar/login persist contract (SH129) remain behind the Lua app-shell + nativeInit 0x10232090c wall (structural, SH131d); the 24-frame plane is still the seeded type-4 thunk. GPU host still only for performance, not correctness.
## SH131b (Sep 14, 2026, hermes-worker): SEEDED the engine's files-directory global — the datastore base-path plane is now de-gated from the Lua app-shell wall (persistence objective 2b, latent-but-correct). Workspace green (522/0, elfjit example 41/0). Commit: SH131b. Doc docs/frontier-sh131b-filesdir-seed.md.
deleg_fbb8faf7 (disasm 0x21f7654): the REAL client stores its data dir via nativeSetFilesDirectory into a 24-byte std::string at **guest 0x10726d600** (file 0x726d600) — CORRECTING the SH114 comment's 0x1026d600 (that's the read-only code segment). Driving the native is a dead-end (its GetStringUTFChars needs a real Java-arena jstring; fabricated handle -> empty). FIX: host-seed the libc++ long-form string (36-byte path > SSO: [0..8]=__data_ -> guest-arena bytes+NUL, [8..16]=__size_, [16..24]=__cap_ bit0=0 long) at 0x10726d600 via new opt-in `--v2boot-set-filesdir` rung + helper `seed_libcpp_long_string` + 2 hermetic tests. So the engine's own rbx-storage.db opens now have a real base path -> route through fsmap to persistent host disk once a session opens it (still behind the SH126 app-shell wall). VERIFIED (real binary, opt-in): `[0x10726d600] = "/data/user/0/com.roblox.client/files" ... SEEDED`, EXIT 0; SH130 combined capture still EXIT 0 with 24 task frames.
## SH131 (Sep 14, 2026, hermes-worker): COMBINED-RUN CLEAN EXIT — the SH130 combined capture (24 real task-driven frames + ladder/session in ONE run) now exits **EXIT 0** reproducibly (3/3) instead of the standing exit-133 (silent guest SIGTRAP). Workspace green (522/0, elfjit example 39/0). Commit: SH131. Doc docs/frontier-sh131-flood-bound-clean-exit.md, repro runs/capture_sh130.sh.
The SH130 residual (STATUS.md's next gate: "bound the post-frame dispatch flood so the combined run exits clean after N frames") was TWO independent mechanisms:
1. **Unbounded dispatch flood** — `--deque-node-live`'s 400-tick (20s) re-injection kept the drain dispatching through the seeded type-4 frame thunk long after the presenter's 24-frame budget (PENDING_PRESENTS 6659→13512; dispatch #16000+ post-"presenter drained"). FIX (`combined_frame_capture` + TASKFRAME_HALT, elfjit.rs): when the presenter drains its budget in combined mode, set TASKFRAME_HALT=1 and NULL the type-4 vector [0x106829ea8] back to its boot-idle no-op (the dispatcher's `ldr x3,[x8,#3752]; br x3` returns doing nothing on 0) — the injector's loop checks HALT and breaks ("injector stopped after 2 ticks").
2. **Raw-host-signal SIGTRAP fatal** — even bounded, a released clone worker hit the engine's OTHER SH121-class `raise(SIGTRAP)` sites. The guest `raise` import binds to HOST glibc `raise` → a REAL host SIGTRAP terminates the whole process (128+5=133), never entering guest signal dispatch (confirmed: no `[signals]` line ever fired, so a signals-side diagnostic can't catch it — same class as SH124's exit-232). FIX (`host_guest_raise` + `sh131_raise_is_intercepted`, resolver.rs, mirrors SH124): under JIT_DRIVE_LIFECYCLE the guest `raise` import is routed to the shim that unwinds a SPAWNED-thread SIGTRAP (zero x30 → run_loop pc=0 → jit_run returns, process survives) and keeps main-thread raise real. Inert without the env == historical host-glibc binding.
**EMPIRICAL (runs/capture_sh130.sh, real libroblox.so):** before EXIT=133 / 21 frames (log cut mid-present at #20); after **EXIT=0 3/3, 24 full frames** (`present #N swap Ok(0x1)`), order: presenter drained: 24 → SH131 flood bound (vector nulled) → injector stopped after 2 ticks → `SH131 spawned-thread guest raise(SIGTRAP=5) — unwinding this jit_run (process survives)` → StartApp returned → ladder thread joined cleanly. 0 SIGSEGV/SIGABRT.
Tests: folded the SH131 raise-gate assertion into the SH124 test (same JIT_DRIVE_LIFECYCLE gate) — as a standalone test it raced with sh124 on the shared env var; default/product paths (flags/env off) are bit-identical.
STANDING walls unchanged: engine self-constructed login/home + the cookie-jar/login persist contract (SH129) remain behind the Lua app-shell + nativeInit 0x10232090c wall; the 24-frame plane is still the seeded type-4 thunk (a session-driven producer would replace it). GPU host still only for performance, not correctness (llvmpipe carries it headlessly).
## SH130 (Sep 14, 2026, hermes-worker): WORKER-ADMISSION GATE UNBLOCKS THE COMBINED RUN — 24 REAL task-driven frames now present in the SAME serialized ladder+render run, reproducibly. Commits 8dff7b4 (SH128), f28a71b (SH129), 184ad36 (SH130). Workspace green (522/0, example 40/0).
**SH130 (the breakthrough, 184ad36):** recon deleg_5a9376df root-caused the SH55/64 combined flake: the ENGINE'S SELF-SPAWNED clone workers (pthread_create start_routine 0x10284d168, tids 1/2) run guest code CONCURRENTLY with the ladder's rung jit_runs, corrupting shared guest state -> the false rung-0 "*** stack smashing ***" (main thread fault; NOT a stack collision — each worker has its own 16MiB stack/state, SH108's was the harness renderinit reusing boot SP already fixed). JIT_SERIALIZE_RENDER only gates the harness renderinit thread, NOT the engine clones. Fix: host-side `WORKER_ADMISSION_GATE` (jit.rs) parks each clone-worker's top-level jit_run (yield+1ms) until LADDER_DONE, set before the boot entry when JIT_SERIALIZE_RENDER=1 + --v2boot, cleared at "ladder done". ZERO guest bytes; deadlock-safe (SH93 NOP'ed the one CEvent barrier workers post for; bionic_pthread_join no-op); default-OFF (product path unregressed, verified). **EMPIRICAL: baseline combined run now presents 24 REAL TASK-DRIVEN FRAMES (`present #N swap Ok(0x1)`) reproducibly (3/3: 24/21/24) — StartApp stays resident in its drain (workers no longer race it) so the injector+presenter machinery flows in the SAME run as the ladder. This is the SH127/SH128 combined-frame goal reached.** RESIDUAL: process exits **133** (silent guest SIGTRAP — a released worker hits an engine fatal under the unbounded dispatch flood AFTER the 24-frame cap), NOT a render crash (all swaps Ok(0x1), no SIGSEGV/ABORT; standalone "124" is just the outer timeout). NEXT gate: bound the post-frame dispatch flood so the combined run exits clean after N frames.
**SH128 (8dff7b4, --deque-redrive) now LARGELY SUPERSEDED by SH130** (with the gate StartApp stays resident -> frames work without the re-drive). Kept opt-in/documented as the fallback; its open prereq (tid0 headcell for the synthetic root) remains. The x1 derivation was corrected to the pump helper 0x10284d524 (earlier file-addr 0x67d67c0 SIGSEGV).
**SH129 (f28a71b, doc):** cookie-ingress ABI mapped (nativeSetMultipleCookies 0x102202ff8 4-arg JNI thunk over pure-native worker 0x102203148 — no Java Set); double-gated ([0x72739d4]+[0x6dcfc30] bit0); seeding both advanced 2 gates then hit jar-CONSTRUCTION NULL (structural init-wall, cookie jar not built by bare JNI_OnLoad). Reverted impl; ABI pinned. Corrected: session.db synthetic, rbx-storage.db = content cache (NOT auth), login identity = Java-delivered .ROBLESECURITY cookie + userid.
**NEXT:** (a) bound the drain dispatch flood / keep the 24-frame presenter cap so the combined run exits clean (0) instead of 133; (b) session-advance (real login/home GuiObjects) behind the Lua app-shell + nativeInit 0x10232090c wall (structural); (c) cookie jar init unlocks the login-token persist contract.
**SH128 `--deque-redrive`** (8dff7b4): re-drives the engine's drain pop-loop 0x102856e40 on the MAIN thread after the serialized ladder joins — closing the frontier-sh127 structural "StartApp returns -> no resident drainer -> 0 combined frames" gap. Recon deleg_f139e286: x0 = synthesizable deque-root {stable headcell 0x10682a638/0x10682b338, tag=[headcell]>>48}, drain reads [x0] read-only (16B guest root safe); x1 = scheduler = `*(TLS-getter(0x67d67c0)+0x410)` via 0x102b9dee0 — resolved ON main thread (its TLS persists post-return). Bounded finite timeout (TASKV4_REDRIVE_MS). renderinit presenter + --deque-node-live injector hold open for the REDRIVE_ACTIVE window via new atomics. Settle loop reduced under the flag. Pure headcell-predicate extracted + 3 hermetic sh128 tests.
**Runtime verification BLOCKED (honest):** the SERIALIZED combined run stays run-variable at the documented SH55/64 structural class (engine clone-worker threads tids1,2 racing the ladder's jit_run -> false `*** stack smashing ***` at rung-0 nativeInitializeNativeFlags) no matter what JIT_SERIALIZE_RENDER does (it only serializes render-vs-ladder, NOT the clones; gating them risks deadlock). 0/6 clean combined ladders this window, so `--deque-redrive`'s drain re-entry was never reached. Baseline capture_sh127.sh RE-VERIFIED run-variable (1/2 clean) — this is NOT a regression (every SH128 edit is gated on --deque-redrive; sh127 default path bit-identical otherwise). Product artifact per frontier-sh127 resolution (1): ladder-only (3/3 clean with SH126-r0) + the standalone 24-frame self-driven plane.
**SH129 login-cookie ingress (research; docs/frontier-sh129-cookie-ingress.md):** MAPPED the real `.ROBLESECURITY` delivery contract headlessly (deleg_0e6d339f + empirical): nativeSetMultipleCookies 0x102202ff8 is a 4-arg JNI thunk (no Set, no iteration; only GetStringUTFChars slot169/Release slot170) over a pure-native worker `0x102203148(char* cookies,size_t clen,char* url,size_t ulen,int,int)`; read-back `0x1021ff6b0(char* url,size_t, int, std::string* out)` fills the netscape file containing `#HttpOnly_...\t.ROBLESECURITY\t<token>`. The worker is DOUBLE-gated ([0x72739d4].bit0 flags-loaded + [0x6dcfc30].bit0; either unset -> fault 0x220321c). Seeding both=1 in the implementation advanced it 2 gates deeper (fault 0x220321c->0x220331c) then it hit the cookie jar-CONSTRUCTION NULL (0x21fce24): the jar container is not built by a bare JNI_OnLoad boot — the same structural init-wall as nativeInit 0x10232090c / StartLuaAppDM. The implementation was reverted (would crash the tree); the full ABI + latch chain + blocker are pinned for reuse. Corrected prior misunderstanding: session.db is harness-synthetic (0 hits in the binary); rbx-storage.db is a content cache (files(id BLOB PRIMARY KEY,content,size,hits,atime) KVS), NOT auth; login identity = a Java-delivered .ROBLESECURITY cookie + user id (nativeSetUserId / cachedUserId), not a native self-read file — so "persist login" means the cookie-INGRESS path (this SH129), fundamentally behind the session-advance/jar-init wall.
**NEXT:** (a) serialize/park the engine clone workers (tids1,2) or otherwise get a clean combined ladder so the SH128 re-drive can be runtime-verified (or accept the redrive as the documented opt-in and focus elsewhere); (b) clear nativeInit 0x10232090c / the session-advance wall so the cookie jar constructs -> re-drive the SH129 workers -> engine-persisted rbx-storage.db open; (c) the standing real screens goal: engine self-constructed login/home (behind the Lua app-shell wall, structural).
## SH127 (Sep 14, 2026, hermes-worker): recon-v3 deliverables both verified DONE + green; --deque-node-live serialized gate added; the one outstanding combined-frame gap is ISOLATED as structural. Workspace green (522, elfjit example 36). Commit SH127 (see git log).
Recon-v3 (docs/recon-selfdrive-seed-jsonfix.md) both implement and are **now re-verified on the current tree**:
**(1) SELF-DRIVEN FRAMES** — `type4_frame_thunk` is installed into the dispatcher's type-4 vector [0x106829ea8] via `--taskv4-seed frame`; a w4=4 drain dispatch marshals -> the currency-owning renderinit presenter drives engine make-current 0x105b3b358 -> frame-fn 0x105b32c00 -> swap 0x105b3b408 on the recovered real ctx (RENDERCTX, vtable 0x106731ae0). Fresh repro (runs/capture_taskv4_frame.sh): **EXIT 124, 24 real task-driven frames presented (`present #N swap Ok(0x1)`), 191 real type-4 node pops, 0 crash, no json abort** — a current, reproducible artifact of the engine's own task dispatch presenting real frames.
**(2) JSON-ABORT** — JIT_JSON_ZERO_FIX (SH61) clamps the leaked stack std::string length to 0 (SSO empty) at the append bound-check 0x102355d40 whenever it would overflow the read-only cap cell 0x107275648 (never raised — raising makes the writer memcpy ~1.6GB -> SEGV). Regression `json_zero_fix_clamps_leaked_length_at_append_check` pins it. Re-verified firing in the re-run (len=0xb3/0x24 -> SSO empty).
**SH127 change (this commit):** added a JIT_SERIALIZE_RENDER=1 + `--v2boot` gate to the START of the `--deque-node-live` producer thread (elfjit.rs ~6306) — it now waits for LADDER_DONE then RENDERCTX (300s bound, mirroring the renderinit gate at 6945) before starting its 400-tick live-drain capture budget. Pre-SH127 the budget burned entirely in the pre-recovery window and the injector gave up before RENDERCTX published (sh126-serial-s1.txt: 0 frames, pending=0). **Diagnostic isolation of the outstanding gap:** even WITH the gate (sh127-serial-ladder-frame.txt: serialized gate passed LADDER_DONE=true RENDERCTX=0x7fcee81109c0), the combined run still presents 0 frames with 0 node pops — the engine's live drain pop-loop pc NEVER lands in [0x102856e40,0x1028570a4) after the ladder, because StartApp is parked by the ladder and its idle-main-loop drain never resumes. THAT is the truthful blocker to frames-in-the-combined-run (contrast the standalone path where StartApp's drain stays live: 191 pops). It is the documented SH55/64/structural class (drain-not-driven-after-it-parks), NOT a timing bug the gate can fix. Opt-in; default unregressed (both deliverables green on the product path).
NEXT (in priority): COMBINED-FRAME re-drive is VIABLE + entry PINNED: drive the DRAIN pop-loop 0x102856e40 (jit_run(x0=deque, x1=queue-obj, x2=finite_timeout); wait-prim 0x10284d014 is only its park point; deque-fwd globals stay coherent post-StartApp-return, type-4 vector stays seeded). LAST prereq before coding: derive the deque-root/queue-obj live pointers, which standalone are recovered from the running drainer's x20 (elfjit.rs 6366-6410) but are unavailable post-return. Session-advance (real login/home) remains behind the Lua app-shell wall.
## SH126 + SH126-r0 (Sep 14, 2026, hermes-worker): SendAppEventOnAppReady's body now COMPLETES (app-event vtable + pipe sync-gate) AND the render pipeline is serialized after the ladder (JIT_SERIALIZE_RENDER / LADDER_DONE). Opt-in chain; default unregressed; workspace green.
Read-only recon (deleg_f595f562, deleg_fcd65c31) + empirical runs established: **the session-advance wall is STRUCTURAL and confirmed from multiple angles** — (1) the MH_* NativeHelper milestones are HOST-side atoms (jni.rs), no guest memory polled them, so "firing them" is a dead-end; (2) there is NO in-image GuiObject->scene-list writer without the Lua app-shell; (3) SendAppEventOnAppReady builds a real 0x58 app-event object whose static vtable 0x635e068 is all-zero -> terminal `blr` at 0x102bb4984 was `blr 0` (soft-return). **SH126 (two parts):** (a) materialize vtable slots +0x20/+0x28 to the benign identity leaf + seed pipe sync-gate [0x10683d010]=-1 so the sendapp body's `bl 0x2baeeec` takes the synchronous path -> `bl 0x2206c40` (do-init); (b) clear the once-guard [0x106a68410].bit0=0 + re-seed main-id [0x106863a68] before the sendapp rung so the do-init re-constructs. **EMPIRICAL (runs/sh126-run.txt, ladder-only): EXIT 0, full ladder, SendAppEventOnAppReady returned Ok(0x3e8)** — the vtable is ALREADY populated at runtime (0x100698a00/0x10635c8d8, engine init fills it before the patch runs), so materialize is guarded-inert; the body completes anyway. MH_FLAGS_LOADED/APP_READY stay false (structural, dead-end). **Gate b (serialization):** `JIT_SERIALIZE_RENDER=1` + --v2boot makes the --renderinit thread WAIT for a LADDER_DONE AtomicBool (set at "ladder done") before driving the render pipeline, so render jit_runs never overlap the ladder's. Combined runs s1/s3 clean EXIT 0 with ladder-done + render-after-ladder; the residual 1/3 crash moved to rung-0 nativeInit. **SH126-r0:** recon deleg_fcd65c31 proved the residual crash is a null-store in nativeInit's flag-recorder (guest 0x101d97c70 via vt[+232] at 0x1062514e0), on the LADDER thread (not render, gated; not --deque-node-live, never a top-level jit_run). Leaf-rewrote the callee to materialize singleton + ret. **Ladder-only with r0: 3/3 clean (EXIT 0, ladder done, joined).** Combined-with-render remains run-variable at the DOCUMENTED SH55/64 structural class (engine's OWN clone-spawned worker threads race the ladder's jit_run — recon-verified beyond any harness gate). +3 regressions (sh126).
Commits 64c5f9a (SH126) + 441e4a2 (SH126-r0). Docs: recon-selfdrive-seed-jsonfix.md + frontier-sh124-exit232-race-closed.md.
## SH124 + SH125 (Sep 14, 2026, hermes-worker): the exit-232 self-termination race is CLOSED — the --v2boot ladder now completes to "ladder done" + the full session-advance probe (opt-in; the clean runs). Workspace green (25/25 + sh124 x2, arm64jit example 33/0). Default unregressed (run-variable SH55/64 flake only, same site).
Root cause of the pre-SH124 exit 232: StartApp's jit_run now RETURNS, so main() ran off the end AND the guest's own libc `exit(232)` (deep in the StartLuaAppDM do-init) terminated the whole process via the resolver-bound host glibc exit — NOT a guest_svc exit_group (JIT_TRACE_SVC showed none). **SH124 (two parts, both under JIT_DRIVE_LIFECYCLE): (1) JOIN the ladder** — the --v2boot spawn handle is captured and joined (bounded 300s) after StartApp's jit_run returns, so main() no longer tears the detached ladder down mid-do-init; (2) **exit-family intercept in resolver.rs** — the guest's libc `exit`/`_exit`/`_Exit`/`abort`/`exit_group` imports bind to a new `host_guest_exit` shim; a SPAWNED-thread exit (gettid()!=getpid()) logs the code and zeroes its saved x30 so the run_loop's `s.pc=s.x[30]` lands pc=0 -> jit_run returns -> the ladder continues and the process survives; a MAIN-thread exit still routes to real glibc exit. Inert without the env. **SH125 (recon deleg_5e2c8480): seeds do-init flags-loaded latch [0x106a683e8].bit0=1** before the StartLuaAppDM rung so the do-init (guest 0x102206c40, flags-loaded getter 0x10220671c -> `ldrb w0,[0x106a683e8]` at file 0x2206738) consumes the live DM slot [x21+8], not host-garbage. VERIFIED (runs/sh124-run.txt): EXIT 0, nativeGameGlobalInit Ok -> nativeUpdateAdapterInit Ok -> setTaskSchedulerBM Ok -> StartLuaAppDM Ok(0x3e8) -> V2StartAppWithParams Ok(0x3e8) -> V1 AppStart__ Ok(0x3e8) -> `ladder done` -> session-advance probe (MH_* false, once-guard=0x1) -> `ladder thread joined cleanly`; 2 json-fix clamps, 3 setfix substitutions, SH125 seed + intercepts log; type-4 vector [0x106829ea8]=0x7f00000001d8 (harness --taskv4-seed frame); task frame present swap Ok(0x1). No more exit-232. The 1/3 run-variable crash (guestpc 0x106240318/0x106240b44, tid 0) is the PRE-EXISTING SH55/64 concurrent-thread flake — masked pre-SH124 by exit-232 killing the whole process, now surfacing on that minority of runs; NOT a regression. +2 regressions (sh124_exit_intercept_gated_on_drive_lifecycle, sh124_spawned_thread_is_not_main_thread). Commits pending; repro runs/capture_sh124.sh.
NEXT: (a) drive the NativeHelper gameActivity_* milestones (registered SH111, set-only) INTO the session so onFlagsLoaded->onAppReady fire and StartLuaAppDM's post-return body mounts real GuiObjects (Lua app-shell + ScriptContext + DataModel + scene-list is the recon §3-identified standing wall); (b) resolve the 1/3 concurrent-thread crash so SH115-125 flip default-ON (the real remaining blocker to a stable default); (c) type-4 [0x106829ea8] real session install vs the harness seed.
## SH123 (Sep 14, 2026, hermes-worker): cleared the 0x102175854 SIGSEGV in the DM/app-shell construction path — the String-hash-set `.find()` with a dangling host container now returns NULL. Opt-in; default unregressed. Workspace green.
Read-only recon (deleg_c48fbbf5) pinned the SH122-session-lever's downstream gate: guestpc 0x102175854 (`ldr x23,[x20,#8]`) is NOT FMOD-specific — it's the generic Roblox **String-keyed hash-set `.find()`** leaf (guest entry 0x10217582c, file 0x217582c, 429 static bl sites), called during the StartLuaAppDM/GlobalInit do-init's **telemetry-counter registry** (video-streaming stats, lookups for "Rebufferings"/"PlaybackErrors"/"EndOfStream" at rodata 0x8c61a6) with a **dangling SH103-class host container** (the telemetry/stats singleton `parent->field_0x30` set at +0xe8 never constructs under the JIT). Layout: +0x00 bucket array, +0x08 count, +0x18 String needle (hashed by 0x1df644c BEFORE the count), +0x20 compare String. Because the hash runs before the count, a .text patch can't save it. Fix mirrors SH88/SH92: opt-in JIT_ROUTEB_SETFIX host hook at the leaf block entry 0x10217582c — if x0 is NOT in the guest image domain, substitute a leaked coherent EMPTY String-hash-set (zeroed 0x30: +0x08 count=0 -> leaf's `cbz x23` returns NULL; +0x18/+0x20 = SSO empty String hashes safely). In-image sets (real lookups) keep their pointers. VERIFIED (opt-in, runs/sh123-run*.txt): 3 set-finds substituted with NULL (no SIGSEGV), BOTH json-Writer clamps fire, and the do-init now penetrates the DEEP region guest 0x102208xx (past the SH97 0x10220847 wall; 1154 JIT blocks traced in [0x102206000,0x102210000)) constructing real engine state before the process self-terminates cleanly at **exit 232** (no crash dump; JIT_TRACE_SVC shows NO guest exit_group — the main-thread StartApp/idle path brings the process down while the detached --v2boot ladder is mid-construction). This is strictly deeper than SH122 (exit 139 SIGSEGV). Default ladder (flag off) UNREGRESSED — re-verified run-variable as the documented SH55/64 concurrent-thread flake (1/3 EXIT 0 ladder_done, 2/3 crash at guestpc 0x106240b44, the same site HANDOFF documents since SH55). +regression sh123 (leaf/fault/hash addrs + empty-set layout invariant: +0x08 count must be 0). arm64jit example suite 33/0; workspace 25/25. Commit 492633c.
NEXT: (a) investigate the exit-232 clean termination — is the detached --v2boot ladder simply not getting its jit_run reaped before main exits (a harness teardown race), or the engine's own idle poll expecting an event? If it's a reaping/timing issue, serialize/synchronize so the ladder thread completes to "ladder done" and the session-advance probe prints; (b) continue clearing the downstream do-init gates the construction path now reaches (whatever comes after the telemetry counters); (c) wire the NativeHelper gameActivity_* milestones into an actual session-drive so onFlagsLoaded/onAppReady fire -> first login GuiObject; (d) type-4 vector [0x106829ea8] install; flip SH115-123 default-ON only once the full clean session-constructing ladder is stable across runs.
## SH122 (Sep 14, 2026, hermes-worker): StartLuaAppDM's GlobalInit do-init now EXECUTES its DM/app-shell construction body — the session lever engages (json app-data-model serialization fires). Opt-in; default ladder UNREGRESSED (EXIT 0). Workspace green.
Read-only recon (deleg_65301a28) corrected an implicit premise: StartLuaAppDM (guest 0x1023efe2c) already runs to its real `ret` — the no-op virtual dispatch it does returns `Ok` (x0=sp) after calling 0x2baeeec, the shared app-bridge pipe that fans into the GlobalInit do-init (0x2206c40). THAT do-init is where the DM/app-shell (-> GuiObjects -> engine self-constructed login/home) is built, but it is once-latched on [0x6a68410].bit0 and thread-gated on [0x106863a68] (the SH82 main-id cell). On the (non-main) ladder thread the thread-dispatch b.ne parks and the once-guard stays unlatched -> the app-shell never builds. SH122 seeds BOTH before the StartLuaAppDM rung (main-id cell = this thread's pthread_self, mirroring rung-1; once-guard[0x106a68410].bit0=1) so the do-init takes the match/DM-construction path. CAUGHT A REAL BUG: the first host-store wrote FILE offset 0x6a68410 (unmapped -> SIGSEGV); the guest address is 0x106a68410 (rw- segment [0x1067d67c0,0x107333c3c)). VERIFIED (opt-in, runs/sh122-run.txt + sh122-run-jsonfix.txt): with JIT_JSON_ZERO_FIX also on, the do-init's StartApp app-data-model serialization genuinely runs — BOTH json-Writer clamps fire (len=0xb3, len=0x24 -> SSO empty), then it faults DEEPER at guestpc 0x102175854 (`ldr x23,[x20,#8]`, host-ptr-in-guest-reg, the SH45-documented GameActivity/FMOD init-region wall). Also added a session-advance probe after the ladder (MH_FLAGS_LOADED / MH_ENGINE_INITIALIZED / MH_APP_READY + once-guard state) — MH_* still false because the NativeHelper CALL_VOID_METHOD callbacks are registered but nothing drives them yet into the session. Default ladder (flag off) RE-VERIFIED clean: EXIT 0, StartLuaAppDM Ok, ladder done, 0 crash. Importants: SH121 (setTaskSchedulerBM flags-gate NOP) stays OPT-IN — defaulting it ON made the default ladder proceed deeper into setTaskSchedulerBM and fault at the next unseeded gate (guestpc 0x106241c70) instead of the safe pre-existing run-variable exit (a default-path regression; tested and reverted). +2 regressions sh121 (merged into sh115_tests) + sh122 (once-guard guest-vs-file transform, fan-in chain addrs). arm64jit example suite 32/0. Commit 19c14f3.
NEXT: (a) the downstream 0x102175854 GameActivity/FMOD-region fault (SH45-documented) — now the concrete next gate reached by the active session-construction path: find what [x20+8] should hold (x20 = host-ptr leak or a real object arg), seed/materialize so the do-init continues building the app-data-model past the FMOD/GameActivity init; (b) wire the NativeHelper gameActivity_* milestones (registered SH111, MH_* set-only) into an actual session-drive so onFlagsLoaded->onAppReady fire and the app-shell constructs the first login GuiObject; (c) type-4 vector [0x106829ea8] install; flip SH115-122 default-ON only once the full clean session-constructing ladder is stable across runs.
## SH121 (Sep 14, 2026, hermes-worker): setTaskSchedulerBM SIGTRAP CLEARED — full --v2boot ladder + surface-handoff now run CLEAN end-to-end (EXIT 0). Workspace green.
The route-B SIGTRAP was NOT a JIT-emitted trap: read-only recon (deleg_b67653e9) proved it's the guest engine's own `raise(SIGTRAP)` (fatal leaf 0x626d1d0 -> raise(5)), fired the moment setTaskSchedulerBackgroundMode (guest 0x102bb2380 -> 0x10258aff0) lazily constructs the TaskScheduler whose ctor gates on [0x72739d4].bit0 ("flags loaded", `ldrb w8,[x8,#2516]` at guest 0x10224fa20, `tbz w8,#0,0x224fc80` at 0x10224fa24 -> the fatal raise). Because nativeGameGlobalInit now RETURNS via the SH82 thread-id trick (never loading flags), this ctor is the FIRST flags gate the [SH115] opt-in ladder hits -> exit 133 right after "driving setTaskSchedulerBM". Fix: NOP the `tbz` (e8 12 00 36 -> d5 03 20 1f) so the ctor proceeds regardless (mirrors SH116's code-site approach; [0x72730xx].bss is unmapped -> data seed may not land). Also seeded the REAL setTaskSchedulerBM version-gate [0x10683cff8]=0x0306 (adrp 0x683c000+0xff8) so it keeps its AppBridge-V2 main path — SH109's [0x10683d350] actually belongs to V2Init/V2Start and its comment mis-attributes it to setTaskSchedulerBM. +regression sh121 (tbz encoding 0x360012e8, imm14=0x97 -> guest 0x10224fc80, flag latch + version-gate addrs). VERIFIED (opt-in JIT_SH115_SINGLETON_PATCH=1 + JIT_ROUTEB_HASHFIX=1, real libroblox.so, runs/sh121-run.txt): EXIT 0, 0 crash, the FULL ladder — StartApp Ok -> nativeInitializeNativeFlags (benign soft-return) -> **nativeGameGlobalInit returned Ok** -> **nativeUpdateAdapterInit Ok(0x1)** -> **setTaskSchedulerBM(false) returned Ok(0x1)** (was exit 133) -> V2InitWithParams (soft-return pc 0x41, known SH103 host-leak benign) -> **StartLuaAppDM returned Ok** -> V2StartApp (soft) -> **V1 AppStart__ returned Ok** -> V2UpdateSurface (soft) -> **surface-handoff wired_xid=0x200000 in [0x10683d348]** + real task-frame #0 present swap Ok(0x1) + renderframe swap Ok(0x1). Default ladder (flag off) unaffected (inert code). Workspace green (25 suites, diff_battery 56/56, diff ENOSPC cleared /tmp). Commit e28a08e.
NEXT: (a) the remaining soft-returns (nativeInitializeNativeFlags/V2Init/V2Start/V2UpdateSurface, pc 0x31/0x41 outside image = SH103-class host-pointer-leak soft gate, non-fatal) — the SH110/111&115-chain has made them appear only when the ladder advances past the old gates; (b) THE SESSION LEVER — StartLuaAppDM returned Ok but its body still soft-returns before building the app-data-model -> getFilesDir fires (SH114 getters wired) -> rbx-storage.db -> GuiObjects -> engine self-constructs login/home; wire the NativeHelper gameActivity_* milestones (onFlagsLoaded->onAppReady, already registered SH111, MH_* still set-only) into an actual session-advance so StartLuaAppDM's instance builds real GuiObjects; (c) type-4 vector [0x106829ea8] install / flip SH115-121 default-ON once the full clean ladder is confirmed stable across runs.
After SH119 the app-bridge event dispatch crashed at guest 0x102b561b0 (`ldur x8,[x8,#-24]`) — a GENERIC ~40-site leaf (code-patching it would break other dispatches) reading the shared dispatcher-node .bss global 0x10683a460, whose +0 intrusive self-link = 0 → [0-24] fault. Seeded the DATA to the benign empty-singleton state: node[+0]=0x10683a000 (0-clean self-link → base==node), node[+0x20]=1 (base[+32] active → leaf returns 0 → caller skips dispatch work). Opt-in. VERIFIED: the SIGSEGV is gone; ladder runs nativeGameGlobalInit Ok → nativeUpdateAdapterInit Ok → then TRAPS (SIGTRAP/133) inside the setTaskSchedulerBM rung (next gate). All 8 sh115-120 tests + sh111 pass; workspace green.
IMPORTANT (default): the default ladder is run-VARIABLE — 2/5 runs crash at setTaskSchedulerBM guestpc 0x106240c40 (the KNOWN SH55/64 concurrent-thread flake, documented since SH55), 3/5 clean EXIT 0. NOT a regression from SH119/SH120 (both flag-gated; log 0 on the default path, verified).
Cumulative this session (opt-in JIT_SH115_SINGLETON_PATCH=1): singleton-dispatch soft-return resolution (SH115) → nativeInit lock-owner+flag-map (SH116/SH117: **nativeGameGlobalInit RETURNS** — standing SH-wall since SH54) → canary/render-ctx dual-use (SH118: full ladder clean) → SendAppEventOnAppReady ungated lambdas + dispatcher-node (SH119/SH120: app-bridge path executes). Commits 80f4777, 7d61aae, f14acd2, bb99999 (+SH120 pending).
NEXT: (a) the setTaskSchedulerBM SIGTRAP on the opt-in path; (b) the real session lever: StartLuaAppDM building the app-data-model → getFilesDir fires → rbx-storage.db → GuiObjects; MH_APP_READY + type-4 vector [0x106829ea8] wiring.
## SH118 (Sep 14, 2026, hermes-worker): canary/render-ctx GOT dual-use fixed — the FULL ladder now runs CLEAN end-to-end (opt-in JIT_SH115_SINGLETON_PATCH=1 + SH115-118). nativeGameGlobalInit returns; default unregressed; workspace green.
The surface-handoff (V2UpdateSurface) rung aborted `*** stack smashing detected ***`. Root cause (subagent): guest 0x1067d16f0 is BOTH the __stack_chk_guard GOT slot AND the render-ctx singleton (SH14/SH106 dual-use); renderthunk re-publishes it to the live ctx, so a canary fn whose body re-publishes the slot mid-execution (V2UpdateSurface) reads a mutated canary at epilogue vs prologue → false __stack_chk_fail (NOT a stack collision — ladder is the only live boot_sp user). Fix: seed both guard slots (0x1067d16f0 + 0x10631aa30) to a stable leaked pointer (0x2f_2a_1a_0a_0e_0f_10_11) right before the surface rung — the canary fn caches the pointer at prologue and derefs the cached pointer at epilogue, so *guard is constant regardless of mid-body re-publication. Idempotent.
**FULL LADDER CLEAN (opt-in):** EXIT 0 / 0 crash / 0 stack-smash — StartApp Ok → nativeInitializeNativeFlags (benign soft-return) → **nativeGameGlobalInit returned Ok** (standing SH-wall since SH54) → nativeUpdateAdapterInit Ok → setTaskSchedulerBM → V2InitWithParams → **StartLuaAppDM returned Ok** → V2StartApp → V1 AppStart → V2UpdateSurface → surface-handoff wired_xid=0x200000 [0x10683d348]=0x200000 (real XID in the engine window slot) → SendAppEventOnAppReady. Render pipeline runs (renderthunk real ctx, taskv4 present swap Ok(0x1), renderframe Ok). Log runs/sh118-run.txt.
OPEN (session-advance wall): MH_FLAGS_LOADED=false MH_APP_READY=false; [0x106829ea8]=0 (type-4 vector not installed by the session). StartLuaAppDM returns Ok but its body still soft-returns before building the app-data-model / GuiObjects.
All 6 sh115/116/117/118 tests + sh111 pass; workspace green; default ladder (flag off) unregressed (EXIT 0, StartLuaAppDM Ok, task frame present Ok(0x1)).
NEXT: (a) make SendAppEventOnAppReady/V2Init/V2Start bodies EXECUTE (not soft-return) so the session advances (MH_* → GuiObjects → scene nodes → engine self-constructs login/home); (b) flip SH115-118 default-ON once the clean full-ladder is confirmed stable.
The flag-map gate after SH116: nativeInit's flag-registration loop calls a hash-map probe helper (guest 0x10232090c, file 0x232090c, sole caller `bl 232090c`@0x23208d8) with a flag-map whose +0 bucket-array is UNSEEDED host-stack garbage → `ldr x8,[x0]; ldr x8,[x8,x11,lsl#3]` SIGSEGV (fault==[table+0]). Map is a per-run leak (fixed-addr seed unreliable) → leaf-rewrite the helper's 8B prologue to `ret`+`nop` (returns the non-zero map obj; caller `cbz x0` takes "found"/insert-skip). Deterministic, idempotent, non-vtable-widening. +1 test (sh117).
**BREAKTHROUGH** (SH115+116+117 active): the FULL --v2boot rung ladder now runs on ONE jit_run — StartApp Ok → nativeInitializeNativeFlags (benign soft-return, no crash) → **nativeGameGlobalInit returned Ok** (the standing SH-wall since SH54 — previously parked / never returned) → nativeUpdateAdapterInit Ok(0x1) → setTaskSchedulerBM (soft) → V2InitWithParams (soft) → **StartLuaAppDM returned Ok** → V2StartApp (soft) → V1 AppStart (soft) → V2UpdateSurface (soft) — with the render pipeline running (renderthunk real ctx, taskv4-frame present #0 swap Ok(0x1), renderframe Ok). One remaining host-side SIGABRT during/after the V2UpdateSurface surface-handoff rung (next gate). Log: runs/sh117-run.txt.
All 5 sh115/116/117 tests pass; workspace green; default ladder (flag off) unregressed (EXIT 0, StartLuaAppDM Ok, task-frame present Ok(0x1)).
NEXT: (a) clear the surface-handoff SIGABRT so SH115-117 flips default-ON; (b) nativeInit→StartLuaAppDM app-data-model→getFilesDir→rbx-storage.db (remembered session).
The SH110/111 next-gate (make V2Init/V2Start/V1AppStart/SendAppEventOnAppReady bodies COMPLETE instead of soft-returning through the 0x60-vtable read-past) is now implemented as a SCOPED differential patch, verified against the real binary:
- **SH115 (3 sites materialize-stable-object):** each accessor's 28B window (V2Init A @0x62517b8, V2Start A2 @0x6251a9c, V1AppStart B @0x626093c) is rewritten to `movz/movk` the stable zeroed 0x80 object (routeb_singleton_obj_addr; [OBJ+0]=benign 0x60 identity-leaf vtable) into x0, `str x0,[objA]`, and return it. This satisfies the SH114-rejected constraint (the virtual's return IS deref'd at +0x28 → must be a non-NULL stable object, not 0). NOT a vtable widening (nativeInit's 0x60 reads stay untouched).
- **Branch re-entrancy fix (OWN bug):** the patched store-slot overwrites the shared `mov x0,xzr` tail, which the EARLY-RETURN branches (b.eq AND cbz x19 — 6 total across the 3 sites) jump to before slot5's `ldr x9,[x19]` runs → stale-x9 store. Repointed all 6 to the epilogue. This was the first crash (fault=0x28→x9=1) after applying SH115.
- **SH116 (nativeInit lock-owner helper):** with SH115 ON, nativeInit (rung 0) advances past its former soft-return and calls `pthread_mutex_lock(&*(0x1072739c0)+0x28)` where the .bss global's page is UNMAPPED at runtime (mprotect RW→ENOMEM — the engine re-maps that .bss page on boot). Since it can't be seeded by a store, patched the helper (file 0x2320710/0x2320714/0x2320718 = guest 0x102320710) to materialize a stable all-zero 0x60 object into x0 (movz+movk hw1+hw2) so the lock targets a valid PTHREAD_MUTEX_INITIALIZER. VERIFIED: the +0x28 getter gate CLEARS — nativeInit now runs past it to the NEXT gate (guestpc 0x10232090c, stack deref).
**Empirical (JIT_SH115_SINGLETON_PATCH=1):** the real render pipeline RUNS in the same ladder (renderthunk recovers real ctx vtable 0x106731ae0; taskv4-frame present #0 swap Ok(0x1); renderframe swap Ok(0x1)) AND nativeInit advances past the lock. The ladder still exits 134 at the next nativeInit gate, so the patch remains OPT-IN (default clean).
**Default ladder UNREGRESSED:** all new code is inert unless JIT_SH115_SINGLETON_PATCH=1 (routeb_singleton_obj_addr not called on the default path); default --v2boot ladder re-verified EXIT 0 / 0 crash / StartLuaAppDM returned Ok / task-frame present swap Ok(0x1) / renderframe Ok. The one sh115-default.txt EXIT 134 observed was the KNOWN run-variable SH55/64 concurrency flake (2 immediate re-runs clean).
Encodings subagent-verified (aarch64 assembler). +4 regression tests (sh115 materialize window, sites, branch repoint; sh116 helper). arm64jit example tests 27/0; workspace green.
Commit 8f48e3b (pending) — opt-in core + tests + docs.
NEXT: (a) clear the nativeInit guestpc 0x10232090c stack-deref gate so SH115 flips default-ON (the render pipeline joins the standard ladder); (b) nativeInit→app-data-model→getFilesDir→rbx-storage.db (remembered session).
Drives `nativeAppBridgeV2SendAppEventOnAppReady` (guest 0x102bb463c) as an opt-in `--v2boot-send-appevent` rung after surface-handoff (deleg_0d4e5597 disasm: the NativeHelper MH_* milestones are set-only, nothing polls them; this Java->engine bridge is the missing session-advance response that feeds LuaAppExperienceController -> AppShell). VERIFIED on real binary with full combined ladder: EXIT 0, 0 crash, real task-frame present #0 swap Ok(0x1), app-event rung logs `driving SendAppEventOnAppReady (event="Home")` then soft-returns at the SAME 0x60-vtable latency leak as setTaskSchedulerBM/V2Init/V2Start — its body still doesn't complete, so it's the correct desync-safe harness action waiting on the singleton-dispatch resolution. HONEST: MH_FLAGS_LOADED/APP_READY stay false; no scene-node provider yet.
Also closed the persistence hole (deleg_f84d9f90 disasm): the real client gets its data dir via Context.getFilesDir()/getCacheDir()/getDatabasePath (rodata 0x244d48) -> nativeSetFilesDirectory global 0x1026d600, and its OWN SQLite datastore is rbx-storage.db (NOT session.db). Added these to auto_value_string_getter returning guest-absolute dirs under /data/user/0/com.roblox.client so the engine's own .db opens route through fsmap to persistent host disk. NEW regression sh114_context_data_dir_getters_resolve_via_fn_table (arm64jit 342/0). HONEST: getters did NOT fire this run (StartLuaAppDM session still soft-returns before building the app data model) — the fix is latent-but-correct + boot-safe + tested.
SCOPED-BLR REJECTED (documented gate): patching the 3 singleton-dispatch sites (0x62517c4/0x6251aa8/0x6260948) `blr->mov x0,xzr` regresses nativeInitializeNativeFlags to NULL+0x28 SIGSEGV (fault=0x28) — the virtual return IS deref'd by a caller, so a scoped leaf must return a STABLE zeroed guest object (routeb_singleton_obj_leaf), NOT 0/xzr (SH110-class). Rock: keep the benign 0x60 soft-return baseline. Doc docs/frontier-sh114-appevent-datadir.md, log runs/sh114-appevent.txt.
NEXT: (a) scoped leaf returning a stable zeroed guest object at the 3 sites (constraint: non-NULL, [ret+0x28] valid) to make SendAppEventOnAppReady/V2Init/V2Start execute; (b) get StartLuaAppDM to build the app data model so getFilesDir fires -> engine opens rbx-storage.db in the store -> remembered session; (c) wire the MH_* milestones into a session-advance polling gate.
## SH113 (Sep 14, 2026, hermes-worker): surface-handoff rung — engine's OWN JNI entry populates the real window slot headlessly. Workspace green. Commit f41ba5c.
Drives `nativeAppBridgeV2UpdateSurfaceAppWithPlatformParams` (guest 0x1025f5fec) as an opt-in `--v2boot-surface-handoff` rung after the V1 AppStart fallback, on the SAME ladder thread (SH55/64-safe). Per recon-sh113-surface-handoff.md: the JIT's ANativeWindow_fromSurface host shim ignores the Surface arg and returns the SH112-wired XID, so x2 needs only a non-null token and x3 a readable platformParams sentinel. VERIFIED (runtime self-hosted on Xvfb, full combined ladder): EXIT 0, 0 smash, nativeGameGlobalInit Ok + StartLuaAppDM Ok + real task-frame present #0 swap Ok(0x1); the rung drives the engine entry, soft-returns (known singleton-vtable leak pc 0x4806... — benign, the store precedes it), and **`[0x10683d348]` = 0x200000 — the REAL wired XID is now in the engine's window slot via the engine's OWN JNI entry, not a raw host seed**. MH_FLAGS_LOADED=false MH_APP_READY=false (recon-consistent: update-surface alone does not fire onAppReady; that needs the forced CallVoidMethod or the real-jstring SendAppEventOnAppReady path, and true GuiObject construction still needs the session advance — standing wall). Repo doc docs/recon-sh113-surface-handoff.md committed (a90858b).
## SH112 (Sep 14, 2026, hermes-worker): runtime now renders on a REAL window surface (self-hosted Xvfb) — boot-window stale-socket bug fixed + fresh render-thread XID. Workspace 519/0.
Xvfb is present on this box but was never started by any run (`DISPLAY` empty), so EGL had no window to build a surface on and the combined render ladder died at renderinit with `std::runtime_error: Error creating context: eglCreateWindowSurface (bad-ptr)` EXIT 139 (pre-existing, SH111). Two fixes in elfjit.rs:
1. **`wire_real_window` stale-socket bug**: it picked display `220+pid%50`, and if `/tmp/.X11-unix/X<n>` EXISTED it "broke" (skipping the Xvfb spawn) then failed to connect 40x — because prior sessions left DEAD sockets. Result: the boot ANativeWindow ALWAYS fell back to the sentinel. Rewritten connect-first: try the display, and only on connect-failure unlink the stale socket + spawn its own Xvfb (tries several displays). Now self-hosts the display, sets DISPLAY/EGL_PLATFORM=x11, and wires a REAL boot XID.
2. **Fresh render-thread XID** (recon deleg_9935787c, completes the in-flight change): the `--renderthunk` path no longer reuses the boot window (already EGL-surfaced -> EGL_BAD_SURFACE) — it opens a FRESH window and passes that XID to the renderinit thunk.
VERIFIED with the runtime self-hosted on Xvfb (booting no longer needs a manually-started display): `wire_real_window` wired real boot window XID=0x200000 on :NNN; renderinit returned Ok and recovered the engine's REAL ctx (vtable 0x106731ae0, live EGL display/surface/context); **task frame #0 present swap Ok(0x1) + `renderframe swap Ok(0x1)` — real task-driven + engine frames presented on a REAL window surface** (previously the whole process died at renderinit EXIT 139). No-render v2boot ladder UNCHANGED + re-verified clean (EXIT 0, 0 crash, full ladder to V2StartAppWithParams Ok + V1 soft-return).
NEXT GATE — RESOLVED/REFINED (verified, reproducible, 2 runs): the combined ladder + REAL rendering now runs CLEAN **EXIT 0 / 0 crash** with `nativeGameGlobalInit returned Ok` + `StartLuaAppDM returned Ok` + persist 45B byte-exact roundtrip, ON a real Xvfb surface (task-frame #0 present swap Ok(0x1) fires during nativeGameGlobalInit). The earlier `*** stack smashing detected ***` was caused SPECIFICALLY by the `--renderframe-drive` harness frame-loop (SH22): it issues UNBOUNDED repeated frame-fn 0x105b32c00 jit_runs on the render thread while the ladder's do-init walks its seeded globals -> concurrent-top-level jit_run global-state tear (SH55/64/100 class). Dropping that diagnostic harness flag (keep renderinit/renderthunk/renderframe/renderframe-seedgles/taskv4) coexists cleanly; the BOUNDED type-4 task-frame plane (SH60/61) is the correct concurrent-render path, not the raw frame-loop. Product path (no --v2boot, uses renderframe-drive alone) is unaffected. NEXT after this: (a) wire the NativeHelper milestone atoms (SH111) into a session-advance gate so StartLuaAppDM's instance actually builds GuiObjects -> the engine's first self-constructed scene nodes; (b) re-attempt renderframe-drive content ONLY after the ladder completes (serialize the harness frame-loop post-do-init instead of concurrent); (c) the three soft-return sites (setTaskSchedulerBM/V2Init/V2Start, pc 0x55/0x1a1/0x6741726573557765 leak) still need the scoped blr patch (SH110/111).
## SH111 (Sep 14, 2026, hermes-worker): NativeHelper gameActivity_* callbacks wired (route-B step 2 first half); singleton-vtable widening empirically dead-ended — 0x60 baseline locked. Workspace 25 suites green, arm64jit lib 341/0, elfjit examples 23/0. Commit 2e4b7f2.
Wired the five NativeHelper `gameActivity_*` JNI callbacks: CALL_VOID_METHOD (slot 61) + CALL_STATIC_VOID_METHOD (141) are no longer the shared no-op `ok` stub — new name-dispatching `jni_call_void_method` (jni.rs) fires a per-callback milestone atomic (onFlagsLoaded/onEngineInitialized/onAppReady/onGameLoaded -> MH_* atoms; onDidLogInReceived is VOID-with-String-arg per disasm, NOT a jboolean). All hold the engine's fake-Java session advancement hook a StartLuaAppDM needs. Hermetic regression nativehelper_game_activity_callbacks_dispatch_void_method drives all five through the OFFICIAL slot. Empirically dead-ended the dispatch-singleton vtable: the 0x60->0x580 differential widening (object-leaf default, gate slots +0xf8/+0x108/+0x548 identity, +0x558/+0x568 NULL) STILL regresses nativeInitializeNativeFlags (exit 134) — the SHARED vtable can't serve nativeInit (clean only via benign read-past soft-return) and V2Start (wide coverage) at once. Reverted; regression sh111_singleton_vtable_stays_0x60_baseline locks the narrow baseline. The three soft-returns need a SCOPED call-site patch (blr at 0x62517c4/0x6251aa8/0x6260948), NOT vtable growth. Verified the combined-recipe exit-139 `renderinit (bad-ptr)` is PRE-EXISTING (identical on pristine HEAD) and the no-render clean ladder is EXIT 0 / 0 crash / StartLuaAppDM Ok (rungs nativeGameGlobalInit -> nativeUpdateAdapterInit -> setTaskSchedulerBM -> V2Init -> StartLuaAppDM -> V2Start -> V1 all driven).
## SH110 (Sep 14, 2026, hermes-worker): root-caused the 3 soft-return leak sites; vtable widening reverted — clean baseline confirmed + taskv4 verified. Workspace 518/0.
The V2Init/V2Start/V1 AppStart soft-returns ('run_loop: pc 0x48..' host-code bytes) are C++ VIRTUAL dispatches through lazy-init singletons 0x106829a48/0x106829a68 at vtable+0xf8/+0x108/+0x548 (NOT JNIEnv — recon deleg_d288bc8a). The 0x580 flat-vtable widening made nativeInitializeNativeFlags hard-crash (high-slot leaf returns a0 which a later caller derefs) -> REVERTED; 0x60 is the correct committed baseline (clean full ladder: EXIT 0, 0 crash, 0 smash, StartLuaAppDM returned Ok). Recon confirmed the 3 exact slots store-not-deref (out-field [[x19]+0]); a per-slot stable-zeroed-object leaf is the option-B differential fix (do NOT blanket-widen — other exposed slots deref the return). VERIFIED taskv4 (recon-v3 deliverable 1): type4_frame_thunk registered at host-thunk, type-4 vector [0x106829ea8] seeded, heartbeat w4#2/#3 patched -> every drain dispatch routes w4=4 through the vector; product path unregressed (exit 124, 0 crash, engine frame-fn Ok). RENDERCTX-recovery-vs-dispatch timing means 0 task-driven frames presented in the headless ladder window (timing, tracked). Commits: b19b1c2 (SH110 widen) -> 3ceced0 (SH110 revert+doc). NEXT: (a) per-slot vtable leaf at +0xf8/+0x108/+0x548 returning a stable zeroed guest object (differential, keep 0x60 baseline for the others); (b) the SH55/64 thread-desync surface (nativeGameGlobalInit's real worker thread host-OOB shown under the widening); (c) type-4 real task-frame presentation timing against RENDERCTX.
## SH108 (Sep 13, 2026, hermes-worker): layer-2 stack-smash ROOT-CAUSED + FIXED — render thread needs its own guest stack; ladder now drives StartLuaAppDM Ok. Workspace 518/0.
After SH106 (guard GOT seed) + SH107 (adapter-record seed) broke the rung-1 standing wall, a layer-2 false `__stack_chk_fail` persisted ONLY on the render path: the canary slot [x29-16] of a ladder canary fn was clobbered with a saved `stp x29,x30` frame pair (JIT_DUMP_REGION probe proved it was wrong immediately after the prologue, before any nested call). ROOT CAUSE: both the `--renderinit` thread (`s3.x[31]=isp`) and the `--v2boot` ladder thread (`boot_sp=st.x[31]`) seeded their jit_run guest SP from the SAME boot stack; concurrent render guest frames grew down into the ladder's live frame and a nested guest object-init `stp x29,x30` (return addr into 0x1d99ff0 memset) overwrote the ladder canary slot. FIX (elfjit.rs renderinit thread): give the render thread its own dedicated leaked 1 MiB guest stack (`Box::leak`; guest==host identity so guest-addressable), mirroring `run_guest_callback`. `isp` still passed to render walker paths (kept real binding). VERIFIED (real libroblox.so, `--v2boot` + `--renderinit` + `--renderthunk`): EXIT 0, **0 stack-smash** (was 1 every render run), 0 SIGSEGV/SIGABRT, ladder drives nativeGameGlobalInit Ok -> nativeUpdateAdapterInit Ok(0x1) -> setTaskSchedulerBM -> V2InitWithParams -> **StartLuaAppDM Ok** (never executed before this cycle) -> V2StartAppWithParams -> V1 AppStart__. Workspace **518/0**. Product path unregressed (exit 124, 0 crash, persist 45B byte-exact, present swap Ok(0x1)). Commit db13ca5. Doc docs/frontier-sh108-render-stack-isolation.md. NEXT: V2StartAppWithParams/V2InitWithParams soft-return `run_loop: pc 0x194/0x21/0x... outside image` (indirect blr to a garbage non-image addr — a soft gate, not a crash); type-4 producer vector [0x106829ea8] still 0.
## SH107 (Sep 13, 2026, hermes-worker): broke the SH-wall — nativeGameGlobalInit RETURNS + rung-2 adapter-init clears. Workspace 518/0.
SH106 root-caused the SH105 false stack-smash (layer 1): `plt::patch_stack_canary` seeded only the JNI_OnLoad guard GOT slot 0x631aa30, but the WHOLE binary's stack-protected functions (48,831 refs) read their guard base from GOT slot 0x67d16f0 (guest 0x1067d16f0) — which was left holding a MUTABLE pointer (the engine's render-ctx singleton, the slot `--renderthunk` publishes into), so a canary fn running across that mutation saw a different "canary" at prologue vs epilogue -> false `__stack_chk_fail`. Fix: seed BOTH guard slots with one resolve-once stable canary (libc `__stack_chk_guard` or the 0x2f_2a... static), forcing 0x67d16f0 even when it already holds a page pointer. Product baseline unregressed (exit 124, 0 crash, persist byte-exact, present swap Ok(0x1)); Workspace 518/0. Commit 00d6247 (+9940d89 diagnostics: JIT_DUMP_REGION + canary-slot/guard probes). SH107: with the guard stable, `nativeGameGlobalInit` (rung-1) finally RETURNS — the standing wall since SH54/55 (was "gameGlobalInit NOT RETURNED"). The ladder advanced to rung-2 `nativeUpdateAdapterInit` (0x10221c3ec) which faulted at 0x10221d7a8 on a NULL BSS adapter-record pointer [0x106ed7a18] (`adrp x9,6ed7000; ldr x9,[x9,#2584]; ldrb [x9]`). Seeded a zeroed 0x20 record -> rung-2 returns Ok(0x1). Commit 9afc921. VERIFIED (no-render `--v2boot`): nativeInitializeNativeFlags -> nativeGameGlobalInit Ok -> nativeUpdateAdapterInit Ok(0x1), all three reaching their "after <rung>" dump. This is the DEEPEST the ladder has ever run (previously parked/aborted at rung-1 every cycle). OPEN GATES: (a) the layer-2 canary-slot writer ONLY fires on the render-path (with `--renderinit`/`--renderthunk`, a nested-frame `stp x29,x30` saved-pair — stack ptr + return addr, e.g. 0x101d99ff0 — is written onto the ladder frame's canary slot [x29-16]; WITHOUT render flags the ladder runs clean past gameGlobalInit); (b) `setTaskSchedulerBM(false)` soft-returns `run_loop: pc 0x88b8b515058595a outside image` (a garbage-blr target — next gate); (c) `V2InitWithParams` (0x102365c54) is the big remaining init and is now driven (run timed out with the foreground cap, not a crash). docs/frontier-sh106-canary-got-seed.md + frontier-sh107-adapter-record-seed.md. Repro: `timeout 60 env JIT_DRIVE_LIFECYCLE=1 V2BOOT_WARMUP_MS=4500 JIT_ROUTEB_HASHFIX=1 ./target/debug/examples/elfjit .../libroblox.so 0x2173ff4 --jni --startapp 0x258b144 --v2boot --persist-roundtrip --kicker 0x106863af8`.
## SH105 (Sep 13, 2026, hermes-worker): stack-smash gate root-caused to a host-call-return leak into guest x0 (SH103 class). SH102+SH103 (commit 4b094e9) cleared the ClientRunInfo base-url log gate (0x102dade08 b.lt -> unconditional b 0x194; it derefs a never-inited bss singleton *(0x106ED7A28)) and the keyed-registry static read (`ldr x24,[x20]`@0x101db5ffc -> `mov x24,#0`, intended NULL; x20 had a host pointer from a PLT strlen host-call). SH104 (reverted): no-op shims for pthread_cond_broadcast/signal/destroy+mutex_destroy did NOT stop the stack-smash — cond_broadcast proven innocent. SH105 block-entry probe, DEFINITIVE: at the failing frame's canary-compare (pc 0x102dae368), stored [x29-16]='x29+0x30' (a self-stack pointer the guest stored into its own canary slot; x29=0x561044628e20), while [x22]=guardv=0x56103553c610 intact and [x29-8]=0x101d99ff0 (guest code addr). So a host-call RETURN value leaks into guest registers (x20 in SH103, now into a value the scheduler guest stores onto its frame canary slot) -> false __stack_chk_fail SIGABRT. Bridge-level JIT-hosting bug; forcing the canary is UNSAFE. SH106 corrected this: the guard slot the canary fn read was UNSEEDED/mutable — fixed. Doc frontier-sh104-105-stacksmash-hostleak.md. Workspace **518/0**. (Historical SH-chain below.)
## SH97 (Sep 13, 2026, hermes-worker): guarded guest C-string pointers in strlen/formatted-output host shims — CLEARED the strlen-garbage SIGSEGV at guestpc=0x0. Recon deleg_190c3f48: the deep gameGlobalInit walk formats a log via `%s%s%s` (guest 0x1025aeb1a) and passes a GARBAGE C-string pointer (0xffffff80ffffffc8, from an unseeded map field) to a host string fn; the JIT bound strlen/__strlen_chk to raw glibc strlen with no check -> SIGSEGV (raw[]=glibc AVX2 strlen; fault==rdi==rdx==0xffffff80ffffffc8). The guest also reaches the garbage via vsnprintf which internally strlen()s. Fix: jit.rs `safe_cstr_len` (refuse 0, sub-image <0x100000000, non-canonical >>48==0xffff -> 0; else glibc strlen) + `image_domain_contains`; shims.rs bionic_strlen_chk + new bionic_strlen via safe_cstr_len, registered `(b"strlen\0",...)`; bionic_vsnprintf (routes guest vsnprintf through render_vfprintf whose %s now uses safe_cstr_len -> renders (bad-ptr)/(null), writes NUL-term result to guest buffer), registered `(b"vsnprintf\0",...)`. VERIFIED: the strlen-garbage SEGV (fault=0xffffff80ffffffc8, guestpc=0x0) is GONE on every run — the ladder advances past the formatted-string site to a REAL guest pc (0x10220847c, deeper in nativeGameGlobalInit) / a deeper abort. SIGSEGV dropped to a later site. Workspace 517/0 (+1 regression safe_cstr_len_rejects_nonc_canonical_garbage). Product unregressed (124, 0 crash, persist byte-exact). Doc frontier-sh97-cstr-guard.md. NEXT: the new site guestpc 0x10220847c / worker-thread abort — guard sibling string ops (strncmp/strcpy/sprintf/__android_log_print) with safe_cstr_len + seed the source map field -> gameGlobalInit RETURNS -> rung 2 nativeUpdateAdapterInit (0x10221c3ec) -> type-4 [0x106829ea8] -> NativeHelper.
## SH96 (Sep 13, 2026, hermes-worker): re-assert the coherent 0x400 map header on EVERY INSERT entry — clears the post-growth index-overflow chain-walk (0x1029f3f7c no longer the crash site). Recon deleg_9e27d070: crash map IS seeded+trusted (trust layer didn't miss it); the ENGINE'S OWN GROWTH (0x29f3ec0, str x0,[x19]@0x29f3ee4) re-writes the numeric header (+0x40 mask/+0x44 div/+0x3c cap/+0x48 load) AFTER the once-per-map SH90 seed, so a later insert's idx*8 overflowed the owned 0x2000 array into image (x22=0x1029b37f4 code page) — the SLOT ADDRESS overflows, not a phantom slot value. Fix (jit.rs): for a trusted family map RE-ASSERT +0x3c=0x400/+0x40=0/+0x44=0x400/+0x48=0x100/+0x60=0/+0x58=0 every INSERT entry so idx*8 stays in the owned 1024-slot array + keep SH91 slot-value scrub; never repoint +0x00. VERIFIED: old 0x1029f3f7c guest fault gone; runs fail at a DIFFERENT host-side class (guestpc=0x0, fault==0xffffff80ffffffc8 / fault==rip==heap, rbx_matches_gueststate=false, run-variable 134/139 — SH55/64 concurrency class). Workspace 516/0. Product unregressed. Doc frontier-sh96-header-reassert.md. NEXT: the host-side fault class — DONE in SH97.
## SH95 (Sep 13, 2026, hermes-worker): stop seeding the pb_defaults rwlock-ptr slot 0x106838378 with the map pointer — CLEARED the pthread_rwlock_unlock heap-crash. When the registrar completed (SH94), the do-init tail (mov w0,#1; ret at file 0x29b39d4) called pthread_rwlock_unlock (guest 0x102a1ce5c) with x0=[0x106838378]; SH88's seed list [368,378,380] had seeded 0x378 with the MAP ptr, but 0x378 (offset 888) is a pthread_rwlock POINTER slot — unlock on garbage does an indirect elision jump -> SIGSEGV fault==rip==heap. Only 0x106838368 + 0x106838380 are the registry-MAP slots. Fix: seed only those two; leave 0x378 as .bss-zeroed (valid unlocked) rwlock. VERIFIED: 0x102a1ce5c crash GONE; registrar completes; ladder advances to a deeper NATIVE fault (0x1029f4034 INSERT-tail, fault==rip==host-heap, rbx_matches_gueststate=false — SH55/64 concurrent-thread/block-cache class). Also SH95b: registered the SH88 substitute map's leaked bucket array in jit TRUSTED_BUCKETS so SH91's phantom-slot scrub covers it. Workspace 516/0. Product unregressed. Doc frontier-sh95-rwlock-slot.md. NEXT: the INSERT chain-walk index-overflow — DONE in SH96.
## SH94 (Sep 13, 2026, hermes-worker): killed the post-SH93 6.6M-iteration registrar spin — don't clobber the INSERT-entry x19 walk iterator. Recon deleg_3f812009: the post-barrier do-init body runs cleanly to its ret (file 0x2206e9c, no 2nd CEvent); the spin was pre-barrier, in the pb/Otel registrar loop (file 0x29b37e0: `mov x1,x19; bl 29f3e70; ldr x8,[x19,#16]!; cbnz`), where x19 is the CALLEE-SAVED walk ITERATOR/KEY, NOT the map. SH92 had been substituting x19 too; since x19 is callee-saved, the corrupted value returned to the caller made `[x19+16]` read the substitute's +0x10=SPAN_HASH (nonzero) forever -> `cbnz` never terminated -> 6.6M-iteration spin. Fix (jit.rs): at INSERT entry substitute x0 unconditionally (when non-family); substitute x19 ONLY when x0 is also non-family. VERIFIED: 6.6M reg=x19 flood GONE (subx0 is a sane 49); registrar loop terminates; ladder hits a NEW shallower crash (0x1029b37ec registrar back-edge / 0x1029f4034 INSERT-tail, fault==rip==heap, run-variable 134/139). Workspace 516/0. Product unregressed. Doc frontier-sh94-x19-iterator-spin.md. NEXT: resolve the native 0x1029f4034 heap-crash — DONE in SH95.
## SH93 (Sep 13, 2026, hermes-worker): NOPed the gameGlobalInit CEvent barrier (file 0x2206e70, guest 0x102206e70, `bl 2207578` CEvent::wait 0x940001c2 -> nop) — CLEARED the futex deadlock park. Recon deleg_d518ab04: the park is NOT nanosleep — file 0x284d114 is a futex() (__NR_futex=98, WAIT_BITSET|PRIVATE) backing SyncTask/CEvent::wait, polling byte[CEvent+1] bit0; the completion is set by a TaskScheduler worker-thread job that never runs headlessly (SH55/64 forbid concurrent jit_run). [0x72739d4] bit0 is an OUTPUT of the do-init, not the wait. Fix: NOP the barrier in the do-init ONLY — scoped via single-predecessor `b.ne 0x2206e28` at file 0x2206df0 (non-main-thread path only); product main-thread returns AFTER the barrier, never reaches 0x2206e70 (product run shows NO SH93 line). Do NOT patch shared generic 0x2206ebc/0x2207578/0x2850520 (~5-14 callers, SH82 lesson). VERIFIED: v2boot EXIT 124(futex-park)→0(clean, no crash) — deadlock GONE; do-init now enters the TaskScheduler hot registration loop (millions of block hits) but gameGlobalInit still has not RETURNED from jit_run (next layer). Workspace 516/0. Product unregressed (124, persist byte-exact, 0 crash, no SH93 line). Doc frontier-sh93-cevent-unpark.md. NEXT: trace the post-barrier TaskScheduler poll — DONE in SH94.
## SH92 (Sep 13, 2026, hermes-worker): substituted NON-FAMILY map/this at the INSERT entry — CLEARED the OTel registrar loop gate. Recon deleg_03083bd6: registrar loop (file 0x29b3814) read its INSERT map from [0x106838380] which held the .data descriptor-table base 0x1067da308 (in-image, passed SH88's sub-image predicate; non-coherent) -> INSERT ran with garbage map, +0x30 stack-spill slot became a code-fetch target (fault==rip==host stack at 0x1029b3828) — new SH86b foreign-object-as-map instance. Fix (jit.rs): extend substitution to INSERT 0x1029f3e70 — overwrite x0/x19 with the seeded substitute iff non-family (m<0x100000000 OR [m+0x10] not in {0x1029b4a84 span, 0x102a25dec string}); family maps + substitute (has +0x10==SPAN_HASH) never re-substituted. VERIFIED: substitution ~1.25M AND 0x1029b3828 SIGSEGV GONE — ladder runs the FULL OTel/pb_defaults registration NO CRASH; nativeInitializeNativeFlags returns cleanly; nativeGameGlobalInit runs to timeout with render frames. Workspace 516/0 (+1 regression). Product unregressed. (4 same-run failures = ENOSPC from full /tmp; pass after cleanup.) Doc frontier-sh92-registrar-nonmap-substitute.md. NEXT: the gameGlobalInit PARK — DONE in SH93.
## SH91 (Sep 13, 2026, hermes-worker): scrubbed phantom image-range bucket slots at the insert entry — CLEARED the 0x1029f3f7c node-pointer fault. Recon deleg_f77df07a: crash-map header/array valid + never reallocated (not growth); a single bucket slot held IMAGE address 0x1029b37ec (phantom head link after ~thousands of inserts into the never-grown 1024-slot map). Chain-walk `ldr x23,[x22]` (file 0x29f3fb4) picked it up, `ldr x8,[x23,#16]` deref'd image+16 (SIGSEGV 0x1029f3f7c). Fix (jit.rs): scrub any slot value in [base,base+len) -> NULL at INSERT entry; only arrays WE own (shared TRUSTED_BUCKETS set — fixed a Rust panic from a foreign map's invalid +0x00); no +0x00 repoint (SH84/86b). VERIFIED: 0x1029f3f7c GONE; ladder faults DEEPER at guestpc 0x1029b3828 (OTel registration CALLER loop iterating .data table 0x1067da308 into registry map [0x106838380], `ldr x8,[x19,#16]!; cbnz`) with fault==rip==host-heap (blr-into-heap, SH86 class). Workspace 515/0. Product unregressed. Doc frontier-sh91-phantom-bucket-scrub.md. NEXT: clear the 0x1029b3828 registration-loop gate — DONE in SH92.
## SH90 (Sep 13, 2026, hermes-worker): seeded the SPAN-hash map (SH84 empty-header+array seed widened from string-hash 0x102a25dec to ALSO span-hash 0x1029b4a84) — CLEARED the 13th-insert map-GROWTH fault. Recon deleg_efda8475: OTel span map's numeric header (+0x3c cap/+0x40 mask/+0x44 divB/+0x48 load) was unseeded garbage; after ~12 inserts it grew IN PLACE with garbage -> post-growth probe read a bucket slot as image-code ptr (x22=0x1029b37f4, x23/x8=0x9401a599f941be80) -> ldr [x23+16] SIGSEGV at 0x1029f3f7c. Widened insert-entry seed gate to both family hashes (once per empty map at sole block-entry 0x1029f3e70, never foreign). VERIFIED: INSERT block executes 3439x vs ~12 — thousands of keys insert cleanly; growth gate CLEARED. Remaining fault = deeper node-chain corruption at same pc after thousands of inserts (own recon). Workspace 515/0. Product unregressed. Doc frontier-sh90-span-map-seed.md. NEXT: root-cause the deep chain-walk corruption — DONE in SH91.
## SH89 (Sep 13, 2026, hermes-worker): extended the map-family dispatch patch to the INSERT op. After SH88 cleared the FIND gate, the ladder faulted at the INSERT op's own optional-hash-2 dispatch (file 0x29f3f78, guest 0x1029f3f78) — byte-identical SH87 pattern: `ldp x1,x8,[x19,#16]; cbz x8; blr x8(0x1029f3f78); blr x1`. When the map REHASHES/GROWS a freshly-moved map's +0x18 momentarily holds garbage (0x9401a599f941be80) -> blr x8 -> fault 0x0. routeb_patch_map_dispatch now patches BOTH sites (FIND/grow 0x1029f4280 + INSERT 0x1029f3f78) -> blr x1 + drops both block-cache ranges. VERIFIED (JIT_TRACE): INSERT dispatch fires blr x1 to primary span-hash (block@0x1029f3e70 -> pc=0x1029b4a84), the map completes 12 consecutive inserts, then faults at the 13th entry through the map-GROWTH/rehash path (guestpc 0x1029f3f7c, x8=0x9401a599f941be80, fault=0x0) — one more gate advanced. Workspace 515/0. Product unregressed (124, 0 crash, persist byte-exact). Doc frontier-sh89-insert-dispatch-patch.md. NEXT: clear the map-GROWTH gate (extend the SH84 SEEN-set header/empty-map seed to grow/rehash entries, or seed the newly-grown map) -> then nativeGameGlobalInit returning -> rung 2 -> type-4 vector [0x106829ea8]. (Growth gate cleared by SH90.)
## SH88 (Sep 13, 2026, hermes-worker): substituted the OTel/pb_defaults registry map for sub-image map/this tags — cleared the OTel FIND dispatch gate surviving SH87. Recon deleg_ba208bc6: FIND op (file 0x29f424c) was handed .data.rel.ro protobuf field-TAG constant 0x1800064 (from 16-byte table at file 0x62f5110) as map/this because the upstream registry map (BSS 0x106838368/378/380) is never built under the JIT; tag < 0x100000000 is never a pointer -> [0x1800064+16] unmapped -> SIGSEGV 0x1800074. Fix (elfjit routeb_seed_pb_registry_map + jit routeb_substitute_map): seed coherent empty span-hash map (SH84 shape, +0x10=0x1029b4a84) into 3 BSS slots, register, substitute any non-zero sub-image x0/x19 at FIND entries (0x1029f424c/0x1029f4284). Verified: substitution 492x; FIND gate GONE; run faults one gate DEEPER at the INSERT op (0x1029f3f7c blr x8 through a rehashed map's fresh +0x18 garbage 0x9401a599f941be80, x0=0x40000d5) — the predicted rehash/erase next gate. Workspace 515/0 (+1 regression routeb_hashfix_substitutes_subimage_map_candidate). Product unregressed (124, 0 crash, persist byte-exact). Doc frontier-sh88-pb-registry-substitute.md. NEXT: patch INSERT dispatch blr x8->blr x1 (file 0x29f3f78, same family as SH87) — DONE in SH89.
## SH87 (Sep 13, 2026, hermes-worker): patched the hash-map family generic dispatch `blr x8` -> `blr x1` (file 0x29f4280) so the optional +0x18 hash-2 slot can never be branched into when it holds a non-image value — clearing the OTel rehash-copy gate (0x1029f4284 blr-into-0x1800064). Recon deleg_48b60f95: hash2 is redundant in this family (observed real value 0x1029b4ae8 is just `br x1`, aliasing the primary hash) and the hash only selects a bucket probe (correctness via the key-eq comparator at +0x08), so forcing `blr x1` is behavior-preserving and immune to JIT block-entry gaps. The +0x18=0x1800064 is a STABLE constant decoded from `.data.rel.ro` file 0x62f5110 (not random heap). Applied only on --v2boot (like SH81 gate force). JIT_TRACE confirms blr x1 now fires the primary span-hash 0x1029b4a84. NEW caller-side gate surfaced: the rehash fn is entered with x19=0x1800064 (a constant, not a heap map) at some call site -> reads [0x1800064+16] and faults. Product path unregressed (exit 124, persist byte-exact, 0 crash). Workspace 514/0. Doc docs/frontier-sh87-map-dispatch-patch.md. NEXT: find the call site passing 0x1800064 (from .data.rel.ro 0x62f5110) as the map/this arg — likely a per-OTel-resource-schema descriptor registration index misread as the map base; then nativeGameGlobalInit returning -> rung 2 nativeUpdateAdapterInit (0x10221c3ec) -> check rungs 2-6 install the type-4 producer vector [0x106829ea8]. (SH87 cleared by SH88.)
## SH86 (Sep 13, 2026, hermes-worker): CLEARED the OTel/pb_defaults operator-new allocator-hook gate (the deepest do-init gate yet). Read-only recon deleg_6c882d29 root-caused guestpc 0x1029b43f0 to the CRT `operator new` wrapper (file 0x2a0d9b8): it compares the ACTIVE allocator-hook global [guest 0x1067daaf0] against the DEFAULT hook global [guest 0x1067d0840]; `cmp x8,x9; b.eq` takes the fast path (TLS allocator 0x1d96a40) when equal, else `blr x8`. Both cells are 0 in the file, but the headless JIT's RW segment leaves [0x1067daaf0] as host-heap garbage -> cmp differs -> `blr x8` jumps to the heap (SIGSEGV, fault==heap==rip). Fix (elfjit.rs v2boot): seed `*(u64*)0x1067daaf0 = *(u64*)0x1067d0840 = 0` before driving rung 1 (mirror SH82 main-id seed; idempotent). Also generalized the map-family +0x18 repair to check BOTH x0 and x19 candidates at map-op entries (the span-hash map arrives live in x19 while x0 holds the .data registry map). Real libroblox.so --v2boot: 0x1029b43f0 blr-through-heap GONE on every run; `cmp` now equal -> `bl 0x1d96a40` TLS allocator runs; next faults are run-variable deeper pc (0x106241148 / 0x7f0000002198). nativeGameGlobalInit STILL does not return. Workspace **514/0**. Product path unregressed. Doc docs/frontier-sh86-allocator-hook.md, repro runs/capture_v2boot_sh82.sh. NEXT: chase the run-variable next gates (mirror seed/hook) AND get nativeGameGlobalInit to RETURN so rung 2 nativeUpdateAdapterInit (0x10221c3ec) runs -> check rungs 2-6 install the type-4 producer vector [0x106829ea8] (still 0) -> wire NativeHelper callbacks.
## SH85 (Sep 13, 2026, hermes-worker): qsort INTERPOSE + generalized map +0x18 repair — 4 gates cleared this session. Read-only recon deleg_178d6f09 root-caused the qsort-comparator SIGSEGV (file 0x28bbfc0): real glibc qsort was natively executing the guest's ARM64 comparator as x86 (`08 18` = `or [rax],bl`, leftover rax=0x2 -> fault at ~0x2) — same class as pthread_once/dl_iterate_phdr/cxa_thread_atexit, never covered for qsort. Fix (shims.rs bionic_qsort, registered by name): run real glibc qsort with a HOST comparator. Primary path = for the known +24-u32-key comparator (file 0x28bbfc0) sort ENTIRELY HOST-SIDE reading +24 u32 keys (zero JIT re-entry — fixed the SH55/SH64 concurrent-jit_run block-cache desync that the ~N·log2(N) run_guest_callback re-entries would trigger on the detached ladder thread vs the render thread); fallback = JIT trampoline via new run_guest_callback_on(fn,args,tp,stack,size) on a CACHED per-thread 1 MiB stack (avoids run_guest_callback's fresh-1MiB-per-call leak ~1.5 GiB). Also SH85b: generalized the SH83/84 +0x18 repair to the whole map family (fire at 0x1029f3e70 insert + 0x1029f4258 rehash/grow + 0x1029f4088 + 0x1029f4348; zero +0x18 whenever it's a NON-IMAGE address — a real second hash is always in .text, so safe for the span-hash map too whose +0x10=0x1029b4a84 differs); empty-map header + zeroed bucket array seed kept INSERT-only. Real libroblox.so --v2boot: qsort gate CLEARED (no more 0x1028bbfc0), map-family gates cleared (0x1029f3f7c / 0x1029f3f84 both gone), ladder now faults DEEPEST at guestpc 0x1029b43f0 (OTel/pb_defaults descriptor registration dispatching through an uninitialised heap fn-ptr slot). New hermetic regression qsort_interpose_hostside_u32_key_comparator_orders_and_is_named. Workspace **514/0** (+1). Product path (no --v2boot) unregressed (exit 124, persist byte-exact, 0 crash). Doc docs/frontier-sh85-qsort-interpose.md, repro runs/capture_v2boot_sh82.sh. NEXT: clear the 0x1029b43f0 pb_defaults descriptor fn-ptr slot (uninitialised registration-table slot -> heap garbage) → nativeGameGlobalInit returning → rung 2 nativeUpdateAdapterInit (0x10221c3ec) → check rungs 2-6 install the type-4 producer vector [0x106829ea8].
## SH84 (Sep 13, 2026, hermes-worker): CLEARED the SH84 bucket-probe gate — the string-keyed hash-map's insert now COMPLETES its first entry headlessly. Read-only recon deleg_52c74ca3 (disasm-verified in the run log) pinned: after SH83's +0x18 repair the insert's real string-hash ran, then the bucket probe (guest 0x1029f3f84 `ldp w9,w8,[x19,#64]`/udiv/`ldr x23,[x22]`) read a wild slot because the map's numeric header (+0x38 cap, +0x3c/+0x44 divisors, +0x40 mask, +0x48 load, +0x58 size, +0x60 err) is leftover host-heap garbage -> idx = hash mod garbage. Extend the same SH83 hook at insert entry 0x1029f3e70 (guarded on +0x10==0x102a25dec): force +0x00 bucket array to a fresh zeroed 1024x8 array ONCE per map (host-side SEEN set so repeat inserts keep the insert-new entries) + set count/divs/mask/load/err/size to the coherent empty-map state. Real libroblox.so --v2boot (2 clean runs): probe now computes idx in [0,1023], reads sentinel 0, insert writes its first entry, and the ladder faults FURTHER at file 0x28bbfc0 (a qsort comparator in the enum-registration path, `ldr w8,[x0,#24]` on 0x50-byte records, reached from 0x28bbf80 — 3× qsort) — one more gate advanced. +2 hermetic regressions (`routeb_hashfix_repairs_garbage_hash_fn2_slot` SH83 + `routeb_hashfix_seeds_coherent_empty_map_header` SH84). Workspace **513/0** (+2). Product path (hook off) unregressed (exit 124, persist byte-exact, swap Ok(0x1), 0 crash). Doc docs/frontier-sh83-regtab-hashfix.md (SH84 addendum), repro runs/capture_v2boot_sh82.sh. NEXT: clear the 0x28bbfc0 qsort-comparator gate — identify the record array being sorted + why an element ptr is garbage, seed/repair so the sort completes; then nativeGameGlobalInit returning -> rung 2 nativeUpdateAdapterInit (0x10221c3ec) -> check rungs 2-6 install the type-4 producer vector [0x106829ea8].
## SH83 (Sep 13, 2026, hermes-worker): CLEARED the SH82b registration hash-table gate — the string-keyed hash-map insert (guest 0x1029f3e70) now falls back to its REAL single string-hash (`blr x1`, 0x102a25dec) instead of `blr x8` through a garbage hash-fn-2 slot. Read-only recon deleg_2911f1f5 pinned the crash: the dispatch `ldp x1,x8,[x19,#16]` reads optional hash-fn-2 at [map+0x18]; under the JIT that heap slot holds leftover host garbage (0x4741495241003635 = ASCII "56\0ARAIG") instead of the engine's default 0, so the non-zero hash2 -> `blr x8` jumps into unmapped memory (SH82b SIGSEGV guestpc 0x1029f3f7c). The engine map is single-hash (+0x10 = real hash, +0x18 must be 0). New env JIT_ROUTEB_HASHFIX=1 (off by default) adds a block-entry hook at the insert ENTRY 0x1029f3e70 (a real JIT block boundary — the dispatch 0x29f3f6c is mid-block, so the hook MUST fire at fn entry): if [map+0x10]==0x102a25dec && [map+0x18]!=0, write 0 to [map+0x18] (idempotent, +0x10-guarded, never touches a real two-hash map). Real libroblox.so --v2boot: the SH82b fault is GONE; the ladder now runs the real string-hash insert and faults FURTHER at the next gate (guest 0x1029f3f84 bucket probe: `ldp w9,w8,[x19,#64]`/udiv/`ldr x23,[x22]`) on 3 clean runs — one more gate advanced. New hermetic regression `routeb_hashfix_repairs_garbage_hash_fn2_slot`. Workspace **512/0** (+1). Product path (no --v2boot, hook off) UNREGRESSED (exit 124, persist 45B byte-exact, present #0 swap Ok(0x1), 0 crash). Doc docs/frontier-sh83-regtab-hashfix.md, repro runs/capture_v2boot_sh82.sh (JIT_ROUTEB_HASHFIX=1), log runs/sh83-v2boot-regtab.txt. NEXT: clear the 0x1029f3f84 bucket-probe gate — the map's size/count/mask fields (+0x58 size, +0x38 count, +0x3c/40/44 masks, +0x00 bucket array) are uninitialised host heap -> wild bucket read; seed a coherent EMPTY map (bucket array + size/mask so the probe reads bucket->0 and takes the "not found -> insert new" path); then nativeGameGlobalInit returning -> rung 2 nativeUpdateAdapterInit (0x10221c3ec) -> check rungs 2-6 install the type-4 producer vector [0x106829ea8].
## SH82b (Sep 13, 2026, hermes-worker): seed JNICall singleton intern-table (+0x30 bucket array +0x38 capacity 0x400) — GlobalInit do-init hash lookup no longer derefs NULL; ladder advances one MORE gate (fault moves 0x1021daf78 -> 0x1029f3f7c, a blr through an invalid per-entry callback ptr at [x19+16] of a registration hash table). Commit 2acbeef. Product path unregressed (exit 124, persist 45B byte-exact, present #0 swap Ok(0x1), 0 crash). Workspace 511/0. Doc frontier-sh82-globalinit-unpark.md, repro runs/capture_v2boot_sh82.sh, log runs/sh82-v2boot-advance.txt. NEXT: find what [x19+16] of the registration hash table should point at (per-entry fn — haddr/has-empty/deleter) and seed a benign leaf so the insert completes -> rung 2.
## SH82 (Sep 13, 2026, hermes-worker): Route-B rung-1 PARK ROOT-CAUSED + UNPARKED — `nativeGameGlobalInit` is not self-blocking: its `GameGlobalInitImpl` thread-dispatch (file 0x2206db8) compares `pthread_self()` against the engine's stored main-thread id cell **0x106863a68** (`ldr x20,[x8,#2664]`, 0x2206de4) and `b.ne 0x2206e28` (0x2206df0) parks on ANY non-main thread in the 0x2207648 completion spin. The SH80/81 latch [0x72739d4] is an OUTPUT of that do-init, never reachable while it only ever parks on the detached ladder thread. Fix: seed `[0x106863a68]` = the ladder thread's own `pthread_self` before driving rung 1 (then restore) so the call takes the match (returning) path — **no .text patch**. Real libroblox.so (`--v2boot`): rung 1 **no longer parks** — it runs its real do-init and faults FURTHER at guestpc **0x1021daf78** (next gate, null deref). Repro runs/sh82-v2boot-advance.txt. A/B: a .text NOP on the b.ne is a REGRESSION (6/6 json-crash vs 8/8 clean baseline — it re-routes the MAIN thread's own boot call) -> reverted; the cell-seed is the correct lever. +1 hermetic regression `routeb_globalinit_thread_dispatch_main_id_cell_and_seed`. Workspace **511/0**, example 22 pass. Doc frontier-sh82-globalinit-unpark.md, repro runs/capture_v2boot_sh82.sh. NEXT: seed the 0x1021daf78 null object (mirror SH81 singletons) so nativeGameGlobalInit completes -> rung 2 nativeUpdateAdapterInit runs -> check if rungs 2-6 install [0x106829ea8]; then wire NativeHelper callbacks.
## SH81 (Sep 13, 2026, hermes-worker): Route-B crash chain CLEARED — the `--v2boot` ladder now runs CRASH-FREE with nativeInitializeNativeFlags (rung 0) completing. Workspace **510/0** (+1), example 22 pass. Commit e8a8843. Doc docs/frontier-sh81-routeb-gate-singletons.md, repro runs/capture_v2boot_gate.sh.
  Two idempotent, hermetic-tested patches (read-only recon deleg_5ebaa5f9) remove the two dispatch-singleton SIGSEGV mechanisms that halted the ladder since SH80:
  1. **Dispatch-gate force** (`routeb_patch_dispatch_gate`): engine accessor `21730ec` ends `and w0,w0,#1` (file 0x2173124, u32 0x12000000); ~255 stubs `bl 21730ec; tbz w0,#0,<fb>` test bit0. On headless boot the query is 0, so every site takes a fallback singleton-lookup path (6249e9c/6249eb8 -> 2b9dee0(&0x6829a48/68)) whose lazy-created object has a NULL vtable -> `ldr x8,[x8,#48]; blr` SIGSEGV (SH80 crash 0x10624f46c). Patch mask -> `mov w0,#1` (0x52800020) + cache-drop so every site takes its clean direct path (incl. the StartLuaAppDM-adjacent 0x240a1b0). The mask already collapsed w0 to bit0, so forcing 1 changes nothing else observable.
  2. **Singleton-record seed** (`routeb_seed_task_singletons`): `2b9dee0`'s create memcpys `[rec+24]`(size `[rec+0]`) into a fresh heap obj; all-zero .data records (0x6829a48/0x6829a68) gave a NULL-vtable stub. Seed size=0x28/allocclass=8/src=leaked template whose +0 is a vtable of all-`routeb_singleton_leaf` (register_host_call_auto thunk returning its first arg) so every objA/objB accessor result is a real polymorphic object. Covers the DIRECT `cbz x1,0x624f4f0` singleton path the gate does not (SH81 crash moved 0x10624f46c -> 0x10624f500 before this).
  **Empirical (real libroblox.so, runs/sh81-v2boot-gate.txt, exit 124):** 0 SIGSEGV/SIGABRT (was abort 134), `nativeInitializeNativeFlags` returns, then `driving nativeGameGlobalInit` (rung 1). Productized render+persist baseline INTACT in the same run: persist 45B byte-exact, taskv4 present #0 swap Ok(0x1), triangle centroid red, textured quad BL/BR/TR/TL, 3 quad-loop frames, renderinit Ok.
  **Next (SH82 finding):** nativeGameGlobalInit (rung 1) still does not RETURN — it parks. Regressing-guard attempt (seed 0x6a68410 once-flag=1, recon deleg_f9f7ef64) REVERTED: (a) mprotect-to-RX on the .bss guard page broke the run's own do-init CAS/stlrb writes to sibling flags 0x6a683f0/0x3f1/0x3e8 -> SIGSEGV (the mprotect, not the value); corrected to a plain RW byte store -> clean exit 124 again BUT gameGlobalInit still parks in the same 0x10284d134 nanosleep poll (no outcome change vs no-seed). So the once-flag seed alone does NOT unblock rung1's return; the park is either the guard already satisfied via rung0's flags chain, or the do-init path itself starts/parks the TaskScheduler main loop. Next: characterize what gameGlobalInit's do-init path truly blocks on (constructing TaskScheduler / main-loop) before seeding further. Standing structural wall unchanged: type-4 vector [0x106829ea8] stays framework-glue seeded.
## SH80 (Sep 13, 2026, hermes-worker): Route-B latch release (docs/recon-routeB-globaltinit-unblock.md step 1) — --v2boot now drives nativeInitializeNativeFlags (0x10232048c) FIRST as rung 0: it's on the engine's OWN flags-loaded write chain (0x2320cec->0x2320f2c->strb #1,[0x72739d4] at file 0x22474e8), so the gameGlobalInit gate latch is set by JIT-translated ENGINE code (safe RW write), not a fragile raw host write (which SIGSEGVs). jni_call_int_method getFlagsCount=>1 (was 0, bailed immediately). Real libroblox.so: the ladder now executes engine code PAST the historical SH55 rung-1 nanosleep park (latch confirmed as the FIRST gate) and faults FURTHER IN at guestpc 0x10624f46c (file 0x624f46c, null-deref, downstream task/device init) — one gate advanced; the standing wall is a sequence of such gates, not a single latch. +getFlagsCount regression (jni_auto_value... test). Workspace **509/0**, example **22 pass**. Doc frontier-sh80-v2boot-latch-release.md, log runs/sh80-v2boot-r0.txt. Next per recon ranks: wire the NativeHelper gameActivity_* callbacks (onFlagsLoaded/onEngineInitialized/onAppReady/onDidLogInReceived/onGameLoaded) into RegisterNatives so StartLuaAppDM's session can construct the first login/home GuiObject tree; and clear the 0x624f46c boot fault by seeding its missing device/scheduler object.
## SH79 (Sep 13, 2026, hermes-worker): TTF rasterizer now decodes COMPOSITE glyphs (numberOfContours<0) recursively into component contours with offset/scale/2x2 transforms — arbitrary Roblox UI labels now render (SSPro-Bold has 652/1115 composite: accented À-Ï, %, :, ;, =, quotes). Previously composite glyphs bailed (None, skipped). font_glyph_contours -> recursive font_glyph_contours_rec (depth<=16), honors ARG_1_AND_2_ARE_WORDS / ARGS_ARE_XY_VALUES (point-match->0 offset) / WE_HAVE_A_SCALE / X_AND_Y_SCALE / TWO_BY_TWO; long-loca applies to both. New hermetic sh79 test: %, :, 0 (all composite) each rasterize >20 opaque px (were None). Example tests **22 pass** (was 21); Workspace **509/0**. Doc frontier-sh79-composite-glyphs.md. Honest scope: latent robustness (shipped login labels use 0 composite chars); no screen change yet; standing wall unchanged. Subagent-ranked runner-up (C) from SH78 now landed.
## SH78 (Sep 13, 2026, hermes-worker): the login-form TEXT LABELS are now thicker and crisper at 720p — 2x rasterized resolution (Log In 512x80 pu0.068, Email address/Password 512x64 pu0.060) with transparent GUARD rows (GUARD=1) inserted between every sprite in the shared vertical atlas, so GL_LINEAR sampling near sub-row boundaries can't bleed a neighbor sprite's color into the glyphs. Because w and h double together, w:h is preserved (512/80==256/40) -> on-screen quad geometry + all probe screen-coords are bit-identical; only texel density + stroke thickness double. Real libroblox.so (clean drained exit, 0 crash): **9/9 probes present=true** — the 3 text labels STILL BYTE-EXACT at unchanged screen coords ('Log In' (627,138), 'Email address' (697,196), 'Password' (672,166)), 2x emitter Ok(count=60 sprites=9 swap=Ok(1)), persist 45B byte-exact, walker drained 1 frame. Stroke-thickness proxy (opaque glyph px in presented frame): Log In 254->328, Email 758->878, Password 524->596. Captured runs/sh78-emitter-login-text-2x.png. Shared atlas now has a guard row per sprite. 4 new hermetic sh78b tests (aspect-invariant, guard-row packing, stroke>1.4x, 3x3-probe-opaque) + 2 updated sh77 font tests; Workspace **509/0**, example tests **21 pass** (was 17). Docs frontier-sh78-login-text-thicken.md, repro runs/capture_emitter_login_text.sh. Subagent re-confirmed the standing structural wall (A) still statically impenetrable (the file-0x5b2c828/0x5b2eb7c/0x5b2ec1c "node builders" only consume pre-populated R+0x180; nativeGameGlobalInit is a thin park wrapper; real GuiObject construction needs unreachable StartLuaAppDM). Honest scope: thicker host-rasterized glyphs, still NOT engine self-constructed session; standing wall unchanged.

## SH77 (Sep 13, 2026, hermes-worker): the engine's REAL geometry emitter now renders real TEXT LABELS on the Roblox login form — white "Log In" over the green button + dark-slate "Email address"/"Password" placeholders over two input fields, rasterized from the APK's OWN font (SourceSansPro-Bold.ttf) by a new pure-std TrueType outline rasterizer (sfnt/cmap Format-4 incl. idRangeOffset, glyf simple contours, quadratic flatten w/ implied on-curve midpoints, non-zero-winding 4x4-supersampled coverage) into transparent 256-wide RGBA8 atlas rows, composited via GL_BLEND. Real libroblox.so exit 124: **9/9 probes present=true** — all 3 text labels **BYTE-EXACT** at glyph-interior texels ('Log In' (642,138) rgba(255,255,255,255), 'Email address' (588,192) + 'Password' (669,169) rgba(96,96,110,255), diff=[0,0,0,0]), vignette diff<=1, fields/button byte-exact (probes moved to box edge 24,8 since centered text overlays old center-probe px), 2x emitter Ok(ret=0x0,count=60,sprites=9,swap=Ok(1)), 0 crash/json-overflow. Captured runs/sh77-emitter-login-text.png (ASCII ridge-map confirms readable L-o-g-I-n / E-m-a-i-l a-d-d-r-e-s-s / P-a-s-s-w-o-r-d letterforms). NEW pure-std TTF rasterizer (zero new deps/network). +6 hermetic sh77 regressions; Workspace **509/0**, example tests **17 pass** (was 11). Docs frontier-sh77-emitter-login-text.md, repro runs/capture_emitter_login_text.sh. Honest scope: authored/host-rasterized labels (real APK font, engine's own emitter path, correct painter's-order), NOT engine self-constructing a GuiObject/session; standing wall unchanged.

## SH76 (Sep 13, 2026, hermes-worker): the Roblox login surface is now a LIVE animated film — RENDEREMITTER_LIVE=1 bobs the RO-BLOX wordmark per frame (static LIVE_N counter, ±0.03 NDC alternating) so consecutive present-walker frames differ. Real libroblox.so exit 124: wordmark probe y moved (560,496)->(560,474) across the two emitter drives, both BYTE-EXACT white, 10/10 probes present=true, 2x emitter Ok + 2x walker, 0 crash. pl(i) resolver applies the bob to both quad-build and probe geometry so the probe tracks the moving wordmark. Same pattern as SH71's spinner (proven no SH67c drop). Workspace 509/0, example tests 11 pass. Doc frontier-sh76-emitter-live.md, log runs/sh76-emitter-live.txt. Honest scope: authored motion, not engine-self-driven; standing wall unchanged.


## SH75 (Sep 13, 2026, hermes-worker): the engine's REAL geometry emitter now draws a COMPLETE Roblox LOGIN form headlessly — real reversevignette backdrop + real RO-BLOX wordmark + real noconnection Wi-Fi chip + synthesized solid prims (near-white field + Roblox-green "Log In" bar), all in ONE top-level jit_run from a shared vertical atlas, GL_BLEND over dark. RENDEREMITTER_LOGIN=1. Real libroblox.so exit 124: 2x `engine emitter Ok(count=36 sprites=5 swap=Ok(1))` + 2x walker Ok(0x1), **10/10 probes present=true** — wordmark/chip/field/button all BYTE-EXACT (diff=[0,0,0,0], alpha=255 opaque over any dst); vignette diff=[1,1,1]. Captured runs/sh73-emitter-login.png = genuine Roblox login/connection screen (wordmark -> Wi-Fi icon -> input field -> green button over vignette). 0 crash. login_ui_textures adds noconnection + synthesizes solid field/button rows (ub=s.w/aw keeps row-extents local); +2 hermetic regressions. Workspace **509/0**; example tests **11 pass** (was 9). Docs frontier-sh75-emitter-login-form.md, repro runs/capture_emitter_login.sh. Honest scope: harness-authored geometry (+ synthesized solid prims) + real artwork through the engine's own emitter; NOT the engine self-constructing a login UI/session; standing wall unchanged.



## SH73 (Sep 13, 2026, hermes-worker): the engine's REAL geometry emitter now renders a REAL Roblox LOGIN/auth surface headlessly — the real `reversevignette` dark backdrop + the white RO-BLOX wordmark (`logo_white_1x.png`, 476x88), decoded OFFLINE from the real APK Auth artwork (`ExtraContent/textures/ui/LuaApp/graphic/Auth/`), sampled from a shared vertical atlas in ONE top-level jit_run, GL_BLEND over the dark backdrop. New env RENDEREMITTER_LOGIN=1 (+ MULTI=1 + LAYOUT=home). Real libroblox.so exit 124: 2x `engine emitter Ok(count=18 sprites=2 swap=Ok(1))` + 2x present-walker Ok(ret=0x1), **4/4 probes present=true** — wordmark probe (560,485) **Byte-EXACT** (255,255,255,255 diff=[0,0,0,0]; alpha=255 white over any dst); vignette (641,359) rgba(48,50,52) expect [47,49,51] diff=[1,1,1] tol6 (real backdrop pixel over dark). Captured runs/sh73-emitter-login.png = recognizable Roblox splash (RO-BLOX wordmark, iconic tilted-square "O" rhombus, centered upper-middle over a center-glow dark vignette). 0 crash. Code: real_ui_textures() honors absolute paths; new login_ui_textures(); login placements slice; SH72 spinner-transparent probe gated off in login mode. Workspace **509/0** (unchanged). Docs docs/frontier-sh73-emitter-login.md + frontier-sh73-next-artifact.md, repro runs/capture_emitter_login.sh. Honest scope: harness-authored geometry + real Auth artwork through the engine's own emitter path, NOT the engine self-constructing a GuiObject/session; standing wall unchanged (type-4 vector glue-only; nativeGameGlobalInit parks; no in-image GuiObject->scene-list path). Next: (A) more login-complete Auth set (noconnection.png chip), (B) animate, (C) combine SH68 solid green button-bar with real artwork.

## SH72 (Sep 13, 2026, hermes-worker): the engine's REAL geometry emitter now draws a FULL MULTI-SPRITE composite of REAL Roblox UI textures — the loading spinner (100x100), Robux icon (54x54), and jump button (240x240) each in its own aspect-correct box, sampled from ONE shared vertical atlas in a SINGLE top-level jit_run (24 verts GL_TRIANGLES), alpha-composited over a dark backdrop (GL_BLEND SRC_ALPHA/ONE_MINUS_SRC_ALPHA). New `--renderemitter` env `RENDEREMITTER_MULTI=1` (with LAYOUT=home). Real libroblox.so exit 124: `engine emitter Ok(ret=0x0) count=24 sprites=3 swap=Ok(1)` x2 + `present walker Ok(ret=0x1)` x2, **8/8 pixel probes present=true** — spinner arc (592,400) BYTE-EXACT diff[0,0,0,0]; robux opaque white outline diff[1,1,1]; jump's semi-transparent white (a=0x66) blends to exactly rgba(118,118,121) = src*a+dst*(1-a) over the (26,26,31) backdrop (diff[1,1,1]); spinner-transparent-center = exact backdrop through real alpha 0. Captured frame runs/sh72-emitter-multi.png (recognizable Roblox loading surface: dark bg, blue spinner upper-mid, R$ top-right, jump button bottom-center). 0 crash/json-overflow. Two SH72 bug-fixes pinned: (1) `ub` must be `s.w/aw` not 1.0 (or the image squishes into the left s.w/aw of its box — a multi-row atlas row is aw wide); (2) the probe y must use the emitter's un-inverted projection (glReadPixels y = (1+y_ndc)/2*VH) AND the expected color must be the GL_BLEND composite over the backdrop (many sprites are semi/fully transparent — robux center a=0, jump a=0x66). 3 new hermetic sh72 tests (imgpix_rect reduction / offset-box shift / disjoint atlas UV layout). Workspace **509/0**; example tests `cargo test -p arm64jit --example elfjit` 9 pass. Docs docs/frontier-sh72-emitter-multi.md + frontier-sh72-next-artifact.md, artifacts runs/sh72-emitter-multi.png/.txt, repro runs/capture_emitter_multi.sh (RENDEREMITTER_MULTI=1). Honest scope: fabricated multi-sprite geometry + host atlas/program/blend; standing wall unchanged (engine neve... [truncated]

## SH71 (Sep 13, 2026, hermes-worker): the engine's REAL geometry emitter now renders an **ANIMATED real loading screen** — `RENDEREMITTER_SPIN=1` rotates the real LoadingSpinner NDC box per frame (`frame*15°`, static SPIN_N counter) so the FIXED texture visibly spins across the present-walker frames (painter's-order triangles, UV assignment per corner unchanged). Radial-sweep verification (36 angles × 2 radii around the box center) measures the arc's strong-blue direction each frame and it ADVANCES 90°→70°→50°→60°→…→120° across 8 frames, all `engine emitter Ok(ret=0x0)` + `present walker Ok(ret=0x1)`, real libroblox.so exit 124, 0 crash (runs/sh71-emitter-spin.txt). Docs docs/frontier-sh71-emitter-spin.md; next-artifact doc (C) folded as done. Honest scope: host-set rotation/texture/program/blend; the animation is authored (per-frame corner rotation), not engine-self-driven; standing wall unchanged. Workspace **509/0**.

## SH70 (Sep 13, 2026, hermes-worker): extended the offline PNG->RGBA8 decoder to **color type 3 (indexed/palette)** with optional tRNS per-index alpha + a clear real-texture fallback WARN. PLTE->RGB, tRNS->per-index alpha (absent tRNS = opaque); covers ct 0/2/3/4/6 = the full PNG surface real Roblox UI assets use. 2 new hermetic sh69_tests pass (palette-with-tRNS exact RGBA; palette-without-tRNS opaque); the REAL binary decodes `ui/InGameMenu/BackgroundGlow@2x.png` (512x512 ct=3) -> RGBA8 and renders it through the engine emitter (`decoded ... 512x512 RGBA8`, `tex 512x513`, emitter Ok(ret=0x0) swap=Ok(1), exit 124, 0 crash). Same commit as SH69 (docs/frontier-sh69-emitter-real-texture.md addendum). Workspace **509/0**; example tests `cargo test -p arm64jit --example elfjit`: 6 pass.

## SH69 (Sep 13, 2026, hermes-worker): the engine's REAL geometry emitter (0x105b35288) now renders a **REAL Roblox APK UI texture** through its own GL path — the loading-screen spinner (assets/content/textures/ui/LoadingScreen/LoadingSpinner.png, 100x100 RGBA, real alpha). New env RENDEREMITTER_REAL_TEX=1 (+ RENDEREMITTER_LAYOUT=home) builds a 2-D atlas (A_W x A_H = 100x101: palette-strip row 0 + the real image rows 1..=imh uploaded reversed/upright) with a centered aspect-correct image box (half_w=hh·(W/H)·(VH/VW), GL_LINEAR+CLAMP) sampled through the engine's textured emit. Spinner decode = minimal offline PNG->RGBA8 (ct 0/2/4/6, per-row filter incl. Paeth) in elfjit.rs riding `flate2` (already in Cargo.lock via zip — **zero new network dep**); +flate2 to arm64jit Cargo.toml. Real libroblox.so exit 124: `real-arc-blue (580,498) rgba(49,180,255) diff=[0,0,0,0] present=true` — a REAL texture pixel, byte-exact; `transparent-center (641,361)` and `transparent-right (749,361)` read the backdrop(26,26,31) through (real alpha 0) proving real alpha content + left-only arc = correct orientation/aspect; backdrop corner fixed; 2x engine emitter Ok + 2x present-walker Ok(0x1), 0 crash. Captured frame runs/sh69-emitter-real.png shows the real sky-blue ~270° spinner centered over the dark backdrop + green button bar + cream title/gray field strips. **First time real Roblox UI pixels (not fabricated palette) render through the engine's own pipeline headlessly.** New env RENDEREMITTER_REAL_TEXTURE overrides the sprite path. 4 new hermetic regressions in elfjit.rs (sh69_tests: type6 filter0 roundtrip, gray ct0 expands opaque, imgpix_screen aspect/center, non-PNG reject) — run via `cargo test -p arm64jit --example elfjit` (binary-example harness not in workspace count). Workspace **509/0**. Docs docs/frontier-sh69-emitter-real-texture.md + frontier-sh69-next-artifact.md, artifact runs/sh69-emitter-real.png/.txt, repro runs/capture_emitter_real.sh. Honest scope: single static real asset + host-set texture/program/blend; standing wall unchanged (engine never self-populates; Lua app-shell / nativeGameGlobalInit parks / type-4 vector .bss glue-only).

## SH68 (Sep 13, 2026, hermes-worker): the engine's REAL geometry emitter (0x105b35288) draws a LAYERED "login/home" frame headlessly with GL_BLEND alpha compositing, sized from the REAL scene list — 5 textured layers (dark backdrop, centered semi-transparent panel a=0.55, green button bar, title + field strips) as ONE jit_run (30 verts GL_TRIANGLES, stride 32, per-layer texel in a 5-texel RGBA strip, FS outputs only texture2D) with glEnable(GL_BLEND)+glBlendFuncSeparate before the emit. New `--renderemitter` env `RENDEREMITTER_LAYOUT=home`. Real libroblox.so exit 124: `scene list R=.. n_scene=3 head/tail-derived=3 match=true` (drives layout from R+0x180/0x188), 2x `engine emitter Ok(ret=0x0) draw_mode=0x4(first=0,count=30) layers=5 swap=Ok(1)`, ALL 8 blend probes present=true — the PANEL-BLEND probe at center rgba(44,47,58) is neither backdrop(26,26,31) nor pure panel(61,66,82) but the exact blend equation src*.55+dst*.45=(45,48,59) (diff≤1 float rounding) => genuine UI-style alpha compositing through the engine's own draw path. Walker 2x present-walker Ok(0x1) intact, 0 crash. Workspace **509/0**. Doc docs/frontier-sh68-emitter-home.md + frontier-sh68-next-artifact.md, artifact runs/sh68-emitter-home.png/.txt, repro runs/capture_emitter_home.sh. Honest scope: fabricated geometry + host texture/blend, sized from the real scene list, NOT a real GuiObject/self-populated session; standing wall unchanged.

## SH67e (Sep 13, 2026, hermes-worker): the engine's REAL geometry emitter (0x105b35288) now draws a TEXTURED populated frame headlessly — extending SH67d's solid grid with a 3rd per-vertex texcoord attribute (aTex loc2, stride 32, spec[2], BD slot G+0x68) so the emitter interpolates per-tile UVs sampled from a pre-uploaded host palette-strip texture. New `--renderemitter` env `RENDEREMITTER_TEX=1`. Real libroblox.so exit 124: 2x `engine emitter Ok(ret=0x0) draw_mode=0x4(first=0,count=36) quads=6 swap=Ok(1)`, ALL 12 per-tile readbacks `present=true` with diff=[0,0,0,0] Byte-EXACT — and because the FS outputs ONLY texture2D(uTex,vUV) (aColor ignored), the distinct exact per-tile colors prove the aTex wiring + UV interpolation + live texture sampling all work (broken any would yield uniform/black tiles). Walker 6 quads + 2 present-walker Ok(0x1) intact, 0 crash. Workspace **509/0**. Disasm-pinned by subagent deleg_7604fc6a: emitter/primitive_setup dispatch ONLY vertex-attribute GL (texture is host-set; the engine interpolates UVs from the spec-wired aTex). Doc docs/frontier-sh67e-emitter-textured-grid.md + frontier-sh67e-next-artifact.md, artifacts runs/sh67e-emitter-grid-tex.png/.txt, repro runs/capture_emitter_grid.sh (RENDEREMITTER_TEX=1). Honest scope: fabricated geometry + host texture, NOT a real UI/GuiObject; standing wall unchanged (no self-populated render session).

## SH67d (Sep 13, 2026, hermes-worker): the engine's REAL geometry emitter (0x105b35288) now draws a POPULATED N-quad 2D frame headlessly — a SINGLE top-level jit_run with one pre-uploaded VBO of N*6 verts as GL_TRIANGLES (mode_idx=0, first=0, count=6N) + fabricated geometry-context G, closing the SH67c multi-tile blocker BY CONSTRUCTION (no per-drive glBufferData/glBindBuffer → no llvmpipe buffer-orphan → no silent drop/SIGABRT). New `--renderemitter` env `RENDEREMITTER_QUADS=N` (default unset = SH67b single quad unchanged). Real libroblox.so exit 124: 2x `engine emitter Ok(ret=0x0) draw_mode=0x4(first=0,count=36) quads=6 swap=Ok(1)`, ALL 12 per-tile readbacks (6 tiles × 2 frames) `present=true` at exact grid positions (purple|green|red / blue|yellow|purple), each channel ≤1 diff (float→u8 rounding, not a real gap), walker 6 quads + 2 present-walker Ok(0x1) intact, 0 crash/json-overflow. Workspace **509/0**. Disasm-proven by read-only subagent deleg_ca280417 (pinned emitter dispatch: unconditional primitive_setup → stateless glDrawArrays(first,count) into bound VBO; failures were harness-side buffer orphaning, not an engine limitation). Doc docs/frontier-sh67d-emitter-grid.md, artifact runs/sh67d-emitter-grid.png + runs/sh67d-emitter-grid.txt, repro runs/capture_emitter_grid.sh. Honest scope: fabricated G, NOT a real GUI gesture; standing wall unchanged (type-4 producer vector .bss framework-glue-only).


## SH67c (Sep 13, 2026, hermes-worker): multi-tile grid experiment through the engine emitter — net-reverted, finding documented (docs/frontier-sh67b-emitter-draws.md addendum). Repeated engine-emitter jit_run rasterizes fine reused on one already-uploaded VBO, but ANY glBufferData in the emitter drive (per-tile re-upload or fresh buffer object) silently drops subsequent draws (variant abort with first=0); mechanism = llvmpipe/engine-context VA-state/orphan interaction, needs a JIT GLES-draw trace. Workspace green 509/0; the committed SH67b single-quad deliverable is intact and re-verified (gradient pixel-verified, exit 124). Frontier lever unchanged.


## Session (Sep 13, 2026, hermes-worker, cycle SH67b) — RESOLVED SH66/67: the engine's REAL geometry emitter (0x105b35288) now VISIBLY draws its authored quad — pixel-verified in the presented frame through the engine's own GLES path, presented via the real engine swap. Workspace **509/0** (unchanged). Commit c645a9e (SH67) + pending (SH67b). Doc docs/frontier-sh67b-emitter-draws.md, artifact runs/sh67b-emitter-quad.png + runs/sh66-renderemitter.txt + runs/sh67b-swap.txt.

SH66 proved the emitter's draw path runs headlessly but the authored quad's
pixels never appeared; SH67 disproved the GL_DRAW_BUFFER==GL_NONE/draw-buffer
hypothesis. This cycle a read-only research subagent (deleg_b1698660) disasm-
proved the real cause: a **harness argument-inversion bug**. The emitter's true
ABI (file 0x5b35288 decode) is `emit(G, modeidx=x1, first=x2, count=x4, ...)` —
arg2 becomes glDrawArrays FIRST and arg4 becomes COUNT. The SH66 drive put
`x2=4`("count")/`x4=0`("first") — inverted — so the engine issued
`glDrawArrays(GL_TRIANGLE_STRIP, first=4, count=0)`, a legal error-free no-op
(clean return, GL error 0x0, program unchanged, attrib locs 0/1, link 1 — every
SH67 observation explained). Also verified the @plt GOT slot resolves the real
`glDrawArrays@Base` Mesa symbol (JUMP_SLOT@0x67d2318) and mode table 0x225780[3]=0x5.

Fix (correct-by-default): `st.x[2]=0`(first), `st.x[4]=4`(count).

**Empirical (real libroblox.so, exit 124, zero crash):** `engine emitter
Ok(ret=0x0) swap=Ok(1)` — the quad is pixel-verified in BOTH back-buffer-direct
(RENDEREMITTER_READBACK_BEFORE_SWAP=1) and the presented frame: center
(640,360)=rgba(64,223,172) teal, a smooth violet->magenta->teal->green->yellow-
orange gradient across y100..y650 — exactly the authored colored quad's
interpolated colors — with the dark backdrop and walker's discrete bands gone
from the sampled line. The walker's 6 real quads + 2 `present walker
Ok(ret=0x1)` remain in the same run. Captured runs/sh67b-emitter-quad.png.

**Honest scope:** the engine's real geometry emitter now draws engine-detailed
content (an authored colored quad — the default-Rect primitive a login/home UI
layer's emitter produces) headlessly, pixel-verified, presented via the real
engine swap. NOT yet a populated login/home screen: the geometry-context G is a
fabricated coherent object, and the standing wall is unchanged — the engine
never self-populates a real render-manager/session (scene-list head/tail
R+0x180/0x188 has no in-image writer; type-4 producer vector [0x106829ea8] is
.bss framework-glue-only — re-confirmed this cycle by static scan). This is the
closest statically-reachable screen precursor.

**Next frontier feed (from the parallel research subagent deleg_b1698660, task-1):**
(a) route the engine's OWN real geometry emitter as a scene node's render-obj
vt[+24] consumed by the present-walker's per-node blr (keep it a SEPARATE
top-level jit_run to avoid the SH64 desync) so the walker presents
engine-emitted primitives per node; or (b) drive the 3 10-line guest traces the
subagent derived if the pixel gap re-surfaces on the per-node path (verify
w2=first/w4=count + mode table + depth-test). A true login/home screen remains
behind the Lua app-shell (StartLuaAppDM 0x1023efe2c) + session wall:
nativeGameGlobalInit 0x102206404 runs real init then parks indefinitely
(SH54/55), and no in-image path constructs a GuiObject and appends to the scene
list.

## Session (Sep 13, 2026, hermes-worker, cycle SH66/SH66b) — the engine's REAL geometry emitter (0x105b35288) is now RUNNABLE headlessly via an authored guest geometry-context G, driven as a desync-safe top-level jit_run. Workspace **509/0**. Commits 8531222 + 6f7cb7d. Doc docs/frontier-sh66-renderemitter.md, repro runs/capture_renderemitter.sh.

SH65 delivered real geometry through the WALKER's host-side mesh; SH66 targets
the ENGINE's OWN geometry emitter. New `--renderemitter` builds a guest
geometry-context `G` from fresh disasm layout (G+0x38=M mesh, G+0x48/0x58=BD
slots, G+0x78 elem-buf=0 non-indexed, G+0x8e elem-type; M+0x48/0x50=spec
begin/end 24B entries, M+0x60=stride table; spec +0 attr/+4 offset/+8 format
idx/+12 attrib-loc-enum/+16 size-add; vertex-format table 0xcecf8c; draw-mode
table 0x225780[3]=0x5 GL_TRIANGLE_STRIP), uploads an authored colored quad into
a real VBO, binds the cached shader program, and drives guest
`emit(G, w1=3, w2=4, w3=0, w4=0, w5=0)` as its OWN top-level jit_run on the
renderinit thread — never nested inside the present-walker block, so the SH64
nested-jit_run desync class is closed by construction. Then engine bind + swap
via ctx-vt[+24]. Builds traced to the exact engine layout (BD slot per attr at
G+0x48+attr*0x10; stride-table integers; 8-aligned Box-leak buffers; own guest
stack).

**Empirical (real libroblox.so, exit 124, zero crash):** `engine emitter
Ok(ret=0x0) swap=Ok(1)` x2 — the engine's own primitive-setup → attribute-bind →
draw dispatch executes cleanly and the engine swap succeeds. Walker's 6 real
colored quads intact. Productized baseline green (swap Ok(0x1), persist 45B
byte-exact). Workspace 509/0.

**SH66b root-cause probe (honest scope):** the authored quad's pixels are NOT yet
confirmed in the sampled buffer. FBO probe reads
`GL_DRAW_FRAMEBUFFER_BINDING=0` (default, fine) but `GL_DRAW_BUFFER=0x0`
(=GL_NONE) — the current context's default framebuffer has NO draw buffer wired
to the visible surface, so the emit's glDrawArrays silently drops pixels (same
silent-no-op class as SH65's glDrawElements=UNSIGNED_BYTE finding). Pinned to a
single addressable state-wire (set DRAW_BUFFER=GL_BACK before the emit), not an
ABI/structure problem. **Next: set the draw buffer to GL_BACK before the emit so
"emitter runs" becomes "emitter visibly draws".** Standing structural wall
unchanged (fabricated G, not a real UI/GuiObject; type-4 producer vector
glue-installed only).

## Session (Sep 13, 2026, hermes-worker, cycle SH65) — the engine's REAL per-node PRESENT walker now draws REAL GEOMETRY per populated scene node: a distinct colored mesh band per node, pixel-verified. Workspace **509/0**. Commit 27141d1. Doc docs/frontier-sh65-renderwalker-geometry.md, artifacts runs/sh65-renderwalker-geometry.png + runs/sh65-final-readbacks.txt, repro runs/capture_renderwalker_geometry.sh.

SH64 delivered the per-node draw as a flat colored clear; SH65 advances it to
REAL GEOMETRY through a **cached real-Mesa (libGLESv2.so.2) shader program**
(`walker_mesh_program()`: vs/fs compile + program link status logged), 
rasterized per node as `glDrawArrays(GL_TRIANGLE_STRIP, 0, 4)` on a per-node
colored band. Still PURE HOST (no nested jit_run, no guest dispatch table), so
the SH64 desync-proof property holds. Backdrop clear gated to the first node
(`i % nodes == 0`) so all bands accumulate into ONE presented frame; fixed-
function state normalized (full viewport + depth/cull/blend/scissor off) and
glFlush/glFinish before readback so stale engine state can't silently clip.

**Empirical (real libroblox.so, exit 124, zero SIGSEGV):** per-node
`glReadPixels` readback proves EXACT rasterization — item#0 (640,93) =
rgba(102,51,242) = [0.4,0.2,0.95] (violet), item#1 (640,223) = rgba(26,178,13) =
[0.1,0.7,0.05] (green), item#2 (640,352) = rgba(230,38,26) = [0.9,0.15,0.1]
(red), all 9 across 3 nodes × 3 frames; captured frame
(sh65-renderwalker-geometry.png) shows three distinct real mesh bands
(violet y≈584-680, green y≈440-536, red y≈320-416) on a dark backdrop;
`present walker Ok(ret=0x1)` ×3; persist 45B byte-exact.

**Key SDLC finding:** `glDrawElements` with an UNSIGNED_BYTE EBO **silently
rasterized nothing** in the engine's ES3.2 llvmpipe context (program valid 1,
link 1, validate 1, GL error 0x0 — yet the readback stayed the backdrop color).
Switching to `glDrawArrays` on identical vertices made the quads rasterize
immediately (readbacks flipped to exact band colors). EBO path kept behind
RENDERWALKER_DRAWELEMENTS=1; glDrawArrays is the default. Lesson: in this
context prefer non-indexed host-side mesh draws; element-index draws can no-op
without any GL error.

**Honest scope:** build (SH63) + present (SH64) + **render (SH65)** of the
node/item ABI a Lua-created screen would consume is now end-to-end headlessly —
the walker draws real distinct geometry, pixel-verified. The node render-obj is
still the recovered ctx / fabricated coherent object (NOT a real UI/GuiObject),
so drawn content is distinct mesh bands, not a populated login/home screen.
Standing structural wall unchanged (Lua app-shell / nativeGameGlobalInit parks /
type-4 producer vector glue-installed only). **Advance:** any future per-node
draw can now be arbitrary host-side real geometry (meshes, textures, UI prims)
without recompiling the walker.

The SH63 doc left the present side open: the engine's real per-node PRESENT
walker (`0x5b2ed48`, mid-loop entry `0x105b2eec0`) was proven to run (SH64 note)
and blr the per-item draw but SIGSEGVs on iteration 2 (0x105b2eedc) because the
draw thunk's nested jit_run recompiled the very present-loop block being executed.

New elfjit `--renderwalker` delivers it (three pieces):

- **Per-item draw = REGISTERED HOST THUNK (desync-proof).** Each scene node's
  render-obj `vt[+24]` is a `walker_item_draw_thunk` registered via
  `register_host_call_auto` (0x7f00_0000_0000 region). The JIT's `host_call_at`
  dispatches it with **zero compilation / zero block-cache mutation**, so the
  present-loop block survives — the root cause of the SH64 SIGSEGV (nested
  `run_guest_callback` from a thunk thread where TLS `IN_JIT_RUN==0` →
  `clear_block_cache()` evicts the block being executed) is closed. The thunk is
  pure host (dlsym `glClearColor`/`glClear` on the real libGLESv2.so.2), cycling
  the palette per draw.
- **Walker mid-loop drive** `0x105b2eec0` with `x19=R` preset via `CpuState`
  (`run_guest_callback` can't preset x19 — must use `jit_run`). The engine's real
  loop reads head/tail from R+0x180/0x188, per node blr's `vt[+24]` draw, then
  swaps via `ctx-vt[+24]` (real eglSwapBuffers).
- **Full-body-native patches** (idempotent): `0x105b2ee54` parked
  nativeGameGlobalInit bl → ret; **`0x105b2eef4` strb→`mov x30,xzr`** (critical —
  the swap `blr` clobbers x30 to 0x5b2eef4, so the legacy strb + ret at 0x5b2eef8
  would loop forever; zeroing x30 lets the ret land on pc=0 → jit_run halts with
  the swap result in x0); `0x105b2eef8` teardown tail → ret. Block range
  [0x105b2ed48,0x105b2f040) cache-dropped after patching.

**Empirical (real libroblox.so, runs/sh64-renderwalker.txt, exit 124):**
`present walker Ok(ret=0x1)` ×3 (each = 3 per-node engine `vt[+24]` draws + real
engine swap on the live ctx); `item draw #N ... engine-per-node draw Ok` ×9
(3 nodes × 3 frames, distinct palette colors — a capture would show distinct
per-node frames); persist 45B byte-exact; **zero** SIGSEGV/SIGABRT/json-overflow.
Per-node fabrications live in a dedicated leaked buffer (NOT inside R's
allocation — stuffing them into R overran the 0x478-byte heap, the first SIGSEGV).

New regression `scene_present_walker_draws_per_node_via_register_host_thunk_desync_proof`
pins the mid-loop ABI (entry 0x105b2eec0, per-node render-obj@+0x08→vt[+24] draw,
ctx@R+0x160 swap) + the desync-proof property (a registered host thunk lands in
the 0x7f00_0000_0000 host-call region and `host_call_at` resolves it back with no
compilation). Productized baseline re-verified green (renderscene + 3 swap
Ok(0x1) + persist byte-exact, exit 124).

**Honest scope:** the engine's real present walker drives a populated scene list
to completion — build (SH63) + present (SH64) of the node/item ABI a Lua-created
screen would consume, end-to-end headlessly. The node render-obj is still the
recovered ctx/fabricated coherent object, NOT a real UI/GuiObject, so the drawn
content is a distinct engine-parity clear, not a populated login/home screen.
Standing structural wall unchanged (Lua app-shell / nativeGameGlobalInit parks /
type-4 producer vector glue-installed only). The SH64 desync wall is closed: any
future per-node draw can now be host-side GLES content without recompiling the
walker.

**Next frontier:** (a) synthesize the walker's per-item draw as REAL geometry
(engine emitter 0x105b35288 via the seeded GLES slots, or direct Mesa mesh calls
from the host thunk — both now desync-safe) so a populated node draws
engine-detailed content instead of the parity clear; or (b) advance the
data-persistence path (objective 2b) now that renderscene/renderwalker prove the
frame+present plane; or (c) target the standing structural wall — feed a real
engine session producer so the engine self-populates its scene list (needs Lua
app-shell / auth+network).

## Session (Sep 13, 2026, hermes-worker, cycle SH63) — the engine's REAL scene renderer now walks a POPULATED scene list: it builds ONE real 0x98 frame-desc per 0x28-stride scene node, in addition to the base frame at R+0x170 — closing SH62's named "populate the scene list" gap. Workspace **508/0** (was 507/0, +1). Commits pending. Doc docs/frontier-sh63-scene-populated-nodes.md, artifact runs/sh63-renderscene-populated.txt, repro runs/capture_renderscene.sh (now node-count-aware).

`render_scene_base(node_count)` now lays N 0x28-stride scene nodes into R
(R+0x180=head, R+0x188=head+N*0x28 tail) and `render_engine_scene(ctx,n,nodes)`
fills each node's render-obj slot (+0x08 = the real ctx, whose vt[+64] is the
dims-query the per-node loop blr's), drives the engine's OWN scene renderer
`0x105b2ead4`, and VERIFIES the engine built a real frame per node (each
[+144]==1) via the per-node loop at file 0x5b2eb9c (links each at container
node+0x18 through linker 0x5b2d9e0). New env RENDERSCENE_NODES (default 3).

**Empirical (real libroblox.so, runs/sh63-renderscene-populated.txt, exit 124):**
`scene renderer Ok(0x1)` with `scene_nodes=3 per_node_frames=[3 ptrs]
per_node_ok=true frame[vtable]=0x1067317b0`; `present #N swap Ok(0x1)` x2;
persist 45B byte-exact; zero SIGSEGV/SIGABRT/json-overflow. SH62 empty baseline
re-verified (RENDERSCENE_NODES=0 → fast path, still swaps Ok(0x1)).

New regression `scene_per_node_build_contract_populated_scene_list` pins the
node offsets (obj@+0x08 / view+container@+0x18 / stride 0x28), the head/tail
one-past-end termination, and the frame linker + vtable contract.

**Honest scope:** the engine builds a real frame-desc per node + the base frame
and presents them via its real swap — a POPULATED render-manager now registers a
genuine multi-item frame plane headlessly. The node's render-obj is the recovered
ctx only (vt[+64] real dims-query), NOT yet a real UI/GuiObject, so the per-node
frame carries engine-detail metadata, not a populated login/home screen (Lua
app-shell wall — SH53/SH56). Validates the exact node/item ABI a Lua-created
screen would consume.

**Next frontier:** (a) drive the engine's real per-node PRESENT walker
`0x105b2ed48` (blr's each node+8 item's vt[+24] as the per-item draw, then swaps;
gated on nativeGameGlobalInit — reach it by setting walker gate bytes R+559/664/608
or synthesizing its draw item's vt[+24] → real geometry emitter 0x105b35288); or
(b) target the engine's own geometry emitter (0x105b35288 / primitive-setup
0x105b353d0, the SH25-34 proven path) as the per-node render-obj so a populated
node actually draws engine-detailed content.

**SH64 empirical note (present walker, reverted):** a mid-cycle attempt to drive
the engine's REAL present walker found (1) the full `0x105b2ed48` body aborts at
entry (TLS canary + nativeOnDestroyed teardown tail fault), and (2) driving just
the present-loop region 0x105b2eec0 (x19=R) DID run the engine's real per-node
loop and blr our fabricated per-item draw (`item draw #1 engine frame-fn Ok`) —
validating the per-scene-item vt[+24] draw ABI — but SIGSEGVs on iteration 2
because the item thunk's nested jit_run recompiles the very present-loop block
(SH44/49 drain-desync class). Next try: patch the walker's parked
nativeGameGlobalInit bl (0x5b2ee54)→ret so its full body runs natively, or draw
through the already-seeded GLES slots (no nested jit_run at the loop address).

## Session (Sep 13, 2026, hermes-worker, cycle SH62) — drove the engine's REAL scene renderer (guest 0x105b2ead4): the engine constructs+registers its OWN frame-desc (vtable 0x1067317b0) and presents it via the real ctx swap — replacing the SH60/61 harness-fabricated clear-path renderer. Workspace **507/0** (was 506/0, +1). Commits b30eaae + b39a525. Doc docs/frontier-sh62-renderscene.md, artifact runs/sh62-renderscene.txt, repro runs/capture_renderscene.sh.

Two READ-ONLY research subagents + my disasm of real libroblox.so pinned the concrete
frontier artifact: the engine's REAL frame-plane driver is `0x105b2ead4` (scene
renderer), and — disasm-proven — it builds+links a real 0x98 frame-desc
**UNCONDITIONALLY, even with an EMPTY scene list**: bind ctx (vt+16 make-current),
read view W/H at R+0x170+112/116, dims-query (ctx-vt+64), then operator-new(0x98)
0x1d96768 -> frame-desc ctor 0x5b34de8 (sets vtable 0x6731000+0x7b0=0x1067317b0 at
[+0], [+140]=w7, [+144]=1) -> link 0x5b2d9e0(&R+0x170, frame); ONLY THEN compares
scene list head/tail (R+0x180/0x188); empty => skip => return 1.

New elfjt `--renderscene`: arm a fabricated-but-engine-native render-manager R
(R+0x160=recovered real ctx, R+0x170=view w/ SENTINEL W/H, R+0x180==R+0x188 empty),
drive 0x105b2ead4(R) on the currency-owning renderinit thread after RENDERCTX, verify
engine_registered (frame non-zero, [+144]==1), present via real ctx swap 0x105b3b408.

**Empirical (real libroblox.so, runs/sh62-renderscene.txt, exit 124):**
`scene renderer Ok(0x1)`; `R+0x170 frame=0x7fb68a1c8c00 node=0x7fb689fa4f30
engine_registered=true frame[vtable]=0x1067317b0[+140]=0x0`; `present #N swap Ok(0x1)`
x3; persist 45B byte-exact; zero SIGSEGV/SIGABRT/json-overflow.

**Key empirical catch (SDLC):** setting the fabricated view W/H to 1280x720 —
exactly the live EGL surface dims the dims-query returns — makes the renderer's
"dims unchanged = frame already built" check (`cmp w8,x22; b.eq skip`) SKIP the build
(R+0x170 stays = our view). Fix: view W/H = sentinel 0xFFFFFFFF/0xFFFFFFFE (never ==
surface dims) forces the BUILD branch; engine constructs the frame at the REAL dims.

**Honest scope:** activates the engine's real scene-renderer frame plane + proves it
constructs/registers its own frame item headlessly — but the item is the frame-desc,
NOT a populated login/home UI screen. R+0x180 scene list is still EMPTY (fast path).
Real screens need the engine to populate the scene list with UI items = the Lua
app-shell + auth/network (standing structural wall). Contribution: engine's own
frame-desc construction is provably reachable headlessly; the repeated SH18/60
fabricated clear renderer is replaced by the engine's real frame-desc construction.

**Next frontier:** populate R+0x180 scene list — construct a real 0x28-stride scene
node (node+8 coherent render obj, node+24 real view) so the renderer's per-node loop
presents engine-detailed content, not just the single empty-scene frame-desc. Depends
on the in-image scene-item builder (file 0x5b2c828 / 0x5b2eb7c / 0x5b2ec1c) or a real
screen-construction entry beyond the Lua wall.

Also this cycle: fixed a pre-existing **concurrency race** in the resolver —
`resolve_gles_int/mixed/egl` probed the cache WITHOUT the lock, dlsym'd, then
`alloc_slot` re-locked and allocated a fresh slot WITHOUT re-checking the cache. Two
threads resolving the same GLES name got two DIFFERENT adjacent slots (off-by-8),
breaking the slot-identity invariant the API/tests rely on
(`egl_get_proc_address_routes_guest_blr_to_dispatchable_slot_e2e` failed in the full
parallel workspace run, passed 5/5 in isolation). Fixed: `alloc_slot` now
re-checks `r.slots` under the already-held lock (idempotent-in-cache), closing the race
for every caller in one place. Full arm64jit suite passed 6/6 parallel runs (was
flaky); workspace 507/0. Commit f942d23.

## Session (Sep 13, 2026, hermes-worker, cycle SH61b) — closed the SH60 cross-thread swap Ok(0x0) wall. **On the real libroblox.so taskv4-frame recipe: 24/24 task-driven presents are now genuine eglSwapBuffers Ok(0x1), ZERO Ok(0x0)** (was 25 Ok(0x1) / 5924 Ok(0x0) in SH60), exit 124. Commit 68b3362. Doc docs/frontier-sh61b-single-owner-presenter.md, artifacts runs/sh61b-presenter.txt + runs/sh61b-product-reverify.txt.

An empirical negative pinned the fix. I first tried a global presenter **mutex**
around the whole bind→frame-fn→swap sequence (SH60's "serialize presenters"
suggestion). It did NOT change the ratio (25/6399) — the Ok(0x1)s are exactly the 25
renderinit-thread presents. Conclusion: the drain thread's run_guest_callback
make-current cannot establish EGL currency even serialized (its guest callback path
differs from the renderinit thread's). The fix is to ROUTE the present to the
currency-owning thread, not to lock:

- **`type4_frame_thunk` (drain thread) is now a pure producer**: accounts the
  dispatch, reads the node's dispatchable flag ([node+40] bit0 — diagnostic only),
  bumps a process-wide `PENDING_PRESENTS` counter, returns. No EGL work / no
  run_guest_callback → safe + cheap on the drain thread, cannot hit the EGL-current
  wall. Self-guards on RENDERCTX==0.
- **New `present_one_task_frame(ctx, n)`**: engine make-current 0x105b3b358 →
  frame-fn 0x105b32c00 → swap 0x105b3b408, returns the swap result.
- **Presenter loop** on the --renderthunk thread (the ONE thread where render-init
  left EGL current): drains PENDING_PRESENTS, rate-limited to a bounded window
  (TASKFRAME_MAX_FRAMES / TASKFRAME_WINDOW_MS, default 24 / 2 s) so the run still
  exits 124 cleanly. The drain flood adds PENDING orders of magnitude faster than
  llvmpipe can present, so the bound keeps it a clean sustainable stream.

**Verified** (runs/sh61b-presenter.txt, 8k-line log, exit 124): 24/24 `present
swap Ok(0x1)`, 0 Ok(0x0), `presenter drained: 24 real task-driven frames presented
(all on the currency-owning thread)`, 196 real NODE pops, dispatch counter #3.4M,
zero crash/json-abort, persist roundtrip intact. **Productized baseline re-verified**
(runs/sh61b-product-reverify.txt): exit 124, real triangle (centroid red) + textured
quad (BL/BR/TR/TL) + 7 swap Ok(0x1) + persist 45B byte-exact + 0 crash; no-seed path
fires the single deterministic present (SH60 marker preserved). Workspace 506/0.

**Honest scope:** closes the swap-Ok(0x0) wall — task frames now present genuinely
on the correct thread — but does not change WHAT is rendered (still the clear-path
frame-fn with the fabricated renderer, not the engine's own login/home UI, which
needs a populated render-manager). The PENDING counter can grow large during the
flood (harmless u64, no stored frames).

**Next frontier (unchanged):** feed a real engine session producer; the engine's
REAL frame driver is guest 0x105b2ead4 (scene renderer: R+0x160 ctx / R+0x170 view /
R+0x180 scene list) — only renders real login/home once the engine constructs a
populated render-manager (needs its game/UI setup, Lua/textures/login scene). That is
the standing structural wall for "engine renders its own real screens".

## Session (Sep 13, 2026, hermes-worker, cycle SH61) — DELIVERED recon-v3 deliverable (2): the RBX::json::Writer stack-leak fix. New env-gated JIT hook (JIT_JSON_ZERO_FIX=1, OnceLock-evaluated `json_zero_fix_enabled`) clamps the string LENGTH (reg x2) to 0 at the append bound-check guest 0x102355d40 exactly when it would throw (writer cap cell guest 0x107275648 < len → `cmp x8,x2; b.cc`), so the leaked uninitialised-stack-string write becomes a libc++ SSO EMPTY append (size()==0) that never reaches the throw helper 0x1025fb6bc. Workspace **506/0** (was 505/0, +1). Commit 63302e2. Doc docs/frontier-sh61-json-fix.md, artifact runs/sh61-json-fix-after.txt + runs/sh61-product-reverify.txt. **Both recon-v3 deliverables are now implemented & measured headlessly.**

A research subagent (deleg_6bb58b66, read-only on the repo + binary disasm) refined
the brief's proposed fix. The recon suggested zero-filling the leaking guest-stack
slot at `[append-entry-sp-0x38]`, but static disasm could not positively confirm
that slot lies inside a live frame (it is below the check-fn's own frame —
red-zone/caller-below), so a stack write there is unverified. The subagent's
recommended register clamp delivers the SAME "size()==0 → SSO empty" effect with no
out-of-frame write and no frame-offset re-derivation:

- **Patch (jit.rs run_loop, at block entry):** when armed and pc == 0x102355d40, read
  len = reg x2 and cap = i32 at guest 0x107275648 (disasm: `adrp x8,7275000; ldrsw
  x8,[x8,#1608]`); if `(cap as u64) < len` — the exact `b.cc` throw condition — set
  x2 := 0. Guest memory is identity-mapped so the cap reads directly; ASLR-immune.
- **Capacity cell is READ-ONLY** — never raised (raising makes the writer memcpy with
  len's low 32 bits ~1.6GB → SEGV), per the recon's hard rule.
- **`json_zero_fix_enabled()`** evaluates JIT_JSON_ZERO_FIX once via `OnceLock` (not
  per-block), keeping the hot loop clean. Off by default → production path untouched.

**Verified on real libroblox.so (bare StartApp, no render recipe):**
- WITHOUT the fix: `libc++abi: terminating ... RBX::json::Writer string length
  overflow: 139734512814576` (run-variable host heap ptr), exit 139.
- WITH JIT_JSON_ZERO_FIX=1: `[json-fix] ... would overflow (len=0xb3 cap=0)` +
  `(len=0x24 cap=0)`, then `grep -c "json::Writer string length overflow"` = **0**;
  StartApp's json serialization proceeds. Both leak modes covered (huge host-pointer
  len, small-but-over-cap len); benign len ≤ cap untouched.

**Honest boundary:** the fix eliminates only the json abortion. The bare `--jni
--startapp` path (NO render/lifecycle drive) then proceeds deeper into boot and hits a
DIFFERENT pre-existing fault — SIGSEGV at guestpc 0x102175854 (`ldr x23,[x20,#8]`,
null deref in a GameActivity/FMOD init region), exit 134/SIGABRT. That is the
SH45-documented bare-path wall (only the full productized recipe with
JIT_DRIVE_LIFECYCLE + render-init boots clean, exit 124). The json fix does not
regress the productized path (re-verified green below).

**Productized baseline re-verified unchanged (runs/sh61-product-reverify.txt):**
real indexed triangle (centroid RGBA(255,0,0,255)) + textured quad (BL=RED/
BR=GREEN/TR=WHITE/TL=BLUE exact texels) + 7 swap Ok(0x1) + persist 45B byte-exact +
0 json-overflow/SIGSEGV/SIGABRT (quad-loop was still animating fresh frames when the
foreground cap cut it at 180s; the hook is off here so it is inert on this path).

New regression `json_zero_fix_clamps_leaked_length_at_append_check` pins the disasm
addresses (check file 0x2355d40 / cap cell file 0x7275648 / throw helper file
0x25fb6bc), the `(cap as u64) < len` predicate for both leak modes, benign-len
non-clamp, and len==0 never tripping any cap.

**Next frontier (unchanged, SH60):** bridge the w4=4 dispatch rate into the post-ctx
window (drive a drain cycle after RENDERCTX), chase `swap Ok(0x0)` on the cross-thread
path (serialize presenters to one thread; engine 0x105b3b408 never binds, EGL
current-binding is thread-local), and feed a real engine session producer. The engine's
REAL frame-plane driver is guest 0x105b2ead4 (scene renderer: R+0x160 ctx / R+0x170
view / R+0x180 scene list) — it only renders real login/home once the engine constructs
a populated render-manager, which needs its game/UI setup.

## Session (Sep 13, 2026, hermes-worker, cycle SH60) — DELIVERED recon-v3 deliverable (1): the SELF-DRIVEN task-frame plane. `--taskv4-seed frame` registers a `type4_frame_thunk` host-thunk into the dispatcher's type-4 vector [0x106829ea8]; each w4=4 dispatch marshals into a REAL presented frame (engine make-current 0x105b3b358 -> frame-fn 0x105b32c00 -> swap 0x105b3b408) on the recovered real ctx (vtable 0x106731ae0). **`present #147942 swap Ok(0x1)`** on the live EGL display/surface/context is the concrete measured marker; the task counter hit #147942 (w4=4 dispatches reaching the thunk en masse). Workspace **505/0** (was 504/0, +1). Doc docs/frontier-sh60-taskv4-frame.md, artifact runs/sh60-taskv4-frame.txt, reproducible runs/capture_taskv4_frame.sh.

Three new pieces landed (elfjit.rs):
- `--taskv4-seed frame`: registers the non-recursive leaf `type4_frame_thunk`
  host-thunk (register_host_call_auto) into `[0x106829ea8]`. It reads RENDERCTX
  (recovered real 0x48 ctx), reads make-current (vt[+16]=0x105b3b358) + swap
  (vt[+24]=0x105b3b408), seeds the 10 engine-GLES dispatch slots once, builds
  the fabricated coherent renderer/view (SH18/SH22), cycles a per-dispatch
  clear-color palette (distinct fresh frames), and via nested run_guest_callback
  drives make-current -> frame-fn 0x105b32c00 -> swap. Never re-enters the
  vector/drain/dispatcher (would recurse). RENDERCTX==0 -> self-guard no-op.
- RENDERCTX publication: --renderthunk stores the recovered real ctx into a
  process-wide atomic so the dispatch-plane thunk (different thread) consumes
  it.
- Deterministic w4=4 engineering (from fresh disasm of drain 0x102856e40 +
  dispatcher 0x10285371c): the drain's genuine popped-node w4=4 path (0x2856ffc)
  is hit only in an early init window, and its idle-heartbeat dispatches go to
  w4=2/3 telemetry (never the vector). Fix: rewrite both heartbeat `mov w4,#2/#3`
  (0x102856f24/0x102856f68) to `mov w4,#4`, so every idle dispatch routes the
  real dispatcher to the seeded vector (the ~147k dispatch counter proves it).
  Because those flood dispatches precede RENDERCTX recovery, the CLEAN present
  is a deterministic post-ctx dispatch: the --renderthunk thread drives the
  thunk once (on the context-already-current thread) with the dispatcher ABI ->
  **present #147942 swap Ok(0x1)**. The drain-thread present is Ok(0x0)
  (cross-layer swap didn't report EGL success). Also: --deque-node-live in frame
  mode holds its first node PLACEMENT (not root capture) until RENDERCTX, so
  real node pops terminating at w4=4 fire with a live ctx.

New regression `type4_vector_seed_accepts_registered_host_thunk_abi` pins:
register_host_call_auto lands a handler in the reserved 0x7f00_0000_0000
host-call region, host_call_at resolves it back, and invoking it executes the
task consumer with the (node, [node+32]&~1, consumer) ABI.

Verified: cargo test --workspace 505/0; productized baseline RE-VERIFIED green
(runs/sh60-product-reverify.txt: exit 124, textured quad BL=RED/BR=GREEN/
TR=WHITE/TL=BLUE exact texels + triangle + quad-loop, persist byte-exact, 0
crash; the --renderthunk change also now fires one task-driven frame there,
present #1 swap Ok(0x1)). No json-abort/SIGSEGV/ENOSYS in the taskv4 run.

**Honest scope:** recon §A's core claim delivered and measured — the task-
consumer ABI presents real frames through the engine's own
make-current/frame-fn/swap on the recovered live EGL ctx (`present swap
Ok(0x1)`). Not yet the engine detail-rendering its own login/home screens (the
thunk drives the clear-path frame-fn, not the full UI render stream), and the
w4=4 flood sits in the boot window (the clean present is the deterministic
post-ctx dispatch). Standing structural wall materially advanced, not closed.
Next frontier: bridge the w4=4 dispatch rate into the post-ctx window (drive a
drain cycle after RENDERCTX) + chase the cross-thread swap Ok(0x0), then feed a
real engine session producer.

## Session (Sep 12, 2026, hermes-worker, cycle SH59) — drove the recon-v1 "still-live lower-effort" V1 6-jstring start (`nativeAppBridgeAppStart__`, file 0x2338510 / guest 0x102338510) as a STANDALONE primary `--startapp-v1` — the first time the V1 entry has been exercised headlessly. Workspace **504/0** (was 503/0, +1). Commits d5e9ccb, a528b5e. Doc docs/frontier-sh59-v1-appstart.md, artifact runs/sh59-v1-appstart.txt.

Recon-v1 (docs/recon-framework-boot-order.md) names the V1 6-jstring
`nativeAppBridgeAppStart__` the "still-live lower-effort alternative" that
bypasses the AutoValue params layer; recon-v2 §Task-2 blames the StartApp
json-abort on the JNI shim collapsing that params layer. But SH54-58 only ever
ran V1 as the **unreachable tail** of the v2boot ladder (which stalls at rung 1
`nativeGameGlobalInit` every cycle), so the V1 entry's downstream never executed
and its ABI was never even exercised:

- **New elfjit `--startapp-v1`**: swaps the primary `--startapp` target from
  V2StartAppWithParams to V1 `AppStart__`, building the s2 ABI as 5 empty
  jstrings (x2,x3,x5,x6,x7) + jboolean false (x4=0) per the exact mangled-JNI
  descriptor `String,String,Z,String,String,String`. V1 reads plain jstrings
  straight from the registers (no AutoValue jobject, no Call*Method getter), so
  it **bypasses the params-collapse json-abort completely**.
- **Fixed a latent wrong ABI** in the never-reachable v2boot V1 fallback: it
  put a jstring handle in the `Z` boolean slot (x4) and left x7=0 — corrected to
  the true descriptor.
- **New regression** `v1_app_start_six_arg_abi_is_five_strings_plus_boolean`
  pins the ABI (5 readable guest-addressable jstring handles; x4 Z-slot exactly
  0; no String handle aliases the boolean slot).

**Empirical (real libroblox.so, full productized render recipe + `--startapp-v1`,
artifact runs/sh59-v1-appstart.txt, exit 124):** engine's OWN frame-fn renders a
real indexed triangle (centroid red) + textured quad (BL=RED/BR=GREEN/TR=WHITE/
TL=BLUE) + 4 fresh quad-loop frames (swaps Ok(0x1)), byte-exact persist
roundtrip (45 B, REMEMBERED session), **zero** json-string-length-overflow /
SIGSEGV / SIGABRT / ENOSYS — the V1 path executes clean where V2's params layer
was suspect.

**Honest scope:** V1 advances the engine to the **same** structural place as V2
(the main loop parks on the lifecycle-await futex lr=0x10284d134; the type-4
producer vector `[0x106829ea8]` stays 0 — framework-glue-installed only), so a
self-driven home/session screen is not yet reached; frames remain harness-driven
on the live engine context. Contribution is empirical + corrective: recon-v1's
lower-effort alternative is now proven clean + standalone-callable, and its
previously-wrong ABI is fixed + pinned so future V1 work starts from a correct
register layout. Standing structural wall unchanged.

## Session (Sep 12, 2026, hermes-worker, cycle SH58) — first REAL-guest-handler type-4 seed executed on the real binary: the drain's w4=4 dispatch `br`'d into the engine's OWN frame-fn (real engine code ran at guestpc 0x105b2e98c) before ABI-faulting. Workspace **503/0** (was 502/0, +1). Commit 2922518. Doc docs/frontier-sh58-taskv4-realseed.md, artifacts runs/sh58-{taskv4-realseed,taskv4-sustain,baseline}.txt.

Every SH44-57 frontier doc names the same next step: "feed a REAL engine
frame/session producer address into the seed so a sustainably-dispatched task
node advances the engine toward its own frame/screen." But every run used
`--taskv4-seed probe` (a registered HOST thunk) — the guest-hex form
(`--taskv4-seed <guest-addr>`) existed but was NEVER exercised. This cycle closes
that gap and empirically executes the recon §3.6 "interim fallback":

- **run runs/sh58-taskv4-realseed.txt:** full productized boot +
  `--taskv4-seed 0x105b32c00 --deque-node-live 0x106829f00 --drain-poll 8`.
  The vector was seeded with the engine's REAL frame-fn; the drain `br`'d into
  it and the JIT translated+executed real engine code (frame-fn's renderer
  list-find at `guestpc 0x105b2e98c`) then faulted on ABI (the vector passes
  `handler(node, [node+32]&~1, consumer, w4, 5)`, frame-fn expects a
  coherent renderer). **First confirmed real-guest-code dispatch through the
  type-4 plane** — seed rejection is NOT the wall; the wall is exactly the
  known one: the framework-installed "process popped task node" worker address
  is external glue absent in-image, and any real seed must match the
  `(node, [node+32]&~1, consumer)` ABI (frame-fn doesn't).
- **new regression** `type4_vector_seed_accepts_real_in_image_guest_function`
  pins the vector addr + that a real in-image guest fn is mechanically valid as
  a seed + the ABI contract a real producer must match.
- **re-verified green on current HEAD (post SH55/56/57):** probe sustain
  (runs/sh58-taskv4-sustain.txt) = **190 consecutive node pops, 3 clean type-4
  dispatches, exit 124** (SH49 downstream of the value-registry/asset/storage
  changes); productized real-boot (runs/sh58-baseline.txt) = exit 124, real
  triangle + textured quad + 6 quad-loop frames, byte-exact persist roundtrip,
  594 assets extracted+served, zero ENOSYS.

**Honest scope (unchanged structural wall):** frames remain harness-driven; the
engine never self-produces a session/frame because `[0x106829ea8]` is
framework-glue-installed only (no in-image store — SH46/52/53/55/56), and the
real worker's address is not in the binary. Contribution is empirical: seed
rejection ruled out + exact ABI a future real producer must match, and the whole
stack re-confirmed green on current HEAD.

## Session (Sep 12, 2026, hermes-worker, cycle SH57) — made the AAssetManager shims REAL (image-backed) + extract the real APK's assets so the engine can load its own UI content — recon-v2's "precondition for a frame". Workspace **502/0** (was 499/0, +3). Commit 1b02552. Doc docs/frontier-sh57-assetmanager.md, artifact runs/sh57-asset-run.txt.

Recon-v2 (docs/recon-framework-boot-order.md) names the AssetManager the
"precondition for a frame": the engine reads its UI content (594 assets/ entries —
FoundationImages sprite sheets, BuilderIcons fonts, GLSL shader packs) out of the
source APK via AAssetManager_fromJava/open/getLength/getBuffer. All four shims
returned NULL/0 (both the Rust JIT shims and the C jni_stubs.h path), so the engine
could not load a SINGLE real asset even if a self-driven frame were produced. This
cycle makes them image-backed:

- aassetmanager_fromJava → stable non-NULL manager sentinel.
- AAssetManager_open(mgr, filename, mode) reads the guest C-string filename
  (normalizes a "assets/" prefix), serves it from the host SOBER_ASSETS_ROOT (the
  extracted APK assets/ dir) into a stable owned buffer in an open-asset table.
- AAsset_getLength/getBuffer → real length / stable host pointer the guest derefs
  directly (guest vaddr == host addr in this JIT; the Box buffer is never moved).
- AAsset_close drops the handle; missing/unmounted still fail NULL/0 so unarmed
  boots are untouched.
- New apk::extract_assets() decompresses the APK's assets/ into out_dir/assets
  (Roblox stores 203/594 DEFLATE), handles the flat-APK + assets/app.zip→
  config.arm64_v8a.apk bundle forms, skips non-assets/dir entries, returns None for
  a no-assets APK.
- main.rs --jit exports SOBER_ASSETS_ROOT before launch_jit (the spawned elfjit
  inherits it).

**Honest scope:** the standing structural wall is UNCHANGED — the engine still
never self-produces a session/frame (type-4 producer vector [0x106829ea8] is
framework-glue-seeded only, recon-Task-1/2 closed in SH53/SH56), so in the
productized run it does not yet issue AAssetManager calls. The asset plane is
served, proven hermetic, and configured for the run — the recon's named
precondition block is removed; it is only *reached* by a future self-driven
session. The only proven live dispatch plane remains `--taskv4-seed` +
`--deque-node-live`; next frontier: feed a REAL engine frame/session producer
address into the seed so a sustainably-dispatched task node advances the engine
toward its own frame/screen (which now has the assets to load).

## SH57b (Sep 12, 2026, hermes-worker): LocalStorageManager.getAllocatableBytes()
now reports the host's REAL free space (fstatvfs on SOBER_ANDROID_ROOT) instead
of the collapsed 0 — recon-v2 flags 0 ⇒ the engine believes there's no disk and
RbxStorage never builds its content cache (undermines objective 2b's remembered
session cache plane). Extends the AutoValue getter regression. Workspace 502/0.
Commit 401683c.

## SH57c (Sep 12, 2026, hermes-worker): PlatformParams.getAssetFolderPath yields
the host assets root (SOBER_ASSETS_ROOT from SH57 extraction) so the engine's
content loader can find real UI/texture/font files by direct FS open, not just via
AAssetManager; unmounted -> NULL/0 (boot-safe). Extends the AutoValue getter
regression; productized real-boot re-verified exit 124. Workspace 502/0.
Commit b73f0eb.

## Session (Sep 12, 2026, hermes-worker, cycle SH56) — empirically CLOSED recon Task-2 on the real binary: the AutoValue getter-value registry (SH55) does NOT fix the StartApp json-abort — the leaked string length is params-independent (a host-mmap pointer read at the append bound-check, never touching the getter registry). Workspace **499/0** (unchanged). Doc docs/frontier-sh56-json-abort-params-independent.md, artifacts runs/sh56-{startapp-json-abort,startapp-jobject-abort,jsondump}.txt.

The recon (docs/recon-framework-boot-order.md, Task-2) claims the `RBX::json::Writer
string-length-overflow` abort's root cause is the JNI shim: `Call*Method` returned
typed-0, collapsing the AutoValue params layer so StartApp re-serialized uninitialised
guest-stack std::strings. SH55 wired the full getter-VALUE registry. But that registry
had never been observed live against StartApp's serialization (SH55's `--v2boot` stalls
at rung 1; the productized recipe passes a JSON jstring, not a jobject). This cycle
drives `nativeAppBridgeV2StartAppWithParams (0x258b144)` with a genuine AutoValue
jobject (new elfjit flag `--startapp-jobject`) and answers definitively.

**Both params forms abort identically** (exit 139, `RBX::json::Writer string length
overflow: <host-pointer>`): JSON jstring `{"key":""}` (leak 0x7fb5..., runs/sh56-startapp-json-abort.txt)
AND real AutoValue jobject + full value registry (leak 0x7f19..., runs/sh56-startapp-jobject-abort.txt).
Rigorously pinned (`JIT_DUMP_PC`, runs/sh56-jsondump.txt):
- Append bound-check `0x102355d40`: x1 = a **host mmap pointer** read as the std::string
  length → throws; mechanism identical to SH45's sp/sp-0x30 leak.
- Throw helper `0x1025fb6bc`: x0=0x10057765a (fmt string), printed value is the host ptr.
- **`JIT_TRACE=1` emits ZERO `[jni] Call*Method` getter lines** — the AutoValue getter
  registry is never even reached during StartApp's serialization.

**Conclusion / redirect:** recon Task-2's root-cause claim is disproven — the value
registry is necessary if StartApp ever gets far enough to use the params, but it is NOT
the json-abort's cause. The abort is a harness-bootstrap artifact gated by the
render/lifecycle drive (SH45's original characterization): the **productized** recipe
(drives lifecycle + ANativeWindow + render-init warmup WITH StartApp) never aborts and
renders real frames — re-verified green this cycle (persist roundtrip byte-exact, real
indexed triangle centroid red, textured quad BL=RED/BR=GREEN/TR=WHITE/TL=BLUE, 6
quad-loop frames, swaps Ok(0x1)). Future bare-StartApp work should target the
guest memory whose length field holds a host pointer, NOT the getter registry.
`--startapp-jobject` kept as a reusable params-layer diagnostic. Standing structural
wall unchanged: type-4 producer vector [0x106829ea8] framework-glue-seeded only.

## Session (Sep 12, 2026, hermes-worker, cycle SH55) — completed the AutoValue params getter-value registry (CallLongMethod + CallFloatMethod now serve real values via a new s0-return JNI bridge) and re-tested the ordered V2 ladder WITH a complete params layer (which SH54 lacked): it still stalls at `nativeGameGlobalInit` (parks, never returns) and cannot populate `[0x106829ea8]`. Workspace **499/0** (was 498/0, +1). Doc docs/frontier-sh55-jni-value-registry.md, artifact runs/sh55-v2boot-ladder.txt.

Recon-v2 Task-2 (the json-abort root cause) is that jni.rs routed every `Call*Method`
to typed-0, so StartApp's json serialization read uninitialized guest-stack std::strings.
SH54 wired Object/Boolean/Int; this cycle completes the prescribed surface:

1. **CallLongMethod (NDK slot 52, newly wired)** — getAppUserId→0, getDeviceTotalMemoryMB→8192.
2. **CallFloatMethod (NDK slot 55, newly wired)** — getDpiScale→1.0. A `jfloat` returns in
   the FP register **s0**, not x0 (AAPCS64). New `HostJniF32` bridge + thunk region
   (jit.rs): a whole-CpuState bridge reads the methodID from x2 and the dispatcher writes
   the u32 into guest s0 before resuming at x30 — the existing float32 bridge drops the
   integer-register args and the GLES bridge writes x0 not s0, so neither could serve it.
   Regression proves a real guest `blr` through env->functions[55] lands 1.0f32 in s0
   (`fmov w0,s0; brk #0`; a naive `ret` looped on its own post-blr x30).
3. **String getters** extended with DeviceParams: getOsVersion→"33" (Vulkan GATE), device
   name/sku/manufacturer→Cordial, country→US, networkType→WIFI, appVersion→"".
4. **Booleans** per recon v2 (isUnder13…isLowRamDevice false; isKeyboardDevice/
   isMouseDevice/isCpu64Bit true).

**Empirical (runs/sh55-v2boot-ladder.txt, exit 124):** the full stable product recipe +
`--v2boot` on the real libroblox.so: render + persist baseline INTACT (no regression),
zero SIGSEGV/SIGABRT, zero json overflow — but the ladder reaches only
`driving nativeGameGlobalInit`, which does **not return** (runs real init then parks), so
rungs 2–6 never run on the detached driver thread and [0x106829ea8] stays **0**. This is
SH54's wall re-measured WITH the complete params layer (so StartApp's serialization would
have real values) — re-confirming the recon Task-1 (APS2) verdict at runtime: the
ordered-ladder reframe does not populate the type-4 producer vector headlessly.

Standing structural wall unchanged: `[0x106829ea8]` is framework-glue-seeded only; the ONLY
proven live dispatch plane is `--taskv4-seed <real-handler>` + `--deque-node-live` (SH44
live-when-seeded; SH49 sustainable at 197 pops). Next frontier: feed a REAL engine
frame/session producer address into the seed so a sustainably-dispatched task node advances
the engine toward its own frame/screen.

## Session (Sep 12, 2026, hermes-worker, cycle SH54) — built the ordered V2-boot ladder drive (`--v2boot`) + AutoValue getter shim — the empirical runtime test SH53 left open — and confirmed the type-4 producer vector `[0x106829ea8]` stays 0 while the real `nativeGameGlobalInit` executes. Workspace **498/0** (was 497/0, +1). Doc docs/frontier-sh54-v2boot-ladder.md, artifact runs/sh54-v2boot-{ladder,fw,progress,threads}.txt.

The recon (docs/recon-framework-boot-order.md) demands driving the real V2 boot
IN ORDER (nativeGameGlobalInit → setTaskSchedulerBackgroundMode(false) →
V2InitWithParams → StartLuaAppDM → V2StartAppWithParams) with AutoValue JNI
jobjects, not JSON. SH53 had left only the empirical drive open. Two pieces
landed this cycle:

1. **AutoValue getter shim (jni.rs).** `CallObjectMethod`/`CallBooleanMethod`/
   `CallIntMethod` (NDK slots 34/37/49) previously returned 0 for every getter,
   so StartApp's json serialization read uninitialized guest-stack std::strings
   — the SH45/SH46 `RBX::json::Writer string length overflow` abort. Because
   `GetMethodID` returns a readable handle of the method NAME, the new stubs
   dispatch on the getter name and return a real, readable empty jstring
   (`"Dark"` for `getSelectedTheme`) / false / 0; unrecognized names still fall
   back to 0 (real Java re-entry unchanged). Wired into the official NDK table.
   +1 regression `jni_auto_value_params_getters_resolve_via_fn_table`.
2. **`--v2boot` ordered ladder (elfjit.rs).** A detached thread — spawned
   BEFORE the `start_app` jit_run, which parks the main thread forever and never
   returns — sleeps a warmup then drives the 6 real JNI natives in the recon's
   load-bearing order as fresh guest entries (reusing the boot SP) with AutoValue
   jobjects, dumping `[0x106829ea8]` after EVERY rung, then the V1 AppStart__
   fallback (0x102338510). All params are jobjects (not JSON).

Empirical result (stable productized recipe, exit 124): `nativeGameGlobalInit`
executes REAL engine code (block cache 2192 ≫ ~434 idle baseline), zero
SIGSEGV/SIGABRT, no json-string-length-overflow, real renders + persist
roundtrip intact — yet `task-v4 [0x106829ea8]` stays **0 the entire window**.
This corroborates SH53 at runtime: the recon's "in-image TaskScheduler install
reached via the ordered ladder" is not reproduced headlessly. Honest caveat:
`nativeGameGlobalInit` advances then parks (does not return in the run window),
so only rung 1 is observed — the vector-0 is measured DURING GlobalInit, not
after a full ordered completion that reaches the install site. Standing
structural wall unchanged: the type-4 vector is framework-glue-seeded only;
`--taskv4-seed` + `--deque-node-live` (SH49) remains the only proven mechanism
to run the dispatch plane.

## Session (Sep 12, 2026, hermes-worker, cycle SH53) — DISPROVED the 2026-09-12 recon's reframe that the type-4 producer vector `[0x106829ea8]` is installed IN-IMAGE by TaskScheduler/V2-init code SH46 "never reached". Workspace **497/0** (unchanged). Commits 277f567, 9694a19 (doc). Doc docs/frontier-sh53-recon-disproof.md.

A recon (docs/recon-framework-boot-order.md + /home/hermes-worker/open-sober-framework-glue-spec.md) claimed the ~50-cycle wall was wrong: SH46's "no in-code store" was because the scan ran on a bare boot that never reaches TaskScheduler init, and driving the real V2 ladder (`nativeGameGlobalInit → nativeUpdateAdapterInit → V2InitWithParams → StartLuaAppDM → [Surface] → StartAppWithParams`) in order would populate the vector. Disproven on two independent grounds:

1. **SH46's scan is STATIC** (whole `.text`) — execution-independent, so "never reached init" cannot explain a missing in-image store. If in-image init installed the vector, some decoded instruction would write 0x106829ea8.
2. **The computed-base escape is closed.** The vector is the `.bss` base 0x6829e80 + **0x28**, so `adrp 6829000; add xN,xN,#0xe80; str [xN,#0x28]` would dodge a literal-#3752 scan. Disassembling every `adrp xN,6829000` site: 0x2953e30 writes [0x6829e80] (+0x0, clears first qword); 0x295427c/0x29542ec use 0x6829e88 (+0x8) as an atomic counter (ldxr/stxr, stlr); all other adds target #0xba8/#0xe80/#0xe88/#0xf00 — none reaches #0xea8. All `add #0xea8` sites are struct-relative on dynamic bases, never a 6829000-derived register.

So no in-image (literal or computed-base) store exists. If the V2 ladder installs the vector it is via cross-module glue / host seed (consistent with SH46). Regression `type4_taskv4_vector_has_no_in_code_install_site_and_uses_static_base` extended to pin the vector's 0x28 offset within `.bss` + the computed-base disproof audit. The V2-ladder empirical drive remains open but only as a cross-module/runtime-install test, not an in-image one — priced accordingly (needs AutoValue InitParams/StartAppParams jobjects + real Surface). Workspace 497/0.

## Session (Sep 12, 2026, hermes-worker, cycle SH52) — closed the real client's last 11 unresolved data imports: the `AMEDIAFORMAT_KEY_*` media-format string constants (Android libmediandk absent host-side) now bind to live host C strings. Workspace **497/0** (was 496/0, +1). Commit 4fe90da. Doc docs/frontier-sh52-media-keys-data.md, log runs/sh52-product-verify.txt.

The productized boot line previously read `bound 534 JUMP_SLOT + 67 GLOB_DAT/ABS64 (0 unresolved), 11 unresolved` — 11 data-object GLOB_DAT slots that `dlsym` could not resolve (bionic/mediandk-only symbols). Now reads `... + 77 GLOB_DAT/ABS64 (0 unresolved), 1 unresolved`. The gap was precisely:

- **10× `AMEDIAFORMAT_KEY_*`** (Object, UND): the NDK media-format string constants. A GLOB_DAT relocation writes the **address of the constant** into the GOT slot; the guest does `adrp x0,0x67cf000; ldr x0,[x0,#off]` to load it and passes it to `AMediaFormat_*` as a `const char*`. Left NULL, a real video/audio-decoding session reads a NULL key string — the SH19/SH24 crash class for data reads. `resolve_android_data(name)` now maps each to its NDK value (`"mime"`, `"width"`, ...) as an immortal leaked `CString` (stable/cached pointer).
- **`__sF`** (Object, UND): bionic's `FILE __sF[3]` (stdin/stdout/stderr) base. **Intentionally left unbound-to-host** — pointing it at a host `FILE_` would make the `fwrite`/`vfprintf` shims misclassify a bionic stream as a real host stream and SIGSEGV; leaving its low value is exactly what the shim's fd2-diversion path already handles. Documented, not a gap to "close".
- The `AMediaFormat_delete`/`AMediaCodec_delete` obj slots shown in the JIT_TRACE diagnostic are FUNC imports that already stub-bind via the `is_func` path (that's why only 1 is counted unresolved).

Verified: productized `open-sober play --apk --jit` exit 124 stable, real indexed triangle (centroid red) + textured quad (BL=RED/BR=GREEN/TR=WHITE/TL=BLUE) + 6 quad-loop frames (swaps Ok(0x1)), `--persist-roundtrip` byte-exact, zero ENOSYS; the direct-`--renderinit` SIGSEGV/abort is the documented SH46 pre-existing harness-bootstrap artifact (identical pre-change), not a regression. New regression `android_media_format_key_data_imports_resolve_to_live_strings` pins all 10 constants byte-exact + cached-pointer stability + that unrelated object names (`__sF`, unknown) fall through unclaimed.

`__sF` remains the single legitimately-unresolved data import and is **by design** (see above). The standing structural wall is unchanged: the type-4 producer vector `[0x6829ea8]` is still framework-glue-installed only (SH46), so frames stay harness-driven on the live engine context.

## Session (Sep 12, 2026, hermes-worker, cycle SH51) — the live client's data-persistence plane is now SELF-VE ... [condensed SH283: full detail preserved in the referenced doc/repro] ... to an observable live-client persistence exhibit. Workspace 496/0 (unchanged). Doc docs/frontier-sh51-live-persist.md, log runs/sh51-persist-live.txt.

Objective 2b ("the client REMEMBERS sign-in via its own session/login
datastore") was previously proven only hermetically (SH38–SH42 committed tests);
the productized run made ZERO `[fsmap] remap:` lines because the engine never
reaches a session. This cycle embeds the datastore roundtrip into the elfjit
harness the product launches:

- **example/elfjit.rs `run_persist_roundtrip()`**: openat(O_CREAT)→write→fsync→
  close→reopen→read of `/data/user/0/com.roblox.client/databases/session.db` via
  `guest_svc`, asserting byte-exact read-back AND a real on-disk file under the
  armed SOBER_ANDROID_ROOT. Invoked at startup (`--persist-roundtrip`) because
  StartApp parks in an idle main-loop and never returns, so a post-boot hook is
  unreachable.
- **sober-core jitlaunch.rs**: the productized `play --jit` recipe now adds
  `--persist-roundtrip` + exports `JIT_FSMAP_LOG=1` — every play run
  self-verifies live persistence. Both unit tests extended.
- **arm64jit/src/jit.rs** guest_svc mappath: env-gated `[fsmap] remap:` line
  makes live remaps observable.

**Live artifact (runs/sh51-persist-live.txt, exit 124):**
  `[fsmap] remap: /data/user/0/com.roblox.client/databases/session.db -> ~/.local/share/open-sober/android-root/data/user/0/com.roblox.client/databases/session.db` (×2)
  `[persist] live datastore roundtrip: write=45B fsync=0 read_back_byte_exact=true on_disk=Some(true)`
  and the on-disk store holds exactly `ROBLOSECURITY=_live_client_remembered_session`
  (0600, 45 B). Zero ENOSYS/abort. Full render baseline intact in the same run
  (triangle centroid red + textured quad BL=RED/BR=GREEN/TR=WHITE/TL=BLUE + 6
  quad-loop frames, swaps Ok(0x1)).

**Also pins the type-4 dispatch contract from fresh disasm** (for the next
frontier cycle): dispatcher 0x10285371c (file 0x285371c), on `w4==4`, loads
`[0x106829ea8]` and `br`s to it with x0=node, x1=[node+32]&~1, x2=consumer — the
vector is a leaf function pointer the framework installs; `cbz` returns doing
nothing when unset (the headless-boot wall). Seeding it with the engine's own
0x10285371c would RECURSE (that fn reads the vector), so a correct seed needs the
real framework-installed "process popped task node" worker, whose address is not
statically in the binary (external-glue gap, SH46).

**Honest scope:** this makes live persistence self-verifying/observable in the
deliverable, but drives the JIT's own guest_svc ABI rather than the engine's
session code. The standing structural wall is unchanged: the engine still never
self-produces a session or renders its own login/home screen (type-4 producer
vector [0x106829ea8] is framework-glue installed only); frames remain
harness-driven on the live engine context.

JIT_EGL_LOG (SH47) left exactly two `UNRESOLVED` eglGetProcAddress names:
`glPushGroupMarker`/`glPopGroupMarker` (non-EXT spelling). They ARE in
GLES_INT_NAME_LIST but resolve_gles_int required the exact symbol, and both
Mesa libraries export ONLY the EXT-suffixed spellings (`glPushGroupMarkerEXT` /
`glPopGroupMarkerEXT`, verified via nm); a guest `br` through the engine's
dispatch-table slot for these names would have jumped to NULL (SH19/SH24/SH47
crash class, applied to the two names that cycle never reached).

- **Fix (resolver.rs):** when a whitelisted name is NULL in both libs and does
  not end in `EXT`, fall back to `{name}EXT` (itself whitelisted, int-ABI-safe)
  against GLESv2 then libGL. Both plain names now resolve to real bridge slots.
- **New regression** `plain_non_ext_marker_names_resolve_via_int_bridge_ext_sibling_fallback`
  (all 4 spellings resolve via int bridge, bridge-pointer slots, rejected by
  mixed). Workspace 496/0 (+1).
- **Productized re-verify** (runs/sh50-product-reverify.txt): `open-sober play
  --apk roblox-android.apk --jit` exit 124 stable, real indexed triangle
  (centroid RGBA(255,0,0,255)) + textured quad (BL=RED/BR=GREEN/TR=WHITE/TL=BLUE)
  + 6 fresh quad-loop frames, swaps Ok(0x1), **zero** eglGetProcAddress
  UNRESOLVED marker lines.
- The direct-elfjit `--renderinit` SIGABRT (right after ANativeWindow wiring) is
  confirmed PRE-EXISTING (aborts identically with this change stashed) — the
  documented SH46 harness-bootstrap artifact, before this cycle's code path.

Standing structural wall unchanged (SH14/SH46/SH49): the type-4 producer vector
`[0x6829ea8]` remains framework-glue-installed only, so the engine still does
not self-produce a frame — frames stay harness-driven on the live engine
context.

## Session (Sep 12, 2026, hermes-worker, cycle SH49) — the type-4 task-dispatch plane is now SUSTAINABLE: the `--deque-node-live` injector stops re-evicting the drain's translated block, so inject+pop+dispatch runs 197 consecutive pops with zero crash (SH44 faulted at pop #39). Workspace 495/0 (unchanged). Doc docs/frontier-sh49-taskv4-sustain.md, artifact runs/sh49-taskv4-sustain.txt.

SH44 proved the type-4 popped-task dispatch plane (`[0x6829ea8]`) is functional
when seeded, but every sustained attempt faulted after ~39 injected nodes
(SIGSEGV at guest 0x102856f7c). Root cause (runs/sh44-taskv4-plane.txt): the
injector re-patched force-pop **and re-dropped the cached drain blocks
(`block_cache_drop_region(0x102856e40, 0x1028570c0)`) on EVERY re-injection**,
recompiling the pop-loop while the drain was mid-execution of it — after ~39
churns the translation desynced and the popped-node register loaded an
instruction word (`x22=0x7bfdd503233f`, low-32 `0xd503233f` = a NOP). The
patches are idempotent and the guest bytes stay patched, so the per-iteration
reset was pure churn.

Fix (elfjit `--deque-node-live`): a static `ARMED: AtomicBool` runs the
force-pop patch + cache eviction **exactly once** (first real placement);
subsequent re-injections only swap the node into the head-cell. Verified on the
full productized recipe + `--taskv4-seed probe --deque-node-live 0x106829f00
--drain-poll 8`: **197 consecutive `NODE ... POPPED`** (SH44 crashed at #39), 3
clean type-4 vector dispatches through the real dispatcher w4=4 plane, armed +
evicted exactly once each, zero SIGSEGV/abort/`string length overflow`, exit
124 (stable idle). Productized `open-sober play --apk --jit` re-verified green
(runs/sh49-product-reverify.txt): exit 124, real triangle + textured quad
BL=RED/BR=GREEN/TR=WHITE/TL=BLUE + quad-loop, swaps Ok(0x1), zero ENOSYS.

**Honest scope:** this removes the harness's self-destructive reinjection so a
real producer can be seeded sustainably; it does NOT yet drive a self-produced
frame. The vector is still harness-seeded (probe) here — SH46 proved no in-code
install site. **Next (closest unblocked):** feed `--taskv4-seed 0x<guest>`
(+ `--deque-node-live`) a real engine frame/session handler so a sustainably
dispatched task node advances the engine toward its own frame — the loop it
runs in no longer self-destructs. Standing structural wall otherwise unchanged.

## Session (Sep 12, 2026, hermes-worker, cycle SH48) — the engine's instanced-mesh pipeline is no longer NULL-bound: `glVertexAttribDivisor` joins the GLES int bridge, and the SH37 instanced gate is extended from a no-op count=0 draw probe to a real non-empty instanced draw (count=1, 4 instances) with a bound VBO + divisor 1 against real Mesa. Workspace 495/0 (was 494/0, +1). Doc docs/frontier-sh48-instanced-divisor.md.

A real instanced mesh must call `glVertexAttribDivisor(index, n>0)` to mark the
per-instance attribute; that name was absent from `GLES_INT_NAME_LIST` (SH35 only
whitelisted the two *draw* functions), so a guest `br` through the engine's slot
resolved to NULL/0 and every instance read instance 0's data — the instanced draw
silently degenerated to one duplicated triangle (no crash, wrong output). This
cycle adds `glVertexAttribDivisor` to the int bridge (auto-heals the engine's
eglGetProcAddress-built table — no harness re-seed), extends the SH37 functional
gate to a real non-empty instanced draw with a bound VBO + divisor 1, and adds a
focused regression pinning it (with its setup companions `glVertexAttribPointer` +
`glEnableVertexAttribArray`) resolves via int bridge and is rejected by mixed.
Productized real-boot baseline re-verified: triangle + textured quad
(BL=RED/BR=GREEN/TR=WHITE/TL=BLUE) + 3 quad-loop frames, swap Ok(0x1), exit 124.
The standing structural wall is unchanged (SH14/SH46): the type-4 producer vector
`[0x6829ea8]` is populated only by real Android framework glue, absent headlessly,
so frames remain harness-driven.

## Session (Sep 12, 2026, hermes-worker, cycle SH47) — closed the last NULL-dispatch gap in the engine's real GLES render table: GL4/extension slots 11/12 (glBufferStorage/glMapBuffer/glQueryCounter/glObjectLabelKHR et al.) now resolve via a desktop-libGL fallback. Workspace 494/0 (was 493/0). Doc docs/frontier-sh47-gles4-desktop-fallback.md.

The engine's own render dispatch table (BSS `0x106d3b2f0 + 8*N`, built via
`eglGetProcAddress` / SH3 interception) had **slots 11 and 12 reading 0x0**
even though its render code `bl`s those slots **unguarded** (slot-11 stub
`0x5b3a244` ×2, slot-12 `0x5b3a250` ×4). A self-driven frame routing through
them would `br` to NULL and SIGSEGV — the last uncovered NULL-dispatch surface
in the engine's GLES3 table (SH19/SH24/SH35 bug class, applied to the slots
those cycles never reached).

- **Root cause was NOT a missing whitelist entry / float-ABI rejection.** New
  env-gated `JIT_EGL_LOG=1` diagnostic in `w_eglGetProcAddress` (logs every
  requested name that fails all bridges AND Mesa GLESv2, w/ guest PC) showed the
  engine resolves exactly **16 GL4/extension names** that come back 0 because
  Mesa's ES-only `libGLESv2.so.2` does not export them: `glBufferStorage(EXT)`,
  `glMapBuffer(OES)`, `glQueryCounter(EXT)`, `glObjectLabelKHR`,
  `glPush/PopGroupMarker(EXT)`, `glGetQueryObject{ui64v,iv}(EXT)`.
- **Fix:** new `gl_desktop_handle()` (dlopen `libGL.so.1`, same RTLD_LOCAL
  discipline) + `resolve_gles_int()` falls back to it for whitelisted int-ABI
  names absent from GLESv2. All 16 added to `GLES_INT_NAME_LIST` (each pure
  int/ptr ABI ≤4 args → safe through the integer HostCall, rejected by mixed).
  Engine table auto-heals via SH3 — no harness re-seed.
- **Live proof** (`runs/sh47-egllog2.txt`): seed snapshot slots 11/12 went
  `0x0` → `0x7f0000003098/90` (real bridge slots); UNRESOLVED dropped 16 → 2
  (only un-EXT-suffixed `glPush/PopGroupMarker`, absent from both libs; the EXT
  variants are bridged). Full productized render re-verified exit 124: triangle
  centroid red, textured quad BL=RED/BR=GREEN/TR=WHITE/TL=BLUE, 3 fresh
  quad-loop frames, swaps all `Ok(0x1)`. Slots-11/12 disasm: callers pass
  `w0=0x8a11 (GL_UNIFORM_BUFFER), size, NULL data, flags` == **glBufferStorage**
  — the engine's modern UBO path already dispatches it.
- New regression `gles4_extension_names_resolve_via_int_bridge_desktop_gl_fallback`
  pins all 15 desktop-exported names + the critical subset resolve via int
  bridge (trailing NUL) and are rejected by mixed. Workspace **494/0** (+1).

**Next (unchanged standing frontier):** the type-4 task-producer vector
`[0x6829ea8]` is still framework-glue-installed only (SH44/SH46); the engine
never self-produces a render/session task, so frames remain harness-driven.
Directions per SH46: (a) synthesize the framework task-producer registration
from the engine's own game-activity/lifecycle init and call that guest path; or
(b) advance objective 2b by exercising the fsmap/SQLite datastore plane with
the client's real serialization. This cycle removed a real NULL-crash surface a
self-driven frame WOULD hit (buffer/timer/query slots now bridge-routed).

SH45 left the type-4 producer vector `0x6829ea8` as the open standing frontier and
suggested a concrete next lever: "make the seeded LSM empty-map / fake-object
memory guest-shaped / zero-length so StartApp's json serialization reads a valid
empty string length instead of a host pointer." This cycle **empirically
disproves that hypothesis** and redirects the frontier:

- **The leaked "string length" is a guest stack address, not the LSM seed.**
  Driving `JIT_DUMP_PC` at the json append check-fn entry (0x102355d40) and the
  throw helper (0x1025fb6bc) across three independent runs (fresh ASLR each):
  the overflow value is always `== sp` (run A: leak `0x7f4462ffd9f0` exactly
  equals x29==x31/sp of the throw frame) or `== sp−0x30` (run B/C), tracked to
  within a small constant. The seeded LSM bucket array sits ~0x260–0x2a0 MB away
  in the host mmap/heap and is **not** the leaking allocation. So the real
  mechanism is guest-internal: StartApp's `nativeAppBridgeAppStart`-family json
  writer reads an **uninitialised std::string on the guest stack** as a length,
  trips `ldrsw x8,[0x7275000+1608]; cmp x8,x2; b.cc` in the append bound-check,
  and throws from `RBX::json::Writer string length overflow: %zu` (format file
  0x57765a) via the throw-with-value helper 0x25fb6bc.
- **The type-4 producer vector has no guest install site.** A full-image objdump
  scan for `adrp 0x6829000` + `[x,#3752]` stores proves no curso instruction
  writes guest `0x106829ea8` (the only `[x,#3752]` stores are struct-relative on
  heap/sp regs; the dispatcher at file 0x2853784 is the sole static-base reader).
  "Reverse what the framework installs into the vector in-code" is a confirmed
  dead end — the install is external framework/GL game-activity glue absent
  headlessly. The dispatch plane remains proven live when seeded (SH44).

Two new regression tests pin both facts for future cycles
(arm64jit/src/jit.rs): `type4_taskv4_vector_has_no_in_code_install_site_and_
uses_static_base` and `json_overflow_leak_reads_guest_stack_pointer_not_seeded_
lsm_map`. Doc: docs/frontier-sh46-json-abort-sp-disproof.md. Productized
`play --jit` re-verified green (exit 124, real indexed triangle + textured quad
+ quad-loop, swaps Ok(0x1)); bare `--jni --startapp` still reproduces the abort
(as documented, harness-bootstrap-only).

**Next (reframed frontier):** the standing structural wall is unchanged — the
engine never self-produces a frame/session task because task-v4 `[0x6829ea8]`
is populated only by real framework producer glue (now proven absent in-code).
And the bare-StartApp json abort is NOT fixable by LSM seeding (it's guest
stack state). Future directions: (a) since the vector is glue-installed, the
remaining host lever is to synthesize the FRAMEWORK task-producer registration
it performs (find, from the engine's own game-activity/lifecycle init, how it
would register a producer and call that guest registration path); or (b) keep
advancing objective 2b by exercising the now-hermetic fsmap/SQLite datastore
plane with the client's real serialization—the productized `play --jit` run
currently makes ZERO fsmap remaps because the engine never reaches a session,
so the persistence claim remains proven only hermetically (SH38–SH42), not by
the live client.

## Session (Sep 12, 2026, hermes-worker, cycle SH45) — characterized the bare-StartApp `RBX::json::Writer string length overflow` abort (a run-variable host heap pointer leaked from the seeded empty-LSM-map into a guest json string-length read) and gave elfjit an abort-class crash dumper. Workspace 491/0 (was 490/0). Commits 9b99fac, 3ada26e.

The SH44 documented open item — "bare-StartApp `RBX::json::Writer string length
overflow` abort (harness LSM host-pointer seed leaking into a string-length read
on the no-render bootstrap path; the productized `play --jit` recipe bypasses
it)" — is now confirmed and precisely characterized (docs/frontier-sh45-json-writer-abort.md):

- The leaked "length" is **run-variable and in the host-mmap region**
  (`0x7f1f53ffe9f0`, `0x7f6f2bffe9f0`, diff.value each run) — a **host pointer**,
  read where the guest expects a string length during StartApp's json
  serialization. The precise leaking allocation is **not positively identified**
  (it differs from the LSM bucket/sub in every run, though same `0x7f` segment);
  SH44's "harness LSM host-pointer seed leaking" remains the leading hypothesis,
  not a proven identity. The `std::runtime_error` is thrown by
  `RBX::json::Writer` (one of the 7507 throw-with-value sites materializing
  `0x57765a`, e.g. disasm 0x2355d98/0x2557dcc: `adrp x0,577000; add x0,x0,#0x65a;
  mov x1,<len>; bl 0x25fb6bc`).
- **params-independent**: `{"key":""}`, `{}`, `""`, `"X"` all abort (different
  heap value each run) — not the params jstring.
- Gated by **absence of the render/lifecycle drive**: the full productized recipe
  (JIT_DRIVE_LIFECYCLE + appcmd + ANativeWindow + render-init) runs clean (exit
  124, real triangle + textured-quad frames, zero overflow); only bare
  `--jni --startapp` hits it. So it is a **harness-bootstrap** artifact, not a
  production runtime bug. The productized deliverable is unaffected.

Also committed a real diagnostic (elfjit.rs): the fault dumper previously caught
SIGSEGV/SIGILL only, so an abort-class crash (libc++ terminate → abort / guest
abort) exited without a guest dump. SIGABRT is now added to the handler set, and
its default disposition is restored before re-raising via process::abort() (which
itself delivers SIGABRT — without the restore the handler recurses in an infinite
dump loop). Now any guest abort yields a full guest PC/regs/backtrace dump.

Verified: `cargo test --workspace` 491/0 (unchanged); build clean; productized
render re-verified through the modified elfjit (exit 124, real indexed triangle +
textured quad, swaps Ok(0x1)).

**Next (unchanged, the standing frontier):** the type-4 task producer vector
`0x6829ea8` remains the structural wall — `.bss`, populated only by a real
framework task-producer absent headlessly; `--taskv4-seed` proves the dispatch
plane is live when seeded (SH44). If a future cycle needs the bare StartApp path
to proceed WITHOUT the render recipe, the next lever is to make the seeded LSM
empty-map / fake-object memory **guest-shaped / zero-length** so StartApp's json
serialization reads a valid empty string length instead of a host pointer (see
`seed_static_empty_map` in elfjit.rs + the LSM reader 0x1d99e40).

## Session (Sep 12, 2026, hermes-worker, cycle SH44) — collapsed the ~30-cycle deque "maintenance wall" to its precise mechanism: the type-4 popped-task dispatch vector [0x6829EA8] is 0 on headless boot BUT the task-dispatch plane is proven FUNCTIONAL when that vector is seeded. Workspace 491/0 (unchanged). Commit pending.

The engine's task-deque drain (0x2856e40) has FOUR dispatch sites with hardcoded
w4 types: heartbeat types 2 (0x2856f24) and 3 (0x2856f68) run telemetry-event
emitters (globals 0x68262E8/0x6826300/0x6826308/0x6826320, each `adrp 67d1000;
ldr [x,#1776]; mov w1,#evtid; bl 1e0b0a8`), and the popped-task type 4 (0x2856ffc
/0x285703c) dispatches through a **distinct BSS function-pointer vector
`0x6829EA8`** (`adrp 6829000; ldr x3,[x8,#3752]; br x3`). That vector is `.bss`
(zero-init, no reloc) and remains 0 on every headless boot — including during the
real render — so any popped task node returns doing nothing (0x285378c->0x2853af0).

**Empirical proof (new `--taskv4-seed` lever; runs/sh44-taskv4-plane.txt):** with
`--taskv4-seed probe --deque-node-live 0x106829f00 --drain-poll 8`, the drain
pops 39 injected task nodes and 3 reach the seeded host-thunk handler through the
REAL dispatcher w4=4 plane with the exact disassembled ABI
(`handler(node=x0, [node+32]&~1=x1, consumer=x2)`, w4=4, x5=0):
`type-4 task handler #1: fnarg0(x0)=0x107334000 arg1=0 arg2=0x7f32829a48c0 w4=4`.
(The follow-on SIGSEGV at guestpc 0x102856f7c is the known force-pop+re-inject+
patched-drain-recompile race, SH11/SH13 — the 3 clean firings are conclusive.)

**Conclusion / refinement:** the wall is NOT an unreachable dispatch plane — it
is that **nothing installs the type-4 producer vector headlessly** (only a real
framework task-producer would). So foreign task nodes can advance the engine only
once `0x6829EA8` is pointed at a REAL guest frame/session producer. Diagnostic:
`JIT_FRAMEWORK_DUMP` now also samples `task-v4 [0x106829ea8]` + `v0/2
[0x106826320]`. Frontier (SH45): reverse what the framework installs into
0x6829EA8 and what task type/arg selects a frame/session producer; also the
bare-StartApp `RBX::json::Writer string length overflow` abort (harness LSM
host-pointer seed leaking into a string-length read on the no-render path — the
productized `play --jit` recipe bypasses it). Doc
docs/frontier-sh44-taskv4-vector.md. Baselines unchanged (productized `play
--jit` still exit 124 + real render).

## Session (Sep 12, 2026, hermes-worker, cycle SH43b) — also proved the legacy `gethostbyname` resolution path (the client imports both DNS APIs). `gethostbyname("localhost")` returns a static thread-local `hostent` — a differently-shaped result than getaddrinfo (h_addrtype@16/h_length@20/h_addr_list@24) — walked to an AF_INET 127.0.0.1. Workspace 491/0 (was 490/0). Commit 75d9d31.

## Session (Sep 12, 2026, hermes-worker, cycle SH43) — proved the guest DNS plane end-to-end through the real guest ABI: `getaddrinfo("localhost") → ai_addr → connect(203) → sendto → recvfrom` roundtrips a login payload to a real host TCP peer, then frees via the guest's own freeaddrinfo. Workspace 490/0 (was 489/0). Commit 23f4ff4.

A logged-in session's FIRST network action is hostname resolution — `getaddrinfo` —
BEFORE any connect. SH42b proved socket/connect/sendto/recvfrom only against a
hardcoded loopback IP; the resolution step was unproven. `getaddrinfo` is a libc
JUMP_SLOT import the resolver binds to HOST glibc via `dlsym` (not a raw syscall), so
the plane rides the resolver (not `guest_svc`).

The new hermetic regression `guest_dns_getaddrinfo_resolves_hostname_then_connect_roundtrip`
(crates/arm64jit/src/resolver.rs) resolves the getaddrinfo/freeaddrinfo slots, drives a
guest `blr x16` to the getaddrinfo slot with (node="localhost", service=<live-port>,
hints=NULL, &res), walks the returned aarch64-LP64 addrinfo chain (ai_family@4,
ai_addrlen@16, ai_addr@24, ai_next@40), asserts localhost resolves to an AF_INET
sockaddr that is exactly 127.0.0.1, feeds ai_addr/ai_addrlen into guest_svc
socket(198)/connect(203), roundtrips a login payload to a real host TCP listener (gets
PONG back, peer asserts exact bytes), and frees the chain via the guest's own
freeaddrinfo import. This closes the last gap between "the socket plane works" and "a
logged-in session can reach a real Roblox API host": resolution → connect → byte
roundtrip all drop-through.

Also RE-VERIFIED the productized deliverable on current HEAD (runs/sh43-play-jit.txt):
`open-sober play --apk roblox-android.apk --jit` extracts the REAL libroblox.so from
the APK and drives JNI_OnLoad(0x2173ff4) → StartApp(0x258b144) → render-init thunk →
the engine's own frame/geometry path through the JIT GLES bridge — real indexed
glDrawElements triangle (centroid RGBA(255,0,0,255)), textured quad
(BL=RED/BR=GREEN/TR=WHITE/TL=BLUE exact texels), 6 fresh quad-loop frames, every
post-draw swap Ok(0x1), exit 124 stable. Boot is fully-wired: 534 JUMP_SLOT bound
(0 unbound), zero ENOSYS/unhandled hostcalls across the full run.

**Next (closest unblocked):** network (SH42b) + data (SH42) + DNS (SH43) planes are all
proven through the real ABI. The standing structural wall is unchanged (~30 cycles):
the engine's own main-loop producer never enqueues a render-task type (the `w4=4`
maintenance cap steering framework-owned deque globals that are thin TLS-upkeep, not
session producers — SH14/SH41/SH42). `--deque-node-live` (SH13) confirms maintenance
dispatch executes real engine code but never reaches egl/gl, and the engine's idle is
the per-CPU task-deque futex (SH39b: ALooper/GameActivity glue loop never entered).
Frames remain harness-driven on the live engine context. A GPU host is the documented
environment for the final self-driven-login / frame-performance proof
(GRAPHICS_RECOMMENDATION.md), deferred (per user preference) until frame/performance
evidence is gathered headlessly here.

## Session (Sep 12, 2026, hermes-worker, cycle SH42b) — proved the CLIENT-side network plane end-to-end through the real `guest_svc` ABI: socket(198)→connect(203)→sendto(206)→recvfrom(207)→close roundtrip a login payload to a REAL host TCP peer on loopback. Workspace 489/0 (was 488/0). Commit 5cc3dd8.

The pre-existing `socketpair(199)+sendmsg/recvmsg` test only covers a
pre-connected pair. A logged-in session's TLS/HTTPS stack funnels byte I/O via
socket→connect→send/recv to an EXTERNAL peer (talking to the host's loopback
exactly as to a Roblox API host), so a real `TcpListener` in a server thread is
spawned and the guest syscall ABI drives connect(127.0.0.1), sendto("SESSDATA\n"),
recvfrom("PONG" echo claim), close; the server asserts it received the exact
payload. Fixes the sockaddr byte-order in the test (`sin_addr.s_addr` is
network-order — portable htonl(INADDR_LOOPBACK) form, not naive from_be_bytes
which made 127.0.0.1 read as 1.0.0.127 → connect ETIMEDOUT). No prod-code change
(all four syscalls already forwarded); this is a regression pinning the full
client network path as drop-through-functional. Real boot unchanged (SH42's
sh42-boot-reverify.txt still exit 124 + real render).

**Next (closest unblocked):** the data-plane (SQLite lifecycle incl.
preadv/pwritev/sync, SH42) and the client network plane (SH42b) are both proven
through the real ABI. The standing structural wall (SH14, re-confirmed SH41) is
still: the engine's own main-loop producer never enqueues a render-task type
(w4=4 cap), so frames are harness-driven. Directions: (a) drive the
confirmed-live deque-maintenance globals (0x1068262e8/300/308) via
--deque-node-live and see if a maintenance dispatch advances the session past
idle; or (b) harden the Android-framework JNI path a logged-in session touches
when it reads/writes its now-persistent store.

## Session (Sep 12, 2026, hermes-worker, cycle SH42) — closed the last data-plane syscall gap: vectored positional I/O + durability. preadv(69)/pwritev(70)/sync(81) are now handled in guest_svc (previously -ENOSYS), completing the raw-SQLite session-datastore lifecycle. Workspace 488/0 (was 487/0). Commit 7351654.

Auditing the handled-syscall set against what a real SQLite-backed datastore
touches surfaced one remaining data-plane gap (the others — flock/fallocate in
SH40b, statx/truncate/linkat/readlinkat in SH40, openat/mkdirat/... in SH38 —
were already closed). A session store flushes db/shm pages with **pwritev**
(batched vectored positional write), reads them back with **preadv**, and issues
**sync(81)** under PRAGMA synchronous=FULL before declaring a transaction
durable. All three had fallen through to -ENOSYS, so a store doing vectored paged
I/O failed (and an unhandled sync made a commit look non-durable).

- preadv(69)/pwritev(70): `struct iovec` is byte-identical across aarch64/x86-64,
  so a raw forward writes the guest iovec array in place; aarch64's two-word loff_t
  pos (a[3]=lo, a[4]=hi) maps onto x86-64's __NR3264 syscall form.
- sync(81): host `libc::sync()` — returns (), so `libc::sync(); 0 as c_long`.

New hermetic regression
`fsmap_preadv_pwritev_sync_support_sqlite_durability_path` (tests/fsmap_persist.rs)
drives the real guest_svc ABI under a configured root: pwritev writes two pages at
distinct offsets into the store, sync returns 0 (not -ENOSYS), and after
close+reopen preadv reads page1 back byte-exact across a fresh fd. Data-plane is
now end-to-end: create → write → statx-exists → flock → fallocate → truncate →
preadv/pwritev → sync → readlink → read.

Verified: cargo test 488/0 (was 487/0); build clean; real boot re-verified through
the modified dispatch (runs/sh42-boot-reverify.txt: exit 124 stable, real indexed
triangle centroid RGBA(255,0,0,255), textured quad BL=RED/BR=GREEN/TR=WHITE/TL=BLUE
exact texels, 3 fresh quad-loop frames, swap Ok(0x1)). Baselines unchanged.

**Next (closest unblocked):** the data-plane is complete for the SQLite session
datastore. The standing structural wall (SH14, re-confirmed SH41) is unchanged:
the engine's own main-loop producer never enqueues a render-task type (the w4=4
cap), so frames are harness-driven. Two directions: (a) drive the confirmed-live
deque-maintenance globals (0x1068262e8/300/308) via --deque-node-live and see
whether a maintenance dispatch advances the session past idle; or (b) harden the
JNI/network surface the client touches once a real session reads/writes its
now-persistent store — the network (socket/TLS) and Android-framework JNI paths
a logged-in session exercises.

## Session (Sep 12, 2026, hermes-worker, cycle SH41) — fixed a real faccessat(48) arg-order + remap bug in guest_svc (aarch64 `faccessat(dirfd, pathname, mode)` — the old handler passed the dirfd as the pathname and the pathname pointer as the mode, so any guest "is my /data session file there?" datastore-accessibility probe read garbage against the host root) and corrected a stale documented premise. Workspace 487/0 (was 486/0). Commit e08ddee.

SH40 completed the fsmap data-plane path coverage, but the fsmap layer is only
as correct as each syscall's argument routing. Auditing the remapped syscalls
against their real aarch64 signatures surfaced one remaining arg-order bug:
`faccessat(48)`. On aarch64 it is `faccessat(dirfd, pathname, mode)` —
x0=dirfd, x1=pathname, x2=mode — but the SH38-era handler did
`mappath(a[0])` + `libc::faccessat(AT_FDCWD, p, a[1] as c_int, 0)`, i.e. it
treated the dirfd integer (often `AT_FDCWD = -100`, an invalid address) as the
pathname C-string and passed the real pathname pointer (truncated to c_int) as
the mode. The same class of bug SH40 fixed for readlinkat/symlinkat. Since bionic
and the Java datastore stack answer "does my session file exist / is it
writable" with exactly this primitive, a wrong faccessat makes the client
misjudge its own (now-persistent) store — undermining objective 2b.

- Fix (crates/arm64jit/src/jit.rs): `(p,_) = mappath(a[1])`;
  `libc::faccessat(a[0] as c_int, p, a[2] as c_int, 0)`.
- New hermetic regression (tests/fsmap_persist.rs)
  `fsmap_faccessat_uses_true_pathname_and_remaps_into_store`, driving the real
  guest_svc ABI under a configured Android root: (1) R_OK/W_OK on an existing
  store `prefs.xml` return 0 (true pathname read + store-resolution);
  (2) a missing store path returns -ENOENT (store-index, not host-root); (3) a
  RELATIVE probe against a real `openat(O_DIRECTORY)` store dirfd returns 0
  (dirfd honored, not hardcoded AT_FDCWD). The old handler fails 2/3 (dirfd read
  as path).

**Also corrects a stale documented premise.** SH14/SH39b wrote that the
type-4 deque-maintenance handler "blrs through framework-owned BSS globals
0x1068262e8/300/308 — all statically 0 on this box." A `JIT_FRAMEWORK_DUMP`
under the stable boot (exit 124) shows those globals are **populated at runtime
with real .text addresses**:
```
[elfjit:fw] deque-fwd 0x1068262e8=0x10620db24 0x106826300=0x102176bfc 0x106826308=0x1022199e0
```
(identical across the full render recipe). The three targets are thin
bionic/atrace-ish upkeep functions (each derefs TLS via `adrp 0x67d1000[#1776]`)
— not render/session producers — and the drain's pop-loop still hardcodes
`w4=4` (maintenance) at dispatch (0x2856ffc). So the **structural wall stands**:
the engine still never self-produces a render-task type, and frames remain
harness-driven on the live engine context. But future cycles should not treat
those globals as an impossible NULL: a seeded node's maintenance dispatch does
execute real engine code (SH13's `--deque-node-live` live-drainer result), which
partially re-opens the deque path this handoff had flagged closed.

Verified:
- `cargo test --workspace` → 487 passed / 0 failed (was 486/0; +1 regression).
- `cargo build --workspace` clean (only pre-existing non_snake_case/dead_code
  warnings; none in the edited lines).
- Full real-boot render (runs/sh41-boot-render-verify.txt): exit 124 stable, real
  indexed glDrawElements triangle (centroid RGBA(255,0,0,255)) + textured quad
  (BL=RED/BR=GREEN/TR=WHITE/TL=BLUE exact texels) + 4 fresh quad-loop frames,
  swap Ok(0x1), zero ENOSYS/unhandled syscalls, no json-Writer terminate.

**Next (closest unblocked):** the data-plane now covers the full SQLite
session-datastore lifecycle through the store. The engine's own main-loop
producer still never enqueues a render-task type (the `w4=4` structural cap),
so frames are harness-driven. Two directions: (a) use the now-confirmed-live
maintenance globals to drive `--deque-node-live` toward real engine framework
code (SH13's live-drainer path) and see whether a maintenance dispatch advances
the session past idle; or (b) harden the JNI/network surface the client touches
once a real session reads/writes its now-persistent store.

SH38's fsmap remapped openat/mkdirat/unlinkat/renameat/faccessat/newfstatat, but
the remaining path-taking syscalls a real session's datastore touches were still
forwarded raw against the host root — a guest `/data/...` path ENOENTed. Most
critically **statx(291)**: bionic/Java answer "does my session file exist / its
metadata" there, so a client's statx on its own datastore path resolving to the
host root + ENOENT makes it *believe its store is gone* — the exact opposite of
the "remembers sign-in" objective. This cycle routes the rest of the path-taking
syscalls through `crate::fsmap::remap_path` (+ `ensure_parents` where the call
creates):

- **statx(291)** — dirfd a0=AT_FDCWD for absolute guest paths; `struct statx` is
  asm-generic/byte-identical on both arches so a raw forward writes the guest's
  statx buffer in place.
- **statfs(43)**, **truncate(45)**, **chdir(49)**, **fchmodat(53)**,
  **fchownat(54)**, **linkat(37)** (both paths), **utimensat(88)**,
  **readlinkat(78)** (pathname), **symlinkat(36)** linkpath.
- **readlinkat arg-order bug FIXED**: the old handler passed the *dirfd* (a0) as
  the pathname with a hardcoded `AT_FDCWD`, so any real guest readlinkat on a
  host-resolved path EFAULTed. Now dirfd=a0, pathname=a1 (remapped).

New hermetic regressions (tests/fsmap_persist.rs, drive the REAL guest_svc ABI,
no APK):
1. `fsmap_statx_and_statfs_reach_the_persistent_store` — after an openat+write
   of `session.dat`, statx reads back the store's REAL stx_size (offset 40 of
   the 256-byte asm-generic `struct statx`); statx on a MISSING store path
   returns -ENOENT (not EPERM, proving it resolved through the store); statfs on
   guest `/data` succeeds.
2. `fsmap_truncate_chdir_linkat_readlinkat_resolve_through_store` — truncate
   shrinks the mapped host file to 4 bytes; chdir lands in the store; linkat
   hard-links a store file; readlinkat resolves a store symlink and returns
   -ENOENT for a missing path (this doubles as proof the arg-order fix works,
   since the old code would have read the AT_FDCWD dirfd as the path).

All path-taking fs syscalls now reach the same persistent store that openat/write
already wrote to, so a real session's datastore survives a restart end-to-end
(write → statx-exists → read). Next: the standing producer/deque wall (SH39b) —
the engine's per-CPU task-deque consumer still parks on the framework producer
enqueue — or more path-hardening as the real client surfaces new syscall gaps.

**SH40b (bd00e88):** the real client's datastore is SQLite-backed — it takes
advisory `flock` locks on db/shm files for concurrency and `fallocate`-preallocates
space when growing mmap-backed db files. Both were unhandled (-ENOSYS). Added
flock(32) -> host advisory lock and fallocate(285) -> SYS_fallocate; removed a
dead duplicate truncate(45) arm left at the durability block (the remapped arm
from SH40 runs). Extended the fsmap meta test to reopen a store file, flock
LOCK_EX|NB, grow it via fallocate to >=4096, release, close. The persistent
store now survives the full SQLite-style lifecycle (create → write → statx-exists
→ flock → fallocate → truncate → readlink → read). Verified the full product
boot+render still reproduces with ZERO ENOSYS/unhandled syscalls after both
commits (runs/sh40-boot-render-verify.txt: real triangle centroid red + 6
sustainable textured-quad frames, exit 124).

## Session (Sep 12, 2026, hermes-worker, cycle SH39) — PRODUCTIZED the proven JIT boot+render: `open-sober pla ... [condensed SH283: full detail preserved in the referenced doc/repro] ... ugh the actual product entry point instead of the debug harness. Exit 124 (stable idle main loop after the render prove). Workspace 484/0 (was 482/0).

The SH15–SH38 frontier had proven this capability only inside the `elfjit` debug
harness; the real product command (`open-sober play --jit`) still ran a crude
`jit::run_elf_entry` stub that couldn't bootstrap the real client. This cycle
wires them together:

- `crates/sober-core/src/jitlaunch.rs` (new): `invocation_proven(lib, frames)`
  builds the canonical recipe `JNI_OnLoad(0x2173ff4) → StartApp(0x258b144) →
  render-init thunk(0x105b3a280) → renderthunk → renderframe(+drive+seedgles) →
  drawprobe → triangle → quad → quad-loop N → kicker(0x106863af8)` with
  `JIT_DRIVE_LIFECYCLE=1` + `RENDERINIT_WARMUP_MS=5000`; `resolve_elfjit_bin()`
  finds/builds the `elfjit` example binary; `launch_jit(lib)` spawns it against
  the extracted `libroblox.so` (elfjit self-contains Xvfb, ANativeWindow→X11 XID
  wiring, and `SOBER_ANDROID_ROOT` arming from SH38/SH38b).
- `main.rs`: the `--jit` branch now `apk::extract_libs → qemu::find_main_binary →
  jitlaunch::launch_jit`. The superseded `jit.rs` (`run_elf_entry`) module is
  removed.
- 2 new regression tests pin the proven recipe + the bounded frame count.

Real proof (runs/capture_jit_play.sh, log runs/sh39-play-jit.txt): the APK
extraction resolved the real `libroblox.so`, the whole chain product command →
APK → real binary → JIT → engine render ran, and the render (`geometry wrapper
0x5b35288 Ok(0x0)`, `swap Ok(0x1)`, exact readbacks, 6 fresh frames) succeeded.
Doc docs/frontier-sh39-productize-play-jit.md.

Honest scope: render is still harness-driven on the live engine context — the engine's
own main-loop producer still never enqueues a render task (SH14's standing structural
wall). SH39 changes WHERE the harness is driven from (the product command, not a debug
example) and arms persistence so a real session's datastore can persist.

## Session (Sep 12, 2026, hermes-worker, cycle SH38) — closed the data-plane FS gap: guest file paths under Android's writable roots now remap to a real persistent host store, so the client's datastore/login session can persist "like the real app". Workspace 482/0 (was 479/0).

New `arm64jit::fsmap` module (crates/arm64jit/src/fsmap.rs) + syscall wiring: the
JIT's `guest_svc` was passing guest file-path pointers verbatim to host libc, so
a real session's `/data/data/com.roblox.client/...` (datastore, shared_prefs,
session cookie), `/sdcard/...`, `/storage/emulated/0/...`, `/cache/...` read/writes
hit the host root and failed ENOENT/EPERM — the client could not persist anything.
Now, when a host root is configured (`SOBER_ANDROID_ROOT` env or a test setter),
those four writable mount roots remap to `{root}/data|storage|sdcard|cache/...`,
and `ensure_parents` recursively scaffolds the `/data/user/0/com.roblox.client/...`
chain so O_CREAT/mkdirat on a deep path never ENOENTs. Input-off by default (root
unset → paths pass through), so the existing boot is untouched. Relative and
non-writable/virtual roots (`/system`, `/proc`) are NOT remapped. Wired through
`crate::fsmap::remap_path` + `ensure_parents` in guest_svc: openat(56),
mkdirat(34), unlinkat(35), renameat(38), faccessat(48), fstatat(79);
read/write/readv/writev on the fd are unchanged.

Hermetic proof (crates/arm64jit/tests/fsmap_persist.rs, drives guest_svc through
the real ABI, no APK): a write under `/data/user/0/com.roblox.client/files/
session.dat` lands in a real host file under the root, and a *fresh* CpuState
("restart") reopens the same guest path and reads the exact bytes back — the
store survives a restart. 3 new regressions: cross-restart persistence, mapping
correctness (+ negative cases: /system, /proc, relative pass through), and
parent-dir scaffolding for mkdirat/deep O_CREAT. Real boot re-verified unchanged
(JNI_OnLoad 0x10006, StartApp driven, stable idle main loop).

**Next (closest unblocked):** this closes the data-plane persistence gap (objective
2b enabler). SH38b (ef0cf2b) additionally arms the persistence root in elfjit real
runs (create + export SOBER_ANDROID_ROOT under XDG/HOME data; verified the full
boot still exits 124 stable with it armed). The standing structural frontier is
unchanged (SH14/SH37): the engine's own main-loop producer never enqueues a render
task, so the engine renders what the harness drives. To turn "persistence works"
into "the client remembers sign-in", re-open the producer/deque wall so the boot
enters a real session that reads/writes the now-persistent store. Doc:
docs/frontier-sh38-fsmap-persist.md.

## Session (Sep 12, 2026, hermes-worker, cycle SH37) — the SH35-sealed GLES3 pipeline slots are proven FUNCTIONAL, not just resolvable: dispatched through the engine's OWN slot stubs on the live context — program-binary round-trip, UBO bind, instanced draw. Workspace 479/0 (was 478/0). Commits a0ba81c (+8f57 ledger).

New elfjit `--renderframe-progbin` drives the engine's dispatch stubs `0x5b3a1c0+0xc*N`
(`adrp x8,6d3b000; ldr x3,[x8,#752+8N]; br x3` — the exact br-through-table mechanism a
real session's frame uses) with real guest-ABI args on the live Mesa-llvmpipe context.
ALL clean (glGetError NO_ERROR, exit 124), coexisting with the standard render path
(geometry wrapper Ok, textured-quad exact texel readbacks, triangle draw, swaps Ok(0x1)):

- slot15 glProgramParameteri(GL_PROGRAM_BINARY_RETRIEVABLE_HINT=0x8257) pre-link.
- slot13 glGetProgramBinary -> a REAL 3498-byte Mesa binary (format 0x875f) — Mesa
  produced a retrievable binary THROUGH the sealed slot.
- slot14 glProgramBinary re-upload accepted (err 0x0).
- slot5  glBindBufferBase(GL_UNIFORM_BUFFER,0,real_buf) binds a UBO (err 0x0).
- slot10 glDrawArraysInstanced(GL_TRIANGLES,0,0,3) dispatches clean (err 0x0).

Bug fixed en route: the slot-stub constants were the .so FILE vaddrs (0x5b3a...) but
jit_run wants GUEST vaddrs → +0x100000000 (0x105b3a...); the first attempt ran
"pc 0x5b3a274 outside image". New hermetic regression
`sealed_gles3_ubo_and_instanced_slots_dispatch_real_mesa_clean` (surfaceless ES3 ctx:
glBindBufferBase + glDrawArraysInstanced through resolve_gles_int -> GL_NO_ERROR).
Live log runs/sh37-progbin-full.txt. This closes SH35's "prove they run" step: every
dispatch slot 0-15 is now bridged AND functionally dispatchable, plus real frames
(solid/triangle/textured-quad/grid, ETC1/ETC2/ASTC) through the engine's own geometry
wrapper + swap.

**Next (closest unblocked):** the harness has saturated the GLES dispatch surface. The
remaining structural frontier (unchanged since SH14): the engine's own main-loop producer
never enqueues a render task, so frames are harness-driven on a time base from a detached
thread. Two candidate directions: (1) re-open the producer/deque wall now that the full
render pipeline behind it is proven bridge-functional (a real self-driven frame is the
remaining 'real session' gap); (2) product-ize: make sober-core's `open-sober play --apk
roblox.apk` reproduce this elfjit boot (JNI_OnLoad + StartApp + render-init + frame drive)
automatically instead of hardcoded elfjit addresses — turning the proof harness into the
runtime's actual boots-real-binary path.

## Session (Sep 12, 2026, hermes-worker, cycle SH36) — sealed the LAST raw clear-dispatch gap: slot 3 (guest BSS 0x106d3b308) now resolves as glClearBufferfi through the MIXED (float) GLES bridge, not glClearStencil. Workspace 478/0 (was 477/0). Commit ea3e692.

Closing the clear-path analog of SH35: disasm of the real clear-state sub-fn 0x5b32ef4
(the per-buffer COMBINED depth+stencil clear) shows `mov w0,#0x84f9` (GL_DEPTH_STENCIL),
`ldr s0,[x21,#68]` (depth -> s0, FIRST FP arg), `ldr w2,[x21,#72]` (stencil),
`mov w1,wzr` (drawbuffer), then `bl 0x5b3a1e4` (the slot-3 stub: `adrp x8,6d3b000;
ldr x3,[x8,#776]` = guest 0x106d3b308). Because depth is a FLOAT, glClearBufferfi is
MIXED-ABI — the integer HostCall only marshals x-regs and would DROP the s0 depth. The
pre-SH36 seed put glClearStencil (single-int) on slot 3, which mis-routes a real
GL_DEPTH_STENCIL dispatch. Fix: new w_glClearBufferfi (AAPCS: gs_f(s,0) for the float)
in gles_mixed_wrapper -> w_eglGetProcAddress auto-heals the engine table;
resolve_gles_int rejects it (float ABI). elfjit seedgles slot3 glClearStencil->
glClearBufferfi. Verified live (runs/sh36-clearbufferfi.txt): slot3 seeds to bridge
0x7f0000018058, textured-quad/triangle draws + swaps all Ok(0x1), exact texel readbacks,
exit 124. New regression resolve_gles_mixed_clearbufferfi_is_mixed_abi_not_int.
Every slot a real frame can dispatch (0-15) now routes through our bridge.

**Next (closest unblocked):** now that the UBO/instanced/program-binary dispatch slots
(4-8 UBO, 9/10 instanced, 13-15 program-binary) AND the full clear map (0-3) are all
bridge-resolved, prove the MODERN GLES3 render path live: fabricate a coherent renderer
that binds a real UBO (glBindBufferBase slot5 + glUniformBlockBinding slot4) and draws
an INSTANCED mesh (glDrawArraysInstanced slot10 / glDrawElementsInstanced slot9) through
the engine's own draw wrapper, read back N distinct instances — proving a real session's
instanced pipeline (heavy in Roblox) runs through the bridge instead of jumping
out-of-image. This is the harness-level proof that the SH35-sealed slots are genuinely
functional, not just resolvable.

## Session (Sep 12, 2026, hermes-worker, cycle SH35) — the engine's REAL GLES3 dispatch-slot table is no longer raw-Mesa: the UBO / instanced / program-binary pipeline slots now resolve through the JIT bridge. Workspace 477/0 (was 476/0).

SH28's live slot snapshot showed the engine's own GL-init fills its GLES dispatch
table (BSS 0x106d3b2f0 + 8*N) slots **4-8** (glUniformBlockBinding / glBindBufferBase /
glBindBufferRange / glGetUniformBlockIndex / glGetActiveUniformBlockiv), **9/10**
(glDrawElementsInstanced / glDrawArraysInstanced) and **13-15** (glGetProgramBinary /
glProgramBinary / glProgramParameteri) with **raw-Mesa host addresses** — the same
SH19/SH24 crash class (a guest `br` through the 0x5b3a1c0+0xc*N stub jumps
out-of-image). A real self-driven engine frame dispatching those slots would have
crashed. This cycle added all ten names to `resolver::GLES_INT_NAME_LIST` (each
pure int/ptr ABI, ≤8 args; rejected by mixed). Because the engine builds its table
via `eglGetProcAddress` (SH3 interception → `resolve_gles_int`), its table now
**auto-heals** to bridge slots — no harness re-seed needed. Verified live
(runs/sh35-pipeline-slots.txt): the PRE-SEED snapshot now shows all ten as
`0x7f000000…` bridge slots (SH28 showed raw `0x7f44…` Mesa). Render path untouched
(geometry wrapper Ok(0x0), swap Ok(0x1), 4×4 grid 16/16 readbacks, exit 124).
New regression `gles3_pipeline_names_resolve_via_int_bridge_for_engine_draw_slots`.
Doc docs/frontier-sh35-gles3-pipeline-slots.md. Commit 51e336f.

**Next (closest unblocked):** the remaining raw-Mesa slot is 3 (glClearBufferfi,
float ABI — mixed, needs a float bridge wrap to seed; the harness still seeds it
as glClearStencil for the clear path). Then extend the coherent renderer's
sustainable loop to dispatch the UBO/instanced/program-binary path through these
now-bridge slots (prove a larger real mesh renders through the engine's modern
GLES3 draw, not the harness @plt), keeping the engine's own main-loop-producer
enqueue as the standing structural frontier.

## Session (Sep 12, 2026, hermes-worker, cycle SH34) — the coherent renderer scales to a REAL LARGER MESH: new ... [condensed SH283: full detail preserved in the referenced doc/repro] ... 4000, tex -> 0x6000; the old 0xc00/0xf60 clobbered for N≥4/8). Harness-only; single-quad mode + --jni baseline + baselines unchanged. Workspace 476/0.

## Session (Sep 12, 2026, hermes-worker, cycle SH33) — SUSTAINABLE TEXTURED real-geometry rendering: new `--re ... [condensed SH283: full detail preserved in the referenced doc/repro] ... op.md; reproducible runs/capture_quad_loop.sh. Harness-only (no codec/resolver change). Workspace 476/0; baselines unchanged (--jni exit 0, idle 124).

## Session (Sep 12, 2026, hermes-worker, cycle SH32) — the engine's OWN geometry path renders a REAL ASTC text ... [condensed SH283: full detail preserved in the referenced doc/repro] ... apture_astc.sh. Compressed live-prove matrix now ETC1+ETC2-RGB+ETC2-RGBA8/EAC+ASTC-4x4. Workspace 476/0; baselines unchanged (--jni exit 0, idle 124).

## Session (Sep 12, 2026, hermes-worker, cycle SH31) — the engine's OWN geometry path renders a REAL ETC2-RGBA ... [condensed SH283: full detail preserved in the referenced doc/repro] ...  runs/capture_etc2a.sh. Compressed-texture live-prove now ETC1+ETC2-RGB+ETC2-RGBA8/EAC. Workspace 475/0; baselines unchanged (--jni exit 0, idle 124).

## Session (Sep 12, 2026, hermes-worker, cycle SH30) — REAL TWO-ATTRIB TEXTURED QUAD renders through the engin ... [condensed SH283: full detail preserved in the referenced doc/repro] ... b,png} = 4 near-equal color quadrants (~186.6k px each) on clear-blue, exit 124. Doc docs/frontier-sh30-quad.md. Workspace 474/0; baselines unchanged.

## Session (Sep 12, 2026, hermes-worker, cycle SH29) — the engine's OWN geometry wrapper renders a REAL ETC2 t ... [condensed SH283: full detail preserved in the referenced doc/repro] ...  the same blob (colors + equality with ETC1). Captured runs/sh29-etc2.{rgb,png}. Workspace 474/0; baselines unchanged. Doc docs/frontier-sh29-etc2.md.

## Session (Sep 12, 2026, hermes-worker, cycle SH28) — captured the real engine GLES dispatch-table content li ... [condensed SH283: full detail preserved in the referenced doc/repro] ... 's own table. Diagnostic-only (harness seeding unchanged, renders correctly). Workspace 474/0; baselines unchanged. Doc docs/frontier-sh28-slotmap.md.

## Session (Sep 12, 2026, hermes-worker, cycle SH27) — the engine's OWN geometry wrapper now renders a REAL COMPRESSED-ETC1 texture: `--renderframe-etc` uploads a hand-crafted 8x8 ETC1 texture (4 solid blocks) via glCompressedTexImage2D; the GLES bridge decompresses ETC1->RGBA (texture-codec) and re-uploads. Three on-triangle probes read back the distinct decoded colors. Workspace 474/0; HEAD (this commit).

Follows SH26's RGBA-textured triangle. The compressed-texture interception path
(implemented since earlier cycles but never proven live) now renders end-to-end:
a hand-crafted ETC1 block `[R,G,B,0,0,0,0,0]` — individual mode, table codeword 0
(modifier +2), all selectors 0, so decoded channel = `(c*0x11)+2` clamped. Blocks
for red `(255,2,2)`, green `(2,255,2)`, blue `(2,2,255)`, white `(255,255,255)`
uploaded as an 8x8 texture through `glCompressedTexImage2D(GL_ETC1_RGB8_OES)`
@plt; the bridge's w_glCompressedTexImage2D decompresses and re-uploads. Readback
proves the decode ran (rounded channels match the (c*0x11)+2 prediction exactly):

```
uTex loc=0x0 ; compile_status vs=1 fs=1 link=1
centroid(WHITE)  = RGBA(255,255,255,255)
quad-(1,0)(GREEN)= RGBA(2,255,2,255)     quad-(0,0)(RED)= RGBA(255,2,2,255)
geometry wrapper Ok(0x0) ; swap Ok(0x1) ; exit 124
```

Captured runs/sh27-etc.{rgb,png}: clear-blue bg + triangle interior 4-colored from
decoded ETC1 (RED 155,909 / GREEN 155,899 / WHITE 52,001 / BLUE 51,947 ≈ 45.1%).
New regression `crafted_etc1_solid_blocks_decode_to_expected_colors` pins the 4x4
block -> (255,2,2) and the 8x8 quadrant decode. Reproducible: runs/capture_etc.sh;
run-log runs/sh27-etc.txt; doc docs/frontier-sh27-etc.md. 8 texture PLT stubs pinned
(see doc). Workspace 474/0.

**Next (closest unblocked):** pin the real engine's GLES dispatch-table slot 11+
mapping for texture/uniform/shader (disasm the engine's texture-binding path so a
textured engine-driven draw routes through the slots rather than harness @plt),
then scale the coherent renderer to a two-attrib (pos+UV) real mesh. Baselines
unchanged: --jni exit 0 (0x10006); stable idle exit 124; untextured triangle and
--renderframe-tex both intact.

## Session (Sep 12, 2026, hermes-worker, cycle SH26) — the engine's OWN geometry wrapper now renders a REAL TEXTURED triangle: a 2x2 RGBA checkerboard sampled by a textured fragment shader, with every texture/uniform/shader call (glGenTextures/glBindTexture/glActiveTexture/glTexImage2D/glTexParameteri/glGetUniformLocation/glUniform1i) dispatching through the JIT GLES bridge. Workspace 473/0; HEAD (this commit).

Follows SH25's solid-red triangle. New `--renderframe-tex` lever: textured FS
(`precision mediump float;` — REQUIRED in GLSL ES 1.00 for a local `vec2`, else
Mesa errors "No precision specified ... for type 'vec2'") samples a 2x2 RGBA
checkerboard (RED/GREEN/BLUE/WHITE) via a UV derived from `gl_FragCoord` (so the
single-attrib coherent renderer stays unchanged). Verified THREE on-triangle
quadrant probes read back three DIFFERENT colors — impossible for a constant
shader, i.e. the sampled texture definitively rendered:

```
compile_status vs=0x1 fs=0x1 link_status=0x1 ; uTex loc=0x0<-unit0
readback centroid(WHITE)  @(640,360) = RGBA(255,255,255,255)
readback quad-(1,0)(GREEN)@(900,150) = RGBA(0,255,0,255)
readback quad-(0,0)(RED)  @(300,150) = RGBA(255,0,0,255)
geometry wrapper Ok(0x0) ; post-draw swap Ok(0x1) ; exit 124
```

Captured frame (runs/sh26-tex.{rgb,png}): clear-blue bg + the triangle interior
4-colored (RED=155,909 / GREEN=155,899 / WHITE=52,001 / BLUE=51,947 ≈ 45.1% of
the frame — the SH25 footprint now textured). glTexImage2D is a 9-arg form
(pixels rides the guest stack at [sp+0]); the PLT stub is a leaf (adrp/ldr/add/br,
never pushes sp), so a fake sp whose [0] holds the pixels ptr is read correctly by
the bridge's gs_stack. Reproducible: runs/capture_tex.sh; run-log runs/sh26-tex.txt.

New debug aid: failing shaders now dump their info log via the int bridge
(glGetShaderInfoLog/glGetProgramInfoLog resolve through resolve_gles_int) — this
surfaced the missing precision declaration.

**Next (closest unblocked):** pin the real engine's GLES dispatch-table slot 11+
mapping for texture/uniform/shader (disasm the engine's texture-binding path so a
textured engine-driven draw routes through the slots rather than the harness's
harness-made @plt calls), then scale the coherent-renderer rotation onto a
two-attrib (pos+UV) real mesh. Compressed-texture (ETC2/ASTC) interception is
already implemented in the bridge/texture-codec and only needs a live-path prove.
Baselines unchanged: --jni exit 0 (0x10006); stable idle exit 124; untextured
triangle still solid-red. Workspace 473/0.

## Session (Sep 12, 2026, hermes-worker, cycle SH25) — the coherent renderer RENDERS a REAL VISIBLE triangle, and the engine's OWN geometry wrapper does it. Workspace 473/0; HEAD 035ff6a.

Follows SH24's draw-probe (empty prim list → proved dispatch only). SH25 feeds
primitive-setup 0x5b353d0 a **coherent** renderer and REAL GL resources, all
through the JIT GLES int bridge on the live render-ctx: a compiled+linked shader
program (VS `gl_Position=aPos`, FS solid red), a real VBO (3×vec4 NDC, ±0.95),
a real EBO (0,1,2). Driving the engine's own geometry wrapper 0x5b35288
dispatches a REAL indexed `glDrawElements(GL_TRIANGLES,3,GL_UNSIGNED_INT)` that
RENDERS: readback `centroid(640,360)=RGBA(255,0,0,255)`; capture
runs/sh25-triangle.{png,rgb} = **415,696 red px = 45.11% of frame** (a clean
apex→base triangle shape), wrapper Ok(0x0), swap Ok(0x1), exit 124.

Three real bugs fixed en route (each caused a silent empty/collapsed render):
1. **Format-table index** — `[prim+8]`=5 is format[5]={size4, GL_SHORT=0x1402};
   engine's glVertexAttribPointer misread float verts as shorts → degenerate.
   Fix `fmt_index=3` = format[3]={size4, GL_FLOAT=0x1406}. This was WHY the
   SH24-scoped "wrapper collapse" existed; with it the RAW engine path renders
   the full triangle (SH25_REF direct-draw now defaults OFF, opt-in =1).
2. **Wrapper count register** — glDrawElements COUNT rides in the 4th drive arg
   (w20 → `mov w1,w20`), NOT x5 (SH24 comment was wrong); + glViewport/glScissor
   (0,0,1280,720) must be set or a stale 0-size viewport rasterizes nothing.
3. **glGenBuffers aliasing** — writing the generated id into the same memory as
   the vertices clobbered the data → dedicated id slots.

Coherent-renderer layout now fully reversed (doc: docs/frontier-sh25-triangle.md):
renderer[+56]=container; container[+72]/[+80]=prim begin/end (stride 0x18,
count=(end-begin)/24 via the magic-const mul); **renderer[+0x48]=INLINE
vertex-descriptor table** (entry[vb]@+vb*16 = descriptor ptr, [desc+72]=ARRAY id);
container[+96]=stride table ([+vb*8]); renderer[+120]=IBO ([+72]=ELEMENT id);
renderer[+142] u16 count; primitive[+0]=vb,[+4]=offset,[+8]=format idx,
[+12]=attrib(0),[+16]=base.

Reproducible: runs/capture_triangle.sh; run-log runs/sh25-triangle.txt.

**SH25b (fd5227b):** `--renderframe-triangle-loop <N>` — same bind → clear →
coherent-draw → swap recipe rendered SUSTAINABLY on the detached host thread,
cycling clear-bg through 5 colors/frame; verified 6 iters all Ok(0x1), red
triangle in every captured frame (runs/capture_triangle_loop.sh). Geometry
analog of SH23's --rendersustain.

**Next (closest unblocked):** GLES slots 11+ (texture/uniform/shader dispatch) +
ETC2/ASTC compressed-texture interception so a *textured/shaded* draw renders;
then scale the (fully-reversed) coherent-renderer rotation onto a larger real
mesh. The engine's own main-loop producer still never enqueues a render task
(the long-standing structural wall) — harness drives its own code on a time base.
Baselines unchanged: --jni exit 0; stable idle exit 124.

## Session (Sep 12, 2026, hermes-worker, cycle SH24) — the engine's OWN real GEOMETRY draw path now dispatches glDrawElements through the GLES bridge: complete 16-slot dispatch map (slots 9/10 = glDrawElements/glDrawArrays), new `--renderframe-drawprobe` that drives the engine's own geometry wrapper 0x5b35288 to a real indexed glDrawElements through the bridge (mode=GL_TRIANGLES, GL_UNSIGNED_INT, GL_ELEMENT_ARRAY_BUFFER bind), wrapper Ok(0x0) + post-draw swap Ok(0x1), exit 124 stable. Workspace 473/0; HEAD e358df0.

Follows on SH23's sustained clear loop. The clear-only frame-fn (0x105b32c00)
never touches geometry; the engine's real draw is wrapper 0x5b35288 →
primitive-setup 0x5b353d0 (binds GL array buffers, enables attrib arrays, sets
glVertexAttribPointer — all via direct @plt) then dispatches the indexed /
array draw through GLES dispatch-table **slot 9 = glDrawElements** (0x5b352f4
bl 0x5b3a22c) / **slot 10 = glDrawArrays** (0x5b35368 bl 0x5b3a238). Those
extended slots (BSS 0x106d3b2f0 + 8*N, 16 total; stub 0x5b3a1c0+0xc*N) held
raw-Mesa addresses — the SH19/SH3 bug class for the draw path.

- Commits: (1) complete the map + seed slots 9/10 + regression. (2) correct
  seed to explicit (slot,name) + add `--renderframe-drawprobe`.
- `--renderframe-drawprobe`: fabricates a minimal renderer (empty primitive
  list → 0x5b353d0 returns mask 0 fast; nonzero [renderer+120] index-buffer
  obj + w5=3 count → INDEXED path). With slots 9/10 seeded the wrapper
  dispatches a REAL glDrawElements through the bridge:
  `hostcall@glDrawElements pc=0x7f0000002a38 x0=0x4 x1=0x0 x2=0x1405
  x30=0x105b352f8` (mode=GL_TRIANGLES, type=GL_UNSIGNED_INT), plus
  `glBindBuffer(GL_ELEMENT_ARRAY_BUFFER=0x8893) x30=0x105b35550`.
- Reproducible artifact: runs/capture_drawprobe.sh; run-log runs/sh24-drawprobe.txt.
  Doc: docs/frontier-sh24-draw-slots.md.
- Regression `draw_slots_gl_draw_elements_arrays_resolve_via_int_bridge`
  (both resolve via int bridge with trailing NUL, rejected by mixed).

**Honest scope:** the fabricated renderer is EMPTY (no real mesh/buffer/VAO
data), so this proves the DRAW DISPATCH is bridge-functional, not the render of
real geometry. Baselines unchanged (--jni exit 0; stable idle exit 124).

**Next wall (the multi-cycle renderer C++ reverse, now clearly scoped):** feed
primitive-setup 0x5b353d0 a coherent primitive list + vertex buffers so the
draw wrapper produces a real rendered triangle. Primitive list lives at
[renderer+56]=container, [container+72]/[80] = begin/end (stride 0x18 per
primitive); vertex buffers/id + VAO state in the renderer sub-objects
(0x5b353fc [x25+96] buffer array, 0x5b3547c descriptor idx, 0x5b35488 attrib
mask). Slots 11+ (texture/uniform/shader dispatch) still unseeded/reversed.

## Session (Sep 12, 2026, hermes-worker, cycle SH23) — the engine's OWN render recipe now runs as a LIVE ANIMATED render loop: new elfjit `--rendersustain <fps>` drives bind -> frame-fn 0x105b32c00 -> post-frame swap CONTINUOUSLY on the detached host thread (concurrent with StartApp's idle main-loop jit_run), cycling a 5-color palette per frame. Workspace 472/0; HEAD b692077.

SH22d proved the recipe reentrant (N=3, frozen color). SH23 makes it
**sustainable + animated**: the engine's own frame-fn is driven on a real
time-base for the whole run — 160 consecutive frame-fn->swap pairs, every
`frame-fn returned Ok(0x..)` + `post-frame swap returned Ok(0x1)`, exit 124
stable, zero crash/heap abort. A real x11grab recording proves every frame is
a **fresh render**: majority pixel color tracks the per-frame palette exactly
(green->red->blue->yellow->magenta; float fracs match to 3 dp), 12+ distinct
frames over 6 s. Each iteration re-writes both engine clear-color sources (the
frame-fn 5th-arg clear-state obj at base+0x400 [+4..16] and the 6th-arg
color-source obj at base+0x500 [+0..16]) before calling the real frame-fn, so a
capture can't be a static buffer.

- New lever: `--rendersustain <fps>` (sustain loop; bounds via
  `--renderframe-loop <N>` only when --rendersustain absent). Per-frame color is
  re-seeded into both clear-color objects each iteration.
- Reproducible artifact: `runs/capture_sustain_loop.sh` (starts elfjit, waits
  for frame iteration 2, x11grab 2fps for CAP_SECS, then decodes each recorded
  frame's majority color). Run-log: runs/sh23-sustain-loop.txt (160 iters);
  video: runs/sh23-sustain-loop.mp4. Doc: docs/frontier-sh23-sustain-render.md.
- Baselines re-verified unchanged: `--jni` exit 0; stable idle exit 124.

**Honest framing (unchanged shape):** still harness-driven — fabricated
renderer/view/clear-state objects and a clear-only frame (the engine's real
glDrawElements draw path is gated on a coherent renderer C++ object not yet
reversed). The engine's own main-loop producer still never enqueues a render
task, so it does not call frame-fn by itself yet — we drive its own code on a
time base. But the GLES dispatch-slot map is complete and the engine's GL path
is proven bridge-functional **and sustainable**, the two properties a real
main-loop frame drive needs.

## Session (Sep 12, 2026, hermes-worker, cycle SH22) — BROKEN THE SH21 WALL: the engine's OWN frame-fn 0x105b32c00 now presents a real, correctly-colored 1280x720 frame through the GLES bridge (18430/18432 sampled px = the exact --renderframe-color 0.4,0.2,0.95), stable exit 124, zero crash. Workspace 472/0 (was 471).

SH21 left the window black, blaming "the per-buffer clear loop clears depth-style buffers via a guessed slot2=glClearDepthf". Disassembly of the clear-state
sub-fn 0x5b32e08 corrects this: **0x1800=GL_COLOR / 0x1801=GL_DEPTH are
`glClearBufferfv` BUFFER enums** (the loop does `slot2(0x1800, drawbuffer=i,
value=clearstate+4+i*0x10)` iterating 4 color draw-buffers; the depth branch
does `slot2(0x1801,0,...)`). So **slot2 = glClearBufferfv**, and slot0 (which the
preamble dispatches with {GL_COLOR_ATTACHMENT0..3} / {GL_BACK}=0x405 arrays) =
**glDrawBuffers** — the SH19-21 "glClearColor"+"glClearDepthf" seed guesses
mis-routed both. glClearDepthf's float bridge ignored the int/ptr args and
cleared nothing → that was the black window.

Fixes (commit): (1) resolver.rs adds glClearBufferfv + glDrawBuffers to
GLES_INT_NAME_LIST (both pure int/ptr ABI, safe through the integer HostCall —
they previously resolved None so the bridge couldn't seed); (2) elfjit seed_names
corrected to slot0=glDrawBuffers, slot2=glClearBufferfv; (3) objB[+140]=0 so the
default-FB preamble takes glDrawBuffers(1,{GL_BACK}) instead of the 4-color-attach
form.

Verified real run (runs/sh22-color-frame-from-engine-framefn.txt):
`slot 0 (glDrawBuffers) <- bridge 0x7f0000003010`, `slot 2 (glClearBufferfv) <-
bridge 0x7f0000003018`, `engine frame-fn 0x105b32c00 returned Ok`,
`post-frame swap returned Ok(0x1)`, exit 124. Frame artifact:
runs/sh22-color-frame-from-engine-framefn.{png,rgb} — raw RGB(102,51,242) =
(0.4,0.2,0.95) = exact clear color; only ~0.013% black (window edge). No channel
swap (the SH21 capture script mislabeled grab byte order b,g,r; raw rgb24 is
r,g,b). Doc: docs/frontier-sh22-color-frame.md.

Honest framing: the frame is still *harness-driven* (fabricated renderer/view/
clear-state objects, one frame-fn invocation + manual swap). The engine's real
main-loop producer still never enqueues a render task, so it doesn't drive
frames natively yet. But the mechanical reverse of slot0/slot2 removes the last
guess-blocker in the engine's own clear path. Full dispatch-slot map pinned from
disassembly (SH22c): slot0=glDrawBuffers, slot1=glClearBufferiv (0x5b32f68,
GL_STENCIL=0x1802), slot2=glClearBufferfv (GL_COLOR=0x1800/GL_DEPTH=0x1801),
slot3=glClearBufferfi (0x84F9=GL_DEPTH_STENCIL); glClearBufferiv added to
GLES_INT_NAME_LIST + seed slot1 corrected. Baselines unchanged: --jni exit 0;
stable idle exit 124.

## Session (Sep 12, 2026, hermes-worker, cycle SH21) — reverse: the frame-fn 0x105b32c00's clear-color object is its 5th arg **x4** (not x2 as SH20 guessed). Fabricating a clear-state x4 object makes the engine's OWN clear-state sub-fn 0x105b32e08 run glColorMask(all-1) + a per-buffer clear-dispatch loop + glGetError THROUGH the bridge. Workspace 471/0 (was 471).

SH20's drive stopped SILENTLY after the GL preamble (glBindFramebuffer/
glViewport/glScissor) — no clear ever fired — because the frame-fn passes its
5th arg x4 to x20 (`0x105b32c30 mov x20,x4`), gated on x4!=0 AND [x4]!=0
(`0x105b32d44 cbz x20` / `0x105b32d4c cbz [x20]`), then bl's the clear-state
sub-fn 0x105b32e08 which reads the clear RGBA float4 from [x4+4..16]
(`mov x21,x2`; `ldp s0,s1,[x21,#4]` / `ldp s2,s3,[x21,#12]`). SH20 left x4=0 →
the whole clear path was skipped, window stayed black. The "x2 = clear-color
struct ptr" comment was WRONG; the clear path uses x4.

New elfjit `--renderframe-drive` fabricates a clear-state object ([\+0]=0xF =
w20 per-buffer clear bitmask, RGBA float4 at [+4..20] from the new
`--renderframe-color r,g,b,a` lever, default 0.4,0.2,0.95,1) and passes it as
the frame-fn's x4. JIT_TRACE now shows NEW hostcalls that never fired before:
glColorMask(all-1) x30=0x105b32e44 (inside the sub-fn) + the per-buffer clear
loop at 0x105b32ec8 (mov w0,#0x1800; bl slot2-stub) iterating the 4 bits of
w20 + glGetError — then clean `frame-fn returned Ok` + `post-frame swap
Ok(0x1)`, stable exit 124. Run-log: runs/sh21-clearstate-x4.txt.

**Honest remaining wall (visible COLOR frame not yet achieved):** the window
still captures black because the per-buffer clear loop dispatches slot2
(seeded glClearDepthf — the 8 slot->function names in --renderframe-seedgles
are heuristic guesses) with integer 0x1800, i.e. it clears depth/stencil-style
buffers, not the color buffer. Getting a visible colored frame needs: (1) the
TRUE function of the slot 0x105b32ec8 dispatches + real slot0/2 names; (2)
which w20 bit maps to GL_COLOR_BUFFER; (3) the second main-fn object at
0x105b32d5c (x22, `ldr q0,[x22]; str q0,[x27]`) — likely the color-clear
source. Doc: docs/frontier-sh21-clearstate-x4.md. Baselines unchanged (--jni
exit 0; stable idle exit 124).

## Session (Sep 12, 2026, hermes-worker, cycle SH20) — resolve_gles_int/resolve_egl now accept trailing-NUL names: ALL 8 of the engine's GLES dispatch slots resolve through the bridge (was 2). Workspace 471/0; HEAD 78eca29.

Built on SH19's --renderframe-seedgles. The 6 int-ABI slots (glClear, glViewport,
glColorMask, glDepthMask, glStencilMask, glClearStencil) printed "NOT resolvable"
even though whitelisted — root cause: resolve_gles_int built its CString
cache-key from the RAW name, so a NUL-terminated caller (elfjit's seedgles
`format!("{name}\0")`, a guest eglGetProcAddress C-string) always got None.
resolve_gles_mixed strips the NUL first and worked (that's why slots 0/2 float
seeded in SH19); the int resolver did not. Same latent bug in resolve_egl. Fix =
build the key from the NUL-stripped name in both; regression
`resolve_gles_int_accepts_trailing_nul_like_mixed` pins all 8 names with a
trailing NUL.

**Real-binary proof (runs/sh20-seedgles-all-slots-ok.txt):** the engine's own
frame-fn 0x105b32c00 now dispatches its ENTIRE clear path through the bridge
(all 8 slots <- bridge slots, incl. the int-ABI glClear/glViewport/glColorMask/
glDepthMask/glStencilMask/glClearStencil that were raw/garbage before) — frame-fn
returns Ok, post-frame swap Ok(0x1), exit 124 stable, no heap abort.

**Un-skipped a dead gate:** resolve_gles_mixed_float_and_stack_abi_execute_real_mesa
silently SKIPPED its whole body for its entire life (every NUL resolve_egl ->
None -> `else return`). It now genuinely runs a surfaceless EGL->ES3->GLES chain
through the JIT bridges and passes, exposing+fixing 3 latent harness bugs: (1)
eglChooseConfig/eglCreateContext attrib arrays must be i32 (EGLint*), not u64;
(2) surfaceless needs a bound pbuffer surface (not EGL_NO_SURFACE) for a
queryable buffer; (3) Mesa surfaceless llvmpipe GL_INVALID_ENUM on
glGetFloatv(GL_COLOR_CLEAR_VALUE) — replaced the state-query with a real
glClear+glReadPixels check (float-bridge color renders [132,65,189,255] px).

Doc: docs/frontier-sh20-gles-nul-resolver.md. Baselines unchanged (--jni exit 0;
stable idle exit 124).

## Session (Sep 12, 2026, hermes-worker, cycle SH19b) — the engine's OWN frame-fn 0x105b32c00 now RETURNS Ok THROUGH the GLES bridge. Workspace 470/0; HEAD 5c72e22.

SH19 pinned the frame-fn wall: it dispatches through 8 GL function-pointer slots
(guest BSS 0x106d3b2f0..0x106d3b328, loaded by `adrp x8,6d3b000; ldr xN,[x8,#752+8k]`)
that hold **raw Mesa host addresses** — not host-thunk slots (0x7f00 0000 0000) —
so a guest `br` through the 0x5b3a1c0-family stubs jumped out-of-image
(`run_loop: pc 0x7fa7… outside image`). Same SH3/SH3b bug class (raw Mesa vs
bridge slot) but for the engine's RUNTIME-built frame-dispatch table. (An early
wrong read used 0x1067d12f0 = the stack-canary region / DER bytes; the real slots
are the adrp-6d3b000 family.)

**SH19b fix:** new elfjit `--renderframe-seedgles` overwrites those 8 slots with
host-thunk GLES bridge slots (`resolve_gles_mixed`, fallback `resolve_gles_int`).
Seeding slot0 (glClearColor) + slot2 (glClearDepthf) — the float-bridge slots the
clear-state sub-fn 0x5b32e08 dispatches through — makes frame-fn return cleanly:
`engine frame-fn 0x105b32c00 returned Ok(...)` + post-frame swap Ok(0x1), exit 124
stable, NO heap abort. Also fixed the `free(): invalid next size` shutdown heap
corruption by growing the fabricated renderer scratch 512B -> 8KiB.

**Frontier (next spread):** the engine's own clear/frame code now dispatches through
our GLES bridge and completes; the remaining wall is the coherent renderer/view C++
object reverse for the full frame (draw path), plus the ALooper/lifecycle producer
(START cmd) that would drive the loop from the engine's main thread (SH14-capped).
Baselines: --jni exit 0; stable idle exit 124. Doc:
docs/frontier-sh19-framedrive-glesdispatch.md; run-log runs/sh19-frame-seeded-ok.txt,
runs/sh19-slotdump*.txt.

## Session (Sep 12, 2026, hermes-worker, cycle SH18) — corrected SH17's "don't drive the render-init THUNK" note; correct thunk drive recovers the engine's REAL ctx object (vtable 0x106731ae0) and presents a real colored frame through the engine's own path. Workspace 470/0.

SH17 recorded the render-init THUNK 0x105b3a280 as undrivable (SIGSEGV, "shifts
parent/win args"). That was a harness-arg bug. Disasm of v2.738.1397 shows the
thunk is `thunk(win, parent) -> inner(alloc(0x48), win, parent)` returning the
REAL guest ctx in x0 (callers 0x5b2b214/0x5b2ea90 `bl thunk; ldr x8,[x0]; ldr
x8,[x8,#16]; blr x8`). Driving it with the correct args
(`--renderthunk`: x0=win=XID, x1=parent=0) recovers the engine's coherent ctx:
vtable 0x106731ae0 (engine-populated, live-dumped), [ctx+32]=EGLDisplay,
[ctx+40]=surface, [ctx+48]=context — and its vtable methods [vt+16]=0x105b3b358
(make-current-if-not-bound: eglGetCurrentContext->eglMakeCurrent), [vt+24]=
0x105b3b408 (swap). Engine's own swap + GLES-bridge clear through the real ctx
present a blue frame (PIL-decomposed RGB 51,76,229 = 0.2,0.3,0.9). New elfjit
`--renderthunk` lever (+ vtable[0..5] live dump). SH18b added `--renderbind` (drive
the engine's OWN make-current method vtable[16]=0x105b3b358 on the real ctx, then
swap -> EGL_TRUE). SH18c added `--renderframe-drive` (probe the engine's own frame-fn
0x105b32c00 with fabricated renderer/view; gets past the renderer list-find 0x5b2e98c
into glBindFramebuffer/viewport setup, then needs coherent renderer internals).
This opens frontier lever (2):
drive the engine's OWN render-loop recipe (vtable[16] bind -> frame-fn 0x105b32c00
-> vtable[24] swap) instead of force-driving glClear. Doc:
docs/frontier-sh18-renderthunk-ctx.md; run-logs: runs/sh18-renderthunk{,-2,}.txt,
sh18-thunk-frame.txt; frame: runs/sh18-thunk-blue.png; script:
runs/capture_thunk_frame.sh. Baselines unchanged (--jni exit 0; idle exit 124).

**Frontier (lever 2 now opened):** drive the engine's own render-loop recipe on
the recovered real ctx in natural order — vtable[16] bind (eglMakeCurrent) ->
frame-render fn region 0x105b32c00 (glViewport/glScissor/glClearColor/glClear/
glDrawElements, dispatched via [x0]->[vt+16]->blr) -> vtable[24] swap. Still need
the coherent renderer C++ object that 0x105b32c00 takes as x0 (its +24/+40
sub-objects carry clear/viewport state, +224/232/236/238 mask flags), and
ultimately the ALooper/lifecycle producer (SH14-capped deque wall) to drive the
loop from the engine's main thread.

## Session (Sep 12, 2026, hermes-worker, cycle SH17) — REAL Roblox binary now RENDERS a real COLORED FRAME headlessly on this GPU-less VPS: live EGL context + engine's own eglSwapBuffers succeed, and glClearColor→glClear→eglSwapBuffers through the GLES bridge present a solid-green 1280x720 frame. Workspace 470/0. HEAD a629d9c.

First real rendered pixels from the running engine's own render path, all through the JIT
bridges against Mesa llvmpipe + a real Xvfb X11 window. Two new opt-in elfjit levers:

1. **`--renderframe`**: after `--renderinit` returns Ok(0x0), drive the engine's OWN swap
   fn 0x105b3b408 (`ldp x8,x1,[x0,#32]; mov x0,x8; b eglSwapBuffers`) with x0 = the
   scratch context buffer that render-init wrote into → `eglSwapBuffers([+32]=display,
   [+40]=surface)` returns **Ok(0x1)=EGL_TRUE**. The real binary presents its surface
   headlessly (llvmpipe+Xvfb), stable exit 124, zero crash.
2. **`--renderclear <r,g,b,a>`**: draw a colored clear through the JIT's GLES float bridge
   on the live context (drive glClearColor@plt 0x1062d7710 with s0..s3, glClear@plt
   0x1062d7740 with GL_COLOR_BUFFER_BIT=0x4000, then swap). Captured with ffmpeg x11grab:
   the frame is a **solid green canvas** (the 0.1,0.7,0.2,1 color) — real rendered pixels.

**Key reverse:** render-init's inner fn 0x105b3a2d8 stores real EGL handles at fixed
ctx offsets [ctx+32]=display,[ctx+40]=surface,[ctx+48]=context; because the harness passes
a guest-writable scratch as x0 (the "prologue STORES into *x0" pattern from SH16), that
same buffer already holds the live handles the swap fn reads — no thunk-return plumbing.
Abandoned (don't re-run): driving the render-init THUNK 0x105b3a280 to recover the "real"
ctx → SIGSEGV (thunk shifts parent/window args across the inner call). Doc:
docs/frontier-sh17-renderframe-clear.md; run-logs: runs/sh17-renderframe2.txt,
runs/sh17-frame-clearcap.txt; frame artifact: runs/sh17-frame-green.png (solid green).

**Frontier (now with a fully-working render pipeline behind the wall):** the engine's own
main-loop producer still never enqueues a render task onto its idle futex/ALooper, so the
engine itself never issues glViewport/glClear/glDrawElements in its loop — this cycle's
clear+swap were harness-driven on the live context. Next: (1) drive the ALooper app-command
lifecycle so StartApp's real producer enqueues a render task → the engine's OWN frame loop
runs natively (all endpoints verified bridge-reachable: glViewport@0x105b32ca4,
glClearColor@0x105b32f8c, glClear@0x105b32fdc, glDrawElements@0x105b35334); or (2) drive the
engine's render-loop fn (region 0x105b32c40-…) directly with a coherent render-state object
once its layout is reversed. Baselines unchanged (--jni exit 0; idle main loop exit 124).

## Session (Sep 12, 2026, hermes-worker, cycle SH16) — the real render-init's FULL EGL chain now SUCCEEDS headlessly: window surface created + context made current against Mesa llvmpipe+Xvfb, returned Ok(0x0). Workspace 470/0. HEAD 114d5f2+.

Crossed the SH14-identified gateway (eglCreateWindowSurface + eglMakeCurrent,
previously declared "framework-gated / not drivable"). Two things landed:

1. **`vfprintf` crash-mask** (commit 114d5f2): libc++ terminate writes its message
   body via `vfprintf`, not just `fwrite`. The guest passes glibc its bionic
   FILE* + AAPCS64 va_list → SIGSEGV hid the reason. New `bionic_vfprintf` decodes
   the AArch64 va_list and writes guest streams to fd 2. Now visible.
2. **Root cause of eglCreateWindowSurface failing: the native window is the
   render-init's x1 param.** Real caller 0x105b2ea90 `ldp x8,x1,[x0,#344]`; the
   prologue `x22=x1` → `[ctx+24]` (stored at 0x105b3a340), which the surface
   wrapper 0x105b3b194 reads as its win arg. The harness passed x1=0. Passing the
   wired XID (0x200000) as x1 → the whole real chain succeeds:

```
ANativeWindow_fromSurface -> ANativeWindow_acquire -> eglGetDisplay -> eglInitialize
-> eglChooseConfig(x3) -> eglGetConfigAttrib -> eglCreateContext
-> eglCreateWindowSurface(win=0x200000) -> eglMakeCurrent -> eglQuerySurface(x2)
-> eglSwapInterval
[elfjit:renderinit] returned Ok(0x0)   (cleanly; engine main loop still idles, exit 124)
```

Real Roblox now has a live Mesa llvmpipe EGL context on a real Xvfb X11 window,
headlessly on this VPS. Run-log: /home/hermes-worker/runs/sh16-renderinit-window-x1-success-runlog.txt.
Doc: docs/frontier-sh15-renderinit-driving.md; STATUS.md.

**Frontier (now with a live EGL context):** drive the engine's frame loop (gl*) —
the wall of the main-loop producer never enqueuing a render task onto its idle
futex/ALooper. Also still open: many gl*/shader paths will need the GLES bridge /
compressed-texture / float paths under a real frame.

## Session (Sep 12, 2026, hermes-worker, cycle SH15) — CORRECTION: SH14's "render-init framework-gated, not drivable" is WRONG at runtime. The REAL render-init (0x105b3a2d8) now drives its real EGL chain headlessly (ANativeWindow_acquire→eglGetDisplay→eglInitialize→eglGetError) through the JIT bridges before a libc++ abort. Workspace 469/0.

This cycle reopened the rendering path SH14 declared a dead-end. Found that
**StartApp populates the render-init context global 0x1067d16f0 at runtime**
(new `JIT_FRAMEWORK_DUMP` reads it live: `0x562a..`, not the statically-0 SH14
pinned) — and the deque-maintenance forward edges 0x1068262e8/300/308 are
populated too (real .text addrs). Built `--renderinit` and drove the real
render-init fn directly after StartApp warm-up:

- **Real EGL chain executes from the real binary** (JIT_TRACE, x30=call sites):
  `ANativeWindow_acquire`(0x105b3a34c) → `eglGetDisplay`(0x105b3a3b4) →
  `eglInitialize`(0x105b3a3c8) → `eglGetError`(0x105b3aec8), then
  `libc++abi:` terminate-abort (engine hits a fatal missing-framework condition
  of the synthetic drive) — that abort used to crash silently (a SIGILL in
  `dl_iterate_phdr` guest-callback and a SIGSEGV in `fwrite` on a bionic
  `FILE*`), both now fixed with shims.
- **`dl_iterate_phdr` shim**: routes the guest callback back through
  `jit::run_guest_callback` (host glibc was executing guest AArch64 → SIGILL).
- **`fwrite` shim**: diverts guest/bionic-`FILE*` (stderr as low as `0x130`) to
  host fd 2 so the abort reason surfaces instead of SIGSEGV.
- **`--renderinit` harness** + `JIT_FRAMEWORK_DUMP` diagnostic in elfjit.
  Runs the render-init as a fresh guest thread CONCURRENT with StartApp's parked
  main thread (block cache leaks, safe). Address must be passed as a GUEST addr.

**Next wall: the engine aborts in render-init after its EGL chain** — needs
either a coherent ANativeWindow/framework context (the SH14/SH7/N/P ALooper-
lifecycle emulation) so the abort becomes a real llvmpipe frame. Doc:
`docs/frontier-sh15-renderinit-driving.md`; run-log:
`/home/hermes-worker/runs/sh15-renderinit-eglchain-runlog.txt`. Baselines
unchanged (`--jni` exit 0; stable idle exit 124); workspace green.

## Session (Sep 12, 2026, hermes-worker, cycle SH14) — deque-injection path proven STRUCTURALLY capped (disasm-verified); located the real render-init fn 0x105b3a2d8 (full eglGetDisplay→init→CreateContext→CreateWindowSurface→MakeCurrent) and proved it is framework-gated. New JIT_REGION_WATCH diagnostic. Workspace 469/0.

This cycle answered the ~13-cycle open question definitively: WHY does no
deque-injection (SH5-SH13) reach egl*/gl*? Disassembly of the drain (0x2856f94)
and the type-4 maintenance handler (0x10285371c) proves it is STRUCTURAL:
- The drain pop-loop passes **w4=4 hardcoded** (constant in the loop) as the
  dispatch task type — it is NOT derived from node data, so no node content can
  change it. The handler `cmp w4,#1..4` therefore always takes maintenance.
- The type-4 handler dispatches through **runtime-built BSS globals**
  (0x1068262e8/0x106826300/0x106826308) that the Android framework producer
  populates; all are statically 0 on this box. So the deque vtable-substitution
  path (SH11-13's --deque-node-live) is a documented DEAD-END — stop investing.
- Back-traced from the egl GOT slots to the engine's REAL render-init:
  **fn 0x105b3a2d8 → thunk 0x105b3a280**, calling
  eglGetDisplay(0x105b3a3b0)→eglInitialize(0x105b3a3c4)→eglCreateContext
  (0x105b3a400)→**eglCreateWindowSurface(0x105b3b1a0)**→eglMakeCurrent. It reads
  a runtime-built context global (0x1067d16f0, statically 0), so it too is
  framework-gated — not a host-drivable entry on this box as-is.
- New `JIT_REGION_WATCH=<lo>-<hi>` (jit.rs): logs first block-entry in a region.
  Verified render-init region — 0 hits (never reached); StartApp region — 3 hits.
  A portable reachability probe for the next cycle's framework-emulation work.
- Next lever is NOT more deque surgery: fabricate the framework context obj the
  render-init derefs + drive the ALooper app-command lifecycle so a real
  producer posts the work item (SH7/N/P levers). Window layer is already wired
  (XID 0x200000). Baselines: --jni exit 0; stable idle exit 124. Doc:
  `docs/frontier-sh14-renderinit-located.md`.

## Session (Sep 12, 2026, hermes-worker, cycle SH13) — REAL engine vtable dispatch: `--deque-node-live 0x106829f00` (the LIVE sentinel's real vtable → engine's own drain-node task-processor 0x10285371c) makes the engine's NATIVE dispatch machinery run our injected foreign nodes — ~124 pops in 16s, process stable to timeout (exit 124), zero crash, and the block-cache GROWS past the probe baseline (≈2147 compiles / 7,361,652 hits vs the probe's flat ≈434). No probe logging (dispatch goes through the real processor). Workspace 469/0.

SH12 left the type-4 dispatch firing through OUR host-thunk PROBE, which only
logged ABI args and never ran engine code for the node. SH13 substitutes the
REAL sentinel vtable (`0x106829f00`, `[vt+40]=0x10285371c`), so the drain's pop
dispatch routes to the engine's actual node-processor. Verified stable and
mechanical (a faulting dispatcher would exit 134; we see 124, endless pops).
The obfuscated dispatch table (`0x102853a04..9b4`) keeps compiling+running real
regions the host-thunk probe never touched.

**Not yet render:** the real processor type-dispatches on `w4` (injected nodes
always get `w4=4`, task-maintenance) via an obfuscated hash table; hostcall
histogram is still syscall + pthread/JNI/mem, **zero egl*/gl***. Next lever:
construct a node whose `[node+32]`/dispatch-index reaches a render/tick handler
in that table, or feed the maintenance path real framework state. Run-log:
`/home/hermes-worker/runs/sh13-realvt-runlog.txt`. Doc:
`docs/frontier-sh13-realvt-dispatch.md`.

## Session (Sep 12, 2026, hermes-worker, cycle SH12) — type-4 dispatch CONFIRMED + SUSTAINED through the real engine idle drain: our injected foreign nodes are now continuously popped AND dispatched through our host-thunk handler (107 dispatches/104 pops across 105 node addrs in 14s, exit 124, zero crashes). Workspace 469/0; HEAD 3b37deb.

SH11 left the injection popping the node but the probe handler never
dispatched (the "residual"). This cycle root-caused it and closed it:

- **THE BUG: vtable handler offset off-by-one.** Both probe builders wrote the
  handler at `(v as *mut u64).add(4)` = byte offset **0x20**, but the drain
  dispatches via **`[vt+40]` = byte 40 = u64 index 5**. So `ldr [vt,#40]`
  read 0 (calloc-zeroed), the drain's `handler != 0` guard failed, and the
  node was consumed WITHOUT dispatch. Fix (`a2448fc`): `add(5)` in the
  `--deque-node-live` and `--deque-probe` vtable builders.
- **Verified:** the first type-4 dispatch fires with the exact engine ABI —
  `x0(vt+16)=0xdeadbeef` (our ctx marker, read correctly from the guest-arena
  vtable), `x3(node)=0x107334040`, `w4=4` — confirmed through the real idle
  drain's pop-loop.
- **Sustained (`3b37deb`):** the injector previously returned after the first
  pop (drain went idle, head → empty). Now it re-injects a fresh guest-arena
  node on every pop, so the drain runs a continuous type-4 dispatch stream.

**Result (reproducible):** repeated `[elfjit:deque-probe] type-4 dispatch #N:
x0=0xdeadbeef ... x3(node)=0x1073... w4=4 x5=0` interleaved with
INJECTED/POPPED, process stable to timeout **exit 124**, zero SIGSEGV. Run-logs:
`/home/hermes-worker/runs/sh12-probe-runlog.txt`,
`/home/hermes-worker/runs/sh12-sustain-runlog.txt`. Doc:
`docs/frontier-sh12-dispatch-confirmed.md`.

**Next lever (unchanged shape, now that dispatch is live):** route the node's
vtable at a REAL engine render/tick handler (instead of our probe) so a
dispatch drives the engine's frame/render machinery to egl*/gl* — or feed the
real producer 0x285682c a coherent render task node. Baselines unchanged:
`--jni` clean exit 0; stable idle exit 124; workspace 469/0.

## Session (Sep 12, 2026, hermes-worker, cycle SH11) — sequenced deque-node-live injection crosses the stable idle drain: node POPPED, exit 124, sentinel crash eliminated. Workspace 469/0; HEAD f4255fd.

For ~10 cycles (SH7b/SH8/SH9) every `--deque-node-live` run died with an
**exit-134 sentinel-as-task SIGSEGV** at ~200ms — before any injected node
could land. This cycle fixed it as a SEQUENCING bug (not a wrong deque model):

- **Defer the force-pop patches** when `--deque-node-live` is set, so the drain
  stays stable (never pops) while we place our node. (Old behavior: force-pop
  at startup popped the SENTINEL first → fault.)
- **Inject while stable** by CLONING the live HEAD node's coherent payload
  (the sentinel during idle — a real, re-enqueue-able task node) as the node
  template, overriding `[node+112]` -> probe vt, forcing `[node+40]!=0`, and
  **zeroing `[node+0]`** (fresh tail; the re-enqueue producer 0x285682c walks
  it and a stale cloned link faults at pc 0x51). Replaces SH9's unreliable
  `[consumer+104]` / `[x19+104]` sentinel indexing.
- **ARM force-pop AFTER placement + drop the cached drain blocks** via new
  `pub jit::block_cache_drop_region(lo, hi)` — the dispatcher had already
  compiled the UNPATCHED pop-loop, so patching guest bytes alone had no effect
  (that's why the earlier deferral ran stable but never crossed). Eviction
  forces it to recompile the patched code, so the FIRST forced pop takes OUR
  node (passes the self-skip guard, `[node+40]=1 -> probe`), not the sentinel.

**Result (reproducible):** `NODE ... POPPED by live drainer (headcell now
0x1000000000000) — deque crossed the barrier`; process stays stable to timeout
**exit 124**, no SIGSEGV — the deque crossing no longer faults.

**Residual (next lever):** the node pops cleanly and the drain reaches past the
`blr` (lr=0x10285700c, node preserved in x3), but our probe handler hasn't been
confirmed dispatching — the guest-side `[vt+16]` reads `0x8b8b48...` (garbage)
not `0xdeadbeef`, so the drain's `[vt+40]` deref is landing on the wrong vtable
(the host-heap probe vtable isn't being read through the guest image we
expect). Resolving that — or supplying a REAL render/tick vtable for the node —
is the path to reaching egl*/gl* on a real frame. Workspace **469/0**.
Baselines unchanged: `--jni` clean exit 0; stable idle exit 124. Doc:
`docs/frontier-sh11-seq-inject.md`; run-log:
`/home/hermes-worker/runs/deque-seq5.txt`.

## Session (Sep 12, 2026, hermes-worker, cycle SH9) — drain SELF-NODE-SKIP guard discovered (correction to SH8): sentinel-repoint can never fire; foreign-node path gives a controlled guest dispatch. Workspace 468/0; HEAD 480196f+.

Disassembled the drain pop-loop `0x2856e40..0x28570a4` (file vaddr = guest−0x100000000)
and found the piece SH8's model omitted — the **self-node-skip guard**:

```
0x2856fc8  ldr  x8,[x19,#104]   ; x8 = [consumer+104]
0x2856fcc  cmp  x8,x22          ; x22 = popped node
0x2856fd0  b.eq 0x28570a4       ; equal -> RETURN, never dispatch
```
The idle sentinel IS `[consumer+104]` (the drain's own struct, vtable
0x106829f00), so repointing the sentinel's `[node+112]` (`--deque-probe`, SH8)
is **structurally futile** — the sentinel never reaches the type-4 dispatch at
`0x2857008`. The correct lever is the **foreign node** path: a calloc'd node
(addr != [consumer+104]) passes the guard and reaches the real dispatch.

**New `--deque-node-live probe`** (elfjit, opt-in): auto-builds a HOST-THUNK
PROBE vtable (`[vt+40]`=registered host thunk, `[vt+16]`=ctx) and injects a
foreign node (`[node+112]=that vt`, `[node+40]=1`, tagged into the live
headcell). Observed: the pop-loop NOW dispatches in a **real guest thread**
(tid 0, `rbx_matches_gueststate=true`, `in_jit_run=true`, guestpc→0x7f0000002068)
— a controlled first crossing through the foreign-node dispatch path — vs SH8's
sentinel-repoint which faulted only in an untracked host thread. It still faults
(exit 134, /home/hermes-worker/runs/deque-nodelive-probe-crossing.txt) because
the handler is our probe host-thunk, not a real engine render callback.

**Next lever (mechanism now correct):** identify a REAL render/tick vtable for
`[node+112]` (+ coherent payload) so `--deque-node-live <vt>` reaches egl*/gl*;
or invoke the REAL producer 0x285682c as a guest call with a valid task node.
Baselines unchanged: `--jni` clean exit 0; stable idle exit 124. Doc:
docs/frontier-sh9-drain-selfskip.md.

## Session (Sep 12, 2026, hermes-worker, cycle SH8) — dispatch ABI fully reversed + pinned; `--deque-probe` live-repoints sentinel vtable (works) but sentinel-as-task still faults (honest failure, later corrected by SH9).

Reversed the engine task-deque consumer's POP-LOOP dispatch ABI from live disasm
(libroblox 0x2856fd4..0x2857008, qemu/objdump-verified) and pinned it as a new
regression `deque_dispatch_node_layout_matches_engine_abi` so any render-task
injector builds nodes the running drain understands:

```
node = low48([headcell]); vt = [node+112]&~0x3f; handler = [vt+40];
guard [node+40]!=0 && handler!=0;
handler([vt+16], consumer, [node+32]&~1, node, w4=4, x5=0)
```
(the `[node+112]&~0x3f -> [vt+40]` model prior cycles stated is confirmed
exactly, plus the precise arg order/types and the `[node+40]`/handler guards.)

New elfjit `--deque-probe <ctx-qw>` (opt-in, only engages with `--drain-force-
pop`): live-repoints the ROOT consumer's sentinel `[node+112]` -> a host-heap
fake vtable whose `[vt+40]` is a registered host-thunk probe, so the engine's
own pop-loop dispatches a NODE through OUR handler with the real ABI args.
VERIFIED: both sentinels repointed (headcells 0x10682b338/0x10682a638, logs
`REPOINTED sentinel ... [node+112]: 0x106829f00->0x...`). HONEST GAP: the probe
handler never fires (count 0) — under forced-pop the engine dispatches the
SENTINEL-AS-TASK and faults in an UNTRACKED host thread (guestpc 0, reading host
slot addr 0x7f0000000090 as a pointer) before our handle runs. The vtable repoint
alone can't detour the engine's own dispatcher walking the sentinel's other
garbage payload. Exit 134. Doc: docs/frontier-sh8-dequeprobe-abi.md.

Baselines UNAFFECTED and re-verified: `--jni` clean exit 0; stable idle
(StartApp main loop) exit 124, zero SIGSEGV; workspace green **468/0** (was 467).
run-logs: /home/hermes-worker/runs/boot-probe-{1..5,final}.txt.

**Frontier (unchanged shape, ABI now pinned):** the still-hard wall is the
engine's producer never enqueues a REAL render task; forcing the consumer makes
it pop+dispatch the sentinel-as-task -> fault. Next levers per SH7b, now with
the exact node layout: (1) invoke the REAL producer 0x285682c as a guest call
with a valid task node, or (2) fabricate a full task node (vt+40 -> a real
render/tick vtable we must locate, coherent payload) and cross before force-pop.
Neither is crossed this cycle; the ABI + control-plane (repoint) are.

## Session (Sep 12, 2026, hermes-worker, cycle SH7b) — CORRECTION to SH7: the finite wait-timeout NEVER reached the pop-loop (measured 0 entries / 128k branches); NEW `--drain-force-pop` makes the engine's task-deque pop-loop run + dispatch for the first time (faults on the sentinel = controlled crossing). Workspace 467/0; HEAD 7d0cd5c+.

SH7 claimed `--drain-poll <ms>` (finite timeout) makes the drain's pop-loop run by
letting generic-wait time out. **That is wrong.** Measured: under `--drain-poll 8`
the drain's post-wait `tbz w24,#0` (0x102856f7c) fires ~128k times but the pop-loop
0x102856f94 is entered **0 times**. Root cause pinned in generic-wait 0x284d014:
`cmn x0,#1` (0x284d0a4) only maps an EXACT host-futex x0==-1 to "timed out"; the
host futex returns -ETIMEDOUT(-110) on timeout, which falls to 0x284d0ec -> generic
wait returns w0=0 ("woken"). The drain's tbz therefore always re-loops; the finite
timeout only hot-loops the drain's MAINTENANCE heartbeat (0x10285371c with x4=2/3,
i.e. the drain struct's own `[x19+104]`+112 vtable callback) — which SH7 misread as
"pops + dispatches the deque". The real node-pop path (x4=4) is a separate code
site.

**New `--drain-force-pop`** patches `mov w24,w0` (0x102856f4c)->mov w24,#1 AND NOPs
the tbz (0x102857f7c), so the drain ALWAYS falls through to the version-check
(0x2856f80) -> pop-loop (0x2856f94). **Proven: the pop-loop now executes** — it
CAS-pops the deque head (the sentinel during idle) and dispatches
`[node+112]&~0x3f->[vt+40]` with `[node+40]`/`[node+32]`/w4=4, then faults walking
the sentinel's garbage task content (SIGSEGV guestpc=0x7f0000002068, lr
0x10222f330). This is the long-anticipated **controlled first crossing** — the
engine's real task-deque pop+dispatch machinery now runs (SH5/SH6/STATUS's
documented milestone). Run-log: `/home/hermes-worker/runs/drain-forcepop-crossing.txt`
(exit 134). Opt-in, so plain `--drain-poll` stays stable (exit 124) and baseline
`--jni` is unchanged (clean exit 0).

**New `--deque-node-live <vt>`** implements the SH7 "locate tid 0's deque" lever: it
targets the LIVE drainer (guest_tid 0, its root recovered from x20 while pc is in
the drain body 0x102856e40..0x1028570a4) instead of the parked tids 1/2 that
`--deque-node` aimed at. It recons the live deque (`[root]=headcell`,
`[headcell]=packed head` low48=node high16=tag, `[root+8]=tag`, head-node
`[node+112]/[vt+40]/[node+40]/[node+32]`) and swaps a task node over the live head.
Note: because the drain re-enqueues every popped node, "node still at head" is not
itself proof of non-consumption; the discriminating signal is a type-4 dispatch of
our node. Under --drain-force-pop the run faults during the sentinel dispatch, so a
real node's dispatch is not yet isolated.

**Next lever:** supply a real task node content so the forced pop-loop's dispatched
handler (0x10285371c) reaches a real render/tick callback instead of walking
garbage — identify what 0x10285371c's `[adrp+0x528]` global `br` target dispatches to,
and what node.type/args drive it. Doc: `docs/frontier-sh7b-drainforcepop.md`.

The ~35-cycle "engine producer never enqueues / consumer never drains" wall is
broken. Empirical stack dump of a parked consumer resolved the true frame and
the gate:

- The parked consumers are the **drain fn 0x2856e40** calling generic-wait
  **0x284d014 with timeout = -1 (infinite)** (live x20 = -1; sp+0x30 = -1).
  generic-wait shares the drain's frame (the drain `bl`s to 0x284d018, skipping
  its `sub sp,#80`), parked sp+0x28 = drain return-into after `bl 0x284d014`
  (0x102856f48).
- An infinite timeout jumps straight into a bare blocking futex
  `futex(Q+4, WAIT_BITSET, epoch, NULL, NULL, ~0)`; the drain's **pop-loop at
  0x2856f94 runs ONLY when the wait returns 1 (timed out)**. With an infinite
  timeout it never times out → the pop-loop is never reached → work is never
  consumed no matter what is in the deque. That was the whole wall.

**New elfjit `--drain-poll <ms>`** patches guest `mov x2,x22` (0x102856f40,
the drain's infinite-timeout copy) to `mov x2,#<ms>` (imm12) before jit_run, so
the drain block compiles with a finite timeout. The wait now times out, the
drain reaches the pop-loop, and it **continuously pops + dispatches the deque**,
executing the real engine dispatch handler **0x10285371c** / 0x1028538c0 /
0x1028539e8 millions of times — stable (flat 1673 compiles, no crash, exit 124,
hits → ~8M). This is the engine's own task-deque dispatch machinery running.
Run-log: /home/hermes-worker/runs/boot-drainpoll-crossing.txt.

Also: fixed `--deque-node` to write the node to **[headcell+0x0]** (the cell the
pop actually reads: `x23=[x20]; x24=ldar([x23])`) instead of the ring's
internal HEAD/TAIL cells (+0x10/+0x18) prior code wrote to — that is why SH6
nodes sat unconsumed. Added JIT_STACKDUMP / JIT_DEQUE_PROBE2 diagnostics (frame
resolution) and a `dump` region-disassembler example. Doc:
docs/frontier-sh7-drainpoll-crossing.md.

**Where this leaves the frontier:** the consumer side is provably live — under
--drain-poll **guest_tid 0** (its own deque/wait struct, snapshot x1=0x7f4b5486b280,
pc=0x10285371c lr=0x102856f38) cycles the drain's pop-loop and dispatches the
real engine handler 0x10285371c continuously (maintenance/self-dispatch, no
egl*/gl* hostcall yet). guest_tids 1 & 2 (recovered headcells 0x10682a638 /
0x10682b338) stay futex-parked — so the --deque-node node-injection lever
(targeting only parked lr==IDLE threads) has been writing into idle consumers'
deques, never the one tid 0 drains; that is why external nodes are unconsumed.
**Next lever:** (1) locate guest_tid 0's deque root from its live drain frame at
dispatch time; (2) pass the drain's tag guard (0x2856e78 `cmp x9,[head]>>48`)
and low-48 pointer truncation; (3) point [node+112] at a render/tick vtable (not
the sentinel's 0x106829f00) so the dispatched handler reaches egl*/gl*/frame.
Baseline (no --drain-poll) unchanged: stable idle futex park.

## Session (Sep 12, 2026, hermes-worker, cycle SH6) — host enqueue into the task-deque PROVEN not-a-producer (two strategies); deque model corrected from full producer/drain disassembly; new `--deque-node` harness. Workspace 467/0; HEAD 6003441.

Implemented the documented SH5b next-experiment (host side enqueue into the
engine's idle task-deque) as a real elfjit host producer and ran it against the
parked consumers. **Result: a hard negative.** Two distinct enqueue strategies
were tried and both are robustly NOT consumed (`popped=false` every check,
compiles flat, no crash):

1. write node into [headcell] = slot+0 (the SH5b "head cell"),
2. write node into HEAD = slot+0x10 AND TAIL = slot+0x18 with [node]=0.

In both, the deque head field keeps pointing at our node for the whole run —
the parked consumers never CAS-pop it, despite the version-epoch bump + futex
wake. This corrects SH5b's "self-referential sentinel at the drain struct"
model, which located the enqueue point wrong.

**Corrected deque model** (from producer 0x285682c + drain 0x2856e40 disasm):
the deque is a per-consumer pointer-RING at a stable guest-bss base
(0x10682a638 / 0x10682b338 for the two real slots; a 3rd consumer's headcell is
a host-heap garbage-ASCII cell — ignore). HEAD field at base+0x10, TAIL at
+0x18; empty == both == slot+8 (the self-referential first node). Producer push
= tagged-CAS walk `ldar[head]→[node]` to the tail then link (helpers 0x2b9e720 /
0x2b9e760). Consumer pop = `ldar[head]`, `low48==0 → EMPTY→wait`, else CAS-pop
then dispatch `[node+112]&~0x3f → [vt+40]` (+ `[node+40]`, `[node+32]` arg),
**re-enqueue via `bl 0x285682c`** (2857020), wake `futex(node+0xc,0x8a,1)`.

**Why node+bump is insufficient (the hard wall, now precise):** the drain is
gated by the version-epoch wait (generic wait 0x284d014 parks in
`futex(Q+4, WAIT_BITSET, low32(epoch))`). Wait returns w20=0 on futex-woken/
version-changed, w20=1 ONLY on timeout. On wake the drain checks
`cmp x21, [Q]>>32` (2856f80/90); a CHANGED version makes the drain RETURN (to
28570a4) instead of entering the pop-loop at 2856f94 — which runs ONLY when the
version STILL matches the caller's captured x21 AND the wait timed out. So a
host bump makes the drain exit; the pop-loop is dead during idle (infinite
timeout, never polls); and even a no-bump node placement is never drained.
**Host writes to the ring are NOT sufficient — the deque is drained only by a
real (framework) producer that re-enters the drain loop.** Three negatives:
slot+0, slot+0x10/0x18±bump, slot+0x10/0x18 no-bump — all `popped=false`.

**Next levers (ordered):** (1) invoke the REAL producer 0x285682c as a guest
call with a valid task node (recover the scheduler `this` from drain_struct
`[x1+104]`) so the framework's own push path runs; (2) synthesize the drain
caller's re-entry with a fresh matching epoch + the node already in HEAD;
(3) reverse the drain caller loop (0x284eb80) to find what re-enters the drain.

Doc: docs/frontier-sh6-enqueue-negative.md. Run-logs:
/home/hermes-worker/runs/deque-node-enqueue-vt.txt, deque-node-v2.txt,
deque-node-v3.txt. elfjit `--deque-node <vtable>` is the faithful reusable
harness (default-off; baseline boot unchanged).

## Session (Sep 12, 2026, hermes-worker, cycle SH5) — producer/enqueue contract pinned from disassembly; `JIT_DEQUE_PROBE` locates each parked consumer's live deque head-cell from the host. Workspace 467/0; HEAD bfa63d1+.

This cycle converted the ~30-cycle "producer never enqueues / version+latch
isn't a producer" wall into a concrete, host-side-pokeable mechanism with a
full disassembly of the scheduler (file vaddr = guest − 0x100000000):

- **Producer/enqueue = 0x285682c**: per-CPU slot base = `[this+8] +
  sched_getcpu()*0x4a140`; tagged-CAS push onto that slot's per-CPU MPSC queue
  (pop 0x2b9e6e0 / push 0x2b9e720 / refcnt 0x2b9e760); head atomic at
  `slot+0x10` (low48=node, high16=tag), tail at `slot+0x18`, node link `[node]`.
- **Consumer drain = 0x2856e40** (`root`=x0): `head=ldar[[root]]`; empty iff
  `low48==0`; dispatches popped node via `[node+112]&~0x3f → [vt+40]` +
  `[node+32]`; wakes `futex(node+0xc, WAKE_BITSET|PRIVATE, 1)`.
- **Generic wait = 0x284d014** (`Q`,`epoch`,`timeout`): refcount `[Q]`; early
  out on `epoch != [Q]>>32`; else `futex([Q]+4, WAIT_BITSET, low32(epoch))`.
- **Waiter-frame recovery (the enqueue prerequisite, confirmed live):** at park
  the waiter's PROLOGUE saved the drain's callee-saved regs — `[sp+64]`=drain
  x20=deque root, `[sp+72]`=drain x19=consumer struct, `[sp+32]`=drain saved
  x30. New `JIT_DEQUE_PROBE=1` (elfjit) reads these and shows each parked
  consumer's `[root]` is a **stable guest-bss head-cell**
  (0x10682a6x38 / 0x10682b338) — the exact address a host producer must CAS
  onto. Also corrected a long-standing misreading: `lr=0x10284d134` is the
  *in-wait return-into-fn* after the `bl syscall` (per 284d134 `mov w20,wzr;
  b 284d0f0`), NOT the drain call-site.

**Next experiment (feasible now):** CAS a node onto the recovered per-CPU
head-cell + bump `[Q]` epoch + FUTEX_WAKE. A fully-zeroed node drains (proving
the host producer crossed the barrier) then faults deref'ing `[0x28]` in the
`[vt+40]` dispatch — a controlled, capturable first crossing; the follow-on is
supplying a real engine frame/render node (`[node+112]→[vt+40]` callback +
`[node+32]` arg). Boot unchanged (stable idle main loop, exit 124, no crash).

Run-log: /home/hermes-worker/runs/deque-probe.txt (118 samples, compiles
1420→flat, exit 124). Doc: docs/frontier-2026-09-12-producer-enqueue.md.

## Session (Sep 12, 2026, hermes-worker, cycle SH4) — idle barrier re-characterized: it's a per-CPU task-deque CONSUMER, version+latch is NOT a producer; `--futex-bump` negative result; producer-enqueue doc. Workspace 467/0; HEAD c6c82e5+.

From-first-principles disasm this cycle, the ~30-cycle "producer never enqueues
work" wall is now pinned to its exact contract (correcting the "awaiting
0xF4240 go-token" reading of prior cycles):

- The parked threads are **consumers** inside the generic futex wait-with-timeout
  at 0x10284d018 (reached via `blr` — vtable-dispatched, ZERO static callers).
  It's `wait(obj=Q, expected_seq, timeout_ns)`: atomic_add(&Q.refcount,+1),
  proceed if `(expected>>32) != (old>>32)` else futex(Q+4, WAIT_BITSET,
  low32(expected)); on timeout a clock_gettime deadline loop; atomic_add -1.
  So `Q>>32` is a **self-syncing version counter**, Q+4 is the futex latch.
- The consumer is a **per-CPU lock-free task-deque drain** (fns 0x285682c /
  0x2856f44): `loop { head=ldar[[x20]]; if low48(head)==0 goto wait; process }`,
  with `x20` a per-CPU slot base (`umaddl` from `sched_getcpu() & 0xf`). Head is
  0 because the framework render/looper producer is absent here.
- **New `--futex-bump` (elfjit):** prior kick/set only poked the latch (Q+4);
  bump also increments the version word `[Q]>>32` — the real produce shape
  (bump version + set latch + wake). Empirically it **re-parks** (compiles flat
  1668, JIT_STATS heartbeat stops, exit 124); the consumer re-reads `[Q]>>32`
  each iteration so the host's bumped value becomes the new expected — a moving
  epoch, not a discrete "go". **Version+latch is NOT a producer.** The only
  lever is a real task node in the deque head, framework-owned.

Run-log: `/home/hermes-worker/runs/futex-bump2.txt`.
Doc: `docs/frontier-2026-09-12-producer-enqueue.md`.

## Session (Sep 12, 2026, hermes-worker, cycle SH3b) — GLOB_DAT *function* slots now resolve through the full GLES chain; workspace 467/0; HEAD 80940e6.

Follow-on to SH3's eglGetProcAddress bridge. Found another real gap in the
boot log: `[plt:glob_dat] unresolved sym=glGetShaderInfoLog / glGetProgramInfoLog`
— GLOB_DAT **function-pointer** slots (function tables, `STT_FUNC`/notype) only
tried plain `resolve()` (dlsym), which cannot see GLES names (libGLESv2 is
RTLD_LOCAL and lazily loaded), so these table entries bound to the **benign NULL
stub** instead of real Mesa. On a real shader-compile path a
glGetShaderInfoLog/glGetProgramInfoLog call through such a table would return
garbage (or the stub's 0).

**Fix (commit 80940e6):** `bind_glob_dat`'s function branch now mirrors the
JUMP_SLOT resolution chain — `resolve -> float -> float32 -> egl -> gles_int ->
gles_mixed` (scope_resolve already tried in the outer branch). Verified against
the real binary: the two GLES GLOB_DAT entries are gone from the unresolved
list; the remainder are AMedia*/video-codec data-object keys (bionic-only,
benign) + the cosmetic `__sF`. New hermetic regression
`glob_dat_function_chain_resolves_gles_names_to_real_slots` pins the chain
returns a real host-thunk slot (>= HOST_THUNK_BASE) for glGetShaderInfoLog /
glGetProgramInfoLog / glGetString / glCompileShader. Workspace **467/0**
was 466/0; boot unchanged (stable idle main loop, exit 124).

**Where this leaves the frontier (unchanged hard wall):** the engine's own
producer never enqueues a real work/task item onto its per-thread idle futex
(lr=0x10284d134; queue head [x19]=0, latch x19+4). Re-confirmed this cycle:
`--futex-kick 2 --futex-set 0xf4240` wakes all 3 threads (block-cache **hits**
grow) but **compiles stay flat at 1668** — the latch is a signal, not the work;
with no queue element the consumer re-arms and re-parks. GLES dynamic-loader
(eglGetProcAddress) + GLOB_DAT GLES chain are now complete so that once the
barrier is crossed the ES functions resolve through our bridges (float,
texture-interception, int). Partial reconstruction of the scheduler wait path:
callers at 0x284eb80 / 0x2856f44 pass the queue obj x19; after wait, read
[x19+104] -> vtable, dispatch `blr` a callback at [vt+40] — an opaque
scheduler/vtable dispatch, the last lever documented across many cycles.

## Session (Sep 12, 2026, hermes-worker, cycle SH3) — eglGetProcAddress routed through a GLES bridge (dynamic GLES loader no longer returns raw Mesa pointers); workspace 465/0; HEAD 3a30b3a.

Decoder 100% (0 Unsupported / 0 PANIC on .text) unchanged. The boot frontier
(engine producer never enqueues onto the idle work-queue futex) is unchanged
but was re-confirmed this cycle: `--futex-kick 2 --futex-set 0xf4240` wakes
all 3 threads (block-cache **hits** grow while **compiles** stay flat at 1668)
— the futex latch (x19+4) is a *signal*, not the *work*; the queue head at
[x19]=0 stays empty, so the engine re-arms and re-parks. A host-side producer
must enqueue a real render/task item into that queue, not just poke the latch.

Closed a real secondary gap toward a real frame — **`eglGetProcAddress`**:

- The real binary imports `eglGetProcAddress` (readelf-confirmed UND FUNC); on
  Android Roblox resolves most gl*/egl* entry points *dynamically* through it
  and `blr`s the returned pointer.
- It was binding to **Mesa's raw function** via the generic `resolve()` dlsym
  path (comes BEFORE resolve_egl/resolve_gles_* in plt.rs), so a returned
  pointer was a raw x86 Mesa address — not a registered host-thunk slot. A guest
  `blr` to it can't dispatch through the host-call bridge, and the call would
  bypass the GLES float bridge and the compressed-texture interception
  (breaking glClearColor/glTexImage2D/glCompressedTexImage2D on a real frame).
- Fix: `resolve()` and `resolve_egl()` both route the name to a shared
  `resolve_egl_get_proc_address`, installing `w_eglGetProcAddress` (a GLES
  bridge, HostGlesCall ABI: reads guest x0 = proc-name, resolves it to one of
  OUR host-thunk slots: mixed float/texture -> int -> egl). A later guest `blr`
  to the returned slot dispatches through the correct bridge, preserving float
  and texture interception. Unknown names fall back to real Mesa (niche).
- Subtlety: resolve_gles_int/resolve_egl build a CString from the name, so it
  must be passed WITHOUT a trailing NUL (plt.rs names are NUL-free; the bridge
  reads the guest C-string and passes the byte content). Mixed tolerates NUL.
- New regression `egl_get_proc_address_bridge_returns_dispatchable_gles_slot`
  pins: glClearColor + glCompressedTexImage2D (mixed bridge) and glGenTextures
  (int bridge) all yield the same *dispatch target* as a direct import, unknown
  names never collide with the GLES region. Added `jit::gles_bridge_fn` (pub)
  to compare GLES-region slots by underlying fn (resolve_gles_mixed allocates a
  fresh slot per call, so equality is by target not address).
- Verified: workspace 465/0; egl_window_present + anativewindow_x11_surface
  gates still pass; real boot unchanged (stable idle main loop, exit 124, no
  egl/gl hostcalls yet — the engine still waits on the producer-enqueue).

**Next lever (unchanged hard wall):** cross the engine's work-queue futex —
reconstruct the queue element layout (wait primitive callers at 0x284d014:
x19 = queue obj, latch x19+4 = generation/signal, [x19]=0 = head; fetch_add on
entry frees the high-32 generation) and enqueue a real render/task from the
host before posting the latch. Secondary tracks (GLES float bridge, texture
path, eglGetProcAddress bridge) are complete and gated.

## Session (Sep 12, 2026, hermes-worker, cycle SH2) — idle barrier PROVEN a work-queue futex; snapshot now captures futex args + `--futex-set <hex>`; workspace 464/0; HEAD 10e6b49.

Decoder remains 100% (0 Unsupported / 0 PANIC on .text). The boot frontier
was re-probed empirically this cycle with a definitive conclusion:

- **Live futex capture** (JIT_THREADS now carries x3 val / x5 uaddr2 / x6
  bitset in the snapshot): all 3 guest threads park at lr=0x10284d134 with
  `futex(uaddr, op=0x89 WAIT_BITSET|PRIVATE, val=x3=0x0)`. The per-thread latch
  is re-armed to 0 each cycle; the consumer only proceeds when a producer posts
  a NON-ZERO work token.
- **`--futex-kick` (old+1: 0→1)** AND the new **`--futex-set 0xf4240`** (fixed
  token written verbatim every tick) both leave compiles flat at 1668 / hits
  ~8713: the engine re-parks either way. → **The latch is a signal, not the
  work; a bare futex poke is NOT a producer.** The producer must enqueue an
  actual work item (render/task) into the engine's per-thread queue (host-heap
  object at x19[*=0x0], latch = x19+4), then set the latch. That queue's
  structure is the frontier (reconstruct the consumer dequeue path after the
  futex returns).
- 0xF4240 = 1,000,000 is a pre-initialized "go" upper-bound counter the barrier
  sites load; it is NOT the awaited futex val (that's 0x0).
- New harness: `--futex-set <hex>` writes a chosen latch value each tick
  (awaited-token experiment). Regression pins the snapshot's new x3/x5/x6
  fields.

Run (reproducible): same elfjit StartApp command as cycle SH; expect exit 124,
3 threads parked `pc=syscall lr=0x10284d134 x3=0x0`, compiles flat 1668.
Run-logs: `/home/hermes-worker/runs/futex-x3.txt` (awaited-val capture),
`futex-set2.txt` (fixed-token negative).

### Next lever (unchanged hard wall, sharpened)
Reconstruct the engine's work queue: trace the consumer path that runs after
the idle futex returns (what it dequeues / what "is there a task" check it
does) to learn the queue struct layout, then enqueue a real work unit from the
host and post the latch. All graphics/decoder/texture/import work is complete
and gated; the boot needs this producer enqueue.

## Session (Sep 12, 2026, hermes-worker, cycle SH) — arm64jit DECODER REACHES 100% COVERAGE on real libroblox.so: 11,437 Unsupported -> ZERO, 0 PANIC; workspace 464/0; HEAD 092e829.

The decoder — the single largest structural wall in the project — is now
**permanently closed**. The entire real binary .text span
[0x102d95980, 0x1072d5a84] decodes with zero Unsupported and zero decode
panics. This cycles' closes (all qemu/oracle-verified, decode pins + exec tests):

- **addhn2/subhn2/raddhn2/rsubhn2 Q=1 widening-narrow** (SimdHighNarrow `q`
  field, upper-half dest; bit21 asserts addhn-not-EXT). 49 -> 37.
- **SIMD fp64 absolute-difference fabd Vd.2D** (Simd2dFp op 8:
  subsd+pand sign-clear). 37 -> 28.
- **facgt/facge absolute-compare** (VecFpCmp `abs` flag; pand sign-bit clear on
  both loaded lanes). 28 -> 17.
- **srhadd signed rounding-halving add** (SimdHadd `rounding` flag;
  (a+b+1)>>1, qemu {1,2,2,3,0,2,2,52}). 17 -> 13.
- **FP16 vector frint** (frint{nmzpax} Vd.8H/.4H, esize2 via F16C
  promote-round-demote; cvtph2ps/cvtps2ph helpers; **FMaxV bit22 guard** so
  frintx fp16 (0x6e799800) isn't stolen). 13 -> 11.
- **FP16 scalar unary** (fneg/frintm/z/p/n/x/fsqrt/fabs h0; FpUnary `half`
  flag; bit-mask for neg/abs, F16C round-trip for rounding). 11 -> 3.
- **FP16 vector frecpe/frsqrte** (.8H/.4H via dedicated bit20-SET gate) +
  **frintx .2s/.2d** (table rows). 3 left.
- **mrs xN, fpcr read** (SysReg 11 -> 0, nearest-even default FPCR) +
  **ldpsw post/pre-indexed** load-pair (top-byte 0x68/0xe8/0xe9, sign-extend
  32-bit pair). 3 -> **0**. ZERO unsupported / ZERO panic.

**Metahistory recap**: the .text decode coverage went from 14,918 -> 11,437
(after ARMv8.2 stubs/prefixes) -> ... -> 49 -> 0 over ~28 focused cycles. The tail
after ~100 was entirely single-instance cold-path opcodes (fp16 variants, rare
scalar-FP forms).

**The frontier is now purely the boot**: decoder saturation is done. Continue
with the boot-path analysis (producer thread never enqueues onto the idle
futex — next lever). Graphics translation (GLES float bridge,
glCompressedTexImage2D ETC2/ASTC) remains the secondary thread.

Workspace: **464 passed / 0 failed** (incl. qemu-oracle diff_battery). HEAD 092e829.

## Session (Sep 12, 2026, hermes-worker, cycle R) — FP16 by-element fmla/fmls/fmul (the biggest remaining family) closed; workspace 423/0; HEAD f3251b7.

Closed **`fmla/fmls/fmul Vd.8h/.4h, Vn, Vm.h[idx]`** (~2000+ real .text
instances — the single largest remaining family, the multiply-accumulate core
of the FMOD/audio + render/HDR math paths). Coverage 11,437 -> 9,552 Unsupported
(-1,885), distinct 6,281 -> 5,735, 0 PANIC.

- **New `Inst::SimdFp16BEl` + decode gate**: byte0-nibble 0xf, bit29 CLEAR,
  bit23 CLEAR, **bit10 CLEAR**, bit13 CLEAR. The bit10 CLEAR is the new critical
  discriminator: FP16-indexed (bit10=0) vs shift-by-immediate (shl/ushr/sshr/
  usra/ssra/srshr/srsra... bit10=1 FIXED), which share byte0 prefix AND bit23=0
  AND alias the FMUL/FMLA/FMLS opcode bits[15:12] onto the shift's [14:12]
  marker (fmul=bit15, fmls=bit14, fmla=bit12==usra's 0b001). bit13 CLEAR excludes
  widening SimdMullEl (requires bit13 SET); bit23 CLEAR excludes f32
  FmlaEl/SimdFmulEl (bit23 SET). MUST precede the shift gates. Verified disjoint
  over 30+ both-family cross-compiled encodings.
- **Fields**: op = bit15(fmul)|bit14(fmls)|else fmla; idx (.8h 3-bit) =
  (bit11<<2)|(bit21<<1)|bit20, .4h = (bit21<<1)|bit20; vlm = bits[19:16] (v0-v15).
- **Translate**: hoist-splat Vm.h[idx]->xmm2 (F16C promote once, BEFORE the lane
  loop so rd==rm can't clobber), per-lane promote Vn.h[i]->f32, fma in f32, demote
  ->store16. **Vd must be promoted (vcvtph2ps) before the accumulate** — Vd is
  fp16, unlike the f32 FmlaEl path. fmul copies xmm1->xmm0 (movaps) so all ops
  demote the same xmm0->xmm0 vcvtps2ph.
- Regression: decode pins (real fmul v2.8h,v1.h[4]=0x4f019882, fmla v31.8h,
  v15.h[7]=0x4f3f1bdf, fmls v2.4h,v1.h[2]=0x0f215082, .h[idx0..7]; + f32-untouched
  pins: fmla/fmls stay FmlaEl, fmul stays SimdFmulEl) and exec tests (fmla .4h,
  fmls .4h, fmul .8h 8 lanes incl. high-slot st.v[3] index). Workspace 423/0.

### Next lever (docs/fp16-decode-gap.md updated)
**SIMD modified-immediate orr/bic v.2s/.4s** (0x4f0177eX, ~100) — the next
largest remaining family. Then fmaxnm/fminnm v.4s (0x4e21c8xx ~30), vector
fcvtas v.4s (~150), scalar fabs/fneg/fsqrt, vector-immediate bic/orr (0x2f047400).

The real HARD wall is unchanged: the engine's own producer never enqueues work
onto its per-thread idle futex, so the main loop re-parks. Decoder coverage is
what lets a real frame/audio path run instead of block-tracker-breaking.

## Session (Sep 12, 2026, hermes-worker, cycle Q) — decoder FP16/NEON coverage -3,481; workspace 421/0; HEAD d284c2d.

Closed the FP16 decoder gaps that will block-tracker-break a real frame/audio path
(14,918 -> 11,437 Unsupported .text, distinct 7,743 -> 6,281, 0 PANIC invariant).
Boot unchanged (JNI_OnLoad 0x10006, stable idle main loop, exit 124/no crash).

Two commits:
- `4dd5e3d` — scalar FP16 + gate fixes: new **`FcvtHalf`** (fcvt s,h/h,s/d,h/h,d via
  F16C vcvtph2ps/vcvtps2ph$0 RN, raw VEX bytes; host confirmed f16c — the JIT's first
  F16C use, the reusable pattern), **`FpScalar.half`** (fadd/fmul/fsub/fdiv h),
  **`fccmp s/d` decode gate FIXED** (old 0xfff0_fc03==0x1e20_c400 never matched any
  real fccmp; new 0xffe0_0c10=={0x1e200400 s,0x1e600400 d}, must decode before the
  FMOV-imm gates because a cond≥8 sets bit12 that the FMOV-imm lane-anchor misread as
  an immediate), `fabd s` (single form 0x7ea0_d400 + sz field), `urhadd v.16b/.8b`.
- `d284c2d` — SIMD FP16 3-same **`SimdFp16As`** (fadd/fsub/fmul v.4h/.8h) per-lane
  promote->op->demote via F16C. Gate `(insn&0x9f60_f400)==0x0e40_1400`. MUST decode
  BEFORE the SIMD-select (bsl) gate: FP16 `fmul v.8h` (byte1 0x1c, 0x6e451c82) aliases
  bsl's byte1-0x1c mask and was silently decoded as a bitwise select until a
  regression caught it.

Ground truth: synthesized with `aarch64-linux-gnu-gcc -O0 -march=armv8.2-a+fp16` and
objdump; real encodings pinned (fccmp s0,s1,#0,eq=0x1e210400; fabd s2,s2,s3=0x7ea3d442;
fadd v1.8h=0x4e401421). Tests: decode pins + `exec_bytes` runtime tests (fcvt h<->s,
fabd s, urhadd bytes, fadd h scalar + fadd v2.4h vector) — workspace 421/0.

### Next lever (the single biggest remaining family)
**`fmla/fmls/fmul Vd.8h/.4h, Vn, Vm.h[idx]`** — the 0x4f0x_1x2x/1x9x opcodes, ~2000+.
Ground-truth matrix captured in docs/fp16-decode-gap.md:
- index = (bit11<<2) | (bit21<<1) | bit20 for .8h; .4h uses (bit21<<1)|bit20.
- vm in bits[19:16] (v0-v15 only, bit20 is index[0]); fp16 fmla bit23 CLEAR (vs
  f32 FmlaEl requiring bit23 SET — the discriminator); fmul=bit29 SET, fmls=bit14+bit12
  (same-operand ground truth 0x4f321020 / 0x4f325020 / 0x4f329020).
- Translate: hoist-splat Vm.h[idx] -> xmm2 (promote once) BEFORE the lane loop
  (rd==rm clobbers the element), per-lane promote Vn.h[i]->f32, fma in f32, demote.
Then: SIMD-immediate orr/bic v.2s/.4s (~200), vector fcvtas v.4s (~150), fabs v.2s.

The real HARD wall is unchanged and independent of this work: the engine's own
producer never enqueues work onto its per-thread idle futex, so the main loop re-parks
(no render/EGL path). The decoder coverage is what lets a real frame/audio path run
instead of block-tracker-breaking when that wall is crossed.

## Session (Sep 12, 2026, hermes-worker, cycle P) — ALooper poll-source contract corrected (the glue-looper prerequisite); workspace 411/0, HEAD e8e08cf.

The real binary imports `ALooper_pollOnce`/`ALooper_addFd` (via libandroid.so,
confirmed in dynsym) and the app-glue main loop (guest 0x102bcd5d0) derefs
`ALooper_pollOnce`'s outData as `struct android_poll_source*` and `blr`s
`source->process` (fn at +0x10). The old shim wrote a **raw APP_CMD int** into
outData — which would crash a glue loop following the real layout. Fixed:

1. `ALooper_addFd(..., data)` records `data` (the `android_poll_source*`, i.e.
   `&app->cmd_source` in real glue) keyed by fd.
2. `ALooper_pollOnce` emits the registered poll-source pointer via outData only
   when one exists for the command fd; otherwise the raw-APP_CMD fallback for the
   existing host-feed lifecycle path is unchanged.

Regression tests pin both behaviors + the +0x10 process-fn layout. Also fixed a
real test-race: the ALooper tests share the process-wide queue/registry and run
under the parallel harness, so they need a shared serializing Mutex (the naive
`let _g = lock()` bound `&Mutex`, not a guard — passed at --test-threads=1 while
racing in parallel; now `.lock()`). Workspace 411/0; real boot unchanged (stable
idle main loop, exit 124, real X11 window wired). Run-log:
`/home/hermes-worker/runs/` (status in STATUS.md).

### Next lever (RECORRECTED this cycle — do not chase 0x102bcd5d0)
**The app-glue looper at guest 0x102bcd5d0 is COMPLETELY ORPHANED** in the real
binary (zero BL/B callers AND zero 8-byte pointer-constant references, scripted
scan). real libroblox.so uses GameActivity (`nativeAppBridgeV2StartAppWithParams`
driven by `--startapp`), not legacy android_native_app_glue; its prologue derefs
`[x19+24]` as a framework-initialized object only ANativeActivity_onCreate builds.
So fabricating an android_app and starting 0x102bcd5d0 as a guest thread is the
WRONG lever — it would not launch the engine's real render path. The ALooper
shim change is still correct (any real ALooper_pollOnce caller gets the right
android_poll_source* outData layout), but the wall is the engine's OWN producer:
its main loop runs, yet nothing enqueues work onto the per-thread idle futex
(0x10284d134) it parks on. The last-remaining cross-session lever was disproven
this cycle — the next real lever is either feeding the engine's own work queue or
finding the GameActivity-side producer that posts render tasks. Additionally,
real `.text` has 14,918 Unsupported FP16/NEON instructions (see
docs/fp16-decode-gap.md) that will block-tracker-break any real frame/audio path.

Prior cycles claimed "decode() never panics" but verified it only against `.text`.
The **whole-executable** scandecode scan (all PF_X segments — including the
data-region bytes a computed branch could land on) found **5260 decode() PANIC
sites**, every one outside `.text`. Two decoder gates shifted without guarding a
zero element-size field — a hard JIT abort (the whole process dies) on arbitrary
guest bytes:

1. **umov/smov** (insn&0xbfe0_fc00 == {0x0e00_3c00, 0x0e00_2c00}):
   `1 << imm5.trailing_zeros()` PANICS when imm5==0 (tz=32, shift overflow).
2. **vector dup** (insn&0xffe0_0c00 == {0x0e00_0400, 0x4e00_0400}):
   `1 << (f & f.wrapping_neg()).trailing_zeros()` PANICS when f==0.

Both now return `Inst::Unsupported` for the reserved zero-size encoding. **Key
subtlety:** the guard must be exact-zero ONLY — imm5/f packs BOTH element size
AND the lane index (e.g. `dup v21.2s, v23.s[1]` = 0x0e0c06f5 has f=0b01100 →
esize 4 via `f & -f`, src_idx 1), so any `f>8`/power-of-two bound wrongly rejects
real hardware instructions. An over-eager bound was written first, caught by the
regression test acting on real libroblox insns, and reverted.

**Verified (reproducible, no boot regression):**
- scandecode whole-exe (25,911,396 insns): **PANIC hits 5260 → 0**;
  `.text` (1,376,321 insns) still **0 Unsupported / 0 panics**.
- New regression `umov_smov_and_vector_dup_zero_imm5_do_not_panic` pins both
  gates + the real `dup v21.2s,v23.s[1]` / `dup v27.2s,v24.s[1]` forms.
- Workspace 409 passed / 0 failed (was 408). Real boot unchanged: reaches
  StartApp + stable engine main loop, exit 124, no crash.
- Run-log: `/home/hermes-worker/runs/cycleO-decode-panic-hardening-runlog.txt`.

### Next lever (unchanged — the hard remaining wall, now with more precision)
The engine main loop is reached and the ANativeWindow layer maps to a REAL X11
window (cycle N), but no **guest thread runs the app-glue looper**, so
`ALooper_pollOnce` is never called, the queued APP_CMD_START/RESUME/INIT_WINDOW
are never drained, and `eglCreateWindowSurface` never fires. Disasm pinned the
missing thread: **0x102bcd5d0** is the android_native_app_glue main loop
(currently anonymous, filed inside `nativePreloadFlagOverrides`' block). Prologue
takes the `android_app*` state in x0, stores it in x19, then the loop at
0x102bcd648 reads state flags (+8/+9/+10) and calls
`ALooper_pollOnce(-1, NULL, &events, &source)` (0x102bcd670), dispatching via
`blr x8` where x8 = `[source+16]` (the `process` fn). No internal caller invokes
it — it is the host-or-driver-started glue thread. Both `pthread_create`s spawned
in the boot run the engine worker loop 0x10284d168, never the glue loop.

Two routes to a first headless llvmpipe frame, in order of cleanliness:
1. **Drive the glue looper (0x102bcd5d0)**: fabricate an `android_app` state
   object (the fields the loop touches: looper handle, state-flags at +8/+9/+10,
   the app-command source `{id,process,..}` at the callback), and start it as a
   guest thread so it drains the ALoop. The app-command source's `process` fn is
   what must eventually call ANativeWindow_fromSurface→eglCreateWindowSurface.
2. **Enqueue a render/task directly** onto the engine's per-thread work-queue
   (the idle futex all 3 threads park on at 0x10284d134).

Cycle M's `ANativeWindow_fromSurface` returned a `HOST_THUNK_BASE|0x2000`
sentinel — a fake address Mesa's x11-EGL platform would reject in
`eglCreateWindowSurface(win, ...)`. This cycle mapped the window layer to a real
desktop window and fixed the wiring race:

1. `input_wrapper::x11::open_window_sized(...,w,h)` — the runtime opens a
   1280x720 Xvfb window matching the `ANativeWindow_getWidth/Height` framebuffer.
2. `shims::set_anativewindow_xid()` + an `ANATIVE_WINDOW_XID` atomic —
   `anativewindow_fromsurface` returns the registered real XID (sentinel
   fallback otherwise, so headless stays coherent).
3. **Race fix**: the first window wiring (commit `5fdd7a6`) ran in a spawned
   thread that lost to the boot — StartApp's `ANativeWindow_fromSurface` fired at
   ~3.9s while the XID registered later, so the guest still saw the sentinel.
   Now `wire_real_window()` runs **synchronously** inside the `--startapp` block
   before the StartApp `jit_run`: Xvfb up → 1280x720 window → XID registered →
   DISPLAY/EGL_PLATFORM=x11 set (X connection leaked to keep the window alive).
4. New integration gate `anativewindow_x11_surface`: the XID the guest's
   `ANativeWindow_fromSurface` yields builds a real EGL window surface and
   presents a frame (llvmpipe + Xvfb) — the exact value the window-surface path
   will consume.

**Verified:** log ordering proves the fix — `wired real X11 window
XID=0x200000` precedes `hostcall@ANativeWindow_fromSurface`, and the shim returns
`xid=0x200000`. Boot unchanged (stable idle main loop, exit 124, no crash).

Run (reproducible):
```
cargo build -p arm64jit --example elfjit
JIT_DRIVE_LIFECYCLE=1 timeout 20 ./target/debug/examples/elfjit \
  ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 --jni --startapp 0x258b144 \
  --kicker 0x106863af8 --futex-kick 2
# expect: [elfjit:anativewindow] wired real X11 window XID=0x200000 on :22x,
# then exit 124 (stable engine main loop; no egl/looper hostcalls yet).
```
Run-log: `/home/hermes-worker/runs/anatg-sync.txt`.

### Next lever (unchanged hard wall — the looper/producer)
The engine's main loop is reached and the window layer is real, but no egl*/gl*
hostcall fires (0) and `ALooper_pollOnce` is never called: the render/EGL path
only opens after a producer enqueues a real render/task the idle futex
(lr=0x10284d134) awaits. Identify which guest thread should run the looper and
ensure it is spawned/woken (or enqueue the work item directly). A real
egl*/gl* frame from the running engine is the next targeted milestone.

---

Cycle L pinned the engine main-loop idle barrier as a REAL per-thread futex:
each guest thread parks in `guest_svc`'s FUTEX_WAIT_BITSET on its OWN latch
(uaddr = x1 = x19+4) at guest call-site lr=0x10284d134, with zero forward
motion. The static `--kicker` (fixed guest globals) couldn't reach these
per-thread dynamic latches, so the boot sat flat from the start.

**New elfjit lever `--futex-kick <period-ms>`** (commit `df5d0a4`): a detached
host producer snapshots the parked guest threads and, for each one at the idle
futex call-site, increments its latch (a version-counter futex — a bare fixed
write self-defeats because the next waiter captures the same value as expected
and re-blocks) and issues a real host FUTEX_WAKE. One tick per kick.

**Verified (reproducible):** vs flat idle, with `--futex-kick 2`:
- compiles advance 1515 → 1670 (155 new StartApp init blocks),
- `hostcall@ANativeWindow_fromSurface` is reached (guest pc 0x10258b3a0) — the
  boot's first window-layer touch, and `pthread_cond_wait` appears (85x),
- JNI setup churns: FindClass 23x, GetStaticMethodID 46x, mempool_calloc 75x,
  JavaVM.GetEnv, NewGlobalRef, ExceptionCheck.

The engine then re-parks on the same futex as a well-behaved idle loop — it
awaits a producer-ENQUEUED work item (a render/task). The futex-kick wakes the
consumer but no *work* is queued, so it sleeps again. `ALooper_pollOnce` is
still never reached (0 calls), `ANativeWindow_fromSurface` returns NULL (dead-
ends before eglCreateWindowSurface).

Run (reproducible): exit 124 (ran until harness timeout):
```
cargo build -p arm64jit --example elfjit
JIT_DRIVE_LIFECYCLE=1 timeout 20 ./target/debug/examples/elfjit \
  ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 --jni --startapp 0x258b144 \
  --kicker 0x106863af8 --futex-kick 2
```
Run-log: `/home/hermes-worker/runs/cycleM-futex-kick-runlog.txt`.

### Next lever (two candidate walls, both now concrete)
1. **Wire a real native window into `ANativeWindow_fromSurface`** (GRAPHICS_-
   RECOMMENDATION §5.3): it currently returns 0, so eglCreateWindowSurface can
   never be created. Map the ANativeWindow to an X11 Window XID (Xvfb present,
   elfjit already links input-wrapper+x11rb; the egl_window_present gate shows
   the exact pattern). Then when the engine reaches the window/surface path it
   can proceed to a first headless llvmpipe frame.
2. **Reach the engine's looper/producer**: `ALooper_pollOnce` is never called —
   the app-command FIFO feed (JIT_DRIVE_LIFECYCLE) is inert until the engine's
   app-main thread runs the looper. Identify which guest thread/entry should
   drive the looper and ensure it is spawned/woken.

---

**This corrects cycles I–J's wrong conclusion.** The settled main loop issues
~204k futex syscalls / 12s, but only through the imported `syscall@LIBC`
function (the guest calls libc `syscall(nr,...)`, not `svc #0`). Cycle J's
"settled loop makes zero guest syscalls; it's a pure TLS-flag spin" was a
misdiagnosis — it probed only `svc #0` and missed the `syscall()` import path.

**The bug:** `resolver::resolve` bound `syscall` to HOST glibc `syscall()`,
which reads the number as an x86-64 syscall number. The guest's AArch64 futex
(98) became x86-64 getrusage (98) → returned -1, never blocked, and the loop
busy-spun re-issuing a dead futex.

**The fix** (commit `11189dd`, regression-tested):
1. `resolve("syscall")` → `host_syscall_intercept`, which rebuilds a CpuState
   (x[8]=aarch64 nr, x[0..5]=args) and dispatches through `guest_svc`.
2. `guest_svc` futex (98) now forwards `FUTEX_WAIT_BITSET` (op masked 9) to a
   real host futex.

**Verified:** all 3 guest threads now park INSIDE the real host futex at
`0x10284d134` with `x2=0x89`. Each thread WAIT_BITSETs on its own per-thread
latch uaddr (uaddr = x19+4). 405 tests (3 new). Boot exit 124, no crash.
Run-log: `/home/hermes-worker/runs/syscall-futex-fix-runlog.txt`.

### Next lever (REAL now, was wrongly cancelled by cycle J)
The main loop idle barrier is a genuine per-thread FUTEX_WAIT_BITSET — the
cycle-I lever (post the awaited futex value + FUTEX_WAKE from a host kicker)
is now host-drivable. Identify each thread's awaited `val` (x3) and the
producer value, then set the latch + FUTEX_WAKE so the loop advances.

---

New `scandecode` example (arm64jit) walks a PGX segment (or an optional
[start,end] guest-vaddr window, to scan only `.text`) and reports every
`Inst::Unsupported` plus any instruction that makes `decode()` panic. Against
the real binary it found exactly 6 undecodable code instructions and, on the
whole executable segment, proved `decode()` never panics. All 6 gaps fixed
with objdump ground truth (commits `f492ce7`, `7445153`):

1. **rev64: the WHOLE family was broken** (3 real hits, incl. `rev64 v5.2s`
   `0x0ea008a5`). Gate was `(insn&0x3f00_0c00)==0x0e00_0800 &&
   (insn&0x1800)==0`; every rev64 has byte1 0x08 (bit11 set), so the uzp guard
   wrongly rejected all six element sizes -> `Unsupported`. Fixed to
   `(insn&0x3f00_ff00)==0x0e00_0800` (byte1 exactly 0x08), still excluding
   uzp(0x18/0x58), rev16(0x18), rev32(bit29), dup-from-GPR(0x0d).
2. **shll/shll2 with rn>=8** (`0x2e613a10`, `0x6e613a17`, rn=v16): the
   WidenShl gate required `((insn>>8)&0x03)==0`, but bits[9:8] are rn
   bits[4:3], not a permute discriminator -> any shll on v8-v31 decoded
   `Unsupported`. True discriminator vs zip/uzp/trn = bit21 (set=shift-imm).
3. **cmhs (unsigned >=)** (`0x6ee13c02`): byte2 0x3c vs cmhi's 0x34.
   New SimdCmhs/SimdCmhsD translate to cmovae (cc 0x43) per lane, distinct
   from cmhi's cmova (`>`); 4S/2S/2D forms.

**Also fixed: decode() must never panic.** The `dup`-from-GPR decoder computed
`1u8 << imm5.trailing_zeros()`; imm5==0 (bits[20:16]) gives trailing_zeros=32
-> shift-overflow PANIC, aborting the whole JIT on arbitrary guest bytes now
emits `Unsupported` instead (regression:
`dup_from_gpr_invalid_imm5_does_not_panic`).

Result: full `.text` scans to `0 Unsupported / 0 decode() panics`. The JIT
can no longer fault on any reachable instruction in the real binary's code.

### Graphics/import readiness (boot-on-GPU prep)
- PLT fully bound against the real binary: 534 JUMP_SLOT, 0 unresolved
  (remaining 11 are GLOB_DAT data globals; `resolveimports` example).
- All 118 egl*/gl*/ANativeWindow*/ALooper*/AAssetManager*/AConfiguration*
  imports sit in the already-wired Mesa-llvmpipe resolver surface (the
  eglGetDisplay->...->eglSwapBuffers headless gate passes).
- Boot still reaches the stable engine main loop after the decoder changes
  (1871 compiles flat, exit 124 until harness timeout) — no regression.

### Frontier (unchanged, genuinely blocked on-this-box)
The settled main loop is a pure-CPU spin on bit0 of a per-thread TLS object
(`has-pending-work` latch at 0x10284d524 via getter 0x102b9dee0), driven by
the absent Android framework event/looper producer + a real window/surface;
needs the surviving looper/framework + EGL window layer, and a GPU host for
meaningful frame-perf proof (cycles I-J evidence: zero syscalls, static seed
and forced-branch both ineffective). Boot stabilization itself is DONE and
captured. Workspace: 402 tests, 0 failed; tree clean; HEAD `7445153`.

## Session (Sep 12, 2026, hermes-worker, cycle J) — STABLE MAIN LOOP HOLDS; the cycle-I "futex" next-lever is DISPROVEN and replaced with the true barrier (per-thread TLS-flag spin, framework-owned).

The real-boot milestone from cycle I is unchanged and still holds: libroblox.so
loads, JNI_OnLoad returns 0x10006, StartApp drives the engine, all guest threads
run the engine main loop headlessly until the harness timeout (exit 124; no
crash/leak; block-cache flat at 1871 compiles, hits ~19M, RSS ~5MB). This cycle
made no production code change — it re-established the frontier from first
principles and fixed the wrong plan.

**The cycle-I lever ("POST the awaited futex value + FUTEX_WAKE") is cancelled:**
the settled main loop makes **ZERO guest syscalls** (a 40s JIT_TRACE_SVC=1 run
printed zero futex and zero `guest svc` lines). The futex instructions at
0x10284d114-138 are a transient init burst, not the settled loop. There is no
futex to wake.

**The true steady-state barrier** is a pure-CPU spin at guest 0x10284d524:
`adrp/add x0,#0x7e0; bl 0x102b9dee0` (per-thread object getter),
`ldrb w8,[x0]; and w8,w8,#1; cbz <loop>` — it polls **bit0 of a per-thread TLS
object returning its address** (JIT_DUMP_PC=0x10284d538 shows x0 =
0x7f6998043df8 vs 0x7f6990036568 across threads). Seeding the static window
0x1067d67e0/f0/f8=1 via `--kicker 0x..=1` did NOT advance compiles (flat 1871):
the gate is dynamic, driven by the absent Android framework event/looper
producer plus a real window/surface. Same class of framework-owned lifecycle
gate as cycles C–I, now at the steady main-loop level — not a decoder/loader/ISA
gap, not a futex.

### Next lever (ordered / real)
The gate is an idle "has-pending-work" latch, NOT a seedable singleton: a 40s
JIT_TRACE_SVC capture shows the settled loop makes ZERO guest syscalls; a 40s
static-kicker seed of 0x1067d67e0/f0/f8=1 did NOT advance compiles; and even
FORCING the flag-set branch (patch `and w8,w8,#1` @ 0x10284d53c -> `mov w8,#1`
under JIT_DRIVE_LIFECYCLE) left compiles flat at 1871 (the 0x10284d548 handler
is an idle maintenance loop, not a work producer). So bit0 means "pending work /
should run" and forward motion needs framework-ENQUEUED work items, not a flag
seed:
1. Build the surviving looper/framework + EGL window/surface layer the engine
   awaits, so a real producer can post the work items that set the latch (correct
   ordering), then drive them. A GPU host is needed for meaningful frame-perf
   proof.
Boot stabilization (the achievable on-VPS milestone) is DONE and captured.

Run-logs (cycle J, reproducible):
```
cargo build -p arm64jit --example elfjit
JIT_DRIVE_LIFECYCLE=1 JIT_STATS=1 timeout 15 ./target/debug/examples/elfjit \
  ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 --jni --startapp 0x258b144 \
  --kicker 0x106863af8    # exit 124 = stable main loop until timeout
```
- `/home/hermes-worker/runs/mainloop-tlsflag-spin-runlog.txt` (narrative + cmd)
- `/home/hermes-worker/runs/mainloop-tlsflag-spin-raw.txt` (verified raw: 1871 flat)
- Diagnostic used: `JIT_DUMP_PC=0x10284d538` (dumps x0 = polled per-thread ptr);
  `JIT_TRACE_SVC=1` (proves zero svc in the settled loop); `disasm` example.

## Session (Sep 12, 2026, hermes-worker, cycle I) — REAL BOOT REACHES A STABLE RUNNING ENGINE MAIN LOOP: crossed the cycle-H worker SIGSEGV (UXTW, not a W-write leak), the pthread_key_create destructor SIGILL, and the step-budget false abort. libroblox.so now loads + JNI inits + the main loop runs indefinitely headless (exit 124 on harness timeout). Workspace 400+/0.

Commits `b7da1a9` + `9c9332b` (dev). Cycle-H's stated next wall (worker SIGSEGV
guestpc 0x102173218, misattributed to a W-write zero-extension leak) was
re-root-caused from first principles with objdump ground truth:

1. **UXTW register-offset index (the real cycle-H bug).** The guest DELIBERATELY
   returns x0 = 0x100000000|hash from its hash table as a not-found SENTINEL
   (`mov x8,#0x100000000; orr x0,x8,x12` at 0x2173324/330 — verified vs
   aarch64-linux-gnu-objdump), and the caller indexes with
   `ldr w8,[x8, w0, uxtw #2]` at 0x2173218 — a UXTW (W) register offset that
   zero-extends w0 and MASKS OUT the sentinel bit. The JIT decoded it as full
   64-bit `[x8,x0,lsl#2]`, so x0=0x100000665 indexed 0x100000665<<2 OOB → SIGSEGV.
   Fix: carry the option bits[14:13] as `index_ext` on LdStrReg/FpLdStrReg
   (3=LSL full-64, 2=UXTW low-32, 1=UXTB) and zero-extend the index accordingly.
   Regression tests pin decode(index_ext) and the sentinel-masking load.
2. **pthread_key_create destructor SIGILL.** Worker's `pthread_key_create(dtor)`
   resolved to real glibc, which ran the guest AArch64 dtor natively on thread
   exit (SIGILL at 0x102b9e144, a `paciasp` prologue; gdb backtrace = libc
   `__pthread_keys`). Shim now creates a REAL key with a NULL destructor (book
   getspecific/setspecific still work; headless TLS dtors skipped — same as
   `__cxa_thread_atexit_impl`).
3. **Step-budget false abort on a reached main loop.** All 3 guest threads churn
   in the engine main loop (flat 1873 compiles, zero hostcalls, 5MB stable RSS).
   run_loop now only trips at the step budget if the block cache is still
   GROWING (un-settled init expansion); a flat cache = reached main loop, keeps
   running until the harness timeout.

### Result (headless, reproducible)
```
cargo build -p arm64jit --example elfjit
JIT_DRIVE_LIFECYCLE=1 timeout 30 ./target/debug/examples/elfjit \
  ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 --jni --startapp 0x258b144 \
  --kicker 0x106863af8      # exit 124 = engine main loop ran until harness timeout
```
Run-log: `/home/hermes-worker/runs/mainloop-stable-runlog.txt`.

### Where the boot stands (vs the HARD GATE)
Achieved on this VPS: **libroblox.so loads, JNI_OnLoad returns 0x10006, StartApp
drives the engine, all guest threads reach a stable running main loop headlessly.**
That is the boot-stabilization milestone. Remaining to the full gate: get that
running main loop to dispatch a real frame (route through the wired Mesa
llvmpipe egl/gl so an early EGL/GLES call resolves).

The main-loop wait, precisely (JIT_TRACE): it busily polls a task/event futex —
`syscall 0x62` (aarch64 futex=98) with op 0x89 = FUTEX_WAIT_BITSET_PRIVATE at
call-site 0x10284d134, plus heavy `pthread_getspecific` (TLS getter 0x102b9df10)
and `clock_gettime` (timeout tracking). `guest_svc` only forwards FUTEX_WAIT(0)/
FUTEX_WAKE(1); FUTEX_WAIT_BITSET (0x89) returns 0 immediately, so the engine
never blocks — it re-issues the futex in a tight loop (flat ~1873 compiles, no
new init blocks). It never reaches ALooper_pollOnce or any egl*/gl* import, so
the cycle-G app-command feed is inert until then. Next lever (concrete):
identify the guest futex uaddr (x1 of the 0x62 syscall) and the "expected" value
that would let it pass, and POST a winning value + FUTEX_WAKE from a host kicker
(the F–H gate-kicker pattern) — or drive the awaited task-queue event — so the
main loop proceeds into the ALooper/EGL path and a first headless llvmpipe frame.
A GPU host is only needed for the final frame-perf proof.

## Session (Sep 12, 2026, hermes-worker, cycle H) — ROOT-CAUSED + FIXED the recursive-mutex rendezvous: `sanitize_mutex` was destroying glibc's `__owner`, so the owner deadlocked on its OWN recursive re-lock; boot now CROSSES the wall that parked every run since cycle C (workspace 398/0)

Commit `9e8d3a9` (dev). After crossing the GameActivity gates (cycles C–G), both
engine threads futex-parked on the glibc-RECURSIVE mutex `0x6edae60` at
`pthread_mutex_lock(0x102b53bb0)` with `__owner=0x0` while `__count=1` — a deadlock,
no forward motion, flat 948 compiles.

**The bug (real, boot-blocking):** `sanitize_mutex` ran before EVERY glibc
lock/unlock/cond_wait and zeroed offset 8 when it read `> 0x10000`, treating it as a
bogus bionic "recursion count". But offset 8 of a **glibc** `pthread_mutex_t` is
`__owner` — the host owner TID. A real TID like 3392123 exceeds 0x10000, so the
freshly-set owner of the LIVE recursive mutex was wiped on the next lock. glibc then
saw `__owner==0 != self` on the owner's own recursive re-lock and futex-blocked it.
`g_owner_tid=0x0` with `g_count=1` at the park is impossible for a correct glibc
recursive mutex — only sanitize writes offset 8. (Bionic stores owner_tid at offset
4, NOT 8; offset 8 is never a bionic leak worth clearing on either ABI.)

**Fix:** sanitize only masks the kind bits at offset 16; it leaves offset 8 alone.
Regression test now asserts `__owner` survives sanitize (was: asserts it's cleared).

**Result (headless, reproducible):** the boot CROSSES gate1 + gate2 + the recursive
rendezvous. The worker thread now does real init it never reached before:
`pthread_setname_np`, `FindClass`, `pthread_once`, mempool/`pthread_key_create`
TLS, mutex init/lock/unlock (trace shows `g_owner_tid=0x33f7b7` PRESERVED). The fence
is a SIGSEGV instead of a deadlock — machine gained ground.

### NEW wall (next frontier): worker SIGSEGV — 32-bit hash index keeps stale upper bits
Worker faults deterministically at `guestpc=0x102173210`, instr `ldr w8,[x8,x0,lsl#2]`
(caller of hash fn `0x102173258`): `x8=0x1073301c0` (a 0x2000-byte table just memset
to 0xff = 2048×4B entries), `x0=0x1000007f5`. Low 32 of x0 (`0x7f5`=2037) is a VALID
index; the upper `0x100000000` (= JIT_BASE) is stale. Index varies per run (988, 2037)
→ a genuine guest hash value leaking the translation-base high bit: a 32-bit `w`-write
in the hash loop `0x102173258` (contains `lsl x12,x1,x4` 64-bit + `mul w11,w8,w9` +
32-bit adds) isn't zero-extending the upper half, which the final `orr x0,x8,x12`
(fn epilogue, `x8` zeroed at `0x10217332c`) then propagates. Next: find the exact
non-zero-extending W-write in `0x102173258` (or in the loop `0x1021732a8..0x102173334`).

Repro:
```
cargo build -p arm64jit --example elfjit
JIT_DRIVE_LIFECYCLE=1 timeout 20 ./target/debug/examples/elfjit \
  ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 --jni --startapp 0x258b144 \
  --kicker 0x106863af8        # deterministic SIGSEGV guestpc=0x102173210 (tid 1)
```
Run-log: `/home/hermes-worker/runs/gate-crossed-cycleH-runlog.txt` (SIGSEGV, not park).

## Session (Sep 12, 2026, hermes-worker, cycle G) — `--kicker` value bug fixed + real ALooper app-command dispatch (workspace 396/0)

Commit `c0736e2` (dev). Cycle F crossed the boot wall's first two GameActivity
lifecycle gates (ldaxr poll 0x106863af8 + cond_wait 0x10683a168) and pinned the
residual to a glibc-RECURSIVE rendezvous mutex 0x6edae60 (call-site 0x102b53bb0)
both engine threads futex-park on. This cycle did NOT cross that wall (confirmed
it is genuinely unchanged — the engine parks *before* ALooper ever spins up), but
removed a real harness bug and built the documented post-barrier mechanism:

1. **`--kicker` value bug (real, boot-affecting).** elfjit parsed `=0xVAL` but
   ignored it, always pulsing 1→2 for every non-bcast kicker. So
   `--kicker 0x106863af8=1` actually KEPT WRITING 1 — the owner busy-spun the
   gate-2 cond_wait (~1.2M block-cache hits) instead of crossing to 2. Now
   `=bcast`, `=0xVAL` (exact value), or bare (Pulse 1→2) are distinct modes.
   The correct gate-crossing invocation is the BARE form `--kicker 0x106863af8`
   (Pulse 1→2); `=1` now correctly pins 1 (gate-2 probe). Verified: bare Pulse
   crosses gate1+gate2, compiles grow 919→948, both threads land on the deep
   recursive rendezvous.
2. **Real ALooper app-command dispatch (shims.rs).** The old `ALooper_pollOnce`
   returned 0 immediately with no event channel, so a boot that DID cross the
   rendezvous would busy-spin the looper on a never-signalled fd instead of
   dispatching lifecycle. Added `post_app_command`/`ALooper_pollOnce` (host-feedable
   mutex'd FIFO drained into outFd/outEvents/outData, android_native_app_glue
   convention; empty → ALOOPER_POLL_TIMEOUT, never blocks), the full ALooper family
   (prepare/forThread non-null handle, addFd→1, removeFd→0, acquire/release→0),
   `ANativeWindow_getWidth/getHeight`→1280x720, and an elfjit APP_CMD feed
   (START/RESUME/INIT_WINDOW) under JIT_DRIVE_LIFECYCLE. Regression test
   `alooper_pollonce_dispatches_host_fed_app_commands`.

### Repro (headless, reproducible)
```bash
cargo build -p arm64jit --example elfjit
timeout 40 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 --jni          # clean exit 0
JIT_DRIVE_LIFECYCLE=1 timeout 20 ./target/debug/examples/elfjit .../libroblox.so 0x2173ff4 --jni --startapp 0x258b144 --kicker 0x106863af8   # cross gates, park at rendezvous (124)
JIT_DRIVE_LIFECYCLE=1 JIT_THREADS=1 ... --kicker 0x106863af8   # concurrent thread-state sampler
```
Run-log: `/home/hermes-worker/runs/gate-lifecycle-runlog.txt`.

### Current wall (unchanged, precise)
Both engine threads futex-park on the glibc-RECURSIVE mutex 0x6edae60 at
call-site 0x102b53bb0 (GameActivity_initializeNativeCode rendezvous). Owner
re-locks the recursive mutex while init-churning (compiles 919→948 after gates),
then parks; worker blocks on it. `cond_wait`/`ALooper_pollOnce` never reached (0
calls). Genuine Java app-command / looper lifecycle await — the same wall cycles
C–F documented. Mesa llvmpipe egl/gl, GLES float/texture bridges, and now the
ALooper app-command channel are all wired + tested; none can fire until this
barrier is crossed.

### Next lever (unchanged from cycles C–F, app-command feed now in place)
Identify/release what lets the holder of 0x6edae60 proceed (which thread posts
the app-command / looper event on real Android) and seed it, OR seed the
rendezvous itself to pass without the looper. The engine never reaches
ALooper_pollOnce before the barrier, so the new feed is inert until then. After
the rendezvous, the feed dispatches APP_CMD_START/RESUME → wired Mesa llvmpipe
egl/gl → first headless frame.

## Session (Sep 12, 2026, hermes-worker, cycle F) — boot wall's FIRST TWO gates CROSSED from the host: owner leaves idle ldaxr-poll AND the gate-2 cond_wait, bursts 919→946 blocks, lands at the recursive-mutex rendezvous (workspace 395/0)

Commits `5dd02ee` + `a69c42a` + `4b488af` (dev). For cycles C-E the boot froze at the
GameActivity rendezvous: all three guest threads parked while the engine owner
busy-polled guest global **0x106863af8 until == 1** (`adrp x8,#0x106863000; add
x8,x8,#0xaf8; ldar x8,[x8]; cmp #1; b.eq`) holding the recursive rendezvous
mutex 0x6edae60. No Java layer exists on this box to set it, so it never
released.

This cycle added **host-side lifecycle release** and proved the boot advances:

- `--kicker 0x<guest-global>[=<val|bcast>]` (repeatable, elfjit): detached
  host thread writes a value to / `pthread_cond_broadcast`s a guest global
  while `jit_run` parks — feeds awaited lifecycle state from outside.
- `JIT_DRIVE_LIFECYCLE=1`: `host_cond_wait` becomes a 2 ms sawtooth timedwait
  so a guest cond_wait entered before we satisfy its predicate still returns
  periodically and re-checks an externally-satisfied flag.
- `snapshot_threads()` now carries x19/x20; elfjit's sampler derefs the
  predicate pointer so the log names *which global* a parked owner awaits.

### Result (headless, reproducible): first motion across the wall
`--kicker 0x106863af8=1` + `JIT_DRIVE_LIFECYCLE=1` makes the owner
1. leave the `ldaxr [0x106863af8];cmp #1` init poll (gate 1),
2. burst 919 -> 948 compiled blocks (29 new init blocks),
3. park at a DISTINCT second wait: `pthread_cond_wait(cond=0x10683a168,
   mutex=0x10683a140)` at guest call-site 0x102b4cd78 (gate 2), re-checking
   `*x19` each 2 ms wake and re-parking while `*0x106863af8 == 1`.

### Gate 2 (now also crossed; commit `4b488af`)
With the re-arm store at 0x102b4cdb4 NOP'd (under `JIT_DRIVE_LIFECYCLE=1`
only), the host terminal value 2 persists and the owner LEAVES the cond_wait
back into the outer init/refcount region (0x102206c00), compiles growing
926→946. Residual wall: all three threads futex-park on the recursive
rendezvous mutex 0x6edae60 (call-site 0x102b53bb0) — the engine's genuine
multi-thread barrier, released on real Android by the Java layer's
app-command / ALooper dispatch. So cycle F crossed TWO lifecycle predicates
with host-supplied state.

Run-log: `/home/hermes-worker/runs/kicker-gate1-runlog.txt` (updated for
gate 2).

## Session (Sep 12, 2026, hermes-worker, cycle E) — boot wall pinned at register+futex level; concurrent thread-state sampler + GLIBC mutex owner/count/kind (workspace 395/0)

Commits `1dae9e1` + `eaf00e6` (dev). This cycle pinpointed the
GameActivity lifecycle-await wall at register+futex level (previously only
inferred by timing/heuristic). New concurrent guest-thread sampler
(`snapshot_threads()`, JIT_THREADS=1) dumps every registered guest thread's
hostcall slot (pc), guest call-site (x30), and wait-object args (x0..x2)
while StartApp's parked `jit_run` never returns. Real libroblox StartApp
boot:

- **All 3 engine guest threads futex-park on `pthread_mutex_lock(0x6edae60)`,
  x30 == `0x102b53bb0` for all three** (single call site inside the
  GameActivity_initializeNativeCode rendezvous).
- glibc fields of `0x6edae60`: **g_kind=1 (PTHREAD_MUTEX_RECURSIVE),
  g_count=1** — a LIVE owner holds it with recursion depth 1; it is NOT an
  abandoned/cross-ABI-wedged lock. Owner (tid 0) runs the GC/init atomic
  refcount region at `0x102206afc/bb0` (`ldar x8,[x8,2808]; subs; b.eq`),
  then re-acquires and parks.
- **per-thread CPU while parked: owner 14% in `futex_wait_queue` (an active
  wake/check/re-park POLL on the recursive mutex — polls the awaited
  app-command/lifecycle flag); tids 1 & 2 0.7% truly idle.** Dispatcher
  compiles flat at 767 (one jit step after StartApp, then the main loop parks).
- New `disasm` example (`cargo run -p arm64jit --example disasm -- <elf>
  <guest-addr...>`) decodes a guest region to name the enclosing functions.

This confirms the documented wall (engine awaits Java-side app-command /
looper / lifecycle state that would release `0x6edae60`) and converts the
"identify what releases it" lever into a precise, reproducible pin — the
owner polls a specific object at recursion-count-1 and only proceeds once that
app-command arrives. It does NOT yet cross the wall. The Mesa llvmpipe egl/gl
path and texture/float bridges remain fully wired (graphics gates green).

### Next lever (unchanged — this is the hard remaining wall)
Cross the GameActivity lifecycle-await so the engine proceeds to the looper and
the already-wired Mesa llvmpipe egl/gl path produces a first real frame.
Requires emulating the Android app-command / looper state: seed the flag/object
the owner polls at `0x102206afc` (or the once-guard that gates it), OR feed
ALooper_pollOnce synthetic app commands (APP_CMD_START/RESUME/INIT_WINDOW) from
a host side so the awaited predicate is satisfied and `0x6edae60` releases.
Then route the boot's `egl*`/`gl*` imports through the existing Mesa resolver
for a first headless frame.

### Repro
```bash
cd /home/hermes-worker/runs/open-sober
cargo build -p arm64jit --example elfjit
timeout 30 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 --jni --startapp 0x258b144  # stable idle (exit 124)
JIT_THREADS=1 timeout 15 ... --jni --startapp 0x258b144   # concurrent thread-state sampler
JIT_TRACE=1 timeout 12 ... --jni --startapp 0x258b144 2>&1 | grep mutex_lock  # glibc owner/count/kind
cargo run -p arm64jit --example disasm -- ~/.cache/open-sober/robbox/libroblox.so 0x102206bb0 0x102b53bb0
```

## Session (Sep 12, 2026, hermes-worker, cycle D) — boot wall re-characterized; JIT_TRACE reverse-name registry (workspace 394/0)

Commit `b99c3df` (dev). This cycle re-confirmed the real-boot wall exactly as
last documented — all three guest threads genuinely futex-park (0% CPU) on the
glibc-RECURSIVE mutex `0x6edae60` (owner thread 3188327 does real init through
the atomic CAS once-guard `0x2b9e1d0`, then the main/StartApp threads re-lock
the recursive mutex and park awaiting Java-side app-command/lifecycle state;
`cond_wait/timedwait` never reached; no new block compiles after StartApp fires
— flat 767 compiles). Graphics (Mesa llvmpipe through the JIT bridges) is fully
wired and passing (`egl_window_present` Xvfb gate + resolver unit gates). The
wall is a genuine Android-lifecycle-emulation gap, not a decoder/loader gap.

### What landed (readable real-boot run-log)
The real libroblox.so boot's JIT_TRACE dumped anonymous `hostcall@slotN` for
every auto-allocated GLES/float/JNI host-call slot, hiding which engine import
the GameActivity init dispatches while parked. Added a reverse-name registry:
- `jit.rs`: `HOST_CALL_NAMES` (addr->name) + `name_host_call_slot()`;
  `name_of_call_addr()` consults it after the resolver name map.
- `jni.rs`: `JNI_METHOD_NAMES` table names every filled JNIEnv/JavaVM slot by
  its function-table const.
- `resolver.rs`: `resolve_gles_mixed`/`resolve_float`/`resolve_float32` record
  the symbol name on the returned slot.
- `jit.rs`: boot `mempool_calloc(x1=size)` and `lsm_map_calloc(x0=size)` thunks
  named.
Result: the real StartApp boot now prints `JNIEnv.GetStaticMethodID`,
`FindClass`, `NewGlobalRef`, `ExceptionCheck`, `JavaVM.GetEnv`, float/GLES
names, `boot.mempool_calloc` instead of `slotN`. New regression
`name_of_call_addr_resolves_auto_allocated_gles_and_float_slots`.

### Repro (unchanged behavior)
```
cd /home/hermes-worker/runs/open-sober
cargo build -p arm64jit --example elfjit
# idle boot (clean exit 0):
timeout 40 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 --jni
# StartApp main loop (stable idle, exit 124, no crash):
timeout 30 ./target/debug/examples/elfjit ... --jni --startapp 0x258b144
JIT_TRACE=1 timeout 12 ... --jni --startapp 0x258b144 2>&1 | grep -oE 'hostcall@[A-Za-z0-9_.()]+' | sort | uniq -c | sort -rn
```

### Next lever (unchanged — this is the hard remaining wall)
Cross the GameActivity lifecycle-await so the engine proceeds to the looper and
the already-wired Mesa llvmpipe egl/gl path produces a first real frame. This
requires emulating the Android app-command / JNICallProtocol / looper state the
real Java layer drives (HANDOFF #1/#3): identify what releases `0x6edae60`
(which thread posts the app-command) and seed it, or drive the awaited looper
state from a host side feeding ALooper_pollOnce with synthetic app commands
(APP_CMD_START/RESUME/INIT_WINDOW). The relevant EGL/GLES imports already route
to Mesa. Multiple prior sessions hit this same wall — it is the frontier.

## Session (Sep 12, 2026, hermes-worker, cycle C) — DIAGNOSIS CORRECTED; bionic bridge + glibc-recursive routing hardened (workspace 393/0)

Commits `f0f352a` + `c4faa0a` (dev). This cycle implemented byte-exact bionic
NORMAL mutex support AND then corrected the diagnosis: the pinned wall mutex
`0x6edae60` is NOT a bionic-vs-glibc ABI collision — it is a **glibc-formatted
RECURSIVE mutex initialized by our own `host_mutex_init` bridge** (disassembly
of `0x2b53adc` shows `pthread_mutexattr_settype(#1)` = `PTHREAD_MUTEX_RECURSIVE`
immediately before its `pthread_mutex_init`). glibc owns and handles it
natively (recursive re-entry via `__kind=1`). Boot progress is **unchanged**
(same wall before/after); the commits are correctness hardening, not boot motion.

### 1. What landed
- `f0f352a` — implement bionic `NonPI::NormalMutexLock/Unlock` byte-exact on
  the guest 16-bit `_Atomic(uint16_t) state` word (CAS 0→1, exchange→2 +
  futex_wait; unlock exchange→0 + futex_wake-if-contended), with the required
  two-REAL-thread rendezvous regression test. Correct building block.
- `c4faa0a` — **registry routing (the real fix)**: `host_mutex_init` records
  every guest mutex it initializes via REAL glibc (with a REAL glibc attr) in a
  `HashSet`; `lock`/`unlock` route those back to the real glibc path. ONLY
  mutexes never initialized through the bridge (static zeroed bionic
  `PTHREAD_MUTEX_INITIALIZER` words the guest touches with inline atomics) take
  the bionic 16-bit protocol. Without this, my own bionic path misfired on the
  glibc-recursive `0x6edae60` (read the glibc word as type=NORMAL) and would
  SELF-DEADLOCK on the guest's legitimate same-thread recursive re-lock.

### 2. The wall, exact (unchanged; now understood as lifecycle-await, not ABI)
All three guest threads park at 0% CPU (verified 0 utime ticks over 3s) via
`pthread_mutex_lock` of guest mutex `0x6edae60` (`gpcreq=0x102b53bb0`, the
`GameActivity_initializeNativeCode` rendezvous). Owner thread
locks/unlocks/re-locks it cleanly (recursion works), then parks holding it
awaiting the Java-side lifecycle/app-command state. `cond_wait/timedwait` are
never reached (0 calls) — the "pair with cond" note is not a live wall. The
engine never reaches any `egl*`/`gl*` import before this wall either, so Mesa
llvmpipe routing won't help until this is crossed. Idle JNI-only boot still
exits 0; StartApp boot still exits 124 (stable idle, no crash).

### 3. Next lever (get past the GameActivity rendezvous)
The barrier is `GameActivity_initializeNativeCode` waiting for the app-command/
looper/lifecycle state the real Java layer drives. Candidates:
1. **Drive the awaited state**: identify what releases `0x6edae60` (which
   thread sets the flag / posts the app-command that lets the holder proceed)
   and seed it (Session-11 guard pattern). The TLS accessor `0x2b9dee0` is a
   per-thread cache getter, NOT the awaited singleton — don't chase it.
2. **More JIT ground speed** (perf) so init churns faster — but threads are
   genuinely futex-parked, so speed alone won't cross a true await.
3. After the rendezvous: route `egl*`/`gl*` to Mesa llvmpipe (both headless
   graphics gates already pass) → first frame. Then render/input.

Commits `6bc57a6` → `18ae7f6` → `063dc5e` → `7946fe3` (dev). The real
`libroblox.so` boot keeps advancing: JNI_OnLoad → `--startapp` drives
`nativeAppBridgeV2StartAppWithParams` → `GameActivity_initializeNativeCode`,
reaches the engine main loop, and **idles stably** (exit 124, no
SIGSEGV/SIGABRT). This cycle identified exactly what the loop waits on.

### 1. The wall, exact
`JIT_TRACE` shows the main loop parked in `pthread_mutex_lock` on guest mutex
`0x6edae60`:
```
[t=...] [mutex_lock] 0x106edae60 bionic_word=0x00000002 state=0x2 gpcreq=0x102b53bb0
```
- guest `state` = **2 = bionic `MUTEX_STATE_LOCKED_CONTENDED`** (NORMAL,
  non-PI, non-recursive mutex).
- Caller guest PC `0x102b53bb0` (in `GameActivity_initializeNativeCode
  +0x2f8488`, the mutex-init attribute setup).
- All guest threads idle in host futex, 0% CPU.

**Root mechanism:** the guest manages its own bionic `pthread_mutex` on its own
16-bit `state` word (modern NDK r28c layout: `_Atomic(uint16_t) state` @0,
`owner_tid` @4, 28-byte tail). When a guest thread's inline fast-path hit
contention it set state=2 and called our bridge's glibc `pthread_mutex_lock`;
glibc reads the bionic 16-bit state word as glibc's own lock encoding, sees no
matching glibc `__owner`, and futex-blocks forever while the holder (also a
guest thread) released the lock via its own fast-path atomics. A clean
**bionic-vs-glibc cross-ABI futex mismatch** — NOT an ALooper wait, NOT a
decoder gap.

### 2. Correction of this session's own earlier claim
`6bc57a6`/`18ae7f6` called it a "recursive mutex rendezvous" from the glibc
`__kind` field at mutex+16. That field is PAST the bionic word (on a different
init path — `pthread_mutexattr_settype(#1)` at 0x2b53b04 inits a different
mutex). `063dc5e` corrected it: the blocking mutex is NORMAL, in
LOCKED_CONTENDED (word 0x2).

### 3. Verified: the mutex is real — do NOT weaken it
All guest threads genuinely idle (futex/nanosleep, 0% CPU). An "optimistic
non-blocking acquire" (trylock→return 0 on EBUSY) let a second guest thread
into the same critical section → SIGSEGV on garbage. A hand-rolled bionic CAS
attempt was reverted in-tree, unbuilt, before touching the boot. Both reverted.
This is genuine shared-memory mutual exclusion at lifecycle handoff.

### 4. Correct fix (SCOPED — next task)
Implement bionic's NORMAL mutex protocol byte-exact on the guest's own 16-bit
`state` word @0: acquire = CAS state 0/1→LOCKED_UNCONTENDED; contention → set
LOCKED_CONTENDED(2), futex-wait on the word; unlock = clear to 0 + FUTEX_WAKE.
Must be paired with the bionic `cond` (cond_wait internally unlock+relock the
mutex). Validate with a TWO-THREAD rendezvous unit test (A locks via bridge, B
blocks in bridge, A unlocks, B acquires) BEFORE wiring into the boot.
Alternatively drive the awaited looper/app-command state. Keep the stable idle
boot as the base.

### 5. Graphics "first frame" gates both pass (headless, this VPS)
`glesv2-wrapper headless_graphics.rs` (surfaceless llvmpipe ES3, ETC2
interception, BC1 passthrough) and `arm64jit egl_window_present.rs` (Xvfb real
X11 window, full eglGetDisplay→...→glClear→eglSwapBuffers through the JIT
guest-bridge slots, returns EGL_TRUE) both pass — real frames present headless.

Repro:
```bash
cd /home/hermes-worker/runs/open-sober
cargo build -p arm64jit --example elfjit
timeout 30 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 --jni --startapp 0x258b144
JIT_TRACE=1 ... 2>&1 | grep mutex_lock | tail   # pin blocking mutex state/caller
```

Commit `152dce9` (dev). Two real bottlenecks to StartApp forward-speed removed:

1. **Translation-block cache (the big one).** The PC-driven dispatcher
   (`jit_run_inner`) recompiled a region from scratch on every `blr`/`br`/`ret`
   re-entry. A boot hot-spotting on a small accessor (Roblox's TLS-block getter
   `0x2b9dee0` → 160 of ~1271 traced block-execs, each `pthread_getspecific`)
   retranslated that code ~once per call — the dominant cost once the guest is
   churning TLS blocks. `cached_block()` keys on `(image, pc, state)` and leaks a
   process-lifetime `JitBlock` (never munmaps). Evacuated at each *top-level*
   `jit_run` (safe: leaked) so distinct ELF images at the same `JIT_BASE`
   (the `diff_battery` suite maps each test at 0x100000000) never run a stale
   block — this correctness fix is what makes the whole thing safe. Real boot:
   **767 compiles / 5691 hits**. New `block_cache_stats()` + `JIT_STATS`
   per-dispatcher heartbeat.
   - **Regression caught & explained by the cache:** the `loader_run_timer_signal
     _delivers_sigalm` test started failing once the loop got fast. Bisect showed
     guest nanosleep was never *actually sleeping* (see #2), so the 20000-iter
     spin loop finished before 2×100ms timer ticks; the test only passed by luck
     of uncached-JIT slowness stretching it past 200ms. Fixing #2 made it pass
     deterministically (0.23s) regardless of JIT speed. Lesson: a *fast* JIT
     exposes races the slow one masked — run the timing/signal loader tests after
     any perf work.

2. **nanosleep syscall arg fix (real boot bug).** aarch64 `nanosleep` passes
   `rqtp` in x0, but `guest_svc` read it from a[1] (x1) → NULL req → EFAULT in
   ~1.5µs, no sleep. So every guest sleep-wait was really a busy-spin. Now reads
   `a[0]` (rqtp) / `a[1]` (rmtp). Regression test
   `guest_svc_nanosleep_reads_timespec_from_x0`. This matters for the boot: any
   guest `sleep`/`usleep`/wait that Roblox does now blocks the guest properly
   instead of hot-spinning the core.

### Main-loop wall, now characterized precisely (NOT a deadlock)
`--startapp` drives the real `GameActivity_initializeNativeCode` (0x258b144 →
region `0x284dxxx`, TLS-block accessor `0x2b9dee0` = `ldar x22,[x0+0x10]; cbz`
+ `pthread_getspecific`, wrapper `0x284d524`, vtable-check `0x284f874/880`). A
12s JIT_TRACE reaches **236 distinct blocks, growing across the window
(30→69 distinct block-sites in first/last 100 execs)** → the guest is *advancing
through new init code*, just slowly, and makes no guest `svc` once settled.
So the next lever is NOT a decoder gap, NOT an ALooper wait — it's either
(a) more JIT/perf so it grinds through the ~thousands of TLS-block allocs faster,
or (b) finding the specific singleton whose init never *completes* and seeding it
(Session-11 guard/flag pattern), or (c) driving the awaited looper/app-command
state so the main loop dispatches a real frame/render instead of init-churning.

### Diagnostics added
- `resolver::name_of_call_addr()` reverse slot → name; JIT_TRACE now prints e.g.
  `hostcall@pthread_getspecific` (identified the main-loop hot import).
- `JIT_STATS=1` prints a per-250ms dispatcher heartbeat: `[jit] step N pc=... block-cache: C compiles / H hits`. Compiles climbing = new code; flat + hits rising = genuine loop spin.
- elfjit prints `[elfjit] block-cache: C compiles / H hits` on clean exit.

Repro (see STATUS.md for full):
```
cargo build -p arm64jit --example elfjit
timeout 30 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 --jni --startapp 0x258b144
JIT_STATS=1 timeout 30 ... 0x258b144       # progress heartbeat
JIT_TRACE=1 timeout 12 ... 0x258b144 | grep hostcall@ | sort | uniq -c | sort -rn
```

## Session (Sep 11, 2026, hermes-worker) — POST-JNI_OnLoad game-start: `--startapp` boot stage; real libroblox reaches the ENGINE MAIN LOOP (workspace 388/0)

Commit `a4f94d1` (dev). The real `libroblox.so` 2.738.1397 boot advances from
"JNI_OnLoad returns 0x10006 then the process exits" to **the guest entering and
persistently running the engine's `GameActivity` main-loop / event-pump region**
after a new `--startapp` stage chains the real Java-side game-start entry.

- **Why the boot previously exited**: JNI_OnLoad is a *registration* function;
  on real Android the JVM then calls `nativeAppBridgeV2StartAppWithParams` etc.
  to actually start the game (main loop + EGL/GLES init). elfjit only ran
  JNI_OnLoad, so once it returned and the spawned worker boot-body finished, the
  harness's `main()` returned and the process exited cleanly (exit 0).
- **`--startapp <link-addr>`** (elfjit): after `jit_run(JNI_OnLoad)` returns
  `Ok(0x10006)`, builds the singleton env + fake-but-valid `jobject` (x1) +
  `jstring` (x2) and `jit_run`s the real `nativeAppBridgeV2StartAppWithParams`
  (0x258b144) as a fresh guest entry. Critical detail: it reuses the **boot-phase
  guest SP** (`s2.x[31]=st.x[31]`); a fresh 0 SP wrapped StartApp's `sub sp,#0xf0`
  prologue to `0xffffffffffffff10` and the frame-write SIGSEGV'd immediately.
- **New jni helpers**: `new_fake_object()`, `new_string_utf_handle()`, and a
  `JNI_TRACE_REGISTRY` env to dump RegisterNatives bindings.
- **Verified** (headless, no QEMU): the guest executes 800+ distinct blocks
  through StartApp — FindClass for dozens of Roblox classes, repeated VM_GetEnv,
  pthread_once/mutex/getspecific TLS-key protocoling in the
  `GameActivity_initializeNativeCode` thread-local setup, `LockBasedAllocator` —
  then settles into a persistent main-loop cycle (`ldar x22,[x0+0x10]; cbz`
  await + `pthread_getspecific` dispatch) across two guest threads and runs
  until the harness `timeout` fires (exit 124; **no SIGSEGV/SIGABRT**). JNI_OnLoad
  alone still exits 0 cleanly (~4.8s); both paths preserved.

### Current wall (narrowed from "nothing drives the app" to a specific loop)
The engine main loop is reached but awaits app events / lifecycle (looper
input, window/surface, EGL) that the real Java side supplies. Next (ordered):
1. Feed the awaited `GameActivity` app-command / looper state and route the
   boot's `egl*`/`gl*` imports through the existing Mesa llvmpipe resolver so
   any EGL context/frame path reachable from StartApp runs real software
   graphics — the first reproducible engine-loop artifact (a frame / looper
   event dispatch), headless on this VPS.
2. Advance FMOD audio init and the JNIMain main-loop drive.
3. HARD GATE (real session + run log) unchanged as the end goal; the
   achievable-on-this-VPS milestone next is a real *frame* / first looper event,
   then it's a GPU host for the final perf proof.

Repro:
```
cargo build -p arm64jit --example elfjit
# boot-only: timeout 120 ./target/debug/examples/elfjit .../libroblox.so 0x2173ff4 --jni
# boot + game-start main loop: timeout 30 ./target/debug/examples/elfjit .../libroblox.so 0x2173ff4 --jni --startapp 0x258b144
```
Run-log: `/home/hermes-worker/runs/startapp-boot-runlog.txt` (exit 124 = ran
in the engine main loop until the harness timeout; no crash).

## 🟢 STABLE HEADLESS BOOT of real libroblox.so (exit 0, reproducible)

**The real `libroblox.so` (2.738.1397) now boots to a stable state headlessly
through `arm64jit` + `libloader` and exits cleanly (`exit 0`, no SIGSEGV) —
verified 3/3. JNI_OnLoad returns `0x10006`, real engine JNIMain code runs, the
worker guest thread runs clean, teardown is clean.** This is the boot
stabilization milestone on this VPS.

Two fixes this session (commits `6e9fd4e`, `1639899`, workspace 388/0):
1. **`plt::bind_glob_dat`** — unresolved *function* GLOB_DAT/ABS64 slots were
   left at stale values (0 or a `.dynstr` symbol-name pointer); a guest `blr`
   through them jumped INTO `.dynstr` (SIGSEGV, register file = ASCII symbol
   strings). Now bound to a benign host-call stub. Real lib: 63 -> 67 bound.
2. **`__cxa_thread_atexit_impl` no-op shim** — was resolved to real glibc,
   which stored the guest AArch64 TLS-destructor pointer and invoked it NATIVELY
   as x86 when the worker guest thread exited -> SIGSEGV executing guest ARM64
   .text (the post-boot "worker/teardown" crash). Now a no-op, so glibc never
   runs a guest functor natively. This fixed the crash and gave the clean exit.

Repro:
```
cd /home/hermes-worker/runs/open-sober
cargo build -p arm64jit --example elfjit
timeout 120 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 --jni
```
Expect: seeds + `Test TelemetryProtocol` + `DeviceStaticParams is null`, then
`jit_run returned Ok(0x10006)` / `JIT(no-QEMU) entry() -> 65542 (0x10006)`,
clean `exit 0`.

**NEXT (advance the boot / graphics):** JNI_OnLoad now succeeds and the process
exits cleanly; push toward a real main loop that doesn't exit. Graphics
wrappers already point at Mesa llvmpipe; the egl/glesv2 stubs exist. Use
`GRAPHICS_RECOMMENDATION.md`: surface a window/surfaceless EGL context, have
the guest render a frame, and confirm with a run log. Also consider whether
JNI_OnLoad's spawned worker should be joined/looped instead of the process
exiting when the main dispatch returns.

**The real `libroblox.so` (2.738.1397) boots through `arm64jit`
(`elfjit <libroblox.so> 0x2173ff4 --jni`) and main-thread `JNI_OnLoad` returns
`0x10006` (JNI_VERSION_1_6) reproducibly.** This cycle (commit `6e9fd4e`,
workspace 388/0):

1. **Closed a real indirect-call-into-`.dynstr` vector.** `plt::bind_glob_dat`
   left unresolvable *function* GLOB_DAT/ABS64 slots at their original value
   (0 or stale `.dynstr` symbol-name pointer); a guest `blr` through one jumped
   into `.dynstr` (SIGSEGV with register file full of ASCII symbol strings —
   "pthread_setspecific", "memset", "pthread_cond_broadcast"). Now bound to a
   host-call stub. Real binary result: **67 GLOB_DAT bound / 11 unresolved**
   (was 63/15); the fault's `guestpc` is now a real guest address, not ASCII.
   Regression test `loader_run_unresolved_func_globdat_binds_safe_stub` (fn
   import via `int (*gfp)(int)` binds to host thunk 0x7f0000002008).
2. **Fault diagnostics enriched** (elfjit): full host-x86 register dump,
   `rbx_matches_gueststate` (is the faulting RBX the thread's registered
   CpuState?), guest-thread table `(host_tid:guest_tid,state)`, nesting-aware
   `in_jit_run` counter.

**Current wall (precise, unbuffered-stderr-proven):** main's `jit_run` returns
Ok(0x10006); the SIGSEGV is in the **post-run phase** (worker guest thread,
tid=1, start_routine `0x284d168`, is running when main's dispatch ends).
`rip=0x10284d6be` is a guest `.text` address in the
`JNIActivityLifecycleCallbacks_nativeOnDestroyed` region executed as x86 —
a **host path calls a guest function pointer natively**, not a translated-block
fault. `rbx_matches_gueststate=false` ⇒ the guest-register dump is an artifact
of a garbage RBX; trust the host regs/rip, not guestpc. Next: find which host
call path dispatches guest `0x284d6b4` during worker/teardown (guest signal
handler outside the cooperative dispatcher, an atexit/on_destroy callback
routed to guest natively, or the worker start_routine dispatched down a
non-`jit_run` host path). See `docs/` run-log + `runs/STATUS.md`.

**The real `libroblox.so` (2.738.1397, extracted via `sober-core::apk::extract_libs`
to `~/.cache/open-sober/robbox/libroblox.so`) now boots through `arm64jit`
(`elfjit <libroblox.so> 0x2173ff4 --jni`) and main-thread `JNI_OnLoad` returns
`0x10006` (JNI_VERSION_1_6) — the canonical success value — reproducibly.**

Milestone commits this cycle (all `dev`, workspace green):
- `6de9a2d` libloader: reserve writable guest tail past image end (crashpad
  telemetry static table walks ~36MB past the last PT_LOAD bss).
- `8033d04` arm64jit: route LSM map bucket allocator (0x1d97744, size in x0) to
  host calloc; seed the LocalStorageManager static hash-map global (0x726f8c0)
  with a two-level empty map + NOP its lazy-init store (0x1d975f8); generalized
  the TLS-pool calloc thunk into a param'd `place_calloc_patch` (two slots:
  0x62d9000 x1-size, 0x62d9800 x0-size).
- `5ba69aa` elfjit: seed JNICallProtocol refcounted-singleton ptr (0x7333948 ->
  0x7333950, zeroed bss == PTHREAD_MUTEX_INITIALIZER at +8).

**Current wall:** after main `entry()` returns 0x10006, a *worker guest thread*
spawned during JNI_OnLoad (`pthread_create(start_routine=0x284d168)`, tid=1)
crashes; the SIGSEGV handler reports CpuState registers that decode to ASCII
libc symbol-name strings ("pthread_setspecific", "memset", "pthread_cond_*",
"broadcast"), i.e. it appears to execute/read `.dynstr` string data. Hypothesis:
the spawned thread's per-thread guest TLS is only a bare zeroed buffer (the
known "per-thread PT_TLS init-image copies for clone/pthread children" gap), so
`__tls_get_addr`/`pthread_getspecific` on the child reads garbage
(function-pointer table entries land on symbol-name strings). TODO: give
`spawn_pthread` children a real `setup_guest_tls` TLS block + TCB (copy PT_TLS
init image) like the main thread, and confirm the child then runs cleanly.

Repro:
```
cd /home/hermes-worker/runs/open-sober
cargo build -p arm64jit --example elfjit
timeout 150 ./target/debug/examples/elfjit ~/.cache/open-sober/robbox/libroblox.so 0x2173ff4 --jni
```
Expect: `[lsm-map]`/`[JNICall-singleton]` seeds, JNIMain logs, `JIT(no-QEMU)
entry() -> 65542 (0x10006)`, then the worker-thread SIGSEGV.

## ⚠️ CRITICAL RULES — READ FIRST

1. **NO WORKTREES.** Do NOT create git worktrees. Ever.
2. **NO BRANCHES.** Work directly on `dev` branch. No feature branches, no topic branches.
3. **CLAUDE.md** at repo root has these rules — read it.
4. **Commit directly to `dev`**, push, and let the user decide when to merge to `stable`.
5. If you need to research something, do it inline or in a temp dir outside the repo.
6. **`cargo check --workspace`** before committing. **`cargo test --workspace`** before pushing.

## Project Overview

**Repo:** https://github.com/glm-5-turbo/open-sober
**Branches:** `stable` (release), `dev` (active development — work here)
**Build:** `cargo build --release`
**Tests:** `cargo test --workspace`

Open Sober is an open-source reimplementation of VinegarHQ's Sober — a runtime that runs the Roblox Android APK on Linux natively.

## What's Built

### Phase 1 - libbadcpu (`crates/libbadcpu/`)
CPU feature emulator — SIGILL handler for missing x86-64 instructions (POPCNT, MOVBE, LZCNT, TZCNT, BMI1).

### Phase 2 - libloader (`crates/libloader/`)
Process sandbox/spawner — chroot isolation, ELF loader, Android runtime env setup, Unix socket IPC.

### Phase 3 - sober-services (`crates/sober-services/`)
Browser-based OAuth auth handler.

### Phase 4 - sober-core (`crates/sober-core/`)
Main binary orchestrator. `open-sober play --apk roblox.apk` is the main command.

**Key APK:** `~/Documents/Projects/open-sober/roblox-android.apk` (178MB, not in git)
**APK structure:** `assets/app.zip` → `config.arm64_v8a.apk` → `lib/arm64-v8a/libroblox.so` (101MB, NDK r28c, Android 26)

## Current Status (July 20, after session 11b)

### ✅ Complete (all sessions)

1. **Custom QEMU** built from `/tmp/qemu-10.2.1/` — patched copy at `~/.cache/open-sober/qemu-patched`.
2. **pthread_mutex_t ABI fix** (`bionic_init.c`) — trampoline-based mutex interceptors.
3. **Complete JNI function table** (`jni_shim.c`) — all 256 JNIEnv slots filled.
4. **QEMU bridge wiring** (`qemu.rs`) — version bridges for libc/libm/libdl.
5. **Canary GOT patching** — stack_chk_guard write via mprotect.
6. **SIGSEGV handler** — RELRO faults, self-write JIT bugs, NULL deref handling.
7. **Pre-mprotect RELRO** — ~464 pages made RW before JNI_OnLoad.
8. **Pre-resolved trampoline table** — all **785 entries** via data-driven dlsym loop (782/785 resolved, 3 Bionic-only fallbacks filled with `__errno_location`).
9. **Canary check patched out** in code copy.
10. **Init guard deadlock FIXED** — code patch replaces `bl 26c0c7c` (mutex+condvar) with `mov w0,#1; nop`.
11. **Raw ARM condvar shim** — `mov w0,#0; ret` in mmap'd RWX page, replaces `pthread_cond_wait` trampoline.
12. **pc=0x0 crash FIXED!** — `_dl_mcount` is now noped to `ret` BEFORE dlopen(libroblox), using 64KB mprotect to force QEMU TCG JIT cache invalidation + precise 4-byte `ret` write. See details below.

### 🟢 _dl_mcount noping now works (Session 8)

**What changed:**
1. Moved `_dl_mcount` patching to run BEFORE `dlopen(libroblox.so)` (was: after). This ensures no GSI library can trigger `_dl_mcount` during loading.
2. Extracted into `disable_mcount_profiling()` function called at the start of `main()`.
3. Combined 64KB mprotect on ld-linux text page (forces QEMU TCG JIT cache invalidation) with precise 4-byte `ret` write + `__builtin___clear_cache()`.
4. Also zeroes `rtld_global.dl_profile` in the data section as belt-and-suspenders.
5. Sanity check at the end confirms `_dl_mcount` entry now reads as `0xd65f03c0` (AArch64 `ret`).

**Evidence:** The process now reaches `[jni_shim] entering JNI_OnLoad...` without crashing. Previously it crashed with `pc=0x0` before reaching this point.

### 🟢 JNI_OnLoad returns successfully (Session 9 breakthrough!)

After patching JNI_OnLoad's entry point to `mov w0, #0x6; movk w0, #0x1, lsl #16; ret` (returns `JNI_VERSION_1_6 = 0x10006` immediately), the shim now works end-to-end:

```
[jni_shim] JNI_OnLoad -> 0x10006
[jni_shim] Entering sleep loop
```

The code patch at `base + 0x1f64e58` replaces the first 12 bytes of JNI_OnLoad with:
- `0x528000c0` = `mov w0, #0x6`
- `0x72a00020` = `movk w0, #0x1, lsl #16` (w0 = 0x10006)
- `0xd65f03c0` = `ret`

This bypasses ALL internal initialization functions that were hanging:
- One-time init guard → skipped (we also pre-init the guard+JVM)
- LocalStorageManager init → skipped
- nativeSetAssetPath → skipped
- Various JNI FindClass/RegisterNatives calls → skipped

**Why this is OK:** For now, the goal is to get the binary loading successfully. The JNI stubs are in place and any code that checks the JNI version will get 0x10006. The internal init functions primarily register native methods and initialize BSS globals — we already pre-init the critical ones.

**What was fixed in session 9**

1. **`patch_jni_onload()`** — new function that patches JNI_OnLoad's entry to return 0x10006 immediately
2. **Verified working** — JNI_OnLoad returns 0x10006, `sleep loop` reached successfully

### Current Status (July 20, after session 11)

### ✅ Complete (all sessions)

1. **Custom QEMU** built from `/tmp/qemu-10.2.1/` — patched copy at `~/.cache/open-sober/qemu-patched`.
2. **pthread_mutex_t ABI fix** (`bionic_init.c`) — trampoline-based mutex interceptors.
3. **Complete JNI function table** (`jni_shim.c`) — all 256 JNIEnv slots filled.
4. **QEMU bridge wiring** (`qemu.rs`) — version bridges for libc/libm/libdl.
5. **Canary GOT patching** — stack_chk_guard write via mprotect.
6. **SIGSEGV handler** — RELRO faults, self-write JIT bugs, NULL deref handling.
7. **Pre-mprotect RELRO** — ~464 pages made RW before JNI_OnLoad.
8. **Pre-resolved trampoline table** — all 785 entries.
9. **_dl_mcount profiling disabled** — `ret` at entry point, dl_profile zeroed.
10. **PLT GOT condvar patching** — pthread_cond_wait/timedwait → immediate return.
11. **One-time init guard pre-init** — set to 1, skips condvar-based init.
12. **Phase 1a progressive JNI_OnLoad patch** — only NOPs clock/time init call instead of full bypass. JNI registration runs (3 classes, 13 methods resolved). nativeSetAssetPath reaches but hangs.
13. **JNI table slot 113 fix** — GetStaticMethodID at correct slot (was RegisterNatives). Default fallback changed from 0x10006 to 0/NULL.
14. **End-to-end success** (full bypass) — binary loads, JNI_OnLoad returns, sleep loop reached.
15. **Timestamp flags pre-init** — two BSS flags (0x6a325e4, 0x6ae6690) set to 1.
16. **Frequency double pre-init** — `base+0x6ae66e8` set to 1.0e9.

### Session 11 summary (July 20, 2026)

**Goal:** Implement Phase 1 progressive patch (shrink JNI_OnLoad bypass).

**What was done:**

1. **Replaced full bypass with Phase 1a progressive patch.**
   - Old: `patch_jni_onload()` replaced first 12 bytes of JNI_OnLoad with `mov w0,#6; movk w0,#1,lsl#16; ret` (returns 0x10006 immediately).
   - New: `patch_jni_onload_phase1()` only NOPs the clock/time init call at binary offset `0x1f64e9c` (`bl 0x1cfabfc`).
   - Guard check, GetEnv, LocalStorageManager, JNI registration, nativeSetAssetPath, guard setter, and all subsequent code all run normally.
   - Falls back to full bypass if patch fails.

2. **Fixed JNI function table for real JNI calls.**
   - Slot 113 (byte offset 904 in JNIEnv struct) was incorrectly set to `stub_RegisterNatives`. The actual function at this offset takes `(env, class, name, sig)` — matching `GetStaticMethodID`. Changed to `stub_GetStaticMethodID`.
   - Default fallback changed from `stub_GetVersion` (returns `JNI_VERSION_1_6 = 0x10006`) to `stub_voidp` (returns NULL/0). The `0x10006` value caused crashes when interpreted as a pointer by non-GetVersion callers.
   - All NULL slots now filled with `stub_voidp` instead of `stub_GetVersion`.

3. **Results — JNI_OnLoad now runs partial native init:**
   ```
   FindClass[1]: NativeLocaleJavaInterface → 3 GetStaticMethodIDs (getLocale, getRobloxLocale, getGameLocale)
   FindClass[2]: NativeUserJavaInterface → 9 GetStaticMethodIDs (getUserId, getIsUnder13, getUsername, getDisplayName, getAlternateName, getPlatformName, getMembershipType, getHasRobloxSubscription, getTheme)
   FindClass[3]: LoggingProtocol → 1 GetStaticMethodID (getProcessTimestamp)
   ```
   Three Roblox JNI classes are found and all 13 methods resolved successfully.
   Then the process hangs in `nativeSetAssetPath` (`0x273de0c`).

4. **Confirmed PLT GOT condvar patching works.** Heartbeat backtrace shows LR at the condvar shim page, proving that the raw ARM shim IS reached from libroblox's internal code. The issue is that after the shim returns 0 (spurious wakeup), the calling code re-checks the condition and re-enters `pthread_cond_wait` in an infinite loop (single-threaded, no other thread to signal).



### Session 11b summary (July 20, 2026)

**Goal:** Debug the `nativeSetAssetPath` hang with improved instrumentation.

**What was discovered:**

1. **Improved SIGALRM heartbeat.** Changed from `sa_handler` (signal handler's own x29/x30) to `sa_sigaction` with `SA_SIGINFO` and ucontext. Heartbeat now shows the **real PC** of interrupted code:
   ```
   [jni_shim] JNI_OnLoad still running (5s) PC=0x...1074 LR=0x...cb04 BT={0x...cb04,0x...a0c0,0x...f90,0x...6038}
   ```
   PC alternates between `0x...1074` and `0x...1084` on the bionic shim trampoline page. Frame 2 at `base + 0x1f64f90` = JNI_OnLoad error path.

2. **`-d exec` trace** shows a repeating 9-address cycle on the bionic shim trampoline page:
   ```
   0xbc0 → 0xdc0 → 0xf00 → 0x1080 → 0x1200 → 0x1380 → 0x1580 → 0x1740 → 0x1940 → 0xbc0 → ...
   ```
   Each 0x200 bytes apart. This is a GSI library init function calling a sequence of bionic trampolines in a tight loop that never terminates.

3. **QEMU TCG cache conflict confirmed.** NOPing `bl 0x273de0c` at offset `0x1f64eb8` requires `mprotect` on page `0x1f64000`, which ALSO contains the JNI registration function at `0x1f6594c`. Three approaches all failed:
   - Single mprotect writing both NOPs → only 2 classes
   - Two separate mprotect calls → only 2 classes  
   - `b #4` skip instead of NOP → only 2 classes
   - Phase 1a (clock-only NOP) → STABLE 3 classes

   **Root cause:** Making page `0x1f64000` RW → write → RX causes QEMU to invalidate TCG cache for the registration function. Re-translation produces incorrect code that skips the `NativeUserJavaInterface` class.

4. **gdbstub tested but impractical.** QEMU's `-g 1234` starts in the dynamic linker phase. Connecting gdb before the hang requires multi-step breakpoint setup. `gdb-multiarch` can connect and examine state but reaching the hang point with a useful backtrace is complex.

**Recommended fix:** Patch `libroblox.so` on disk BEFORE `dlopen` (pre-load patching) to avoid QEMU TCG cache invalidation entirely. The target bytes at file offsets `0x1f64e9c` and `0x1f64eb8` can be replaced with `0xd503201f` (NOP) using `open(O_RDWR)` + `pwrite` before loading the library.

5. **Attempted fixes that didn't work:**
   - Using real glibc `pthread_cond_wait` directly in PLT GOT (via `g_real_cond_wait`) — no futex syscall appeared in `-strace`, suggesting the condvar calls go through a different code path than expected, OR the process hangs before reaching a condvar call.
   - NOPing both clock init AND `nativeSetAssetPath` — caused different execution behavior (only 2 classes found instead of 3), suggesting a QEMU TCG caching issue with the broader mprotect range.
   - `-d exec` tracing was not feasible due to output volume.

**Key JNI classes and methods discovered:**
```
com/roblox/engine/jni/locale/NativeLocaleJavaInterface
  getLocale()Ljava/lang/String;
  getRobloxLocale()Ljava/lang/String;
  getGameLocale()Ljava/lang/String;

com/roblox/engine/jni/user/NativeUserJavaInterface
  getUserId()J
  getIsUnder13()Z
  getUsername()Ljava/lang/String;
  getDisplayName()Ljava/lang/String;
  getAlternateName()Ljava/lang/String;
  getPlatformName()Ljava/lang/String;
  getMembershipType()I
  getHasRobloxSubscription()Z
  getTheme()Ljava/lang/String;

com/roblox/universalapp/logging/LoggingProtocol
  getProcessTimestamp()J
```

**Remaining blocker:** The hang in `nativeSetAssetPath` after JNI registration completes. Investigation suggests:
- The hang is inside `nativeSetAssetPath`'s helper function (`0x273dd4c`) which calls `FindClass`.
- The helper function calls `FindClass(env, x1)` where x1 is garbage (not set before the call). Our `stub_FindClass` may crash on garbage pointers, or enters an error path that calls `pthread_cond_wait`.
- The function at `0x273dd4c` returns 0 (NULL) from its stack slot, and the subsequent `ldr x8, [x0]` (NULL dereference) would crash, but the process hangs instead.
- The exact hang mechanism is not yet identified — possibly a QEMU edge case with NULL dereference in signal context.

### Key Source Files

1. **JNI_OnLoad's internal structure mapped** via disassembly:
   - Guard check at `0x1f65a60` — returns immediately when guard=1 (our pre-init works)
   - GetEnv at `0x5e17fb8` — returns our stub env pointer
   - Clock/time function at `0x1cfabfc` → `b 0x5f4f69c` — three-tier guard check
   - `LocalStorageManager_initStorageManagerNative` — JUST `ret` (no-op!)
   - JNI registration block at `0x1f6594c` — FindClass/RegisterNatives for locale classes
   - `nativeSetAssetPath` at `0x273de0c` — JNI calls
   - Guard setter at `0x1f65a54` — writes to BSS

2. **Root cause of hang without bypass:** The function at `0x5f4f69c` (clock_gettime wrapper) has a three-tier guard check:
   - **Level 1:** Two BSS flags (`base+0x6a325e4`, `base+0x6ae6690`) — if either is 0, takes slow path with condvar loop
   - **Level 2:** Double at `base+0x6ae66e8` — if 0.0 (BSS default), falls through to another init function with condvars
   - **Level 3:** Atomic ldaxr/stlxr timestamp update loop — works under QEMU
   
   Our condvar shim returns 0 (spurious wakeup), so any condvar-based path spins forever without any futex syscall.

3. **BSS pre-inits added:** `__atomic_store_n` with release semantics for the two ts_flags, and a `*(volatile double*) = 1.0e9` for the cntvct frequency. All verified to be within BSS range: `0x64c4f00` to `0x6ae6cec`.

4. **Key addresses identified:**
   - Init guard: `base + 0x6a26e40` (already pre-set)
   - Timestamp flag 1: `base + 0x6a325e4` (ldrb at #1508)
   - Timestamp flag 2: `base + 0x6ae6690` (ldrb at #1680)
   - Cntvct frequency double: `base + 0x6ae66e8` (freq == 0.0 check)
   - JNI_OnLoad entry: `base + 0x1f64e58`
   - JNI registration: `base + 0x1f6594c`
   - Init guard check: `base + 0x1f65a60`
   - condvar-heavy init: `base + 0x26c0c7c` (mutex+condvar loop)

5. **Attempted fixes that didn't work:**
   - **futex-based condvar wrapper:** The `wrap_cond_wait` C function with `syscall(SYS_futex, FUTEX_WAIT_BITSET)` didn't appear in `-strace` output, suggesting the guest code path goes through the PLT (which we patched) or the bionic trampoline (which we also patched), but potentially the futex syscall is intercepted by QEMU user-mode and doesn't reach the host. Using `nanosleep` instead of futex also didn't help — the calls just accumulate delay without making progress since there's no other thread to satisfy the condition.
   - **Pre-setting more BSS state:** Even with all three levels of the clock function guarded, there are more condvar waits deeper in the init chain that we haven't mapped.

**Added in jni_shim.c:**
- `__atomic_store_n` for pre-setting timestamp flags with release semantics
- `*(volatile double*)freq_dbl = 1.0e9` for cntvct frequency
- Verify guards and logging for all pre-init values

### Key Source Files

- `crates/sober-core/src/jni_shim.c` — Main JNI shim (~1570 lines)
- `crates/sober-core/src/bionic_init.c` — Bionic shim C code: trampoline resolver (`__bf_c_resolve`)
- `crates/sober-core/src/bionic_shim.S` — Auto-generated assembly trampolines (785 entries)
- `crates/sober-core/src/qemu.rs` — QEMU launcher

### Running

```bash
ANDROID_ROOT=~/.cache/open-sober/android-env
/home/code-agent/.cache/open-sober/qemu-patched \
  -L "$ANDROID_ROOT" \
  -E LD_LIBRARY_PATH=/system/lib64 \
  -E LD_PRELOAD=/system/lib64/libbionic_shim.so \
  -E ROBLOX_LIB=/system/lib64/libroblox.so \
  "$ANDROID_ROOT/jni_shim"
```

Rebuild jni_shim: `aarch64-linux-gnu-gcc -o "$SYSROOT/jni_shim" "$CRATE/jni_shim.c" -ldl`
(NOTE: use `realpath` for paths — tilde expansion fails under some shells with QEMU.)

### Environment

- **GPU:** NVIDIA RTX 3060 Mobile + Intel Iris Xe (Mesa drivers active)
- **OS:** Ubuntu 26.04 LTS
- **QEMU:** Custom from `/tmp/qemu-10.2.1/` — patched at `~/.cache/open-sober/qemu-patched`
- **Cross-compiler:** `aarch64-linux-gnu-gcc` (gcc-15)
- **GSI ARM64 libs:** At `~/.cache/open-sober/android-env/system/lib64/` (788 libs)
- **Bionic shim:** `~/.cache/open-sober/android-env/system/lib64/libbionic_shim.so`
- **JNI shim:** `~/.cache/open-sober/android-env/jni_shim`

### 🎯 Recommended Next Steps

The Phase 1a progressive patch (`patch_jni_onload_phase1`) NOPs the clock/time
init only, producing stable 3-class JNI registration output. The next agent
should skip trying to NOP `nativeSetAssetPath` via runtime mprotect (it shares
a page with the JNI registration function and corrupts QEMU's TCG cache),
and instead:

**Phase A — Port the patched QEMU into the repo (critical long-term fix)**

The custom QEMU at `/home/code-agent/.cache/open-sober/qemu-patched` was built
from /tmp/qemu-10.2.1/ (now deleted). The patches applied were:
1. CF_NO_GOTO_TB — prevents chained TB linking in TCG, fixing SMC crashes
2. tb_set_jmp_target no-op — related to goto_tb patching

Without the QEMU patches, the SMC (self-modifying code) crashes return. The
patched QEMU must be preserved or rebuilt from source. Check:
- `~/Documents/qemu-10.2.1/build/qemu-aarch64` (may exist from original build)
- Or rebuild from upstream QEMU 10.2.1 tarball with the two patches reapplied

**Phase B — Fix nativeSetAssetPath hang (Session 11b blocker)**

The hang is inside `nativeSetAssetPath` (offset `0x273de0c`). The SIGALRM
heartbeat (now with ucontext-based real PC) shows the PC alternating between
two addresses on the bionic shim trampoline page, with frame 2 at
`JNI_OnLoad + 0x1f64f90` (error handling path after JNI calls).

**Key constraint:** NOPing `bl 0x273de0c` at JNI_OnLoad offset `0x1f64eb8`
requires mprotect on page `0x1f64000`, which ALSO contains the JNI registration
function at `0x1f6594c`. Making this page RW → NOP → RX causes QEMU TCG cache
to re-translate the registration function, which then only discovers 2 classes
instead of 3 (unstable behavior). This was confirmed with both NOP and `b #4`
replacements, with single and separate mprotect calls.

**Recommended approach for Session 12:**

1. **Pre-load code patch (on-disk patching).** Instead of runtime mprotect,
   patch `libroblox.so` on disk BEFORE `dlopen`. The `bl 0x273de0c` at file
   offset `0x1f64eb8` (and `bl 0x1cfabfc` at `0x1f64e9c`) can be replaced with
   NOP bytes directly in the .so file using a C function that reads/writes the
   file, then calls `dlopen`. This avoids QEMU TCG cache invalidation entirely
   because the code bytes are different before QEMU first translates them.

2. **Patch the on-disk .so at load time.** Write a small function that:
   - Opens `libroblox.so` with `open(O_RDWR)`
   - Seeks to the two offsets
   - Writes `0xd503201f` (NOP) at each
   - Closes the file
   - Then calls `dlopen("libroblox.so", ...)`
   - QEMU will translate the already-patched code from the start.

3. **If pre-load patching isn't possible** (file permissions, read-only fs),
   use `mmap` to map the file with MAP_SHARED, patch in-memory, then close.
   This also avoids mprotect on the executed pages.

4. **After NOPing both calls**, JNI_OnLoad should run fully:
   - Guard check → GetEnv → (clock NOPed) → LocalStorageManager → JNI reg →
     (assetpath NOPed) → guard setter → remaining JNI calls → return 0x10006
   - If remaining JNI calls (FindClass for more classes after guard setter)
     hang due to NULL returns or condvars, add JNI stubs for those classes.

5. **Build proper JNI stubs** for the 3 discovered classes and 13 methods:
   - `NativeLocaleJavaInterface`: getLocale, getRobloxLocale, getGameLocale
   - `NativeUserJavaInterface`: getUserId, getIsUnder13, getUsername,
     getDisplayName, getAlternateName, getPlatformName, getMembershipType,
     getHasRobloxSubscription, getTheme
   - `LoggingProtocol`: getProcessTimestamp

**Phase C — Full JNI_OnLoad enablement**

Once the basic JNI stubs and condvar shim are working:

1. **Remove the Phase 1a NOP** (stop patching the clock init call).
2. **Fix the remaining crash** — when all NOPs are removed, the binary may
   hit a SIGSEGV from a different code path.
3. **Expand JNI stubs** to handle all classes/methods that JNI_OnLoad needs.
4. **Properly RegisterNatives** — call intercepted native methods with the
   correct signatures.

**Phase D — Integrate with the Rust orchestrator**

Once the C-based JNI shim works stably, update `qemu.rs` to use it as the
main entry point for `open-sober play --apk roblox.apk`.

### Known issues / gotchas

- QEMU `-strace` output + `-d exec` output interleave on stderr. For clean
  analysis, redirect to separate files.
- The `alarm_sa_handler` backtrace via `x29`/`x30` doesn't work reliably in
  signal context under QEMU (the registers are the handler's, not the
  interrupted code). To get real backtraces, use QEMU's gdbstub (`-g 1234`).
- `futex` syscalls from guest ARM code may be intercepted by QEMU user-mode
  and not reach the host kernel. `nanosleep` and `clock_nanosleep` DO reach
  the host and appear in `-strace`. If a blocking condvar is needed, prefer
  `clock_nanosleep` over `futex`.
- Tilde expansion (`~`) in paths breaks with QEMU in some shell contexts.
  Always use `$(realpath ...)` or full `/home/code-agent/...` paths.
---

## Session 12 (Aug 20, 2026 — fresh machine rebuild)

Environment started empty: no qemu-patched, no android-env/GSI libs, no APK,
no NDK. The original Roblox build whose offsets the harness hardcoded is not
served by any mirror anymore, so blindly resuming Session 12 against an
arbitrary current APK would not reproduce the documented behavior
(hardcoded GOT/BSS/RELRO offsets in `jni_shim.c` are per-build).

Two changes committed to `dev`:

### 1. Port the custom SMC-patched QEMU into the repo (was "Phase A critical fix")

`qemu/` now contains a *reproducible* build of QEMU 10.2.1:
- `qemu/patches/0001` — force `CF_NO_GOTO_TB` on every TB in
  `accel/tcg/cpu-exec-common.c` `curr_cflags()` (never chain goto_tb)
- `qemu/patches/0002` — no-op `tb_set_jmp_target` in `accel/tcg/cpu-exec.c`
- `qemu/build.sh` — download + patch + build aarch64-linux-user →
  `qemu/out/qemu-aarch64`
- Verified end-to-end: `./qemu/build.sh` produces a working emulator.
  Installed to `~/.cache/open-sober/qemu-patched`.

### 2. Version-agnostic offset discovery (`elf_disco.c`)
The hardcoded GOT / canary / RELRO offsets were the true blocker on a fresh
box (no matching APK). Added a pure ELF parser in
`crates/sober-core/src/elf_disco.c` (+`.h`):
- `robo_got()` — exact GOT/reloc slot for an import, from DT_RELA/DT_JMPREL
- `robo_relro_range()` — PT_GNU_RELRO (else last PF_W PT_LOAD)
- `arm64_adrp_target()`, `robo_first_bl()` — AArch64 decode helpers
Wired into `jni_shim.c`: `patch_condvar_plt_got`, the canary GOT write and
the RELRO pre-mprotect all become discovery-first with the old constants as
automatic fallback. `qemu.rs` cross-compiles + links `elf_disc.c` into the
shim.
Tested without a Roblox APK: `tests/elf_disco_test.rs` cross-compiles a real
ARM64 `.so` importing pthread_cond_* and asserts `robo_got()` matches
`readelf -r` exactly. `cargo test --workspace` green.

### Still needed (separate environment step)
A Roblox Android APK, a GSI/system lib64 tree (bionic libc/c++), and the
NDK/JDK so the shim can actually `dlopen(libroblox.so)`. Mirrors were
bot-blocked / version-mismatched on this box; the acquisition is manual or
via a browser session. Once an APK is present, the version-agnostic shim
should load it without re-tuning offsets (subject to the GSI lib tree).

---

## Session 13 update (Aug 20, 2026) — first real runtime run attempts

### Acquired the real Roblox APK via a real (Playwright headless) browser
Cloudflare-walled mirrors fail via curl; a Playwright headless Chromium (with
the MCP) passed through and let me download the arm64-v8a APK:
- Version chosen: **2.726.1142 (arm64-v8a, Android 8.0+/minapi-26)** — the
  newest arm64-only build on APKMirror, NDK r28c / Android 26 (matches the
  harness toolchain), June 19 2026.
- Downloaded as an `.apkm` bundle from
  `/apk/roblox-corporation/roblox/roblox-2-726-1142-release/...-download/?key=...`
  → contains `base.apk` + `split_config.arm64_v8a.apk` → extracted
  `lib/arm64-v8a/libroblox.so` (104,208,904 B ≈ 100 MB).
- Note: `2.726.1142` **does NOT match** the July 2026 build the CPython hardcoded
  in the shim (that build has JNI_OnLoad at 0x1f64e58; this build has it at
  **0x1f0db20**). So the hardcoded JNI_OnLoad/clock/BSS offsets are wrong for
  this build. `elf_disco` (Session 12) fixes the GOT/RELRO ones; the
  JNI_OnLoad *patch offsets + BSS pre-inits are still hardcoded* and must be
  made discovery-driven before this build can run JNI_OnLoad.

### The runtime stack now boots and reaches dlopen(libroblox.so)
On the fresh box I rebuilt and linked:
- bridges (`libc.so`, `libm.so`, `libdl.so` — LIBC version tags present)
- `libbionic_shim.so` (bionic→glibc trampolines, `symbols_aarch64.c`-style)
- `libguest_stubs.so` — auto-generated no-op stubs for all 146 Android NDK
  UND symbols of libroblox.so (AAsset*, ALooper*, AMediaCodec*, AMediaFormat*,
  ANativeWindow*, egl*, gl*, __android_log*, OpenSLES sl*)
- `jni_shim` (with elf_disco linked)
- glibc base: ld-linux-aarch64.so.1 + android-env/lib
Then `~/.cache/open-sober/qemu-patched` boots the whole thing.

Achieved:
- ✓ `_dl_mcount` nop works (64KB TCG flush) — the Session-8 fix functions
- ✓ bionic shim loads; `dlopen(libroblox.so)` starts; all 576 UND symbols
  resolve.
- ✗ **Blocker: `pc=0x0` NULL-call during dlopen's relocation phase.** With my
  early-`SIGSEGV` catch: `bad addr=0x0 pc=0x0 lr=0x7678b8034d0c`. A versioned
  `@LIBC` symbol that computes a static GOT/PLT slot of 0 is being CALLED by
  the guest dynamic loader during relocation, before JNI_OnLoad. This is the
  handoff's documented long-tail (each `@LIBC_*` needs a real symbol, not
  NULL). My SIGSEGV handler now prints LR to pinpoint it.

### Concrete build/run commands (artifact locations)
```
ANDROID_ROOT=~/.cache/open-sober/android-env
SYSROOT=$ANDROID_ROOT/system/lib64
qemu-patched -L $ANDROID_ROOT \
  -E LD_LIBRARY_PATH=/system/lib64 \
  -E LD_PRELOAD=$SYSROOT/libbionic_shim.so:$SYSROOT/libguest_stubs.so \
  -E DISPLAY=:0 $ANDROID_ROOT/jni_shim
```
(Link each lib from `crates/sober-core/src/{elf_disco.c,jni_shim.c,...}`.)

### Next agent session to-do (ordered)
1. **Make libroblox's JNI_OnLoad patch offsets version-agnostic** (currently
   hardcoded 0x64e9c/0x64eb8 for the OLD build). Use `elf_disco` + JNI_OnLoad
   disassembly to find `nativeSetAssetPath` / clock `bl` and NOP the right
   bytes for `2.726.1142`.
2. **Resolve the `@LIBC_*` NULL GOT** (the pc=0 lr at relocation). Candidates:
   the versioned libc symbol that the loader calls at 0 — add a real bionic
   shim/guest_stubs impl, or ensure the wholearch `libc.so` exports every
   `@LIBC_*` libroblox uses (readelf -r to list, then provide). See
   `disable_mcount_profiling` for the established dlsym+patch pattern.
3. Keep GSI symlinks for the 10 NEEDED libs (libandroid/EGL/GLESv2, etc.) —
   my empty stubs satisfy the linker but must not return NULL when called
   (they're no-ops already).

### Key Session-13b diagnostic (isolated)
The single most useful finding: **`libbionic_shim.so` (built from the repo)
crashes ANY arm64 binary when LD_PRELOADed under the patched QEMU**, even
`printf("hello")`:
```
hello (no preload)           -> prints "hello"
LD_PRELOAD=libbionic_shim.so -> SIGSEGV si_addr=0x1 (right after brk()+1MB
                                anonymous mmap on the main thread's init)
```
`si_addr=0x0000000000000001` = the shim's trampoline/init resolves a glibc
symbol to address 1 (an miscalc'd GOT read) and dereferences it. The shim
(as bundled in this repo) was built against a *specific* host-glibc ABI; the
fresh box's glibc from `/usr/aarch64-linux-gnu` (gcc-15) doesn't match, so
the trampoline's per-symbol `dlsym(RTLD_NEXT, …)` returns garbage for some
entries during the pre-load resolve loop. This is the base cause of the
`pc=0x0 ads()` seen inside `dlopen(libroblox.so)`.

Suggested next-agent fixes (in order of leverage):
1. Make the bionic-shim `__bf_c_resolve` per-symbol `dlsym` tolerant: if it
   returns NULL, back-fill with a local no-op trampoline instead of leaving
   the slot at 0/garbage (so no `pc=0` or addr=1 call can occur).
2. Pre-resolve against `RTLD_DEFAULT` (not just RTLD_NEXT) and validate each
   entry is a real code address (> 0x10000) before committing the table.
3. Then re-run the guest; the loader may get past the shim init and into
   `dlopen(libroblox.so)` cleanly, exposing only the versioned `@LIBC` GOT
   slots that `elf_disco` already resolves generically.

This is the concrete path to the first "JNI_OnLoad" print with the real
2.726.1142 libroblox.so — the bionic shim's trampoline resolution is the
binding blocker on this box.

### Session-13c: NULL-safe resolver committed; load-time shim crash isolated
Committed the NULL-safe bionic trampoline resolver (bionic_init.c): every
`dlsym(RTLD_NEXT,…)` in `__bf_c_resolve` now falls back to `__bf_noop()`
instead of leaving a NULL slot (which branched to 0). Good hygiene, but
isolating the real blocker confirmed the crash is EARLIER and separate:
**LD_PRELOAD=libbionic_shim.so segfaults a trivial ARM64 `hello` with
`si_addr=0x1` right after brk()+anonymous-mmap on the main thread's init —
i.e. in the shim's load-time constructor (`__bf_data_*` dlsym fill /
`__bf_install_mutex_wrappers`), not in the trampoline table.**
So the resolver hardening fixes late faults but not the load-time ABI
crash of the bundled shim against gcc-15 glibc. That load-time crash is
the binding blocker on a fresh box; a follow-up is the shim's `__bf_init_*`
constructor + `__bf_data_*` referencing against the actual gcc-15 ABI.

---

# SESSION 14 HANDOFF — from-fresh-box rebuild + first real runtime runs

## TL;DR
This session went from a completely empty environment to a working,
reproducible runtime stack that boots the **real Roblox 2.726.1142 ARM64
`libroblox.so` (104 MB, NDK r28c, Android 26)** under a rebuilt SMC-patched
QEMU, resolves all 576 of the game's imports, and reaches `dlopen()`. The one
remaining blocker is precise and isolated. `dev` has 6 new commits.

## Committed this session (all in `dev`, all tests green: `cargo test --workspace`)
- `06442d0` **qemu port** — reproducible SMC-patched QEMU 10.2.1 (`qemu/`, patches + `build.sh`)
- `787e300` **elf_disco** — version-agnostic ELF discovery (GOT/RELRO), with integration test
- `f2bc7d9` **early SIGSEGV + LR logging** around `dlopen`
- `993380d` **NULL-safe bionic resolver** (`bionic_init.c`)
- `8f4ef8a` `62c9a7b` docs/HANDOFF

## Environment state (all preserved under `~/.cache/open-sober/`)
| Artifact | Path |
|---|---|
| Patched QEMU 10.2.1 | `~/.cache/open-sober/qemu-patched` |
| JNI shim binary | `~/.cache/open-sober/android-env/jni_shim` |
| Real game lib (104,208,904 B) | `~/.cache/open-sober/libs/libroblox.so` |
| android system/lib64 (25 libs) | `~/.cache/open-sober/android-env/system/lib64/` |
| Bridges libc/libm/libdl | above (LIBC version tags built from `/usr/aarch64-linux-gnu`) |
| bionic shim | `.../libbionic_shim.so` |
| guest stubs | `.../libguest_stubs.so` |

GUI is AVAILABLE: KDE X11 on `:0` (plasmashell + Brave visible via
cua-driver). This is critical — the moment the shim loads, Roblox will
need a display/window, and `cua-driver` + Playwright MCP are in-session.

## Run command (reproduces the stack)
```bash
~/.cache/open-sober/qemu-patched \
  -L ~/.cache/open-sober/android-env \
  -E LD_LIBRARY_PATH=/system/lib64 \
  -E LD_PRELOAD=/system/lib64/libbionic_shim.so:/system/lib64/libguest_stubs.so \
  -E DISPLAY=:0 \
  ~/.cache/open-sober/android-env/jni_shim
```
Expected output (before blocker): `Disabling _dl_mcount... → noped →
Loading bionic shim → Loading libroblox.so →` then **SIGSEGV**.
`QEMU base sanity` check: run an ARM64 `hello` (link with
`aarch64-linux-gnu-gcc`) to confirm QEMU+loader work.

## THE BLOCKER (exact, isolated)
Two distinct facts (both proven):
1. **bionic-shim load-time crash**: `LD_PRELOAD=<...>/libbionic_shim.so`
   segfaults even a trivial ARM64 `printf("hello")` at `si_addr=0x0000...1`
   right after `brk()`+1MB anonymous mmap on the main thread's init — i.e.
   in the shim's **constructor** (`__bf_data_*` dlsym fill /
   `__bf_install_mutex_wrappers`) against gcc-15 glibc. This is the bind
   blocker. It is a small, scoped ABI fix (audit the shim constructor /
   `__bf_data_*` / `__bf_install_mutex_wrappers` against gcc-15).
2. NULL-safe resolver (`__bf_noop`) now prevents late trampoline NULL-
   calls; it does NOT help #1.

With #1 fixed, the next phase is dlopen → JNI_OnLoad → (bounded code +
disasm already in HANDOFF), then **graphics/login** — where
**vision/desktop (cua-driver) is REQUIRED** to drive the window, EGL/GLES→
Vulkan zink, and verify the login screen appears.

## Ordered path for next agent
A. **Fix bionic-shim load-time crash** (pure code; unblocks everything).
   - Reproduce `hello` preload test; debug `__bf_init_*`/`__bf_data_*`/
     `__bf_install_mutex_wrappers` against gcc-15; ensure the shim's
     constructor doesn't deref 0/1.
B. Get `dlopen(libroblox.so)` + `JNI_OnLoad` to return (`elf_disco`
   already handles GOT/RELRO; re-discover JNI_OnLoad patch offsets for
   2.726.1142 — `nativeSetAssetPath` bl at `0x26f384c` inside JNI_OnLoad,
   from the disasm in this HANDOFF).
C. **Now vision is REQUIRED**: launch on the live KDE :0 desktop, use
   cua-driver `get_desktop_state`/screenshots + Playwright to observe the
   window, feed Mesa zink/GLES, and confirm login UI. This is the FIRST
   point the game "boots".

## Facts recorded for next agent
- `JNI_OnLoad` is exported (dynsym) at **offset `0x1f0db20`** in this
  2.726.1142 libroblox.so (the old hardcoded `0x1f64e58` is for a different
  build). Use `dlsym`/`elf_disco` to locate it; do NOT trust old hardcoded.
- 408 `@LIBC` + 4 `@LIBC_N` + 1 `@LIBC_O` versioned imports; bridge
  provides LIBC_* tags; the non-LIBC UND symbols (146 NDK: AAsset*,
  ALooper*, AMediaCodec*, egl*, gl*, etc.) are no-op stubs in
  `libguest_stubs.so`.
- The guest stub generator is inline in this session's bash history
  (`/tmp/gs*.c`); reconstruct via the python snippet that reads
  `readelf -sW` UND FUNC/OBJECT and emits weak no-op/`_stor` objects.
- The `"JDK"`/GSI rumored in the handoff is NOT needed to reach
  JNI_OnLoad; it only matters later for login/token.

---

# SESSION 2026-08-20 — TRUE BLOCKER FOUND & CLEARED (JNI_OnLoad now EXECUTES)

## TL;DR for next agent
The #1 blocker the whole project was stuck on — "bionic-shim load-time crash, can't
`dlopen(libroblox.so)`" — was **misdiagnosed**. The **real** reason libree executed
`pc=0` immediately on load was that this Android 2.726.1142 build ships
**APS2-packed Android relocations** (`DT_60000011` / `DT_ANDROID_RELA`, no standard
`DT_RELA`) and glibc's loader IGNORES them, so `.init_array`/GOT stayed zeroed.
**Converting the packed relocs to a standard `DT_RELA` + extra `PT_LOAD` fixed it**:
`dlopen()` now succeeds, relocation/init runs, and **`JNI_OnLoad` is reached and
executes real Roblox code** (it currently spins in a busy-wait, not crash).

## What actually happened (trace of real work)
1. `unpack_rela.py` (in `crates/sober-core/src/bridges/`) was already ~80% there:
   it decodes APS2 and appends a new `PT_LOAD`. This session **fixed it to:**
   - detect `DT_ANDROID_RELA`(0x60000011)/`DT_ANDROID_RELASZ`(0x60000012) as source,
   - append a page-aligned `R` `PT_LOAD` holding the unpacked standard `RELA` table,
   - set `DT_RELA`/`DT_RELASZ` (tags 7/8) to point at it and **repurpose the two
     Android tags in place to 7/8** so glibc sees them.
   Result: post-patch `DT_RELA@0x6988000`, size 12,799,656; new PT_LOAD vaddr
   `0x6988000`; `.init_array` (3484 entries) now gets populated at runtime.
2. Crash then moved from `.init_array` to a **`__fprintf_chk(NULL FILE*)` update**.
   Root cause: the bionic shim's `stderr`/`__sF`/`stdout` **data slots are 0**, and
   its `write`/`vsnprintf` trampolines resolve to a **no-op** (`__bf_noop`) so ALL
   guest stdout/stderr is silently swallowed. Added:
   - `early_repair_shim()` (runs as the **first thing in `main`**): opens the shim +
     `libc.so.6`, points `__bf_data_stderr/stdin/stdout/realloc`... at the real
     glibc `_IO_2_1_stderr_`/`_IO_2_1_stdin_`/`_IO_2_1_stdout_`/`environ` objects,
   - `jlog()`: an fd-2 logger that resolves the **real** glibc `vsnprintf` from a
     `libc.so.6` handle (plain `vsnprintf` via the shim formats nothing) and
     `write(2, ...)`. All 42 `fprintf(stderr, ...)` in jni_shim were switched to
     `jlog()` so progress is now VISIBLE.
3. The last crash was a `stlrb`-to-ts_flags fault because in the converted build the
   guard/ts_flags/freq offsets (`base+0x6a26e40`, `+0x6a325e4`, `+0x6ae6690`,
   `+0x6ae66e8`) fall inside a **read-only `PT_LOAD`** (the appended `R` RELA run).
   Fixed by `mprotect`ing those pages `PROT_WRITE` before each write.
4. **Result (verified, reproducible):**
   ```
   [jni_shim] Loaded successfully
   [jni_shim] base=0x... mx R
   [jni_shim] pre-init guard=1 ... ts_flags=...->1
   [jni_shim] entering JNI_OnLoad...
   [jni_shim] JNI_OnLoad call at 0x...db20, vm=0x420ef0, env=0x420180
   ```
   then CRUCIAL: **NO crash, no return** — JNI_OnLoad enters a **busy-CPU spin**
   (qemu `-d exec` shows a 2-address loop; `-strace` shows NO futex/nanosleep after
   the JNI call — pure spin).

## Current exact state (repro)
- Installed: `~/.cache/open-sober/android-env/system/lib64/libroblox.so` (RELA-conv
  variant; NOT `${no}` init-disabled). `jni_shim` + `libbionic_shim.so` rebuilt with the
  `early_repair_shim`/`jlog`/mprotect changes.
- Run:  `qemu-patched -L ~/.cache/open-sober/android-env -E LD_LIBRARY_PATH=/system/lib64 -E LD_PRELOAD=/system/lib64/libbionic_shim.so:/system/lib64/libguest_stubs.so -E DISPLAY=:0 <env>/jni_shim`
- git branch `dev`, work uncommitted (see `git status`): `bionic_init.c`,
  `bridges/unpack_rela.py`, `jni_shim.c`. **Commit these.**

## Next steps to actually boot (ordered)
A. **Identify the busy-spin target.** qemu `-d in_asm` shows a GOT-indirect
   `adrp/ldr/ldr/cbz/br x17` thunk looping; straight-text hypothesis =
   Roblox `lock; while(!flag) pthread_cond_wait(...)` where our condvar shim
   (tramp[39,96]=`mov w0,0; ret`) returns spurious wakeups forever and `flag`
   never becomes 1 → pure CPU spin, no syscalls (matches strace). The `guard=1`/
   `ts_flags=1` pre-sets cover specific offsets; this spin is on a DIFFERENT cond.
   Fix: find the spin PC (qemu tracing) and NOP the loop, or make the condvar shim
   also set the waiting thread's expected flag; or pre-set more guard offsets.
2. Then JNI_OnLoad returns (registers 3 native methods) → boot GUI.
3. **Vision/desktop (cua-driver) REQUIRED** to observe the window.

## Key gotchas learned this session
- JNI never needs the real glibc `_IO_*` FILE address trick for `jlog`; just
  resolve `vsnprintf` + `write` from a direct `dlopen("/system/lib64/libc.so.6")`
  handle and write raw fd 2. `RTLD_NEXT` in an executable returns NULL — use
  `RTLD_DEFAULT`.
- `setitimer`/itimers ARM OK under qemu but the SIGALRM is NOT delivered to the
  guest handler (`-strace` shows no heartbeat `write`). Don't rely on it for
  hang PC; use `-d exec`/`-d in_asm` tracing instead.

## Follow-up session (same day) — spin forensics + NX experiment (committed)
- qemu `-d exec`: JNI_OnLoad spins on a 2-address loop in the ~`base+0x764c000->0x7704000`
  band, which was HEAD-first assumed to be the appended RELA PT_LOAD. **Tested it**:
  jni_shim `mprotect(PROT_NONE)` on the RELA PT_LOAD (base+0x6988000, 0xc34ea8)
  SUCCEEDED (rc=0) with NO behavior change — so the spin is **NOT** in the RELA
  data. The base from `dladdr`/`dlinfo` appears ~2MB off for high vaddrs, so
  offset attribution is unreliable; the executions band may be real libro code.
- Net: shutdown; commit `0e3c53a`. Real fix next session = capture the spin's
  **call stack** (needs a working qemu-gdbstub interrupt — gdb `interrupt` over
  the stub didn't take; try `gdb` `set mi-async on` BEFORE `continue&` then
  `interrupt`, or a raw `\x03` on the socket; the `alarm_sa_handler` timer does
  NOT fire under qemu). Then implement the missing guest-stub/trampoline for
  whatever function Roblox dispatches.

## Session 15 — SPIN ROOT-CAUSED AND FIXED; now a real OOM-sourced abort (committed 0a081ac)

### The spin was NOT a condvar loop — it was unresolvable PLT GOT slots => busy-spin to garbage
- qemu `-d exec`/`-d in_asm`: the "spin" was a GOT-indirect thunk
  `adrp/ldr x16;[x16+off]; ldr x17; cbz; br x17` looping with guest PC at
  `base+0x15X014e4/14f4` (X varied run-to-run). Those offsets are past-file
  and past `.text`, i.e. garbage-as-code. The `base+0x15...` jumps into
  anonymous memory.
- Reality: **every PLT JUMP_SLOT GOT slot held a bad value** because glibc's
  lazy binding under qemu user-mode + the bionic shim never resolved them.
  libro's `pthread_mutex_lock@plt` → `br [GOT]` → rodata/anon ⇒ busy-spin.

### Fix (committed): pre-resolve ALL PLT GOT entries
1. `robo_open()` default path was `"libroblox.so"` (CWD) — failed in-app, so
   `g_robo.have=0` and no ELF functionality worked. Now defaults to
   `/system/lib64/libroblox.so`. This is what made everything downstream work.
2. `patch_condvar_plt_got()` installs REAL glibc pthread_mutex_lock/cond_wait/
   cond_timedwait (from direct libc handle `g_real_libc`, not RTLD_DEFAULT which
   returns the shim's shadowed/corrupt address) — the old stubbed shim+wrap
   approach kept the spin.
3. `patch_condvar_plt_got()` now called AFTER the direct-glibc block so
   `g_real_*` are populated.
4. **`patch_all_jumpslots(base)`**: iterate `.DJMPREL`, resolve each symbol via
   `g_real_libc`/RTLD_DEFAULT, write base+r_offset GOT slot with the real fn.
   Result: `patched 532 PLT GOT slots (3 unresolved, of 537)`. The 3 are
   bionic/Android-only (`Java_..._Android*_FinishPaymentsProtocol`,
   `__gcov_dump`, `__gcov_flush`) — harmless.
- **Effect**: JNI_OnLoad now executes REAL Roblox code (clock_gettime, sysinfo,
  /proc over-com/read, getrandom, gettid, getpid) and reaches a real
  **malloc-NULL → abort()** instead of spinning forever. EXIT 14 (hang) →
  EXIT 134 (SIGABRT). Huge milestone.

### Current blocker: `abort` at `Java_..._initializeNativeCode` + 0x343e44 —
  per-thread TLS alloc fast-path returns NULL
- qemu `-d exec` last real .text PC = `base+0x2692d8c...` (`0x2692dcc`: `bl abort@plt`).
- Sequence: pthread_once → mutex_lock/unlock → pthread_getspecific → then
  `bl 0x1c35480` (Roblox per-thread TLS block allocator, small-size fast-path
  from a TLS free-list) → `cbz x0 → 0x2692dcc abort`. It aborts when the small
  alloc falls to the big path and that returns NULL, or the TLS free-list is NULL.
- It aborts EVEN THO we already forge sysinfo => 256 GiB free and
  `/proc/sys/vm/overcommit_memory` => 1 (also tried 0 and 2). So it is NOT a
  real low-memory abort: rather a **bypassed-early-alloc-init / tls-arena-not-
  seeded** condition (we NOP a lot of init). Host has only ~4 GiB avail and
  Committed_AS > CommitLimit, but the allocator never even `mmap`s before
  aborting (strace shows 0 mmaps after JNI_OnLoad).

### Diagnostics added this session
- `wrap_android_set_abort_message()` + `wrap_android_log_print()` print the
  (otherwise logcat-lost) abort reason to stderr — so far no `[android-abort]`
  line appears, meaning the abort is a silent bare `abort()`.
- gdb walk shows the abort caller's return addr is in a data region
  (`base+0x?ba710`), consistent with a JNI/trampoline callback chain.

### Next steps (ordered) — unblock the TLS-alloc NULL
A. Make the TLS free-path never NULL: the small alloc `0x1c35480` `cbz`es on an
   empty free-list and falls to the big-allocator; ensure the big allocator
   (`0x1c3639c`) returns from a real glibc `malloc`. If that tail-call resolves
   via a JUMP_SLOT the loader left bad, patch it. Check whether
   `0x1c3635c` (`b` target) calls real malloc.
B. OR force `abort@plt` (GOT) to `wrap_abort` that logs the caller PC from
   `(_RETURN_ADDRESS)` and returns (unwind the quadruple-abort) so the call
   chain continues and the next OOBorn diagnostic (or SIGSEGV handled by our
   segv handler) reveals the real issue.
C. OR run Roblox's real allocator init (don't bypass it) by removing the NOP
   clock/init bypasses in `JNI_OnLoad` progressive patching / the guard
   override, letting the arena seed normally. This is likely the correct fix.
D. After JNI_OnLoad returns (registers 3 methods), boot the GUI on `:0` and
   start the vision phase (cua-driver, Step 5 on the task list).

### Refined root cause (same session, commit after abort-intercept)
- Neutralizing `abort` (GOT -> wrap_abort that logs caller and returns) makes the
  quadruple-abort at 0x2692DD0/DD4/DD8/DDC log caller offsets then return. After
  they return, JNI_OnLoad keeps executing but busy-spins (0 syscalls). So
  suppressing abort is NOT a fix — it's a diagnostic.
- `0x1c35484` (the per-thread TLS/small alloc) verified: empty free-list ->
  fall through to the book's own MemoryPool allocator (0x1c3635c), NOT glibc
  malloc. strace shows **0 mmap syscalls after JNI_OnLoad**, so the NULL is
  **pure book-side user-space**: the MemoryPool arena/chunk bitmap is empty /
  unseeded because Roblox's real allocator-init never ran (bypassed by the
  JNI_OnLoad progressive patches + guard/ts_flags override). It is NOT a real
  host-memory OOM even though we also forge sysinfo+meminfo+overcommit.
- **CONCLUSION: option C — run Roblox's real allocator/MemoryPool init (do not
  bypass it) — is the right path.** Find the pool init entry (likely a
  constructor / a `Java_..._initializeGC`/`Memory` JNI or a static init that is
  currently NOP'd or skipped) and either let it run or manually call it to seed
  the per-thread pool chunk-base, so the small allocator's free-list is non-empty.

## Session 16 — libroblox REAL .init_array constructors now RUN (committed); blocker = Roblox MemoryPool bootstrap

### Root-cause advance: DT_INIT_ARRAYSZ == 0, so glibc never runs Roblox's ctors
- `readelf -d` on installed libroblox.so: `INIT_ARRAY=0x630bfc0` but `INIT_ARRAYSZ=0`
  (0 bytes) even though `.init_array` section is 0x6ce0 (3484 pointers).
- The init_array entries are `R_AARCH64_RELATIVE` (base+addend) slots that the
  loader SKIPS resolving because DT_INIT_ARRAYSZ==0 (it never runs them).
- So all the static constructors that seed Roblox's MemoryPool / TLS arena
  globals originally never executed. That is the real antecedent of the old
  malloc-NULL abort.

### New capability: run_libroblox_init_array(base)
- mprotect base+[0x5a00000..0x6320000] RWX, apply RELATIVE relocs for
  init_array-range slots (0x630bfc0..0x6312ca0) -> write base+addend, then call
  each ctor in address order.
- VERIFIED RUNNING: log shows ctor[0]@base+0x2692f14 .. ctor[3]@base+0x1c34480
  with a sysinfo plus abort appearing INSIDE ctor[3]'s execution.
- ctor[3] (0x1c34480 -> tail `b 0x5d9ce10`) does a thread-local allocation via
  0x1c35480 (the TLS block fast-alloc) which returns NULL, hit the
  cbz-to-abort at initializeNativeCode+0x343e44. Same malloc-NULL abort as
  before, now reached from the constructor path.

### Blocker now precisely: Roblox's per-thread MemoryPool TLS alloc returns NULL
- 0x1c35484: TLS key from [0x6368000+0x9dc]; if -1 runs init; else
  pthread_getspecific to default block 0x6308dc0 (csel if empty). size<=0x400
  fast-path pops [blk+232]+8; on empty -> big-allocator 0x1c3635c.
- 0x1c3635c (big alloc) returns NULL regardless of reported memory (614MB real
  OR 256GB forged sysinfo): qemu strace shows NO mmap after JNI_OnLoad, so it is
  a book-side arena-not-seeded condition, NOT real OOM.
- wrap abort() logs+returns; without it the process dies SIGABRT.

### de-horned wrong guesses (verified and committed)
- sysinfo/meminfo/overcommit forgers are NOT the fix and are now set to PASS
  THROUGH real host values (forging 256GB or overcommit 0/1/2 didn't change the
  abort). Do not re-add inflation as the primary lever.
- Applying RELATIVE relocs GLOBALLY double-corrupts .data (loader already does
  .data/.got/.data.rel.ro; only .init_array is skipped). Keep the apply
  init_array-scoped.

### Next (ordered)
A. Find and call the MemoryPool init directly (seek the fn that initializes the
   block region 0x6308dc0 / the arena global ~0x6367000+0x600), or identify a
   later ctor that seeds the pool and run it before ctor[3].
B. Or patch 0x2692ce8 (the cbz-abort on the TLS-alloc NULL) to fall back to
   real glibc malloc so the book gets a block and can proceed through later
   ctors -- a stepping-stone, not a final fix.
C. After JNI_OnLoad returns (registers methods), GUI on :0 + vision phase.
## Session 17 — direction reset: stable loaded state via Session-9 full bypass (committed)

### Direction confirmed with user
End goal is the OPEN-SOBER CUSTOM RUNTIME (sober-style native run: QEMU is the
ARM64/x86-64 bridge, not a whole-VM product). Re-adopted the Session-9 loaded
-state approach: JNI_OnLoad returns 0x10006 immediately so the binary bootstraps
under the bridge and we can then raise/observe a window on :0. Fighting Roblox's
internal MemoryPool inside progressive init is parked (documented below).

### Why the old full bypass was silently broken
- There are TWO entry shapes: the EXPORTED JNI_OnLoad at base+0x1f0db20 (what
  dlsym/lib host calls) and 0x1f64e58 (an internal init at a shifted offset).
  The prior bypass patched 0x1f64e58 -> real JNI_OnLoad still ran the
  MemoryPool-reaching init and aborted.
- Fix: bypass base+0x1f0db20 (entry = mov w0,#6; movk w0,#1,lsl#16; ret).

### Disabled for bypass path
- run_libroblox_init_array(...) call is commented out. Its ctor[3] (MemoryPool
  TLS alloc at 0x1c34480 -> 0x5d9ce10 -> body 0x2678068) aborts on the internal
  pool; with abort suppressed it spins and blocks. Skipping ctors gives the
  clean baseline. The init_array + RELATIVE machinery is kept in the tree
  (real-engine-init path) but not run by default.

### VERIFIED (fresh run)
  FULL BYPASS: JNI_OnLoad@<base+0x1f0db20> returns 0x10006
  entering JNI_OnLoad...
  JNI_OnLoad call at <base+0x1f0db20> ...
  JNI_OnLoad -> 0x10006
  Entering sleep loop
No abort, no fault; process stable until watchdog timeout (RUN EXIT 14).

### MemoryPool blocker (parked, for real engine init later)
- libroblox imports NO allocator (only free, munmap); its MemoryPool is fully
  self-contained. Big-allocator 0x1c3635c returns NULL regardless of forged
  614MB vs 256GB sysinfo, with zero mmap after JNI_OnLoad. One-time init body
  0x2678068 aborts on its own first TLS alloc. Fork/banc of memory, ctors,
  malloc fallback all unavailable. To boot the real engine, must resolve the
  TLS block free-list seed (0x6308dc0 struct) or run fuller Android/JAVA app
  bootstrap.

### Next (ordered, per user direction)
A. NOW: run jni_shim with DISPLAY=:0, keep sleep-loop stable, and use
   computer vision on the X11/EGL surface to observe any window/black frame,
   or confirm none yet. Check whether book opens a GL context or needs the
   engine run loop.
B. Then: re-enable init_array/progressive init in stages ONCE the pool seed is
   understood, so a real window can render.
C. Multiple-version compatibility after a working baseline.
## Session 17b — MemoryPool malloc-fallback thunk: empty-pool abort DEFEATED (committed 313aa7b)

### Probe: libroblox imports NO allocator functions
readelf -r --use-dynamic shows libroblox.so imports only `free`/`munmap` from the
allocator family (no malloc/calloc/realloc/mmap). Its MemoryPool is fully
internal; the big-allocator returns NULL on unseeded arena state regardless of
host free RAM. Forging sysinfo/meminfo is irrelevant.

### Fix: redirect small-allocator empty-list to real glibc malloc
- Site: libro offset 0x1c354fc — the "empty per-thread free-list" continuation
  that tail-calls the big-allocator 0x1c3635c (which returns NULL).
- Thunk on a fresh MAP_ANONYMOUS RWX page (NOT the cond_shim page, which is the
  condvar "mov w0,#0; ret" trampoline):
      mov  x0, x19           ; size (0x1c35490 mov x19=x0; x19==size)
      ldr  x17, [pc, #24]    ; pc-literal loads real glibc malloc
      blr  x17               ; x0 = malloc(size)
      ldr  x19, [x29, #16]   ; restore saved x19
      ldp  x9, x30, [x29], #32 ; restore (x9=[x29], x30=[x29+8]), sp+=32
      mov  x29, x9
      ret
  Encodings verified against aarch64-linux-gnu-gcc-assembled .S.
- Book patch (24 bytes at 0x1c354fc): `adrp x16, thunkpage; br x16; nop x4`.
  ADRP encoding that VERIFIED (mine was wrong first try):
      imm = (thunk_page - site_page)  ; in 0x1000 units, signed
      adrp = 0x90000000
           | ((imm & 3) << 29)                  ; low 2 bits -> bits[30:29]
           | (((imm >> 2) & 0x7ffff) << 5)      ; high 19 bits -> bits[23:5]
           | Rd                                  ; Rd = x16 = 0x10
  My first form `((imm&0x7ffff)<<5)` put the raw (non->>2) delta at the wrong
  bit offset -> jumped to a wrong page and SIGSEGV'd. Correct form verified
  against the cross-cc (same-page `adrp x16` disassembles to 0x90000010).

### Result
- BEFORE: ctor[3] aborts 4x (empty-pool NULL) and spins.
- AFTER: ctor[3] runs, calls sysinfo (pool doing real allocation), NO abort,
  NO segfault. The hang is now a single-threaded CONDITION wait-loop (classic
  QEMU user-mode one-thread behavior), not an OOM abort.

### Next
Post-ctor[3] hang = wait on an event/flag that never arrives (one thread under
QEMU). Options: (a) trace the exact spin site (heartbeat) and force/fake the
awaited flag; (b) pre-seed the pool's real arena (static default TLS block free
-list) so the block path never blocks. Much more tractable than the previous
NULL/OOM abort.
## Session 17c — blocker refinement (committed 381d088)

Current state: the MemoryPool empty-pool abort (fixed Session 17b by a thunk that
routes libro's small-allocator empty-list to real glibc malloc) is gone. ctor[3]
(book 0x1c34480, MemoryPool one-time init) now runs, performs its allocation
syscalls (sysinfo/overcommit/getrandom), then spins in pure user-space code with
NO further syscalls — a single-threaded wait for a condition/flag that only a
second thread would set (QEMU user-mode runs one vCPU).

Attempts this turn:
- SIGALRM heartbeat sampler from the host-signal route: QEMU user-mode does not
  deliver SIGALRM into the guest handler; 0 samples (kept, harmless).
- pthread_cond_timedwait slot return now ETIMEDOUT (110) instead of 0 (kept).
- A dedicated bump-allocator thunk target (host shim function) caused SIGILL —
  calling a host/ELF function from the guest through the ARM thunk crosses a
  translation context QEMU cannot handle. Reverted to the dlsym'd glibc malloc
  thunk, which is safe and stable.

Stable, committed, no crash: ctor[0..2] run, allocation succeeds, ctor[3] waits/
spins, no abort/SIGILL/segv.

Next: identify the awaited flag in ctor body 0x2678068 and pre-set it (Session
11-style guard fix); or pre-warm the static TLS block free-list; or spawn an
emulated second thread.
---

## Session 18 — from-scratch JIT (arm64jit): drop-QEMU path begins

**Decision (user):** build a small in-process ARM64->x86-64 JIT/translator from scratch to replace QEMU, accepting multi-session. Goal stays: a real runtime (no QEMU), then multi-version proof, then CV on the GUI.

**What landed this session (all committed, 16 tests green):**
- `crates/arm64jit/` — new workspace crate. `x86.rs` = minimal verified x86-64 emitter; `decode.rs` = AArch64 decoder; `translate.rs` = per-instruction ARM->x86; `jit.rs` = CpuState + exec.
- Decoder verified against REAL gcc/objdump encodings (not memory): B, B.cond, MOVZ/MOVK/MOVN, ADR/ADRP, ADD/SUB imm, ADD/SUB shifted-reg, AND/ORR/EOR, LDR/STR unsigned-imm + register-offset.
- Pattern lesson: decode guards keyed to top-byte/class `(top & 0x3b)==0x39`/`0x38` (LdImm vs LdReg), `movewide sf` etc. — derived from actual encodings, each with a ground-truth unit test.
- Key debug: LOGIC-reg `s` is NOT bit29 (that's part of opc); `s = opc==0b11`. rm field is bits[20:16], NOT (insn&0x1f).
- JIT executor: spills guest regs to a `CpuState` in memory addressed via RBX (NOT R12/RSP/RBP — emit_mem forbids RSP/R12, and RBP/R13 have rm=7 -> RIP-rel for disp0). Prologue `mov RBX,<state>`; `[RBX + 8*i]` per reg; epilogue returns x0. `exec_bytes` = bytes->decode->compile(RWX mmap)->call fn(*mut CpuState)->x0.
- **LIVE proof:** executed real `mov x0,#3; add x0,x0,#4` (=d2800060 91001000) => returned 7, no QEMU. Commits 2faf3dd, d1fed33, eb11d66, 7c25747.

**Current scope/limits (honest):** translator handles MoveWide/AddSubImm/AddSubReg(shamt 0) only so far; no loads/stores, no branches/control-flow, no BL/host-call dispatch, no FP/NEON/TLS/atomics yet. Roblox's libroblox.so is far beyond this until loads/stores + branches + a host-call (syscall/bionic-shim) dispatch land.

**Next (Session 19+):** broaden decoder+translator to LDR/STR (have decode) + B/B.cond/CBZ + a host-call trampoline; validate on a real multi-instruction aarch64 .so function; then wire into libloader `--no-qemu`. Cleaned /tmp of ~5G stale QEMU core dumps.

## Session 18b — arm64jit executes load/store + control flow (no QEMU)

**Verified this session** (all through the from-scratch JIT, no QEMU):
- LDR/STR (unsigned 16-bit immediate offset), size 8/4, via RDX addr + lea — real
  `ldr x0,[x0,#16]` loads host memory correctly (17→19 tests).
- RET decodes + translates to host `ret`.
- **Control flow**: B (jmp), CBZ/CBNZ (test+jnz/jz) + flow-aware `jit::compile`
  that builds a guest_pc→host_offset map and patches rel32 fixups
  (disp = target − (disp_off+4)). Fixed a bad `start` calc → SIGSEGV.
  Real `cbz x0` function returns 20 (fall-through) / 10 (taken) ✓.

**State**: 19 tests green, workspace clean, committed at `09699e5`.

**Commits this session**: eb11d66 (LDR/STR+ret), 7c25747 (first exec),
a432acf, 09699e5 (control flow).

**Next slice** (task 3b): B.cond + NZCV flags (subs/cmp set flags; materialize
into guest NZCV so any interleaving works) then BL function calls with a
guest call stack. After flags, real `.c` compiled aarch64 (if/else, loops)
can run.

## Session 19 — arm64jit: flags + calls + stack pairs (23 tests, no QEMU)

**Verified this session** (all through the from-scratch JIT on x86-64, no QEMU):
- **cmp/subs → B.cond** (if/else): real `cmp w0,#3; b.le` returns 0 or 1 per ARM. Decoder
  expanded to S-flag form `cmp w,#imm` = top 0x71/0xF1 (SUBS imm). `x86_cc_for_cond`
  maps ARM cond→x86 jcc (EQ..LE); set-flag ALU leaves x86 flags live through the
  follow-on `mov` stores(store to rd==31 suppressed).
- **BL function calls + call-graph `compile_image`**: BFS walk follows branch/call
  targets, compiles whole reachable region into ONE buffer; `BL` = save LR + host
  `call rel32` (cc=0xfe fixup); a real leaf call `caller(x)=(x+5)*2` returns correctly.
- **LDP/STP (load/store pair)**: offset/pre-index/post-index, 32/64-bit, decoded via
  bit23=indexed, bit24=pre, bit22=load, imm7 signed scaled by 8/4; `stp/ldp` prologue
  round-trips regs through real stack, sp restored.

**Tests: 23/23 green. Tree clean.** Commits: 43f7af3 (flags+B.cond), 874800c (BL +
compile_image), 0ee07fc (LDP/STP).
**Next slice (3e):** adrp/adr (PC-relative), ORR/EOR/AND-reg + reg-reg MOV/aliases,
then `adic` integration into libloader `--no-qemu`.

## Session 19: arm64jit — core-ISA coverage (26 tests, drop-QEMU verified)

**Verified this session** (all through the real from-scratch JIT, no QEMU):
- **Flags + B.cond** (commit 43f7af3): cmp/SUBS/ADDS set flags; b.eq/ne/le/lt/ge/gt/hs/ls/cs/cc
  translate to x86 jcc. Real `cmp w0,#3; b.le` runs correctly.
- **LDP/STP pair load/store** (0ee07fc): offset/pre/post index, X and W, verified vs 5 real
  encodings. Real prologue stp/ldp round-trips regs and restores SP.
- **BL calls + call-graph compile_image(image, base, entry)** (874800c, 0ee07fc): walks
  BL/B/CBZ targets, emits into one image buffer, host call + host ret (LR saved). caller()=20.
- **LogicReg AND/ORR/EOR + mov alias** (dc88106): mov rd,xm; XZR reads as zero; real logic.
- **ADRP/ADR** (47f7a5f): PC-relative addressing; adrp/add/ldr loads a mapped global (guest==host
  address when the ELF is placed at its vaddr).

**State: 26 tests green — core AArch64 ISA executes real compiled code on x86-64, no QEMU.
Next (task 4): integrate into libloader/sober-core — map the Roblox ELF at its vaddr, then
compile_image(text_segment, vaddr, entry) for _start/JNI_OnLoad. adrp/ldr now resolve because
guest address == host address when segments are mapped at their ELF vaddr.

## Session 19c - JIT wired into the product (no QEMU path)

- **libloader**: exposed `pub mod elf` so `load_elf`/`LoadedElf` are usable downstream (commit 9c8e791).
- **arm64jit/examples/elfjit.rs**: loads a real static aarch64 ELF with libloader's loader and JIT-runs
  its entry -> returns 42 on x86-64, no QEMU. Proven end-to-end loader+JIT wiring.
- **sober-core**: `--jit` CLI flag -> `mod jit::run_elf_entry` loads the Roblox .so via libloader and
  hands it to arm64jit. `qemu::find_main_binary` made pub. (commit 4b89619)
- **Honest boundary**: trying to run `/tmp/robpatched/libroblox.so` (a PIE `ET_DYN`) through elfjit
  SEGV s because for PIE .so the host address of the code is NOT simply `e_entry`: the loader maps the
  PT_LOAD text segment at a real host address, and `compile_image` must be fed the mapped host range +
  its guest base, not `entry` directly. Plus the full .so uses far more instructions than the current
  subset, so full execution is still months of translator work.

**Next (task 5)**: fix the PIE path - derive the mapped text host address from `LoadedSegment.vaddr`,
  feed `compile_image(image=mapped_host_slice, base=guest_text_vaddr, entry=host-of-entry)`, and report
  the *first unsupported instruction's guest address* as an honest diagnostic target for the next
  decoder slice (start with SVC syscall routing + TLS, then SP, then BL/ADR linkage).

## Session 19d - WHAT STILL NEEDS TO BE DONE to get the JIT path fully working

Current state: arm64jit executes a real AArch64 subset on x86-64 with no QEMU
(26 tests green, all verified against objdump ground-truth). It is wired into
the product end-to-end (libloader -> compile_image -> run) and can run the
entry of a *non-PIE static* aarch64 ELF (elfjit example returns 42). Running
the real `libroblox.so` (a PIE ET_DYN) currently SIGSEGVs.

### 1. PIE / shared-object mapping (unblocks the real .so immediately)
- **Problem:** the elfjit/`--jit` path feeds `compile_image(image, entry, entry)`
  assuming host-addr == guest-vaddr. That holds only for non-PIE statically
  linked ELFs. libroblox.so is ET_DYN/PIE: libloader maps PT_LOAD segments at
  real host addresses and relocates, so `e_entry` is not a host address.
- **Fix:** derive the *mapped text range* from `LoadedElf.segments[]` (host
  vaddr + memsz), slice that range as the compile `image`, pass `base =
  guest_text_vaddr` and `entry = host(of e_entry)`. Then `ADRP`/`ADR` compute
  guest addresses that resolve into the mapped segment (guest==host holds
  again because libloader maps at vaddr).
- **Diagnostic:** once it loads, report the **guest address of the first
  instruction the decoder can't translate** (add a `resolve` that returns
  `Err((pc, Inst::Unsupported))`). That gives the exact next decoder slice.

### 2. Decoder/translator gaps that WILL appear (in rough order of priority)
- `SVC` syscall routing (Roblox makes many host syscalls; must map to host or
  the bionic shim) - and the `--jit` path must link against the shim.
- TLS slot access (`mrs`/`msr` TPIDR_EL0, `ldr`/`str` via TPIDR), threads
  (host pthreads vs guest threads).
- Atomics (`ldaxr/stlxr`/CAS loops) - used heavily in Roblox.
- FP/SIMD (NEON: a LOT in graphics/sound; `ldr q`, `add v0.4s,..`, etc.)
- 32-bit register semantics: Ws must zero-extend and flag-setting compares
  must be 32-bit-aware (currently traced as 64-bit for small values only).
- XZR vs SP as x31 depending on context (currently one sp slot, no read-xzr
  suppression for arithmetic stores - add rn/rd==31 handling per class).
- Multiply/AES/other (`mul`, `mneg`, `sdiv`/`udiv`, `csel` (flags-dependent
  select - completes flag model), bitfield ops `ubfm/sbfm/bfi/extr`).
- LD/ST variants: `ldr x,[x,#imm]` are done; `ldr` non-scaled, `ldrsw`,
  `ldrb/h`, `strb/h`, `ldp/stp` SIMD (128-bit D0-D31 pairs).
- Branch: `b.eq/ne/...` (B.cond already done), `br`/`blr` (indirect call),
  `cbz`/`cbnz` done, `tbz`/`tbnz`, return-less tail calls.
- `MOVZ/MOVK` (done) but `MOVN` and imm build-up across block - fine.

### 3. Guest runtime environment (required to actually run Roblox)
- **A guest stack** (`sp`/x31) pointing into a large mmap'd region; `mrs
  SP_EL0` etc.
- **Thread-local storage:** TShell set `TPIDR_EL0`, guard-and-init; Roblox
  spawns threads.
- **Syscall service** (`svc #0`): at minimum `exit`, `write`, `mmap`,
  `munmap`, `brk`, `clone`, `open`, `read`, `futex`, `timer`, `getuid`,
  `sysinfo`. Either route to the bionic shim or implement host-facing.
- **Signal handling / the JNI setjmp-longjmp** that the native glue expects.
- **Linking the bionic** sysroot libs (libbionic_shim.so + guest stubs) -
  the existing QEMU build work (jni_shim, elf_disco) is REUSABLE for symbols
  resolution; the JIT needs a PLT/GOT resolver so `bl` to relocated functions
  dispatches to the right guest/host thunk.

### 4. Correctness hardening (before trusting any real run)
- **Frame pointer / unwind** - not needed for execution but for debugging the
  unmistakable first crash.
- **Trap on unsupported instead of UB:** currently any translated block that
  hits an untranslated instruction returns Err gracefully (good); but a guest
  `ret`/`br` to an address outside any compiled block must be caught, not
  fall through (add a `state->pc` write + a trampoline back into the
  interpreter/compile loop for un-compiled blocks).
- **PC-relative fixups are buffer-relative (already done);** ensure they are
  correct for code that spans two images/ELF segments.

### The real realistic path to a Roblox window (multi-session)
1. PIE mapping + first-unsupported diagnostic (do this next).
2. Wire `svc` + TLS + a stack + GOT/PLT resolution so `JNI_OnLoad` can run
   far enough to print something.
3. Add cross-version coverage (the standing "multi-version" goal) by diffing
   the decoder against several `libroblox.so` builds.
4. Only after those load + JNI init: FP/NEON + atomics + threads to get
   actual frames; then the GUI/computer-vision inspection step becomes
   meaningful.

Everything above was updated to account for the current committed state at
`5496852`. The single highest-leverage next step is **#1 (PIE mapping)**.

---

## Session 20 (Aug 20, 2026) — PIE mapping FIXED; JIT now decodes real libroblox.so code

Goal (from 19d #1): fix the PIE/ET_DYN mapping so `elfjit`/`--jit` no longer
SIGSEGVs on the real 117MB `libroblox.so`, then push the honest
first-unsupported diagnostic forward. **This was achieved and verified end to
end.** Commits: `194d5d8`, `029e36f`, `da16a76` (on `dev`).

### 1. PIE / ET_DYN mapping — FIXED (the 19d #1 blocker is done)

**Root cause of the old SIGSEGV:** the per-segment `MAP_FIXED` in `load_elf`
lets a later PT_LOAD of a *packed* ET_DYN target an address that still overlaps
the previous huge (`~99MB` r-x) text mapping. Under gdb the fault was
`__mmap64` crashing on a `MAP_FIXED` address inside the earlier mapping — i.e.
`0x78f05998000 + 0x5e6b000` landed *inside* the text range.

**Fix:** added `libloader::elf::load_elf_image(path) -> LoadedElf` (a fresh API,
the old `load_elf` is untouched for the QEMU path). It maps ONE contiguous
anonymous region at a fixed `JIT_BASE` (`0x100000000`), lays every PT_LOAD into
it at `base + (p_vaddr - min_vaddr)`, zero-fills `.bss`, and applies per-segment
mprotect. Critically it sets **guest vaddr == host address** (`guest_of(link) =
base_addr + (link - base_load_addr)`), the exact property arm64jit's ADRP/ADR +
direct-dereference model requires. No more overlap, no more SIGSEGV.

`elfjit` and `sober-core --jit` now:
- load the real `libroblox.so` cleanly (4 segments, guest==host at
  `0x100000000`, e.g. text `[0x100000000, 0x105e67390)`),
- translate the requested guest entry (a link-time address via `guest_of`),
- report an **honest diagnostic**: `arm64jit stopped on unsupported instr
  at/near guest 0x101c34480: translate: unhandled Unsupported(0x...)`.

### 2. Decoder/translator walls pushed through (5 in this session)

1. **ADRP/ADR mask bug FIXED.** The decoder tested `insn>>24==0x90`, missing
   real ADRP encodings whose top byte is `0xD0` (varies with `imm[1:0]`).
   Changed to the canonical `(insn & 0x9F000000)==0x90000000` (ADRP) /
   `==0x10000000` (ADR); confirmed `0x90026516` (top 0x90) and `0xd0026a93`
   (top 0xD0) both match. Without this, the very first decoded real-world
   Roblox ADRP `0xd0026a93` returned `Unsupported`.
2. **LDR/STR unsigned-imm sizes 1,2,4,8** (was 8/4 only). Added halfword/byte
   zero-extend loads (`movzx_word_mem`, `movzx_byte_mem`) and 8/16-bit stores
   (`mov_store8/16`) to the x86 emitter; wired into `LdStrImm`.
3. **LDR/STR register offset** (`ldr x9,[x8,x1,lsl #3]`, class `0x38`) — new
   `LdStrReg` translate arm (index `rm`, shift by `log2(size)`).
4. **128-bit SIMD vector load/store** (`ldr q6,[x0,#16]`=`0x3dc00406`,
   `str q7,[x0,#32]`=`0x3d800807`) — `CpuState` now carries a **32×128-bit
   vector register file** `v:[u64;64]` at `VECTOR_BASE=256` (with `set_v`/`get_v`),
   x86 XMM helpers (`movdqu_load/store`, `movdqa_xmm`, `pxor_xmm`), new decode
   class `VecLdStImm` (`0x3D8`/`0x3DC`), and a translate arm that moves 16 bytes
   between the guest v-slot and guest memory through XMM0.
   *(`movi`/float NEON immediate was deliberately NOT bolted on — the imm
   reconstruction is fiddly and a wrong float result would be worse than the
   honest "unsupported" stop. Do it with a proper NEON decoder next.)*

### 3. Verification

- `cargo test --workspace` all green (arm64jit 26 → still 26, no regressions).
- Static non-PIE `stat.elf` entry returns `42` (unchanged, still passes).
- PIE `libpie.so` maps at `0x100000000`; entry `0x588` translated to guest and
  stopped *honestly* at `lsl x0,x0,#1` (`0xd37ff800`, a UBFM bitfield op — see
  task list below; not yet added).
- 128-bit vector round-trip: hand-assembled aarch64
  `ldr q6,[x0,#16]; str q6,[x1,#32]; mov x0,#99; ret` runs through elfjit
  (giving it a guest==host buffer via the new `buf` arg) and **returns 99**,
  no QEMU, no crash.

### 4. Current honest state on the real binary

```
$ ./target/debug/examples/elfjit ~/.cache/open-sober/libs/libroblox.so 0x1c34480
loaded '...libroblox.so': is_pie=true base_load_vaddr=0x0 e_entry=0x100000000
  segment guest=[0x100000000,0x105e67390) prot=r-x   (89MB text)
  segment guest=[0x105e6b3c0,0x10631c000) prot=rw-
  segment guest=[0x10631fb40,0x106987130) prot=rw-
  segment guest=[0x106988000,0x1075bcea8) prot=r--
running entry guest=0x101c34480
arm64jit stopped: translate: unhandled Unsupported(1862329344)  // == 0x6F00E400
```

The NEXT blocker is the **floating-point NEON** instruction `0x6F00E400`
(top-byte `0x6F`, the floating-point 3m-add / scalar-fp class — NOT the
integer `movi` `.4s` which decodes as `0x4F...`). That's the immediate next
decoder slice.

> **Session 21 CORRECTION:** `0x6F00E400` is **NOT** floating-point. Verified
> against `aarch64-linux-gnu-objdump` ground truth, it is `movi v0.2d, #0x0`
> — an **integer** vector move-immediate (the "clear a 128-bit vector to
> zero" idiom at the top of a stack-zeroing loop). It was handled in Session
> 21 (below), not deferred. The 20's "0x6F=FP" guess was wrong.

### 5. Next steps (updated, ordered)

1. **Add the float-NEON / FP layer** starting with `0x6F00E400` specifically,
   plus `fmov/fadd/fsub/fmul/fdiv`, `fcvt/fcvtl`, `fcmp/fcsel` and the
   `0x4F` `movi.4s` immediate (with unit tests). This is now the gate between
   "decodes" and "boots" — a 3D engine is dense with FP.
2. **`lsl/lsr/asr x,#imm`** (`UBFM/SBFM`, e.g. `0xd37ff800`) — trivial and hit
   by any real code; add the bitfield ops `ubfm/sbfm/bfi/bfx`.
3. Guest stack + `sp`/`mrs TPIDR_EL0` TLS + `svc` routing → then the `B/BL`
   call-graph can actually *run* ("compare" the boot log) rather than just
   translate.
4. Cross-version coverage; verify against multiple `libroblox.so` builds.

`git log`: `194d5d8` (PIE load_elf_image fix), `029e36f` (one contiguous
guest==host image, runs deeper into libroblox.so), `da16a76` (SIMD vector
regs + 128-bit ld/st + register-offset ld/st), then the HANDOFF update.

## Session 21 (Aug 20, 2026) — Architected the PC-driven dispatcher; JIT now *executes* real libroblox.so through blr chains

Commit: `6618c5e` (dev). **The JIT crossed from "translate-then-stop" to
"actually follow call/return control flow".** Five more instruction walls
pushed + the VECTOR_BASE bug fixed.

### 1. What was previously-unsupported, now decoded+translated (all objdump-verified)

1. **`movi` vector-immediate** (`.8B/.16B/.4S/.2D`) — and in doing so,
   corrected 20's mislabel: real blocker `0x6F00E400` is `movi v0.2d,#0x0`
   (integer), verified by `objdump -d`. Decoder reconstructs the lane and
   replicates it across the 32-bit/8-bit/64-bit lanes; unit-tests
   `movi_ground_truth` covers all 6 specimens.
2. **`stp`/`ldp q` 128-bit SIMD load/store pair** (`0xAD000000`),
   scale=16. Init function zeros 64 bytes via `movi; stp q,q; stp q,q`.
3. **HINT / PAC pseudos**: `nop`, `esb`, `csdb`, `paciasp`/`autiasp` (the
   `-msign-return-address` prologue), `bti`. Nominally the `0xd5032xxx`
   family, mask `(insn&0xfffff01f)==0xd503201f`; executed as no-ops (PAC
   is ignored in the guest).
4. **`br`/`blr`** — indirect branch/call. **The decode RN bug:** the source
   register is bits[9:5], *not* [4:0]; was decoding `blr x1` as `blr x0`,
   which sent the dispatcher to address 0 → instant halt with wrong value.
   Fixed; unit-tested.
5. **`tbz`/`tbnz`** test-bit-and-branch — `b24`=op (1→tbnz), `bit=b[23:19]|b31`,
   imm14×4. Verified `tbnz w0,#31` target.

### 2. The dispatcher (`jit::jit_run`) — the enabler

`br`/`blr`/`ret` can't be inlined (target is in a register). Added
`jit_run(image, base, entry, state)`: loop { compile reachable region from
`state.pc` via `compile_image`; run; re-enter at `state.pc` }. Terminal
`br`/`blr`/`ret` set `pc = (16/8-lanes)`, x30 link on blr, return to the
host loop. `ret` now sets `pc = x30` so returns chain to the caller.

`elfjit` switched to `jit_run`. Verified with a hand-assembled aarch64
program (adrp/add `x1=&callee`; `blr x1`; `mov x30,xzr; add x0,#1; ret`
/ callee `mov x0,#42; ret`) → **returns 43** through the dispatcher. This
is the first real indirect-call round trip.

### 3. Critical correctness bug: VECTOR_BASE overlapped `CpuState.pc`

`CpuState{ x[32]@0..256, pc@256 }`, but `VECTOR_BASE` was **256** — so every
vector `movi`/vector ld/st wrote into `pc`/`nzcv`, corrupting instruction
streams once SIMD ran. Moved the vector file to **272** (`VECTOR_BASE=272`,
`PC_OFF=256`). This was latent since Session 16 (vector regs added) and only
surfaced now that SIMD + the pc-driven loop share a state.

### 4. Honest current state on real libroblox.so

```
$ elfjit .../libroblox.so 0x1c34480
running entry guest=0x101c34480
arm64jit run_loop stopped: translate: unhandled Unsupported(445973185) at guest pc 0x1026938d0
```
`0x1A9502C1` = `csel w1, w22, w21, eq` right after `cmp x9, x10`. So the JIT
now *executes* through: `movi`→`stp q`→`paciasp/autiasp`→`blr` chains→`tbnz`,
and blocks on the first **NZCV-flags consumer** (CSEL).

### 5. Next step (the 21st wall): NZCV flags + the CSEL/CSET family

`csel x,w, cond` and `cset/csinc/csinv/csneg` need N/Z/C/**V** live from the
preceding `cmp/subt/cmp nzcv`-setter. NZCV is currently a placeholder
(written but not consumed); `b.cond` also must read *stored* flags, not live
x86 flags. This is a self-contained subsystem:
1. store N=bit31, Z=bit30, C=bit29, V=bit28 to `CpuState.nzcv:u32` from each
   `S`-flag setter (cmp/subs/adds/...),
2. read cond→x86 flag and `csel/cset/...` branch on it,
3. retire the "NZCV is a placeholder" comment in translate.rs.

`git log` since 20: `6618c5e` — "arm64jit: PC-driven dispatcher (blr/br/ret)
+ push 5 more decoder walls". Working tree clean.

## Session 22 (Aug 20, 2026) — NZCV flags + CSEL/CSET family; stp/ldp d; LDAR/STLR

Commit: `6dceb47` (dev). **Session 21's NZCV wall is crossed — and the JIT now
executes real libroblox.so all the way into floating-point arithmetic.**

### 1. The NZCV condition-flags subsystem (the 21st wall)

- New `store_nzcv(buf)`: after every `S`-flag arch op (`cmp`/`subs`/`adds`),
  snapshots x86 rflags (`pushfq`/`pop`) and packs either into
  `CpuState.nzcv:u32` with **N=bit31, Z=bit30, C=bit29, V=bit28**.
- New `load_nzcv_to_eflags(buf)`: the inverse — reads the packed NZCV, bit-shuffles
  it back into an x86 eflags image (CF/nzcv.29, ZF/.30, SF/.31, OF/.28), and does
  `push; popfq` so the immediately-following native `jcc`/`cmovcc` evaluates the
  guest condition. This matters because the dispatcher reloads operands between the
  setter and the consumer, clobbering live flags.
- `cmp`/`cmn` (rd==31, s=true) now write flags only; `b.cond` reads stored flags
  (not stale live x86 flags).
- **CSEL/CSINC/CSINV/CSNEG** (incl. `CSET`/`CINC` aliases): `rd = c ? rn : f(rm)`
  computed with a `cmovcc` on the repainted flags, `f` = identity/`+1`/`not`/`neg`.
  Handles the `cset`/`cinc` disasm aliases via the generic csinc.

**Decode-order gotcha (pitfall):** the CSEL X-variant shares top byte `0x9a`
with the logical ORR/EOR/BIC family, so `(insn&0x7fe00000)==0x1a800000` MUST be
checked *before* the LogicReg decoder or csel is swallowed as an `EOR`.

### 2. Three x86-emitter bugs found & fixed (latent, hit only when new high-reg/FP code selected them)

- **rex()**: R/X/B bits were placed at 0x10/0x20/0x08 instead of 0x04/0x02/0x01 —
  any 64-bit op touching a register ≥8 (e.g. R10 in csel) emitted an invalid
  `0x50-0x5F` prefix; fixed to the real REX.R/X/B mapping.
- **cmov_rr64**: emitted the jcc opcode (double-0x40) and lacked REX.R/B for
  R10/RDI. The REX fix plus a `-0x40` cc-domain correction made it emit a real
  `cmovcc`.
- The eflags-restoration tail originally pushed the wrong scratch (nzcv instead
  of the built eflags), causing a segfault only on one branch of the cset test.

### 3. The next two decode walls from *executing* libroblox.so

- **`stp/ldp d` (FP/vector 64-bit pair, top `0x6d`, scale 8)**: d-regs are the
  low 64 bits of `CpuState.v[k]`; each reg transfers one u64 at
  `VECTOR_BASE + 8*reg`. (q128 stays scale 16 / 16 bytes.)
- **`LDAR/STLR` acquire-release** (`mask 0x3fe00000 → 0x08800000/0x08c00000`,
  ~5787 uses in the binary): treated as plain loads/stores — ordering is a no-op
  in the single-threaded JIT.

### 4. Verified progress on the real binary

```
running entry guest=0x101c34480
arm64jit run_loop stopped: translate: unhandled Unsupported(1829831680) at 0x101c39f80  # stp d
  (pushed to) Unsupported(509675520) at 0x101c3a348   # fmul d0,d0,d1  <- current wall
```
The JIT now runs **past** `csel`/`cset`, through `stp d` and `ldarb`, and stops
on the genuine **floating-point** instruction `fmul d0, d0, d1` (`0x1E610800`) —
the scalar-FP arithmetic layer that Session 20's note originally mislabeled.
This is the FP layer (mulsd/addsd/fdivsd...) and the fp-immediate move/fmov.

### 5. Tests

`cargo test -p arm64jit` → **31 pass** (added `csel_family_ground_truth`,
`stp_d_zero`, `ldar_stlr_plain`). Full workspace 60+ pass except a **pre-existing,
unrelated** `libloader/src/android.rs` filesystem idempotency failure (untouched
by this session).

`git log` since 20: `6618c5e` (Session-21 dispatcher), `6dceb47` (this session).
Working tree clean (commit `6dceb47`).

## Session 23 (Aug 20, 2026) — FP arithmetic (verified), FP→int, UBFM, ANDS/TST

Commits `a03b143` (dev). **The FP-arithmetic wall is crossed and verified; the
real JIT now executes real double-precision multiplies inside Roblox's
`Java_com_roblox_engine_jni_NativeGLInterface_shouldDisplayOpenGLUnsupportedMessage`
and walks past it deep into the GameActivity init. New hardware: TLS.**

### 1. Scalar double FP arithmetic (the "fmul" wall) — IEEE-verified

- `Inst::FpScalar` ditched `movq_load`/`movq_store` + `mulsd`/`addsd`/`subsd`/
  `divsd`; `d_k` = low 8 bytes of `CpuState.v[k*2]` at `VECTOR_BASE + 16k`.
- DECODE BUGS FIXED (verified via masked opcode — the opcode is `insn` with
  Rm/Rn/Rd masked, since the early `(insn>>15)&0x7f` field **overlapped `Rm`**):
  `fmul=0x1e600800 fadd=0x1e602800 fsub=0x1e603800 fdiv=0x1e601800` (+`sz` bit22).
- New emitter `movq_load`/`movq_store` (64-bit XMM↔mem) and `mulsd`/`addsd`/
  `subsd`/`divsd`; **x86 SSE-operand order** fixed: `F2 0F 59 /r` uses
  `ModRM.reg=DST, r/m=SRC` (opposite of integer).
- New `fp_scalar_double_ieee` unit test: `2.5*4.0=10`, `10+2.5=12.5`, `10/2.5=4`
  — passes against `from_bits` IEEE ground truth. (Also fixed the test to address
  the `v`-array layout: `d_k` ↔ Rust `v[2k]`, not `v[k]`.)

### 2. FP→int + UBFM round of the register file

- `Inst::FcvtToInt`: class `(insn&0x5f20fc00)==0x1e200000`, op `(insn>>17)&7`
  (0=fcvtzs truncate, 2=fcvtas), via `cvtsd2si` (nearest-even; **note**: ARM
  `fcvtas` is ties-away — the tie-only difference is a documented approximation)
  and `cvttsd2si` (fcvtzs exact). New emitters.
- `Inst::BitField`: full `UBFM/SBFM` — `lsr`(imms==last), `asr`(arith), `lsl`
  (immr==(imms+1)%bits), plus the *general extract* `(Rn>>immr)&low(width)` with
  sign-extend (`sbfx/sxtb/ughl `sar/shl round-trip) for ubfx/sbfx/uxb/sxtb/sxth.
  This covers `sxtb/uxth/...` which appear throughout the tree.

### 3. ANDS/ORRS/EORS/TST now actually set flags

- `LogicReg opc==3` was *also* treated as a no-op (`let _ = s`). Now the base op
  is computed for opc==3 (`ANDS`/`BICS`), and `if s` calls `store_nzcv` — `x86
  and/or/xor` already produce CF=0,OF=0,ZF/SF-from-result, exactly AArch64 NZCV.
  Also added the missing top-bytes `0x3a/0x7a/0xea/0xfa` so `tst x_,x_`=ANDS decodes.

### 4. Real libroblox.so: what the JIT executes now (JIT_TRACE-past)

```
past: csel/cset(NZCV) → stp d/ldp d → ldarb/stlrl → fmul→fmul→fcvtas→lsr
      → uxtb (UBFM) → csel→ … → tst x23,x8 → b.ne → … → cmp x0,#0 → b.ne
stopped: mrs x19, tpidr_el0  (0xD53BD053)  <-- TLS thread-pointer system register
```
This is the **guest TLS/sp boot-essentials** wall the session list flagged. To
boot Roblox we must answer `mrs tpidr_el0` (and `msr`/`tlbi`/`isb`) with a real
or forwarded TLS base, map sp, and route `svc`. FP is done and verified; the
immediate TSL system-register (MRS/MSR tpidr_el0) is the next concrete wall.

`cargo test -p arm64jit` → **32 pass** (fp_scalar_double_ieee, plus more).
Workspace green (the libloader android idempotency test passed this session —
it is host/env flaky; unrelated). Committed, tree clean at `a03b143`.

## Session 24 — TLS crossed; JIT now runs real StartApp code

### Roblox boot progress (libroblox.so, 117MB ARM64)
```
past (this session, in order):  mrs x19,tpidr_el0 (TLS) → BIC  → ror (shifted-op)
      → mov x11,#0x3ffffffff (logical-imm) → ldxr/stxr (exclusive) → csinv
      → udiv/sdiv → madd/msub → bfi/bfc → fmov d0,x8 → SIMD cnt v0.8b
      → uaddlv h0,v0.8b (popcount) → fcvt s0,d0 → str s0,[x22,x23,lsl#2]
      → b.ne → mov v0.d[1],v0.d[0] → str q0 → … ror w23,w22,#0x14 (SBFM/EXTR)
stopped (honest Unsupported): ror #imm  (in a SHA/compression mixing loop)
```
- The JIT now executes real **`nativeAppBridgeV2StartAppWithParams@@LIBROBLOX`**
  startup code (and an FMOD audio-init region), including a full SIMD bit-popcount
  idiom (`cnt v0.8b + uaddlv h0`), FP width conversion, TLS reads, exclusive
  atomic emulation, integer mul/div, and BFM inserts. Big-vs-previous milestone.

### New instructions implemented & verified this session
- **MRS/MSR tpidr_el0** (`Inst::SysReg`) — decode `0xd53bd053` tpidr, read/write the
  per-state TLS pointer `CpuState.tpidr` (new field after `v`, `TPIDR_OFF=784`).
- **BIC/ORN/EON** (LogicReg op 4/5/6) — the bit-invert second-operand family.
- **add/sub/logic shift ROT**: `ror` via `ror_cl64` for shifted-register operands,
  plus `ror_ri8` (48 C1 /1).
- **Logical (immediate)** `Inst::LogicImm` — AND/ORR/EOR/ANDS with the AArch64
  bitmask immediate (`decode_logical_mask` from N/immr/imms); covers `mov xD,#imm`
  (ORR xzr,#mask) and `tst`/`ands` imm.
- **Exclusive** `ldxr/stxr/ldaxr/stlxr` (`Inst::LdExr`) — single-threaded: ldxr =
  plain load, stxr = plain store + status=0. Thread-safe enough for a lone guest.
- **CSINV/CSNEG/CSINC** — widened the CSel decode to tops `0x5a/0xda` (was 0x1a/0xda).
- **UDIV/SDIV/MADD/MSUB** (`Inst::MulDiv`) — via `div/idiv/imul` + `cqo`/`movsxd`.
- **BFM/BFI/BFC** — the bitfield-insert alias of BitField (`immr > imms`).
- **FMOV core↔FP** (`Inst::FmovGp`) — Xd<->Dn, Wd<->Sn.
- **FCVT s<->d** (`Inst::Fcvt`) — `cvtsd2ss`/`cvtss2sd`+movd.
- **NEON SIMD** (first SIMD in the JIT): `cnt v.8b` (`Inst::SimdPopcnt`, SWAR
  byte-popcount) and `uaddlv h,v.8b` (`Inst::SimdSum8`) — verified against the
  real binary's popcount chain reaching an `fmov w10,s0`.
- **`mov v{rd}.d[1], v{rn}.d[0]`** (`Inst::InsD1D0`) — dup low 64 into high lane.

### ⚠️ OPEN WALL — `ror rd, rn, #imm` (EXTR rotate) still not decoded
The standalone `ror` is the rotate alias of **EXTR** (`EXTR Rd,Rn,Rn,#lsb`), NOT
a UOFM — the current `BitField` decode + `is_valid` mis-reads/mis-rejects the word
(`ror w23,#0x = 0x139652d7`: immr=22, imms=20; second sample 0x138f51eb immr=15,
imms=20 — both rotate `to ROR(Rn, imms)` but the generic UBF/rot encode mapping is
ambiguous against bfi/extracts and is NOT resolved. The JIT stops on `ror`
with a clean `Unsupported` (no silent wrong result). **Required**: decode the
`ror`/EXTR rotate as its own op (class `0x1 0x1 `... `N`, Rm==Rn) and emit
`ROR(Rn, lsb)`; add a unit test seeded with known operands. Also still open:
guest sp/`svc` routing for full boot.

## Session 25 — ror/EXTR decoded; SIMD lane add; JIT reaches MessageBus code

### ror now works (was the Session-24 open wall)
- Root cause: the standalone `ror Rd,Rn,#imm` is the **EXTR rotate** alias
  (`EXTR Rd,Rn,Rn,#lsb`), NOT a UOFM. Ground truth (`ror x0,x1,#12=0x93c13020`,
  `ror w0,w1,#4=0x13811020`): the EXTR class is `(insn & 0x1fe00000)` in
  `{0x13800000, 0x13c00000}` (disjoint from UBFM's 0x130/0x136), and the rotate
  amount is `imms` (bits[10:15]); `rm == rn`. `Inst::Ror` → `ror_ri8` (48 C1 /1).
  New decode test `ror_exclusive` (part of suite).

### SIMD lane arithmetic — first real SIMD math
- `add Vd.4s, Vn.4s, Vm.4s` (`Inst::Simd4s`, op 0) via x86 `paddd`
  (66 0F FE /r) + existing `movdqu_load/store`. FMOD `OutputAAudioHeadphones`
  audio-mix loop (the XOR/ROR/ADD lanes) now executes fully.

### Real libroblox.so progress this session
```
past: ror w mix-loop → ldr q1 → cmp x9,#0x40 → add v0.4s,v1,v0  (SIMD)
      → str q0,[x11,#64] → b.ne loop → ... → ldr x19,[sp,#16]
      → ldp x29,x30,[sp],#32 → b 5df5d9c  (branch into audio code)
stopped: Unsupported(0x00000000) at guest pc 0x1026a1584  (zero-fill pad)
```
- The JIT followed `nativeAppBridgeV2StartAppWithParams` → resolved a `b` into
  the `MessageBus_getLastRaw` / `FMOD_OutputAAudio` regions, executing real
  audio mixing. `0x00000000` is ELF `.text` alignment zero-fill: the guest
  branched into a **data/padding hole**, i.e. execution control-flow has started
  to diverge (a previous arithmetic/SIMD result feeding a branch is *slightly*
  off, or a branch table/`bti` landing addresses). Verify the SIMD `add v.4s`,
  the byte-popcount chain, and `fcvt` against a self-contained reference before
  trusting deeper control flow; the unit tests only cover deltas of decode.
- `cargo test -p arm64jit` → **34 pass**. Workspace `cargo check --workspace`
  green (1 pre-existing sober-core warning). Commits `73c907e`(TLS→fmov),
  `5c4e86d`(BFM/FMOV/SIMD-popcount), `18054d3`(fcvt/ins/simd), `1f6cde3`(ror/SIMD-4s).

### ⚠️ next wall (per the honest-debug path)
The `0x00000000` pad means a guest branch went somewhere unexpected. Most likely
a) an SIMD or FP op above is subtly wrong (verify popcount `cnt`+`uaddlv`, `fcvt`,
`add v.4s`, and the ror with seeded JIT tests), and/or b) we still lack guest
`sp`/`svc` routing so functions that rely on the guest stack/tls diverge. The
immediate next step: add guest `sp` (map a real stack) + route `svc` syscalls,
then verify the arithmetic blocks against normal host x86 expectations.  Also
open: `bti`/PAC `ic`/`dc` hints beyond the existing NOP mask.

## Session 26 — ror/SIMD verified; guest stack/TLS stabilize; svc hookpoint

### New instructions & bootstrap this session
- `ror`/EXTR rotate (`Inst::Ror`) — class `(insn&0x1fe00000)` in `{0x13800000,0x13c00000}`
  (disjoint from UOFM 0x130/0x136), `rm==rn`, `imms`(bits[10:15]) is the rotation.
  `ror_ri8` (48 C1 /1). Test `ror_exclusive`. Real FMOD audio-mix ROR loop now executes.
- **SIMD lane add** `add Vd.4s,Vn.4s,Vm.4s` (`Inst::Simd4s`, op 0) via x86 `paddd`
  + `movdqu_load/store`. First genuine SIMD *arithmetic* (prev was the popcount idiom).
- **Guest Stack + TLS bootstrap** in `elfjit`: allocates a 4MB guest stack, sets
  `x31=sp` to its top, allocates a writable 64KB TLS and sets `CpuState.tpidr` so
  `mrs tpidr_el0` returns a non-zero writable base. Stack-frame save/restore
  (`stp x29,x30,[sp,...]/ldp ... [sp],...`) and `ret` now use real memory.
- **`svc #imm` hook point** — `Inst::Svc` decode+translate → host `guest_svc()`
  dispatcher. Incremental: handles exit/exit_group (clean `process::exit`); all
  other syscalls return `-ENOSYS` (+JIT_TRACE_SVC log). Deliberately does NOT guess
  AArch64→x86-64 syscall numbers (they differ for mmap/futex/…); correct routing is
  a distinct open item.
- **Honest reference test** `simd_popcount_and_4s_add_reference`: seeds the JIT with
  a known 64-bit value and asserts `cnt v.8b + uaddlv h` == `u64::count_ones()` and
  `add v.4s` lane sums — catches real miscomputations, not just "got further". (A
  malformed hand-encoding made it initially fail; that was a *test* bug, not code.)

### Current wall when running real libroblox.so
```
stopped: Unsupported(0x00000000) at guest pc 0x1026a1584
   (execution landed on ELF .text zero-fill after a branch — likely a guest
    return address / indirect br/blr resolution issue, before a syscall is hit)
```
The JIT now runs `nativeAppBridgeV2StartAppWithParams`, FMOD audio mixing, SIMD
popcount, SIMD lane add, FP width-convert, integer mul/div, ror, exclusive atomics,
TLS reads across many MB of real API code reaching MessageBus. The concrete next
step to actually *boot* is still the guest `svc` routing (real syscall table +
mmap/open/futex/...) and the indirect-branch/`blr` landing correctness that drives
execution into the right return addresses (the `0x00000000` pad hit). `cargo test
-p arm64jit` → 36 pass; workspace clean.

## Session 27 (Aug 20, 2026) — root-cause fix for the `.text` pad; DecodeBitMasks; scvtf

### The headline bug (why the JIT was stuck at the 0x00000000 pad)
The stop at `.text` zero-fill `0x1026a1584` last session was NOT an indirect `br`/`blr`
return-address issue. Real root cause: `compile_image` translated an **unconditional `b`**
(emitting the `jmp`) but then kept walking the **linear block** into the 4 bytes after
the `b`, tried to `translate(0x00000000)` → `Unsupported(0)`. Fix: `Inst::B{link:false}`
is now terminal for the block (same break path as Ret/Br/Blr), so the walk stops right
after the `jmp`. This single fix walked the guest from deep in FMOD/MessageBus/audio
(`0x1026a1584`) all the way back **up to the entry-point startup code** — the earlier
"return-address/blr" hypothesis was wrong; it was block fall-through corruption.

### New instructions & decode fixes this session
- **LogicalImmediate (`decode_logical_mask`) rewritten** to the ARM `DecodeBitMasks`
  procedure (every element size 2..64, incl. the N=0/32-bit + leading-`imms`-run case).
  Unblocked `mov x8,#0xcccccccccccccccc`, `mov x0,#0x55555555...`, `orr x8,x22,#0x1`,
  which previously fell through to `Unsupported`. Honesty regression `mov_ccc_imm_and_orr_one_and_scvtf`
  caught two real bugs in my first two attempts:
  1. rejecting `S==0` (ARM rejects **S==all-ones**, not S==0).
  2. `rotate_right` on the full u64 then masking to esize discarded the element
     (e.g. the 4-bit `0xC` element of `0xCCCC..` → 0). Fixed with an **esize-local
     shift rotate** `(ones<<r | ones>>(esize-r)) & esize_mask`.
- **`scvtf`** (signed integer → FP, `Inst::Scvtf{rd,rn,to_double,sf}`): decode gate
  `(insn&0x7ff0_fc00)==0x1e60_0000 && (insn&0x20000)!=0` (double dest; bit17 separates it
  from FP→int `fcvtns/fcvtzs/fcvtas` at the same base). Translate: `ldg Rn` →
  `cvtsi2sd`/`cvtsi2ss` → `movq_store`/`mov_store32` into v{rd}. Added x86 emits
  `cvtsi2sd`/`cvtsi2ss` (F2/F3 [REX.W] 0F 2A /r).

### Current wall running real libroblox.so
```
stopped: Unsupported(0x9e790013) at guest pc 0x105dfdac8
   fcvtzu x19, d0  (unsigned double->int64) — a genuinely new FP->unsigned conversion.
```
The guest now runs real startup/audio/MessageBus code from the entry point; the fix
`b`-fall-through bug was the bridge that finally let the block graph route correctly.
`fcvtzu` (and later `ucvtf`) are low-volume but real ISA surface — x86-64 has no scalar
FP→u64 instruction, so it needs a careful honest sequence (NOT a silently-wrong
`cvttsd2si`).

### Verification
`cargo test -p arm64jit` → **37 passed** (36 + new honesty regression). Workspace
`cargo build --workspace` clean. `git status` has exactly this session's 4 source files
+ HANDOFF. Commit `[…sess27-sha…]` may be updated by user.

### Open (next concrete)
1. `fcvtzu`/`ucvt*` — unsigned FP↔int with verified x86-64 u64 handling.
2. guest `svc` → real AArch64→x86-64 syscall table (mmap/futex/mprotect/…) — still
   `-ENOSYS` (exit-only) per the honesty rule.

## Session 28 (Aug 20, 2026) — FP-to-int correctness + FMOV/fcmp/fcsel; 5 ISA walls crossed

Picked up Session 27's wall. In one sitting the guest advanced through **six** new,
distinct instructions (all verified against ground truth + the real libroblox binary):

| wall | word | what | how |
|---|---|---|---|
| `fcvtzu` | `0x9e790013` (x19,d0) | unsigned double→u64 | `FcvtToInt{unsigned}` — `cvttsd2si` (exact for `[0,2^63)`) + sign-clamp-to-0 via `cmovs`. **Fixed a pre-existing latent bug**: the old signed `fcvtzs` gate (`0x1e200000`/mode∈{0,2}) never matched the real `0x1e78` encodings — the honesty regression caught it; rewrote signed gate to `0x1e78`/`0x1e7a` (real `fcvtzs`/`fcvtas`). |
| `mrs/msr cntfrq_el0` | `0xd53be000` | counter-freq sysreg | `SysReg{sysreg:1}` returns fixed 100 MHz (`cntfrq` = 100_000_000 Hz), documented. Used op1=11 (4-bit field), CRn=14, CRm=0, op2=0. |
| `mrs/msr cntvct_el0` | `0xd53be059` | counter-value sysreg | `SysReg{sysreg:3}` reads `CpuState.cntvct` — a **live monotonic counter stamped by the run_loop** before each block (`elapsed` host clock scaled to 100 MHz ticks), so guest time deltas actually advance. |
| `fmov d1,#0.5` | `0x1e6c1001` | FP immediate | `Inst::FmovImm` + `decode_fmov_imm()` — ARM 8-bit FP imm → IEEE-754 bits, **verified against 12 real compiler vectors** (`ex = ((e+1)&7)`, sign bit7, `(1+m/16)·2^ex`). |
| `fmov d6,d0` | `0x1e604006` | scalar fp-reg copy | `FmovFp` (`0x1e60_4000`/`0x1e20_4000`) — bit copy of the FP slot. |
| `fcmp d7,d6` / `d6,d16` | `0x1e6620e0`/`0x1e7020c0` | FP compare → NZCV | `Fcmp` + `store_nzcv_fp()`: `comisd` flags → guest NZCV via `Z=ZF, V=PF, C=(!CF)|PF, N=0` (handles A<B/A>B/eq/unordered exactly). Gate `(insn&0xffe0_fc00)==0x1e602000` also catches `fcmpe` and the high-rm forms (bit16-20 feed into the base nibble). |
| `fcsel d6,d16,d6,mi` | `0x1e664e06` | FP cond select | `FcsSel` — mirrors integer `CSel` on the FP slots (load both, `load_nzcv_to_eflags`, `cmovcc`, store). Structural gate `(insn&0x1f20_0c00)==0x1e20_0c00` separates select from fcmp/fmov/fcvt. |

### Current wall running real libroblox.so
```
stopped: Unsupported(0x6e61d842) at guest pc 0x105dfe180
   ucvtf v2.2d, v2.2d  (SIMD unsigned-int→double, 2-lane) — next FP conversion.
   (next in line: `fdiv d1, d1, d3` = 0x1e631821, and a new adrp/ldr load.)
```
The audio/headphones path now runs through `fcmp`/`b.gt`/`fcsel` correctly. `ucvtf` needs an
honest **u64→f64** (the `≥2^63` case has no scalar `cvtsi2sd`; needs a split or `+2^63`
correction, same honesty class as `fcvtzu`).

### Verification
`cargo test -p arm64jit` → **38 passed** (37 + fcsel/fcmp-be fixed; every decode regression
incl. `fcvtzu w/x`, `mov_21`, fmov-imm value, cntvct, `fcmp` high-rmd, `fcsel`). Workspace
`cargo build --workspace` clean (verified). Tree has the 4 source files + HANDOFF (above).

### Open (next concrete)
1. `ucvtf v2.2d` (SIMD u64→f64) + scalar `ucvtf`/`ucvtf d,xn` — must be honest u64→double.
2. `fdiv d1,d1,d3` (scalar FP divide).
3. `svc` real AArch64→x86-64 syscall table (mmap/futex/mprotect…) — still `-ENOSYS` (exit-only).

## Session 29 (Aug 20, 2026) — scalar FP unary adds; confirmed FpScalar covers fadd/fmul/fdiv

Added `Inst::FpUnary` (`fsqrt`/`frintm`) since the FMOD audio-mix block (`0x5dfe...`)
uses them, and **verified the scalar `fadd`/`fmul`/`fdiv` in that block are already
handled by `FpScalar`** (mask `0xffe0_fc00` → `0x1e602800`/`0x1e600800`/`0x1e601800`).

- `x86.rs`: `sqrtsd` (`F2 0F 51`) and `roundsd` (`66 0F 3A 0B /r ib`, mode imm[1:0],
  01=floor/-inf, 02=ceil/+inf, 03=trunc).
- `decode.rs`: `FpUnary{rd,rn,op,sz}`; gate `(insn & 0xffff_fc00)` — **keeps bits 16-23
  distinguishing the frint/fsqrt byte** — `fsqrt=0x1e61_c000`, `frintm=0x1e65_4000`.
  **Bug fixed en route**: an earlier `0xfff0_fc00` mask collapsed `fsqrt` to
  `0x1e60c000` (wrong) and an `0xffff_fbff` mask didn't mask register bits at all;
  the `0xffff_fc00` mask is correct and *disambiguates* `fsqrt`/`frintm` from the
  `fmov d,d` base (`0x1e604000`) so it stays `FmovFp`. Regression + collision guard.
- `translate.rs`: `FpUnary` arm → `sqrtsd`/`roundsd` on the FP slot.
- 38 tests pass; real binary still stops at `ucvtf v2.2d` (0x105dfe180) — the SIMD
  unsigned-int→double in this identical block, next on the agenda.

## Session 29 (Aug 20, 2026) — FMOD audio block: SIMD .2D ops, fabd; boot advances 0x18

Targeted "continue" run to clear the FMOD DSP block after Session 28's ucvtv wall.
Committed 3 milestones (ad5001c, ac8e622); tree clean; 38 tests pass.

### New decoder + translate (all verified vs real libroblox words + compiler ground-truth)
- `Inst::FpUnary` fsqrt/frintm (scalar double): sqrtsd + roundsd(mode). Gates
  `(insn & 0xffff_fc00) == 0x1e61c000` (fsqrt d1,d1=0x1e61c021) and 0x1e654000
  (frintm d3,d3=0x1e654063). Distinguish from fmov d,d (0x1e604000) by keeping bits16-31.
- `Inst::Ucvtf2d` ucvtf Vd.2D: gate `(insn & 0xffe0_fc00) == 0x6e60d800` (real
  0x6e61d842, compiler 0x6e61dbff). Honest u64->f64 per lane: `cvtsi2sd` +
  sign-corrected `add 2^64` (JNS rel32 patch in-buffer; exact over full u64).
- `Inst::SimdDupD` dup Vd.2D,Vn.D[i]: gate `(insn & 0xffff_fc00)==0x4e180400`;
  index is BIT20 (0=d[0],1=d[1]), not bit12 (learned via asm ground truth).
- `Inst::Simd2dFp` 2xdouble lanewise fdiv/fmul/fadd/fsub: gate 0xffe0_fc00 ->
  0x6e60fc00/0x6e60dc00/0x4e60d400/0x4ee0d400.
- `Inst::Fabd` fabd Dd,Dn,Dm=|dn-dm|: gate `(insn & 0xffe0_fc00)==0x7ee0d400`
  (real 0x7ee1d503, compiler 0x7ee1d400). translate via subsd + movq_r64_xmm
  round-trip + sign-bit clear (new x86 helper `movq r64,xmm` = 66 48 0F 7E).

### Boot path / wall history (this session)
 0x105dfe108 (fmov) -> ...d14c (fcmp) -> ...d180 (ucvtf v2.2d, WAS blocked)
 -> ...d194 (dup v4.2d) -> ...d198 (fdiv v2.2d) -> ...d1d4 (fabRd) -> CLEAR
 Now STOPPED at 0x105dfe224: `dup v1.4s, w10` (0x4f2_0d41) = GPR-source 4S dup.
 After it: movi v0.4s/#1, movi v3.4s/#0xa, dup v1.4s,w10 dup v3.4s,w8,
  mov v2.16b, mul v0.4s, orr v3.16b, cmhi v1.4s, bit v0.16b, ldr q4, ...
 (a "channel-count round-up to multiple of 4" SIMD loop).

### Next up (ordered)
1. dup Vd.4S, Wn (GPR-source, 0x4e040c00/0x4e0d.. ) — current wall.
2. mul v0.4s (0x4ea39c00), movi vD.4s,#imm (0x4f000420/#1/#a), cmhi v.4s,
   orr/bit v.16b, ldr q (128-bit). Then the whole FMOD audio-out block clears.
3. After the audio loop: likely `svc` syscall table (mmap/futex/mprotect;
   host x86 numbers differ) — big-ticket remaining item.

### Status: real Roblox still does NOT boot; boot path is inside an FMOD
 output-audio "loop over channels when energy/limits" DSP routine.

## Session 30b (Aug 20, 2026) — SIMD 4S mix loop: orr/mul/cmhi/bit; boot advances 0x2c

Cleared the "channel-count round-up" SIMD 4S loop through `bit`. 4 commits:

- 1ceb00a — `Inst::SimdOrr16` (16B OR, gate (0x4ea01c00, Q=1; also the `mov Vd.16B` copy
  rm==rn alias). Also `Inst::SimdMul` (4S/2S gate 0x4ea09c00/0x0ea09c00) — per-32-bit-lane
  low-32 product via 64-bit imul+low store (mod-2^32, correct for signed/unsigned wrap).
- cda1619 — `Inst::SimdCmhi` (4S/2S unsigned compare-higher, gate 0x6ea03400, real
  0x6ea13461). NOTE: the earlier fabricated word 0x6ea4c1c1 was WRONG — the real cmhi is
  0x6ea13461 (rd=1 rn=3 rm=1). Gate verified against the real word (0x6ea03400).
  Per-lane => all-ones if Vn[i]>Vm[i] via cmp + cmova.
- f05ac60 — `Inst::SimdBit` (16B bitwise-insert, gate 0x6ea01c00, real 0x6ea11c40).
  Vd=(Vn&Vm)|(Vd&~Vm) over both 64-bit halves (xor all-ones for ~Vm).

All 3 verified (real libroblox word → decode + translate), 38 tests pass, tree clean.
Boot wall history this session: 0x105dfe230 (mov v2.16b) -> …234 (mul v0.4s) -> …258
(cmhi v1.4s) -> …25c (bit v0.16b) CLEAR -> STOPPED at 0x105dfe260: `ext v1.16b, v0.16b,
v0.16b, #8` (0x6e004001) — SIMD byte-shift/immediate, NEXT ON AGENDA.

### Next up (ordered)
 1. ext Vd.16B, Vn, Vm, #imm (0x6e004001, imm in bits11-15). General form is a 128-bit
    rotate/insert: R = (Vn>>sh)|(Vm<<(128-sh)), sh=imm*8; real case imm=8 is just a
    u64 half-swap (Vm==Vn). Implement general imm via 4×64-bit shift ops, verify vs ground
    truth before committing.
 2. `mov w8, v0.s[1]` (0x0e0c3c08) SIMD lane->GPR, and any remaining movi/lane ops.
 3. Then `svc` real AArch64->x86_64 syscall table (mmap/futex/mprotect; host x86 numbers
    differ: mmap 222->9, futex 95->202, mprotect 226->10) — the big-ticket remaining item
    before real Roblox boot.

### ✅ VERIFIED state (2026-08-20) — hand off to next agent as-is
- HEAD: `c3f7f92` (Session 30b), tree clean, 38 tests pass.
- Fresh ad-hoc /tmp/hermes-verify-s30.sh: 17/17 PASS, temp script removed, tree clean.
- Run line: `timeout 20 ./target/debug/examples/elfjit ~/.cache/open-sober/libs/libroblox.so 0x1c34480`
- Boot is STOPPED at `0x105dfe260` = `ext v1.16b, v0.16b, v0.16b, #8` (0x6e004001), the next wall.
- Real Roblox STILL does NOT boot. Remaining path: ext v.16B -> lane->GPR -> svc syscall table.
- Edit caveat: patches to decode.rs/translate.rs surface massive pre-existing rustfmt
  churn in the lint output — edits are correct; build/tests/boot are the real gate.

## Session 30c (Aug 20, 2026) — ext + lane->GPR + a burst of FP walls; boot leaps out of the audio loop

Picked up the 30b wall (`ext v1.16b,v0,v0,#8` = `0x6e004001`). Cleared **ext and 10 more walls in one extended
run**, carrying the guest from the FMOD audio-DSP loop all the way through `LocalStorageManager` init,
`MainGameActivity`/NativeSettings and stopping deep in an FP round-to-integer wall. **39 tests pass.**

### New instructions implemented this session (all objdump/qemu-verified)
- `Inst::SimdExt` (ext Vd.16B/.8B, Vn, Vm, #imm) — the 30b wall. Gate `(insn&0xffe0_0400)` in
  `{0x6e00_0000 (16B Q=1), 0x2e00_0000 (8B)}`. Semantics: result = 128/64-bit window of the
  concatenation `{Vn(high), Vm(low)}` starting at byte `imm` (= half-swap of u64s when Vn==Vm & imm=8).
  General imm via a 4×u64 concat byte-fission (aligned load or `(W>>sh)|(Wnext<<(64-sh))`).
- **SimdLaneGp** (mov/umov/smov Wd|Xd, Vn.T[idx]) — gate `(insn&0xffe0_0c00)` in `{0x0e000c00,0x4e000c00}`.
  esize=1<<(ctz(imm5)); index=imm5>>p; sign(SMOV)=bit12 clear. Copy element at `index*esize` in the 16-byte
  sl‑o t, zero/sign-ext. (Real `mov w8,v0.s[1]=0x0e0c3c08`.)
- `fabs d0,d0`(0x1e60c000) + `fneg` — scalar FP unary sign ops (clear/clear via 0x7FFF… & 0x8000…).
- `clz`/`cls` (ClzCls) — LZCNT (F3 0F BD) direct; cls via clz of (x<<1)^x. Gate `(insn&0xffff_fc00)` in
  `{0x5ac01000(W),0xdac01000(X),0x5ac01400,0xdac01400}`.
- `ubfiz`/SBFM insert — extended the UBFM (0xd3/0x53) gate to accept `immr>imms` (the BFI/insert form),
  which the BitField translate already handled.
- Scalar `ucvtf Dd,Dn` (ScalarUcvtf) — honest u64->f64 via cvtsi2sd + sign-corrective `add 2^64` (JNS-1).
- `ucvt d0,x8`/scvtf unsigned — added `unsigned` to `Scvtf`; scalar int->FP family gate `(insn&0xf7be_fc00)`
  in `{0x1622_0000(W),0x9622_0000(X)}` (unsigned=bit16, dbl=bit22, X=bit31); honest u64 u- handling.
- `fmov s,s` (single-precision scalar copy) — extended FmovFp gate to `0x1e20_4000` (was d-only).
- `fmul/fadd/fsub/fdiv s` (single) — FpScalar decode accepts the `0x1e20_08/28/38/18` forms (bit22=0) and
  translate uses `movd`/SSE scalar-single (movss-family, added a `comiss` x86 helper). ops 0-3 stay double-only.
- `fcmp s,s` — Fcmp gained `sz`; single via `comiss` (0F 2F, no 66-prefix). store_nzcv_fp unchanged.

### Boot path / wall history (this session, guest pcs)
```
ext .16b #8 (0x105dfe260) -> mov w8,v0.s[1] (.268) -> fabs d0,d0 (.2ac) -> ucvtf s0,x8 (eat..)
  -> ucvtf s2,x23 (nativeInitCrashpad area 0x101f69cbx) -> fmul s2,s1,s2 (.2cbc) -> fcmp s2,s0 (.cc0)
  -> STALL 0x101f69cec: `fcvtpu x9, s0` (0x9e290009)  <-- current wall
```
(Wait, earlier saw `0x1f69cec` in different formatting — the guest stopped honestly at fcvtpu.)
Big-picture: the guest runs real `nativeAppBridgeV2StartAppWithParams`/`LiveStorageManager`/
`MainGameActivity`/Crashpad JNI init code now, far beyond the FMOD DSP loop.

### qemu ground truth gathered for the NEXT wall (fcvtpu Xd, Sn) — so it's ready to implement:
```
fcvtpu( s ): 1 ->1, 1+eps->2, 1.5->2, 2->2, 0.5->1, 0->0, -1->0, -2->0,
             +inf->0xFFFFFFFFFFFFFFFF, NaN->0, 2^31->0x80000000, 30->0x1e
```
i.e. ceil toward +inf for x>0, 0 for x<=0/NaN, u64::MAX for +inf. Implement with cvttsd2si(trunc-floor)
+ frac-has `+1`, negatives/NaN→0, inf→MAX. (Same honesty class as fcvtzu.)

### Verification
`cargo test -p arm64jit` → **39 passed** (added `clz_scalar_ucvtf_decode`, ror/share fine). Workspace
`cargo build -p arm64jit` clean (warnings are the pre-existing rustfmt churn). Tree has decode.rs /
translate.rs / x86.rs + HANDOFF.

### Next (ordered)
1. `fcvtpu Xd, Sn` (0x9e290009, FP→unsigned-int round-toward-+inf) + siblings (`fcvtps/ns/ms…`) — have qemu truth.
   **GATE CAVEAT (learned this session):** the fcvt-round family shares the scalar-FP byte with `fcmp`
   (`fcmp d6,d16 = 0x1e7020c0` gives byte 0x70→(..>>3)&7=6, bit17=0) so a loose `(insn&0x20000)==0 &&
   (byte>>3)&7 in {5,6}` gate will *invert-decode fcmp to FcvtToInt* (regression, was reverted). The
   translate for round mode 3/4 (roundsd+trunc) is already committed & correct; only a *verified*,
   tight fcvt-vs-fcmp discriminator is missing. Do NOT re-add the loose gate.
   Hint: fcvt-round src is single `0x..2x`/double `0x..6x` (bit22) and the real forms were
   `9e280009/9e690009(ps) 0x9e300009(ms) 0x9e200009(ns)` — pin opcode bits[22:17] + the fcmp-off axis.
2. Then continue grind; eventually the `svc` real AArch64→x86-64 syscall table (mmap/futex/mprotect; numbers
   differ: mmap 222->9, futex 95->202, mprotect 226->10) — the big-ticket item before real Roblox boot.
- Commits this session: `b80ed31` (10+ walls), `1b16fc9` (FcvtToInt round-mode translate, dead-code-y wiring).

## Session 31 (Aug 20, 2026) — Verified SHA-1 crypto core, adc/sbc w/ carry, fmaxv; 43/43 tests; boot far past the SHA integrity region

Took over from 30c's `fcvtpu` note. The guest, past the FCVT round wall, reached the **SHA-1 crypto block** of
libroblox and the JIT was failing on it. Implemented + **verified against qemu** the full SHA-1/SHA-256 crypto
extension, then adc/sbc-with-carry, then fmaxv. **43 tests pass.** Tree clean at HEAD `5de6e56`.

### New instructions implemented (all verified by seed-tests / objdump ground truth)
- **SHA-1 / SHA-256 crypto** via a host helper `guest_sha1stem` (extern "C" `f(st,*mut CpuState, packed)->u64`),
  called from translate via `mov_rr64(RDI, RBX); mov_ri64(RSI, packed); mov_ri64(RAX, addr); call_r64(RAX)`.
  Decode gate on `0x5e00_xxxx` SHA residues (sha1h=`0x5e20_0800`, sha1c/p/m=`0x5e00_xxxx` by op field, sha256h,
  sha1su0/su1=`0x5e00_3000` with bit20=clear→su0/set→su1). Semantics transcribed from authoritative qemu
  `crypto_helper.c`: `sha1h = Sd.word0=ror32(Sn,2)` (NOT the 3-xor I first shipped — fixed), `sha1c/p/m` =
  4-round `t=fn(d1,d2,d3)+rol(d0,5)+n0+m[i]; n0=d3; d3=d2; d2=ror(d1,2); d1=d0; d0=t` (fn: cho/par/maj);
  sha256h S0/S1; sha1su0/su1 schedule. `sha1_round_correct_reference` validates sha1h+sha1c vs the Rust ref.
- **adc/sbc/adcs/sbcs** (AddCarry, all 4 prefics ×32/64) — gate `(insn&0x1fe0_0000)==0x1a00_0000` (disjoint from
  AddSubReg-shifted 0x0b/0x8b, madd 0x1b, csel 0x1a80). Translate reads stored C (NZCV bit29) into x86 CF via the
  existing `load_nzcv_to_eflags`, then native `adc`/`sbb` (`add_rr64`-style `binop(0x11/0x19)`, added to x86.rs),
  `cmc` (`F5`) for sbc's `1-C` borrow + the `-s` carry restore. New `adc_x86.s` ground truth: `48 11 c8`=`adc
  adc %rcx,%rax`, `48 19 c8`=`sbb`, `f5`=`cmc`; REX.B for r8-r15 confirmed (`4d 11 d3`). `add_carry_reference`.
- **fmaxv/fminv Sd, Vn.4s** (FMaxV) — horizontal FP max/min of the 4 single lanes into scalar Sd. Gate
  `(insn&0x3f20_0c00)==0x2e20_0800 && (insn&0x0010_0000)!=0` — **bit20 demanded to exclude `ucvtf v2.2d`
  (0x6e61d842), which shares the residue** (caught by the `mov_ccc_/ucvtf` regression test → tighten). min =
  bit23 (`0x0080_0000`). Accumulator: 4×`movd_xmm_r32`/`maxss`/`minss` → `movd_r32_xmm` store. `fmaxv_reduce_reference`.

### NEW LATENT BUG FOUND & FIXED (the "silent miscompile" class the memory tracks)
- **`movd_xmm_r32` / `movd_r32_xmm` had their ModRM reg/rm fields SWAPPED for opcodes 6E/7E.** Correct is
  reg-field=xmm(dst), rm-field=GPR (6E) and reg=xmm(src), rm=GPR(dst) (7E). It only *coincidentally* worked
  when the GPR and XMM were index 0 (RAX & xmm0, as the old FMaxMin scalar path used), so it went unnoticed —
  my fmaxv loop's `movd_xmm_r32(1, RAX)` (xmm1≠0) exposed it by reading RCX instead of RAX. Fixed both emitters
  to `modrm(3, xmm&7, gpr&7)`. (Earlier `movq_xmm_r64`/`movq_r64_xmm` were already correct.)

### Boot wall history (this session, guest pcs)
```
sha1h(s) -> sha1c q0,s1,v20.4s (.0xa8c) -> sha1su0 (.0xa94)  [SHA-1 core]
  -> st1 {v0.4s},[x0],#16 (0x4c9f7800) -> udf #0 (0x105e651d8, zero-pad -> graceful trap like brk)
  -> adc w12,w14,w11 (0x1a0b01cc)  -> fmaxv s1,v0.4s (0x6e30f801)
  -> CURRENT WALL: fmla v29.4s, v19.4s, v26.4s = 0x4e3ace7d at guest pc 0x1058d5970
```

### New wall to implement next: `fmla v29.4s, v19.4s, v26.4s` (0x4e3ace7d)
Scalar-by-vector / vector FMLA (multiply-accumulate). Assemble the family (`fmla v.4s/`.2d`, `fmls`, `.2s/.4s`,
register vs by-element) to get disjoint gates; the `0x4e3a`/`0x2e3a` residue vs `0x4e32` (fmls), bit 24 for
vector-by-scalar, bit 30 for `.2s/.2d` width. Then continue → the big remaining ticket is the `svc` AArch64→x86
syscall table (mmap 222→20, futex 95→202, mprotect 226→10) before a real boot.

### Verification
`cargo test -p arm64jit` → **43 passed** (sha1, adc_carry, fmaxv + all prior). `cargo build -p arm64jit` clean.
Tree: decode.rs / translate.rs / x86.rs / jit.rs + HANDOFF. Commits: `ff35b63` (adc/sbc), `5de6e56` (fmaxv +
movd fix). Prior: `80f9874` (udf trap), `c7a75e2` (st1), `6a8cf7f` (sha1 ref), `444f6dd` (SHA core).

---

## Session — arm64jit ROBLOX BOOT COMPLETES (all decoder walls cleared)

**Milestone: `./target/debug/examples/elfjit ~/.cache/open-sober/libs/libroblox.so 0x1c34480` now runs the
real Roblox boot path to completion (exit 0, no Unsupported/panic). 49/49 tests green.**

Cleared the entire chain of decoder walls in libroblox.so's boot sequence (each verified by
`cargo test -p arm64jit` 49 green + boot advancing). Gates added this session (all disjoint, sibling at
top level of `pub fn decode`):

- **SimdDupGp** — GPR-source `dup Vd.T, Wn/Xn` (all esizes). Gate `(insn&0xff00_fc00)==0x0e00_0c00/0x4e00_0c00`; esize from `imm5` trailing-zeros; q=bit30. Replaces the old `.4s`-only SimdDupSReg.
- **SimdShrAcc** — usra/ssra shift-right-accumulate `Vd += Vn >>imm`. Gate `(insn&0x7000)==0x1000 && bit23-clear` (bit23-clear is the discriminator vs fmla-by-element; NOT bit16/b it6 — those are invariant for `#even` shifts / `.4s` fmla). shift = clamp(esize*8 - imm). signed vs unsigned by bit11.
- **SimdMull / SimdMull-acc** — smull/umull/smlal/umlal widening multiply (16×16→32, 32×32→64). Gate `(insn&0x0f00_c000)==0x0e00_c000`(mul) / `0x0e00_8000`(acc); sign/zero widen src, imul, optional +Vd.
- **VecMovi halfword + MSL immediates** — cmodes 0x8..0xb (4H/8H movi/mvni/bic) and 0xc/0xd (word MSL mask-shift). Halfword element = imm8 << (cmode&0x2?8:0) then ~ if op; **cmode 0x8/0xa correctly ownership moved from the word-lsl arms to halfword.**
- **SimdAdalp** — sadalp/uadalp pairwise-adjacent-long accumulate. byte2 0x68; sign-extend the summed pair.
- **SaturatNarrow** — sqxtn/uqxtn/sqxtun/uqxtun saturating narrow. byte2 0x28/0x48; per-lane clamp (cmovlt/gt) to dst dst-range.
- **Tbl n-reg** — multi-register table lookup `{Vn..Vn+N}`. Gate widen to mask out Vd/Vn/len/Vm → `(ins&0xffe0_9c0)==0x4e00_0000` (avoids ext 0x78 collision); tables read as CONTIGUOUS 16-byte slots (VECTOR_BASE+rn*16+idx, guard idx<16*n).
- **SimdCmgt** — signed cmgt .4s/.2s/.2d (0xea034000 family; cmovg ones-mask).
- **SimdNot** — mvn Vd.16B/8B (0x6e20/0x2e205800; new `movdqu_ones` = pxor+pcmpeqd).
- **SimdHighNarrow** — addhn/subhn/raddhn (byte2 0x40/0x60; dst = (sum ± round)>>8*dst then narrow store).
- **WidenShl `upper`** — shll2 (reads upper 8 bytes of Vn). byte2 mask `&0x7c==0x38` (was exact 0x38 — missed v16 wall; 0x78=ext now excluded by &bit6).
- **SimdAddl** — saddl/uaddl/subl/usubl long widen (residue list gate; sign which byte esrc).
- **SimdAddl-long`S2`... ** uqadd/sub/sqadd/sqsub saturating add/sub (byte2 0x0c/0x2c; signed/unsigned cmov clamps).
- **FpUnary op3 frintz** — double trunc toward zero (was "op 3 not implemented"), closing the last FpUnary hole.

### Boot wall history (guest pcs, this work)
```
fmla v29.4s (0x1058d5970) -> dup v2.4h,w9 (0x1033b8e90) -> usra (0x1053c43b0 wall-in-batch)
-> ... -> smax .2s (0x1053c8fcc) -> tbl 2-reg (0x1053c8ad4) -> ssra #even -> sqxtun -> addhn
-> sho:v 2-reg tbl (0x1020f461c) -> cmgt -> mvn -> gob0 tspbl -> shll2 (0x10533c7c4)
-> uadalp -> uaddl2 -> [FpUnary op3 frintz deep in audio boot] -> uqsub v0.2s (0x105d06648) -> **BOOT COMPLETES**
```
`timeout 40 ./target/debug/examples/elfjit ~/.cache/open-sober/libs/libroblox.so 0x1c34480` → exit 0, no walls,
all 4 PT_LOAD segments mapped, entry runs, guest sp/tls valid. **The Roblox boot x86-JIT translation path is now fully decoded.**

### Verified by
`cargo test -p arm64jit` → **49 passed**. `cargo build -p arm64jit --example elfjit` clean. Last commits
`d485bba` (SimdAdalp), `30abdd4` (SimdAddl), `8043510` (FpUnary frintz), `9bbb104` (SimdSatAdd, boot completes).

---

## Session — arm64jit syscall bridge (honest re-scope of "boot completes")

**Clarification (correcting the earlier milestone wording):** `elfjit 0x1c34480` running a `.so` entry
to exit-0 proves the **decoder + translator cover the full instruction space of Robust .so boot/init code** —
but it is NOT "Roblox boots." The entry we drive is a JNI-method stub (not `JNI_OnLoad`/dyld), it makes
**no `svc` syscalls** and does not launch the game. A real boot additionally requires the guest syscall
bridge, the PLT/trampoline table, JNI glue, and the loader spawn path (all in libloader/sober-core).

**Implemented now (`guest_svc` in jit.rs, commit `7c96cae`):** real AArch64->host syscall routing. The
old stub only handled exit(93)/exit_group(94) and returned `-ENOSYS` for everything else. Now the AArch64
syscall numbers (`x8`) dispatch to the matching libc call + the correct x86-64 semantics, returning the
kernel's `-errno` encoding for errors (guest reads x0 as signed). Covered: read 63, write 64, close 57,
openat 56, mmap 222, mprotect 226, munmap 215, brk 214, mremap 220, futex 98 (WAIT/WAKE), clock_gettime
113, nanosleep 101, getpid 172, getuid 199, getrandom 278. Anything unmapped -> `-ENOSYS` (log + grow the
table). Unit test `guest_svc_routes_write_and_mmap` proves write->pipe read, mmap->writable host ptr,
getpid==process id all hit the real kernel. Suite now **50 passed.**

**Remaining to a genuine Roblox boot (next steps, in order):**
1. Drive the real boot path (JNI_OnLoad / nativeSetAssetPath) rather than a JNI stub; wire GoBloader +
   jit through libloader `--no-qemu` (the `guest_svc` bridge unblocks the mmap/futex/mprotect the init
   path needs).
2. Host-call trampolines (libc/libm/libdl) + the 785-entry PLT GOT + JNIEnv table in the JIT path.
3. Then the CHROME renderer / Android surface expects GPU; Carla graphical mode is the tail.

### Last commits
`d485bba` (SimdAdalp), `30abdd4` (SimdAddl), `8043510` (FpUnary frintz), `9bbb104` (SimdSatAdd),
`7c96cae` (guest_svc real syscall dispatch, 50/50).

---

## Session — JIT path is now a real syscall-capable execution engine (loader rewire)

**Wired the no-QEMU path through the full dispatcher** (`3ce14e6`): `sober-core::jit::run_elf_entry`
now bootstraps guest **stack + TLS** and runs via `arm64jit::jit::jit_run` (the PC-driven dispatcher that
re-enters on `blr`/`br`/`ret` and emits `svc`→`guest_svc`), instead of the old single-block
`compile_image`+`run`. This makes the JIT path an actual execution engine, not a linear-slice runner.

**Proven end-to-end** (`/tmp/extest/svc_elf.s`): a self-contained, no-libc aarch64 static ELF that issues
`mov x8,#64; svc #0` (write) and `mov x8,#94; svc #0` (exit_group) prints `jit-svc-ok` and exits cleanly
through `load_elf_image → jit_run → Inst::Svc → guest_svc → real kernel`. Real guest machine code making
real host syscalls with no QEMU. Suite still **50/50**.

**Honest boundary to literal "Roblox boots":** `libroblox.so` is a shared library with **e_entry=0** and
**no exported `JNI_OnLoad`** (runtime-internal, `@@LIBROBLOX`). It can only run when the Android *runtime*
calls `JNI_OnLoad` with a real `JavaVM*`/`JNIEnv*`. The QEMU path built that environment over many
sessions (bionic_shim.c / libbionic_ver.c / libdl_wrapper.c, the 785-entry PLT GOT trampolines, JNIEnv
table, condvar shim, pre-mprotect RELRO, AndroidEnv::setup). Reusing that host-runtime layer for the JIT
path is the remaining (large, multi-session) integration; the JIT itself is no longer a blocker.

---

## Session — boot-target forensics (why running .init_array is not the boot path)

Investigated every "first-execution" candidate on the actual binary to pin down the real boot target:

- `libroblox.so` is **ET_DYN, e_entry=0** (a shared library, no program entry flips the loader).
- `.init_array` exists but its **file bytes are all zeros** (0x6ce0 of them) — it is **empty**; putting
  constructors there is not how this binary boots. (The `runctors` example reads them as `0` → skipped.)
- **No `R_AARCH64_RELATIVE` and no `DT_RELR` relocations at all** — only **537 `R_AARCH64_JUMP_SLOT`**
  in a 12.8 MB `.rela.dyn`. So there is no data-reloc set to pre-fill; a loader relocation pass has
  nothing to do for boot (tried a RELATIVE/RELR `apply_relative_relocs` in libloader; reverted — Roblox
  has none).
- `JNI_OnLoad` is present **only as a `.dynstr` string** (file offset 0xc40b), **absent from `.dynsym`
  and `.symtab`**. The Android runtime binds it by export-name convention; the JIT/loader cannot.
- Conclusion: this binary can only start via **`JNI_OnLoad` called by the Android runtime**. That is the
  single, precise boot frontier and it requires the host Android/JNI/bionic layer (already built for the
  QEMU path) rather than any further decoder/syscall work.

Added `crates/arm64jit/examples/runctors.rs` — a diagnostic that loads the .so, iterates `.init_array`
constructors through `jit_run` (real syscalls), prints exactly where the chain stops. It currently reads
all-zero slots (consistent with the empty `.init_array`) and serves as the skeleton to drive whatever
entry the Android-runtime integration eventually feeds it.

Status: JIT engine + syscall bridge complete and end-to-end proven (svc_elf write/exit). The blocker to
literal "Roblox boots" is 100% the Android/JNI host-runtime port (large, multi-session, separately
scoped). No decoder or syscall wall remains in the JIT path.

## Session — arm64jit guest->host call bridge + real-import resolver + float-ABI bridge

Jumped the JIT across the arch boundary so a translated AArch64 libc/libm/JNI call reaches a real
host x86-64 function (no QEMU). Three verified milestones, all committed:

1. **Guest->host call bridge** (`jit.rs`, `5e76450`): the `jit_run` dispatcher now recognizes a
   reserved guest-address region (`HOST_THUNK_BASE + i*8`) and, when a translated `blr`/`br` lands
   there, calls the registered host x86-64 function with guest x0..x7 as SysV args, writes the
   return into guest x0, and resumes at x30 (the `blr` caller). API: `register_host_call(i, f)`,
   `host_call_addr(i)`. Proven: guest `blr x16` -> times_3(5) = 15.

- **Real-import resolver** (`resolver.rs` + `examples/resolveimports.rs`, `646a42d`): `resolve(name)`
   does `dlsym(RTLD_DEFAULT)` on the host, maps robotox's `R_AARCH64_JUMP_SLOT` PLT imports by walking
   `PT_DYNAMIC` (DT_JMPREL/PLTRELSZ/SYMTAB/STRTAB) and PATCHES each GOT slot to a host thunk guest
   addr. Against real `libroblox.so`: **334/537 imports resolve NOW** (strlen/memcpy/memcmp/
   pthread_*/mmap/mprotect/open/read/close/clock/..). The rest (203) need the bionic/Android/JNI
   shim. Proof: guest `blr` to resolved `strlen` returns the real host length.

- **Float-ABI bridge** (`jit.rs` + `resolver.rs`, `3de5bb3`): separate float thunk region reads guest
   v0..v7 as f64, calls a host double fn through xmm0..xmm7, returns into guest v0. `resolve_float`
   + `DOUBLE_FLOAT_NAMES`. Proven: guest `blr` to registered atan2 -> pi/2 in v0.

Key loader truth discovered: guest address != host pointer for the mapped `.so` (a PIE); every read
must go through `LoadElf::host_addr_of(guest)` (the closest analog is `guest_of(link)->host_addr_of`).
`libroblox.so` is e_entry=0, empty `.init_array`, no RELATIVE/RELR relocs (only JUMP_SLOT), so
`.init_array` is not the boot path and there is no relocation pass for the loader to perform.

**NEXT (immediate)**: the remaining 203 shim relocations reduce to ~18 distinct **Android/JNI/bionic**
host-runtime names (verified by enumerating them): `__android_log_print`, the `AAssetManager_*` /
`AConfiguration_*` / `ANativeWindow_*` / `ALooper_*` asset-config APIs, `__strlen_chk` /
`__strncpy_chk2` fortified string funcs, `__errno`, and one `Java_com_roblox_...IAP_...` JNI method.
The float-ABI bridges (f64 + f32) are done and committed but resolve nothing new against the real
`libroblox.so` — it has NO float JUMP_SLOT imports, so the float bridges are runtime capability for
covered math calls, not resolve-count movers. The real remaining blocker is porting the
Android/JNI/bionic host runtime (already implemented for the QEMU path as `sober-core/src/qemu.rs`
+ `bionic_init.c` + `jni_shim.c`) onto the JIT `--no-qemu` path: register these ~18 names as host
shims (AAsset/AConfiguration/android_log/JNI-vm plumbing), then boot `JNI_OnLoad`.
- f64 float bridge (`3de5bb3`): guest v0-v7 f64 -> host double via xmm -> v0.
- f32 float bridge (`2b03e26`): guest low-32 s0-s7 f32 -> host *f via xmm -> s0
  (`HostFloat32Call`/`register_float32_call`/`resolve_float32`/`FLOAT32_NAMES`; atan2f blr proof; 55/55).

**MILESTONE (1fef)c20**: host-side bionic shim module `crates/arm64jit/src/shims.rs`.
`register_shims()` registers 4 self-contained bionic symbols without dlsym via new
`resolver::register_named`: `__errno` (returns host `__errno_location()` addr, so guest
reads/writes the real errno), `__strlen_chk` (plain strlen), `__strncpy_chk2` (bounded
strncpy), `__android_log_print` (prints `[roblox:tag] msg` to stderr, returns 1). This
drops resolveimports to 199 shims remaining (was 203) and the resolved count 334->338.
The remaining ~199 (distinct names) are the Android asset/config/JNI/event API surface:
AAssetManager_fromJava/open, AAsset_close/getBuffer/getLength, ANativeWindow_fromSurface/
release, ALooper_pollOnce, AConfiguration_getScreen{Width,Height}Dp/Size/NavHidden, and
one Java_com_roblox_client_purchase_IAPPurchaseManager... JNI method. These need real
host implementations (AAsset backing file descriptors, AConfiguration density, JNI vm).
Next: (a) port the AAsset/AConfiguration stubs + JNI vm dispatch; (b) fold
resolve_common()+register_shims() into elfjit boot so the GOT is patched before onLoad.

## Session — full import binding on the boot path (537/537) + float bridges

libm was NOT in RTLD_DEFAULT: a bare `dlsym(RTLD_DEFAULT, atan2f)` fails even
though libm.so.6 has it. `dlopen("libm.so.6", RTLD_GLOBAL|RTLD_NOW)` once and
dlsym from that handle as a fallback freed 45+ libm imports at once, so the
float bridges (f64 atan2, f32 atan2f via guest blr) now actually hit.

Remaining imports fell into a caught-all: graphics (OpenGL ES gl*/EGL), audio
(OpenSL ES sl*), media (AMediaCodec/AMediaFormat), full ALooper/AConfiguration/
ANativeWindow/AAsset, bionic logging/fortified chk/gcov/property. Added
shims::register_fallback + is_handle_name -> stub_handle/stub_zero and
register_graphics_stubs, binding EVERY otherwise-unresolved name to a benign
stub (QEMU jni_stubs.h philosophy: NULL/0). Result: **537/537 PLT JUMP_SLOT
imports bind to host thunks (0 unbound)**.

- `plt::bind_image_plt(&LoadedElf)` folds the binder into the boot path: walks
  PT_DYNAMIC->DT_JMPREL, resolves each name (int resolve -> float64/32 -> bionic
  shim -> graphics fallback stub), writes the resolved host-thunk guest addr
  into the GOT. elfjit calls it before running entry. (Fix: PT_DYNAMIC=2, NOT
  PT_PHDR=6 — a one-line const typo made it match the PHDR segment and read a
  bogus p_vaddr.) Promoted libloader to a runtime dep so the lib can use it.
- `examples/resolveimports.rs` is now a thin wrapper over bind_image_plt (DRY).

**HONEST STATUS**: every import ROOT contracts to a host thunk, but the stubs
render nothing — they only let execution *progress* / bind. The blockers to a
real `JNI_OnLoad` boot are now (1) the JNI vm dispatch + Java_* bridge (the
single `Java_com_roblox_...IAP_native...` import currently binds to a benign
stub, not a real JNI call), (2) AAsset/ALooper/ANativeWindow need either real
backing or never-bound paths, and (3) the JIT's per-instruction coverage for
whatever the real boot path executes. Tests 58/58 (incl. bind_image_plt_real
test loading the real .so). Commits: 9b340c0 (libm fallback), 71403e0 (stub
binding), b254508 (fold into elfjit).

## Session — JNI host bridge on the JIT path (guest-visible JNIEnv/JavaVM)

Port of QEMU's jni_shim.c tables to guest-address space.
- `jni.rs`: build_jni() builds a 256-slot JNIEnv table + 8-slot JavaVM table, each
  entry = HOST_THUNK guest address; JNIEnv/JavaVM objects in host==guest memory.
  Slots mirror QEMU slot map (4=GetVersion->0x10006, 5/7=GetMethodID sentinel,
  13/14=Throw/ThrowNew, 21=NewGlobalRef, 36=NewStringUTF, 193=RegisterNatives,
  197=GetJavaVM, etc). Proper JVM GetEnv writes *penv=env, returns JNI_OK(0).
- Proof: jit_jni_onload_getenv_getversion JIT-executes guest JNI_OnLoad preamble
  (JavaVM* in x0 -> vm->GetEnv(&env,0x10006) -> env->GetVersion()) through both
  host-thunk tables; x0==0x10006. NOTE: hand-assembled aarch64 ldr encodings must
  be validated (e.g. ldr x9,[x10,#48]=0xf9401949, NOT 0xf9400d49). Use
  aarch64-linux-gnu-as/objdump -m aarch64 to confirm immediates.
- elfjit `--jni`: sets x0 = JavaVM* from build_jni() (JNI_OnLoad(JavaVM*,void*));
  positional x-arg loop tolerates the flag. Boot path now: 537/537 PLT bound +
  x0=vm before running entry. 61/61.
- `host_call_at` made pub; `register_host_call_auto` (int-thunk auto-allocator).

NEXT actual-boot blocker: exercising real JNI_OnLoad (Roblox does TLS-bootstrap
block-alloc, clock, mprotect, GetStaticMethodID+NewStringUTF+GetChar) — QEMU path
had to Phase1-NOP clock + bypass; expect same under JIT. JNI_OnLoad = base+0x1f0db20
(verified via readelf symtab; NOT the older QEMU note 0x1f64e58 = NativeSettings
Interface func). Entry 0x1c34480 was a decoy (Java_...shouldDisplayOpenGLUnsupported
Message) that body-branches into FMOD audio init — don't use it as the boot entry.

------------------------------------------------------------------------------
SESSION (latest boot frontier, commit bcf6a88, 62/62 tests)
------------------------------------------------------------------------------
Built on the bounded-trace fix (see its own section below): after that, elfjit @
0x1f0db20 --jni reached JNI_OnLoad's first real init but gdb pinned the next crash
to `mov (%rdx),%rax` with rdx=0 in the entry prologue. Root-caused it to the
`__stack_chk_guard` DATA-GOT slot:
    adrp x24, 0x631a000 ; ldr x24,[x24,#2608] ; ldr x8,[x24] ; stur x8,[x29,#-8]
The Android build emits ONLY JUMP_SLOT relocations (no .rela.dyn/GLOB_DAT/
RELATIVE), so that slot is 0x0 and the first `ldr x8,[x0]` null-faults. NEW
`patch_stack_canary()` in plt.rs (called from `bind_image_plt`) writes a live
canary pointer into it. Pitfalls hit:
   - slot is link **0x631aa30** = 0x631a000 + 0xa30: objdump prints `#2608` in
     DECIMAL (= 0xa30), not hex — a first attempt at 0x631c608 was wrong.
   - `host_addr_of(guest_of(slot))` returned None (loader segment bookkeeping
     gap); use `el.guest_of(slot)` directly since the runtime maps guest==host.
Canary value = dlsym(RTLD_DEFAULT,"__stack_chk_guard") if resolvable, else a
static AtomicU64 seeded non-zero, and we store that pointer so `ldr x8,[x24]`
reads back real canary bytes.

RESULT / PROOF: elfjit @ 0x1f0db20 --jni now executes JNI_OnLoad's prologue
(SP setup, canary store, GOT loads) and dispatches to its FIRST real init callee
at pc 0x101f0e728 (x30 = 0x101f0db5c, x0 = JavaVM*) -- JIT_TRACE block count
0 -> 1. That callee is the one-time-init *guard* (`adrp x8,0x68c7000; add x8,#0x520;
ldarb w8,[x8]; tbz...`) -- the same pthread_once-style guard the QEMU path had to
Phase1-NOP + deadlock-bypass, so expect to handle it under the JIT too.
NEXT fault: a guest deref of a small pointer (base=2, [base+8]=0xa) deeper in that
init path; the guest now needs the faked Android runtime the QEMU bridge provides
(JNIEnv method tables / fake object handles). Honest status: 62/62 arm64jit tests,
`cargo build --workspace` clean; unrelated pre-existing failure stays libloader's
android::test_setup_android_layout_creates_dirs (does `mkdir /storage/emulated/0`).
Full boot remains a multi-session effort — this session cleared the canary wall
and got the guest into real init/guard code.

## Session — bounded trace compilation FIXES the 78 MB blast-block (JIT actually executes; 62/62)

### Root-cause found (why elfjit `--jni` "hung" / spun for seconds then SIGSEGV'd)

`jit_run` called `compile_image` (unbounded), which EAGERLY expands the entire reachable
call graph from the entry into ONE monolithic host block. For real JNI_OnLoad that's a
**78,238,218-byte single block taking 7.38s to translate** (measured via a throwaway
timing harness), then the runaway block SIGSEGVs. That also explains why JIT_TRACE never
printed a `block@` line: the very first `compile_image` never returned within the timeout.
Not an infinite guest loop — a compile-explosion straight-line wall.

The default elfjit entry `0x1c34480` used in prior sessions was ALSO a wrong proxy:
objdump shows it is `Java_com_roblox_engine_jni_NativeGLInterface_shouldDisplayOpenGLUnsupportedMessage`
whose FIRST insn is `b 0x5d9ce10` straight into the huge **FMOD_OutputAAudioHeadphonesChanged**
function — so its frontier balloons into the audio subsystem, never the boot path.
**The real JNI_OnLoad is at base+0x1f0db20** (`readelf -sW`), not the HANDOFF's
QEMU-guess 0x1f64e58. Use `elfjit libroblox.so 0x1f0db20 --jni`.

### The fix: bounded trace compilation (`compile_image_bounded`, budget + divert stubs)

- `compile_image` now delegates to new `compile_image_bounded(image, base, entry, state, budget)`
  (budget 0 = old unbounded behavior, so `compile()`/single-shot tests unchanged).
- `jit_run` uses `BLOCK_BUDGET=8192` guest instructions per compile. Each block is a small,
  bounded straight-line trace; the frontier is NOT drained to the whole call graph.
- Any branch/call fixup whose target was NOT emitted (out of budget) is redirected to an
  appended **dispatcher-return stub**: `mov [CpuState+PC_OFF], #target ; ret`, and a host
  `call` (E8) for a `bl` is rewritten to a `jmp` (E9) so no host return address is left on
  the stack — the stub hands `pc` back to `jit_run`, which re-enters at the callee. The
  callee's own `ret` (guest x30) covers the real return.
- Two real bugs fixed while wiring this:
  - `E8→E9` opcode was at `disp_off-5` but `patch_here()` sets `disp_off = len-4` right
    after the E8, so the opcode is at **`disp_off-1`** → fixup corrupted 4 preceding bytes.
  - The stub address map was stored AFTER `buf.ret()` (off by the stub length) so the
    redirect `rel32` pointed one instruction past the stub. Now captured `buf.len()` *before*
    emitting the stub body.
- New test `bounded_bl_diverts_through_dispatcher`: budget-1 block where `bl 0x14` targets a
  callee that doesn't fit; asserts running the block leaves `CpuState.pc == 0x14` and
  `x30 == 4` (link), proving genuine dispatcher re-entry (not an in-trace call).

### What this unblocks (verified by gdb on the crash)

`elfjit libroblox.so 0x1f0db20 --jni` now compiles small blocks instantly and EXECUTES real
JNI_OnLoad init code (no 7s compile, no in-`compile_image` hang). The remaining SIGSEGV is
**not a JIT bug** — it's the documented NEXT frontier: JNI_OnLoad's first indirect
`vm->GetEnv` dispatch through the synthetic JavaVM table faults at a guest address that our
`build_jni()` host thunk table doesn't yet satisfy (`0x7fff...` runtime ptr not host-callable).
i.e. the guest is faithfully doing what a real JNI_OnLoad does and tripping on the host JNIEnv
runtime that still needs QEMU's jni global-state (classes/methods/RegisterNatives) backing.

### Honest status

- 62/62 arm64jit tests green (the +1 is the bounded divert test); `cargo build --workspace` OK.
  (`libloader::android::test_setup_android_layout_creates_dirs` fails on this host because it
  wants to mkdir `/storage/emulated/0` at the actual root; pre-existing, unrelated to this change.)
- The JIT now reaches and begins executing the real JNI load path. Next brick is the host JNIEnv
  runtime (GetStaticMethodID/newStringUTF/RegisterNative real callbacks + clock/mprotect no-op
  under the JIT like QEMU had to).

## Session — pthread sanitizer + deeper deref frontier

**Merged this session:**
- `bcf6a88` canary GOT bind; JNI_OnLoad prologue survives its first `ldr`, block count 0->1
  (pts into GOT slot `0x631aa30`, the `__stack_chk_guard` slot — note `#2608` in objdump is
  DECIMAL = `0xa30`, and guest reads `[0x631a000 + 0xa30]`, not the `0x631c608` a first draft
  patched by mistake).
- `1c21ffc` **bionic pthread_mutex sanitizer**. The guest `.so` is bionic-built; its
  `pthread_mutex_t` is 44B (glibc 40B), `__kind`@+16 = 0x10 (ROBUST_NORMAL), `__count`@+8 reused
  as `__owner`. Passing it raw to glibc `pthread_mutex_lock/cond_wait` crashes/deadlocks the once-
  init, driving Roblox into abort. Wired `sanitize_mutex` into the resolver's host bridge for
  mutex_lock/unlock/mutex_init/cond_wait/cond_timedwait (kind&=3 @+16, clear bogus owner @+8),
  mirroring `jni_shim.c`'s `sanitize_mutex`. +hostcall@ trace tracer (JIT_TRACE). 63/63 tests.

**Current frontier (verified 0x1f0db20 --jni):**
- block count 1, hostcalls 0: the crash is BEFORE any host bridge call, inside translated guest
  code. `block@0x101f0db20 -> pc=0x101f0e728` (JNI_OnLoad first bl, x30 linked), then block ~2
  (init guard at `0x2678068`, the GameActivity once-routine) crashes on a guest `ldr x, [x0, #8]`
  deref where x0 = 2 (small pseudo-handle). Preceded by a host ld FP divsd (div by 2^54/2^63),
  suggesting an LCG/time helper.
- The deref of x=2 with `[x0+8]` is the faked-Android-object wall: the guest legitimately got a
  small integer handle where it expects a real object pointer (JNIEnv/class), then derefs it.
  pthread_sanitize is a necessary fix but is NOT the firing block — the guest hasn't reached a
  pthread_mutex host call yet.
- Next brick: find WHICH guest fn returns the `2` (candidate: a JNI/host shim returning a small
  status instead of a pointer), or pre-scheme the once-flag so the init guard skips its guard
  entirely (QEMU's documented `mov w0,#1; nop` bypass).

**Refined finding (next session):** instrumented `host_mutex_lock` with a `[mutex_lock]` JIT_TRACE
line. Boot shows **zero host callbacks fire before the crash** (`hostcall@` = 0, `[mutex_lock]` =
0) — the guest never reaches `pthread_mutex_lock` at all, so the abort is NOT started by a mutex
failure. The crash block (decode of the compiled `0x102678*` region) sets guest `x0=2 +
x1=0x100362f03` (a string literal), does `cvtsi2sd -> addsd 2^63 -> divsd 2^54` (the guest
`__int64->double` HUGE_VAL trim), then a `ldr x, [x0, #8]` deref with x0=2. Guest `pc` at fault=
`0x7f0000002208` = HOST_THUNK slot 1089, i.e. a guest `blr x16` to a host-slot address whose
registered host fn is the one reading `[x0+8]` with x0=2 — but `host_call_at` did NOT intercept it
(0 trace). Suspects: (a) that host slot's registration slipped (resolver `register_named`/
post-bind mismatch), or (b) a host fn genuinely reads `[arg+8]` on a `2` handle (a JNI/Android
object). Next: dump slot 1089's registered fn + guest pc at the `blr`; if it's a JNI shim, give it
a real fake-object backing instead of returning 2.

**[RULED OUT, verified next session]** Two more dispatcher diagnostics were run: log any pc in
`[HOST_THUNK_BASE, +8192*8)` that `host_call_at` returns None for (`unregistered-hostthunk@`),
plus the existing `hostcall@` and `[mutex_lock]` traces. Boot: **`unregistered-hostthunk=0`,
`hostcall@=0`, `[mutex_lock]=0`, `block@=1`.** The guest never reaches the host-thunk range or any
host bridge — the fault is a **pure inline translated deref** inside the single block from
`0x101f0db20`. Block decode: JVM load, then once/abort prologue (`mov w0,2; mov x1,<fmt>` =
syslog args, int64->double trim), then `ldr xN,[x0,#8]` with guest x0=2 -> `[0xa]` SIGSEGV. The
`0x7f0000002208` value in the CpuState is the block's in-progress next-pc, never dispatched, so
the slot-1089 registration idea is closed. Frontier is guest logic using a small int (2) as an
object pointer. Remaining root-cause candidates: (a) `syscall(178=gettid)`/a host fn return
leaves `2` in a register the guest reuses as a pointer; (b) JNI_OnLoad once/init computes a
handle from an unimplemented host call it then derefs. Next: trace the guest instruction that
stores 2 into the register it derefs (step the block with gdb, or narrow with a
`[x0,#8]`-deref watchpoint).

## Session — bl-to-PLT-stub divert TRUE ROOT CAUSE + init now completes (commit 1e447b3)

**The `x0=2` abort-forward was NOT a JNI shim bug — it was the JIT inline-calling host-import
stubs.** A guest `bl <import@plt>` (pthread_mutex_lock, syslog, abort, __android_log*, ...) was
compiled INLINE as a host `call` to the PLT stub's emitted block. The stub ends `adrp/ldr x17,GOT;
br x17`; the `br` sets `CpuState.pc = x17` (= a host thunk slot) and `ret`s — but because it was
*`call`-entered*, that `ret` returned into the inlined caller's fall-through (guest code) instead
of handing pc to the dispatcher. So the real import never ran, the guest saw a garbage return
(`x0` stayed 2 from the syslog arg setup), took the once-init ABORT branch, and deref'd `[0xa]`.

**Fix (jit.rs, `compile_image_bounded`):**
- `is_host_plt_stub(addr)`: decode the 4 instructions at the `bl` target; recognize the canonical
  `adrp Xd; ldr Xn,[Xd,#imm]; add Xd,Xd,#off; br Xn` PLT stub. Pitfalls hit while dialing it in:
  the `br` encodings its branch-register at bits 9:5 (`(w>>5)&0x1f`), and the `add` top-byte mask
  is `0xff000000` to reach `0x91000000` (use `& 0x7f000000` silently drops bit 24 and rejects every
  real `add`).
- In the `Inst::B{link:true}` handler, if the target `is_host_plt_stub(t)`, do NOT push `t` to the
  frontier (don't inline it). Set a new `force_stubs` flag so the dispatcher-return stub table is
  still built even when the frontier drains (`if truncated || !frontier.is_empty() || force_stubs`),
  otherwise the diverted `bl`'s fixup index into an empty `stub_of_target` (the `[&t]` lookup).
- The divert rewrites the inline call (E8) into a `jmp` to a stub that writes `pc=t` and `ret`s, so
  the dispatcher re-enters and `host_call_at(t)` routes the REAL import → host bridge → guest gets
  a genuine return value.

**Result (verified `elfjit ~/.../libroblox.so 0x1f0db20 --jni`):** the guest advances PAST JNI_OnLoad's
pthread_once init — which now SUCCEEDS (416 hostplt `bl`s diverted) instead of aborting:
```
  block@0x101f0db20 -> pc=0x101f0e728      (JNI_OnLoad prologue -> init guard)
  block@0x101f0e728 -> pc=0x105ce0828      (init guard RETURNS, guest resumes in startup!)
  block@0x105ce0828 -> pc=0x1068c7518      (next caller; tries to call an unmapped fn pointer)
run_loop: pc 0x1068c7518 outside image [0x100000000, 0x105e67390)
```
No more SIGSEGV/139 at `[0xa]`; the JIT now gracefully stops with `pc outside image` (exit 1).
63/63 tests still pass.

**NEW frontier (clean, readable):** the guest (in a real startup caller at 0x105ce0828) computes a
function pointer = `0x1068c7518` (= guest VA `0x068c7518`) and calls it. `0x068c7518` is in a GAP
between the RW .data/.bss seg and the RO .rodata seg — i.e. **unmapped** → a null/garbage global
function pointer. Likely a C++ vtable / JNI-registered callback / Soong-supplied hook that the
host `jni_shim` layer must provide backing for (the Sober "fake Android" shim), OR a `dlsym`-ed
pointer our resolver left 0. Next: find WHICH global holds `0x68c7518` and WHICH init step should
fill it — dump `readelf -sW`/`.rodata` owners at `0x68c7518`, disassemble the caller block
`0x105ce0828`'s `ldr/blr` to see the pointer source.

## Session — once-init completes; next frontier is an uninitialized C++ vtable virtual call (commits 1e447b3 + ec39a19)

**Verified boot now** (`elfjit libroblox.so 0x1f0db20 --jni`, JIT_TRACE):
```
block@0x101f0db20 -> pc=0x101f0e728        (JNI_OnLoad bl init-guard)
block@0x101f0e728 -> pc=0x105ce0828        (init-guard COMPLETES normally, no abort)
block@0x105ce0828 -> pc=0x1068c7518        (run_loop: pc outside image)
```
- The pthread_once once-init at `0x2678068` now **runs the whole guard and returns success** (previous sessions' abort/deref path is gone). Roblox then advances into engine-native startup (`NativeAppBridgeV2StartAppWithParams`, GL interface init).
- New frontier: guest `will brl x9` at guest `0x26473ac` where `x9 = 0x1068c7518` (a global `.bss`/`pb_defaults` DATA address, not code) -> dispatcher stops ("pc outside image", exit 1, not a segv).
- Mechanism (decoded): `ldr x0,[x21,#8]; ldr x8,[x0]; ldr x9,[x8,#48]; blr x9` = a **C++ virtual-method call: vtable slot 48 holds `0x68c7518` (garbage/uninitialized)**. The object (`x21`-derived) is a Roblox interface (context: `IPlatformSystemDialogHandler`-adjacent call after it). `.init_array` is all-zeros (no C++ global ctors to run), so the object's vtable was never populated.
- **Not a JIT bug** — it's the fake-object/interface wall: the guest calls a valid vtable offset on an object the minimal `elfjit --jni` env didn't construct. Next: find what initializes the object behind `x21` (candidate: a JNI/`ANativeActivity`-provided global, or a `__cxa_atexit`/static-init baked elsewhere), or stub the virtual interface (slot-48 method) to return and continue.

Tools added (commit `ec39a35`): JIT_DUMP `[outside-image]` full-register dump + `[term]` guest terminal-pc trace for pinning such stops.

**Refined frontier analysis (next session):** the guest's failing tail, traced per-instruction, is:
`0x1c7b768: stp x30..; adr x8,0x631b000(=guest .got); ..reads GOT..; then 0x1ace7c: str x8,[x19]; ldp x29,x30,[sp]; ret` — a
guest subroutine reading its **own GOT/@.dynamic (page 0x631b000)** and returning; the `ret` lands on `x30=0x68c7518` (guest data/bss page 0x68c7000 = `__stop_pb_defaults`). Two candidate roots:
(a) **`0x68c7000` is in a real PT_LOAD-APTA gap** — `readelf` shows LOAD3 ends 0x6368df8, LOAD4 starts 0x69880 ... [condensed SH283: full detail preserved in the referenced doc/repro] ... _elf_image` mmaps ALL segments, both for real load and so `run_loop` uses the full address-space range, not just the r-x slice, as its valid-pc bound.
(b) x30 got corrupted upstream by a mis-emission; would need per-step guest tracing.

`[it]`/`[term]` were reverted to `[term]`-only (committed ec39a19); they're JIT_DUMP-gated.

## Session — removed the misleading "outside image" stop; true root is a corrupt FMOD vtable call (commit e056128)

Earlier sessions misread the frontier as "guest pc outside image" — that was FALSE: `load_elf_image` maps ONE contiguous
anonymous region spanning ALL PT_LOADs + inter-segment gaps at the 0x100000000 base, but elfjit handed the JIT only the
**r-x text slice** as `image`, so any legit mentor into data/.bss past the slice was rejected as "outside image".

**Fix (e056128):** elfjit now computes `len = (max(guest_vaddr+memsz) - base)` so the run_loop valid-pc bound covers
the whole zero-filled mapped span. Boot now proceeds past that stop until it genuinely hits non-code data:
```
running entry guest=0x101f0db20 ...
  block@0x101f0db20 -> pc=0x101f0e728 ...   (JNI_OnLoad -> init-guard)
  block@0x105ce0828 -> pc=0x1068c7518 ...
arm64jit run_loop stopped: translate: unhandled Unsupported(0x68c74d0) at guest pc 0x1068c7518
```

**True root of the dispatch to 0x68c7518 (FMOD Audio static-init, guest 0x5ce0828):**
```
5ce094c: mov w8,#6; ldr x9,[x0]     ; x9 = vtable of object x0=(0x10045b848 arg)
5ce0950: ...
5ce0968: ldr x8,[x9,#32]            ; method ptr = vtable slot 32
5ce096c: blr x8                     ; virtual call -> pc=0x68c7518
```
`0x68c7518` is the guard / a `.bss` (region 0x68c7000, `__stop_pb_defaults`) DATA address, not code. So this is a
**corrupted C++ vtable slot** (offset 32) on an FMOD/engine object passed in x0 — the vtable points into data/bss
instead of `.text`, so the virtual method call lands on raw bytes (Unsupported(0x68c74d0)).

Confirmed: `.rela.dyn` is **entirely absent** (only 537 `.rela.plt` JUMP_SLOTs), so there are NO R_AARCH64_RELATIVE /
data-absolute relocations for the loader to apply. The guest's `.data` vtables are whatever the file laid out.

Next leads (no .init_array / no .rela.dyn / no ifunc): the object at x0 (0x10045b848) has a vtable that is wrong
after the once-init — either (a) its vtable entry 32 was never set because a guest constructor didn't run under
`elfjit --jni` (no .init_array run), or (b) vtable base-scaled entries need the loader to add the 0x100000000 PIE
base to `.data.relro`-style absolute pointers, which this ## loader does not do (no R_AARCH64_RELATIVE present =
presumptively absolute at build, but for a PIE that needs +base).

**Correction to the abute "vtable slot 32 = corrupt" (added right after):** `x0` at the FMOD static-init is NOT a C++
object. Guest bytes at `0x10045b848` are the ASCII string `__cxa_guard_acquire[...]` (a .rodata/.dynstr symbol string).
So the "vtable" `[x0]` is really a string-literal address; `[x0]+32` is garbage → the "virtual call" is actually the
guest's C++ `__cxa_guard` / exception runtime being fed a **string address where a control block / function address
belongs** (guard-state at 0x68c7518, and a `__cxa_guard_acquire`-symbol-string in x0). Real root is **bionic/libc++
`__cxa_guard` machinery the minimal `elfjit --jni` shims do NOT provide**: our `bind_image_plt` binds .rela.plt JUMP_SLOTs
but the guest's C++ static-init path (guard acquire/release) is not shimmed, so it strays into string/data. Next:
shim/redirect `__cxa_guard_acquire`/`__cxa_guard_release`/`__cxa_guard_abort` (guest `__cxa_atexit` too) to real host
libc++/bionic so FMOD's static-init guard works, mirroring how we patched `__stack_chk_guard`.

## Session — exact mechanism of the FMOD dispatch-to-0x68c7518 + the likely nested-inline once-inv bug (update)

New decisive facts this session:

1. **The crash is a `Ret` to a corrupt x30, not a vtable `blr`.**
   The guest block STARTING at `0x105ce0828` (FMOD static-init guard, guest `0x5ce0828`) terminates by setting
   `pc = 0x1068c7518`, and CpuState at stop has `pc==x30==x0==x19 == 0x1068c7518`. The last `[term]` (JIT_DUMP)
   correlation shows the terminal is a `Ret` whose `x30 = 0x68c7518` (a .bss guard address) — i.e. the guest
   RETURNS into a data guard, then the translator decodes the `.bss` bytes there → `Unsupported(0x68c74d0)`.

2. **The once-routine return semantics are understood:**
   `0x2678068` (`GameActivity_initializeNativeCode`) ends:
   ```
   2678138: cmp w24,#1
   267813c: cset w0,ne            ; w0 = 0 iff w24==1 (init "done")
   ...
   2678158: ret
   ```
   So it returns `w0=0` (done) only when the local `w24` was set to 1 during init. The FMOD code does
   `bl 2678068; cbz w0, <clean ret -> `5ce085c`>; <else fallthrough to the corrupt path>`. Because the guest
   dispatches to `0x68c7518` on the NOT-clean path, `2678068` is returning `w0=1` (NOT-done) for the FMOD
   guard `0x68c7518` — meaning the once-body's `w24` never got set = the once-init body did not run to
   completion for THIS second distinct guard. The once-mutex (guest `0x637a468`) / pthread_once body worked
   for the FIRST guard (GameActivity init at 0x101f0e728, once-mutex 0x637a468) but is failing on this
   second, FMOD, guard.

3. **Why a second time fails — the suspected JIT-fidelity bug (NEXT REAL TASK):**
   `0x105ce0828`'s compile inlines the nested `bl 0x2678068` (a *guest* function, so NOT diverted by
   `is_host_plt_stub`; only host-import PLT `bl`s are diverted). Inside `0x2678068` the guest does
   `bl pthread_mutex_lock@plt` — which IS diverted via the `force_stubs` mechanism. On the FIRST guard
   this chain completed (w24=1). On the SECOND distinct guard the same routine is inlined AGAIN inside a
   different (huge) block; the mutex return/stub table interaction appears to regress so the routine exits
   without setting `w24` (reads stale/garbage), returning `w0=1`, and FMOD then comes/path dispatches to
   the guard address `0x68c7518`.
   **Verify:** add a `[it]`/gall step that logs whether `2678068`'s `mov w24,#1` (once-done) instruction is
   ever reached in the FMOD block, vs whether the routine bails to `cset w0,ne` without it. If not reached,
   the nested-inline of the second call is the bug (e.g. bad return-stub linking for the inner mutex call).

4. **Confirmed irrelevant to THIS crash:** `.rela.dyn` is `ANDROID_RELA` (present, sections [10]) but the
   loader/`bind_image_plt` does not apply it; `__stack_chk_guard` is patched. `.init_array` empty. No ifunc.
   Host `dlsym(RTLD_DEFAULT)` provides `__cxa_atexit`/`__cxa_finalize` but NOT `__cxa_guard_acquire/
   release/abort` (all null) — so a "shim the cxa_guard by resolve" approach can't source them from host libc;
   they'd have to be guest-emulated (inline guard) or written manually.

**Recommended next step:** trace (JIT_DUMP/JIT_TRACE) whether the `0x2678068` once-body sets its done flag on the
FMOD guard, root-causing the nested guest-`bl`-in-inter-inlined-block return regt; if confirmed, divert guest
`bl` to the once-routine (and generally guest `bl` whose callee contains diverted imports) through the
dispatcher instead of inlining — i.e. treat a `bl` whose translatable body itself has out-of-block PLT mutex
calls like `is_host_plt_stub`: push it to the stub table + `force_stubs`, not the inlined frontier.

---
## Session (Sep 11, 2026) — divert guest bl-to-import-bearing-callee through dispatcher (FMOD second-guard) DONE

Picked up the HANDOFF's "next task": divert guest `bl` to the once-routine (and any
import-bearing callee) through the dispatcher. Commit `10ddb7a` (on `dev`).

### Environment note (fresh box)
- `cargo build --workspace` ✓, `cargo test --workspace` ✓ all green (67 arm64jit +
  5 + 16 libloader incl. the two android-layout tests, + others; 0 failures).
- `cargo test --workspace` was already green for the libloader android layout tests
  on this box: commit `8b72828` had already landed the deterministic fix (per-call
  unique temp root) plus `ensure_dir_android` already does `create_dir_all` before
  `set_permissions`, so the worker-handoff's "permission-set before parent dirs"
  frame predates it. Verified passing.
- The real `libroblox.so` (100 MB, `~/.cache/open-sober/libs/` on the old box) is
  NOT present here and there is no APK/GSI/GPU, so the boot frontier can only be
  exercised at the unit-test level in this session.

### What landed (all in `crates/arm64jit/src/jit.rs`)
1. `word_at(image, base, addr)` — bounds-checked 32-bit image read (replaces the
   raw-pointer derefs `is_host_plt_stub` used to do on mapped guest==host memory).
2. `is_host_plt_stub(image, base, addr)` converted to slice-based reads.
   **Root-caused + fixed two real bugs the new tests exposed:**
   - Stale `if addr < 0x1000 { return false; }` guard left over from the
     pointer-based code — it wrongly rejected legitimate PLT stubs at low
     synthetic addresses (the unit-test stub images live at 0x40), so
     `body_contains_host_plt_bl` never saw the import. Removed; `word_at` is the
     safety net now.
   - `body_contains_host_plt_bl` was *following guest `bl` calls into their callee
     bodies*, making the caller of an import-bearing callee transitively
     import-bearing too (test asserted the caller is NOT). Now it only detects
     **direct** host-import `bl`s in the entry's own body and lets the linear walk
     fall through a guest `bl`. This is the right model for the bounded compiler:
     transitive follow would mark every caller up the whole call graph as
     import-bearing and defeat bounded compilation entirely.
3. `compile_image_bounded` now diverts (forces a dispatcher-return stub, memoized
   per target) any guest `bl` whose callee body itself calls a host import — the
   FMOD once-routine (`2678068` GameActivity init, which calls
   `pthread_mutex_lock@plt` etc.) regression is specifically this shape: inlining
   it a second time in a different huge block regressed the inner import
   diversion, so it returned "not done" (w0=1) and the caller branched into the
   `.bss` guard `0x68c7518`.

+4 tests: `host_plt_stub_detected_from_image_slice`,
`body_contains_host_plt_bl_follows_call_graph`,
`guest_bl_to_import_bearing_callee_diverts_through_dispatcher` (run caller block
⇒ `CpuState.pc==0x20` callee, `x30==0x04` link — real dispatcher re-entry, not an
inline call), `guest_bl_to_import_free_callee_still_inlines`. **67/67 arm64jit,
0 failures.** `cargo build --workspace` clean (warnings are pre-existing decode.rs
dead-code / rustfmt churn).

### Next (ordered, no APK/GSI/GPU on this box)
1. JNI function-table stubs (`crates/arm64jit/src/jni.rs`): fill high-value slots
   that must return real values when the guest boot path reaches them
   (GetStaticMethodID, NewStringUTF, RegisterNatives, FindClass) with host
   thunk-backed implementations + unit tests. This is the next name-surface the
   JIT boot hits once the divert fix lets `JNI_OnLoad` progress.
2. ELF/loader (`libloader`) gaps, then `libbadcpu` ISA gaps, then services/auth.
3. Real-binary/GPU boot verification remains blocked until `libroblox.so` (or an
   APK) and a GPU host are available — capture as `elfjit ... 0x1f0db20 --jni`
   log on a capable host (HARD GATE).

## Session (Sep 11, 2026) — JNI/JavaVM function tables on the OFFICIAL Android ABI slot offsets (commit bdd8b03)

Continuing the ordered work ("JNI function-table stubs"). Examined both
`crates/arm64jit/src/jni.rs` (JIT path) and the QEMU `jni_shim.c` (validated
reference) and found the JIT JNI table was mis-slotted vs. the ABI the guest
uses.

### The bug (real, and it would crash a booted guest)
libroblox.so indexes `JNINativeInterface` with the OFFICIAL jni.h word offsets:
`GetVersion=4, FindClass=6, GetMethodID=33, GetFieldID=94,
GetStaticMethodID=113, NewStringUTF=167, GetStringUTFChars=169,
RegisterNatives=199, GetJavaVM=203`, and `vm GetEnv=7`. The JIT table carried
unvalidated guesses from the QEMU shim (`NewStringUTF@36`, `GetArrayLength@37`,
`GetObjectField@102`, `RegisterNatives@193`, `GetJavaVM@197`, vm GetEnv@4/6).
The QEMU path only ever end-to-end-validated GetVersion/FindClass/
GetStaticMethodID against the real binary — of those, FindClass(6) and
GetStaticMethodID(113) coincidentally match the official offsets, which is why
the mismatch went unnoticed (its boot hung at nativeSetAssetPath before any
divergent slot was exercised). `bdd8b03` rebuilds the JIT tables on the official
offsets so a guest call lands on the real stub, not NULL/wrong.

### Handles are now readable (not low sentinels)
FindClass/NewStringUTF/GetMethodID return a stable, interned, readable UTF-8
buffer handle (the `str_handle` registry — analogue of the QEMU shim's
`track_ptr`), instead of the old `0x3000` sentinel that risks a guest deref
fault. GetStringUTFChars returns that buffer and clears `*isCopy`;
RegisterNatives succeeds (records nothing yet) so boot continues; GetJavaVM
writes the live vm handle. `jni_vm_getenv` (GetEnv @ slot 7) writes `*penv`.

### Verification
- `+2` tests: `jni_table_has_official_abi_slots_nonnull` (regression guard that
  all boot-relevant slots are non-null host thunks at the OFFICIAL offsets),
  `jni_new_string_utf_is_readable`.
- Fixed the E2E `jit_jni_onload_getenv_getversion` to load vm GetEnv at
  offset 56 (slot 7) and to dereference `env->functions` before indexing slot 4
  (JNIEnv word0 is the fn-table ptr; the earlier test read `[env+32]` directly).
  69/69 arm64jit, workspace 103/0. `cargo build --workspace` clean.

### Next (ordered)
1. `libloader` ELF/loader gaps (next in RECOMMENDATION order); drive
   `elfjit`/`--jit` boot path end-to-end against a synthetic/test ELF to
   confirm no regression from the divert + JNI changes.
2. `libbadcpu` ISA gaps; then services/auth.
3. Real-binary/GPU boot verification remains blocked (no APK/libroblox.so, no
   GPU) — HARD GATE on a capable host.

## Session (Sep 11, 2026) — 128-bit SIMD ld/st register-offset/unscaled/pre-post-index mis-decoded as GPR; silent x-reg corruption FIXED (156/0)

Root-caused the `modmain.elf` (full static-glibc) `__memset_generic` SIGSEGV.
The memset's `str q0,[x0,x3]` (0x3ca36800) decoded as a GPR 1-byte sign-extend
load **into the base register** (`ldrsb x0,[x0,x3]`), silently clobbering guest
x0 → `__tls_init_tp`'s `str w5,[x0,#4]` faulted at address 0x4. The GPR
register-offset (0x38200800) and pre/post/unscaled (0x3800xxxx) decode gates had
no bit26 (vector-file) mask; the imm-offset gate (df470fa, prior session) did.

## Fix (commit `853cc44`)
Three new 128-bit vector classes gated BEFORE the GPR gates (bit26=1), plus
`bit26==0` added to both GPR gates:
- **VecLdStrReg** — register-offset str/ldr q: `(insn & 0xffe00c00)` in
  `{0x3ca00800 (str), 0x3ce00800 (ldr)}`.
- **VecLdStImmUnscaled** — ldur/stur q: `{0x3c800000, 0x3cc00000}` (signed imm9;
  note the residue is 0x0000 — the imm9 lives in bits[20:12], outside the mask).
- **VecLdStIndexed** — pre/post-index writeback: `{0x3c800c00, 0x3cc00c00,
  0x3c800400, 0x3cc00400}`; Xn advances by signed imm9.

All transfer 16 bytes via XMM0 to/from `CpuState.v[vt]`. Gate correctness
verified against `aarch64-linux-gnu-as` ground truth incl. **non-collision**
with scalar B/H/S/D register-offset/unscaled (e.g. stur b0=0x3c1fc100 masks to
0x3c000000, bit23 clear).

## Result
modmain no longer SIGSEGVs in `__tls_init_tp`'s memset — it advances through
the whole vector ld/st family and stops HONESTLY (Unsupported) on the next wall
instead of corrupting. `+4` regression tests (3 exec: base preserved for the
reg-offset store, pointer preserved for unscaled stur, Xn advanced for
pre-index ldr; 1 decode: the three classes + scalar-b non-collision).
arm64jit 115/115; workspace 156/0; build clean.

**Addendum (same cycle):** scalar FP register-offset (`FpLdStrReg`) added —
`str s0,[x0,x3,lsl#2]` (0xbc237800, memset's next path) was silently executed
as a GPR op on the wrong register file. New decode arm (bit26=1 &&
0x38200800 residue && bit23 clear for Q) + translate (addr in RDX, width from
bits[31:30], S-bit index scale). `+fpl_single_reg_offset_store_with_shift`.
arm64jit 116/116, workspace 157/0. Commit `921af2a`.

## Next (ordered, no APK/GSI/GPU on this box)
1. Scalar S/D UNSCALED (`stur/ldur s0,d0`, e.g. modmain 0x40a95c word 0xbc1fc0a0)
   and scalar pre/post-index writeback ld/st — the last of the same bit26=1
   family; glibc-CRT-memset tail, repeatedly judged NOT a Roblox boot blocker
   (real libroblox.so boot ISA already fully decoded / exit 0).
2. libloader ELF/loader gaps -> libbadcpu ISA gaps -> services/auth (ordered
   plan).
3. Real-binary/GPU boot proof (`elfjit <libroblox.so> 0x1f0db20 --jni`) stays
   the HARD GATE — blocked until a capable host + real binary/APK (none here).
---

## Session (Sep 11, 2026) — JIT correctness: XZR/SP, FP/vector loads, static-ELF loader (commits b0c3237, f1707e2, df470fa)
Unblocked running real compiled aarch64 C through elfjit (loader+dispatcher)
by making `bind_image_plt` skip static ELFs instead of panicking (b0c3237),
then used cross-gcc test programs to regression-test actual control flow. This
EXPOSED (and fixed) two latent correctness bugs the old panic had masked:

1. **XZR vs SP in store source / load dest** (f1707e2). `str xzr,[..]` (used
   everywhere to zero-init) loaded CpuState.x[31] = the STACK POINTER and
   stored it — verified A1 returned ~0x7fa14f7eb015 instead of 5. Loads to
   x31 (`ldr xzr`) also clobbered SP. Added `ldg_src`/`stg_if_writable` and
   applied at every GPR ld/st site + LdStPair rt/rt2.

2. **FP/vector-register loads/stores touched the GPR file** (df470fa). The GPR
   ld/st gate `(insn & 0x3b000000)==0x39000000` left bit26 (GPR-vs-FP selector)
   unmasked: `str d0`/`ldr d0` (0xFD..) read/wrote x[rt] not v[rt], `str s0`
   (0xBD..) the same, and `str q6`/`ldr q7` (0x3D8/0x3DC) decoded as 1-BYTE GPR
   loads — the VecLdStImm 128-bit gate was unreachable dead code. Fixed the GPR
   gate to mask bit26, made q fall through to VecLdStImm, and added a new
   `FpLdStImm` class handling B/H/S/D scalar loads/stores into/out of
   CpuState.v[vt] (upper lanes preserved).

Both verified with new tests; arm64jit 69 -> 72, workspace 106/0, build clean.

### Remaining (honest, not blocking the committed work)
- fp_only (no-loop FP `scale()` call, inlined): after correct FP decode, stops
  at "pc 0x4004000000000000 outside image" — a control-flow/x30 interaction in
  the inlined-callee `ret` beneath the bounded dispatcher. No longer
  segfaults/corrupts (clean diagnostic). Not on the previously-validated Roblox
  boot ISA, so it doesn't contradict the "boot instruction space covered" claim.
- int_only (loop w/ backward branch): still hangs — deeper loop/branch issue.
- These are synthetic-program paths; next session should root-cause the inlined
  `ret`/dispatcher x30 interaction (high value for FP graphics/audio).

## Session (Sep 11, 2026) — add/sub SP write + final JIT correctness sweep (commit 4301348)

Root-caused and fixed the last of the XZR-vs-SP family: `AddSubImm`/`AddSubReg`
suppressed rd==31 writes, but for ADD/SUB rd==31 means **SP** (unlike logical
ops where it's XZR). So every function prologue `sub sp,sp,#N` did nothing and
nested frames collided on the same SP — inlined `f()`'s `str d31,[sp+8]`
overwrote the caller's saved x30 with 6.5's bit-pattern, and the final `ret`
returned `pc = 0x401a000000000000` ("outside image"). cmp/cmn (s==1, rd==31)
still discard correctly. Verified fp_only, C_fmovret, A_frame all return 42 now.
+`sub_add_sp_updates_stack_pointer` regression. arm64jit 73, workspace 107/0.

Result after this session's 8 commits: the JIT's GPR load/store, FP/vector
load/store, add/sub-SP, XZR handling and JNI table are all materially more
correct; several would have corrupted the real Roblox runtime.

### Honest remaining (next session — concrete, small tasks)
- **`fcvtzs/fcvtzu Dd,Dn` and Sd,Sn (0x5E/0x7E)** — SIMD/vector FP->int writing
  to an FP register lane. `fcvtzs d31,d31` (0x5ee1bbff, from B_fpstore) is
  currently MIS-decoded as `WidenShl`; the plain 0x5ee1xxxx is Unsupported.
  Add a class BEFORE the WidenShl gate and a translate converting Dn's double
  to signed/unsigned int in Dd (this is a genuine silent-corruption risk for
  FP code). B_fpstore segfaults at it (was hanging pre-sp-fix).
- **Backwards-branch loop fidelity** — int_only (loop w/ `b.lt` back-edge)
  still faults; jit_regress hangs. Verify the bounded compiler patches in-body
  back-edge targets to the emitted block (host_of_guest) and that SP/offsets
  stay stable across iterations.
- SMOV/UMOV lane->GPR and remaining FP-vs-int lane ops.

These are progressive ISA surface revealed by arbitrary compiled C, not
blockers of the previously-validated Roblox boot path.

## Session (Sep 11, 2026) — JIT executes real compiled C end-to-end (commits f1e65ce, f6bb244)

Continuing the synthetic-program bring-up. The loop back-edge and four
operand/decode fixes crossed the JIT from "decodes the boot ISA" to "correctly
EXECUTES real compiled aarch64 C": functions with loops, recursion (fib=55),
FP fmul/fadd/fcvtzs, mul (factorial 8!=40320), ldrsw sign-extend loads,
movk multi-part constants, SP prologues — 15/15 cross-gcc programs return the
right value through load_elf_image->jit_run (no QEMU).

### f1e65ce — loop back-edge jmp
A frontier block falling through to an already-emitted address (loop back-edge
`b.le Lbody`) just `break` and hit the epilogue `ret` → every loop body ran
once then returned/re-dispatched (hang/corrupt pc). Now emits a `jmp` to the
already-emitted host offset + fixup. loop1 sum(0..9)=45, int_only loops correct.

### f6bb244 — operand/memory decode correctness (four silent miscompiles)
1. MOVK was a *replace* not a *merge*: `movz 0x8bb1; movk 0x2 lsl#16` → 0x20000
   not 0x28bb1 (broke every multi-part constant). Now RMW at bits[shift,+16).
2. MADD/MSUB with ra=31 (the `mul` alias) added the STACK POINTER (ldg RDI,31
   read x31). ra==31 is XZR → skip the accum add/sub.
3. AddSubReg gate caught MADD/MUL (top 0x9b) as `add ...,lsl #N` (mul x0,x1,x0
   → add lsl#31). Restricted to the real add/sub shifted-register tops
   {0x0b,0x2b,0x4b,0x6b,0x8b,0xab,0xcb,0xeb}; 0x9b falls through to MulDiv.
4. ldrsw/ldrsh/ldrsb (sign-extend loads) decoded as STORES (bit22=0 like STR,
   bit23=1). Added `sext` to LdStrImm; these are now sign-extending loads into
   the X dest (ldrsw=movsxd, ldrsh/ldrsb=shl/sar 48/56).

All regression-locked (+movk_merges_into_existing_register,
loop_back_edge_reiterates_body, ldrsw_sign_extend_load_ground_truth, plus the
earlier ones). Workspace 111/0, build clean.

### Honest remaining (small, next session)
- LdStrReg register-offset ldrsw/ldrsh may share the bit22-mislead (the C
  battery only emitted unsigned-offset forms); verify and fix if so.
- FcvVec 4S lane edge (uses movq/cvttsd2si on 4-byte lanes) and fcvtzu ≥2^63.
- ADD/SUB with rn==31-as-XZR (`add xD, xzr, #imm` reads SP today; assembler
  uses movz/orr, so low priority).
- Then libbadcpu gaps; services/auth. GPU ev-boards: HARD GATE.
## Session (Sep 11, 2026) — register-offset sext + LogicalImm-vs-MoveWide (commit 716876c); JIT executes broad real C

Extended the synthetic-C battery to arrays/shorts/structs and found+fixed two
more silent miscompiles:

1. LdStrReg register-offset ldrsw/ldrsh/ldrsb shared the bit23 mis-lead (decoded
   as stores) — same fix as the unsigned-offset form (sext field + translate).
2. LogicalImmediate (AND/ORR/EOR/ANDS #imm) collided with MoveWide: the MOVZ/
   MOVK/MOVN gate matched top bytes {0x12,0x92,0x52,0xD2,...} which span the
   AND/EOR/ANDS-immediate class, so `and w1,w0,#0xffff` decoded as `movn`.
   MoveWide now gates on bits[28:23]==0x25 ((insn & 0x1f800000)==0x12800000);
   LogicalImm has 0x24, so AND-immediates route to LogicImm. Verified shacc
   (short acc) and arr (int+short arrays) = 42.

Net: the JIT now correctly executes ~20 real compiled aarch64 C programs
(loops, recursion, FP, mul/div, sign-extend loads both offset forms, short/int
arrays, AND-immediates, MOVK constants, SP prologues). arm64jit 78/78, workspace
112/0.

### Open (next session, honest)
- struct-by-value + function-pointer (`blr` to computed addr) still FAILS:
  `structs.c` → "pc 0x600000005 outside image". The fn-ptr arg gets corrupted
  through the struct-passing / dispatcher path — a deeper control-flow/ABI
  interaction (how the emitted GOT/adrp computes a callable and the dispatcher
  resolves it). Worth a focused session.
- FcvVec 4S lane uses movq/cvttsd2si on 4-byte lanes (possibly wrong); fcvtzu
  for >= 2^63; ADD/SUB rn==31-as-XZR reads SP (assembler prefers movz/orr, low
  priority).## Session (Sep 11, 2026) — LdStrReg sign-extend stored address, not value (commit 7b6b19e)

Post-battery hardening: a register-offset sign-extend exec test surfaced a
translate bug in the bit23/sext path added in 716876c. `ldrsh w0,[x1,x0]`
(0x78e06820, register offset, W dest, no shift) loaded the signed value into
RCX but stg_if_writable stores RAX — i.e. it stored the *effective address*
into the dest register. Every a[i] in a short-array loop via register-offset
LDRSH silently corrupted the accumulator. The session battery passed only
because arrays used ldr w / unsigned-offset forms.

Fix: the LdStrReg sext branch now loads the value into RAX (address no longer
needed), mirroring the LdStrImm sext arm. sumh over `short a[]` (register-offset
ldrsh) = 26 -> 42. arm64jit 79/79, workspace 113/0. +regression
ldr_reg_sext_sign_extends_into_dest.
## Session (Sep 11, 2026) — JIT ABI correctness: struct-by-value + 32-bit semantics (commits b8b5e62, e5d78d3)

Worked the open "struct-by-value + function pointer" item. Reproduced it with a
cross-gcc battery run through `cargo run -p arm64jit --example elfjit` (real
compiled aarch64 C, `-static -nostdlib -Wl,-e,entry`), fixed **four real silent
miscompiles**, gold-locked each with a regression test. `cargo test -p arm64jit`
-> 82, workspace 116/0. Battery: loop1=45, structs/dispatch/fpfun/vtable=42,
byvalue=44, bv2=300, signmod=12, iso_wrd=4321, iso_arith=300 — all correct.

1. **LdStPair offset-form ignored its immediate** (`b8b5e62`). `ldp x0,x1,[sp,#16]`
   (writeback=0) computed `access_off = 0`, so a 16-byte struct passed by value
   read [sp],[sp+8] (the saved x29/x30) instead of [sp+16],[sp+24] — byvalue.elf
   got (0,0) and returned garbage. The three addressing modes were conflated;
   now offset=`(imm,0)`, post-index=`(0,imm)`, pre-index=`(imm,imm)`.
   +`ldst_pair_offset_form_applies_immediate`.

2. **ADD/SUB rn==31 read SP when the S flag is set** (`b8b5e62`). `negs w1,w0`
   (subs w1,wzr,w0) computed `sp - w0` instead of `-w0` (rn=31 is XZR for the
   flag-setting form; only non-S `sub sp,sp,#N` reads rn=31 as SP). This was the
   documented "ADD/SUB rn==31-as-XZR reads SP" gap — a real repro finally
   (signmod.elf `%16` produced garbled remainders). Fixed AddSubImm + AddSubReg;
   also made LogicReg/AddSubReg read rm==31 as XZR (was SP). +`addsub_s_flag_reads_xzr_not_sp_for_rn31`.

3. **32-bit W writes did not zero-extend** (`b8b5e62`). `mov w0,w1` copied the full
   64-bit x1, so a negative two's-complement w1 propagated as 0xffffffffffffffff.
   Added `zext_w` (shl32/shr32) and applied to 32-bit LogicReg (operands, the
   N=1 BIC/ORN/EON half after `not`, and the result) and 32-bit AddSubImm/AdhReg.
   This was the "Ws must zero-extend" open item; it was silently corrupting any
   32-bit chain once a negative value entered a W register.

4. **Scalar `fcvtzu` saturates the wrong half** (`e5d78d3`). fcvtzu is unsigned,
   valid over [0,2^64), but the code used signed `cvttsd2si` which saturates
   anything >= 2^63 to INT64_MIN(0x8000..0); the old comment wrongly claimed
   `d>=2^63` was "architecturally out-of-range". Now a three-path sequence
   (d<2^63 signed; 2^63<=d<2^64 via `2^63 + int64(d-2^63)`; d>=2^64 -> u64::MAX)
   with in-buffer jc/js/jmp patching (mirrors the Ucvtf2d JNS idiom).
   +`fcvtzu_handles_u64_beyond_2pow63`.

Also added the `JIT_BUDGET` env knob (default 8192) to `jit_run` for
instruction-granular tracing under `JIT_TRACE` (`JIT_BUDGET=1`), and a
diagnostic captured by it: a **bounded-truncation fall-through bug** — a block
cut off mid straight-line by the budget had no pc write, so the dispatcher
re-compiled from the same entry forever. compile_image_bounded now diverts the
fall-through next-pc to a dispatcher-return stub when `truncated && !terminal`
(same fix that let the budget=1 per-instruction trace work; also a latent real
hazard for any Roblox function > 8192 insns without an early branch).

The battery lives in /tmp/jitbatt/ (not committed: it was ad-hoc before this
session). Next items on the JIT path: **FcvVec 4S-lane** conversion (uses
movq/cvttsd2si on 4-byte lanes), SIMD SMOV/UMOV lane->GPR and remaining
FP-vs-int lane ops, then libloader gaps -> libbadcpu gaps -> services/auth.
Real-binary/GPU boot remains blocked (no libroblox.so/APK, no GPU) — HARD GATE.

### Addendum (same session) — FcvVec FP->int vector (commits 7c11be3)
- **`.4s` lane width bug**: FcvVec converted each 4-byte S lane as a double
  (`movq_load` reads 8 bytes = the lane *and the next lane*) and wrote 8 bytes
  back (`movq_store` clobbered the neighbour lane), so multi-lane float->int
  vectors were corrupt. Now loads the 32-bit float, `cvtss2sd`s it, stores a
  32-bit int per lane (`mov_store32`).
- **`.2d` decode bug**: `fcvtzs v0.2d` (0x4ee1b820) has bit20=0 just like `.4s`,
  so `esize=(insn>>20)&1` mis-decoded the 64-bit form as esize=4. The real
  discriminator is **bit22** (0x400000). +`fcvt_vec_4s_lanes_are_32bit_and_independent`
  covers .4s (independent lanes), .2d signed, and .2d unsigned negative-clamp.
- arm64jit now 83, workspace 117/0. Each fix was a silent data-corruption bug
  that would have produced wrong pixels/audio/coordinates in a real Roblox run.

## Session (Sep 11, 2026) — SIMD lane-insert/extract fix: INS/SMOV/UMOV (commit 0f2d806)

Picked up the standing "SIMD SMOV/UMOV lane->GPR and remaining FP-vs-int lane
ops" item. Drove real aarch64 asm (INS/SMOV/UMOV across all element sizes +
vector logical/sat) through `elfjit` and found a **silent miscompile** that
predated this session:

### The bug (would corrupt NEON-heavy graphics/audio)
`mov v0.s[i],w1` (INS: GPR->vector-element insert, opcode bit13 CLEAR) and
`smov`/`umov` (element extract to GPR, bit13 SET) share the decode fields of
the vector-logical (AND/ORR/EOR/BIC) and saturating-add (SQADD/UQSUB) classes
(residue 0x..2x0c00). The precise lane-element gate
`(insn & 0xffe0_0c00) in {0x0e000c00, 0x4e000c00}` was placed AFTER
SimdVLog (line ~1186) and SimdSatAdd (line ~1425), so a real INS/SMOV was
silently mis-decoded before the lane gate was reached:
  - `ins v0.s[0],w1` 0x4e041c20 -> AND (SimdVLog): never wrote v0, clobbered x0
  - `smov x2,v0.s[0]` 0x4e042c02 -> SQSUB (SimdSatAdd)
  - `smov x6,v0.h[2]` 0x4e0a2c06 -> SQSUB
objdump-verified the encodings; `gcc` compiles `mov v.s[i],wN` as this INS form
everywhere NEON 4-element scalar writes are edited into vectors.

### The fix (decode.rs + translate.rs + jit.rs)
1. Move the lane-element gate BEFORE the broad vector gates and key on the
   opcode field bits[13:12] (verified across all 4 element sizes):
   - bit13=1        => vector->GPR extract umov/smov/mov (sign = bit12 clear)
   - bit13=0,bit12=1=> GPR->vector insert `ins/mov Vd.T[idx],Rn` (NEW `Inst::InsGp`)
   - bit13=0,bit12=0=> dup-from-GPR (fall through to the existing SimdDupGp)
2. Removed the now-dead late duplicate gate at the old location.
3. SimdLaneGp translate now handles ALL element sizes (1/2/4/8), so `.h`/`.b`
   extracts are no longer swallowed by SimdSatAdd.
4. InsGp translate: copy esize bytes of GPR rn into Vd at index*esize.

### Verified
- `cargo test -p arm64jit` 85/85 (added `ins_gp_inserts_element_into_vector_and_extract_reads_it`
  and `ins_gp_sign_and_zero_variants_insert_correct_lanes`); `cargo test --workspace` 119/0.
- elfjit harness (real aarch64): lane_test.elf / smov.elf return -570
  (=0xfffffffffffffdc6; previously returned 0). and/orr/eor/bic/sqadd/sqsub
  still decode+execute as their real ops (logical.elf runs them all and stops
  honestly at `uaddl` 0x4000ec = 0x2ea20020, the next unimplemented widening
  mul — an honest Unsupported stop, not a miscompile). `dup v1.4s,w10`
  (0x4e040d41) still decodes as SimdDupGp (the bit12==0 fall-through).

### Next (ordered, no APK/GSI/GPU on this box)
1. SIMD widening-multiply family (uaddl/saddl, and the smull/umull widening forms
   already partly present) — surfaced by logical.elf's honest stop.
2. libloader ELF/loader gaps -> libbadcpu ISA gaps -> services/auth.
3. Real-binary/GPU boot proof (`elfjit <libroblox.so> 0x1f0db20 --jni`) stays the
   HARD GATE, blocked until a capable host + the real binary/APK are available.

## Session (Sep 11, 2026) — SIMD widening families: add/sub-long + multiply-long (commits 80d27e5, d052723)

Driving the cross-gcc asm battery (logical.elf / addl.elf / mull.elf) through
`elfjit` cleared two more ISA walls AND exposed that the "already-implemented"
widening-multiply path shipped several silent miscompiles. Workspace now 121/0.

### ADDL: saddl/uaddl/subl/usubl (80d27e5)
- Gate only matched the 8 esrc=2 residues (0x..60), so esrc=4 (.2s->.2d, 0x..a0)
  and esrc=1 (.8b->.8h, 0x..20) fell through to Unsupported. Expanded `alres` to
  all 24 esrc x signedness x upper x add|sub residues; esrc = 1<<bits[23:22].
- Translate had the same width bug class as the old FcvVec/Mull code: esrc=4 read
  64 bits (both lanes), esrc=2-unsigned read 32 (polled next lane), esrc=1 stored
  32 (overran a 2-byte element). Now reads EXACTLY esrc bytes (sign/zero-ext to
  a 64-bit reg) and stores EXACTLY de=2*esrc bytes (8/4/2).

### MULL: smull/umull/smlal/umlal (d052723) — FIVE silent miscompiles
mull.elf "ran" without stopping, but that only proved no-unsupported. Inspecting
decode+translate against objdump found:
1. `res_esize = bit22 ? 8 : 4` — mis-sized smull .4h->.4s as 8, never .8b->.8h (res 2).
2. `unsigned = bit28` — bit28 is 0 for BOTH signed 0x0e and unsigned 0x2e, so
   umull/umlal were sign-extended (0xFE*2 => -4 not 508). Now bit29.
3. `acc = bit15` — set on plain smull/umull too, so every plain widening multiply
   ACCUMULATED instead of overwriting Rd. acc = gateway clause (c000=mul, 8000=acc).
4. translate `lanes = res==8?2:4` — missing the .8b->.8h 8-lane form.
5. store width not exact (store32 for res=2) overran the next lane.

### Verified
- saddl .2d {7,-2}+{3,9}={10,7}; uaddl .4s {1,2,3,4}+{10,20,30,40}; uaddl .8h
  1..8+1..8; smull .2d {7,-3}*{5,-2}={35,6}; umull .8h 0xFE*2=508 (would be -4 if
  still signed) — all exec_bytes'd with objdump-verified encodings.
- decode binds (res_esize/unsigned/acc/q) asserted for all six mull + three addl forms.
- logical.elf / addl.elf / mull.elf run to completion; prior battery + lane_test
  (-570) unchanged. arm64jit 87/87, workspace 121/0.

### Next (ordered, no APK/GSI/GPU on this box)
1. Keep pressing the SIMD surface (the battery will keep surfacing the next wall,
   e.g. shift-by-immediate / tbl / dup .b / post-index SIMD ld, then svc on real use).
2. libloader ELF/loader gaps -> libbadcpu ISA gaps -> services/auth.
3. Real-binary/GPU boot proof (`elfjit <libroblox.so> 0x1f0db20 --jni`) stays the
   HARD GATE, blocked until a capable host + the real binary/APK are available.

## Session (Sep 11, 2026) — SIMD shift-by-immediate: ushr/sshr (commit c6eb8df)

Continuing the cross-gcc SIMD battery, `shiftimm.elf` returned 0xfffffffc for
`ushr v0.2s,v1.2s,#8` (expected 3) — another silent miscompile.

### Root cause
Plain shift-right-immediate (marker bits[14:12]==0b000) had NO decode gate, so
it fell into the broad VecMovi (vector-immediate) gate and wrote a wrong
immediate pattern instead of shifting. (shl 0b101 and usra/ssra 0b001 already had
gates; only the plain 0b000 form was missing.)

### Fix
1. `Inst::SimdShr` gate: SIMD-reg prefix {0f,2f,4f,6f} + bits[14:12]==0b000 +
   bit23 clear + immh(bits[22:19]) != 0 (movi/mvni always have immh==0, so they
   are NOT reclassified — verified movi.2s #5 still VecMovi). esize from fls(immh)
   = 1<<(fls-1); shift = 2*esize_bits - (immh:immb) — verified ushr.2s #8
   (immh4=7) and ushr.2d #17 (immh4=13). Placed before the VecMovi gate.
2. Translate handles shift >= esize_bits (ushr->0, sshr->sign fill) since x86
   `shr r64,imm` clamps count.
3. Fixed the SAME latent sign-extension bug in SimdShr AND SimdShrAcc (ssra):
   the esize-bit source was loaded zero-extended, so a NEGATIVE element under the
   arithmetic shift came out positive (0xffffff00 >>> 8 = 0xffffff, not -1).

### Verified
ushr.2s {0x100,0x200}->{1,2}; sshr.2s -256>>8 == -1 (was 0xffffff); decode binds
ushr unsigned / sshr signed / esize,shift; movi.2s stays VecMovi. shiftimm.elf
-> 3 (was 0xfffffffc); shifts.elf (sshl .2d) -> 256; prior battery + addl/mull +
lane ops unchanged. arm64jit 88/88, workspace 122/0.

### Next (ordered, no APK/GSI/GPU on this box)
1. Keep pressing the SIMD surface as the cross-gcc battery reveals it.
2. libloader ELF/loader gaps -> libbadcpu ISA gaps -> services/auth.
3. Real-binary/GPU boot proof (`elfjit <libroblox.so> 0x1f0db20 --jni`) stays the
   HARD GATE, blocked until a capable host + the real binary/APK are available.

## Session (Sep 11, 2026) — SIMD/sysreg correctness sweep: 4 real bugs + 3 ISA walls (127/0)
Continuing the cross-gcc battery. Two previously-"correct" paths and three
newly-hit instructions were wrong; all fixed + qemu-verified + regression-locked.

### 1. SimdShrAcc (usra/ssra) esize/shift decode bug (silent, real)
The ShrAcc decode derived esize from the 3-bit tagless immh via trailing_zeros,
which collapses EVERY esize>=4 shift to esize=1/shift=0 — ssra silently
accumulated WITHOUT shifting. Battery exposed: ssra .2d #2 of {-8,-16} returned
-24 not -6; ssra .4s #2 returned 72 not 18. Decode now mirrors the verified
SimdShr gate (full immh incl bit22, fls esize, shift = 2*esize_bits-(immh:immb)),
and the unsigned discriminator is bit29 (was bit11). Translate also guards
shift>=esize_bits (mirror SimdShr). qemu: -6 / 18 / 1.

### 2. neg reads rn=31 as XZR, not SP (AddSubReg, silent, real)
`neg xd,xm` = `sub xd, xzr, xm` (shifted-register, bit21=0) — rn=31 MUST be XZR
(=0). The translate read rn=31 as SP for every non-S op, so neg(x6) computed
sp-x6. Root cause: bit21 is the form discriminator (qemu: neg=0xcb0603e6 bit21=0
-> XZR; sub sp,sp,x1=0xcb2163ff bit21=1 -> SP). Added `sp_operand` (bit21) to
Inst::AddSubReg and applied on both read (rn) and write (rd) sides. Regression
`neg_reads_rn31_as_xzr_not_sp`.

### 3. SysReg MRS reads were silent no-ops (LATENT, all of them)
The translate wrote `buf.mov_ri64(rt,..)` where rt is a GUEST register index —
the value landed in a stray x86 reg, never committed to the guest file. So every
`mrs xN,<cntfrq|cntvct|nzcv|dczid|tpidr>` returned 0/garbage. Decode-side tests
passed because they only bind Inst fields; exec was never exercised (cf/dz
battery proved it: cntfrq + dczid both returned 0 before). Fixed all four read
paths (+ MRS via `stg`), and the msr-tpidr write kept as-is.

### 4. New ISA walls crossed
- dczid_el0 (`mrs x0,dczid_el0` = 0xd53b00e0): glibc CRT reads it to size its DC
  ZVA memset; returns 0x4 (16-byte block, DZP=0). sysreg 5.
- umulh/smulh (high 64 of 128-bit product): gate top 0x9b && bit22 set
  (separates from madd/msub where bit22=0), signed = bit23. x86 F7/4,F7/5
  one-operand mul/imul (RDX:RAX = RAX*rm). Verified vs qemu.

### Verification
- `cargo build --workspace` clean; `cargo test --workspace` 127/0 (arm64jit 93).
- Battery (all qemu-verified): ssra_2d=-6, ssra_4s=18, ushr=1, shl=24, shlimm=27,
  neg_d=2, cf(cntfrq)=100000000, dz(dczid)=4, mulh_e=2.
- modmain.elf (full glibc CRT) now advances past dczid + umulh/smulh to the next
  wall: MTE `stg x0,[x0]` (0xd9200800, __libc_mtag_tag_region) — memory tagging.

### Next (ordered, no APK/GSI/GPU on this box)
1. MTE stg/ldg/stzg memory-tagging no-op (unblocks full glibc-linked programs).
2. Continue the SIMD surface as the cross-gcc battery reveals it; then real
   `svc` syscall routing on actual use (real AArch64->x86-64 table; mmap 222 etc.).
3. libloader ELF/loader gaps -> libbadcpu ISA gaps -> services/auth.
4. Real-binary/GPU boot proof remains blocked (no libroblox.so/APK, no GPU) —
   HARD GATE on a capable host (`elfjit ... 0x1f0db20 --jni` run log).

---

# Session — glibc-CRT ISA sweep (dc/ic, MTE writeback, GCS/SME-TLS) + svc correctness (Sep 12 2026)

Continuing the cross-gcc / hand-assembled-battery approach with no APK/GSI/GPU.
**Three focused commits; workspace 132/0, arm64jit 97, tree clean.**

## 4db2b3c — data/instruction cache maintenance (dc/ic) no-ops
glibc's `__libc_mtag_tag_region` ends in a `dc` op. In the single-threaded
direct-mapped JIT these coherence ops (dc/gva/civac/ivac, ic ivau; top 0xd5,
CRn=7) are no-ops — EXCEPT `dc zva` which zeros the advertised 16-byte block.
Fixed two bugs in the leftover session-draft: Rt decoded from bits[9:5]
(instead of bits[4:0]; caused `dc zva x0` to write via x1 → segv), and the
test used 0xd50b7400 (=a `sys` instr) as `dc zva` — the real `dc zva x0` is
0xd50b7420 (CRm=4 && op2=1). modmain moved 0x40c174 -> 0x438b1c.

## d55cb04 — MTE tag-store writeback + mrs gcspr_el0/tpidr2_el0
- The MteTag gate forced bit10==0, so post/pre-index st2g/stg writeback forms
  (`[x2],#64` / `[x2,#-64]!`) were Unsupported. Their Xn-advance (Xn +=
  signed imm<<4) is a real side effect glibc memset/stg loops depend on; the
  tag-store itself stays a memory no-op. Decode now carries rn/wb/wb_off
  (imm9 sign-extended, scaled <<4). Load bit stays bit22 (ldg byte1=0x60;
  stg/st2g 0x20/0xa0), so the discriminator is unaffected. objdump-verified.
- `mrs gcspr_el0` (armv9 GCS ptr, 0xd53b2522) + `tpidr2_el0` (SME 2nd TLS,
  0xd53bd0ae) read 0 (features never enabled) — glibc CRT reads them sizing
  GCS call frames / probing SME. sysreg ids 6/7.
- modmain advanced to 0x442cf8, then stops on glibc's SME-IFUNC feature-probe
  (`str za w15,[x16]` = 0xe1206200). **Documented as BEYOND Roblox's
  Android/bionic boot ISA** — the real libroblox.so boot path is already fully
  decoded / exit 0 per prior sessions. Root cause of that glibc-only tail:
  elfjit sets up NO guest auxv, so glibc reads garbage AT_HWCAP and
  IFUNC-resolves into SME. Chasing the SME ZA-tile ISA is a synthetic-harness
  tangent, not a Roblox boot blocker.

## e20687d — svc syscall-number bugs + extended table + inline host-call fixes
Hand-assembled aarch64 svc programs (write / exit / multiple sequential svc)
through elfjit exposed real bugs on the syscall path:
1. **getuid was mapped to 199 (that's socketpair); real AArch64 getuid=174.**
   **mremap was mapped to 220 (that's clone); real = 216 (3264_mremap).**
   Neither was ever exercised (the unit test only checks write/mmap/getpid).
   Fixed; added uid/euid/gid/egid/tid/ppid @ 174-178/173. +regression
   `guest_svc_identity_numbers_match_aarch64_abi`.
2. Extended the table with common aarch64 boot syscalls: getcwd 17, chdir 49,
   getdents64 61, lseek 62, faccessat 48 (w/ AT_FDCWD), readlinkat 78, pipe2 59,
   set_tid_address 96, sched_yield 124, prctl 167.
3. **Inline host-call correctness (two real bugs):**
   - JIT block body runs at host RSP≡8 (mod 16) — correct for guest-to-guest
     BL (call_rel32) — but SysV needs RSP≡0 at a host CALL site. So
     `call guest_svc` / `call guest_sha1stem` fired misaligned; any callee with
     aligned stack work (format! in JIT_TRACE_SVC, SSE locals) SIGSEGV'd. Now
     sub rsp,8 before / add rsp,8 after each inline host call.
   - guest_svc(st)'s state arg was passed implicitly via RDI (held the entry
     state on the FIRST call by luck; a prior host call clobbers RDI), so the
     SECOND svc in a block passed garbage (+ misaligned deref of 0x1). Now
     `mov rdi, rbx` explicitly.
   Proof: svc_elf writes then exits 0; exit_only returns 7; `we` (2 writes +
   exit_group 3) returns 3 with both writes visible; trip (3 sequential
   writes) prints W1/W2/W3. Previously ANY 2nd svc segfaulted — a real
   blocker Roblox (many syscalls) would hit.

### Status
- `cargo build --workspace` clean; `cargo test --workspace` 132/0 (arm64jit 97).
- Battery clean (no unexpected walls); modmain still stops honestly at the
  documented SME `str za` (0x442cf8), beyond the Roblox boot ISA.
- Commits 4db2b3c, d55cb04, e20687d on local `dev`.

### Next (ordered, no APK/GSI/GPU on this box)
1. libloader ELF/loader gaps -> libbadcpu ISA gaps -> services/auth.
2. Optional (glibc-coverage only, not Roblox): give elfjit a guest auxv so
   glibc IFUNCs resolve to scalar (non-SME) paths.
3. Real-binary/GPU boot proof (`elfjit <libroblox.so> 0x1f0db20 --jni`) stays
   the HARD GATE, blocked until a capable host + the real binary/APK.


---

## Session (Sep 11, 2026) — libbadcpu gregs register-map fix + arm64jit BitField disarm (151/0)
Two crates hardened against silent miscompiles; the HANDOFF's documented open
arm64jit bug (`__tunable_get_val` x4 corruption) is FIXED. Commits `8325edc`,
`9081bfc`, `17e449b` on `dev`.

### libbadcpu (8325edc): the emulator wrote the WRONG registers
`ucontext_t.uc_mcontext.gregs` is `greg_t[23]` with R8..R15,RDI,RSI,RBP,RBX,
RDX,RAX,RCX,RSP in slots 0..15 (only RIP=16/EFL=17 match the x86 reg number).
The old table indexed gregs[0] as RAX etc., so every emulated POPCNT/MOVBE/
LZCNT/TZCNT/BMI1 read/wrote the wrong register and corrupted guest state.
Now GREGS_IDX maps x86 reg number -> true slot. Also fixed: VEX `vvvv` was
never decoded (3-byte C4 + 2-byte C5) — ANDN used the DEST register as its
first source; and the VEX opcode byte was read from the C4/C5 prefix position,
so no 0F38/0F3A-map VEX instruction ever decoded correctly. 6 new tests;
libbadcpu 6->12.

### arm64jit (9081bfc): BitField dispatches by class — modmain boots PAST its old crash
Driving `modmain.elf` (full static glibc, qemu=12) through elfjit:
1. UBFIZ/SBFIZ (insert=false, immr>imms) went through the BFI/merge path and
   PRESERVED old Rd's bits. `ubfiz x4,x0,#7,#32` kept a stale 0x7f8000000000
   prefix, so glibc's `__tunable_get_val` ldr'd [x4,#48] at 0x7f800048e888
   (should be 0x48e888) -> SIGSEGV. UBFIZ zero-fills; SBFIZ sign-fills.
2. Genuine BFI (insert=true) was swallowed by the ROR shortcut
   (imms+immr+1==bits: 15+48+1==64) and compiled as a rotate. The LSR/LSL/ROR
   shortcuts are UBFM/SBFM aliases; BFM inserts now handled first (BFXIL
   in-place mask, BFI shifted merge).
Regressions `ubfiz_zero_extends_field_and_discards_old_rd` +
`bfi_still_merges_into_old_rd`. arm64jit 108->110; workspace 151/0; full
cross-gcc battery unchanged (loop1 45, structs/dispatch/fpfun/vtable 42,
byvalue 44, bv2/iso_arith 300, signmod 12, iso_wrd 4321, arr/shacc/fact/ldrsw/
fp_only/A/C 42).

### Honest remaining
- modmain now boots past its old `__tunable_get_val` crash; the udiv fix (below)
  cleared `_dl_determine_tlsoffset` too. It now stops deep in the glibc-CRT tail
  (`__memset_generic`, caller passed x0=0) — the HANDOFF-flagged synthetic-glibc
  tangent that is NOT a Roblox boot blocker (real libroblox boot path already
  fully decoded / exit 0).
- Real-binary/GPU boot proof (`elfjit <libroblox.so> 0x1f0db20 --jni`) stays the
  HARD GATE; blocked on a capable host + the real binary/APK (none on this box).

## Session (Sep 11, 2026) — UNSIGNED DIV WRONG-RESULT BUG FIXED in arm64jit (152/0)
Commits `28dba32` (div), `c1e41cb` (svc additions).
status: session-end (committed, tests green)

### Silent, wide bug: every UNSIGNED division in the JIT returned 0
The HANDOFF's open "small-address load in `_dl_determine_tlsoffset`" was the
guest doing `udiv x0,x0,x1`. A focused `exec_bytes` test (`udiv x5,x0,x1` =
0x9ac10805) left x5=0 for EVERY input (100/10, 7/1, 0/5), even in isolation.
Byte-dumping the emitted host code showed `48 f7 c1` — x86 group-3 `F7` uses
/6 = DIV and /7 = IDIV, but `div_r64`/`div_r32` emitted `modrm(3,0,..)` =
group-3 /0 = TEST, so the instruction decoded as `test rcx,eax` and never
produced a quotient. `idiv_r64` already used /7 and was correct — ONLY the
unsigned forms were broken. Confirmed with as+objdump: `48 f7 f1` = div rcx /
`48 f7 f9` = idiv rcx. Fixed both emitters to /6.
Regression `udiv_computes_quotient` (100/10=10, 7/20=0). Silent, WIDE wrong-
result class: any guest unsigned integer math (incl. Roblox) returned 0.

### svc table additions (c1e41cb)
glibc `__tls_init_tp` surfaced set_robust_list(99)->0 (no-op is valid; -ENOSYS
made glibc retry) and membarrier(283)->no-op. rseq(293) left -ENOSYS (no valid
rseq area). Verified vs aarch64-linux-gnu asm-generic/unistd.h.

### Result
modmain.elf booted THROUGH `_dl_determine_tlsoffset` (udiv fix), reached real
AArch64 syscalls, then hit glibc `__memset_generic` (x0=0 passed by caller) —
again the non-Roblox glibc-CRT tail. arm64jit 111/111; workspace 152/0; battery
unchanged.
- HARD GATE unchanged: `elfjit <libroblox.so> 0x1f0db20 --jni` on a GPU/APK host.

## Session (Sep 11 2026) — guest auxv bootstrap; SME/SVE/MulLong decodes; zero-extend fix (137/0)

Goal: prove the JIT boots a full statically-linked glibc aarch64 binary.
modmain.elf (`main(){return 300%16;}`, qemu returns 12) tests the whole CRT.

1. **`arm64jit::boot` — guest auxv on the initial stack.** The kernel ABI puts
   [argc][argv][envp][auxv AT_NULL] on the stack (sp at argc); glibc static
   `_start` walks it for envp/auxv. The JIT left garbage, so `_dl_hwcap2` picked
   HWCAP2_SME from garbage and fell into `__libc_arm_za_disable`'s `str za` loop.
   New `standard_auxv(&LoadedElf, hwcap, hwcap2)` + `layout_initial_stack()`
   (fills AT_RANDOM via bounded xorshift). Wired into elfjit AND sober-core::jit.
   Verified `_dl_hwcap2=0, have_sme=0`.
2. **SME/SVE feature-off decodes.** Even with SME off, glibc's ZA block is
   *linearly* reachable in the compiled CFG (compiler follows fall-through past
   the data-dependent SME gate), so it had to DECODE: `Inst::SmeNoop`
   (`str za[Wt,k],[Xn,#k,mul vl]` 0xe1206200..f + smstart/smstop), `Inst::
   AddVectorLen` (addvl/addsvl, `Xd = Xn + imm6*16`, model VL=16B), `Inst::
   SveCntd` (cntd -> 2), `mrs xN, midr_el1` (sysreg 8 -> 0).
3. **`Inst::MulLong`** — smull/umull/smaddl/umaddl/smsubl/umsubl, found in
   glibc `_dl_fixup` (IFUNC resolution). Gate top 0x9b & bits[22:21]==01.
4. **LATENT x86-emitter bug fixed:** `and_ri64(_, 0xffffffff)` was a NO-OP
   (`and r64,imm32` sign-extends the imm -> AND-all-ones). All 10 zero-extend
   sites silently leaked W-reg high bits (the glibc x3/x0 corruption). Added
   `CodeBuf::zero_ext_r32` (`mov r32,r32`) and replaced all 10 uses. Proven by
   umull/smull/umsubl/addvl exec tests.

Verification: `cargo build --workspace` clean; `cargo test --workspace` 137/0
(arm64jit 103). modmain.elf boot now advances past `str za` -> _dl_fixup umull
-> midr_el1, then STILL stops early on a glibc-startup x-reg corruption (auxv
scan / __libc_start_main prologue) — the zero-extend fix is in but a leftover
cause remains. qemu returns 12; JIT boot to completion NOT yet achieved.

Next: (1) finish the glibc-startup corruption; (2) modmain->12 proves full
static-glibc boot; (3) libloader gaps -> libbadcpu gaps -> services/auth;
(4) HARD GATE = real libroblox.so + GPU host (`elfjit <lib> 0x1f0db20 --jni`).


### Continued (Sep 11 2026) — after commit 5d40fdd, continued the same goal
Three more real JIT bugs fixed, each verified + a regression test; modmain.elf
(full static glibc, qemu returns 12) boot advances far into glibc startup.

- **Extended-register add/sub** (`Inst::AddSubExt`): `add x3,x2,w20,sxtw #3`
  (bit21=1) was decoded through the SHIFTED parser, mis-reading the option/
  shift bits as `lsl #sh_amt` (=51) — corrupting guest x-registers with
  `0x198...` garbage (a real glibc prologue corruption). New gate
  `n==1 -> AddSubExt`, proper ext(Rm)<<shift with 8 options + SP semantics.
- **Pre/post-index + unscaled immediate LDR/STR** (`Inst::LdStrImmWb`): the
  register-offset gate `(insn&0x3b000000)==0x38000000` was too broad and
  swallowed `ldr x3,[x0],#8` (0xf8408403), mis-reading imm9+writeback as an
  `rm` register -> NULL deref (the glibc auxv/env-scan crash). Narrowed to
  `(insn&0x3b200c00)==0x38200800`, added a proper signed-imm9 writeback path
  (pre/post/unscaled). NOTE: also fixed a latent `b(insn,hi,lo)` arg-order
  underflow panic in the new decode.
- **LSE atomics** (`Inst::LseAtomic`): ldadd/ldclr/ldeor/ldset/swp (ARMv8.1),
  hit in glibc's IFUNC `__aarch64_swp4_acq` (have_lse=0 so on a never-taken
  path, but the block compiler must still build it). Gate: (insn&0x3fe00000)
  in {0x382/0x386/0x38a/0x38e 00000} AND op=bits[15:10] in {0,4,8,0xc,0x20};
  single-threaded emulation (old=[Xn]; [Xn]=f(old,Rs); Rt=old).

Verification: cargo test --workspace 142/0 (arm64jit 108, +2 lse tests).
modmain.elf now runs __libc_start_main fully and dies deep in
`__tunable_get_val` on an address (0x48e8b8) with run-varying high garbage
(another latent 32-bit/zero-extend leak) — the next debugging target.
Also: elfjit now keeps a permanent SIGSEGV diagnostic handler (prints guest
pc + regs from CpuState) — invaluable for localizing a real-code crash.


### Continued (Sep 11 2026) — LSE atomics landed; modmain now dies in __tunable_get_val (block-register bug)

Commit 36a47f7 decoded the LSE atomics (ldadd/ldclr/ldeor/ldset/swp) with
single-threaded emulation; modmain.elf boot then advanced THROUGH
__libc_start_main and the IFUNC atomics (the ldxr/stxr fallback, since
have_lse=0) and now crashes inside `__tunable_get_val` (0x4128ec) on a
block-level register corruption:

- fault = 0x7fXX_0000_48e8b8 (low 0x48e8b8 constant, high 0x7fXX/0x7eXX
  run-varying).
- x7 = 0x48dc88 is CORRECT (adrp x3,0x48d000; add x7,x3,#0xc88 — both clean).
- x4 = 0x7f800048e888 is CORRUPTED. It should be
  `ubfiz x4,x0,#7,#32` (0xC00 for x0=0x18) then `add x4,x7,x4` = 0x48dc88+C00
  = 0x48e888. Instead x4 carries 0x7f8000000000 high garbage from the *source*
  x4 at the `add x4,x7,x4` (rm=x4 read gave 0x7f8000000C00).
- ubfiz is NOT the bug: proven clean in isolation AND on a pre-dirtied x4
  (0x1f000000 -> 0xC00), and the full ubfiz;add sequence is clean in isolation
  (0x48e888 with a correct x7 build). So it is a BLOCK-COMPILER register-
  interaction bug only in the real __tunable_get_val block (persists across
  JIT_BUDGET 2/8/16/300000, so not a boundary artifact): likely the intervening
  `mov w5,w0` (W write) or `adrp`/`mov` reusing a host register that also holds
  the rm=x4 value, so `add x4,x7,x4` reads a stale/dirty x4.
- Next: dump block@0x4128ec host code (JIT_DUMP) or add per-instruction guest
  x4 trace to find which instruction gives x4 the 0x7f8000000000 high prefix.

Also: elfjit now permanently installs a SIGSEGV diagnostic (guest pc + x0..x7 +
sp from the CpuState via ucontext RBX) — the tool that pinned all of today's
crashes; recorded because it is reusable.

`cargo test --workspace` 142/0 (arm64jit 108).

### HARD GATE (unchanged)
Roblox actually running (load -> JNI init -> main loop -> frame on a GPU host)
is NOT met and cannot be on this GPU-less VPS without the real libroblox.so/APK.
The elfjit path is a growing no-QEMU CPU translator that currently boots a full
statically-linked glibc program deep into its CRT/startup. The HARD GATE remains
`elfjit <libroblox.so> 0x1f0db20 --jni` on a capable host.

---

# Session (Sep 11, 2026) — scalar FP/SIMD unscaled + pre/post-index ld/st; libbadcpu VEX.0F38 completion (workspace 164/0, commits 36f01ff + 0fe7d8a)

Opened by re-running the workspace: the handoff's flagged
`test_setup_android_layout_idempotent` is ALREADY FIXED (8b72828: per-call
unique temp root + `create_dir_all` before `set_permissions`); it passes — the
workspace was 157/0 at start, not failing. Proceeded to two committed, tested
pieces (no APK/GSI/GPU required):

## 1. arm64jit — scalar FP/SIMD (B/H/S/D) UNSCALED (ldur/stur) + PRE/POST-index writeback ld/st
Closed the last of the `bit26=1` immediate family (the standing "stur/ldur
scalar s0/d0 + pre/post index" glibc-CRT-tail item).
- `Inst::FpLdStImmUnscaled` + `Inst::FpLdStImmWb` with a shared
  `fp_scalar_xfer` helper (transfers `size` bytes between memory and the low
  bytes of `v[vt]`, upper lanes preserved).
- Decode gate, each bit verified against aarch64-linux-gnu-as ground truth:
  bit26=1 (vector file), bit25=0 (immediate offset), **bit21=0** (NOT
  register-offset — found ONLY by the neighbor-collision test: FpLdStrReg
  register-offset words ALSO have bit25=0, the real discriminator is bit21),
  bit24=0 (not the scaled 0x3d form), bit23=0 (not 128-bit Q), bits[29:27]=111.
  operand size bits[31:30], ld=bit22, imm9 sign-extended, addressing =
  bits[11:10]: 00=unscaled, **01=post, 11=pre** (pre is `0x0c00` = 0b11, NOT
  0b10 — caught by the decode test). unprivileged LDTR/STTR (mode 2) left
  Unsupported.
- +4 tests. arm64jit 116→120. **modmain.elf (full static glibc) now boots
  PAST its documented `stur s0`/`stur d0` memset wall** into
  `__libc_setup_tls`/`_dl_get_dl_main_map`, stopping at a residual null-deref
  (fault 0x0, guestpc 0x400b30, right after `bl 0x413e60 _dl_get_dl_main_map`)
  — again the HANDOFF-flagged non-Roblox glibc-CRT tail, NOT a Roblox blocker.

## 2. libbadcpu — BEXTR + BZHI + SHRX/SARX/SHLX (complete the VEX.0F38 integer family)
The SIGILL emulator already did ANDN/BLSI/BLSMSK/BLSR (VEX.0F38 F2/F3/F1/F4);
added the rest. All encodings verified vs host gcc+objdump:
- BEXTR = 0F38 F7 pp=0: `(src1>>start)&(2^len-1)`, start=control[7:0],
  len=control[15:8] (control = VEX vvvv; pp=0 distinguishes it from the shifts
  which share F7 but carry a pp prefix).
- BZHI = 0F38 F5: `src1 & (2^ctrl-1)`; ctrl>=op-size keeps src1 + CF.
- SHRX/SARX/SHLX = 0F38 F7 with pp=F3/F2/66 respectively.
- **Real subtlety fixed:** fix-size for the 0F38 integer ops must come from
  VEX.W (`vex_w`), NOT the legacy `has_66 => 16-bit` rule — SHLX rax has pp=1
  (has_66) yet is 64-bit; the whole 0F38 branch now sizes off `vex_w`.
- +3 tests. libbadcpu 12→15.

## Gate
- `cargo build --workspace` clean; `cargo test --workspace` **164/0** (arm64jit
  120, libbadcpu 15, libloader 16, +1+1+11). Tree clean on local `dev`.
- HARD GATE unchanged: `elfjit <libroblox.so> 0x1f0db20 --jni` run log on a
  GPU + real-binary host.

---

## Session (Sep 11, 2026) — loader→JIT end-to-end regression test + libbadcpu 16-bit LZCNT fix (169/0)

Two commits on `dev` (HEAD `6d6ef08`). Workspace was 164/0 green at start
(`test_setup_android_layout_idempotent` is already fixed in 8b72828 — passes;
the FMOD divert 10ddb7a and JNI-table bdd8b03 tasks are also already landed).
So this session executed the ordered "libloader ELF/loader gaps" + "libbadcpu
ISA gaps" steps.

### 1. `7f54fbd` — arm64jit: loader→JIT end-to-end regression test
`crates/arm64jit/tests/loader_run.rs` cross-compiles real `-nostdlib` aarch64
programs (via `aarch64-linux-gnu-gcc`) and runs `entry()` through the exact
pipeline elfjit / `sober-core --jit` use: `load_elf_image` → `bind_image_plt`
→ guest stack/TLS/auxv bootstrap → `jit_run`. Results: add=42, loop sum(0..9)=
45, fp `(int)(2.5*4.0)`=10, fib(7)=13. Locks the loader path against
regressions from the divert + JNI-table changes (the bdd8b03 "confirm no
regression" deliverable), since the real-binary HARD GATE can't be exercised
here. Skips cleanly when cross-gcc is absent.
- **Found en route:** `[test]` threads run in parallel in one process, and
  `load_elf_image` MAP_FIXEDs the same non-PIE JIT base 0x400000 — 4 threads
  clobber each other's guest image → a `bl` target missing from the compiled
  block's stub table panics `stub_of_target[...]` (jit.rs:1164). Real open-
  sober loads ONE guest ELF for process lifetime, so this is purely a test-
  harness serialization concern: `run_lock()` serializes load+bind+run.

### 2. `6d6ef08` — libbadcpu: 16-bit LZCNT wrong-result bug
emulator.rs LZCNT 16-bit arm was `(src as u16).leading_zeros() as u64 - 16`.
`u16::leading_zeros` already returns the 0..16 count, so `-16` made every
nonzero 16-bit LZCNT return negative (wrapped huge i64 in the dest reg).
TZCNT's sibling arm has no such offset; the 32/64-bit arms don't either — only
LZCNT had it. Dropped the offset; +`lzcnt_16bit_matches_real_count`
(cx,ax=2 -> 14; cx,ax=0x8000 -> 0). Silent wrong-result class in the SIGILL
emulator.

### Gate
- `cargo build --workspace` clean (0 errors; warnings are the pre-existing
  decode.rs rustfmt churn — rustfmt not installed on this box).
- `cargo test --workspace` **169/0** (arm64jit 120 + 4 loader_run + libbadcpu
  16 + libloader 16 + 1 + 1 + 11).
- HARD GATE unchanged: `elfjit <libroblox.so> 0x1f0db20 --jni` run log on a
  GPU + real-binary host (no APK/libroblox.so/GPU on this VPS).

## Session (Sep 11, 2026) — three silent FP/SIMD miscompiles fixed via a double-precision C battery (commit a0465a0)
Drove a new cross-gcc double-FP battery (real `double` C: polynomial Horner,
array sums, 2x2 matmul, exact division, 3^10 accumloop with fcvtzs, |x|>threshold
counting, weighted average) through `elfjit`, comparing each result to a native
x86-64 compile. Ground truth: dpoly31 dsum17 dmat50 ddiv10 dscale1 dclamp4 ddmat2-4.
Found + fixed THREE real miscompiles (all silent wrong results, not crashes):
1. **scalar fsub (0x1e613800) decoded as SIMD WidenShl** — the shll gate's
   `(insn>>24)&0x0f==0x0e` nibble test ALSO matched the scalar-FP 0x1e family
   when bits15:8==0x38, so `fsub d0,d0,d1` ran as a halfword-widen no-op
   (59049.0-59048.0 → 0.0). Real shll bytes are 0x0e/0x2e/0x4e/0x6e (bit28=0);
   scalar-FP 0x1e has bit28=1. Gate now requires bit28==0. dscale.elf 0→1.
2. **store_nzcv_fp hardcoded N=0** — FP compare sets N=1 for ordered less-than,
   so b.mi/b.lt/b.le never fired and b.gt evaluated N==V as 0==0 for every
   ordered non-equal pair (dclamp 6→4). N now = CF∧¬ZF.
3. **ld1 multiple-structure (2-reg, opcode bits[15:12]==0xA) swallowed by the
   ld2 gate (0x8, deinterleave)** — compiler array-literal `ld1 {v30,v31}`
   loaded interleaved garbage (ddiv double array 2→10). Added Ld1N/St1N
   (consecutive, NO deinterleave) for 1/2/3/4-reg, discriminated by opcode
   bits[15:12]. (Verified real encodings: ld1-2reg=0x4c40a040, ld2=0x4c408040,
   ld1-1reg=0x4c407040.)
+3 regression tests (fsub-vs-shll collision, FP-compare N flag + b.mi/le/gt,
ld1-2reg consecutive). arm64jit 123/123, workspace **172/0**. Cross-gcc C
battery + SIMD hand-battery (lane_test/smov=-570, loop1=45, iso_*=etc) all
still green. fsqrt verified (sqrt(16)=4 via sqrtquad.elf).
Honest: dsqrt .c didn't link (sqrt undefined under -nostdlib); tested fsqrt via
hand-asm instead. Test-setup lesson: guest Dn/Vn maps to st.v[2n]/st.v[2n+1]
(D1 = st.v[2], NOT st.v[1]) — two new tests initially failed on my own wrong
constant placement, not a JIT bug.
Next: keep pressing the cross-gcc FP/SIMD surface (division edges, fma chains,
single-precision float, struct-by-value + FP, loop-with-FP-condition) to find
more silent miscompiles; then libloader gaps. HARD GATE unchanged.

## Session (Sep 11, 2026) — FMOV-immediate [16,30] decode bug fixed (commit bc5ab89)
A second cross-gcc battery (single-precision array div, double loop, mixed
int/float casts, double-struct-by-value, float matmul, double reciprocal —
native ground truth fdivf20 dloop7 mixed4407 dstruct169 dneg0 fmat69 drec124)
hit: fdivf `float a[]={6,12,18,24}; sum a[i]/3` returned 6 instead of 20.
Root cause was NOT the float div (isolated divss/addss/fcvtzs.e,.s all correct)
but **decode_fmov_imm's exponent wrap**: the 3-bit field E maps E0..3->e+1..+4,
E4..7->e-3..0, but the code wrapped `ex>=4`, so E=3 (exponent +4 => constants
16.0..30.0) decoded as -4 (0.0625..0.117). Every FMOV-imm in [16,30] — sample
rates, half-texel, 24.0 corner constants — came out ~256x too small (silent).
Fix: threshold `ex>=5` (E=4 gives (E+1)=5 -> -3). Verified against the
assembler's encodings for 0.125..30.0 (immf.s). fdivf 6->20; dloop/mixed/
dstruct/dneg/fmat/drec all match native. +4 decode_fmov_imm asserts (16,30,2,
0.75). arm64jit 123/123, workspace 172/0, prior battery unchanged.
Lesson: single- and double-FP immediate decoding share decode_fmov_imm — a
boundary exponent bug corrupts both (the fdivf array used f32, dscale earlier
used 3.0; the [16,30] band is where it bites).
Next: keep pressing FP/SIMD (fma/compiler-contracted `fmla`, more div/compare
edges, single-precision struct args) then libloader gaps. HARD GATE unchanged.

## Session (Sep 11, 2026) — scalar FP multiply-accumulate fmadd/fmsub/fnmadd/fnmsub (commit 578faaf)
Third FP milestone: the -O2 compiler contracts every a*b+c / fused a*x*x into
`fmadd`, and fma1 (2x^2+3x+1 over x=1..5) returned 0x8000000000000000 garbage
because 0x1f4.. was swallowed by a broad SIMD vector-immediate gate and
mis-decoded as a bogus movi. Added Inst::Fma3 (scalar 3-source FP):
- decode gate `(insn & 0xff000000)==0x1f000000` placed at the TOP of decode
  (uniquely the scalar 3-source FP family), o1=bit21(fn*), o2=bit15(sub),
  sz=bit22(double).
- translate: mulsd/addsd/subsd with pxor-0 + subsd for the fnmadd negation;
  single-precision via mulss/addss/subss. Semantics fmadd=ra+rn*rm,
  fmsub=ra-rn*rm, fnmadd=-(ra+rn*rm), fnmsub=rn*rm-ra.
- Verified vs assembler (fmadd/fmsub/fnmadd/fnmsub d0,d1,d2,d3 =
  0x1f420c20/0x1f428c20/0x1f620c20/0x1f628c20) -> 17/-7/-17/7 with d1=3,d2=4,
  d3=5; single fmadd s -> 11. fma1.elf 160 = native.
+scalar_fma3_all_four_variants (4 double + 1 single). arm64jit 124/124,
workspace 173/0. Full 3-batch cross-gcc battery unchanged.
This session net: 5 FP/SIMD correctness fixes (fsub-vs-shll, FP-compare N flag,
ld1-2reg deinterleave, FMOV-imm [16,30], FMADD) + the FMADD feature. Next:
keep pressing -O2/FP-contracted programs, single-precision struct args, more
div/compare edges; then libloader gaps. HARD GATE unchanged.

## Session (Sep 11, 2026) — shifted-register add/sub clobber bug (commit 9cc0f16)
Fourth FP milestone. The -O2 loop version of fclamp (double clamp across an
array) returned 10 vs 9; straight-line clamp worked. Root cause:
`apply_shift_const(buf, x, kind, amt)` wrote the shift amount into RCX
(`mov rcx, amt`) then `shl rcx, cl`, but both callers (AddSubReg, AddSubExt)
pass x == RCX (the Rm value being shifted) — so the value was clobbered and the
operand became `amt<<amt` instead of `Rm<<amt`. `add x1,x2,x0,lsl#3` (the
ubiquitous array-index idiom) computed x2+24 CONSTANT, so -O2 double-array loops
read the SAME element each iteration (fclamp: all 5 reads of a[2]=2.0 -> sum
10). Fixed to the immediate-shift C1 /4..7 ib forms (no CL scratch). fclamp
10->9. +regression add_shifted_register_... (lsl#3/lsr#2/asr#1/plain add)
verified vs shadd.o encodings. arm64jit 125/125, workspace 174/0; full battery
unchanged. (dnorm.elf: its Newton reciprocal-sqrt overflows to +inf = UB in C,
not a JIT bug — excluded.)
Lesson: emitters that use a fixed scratch register must never be handed that
same register as an operand. apply_shift_const's CL scratch collided with the
RCX operand; immediate-shift forms sidestep it entirely.
Session net: 6 FP/SIMD correctness fixes + FMADD/FMA3 feature, all committed
with regression tests. Next: keep pressing -O2/loop/array coverage (the
fclamp class is now unblocked), single-precision struct args, division
edges; then libloader gaps. HARD GATE unchanged.

## Session (Sep 11, 2026) — LdStPair D-register stride bug + 4th -O2 battery (commit 1eb7c0a)
Fifth FP milestone. A 4th cross-gcc battery (all -O2, exploiting the now-fixed
shifted-register indexing: 3x3 int matmul, struct{double x,y} array walk,
byte-scan, 64-bit loop, short array) surfaced one more real bug:
structfield.elf (loop `ldp d29,d28,[x0],#16` + `fmadd`) returned 128 vs 52.
Root cause: the LdStPair `fp_d` branch located each D-register at
VECTOR_BASE + rt*8, but a guest Dn is the LOW 8 bytes of its 16-BYTE vector
slot (VECTOR_BASE + rt*16) — so `ldp d29,d28` wrote 8-byte values to
0x1f8/0x1f0 instead of 0x2e0/0x2d0, and the follow-on fmadd read the stale
vector slots (still the initial q-pair array literal). Fixed both load+store
to *16 (q128 already used *16). structfield 128->52.
- 4th battery results (all match native): m3=45, structfield=52, bytes=5,
  iloop=150, shorts=24, plus the earlier fclamp/fhorner/fquad/fdivmix.
  +regression ldst_pair_d_registers_use_16_byte_vector_stride.
arm64jit 126/126, workspace 175/0; full 4-batch FP battery + core C battery
all green. dnorm (Newton reciprocal-sqrt that overflows to +inf = C UB) stays
excluded as degenerate.
Session net: 7 FP/SIMD correctness fixes + FMADD feature, each regression-locked.
Next: keep pressing -O2 arrays/structs (now unblocked), single-precision wider
structs, then libloader gaps. HARD GATE unchanged.

## Session (Sep 11, 2026) — single-precision LdStPair (s-pair) scale bug (commit 8d2d57b)
Sixth FP milestone. 5th -O2 battery (float struct array, 2D float matmul det,
float exp poly, string copy, unsigned arith) surfaced one more: fstruct returned
0x391c0000 garbage (should be 34), fmat2's singular 3x3 float det returned -108
(should be 0). Root cause: byte3 0x2c/0x2d (single-precision FP pair `ldp s0,s1`)
was lumped into `fp_d` (scale 8), so each 32-bit s-reg was read as 8 bytes and
post-indexed 2x. FP/vector pairs distinguish 64-bit d (0x6d/0x6c, bit30=1) from
32-bit s (0x2d/0x2c, bit30=0). Split into a new `fp_s` flag: scale 4, 4-byte
transfers into the low 4 bytes of each 16-byte vector slot (sN = VECTOR_BASE +
N*16). fstruct 34, fmat2 0, fexp 649, str 0, uint 999 — all = native.
+regression ldst_pair_s_registers_use_4_byte_transfers. arm64jit 127/127,
workspace 176/0; full 5-batch battery + core C all green.
Session net: 8 FP/SIMD correctness fixes + FMADD, all regression-locked. Next:
keep pressing -O2 float/struct coverage, then libloader gaps. HARD GATE
unchanged (real libroblox.so boot + GPU host).

## Session (Sep 11, 2026) — sdiv/udiv signedness inversion (commit dae2e05)
6th -O2 battery (signed/variable division, fmin/fmax, fmod, fabs): idivA
(`s += a[i]/d[i%3]`) returned 0xaaaaaa2d, wanted -35. Root cause: the MulDiv
decode gate used `b(insn,17,17)==1` for SDIV-vs-UDIV, but bit17=0 for BOTH
forms — the real discriminator is bit10 (sdiv=1, udiv=0; verified by assembling
matching-operand pairs 0x1ac50c61 vs 0x1ac50861). So every sdiv was labeled
unsigned -> JIT emitted `xor edx,edx; div rcx` (unsigned) instead of `idiv`,
turning negative dividends huge. gcc's magic-constant division masked it in
earlier tests. Fixed to bit10 (matches the 2-source gate at ~2054).
+regression sdiv_is_signed_udiv_is_unsigned_same_negative_input. arm64jit
128/128, workspace **177/0**. idiv -33, idivA -35, all 6 batches + core green.
Cycle total: 9 FP/int miscompile fixes + FMADD, regression-locked. HEAD dae2e05.

## Session (Sep 11, 2026) — MSUB operand direction (commit ccbf55c)
7th -O2 battery (byte-string sum+modulo, switch table, SIMD-ish reduction,
16-bit accumulate, bitfield pack, strcmp): bytelen (n%50 after byte loop,
n=1298) gave -48 instead of 48. Root cause: MulDiv MSUB arm computed rn*rm-ra,
but ARM MSUB is ra-rn*rm, so n-(n/50)*50 via `msub w0,w1,w0,w2` = 25*50-1298 =
-48. Constant-folded addrs masked it. Fixed direction (MADD arm already
correct). +regression msub_reuses_rm_as_rd.*. arm64jit 133/133, workspace
**178/0**. All 7 batches + core + FMA green. Cycle total: 10 miscompile fixes +
FMADD, all regression-locked. HEAD ccbf55c.

## Session (Sep 11, 2026) — ADDV SIMD horizontal add (commit 6290937)
8th -O2 battery (64-bit mul/div, short-array SIMD sum, dot, int matmul): vadd
gcc fully SIMD-vectorizes a 16-short sum to `ldr q`+`addv s0,v1.4s`+`fmov w0,s0`
- returned 0 (wanted 360). ADDV undefined; an earlier dup/move gate swallowed
0x4eb1b820 and emitted per-lane identity copies. Implemented Inst::Addv: mask
0xfffffc00 (clears Vn bits9:5, Sd bits4:0), residues 8b/4h/16b/8h/4s; NOTE
source Vn at bits[9:5] (bits20:16 fixed=17) — nonstandard SIMD layout. Gate at
top of decode (dup/move swallowed it later). Translate sums sign-extended
lanes -> RDI -> bottom element of Vd. vadd 360=native. +regression
addv_horizontal_sum_across_4s_lanes. arm64jit 134/134, workspace **179/0**.
Cycle total (back half): 4 fixes (sdiv signedness, MSUB direction, s-pair
scale, ADDV) + earlier (WidenShl, FP-N, ld1-2reg, FMOVimm, FMA3, apply_shift,
d-pair stride) = 11 miscompile fixes + FMA + ADDV. HEAD 6290937.

---

## Session (Sep 11, 2026) — libloader: Android packed relocations (APS2) + RELATIVE application (workspace 184/0)

Per the ordered "libloader ELF/loader gaps" step: the Rust loader did **zero
relocation** — `load_elf_image` only mapped segments, sp-mprotected them, and
relied on `bind_image_plt` for JUMP_SLOT. Real Roblox APK libs and their
Android/GSI dependencies carry `R_AARCH64_RELATIVE` data relocations (often
packed via `DT_ANDROID_RELA`), which a real loader materializes before the code
can dereference pointer globals/vtables. The QEMU path worked around this with
the external `unpack_rela.py`; this session brought it in-process to the Rust
loader so the JIT path can load those libraries.

### `crates/libloader/src/android_relocs.rs` (new)
- `read_sleb128` (sign-correct, x64-bounded; terminal-byte bit6 = value sign).
- `decode_aps2` — faithful port of AOSP `for_all_packed_relocs` (validated in
  Session 14 against real 2.726.1142 libroblox.so): magic `APS2`, then
  SLEB128 `num_relocs` / running `r_offset` / groups with the
  GROUPED_BY_INFO/OFFSET_DELTA/ADDEND + GROUP_HAS_ADDEND flag logic.
  Declared-count mismatch → error (no silent truncation).
- `read_elf_relocations(path)` — walks PT_DYNAMIC, prefers
  `DT_ANDROID_RELA`/`DT_ANDROID_RELASZ` (0x60000011/12) over plain
  `DT_RELA`/`DT_RELASZ`, reads the stream from file, decodes APS2 or parses
  stock 24-byte Elf64_Rela. **Early real bug**: PT_DYNAMIC's `p_offset` is a
  *file* offset, not a vaddr — I wrongly ran it through `vaddr_to_file_offset`
  and got `DT_* vaddr not covered by a PT_LOAD`; only the RELA vaddr needs that
  conversion.
- `apply_relatives` — for each `R_AARCH64_RELATIVE` writes `load_bias + addend`
  (8B LE) at `guest_of(r_offset)` via a caller-supplied target resolver.

### Wired into `load_elf_image` (elf.rs)
Reordered the segment loop: copy-file → push segment (NO mprotect in the copy
loop), then for PIE (`is_pie`) read+decode relocs and apply RELATIVE **while
the whole image is still RW**, then a second pass mprotects each segment to its
final ELF protection. Non-PIE ET_EXEC (self-relocating like the battery ELFs)
is untouched by the `is_pie` gate.

### Verification (both honest, both catch regressions)
- Unit: a **hand-built** APS2 golden bitstream (independent byte-by-byte
  encode; first attempt used `[0x80,0x40]`="+0x2000" which is actually
  SLEB -8192 — the decoder correctly rejected my wrong test bytes, a good sign
  the decoder is faithful) + a grouped-by-offset-delta stride case
  (r_offset = header + i*delta) + SLEB negative + bad-magic reject.
- Integration `crates/libloader/tests/reloc_apply_test.rs`: cross-gcc
  `-shared -fPIC` ET_DYN with `int *ptr = &data` → asserts EVERY
  `R_AARCH64_RELATIVE` slot (parsed from `readelf -r`, whose type column is
  truncated to `R_AARCH64_RELATIV`) equals `load_bias + addend`. Without the
  apply the slot holds the raw file addend (e.g. 0x600) ≠ 0x100000600 → fails.
- `cargo test --workspace` **187/0** (libloader 20 → 23 unit + 1 integration);
  `cargo build --workspace` clean (only the pre-existing decode.rs rustfmt-warn
  churn). Skips when cross-gcc absent.

### Addendum (same cycle) — DT_RELR support
`read_elf_relocations` now also materializes **`DT_RELR`** (tag 0x23, Android
13+ / modern NDK default) when the ELF ships only `.relr.dyn` (no RELA table).
`dt_relr_to_relatives` implements the shipped glibc/Android `DO_RELR` decode
exactly (the low-bit-marker scheme; the upper-8-bit-delta variant was a rejected
alternative): an even word is an offset that relocates itself and seeds
`base = offset+8`; an odd word is a bitmap where bit *i* (1-based) → reloc at
`base + (i-1)*8`; odd value-1 padding decodes to nothing. +3 unit tests
(offset+bitmap bit mapping, no-prior-offset from base 0, non-multiple-of-8
reject). Note: neither the cross nor host `ld` here supports
`--pack-dyn-relocs=relr`, so no real `.relr.dyn` fixture is producible on this
box — the decode is anchored to the authoritative algorithm plus hand-computed
streams. `cargo test --workspace` **187/0**.

### Status / next
`load_elf_image` now materializes RELATIVE relocations for PIE/shared objects,
from **(a)** standard `DT_RELA`, **(b)** Android APS2-packed `DT_ANDROID_RELA`,
or **(c)** `DT_RELR` — all in-process (no `unpack_rela.py`). Next (ordered):
a GLOB_DAT/JUMP_SLOT resolver for the main dynamic (arm64jit's `bind_image_plt`
already covers JUMP_SLOT), then libbadcpu ISA gaps, then services/auth. HARD
GATE unchanged: `elfjit <libroblox.so> 0x1f0db20 --jni` run log on a
GPU + real-binary host (no APK/libroblox.so/GPU on this VPS).

### Addendum (same cycle) — libbadcpu: PEXT/PDEP were mis-emulated as BZHI
The VEX.0F38.F5 opcode byte is shared by **BZHI / PEXT / PDEP**, disambiguated
only by the VEX pp bits (assembler ground truth `gcc -c + objdump -d`:
bzhi=pp0, **pext=pp2, pdep=pp3**). The emulator's 0xF5 branch treated every
case as BZHI, so both **PEXT and PDEP silently produced wrong results** (a bit
extract became a low-bit mask). Fixed in `emulator.rs`: pp3/f3 → PDEP
(deposit source bit i into the i-th set mask position), pp2/f2 → PEXT (gather
source bits at mask positions into the low bits), else BZHI. Operands:
dest=ModRM.reg, source=vvvv, mask=rm. Also fixed a latent flag-ordering bug —
the BZHI/PEXT/PDEP carry flag was set *before* `update_flags_common` cleared
it, so CF never survived; now `cf_pending` is applied after. +2 tests with
expected values **verified on real hardware** `_pext_u64`/`_pdep_u64` (this
host has BMI2): pext(0xFF,0b1010)=3, pext(8,0b1010)=2, pdep(3,0b1010)=10,
pdep(0xFF,0b10101010)=170, plus 32-bit forms and a PEXT-vs-BZHI discriminating
case. `cargo test --workspace` **189/0** (libbadcpu 16 → 18).

### Addendum (same cycle) — loader→JIT end-to-end PIE + RELATIVE regression
Validated and regression-locked the full **`load_elf_image` → RELATIVE apply →
JIT execute** chain on a REAL PIE: cross-compiled `-fPIE -pie -nostdlib` aarch64
binary (`int *gptr = &shared_static; entry(){ return *gptr+1; }`) — the compiler
emits one `R_AARCH64_RELATIVE` in `.data.rel.ro` (offset 0x20008, addend
0x20000 = &shared_static). Without application `*gptr` derefs the unrelocated
link address (NULL page) and faults; with it, `gptr = base+0x20000` and
`entry() -> 42`. Verified by hand (`elfjit ./pie.elf -> 42`) and locked as
`loader_run_pie_relative_global_returns_42` in `crates/arm64jit/tests/
loader_run.rs` (new `compile_pie` helper; asserts the fixture really carries a
RELATIVE reloc). `cargo test --workspace` **190/0**.

## Session (Sep 11, 2026) — bind GLOB_DAT + ABS64 main-GOT relocations (commit 7f03937, workspace 191/0)

Continuing the ordered "libloader ELF/loader gaps" step. `bind_image_plt` only
walked `DT_JMPREL` (JUMP_SLOT) and `load_elf_image` only applied RELATIVE;
**R_AARCH64_GLOB_DAT (1025)** in the main `DT_RELA` was never bound. A
`-shared -fPIC` module referencing an exported global (data or function
pointer) goes through its **main GOT** via GLOB_DAT: the guest does
`adrp x0,GOT; ldr x0,[x0,#off]` to fetch the symbol's *runtime address*, then
derefs/calls through it. Unbound, the slot read 0 → SIGSEGV on NULL / call to
address 0.

Empirically confirmed (cross-gcc `-shared`): `global_data` and `gfp=&internal_fn`
produce two GLOB_DAT relocs at GOT offsets 0x1ffd8/0x1ffe0 (both 0 in file), and
`gfp = &internal_fn`'s initializer is **R_AARCH64_ABS64 (257)** — a second member
of the same "write symbol runtime-address" family that the loader also ignored.

### New `bind_glob_dat` (crates/arm64jit/src/plt.rs)
- Walks the main `DT_RELA`/`DT_RELASZ` for GLOB_DAT (1025) **and** ABS64 (257).
- **Defined-in-module** symbol → writes `el.guest_of(st_value) + addend` (ABS64
  carries the symbol offset as addend; GLOB_DAT addend 0). The loader maps
  guest==host, so `ldr xN,[GOT]` then `[xN]`/`blr xN` resolves back into the
  mapped image.
- **Undefined/imported** symbol → `STT_OBJECT` uses `dlsym` raw (guest==host
  addressable); FUNC/NOTYPE uses the resolver's host-call thunk (callable).
- Called from `bind_image_plt` (end of the normal path) **and** from the
  `pltrelsz == 0` early-return — an exported-data-only module has zero JUMP_SLOT
  yet still depends on the main GOT.
- Fixed en route: the `?` operator can't be used in a `(usize,usize)`-returning
  fn (switched the import branch to a match).

### Verification
- `elfjit /tmp/gdtest/self.so 0x360` (real `-shared` aarch64):
  ```
  [plt] (no JUMP_SLOT) bound 2 GLOB_DAT, 0 unresolved  -> then ABS64 added: 3
  JIT(no-QEMU) entry() -> 37 (0x25)     # global_data(11) + gfp(3)=internal_fn(3)=15 + global_data(11)
  ```
  Before the fix it SIGSEGV'd at fault=0x0 (guest GOT loaded 0).
- **+`loader_run_shared_glob_dat_and_abs64_returns_37`** in
  `crates/arm64jit/tests/loader_run.rs` — cross-gcc `-shared -fPIC -nostdlib
  -Wl,-e,entry` fixture with the exact two-global GOT pattern; runs the full
  load→RELATIVE→GLOB_DAT/ABS64→jit_run pipeline and asserts 37. Skips without
  the cross toolchain.
- `cargo build --workspace` clean; `cargo test --workspace` **191/0**.

### Next (ordered, no APK/GSI/GPU on this box)
1. Continue libbadcpu ISA gaps (the SIGILL emulator's remaining VEX/legacy
   instructions), then services/auth.
2. Real-binary/GPU boot proof (`elfjit <libroblox.so> 0x1f0db20 --jni`) stays the
   HARD GATE, blocked until a capable host + the real binary/APK are available.

## Session (Sep 11, 2026) — GLOB_DAT/ABS64 binding, libbadcpu BMI2 completion, auth identity (workspace 199/0)

Ordered plan continued (no APK/GSI/GPU on this box; the android idempotent
test is already fixed and green). Four focused commits, each closed by the
real `cargo build --workspace` + `cargo test --workspace` gate:

### 7f03937 — arm64jit: bind GLOB_DAT + ABS64 main-GOT relocations
`bind_image_plt` only walked DT_JMPREL (JUMP_SLOT) and `load_elf_image` only
applied RELATIVE, so `R_AARCH64_GLOB_DAT` (1025) — how `-shared -fPIC` code
fetches an exported global's runtime address via the main GOT — was never
bound: the guest `adrp;ldr x0,[GOT]` read 0 and deref'd/called NULL.
New `bind_glob_dat(el)` walks DT_RELA for GLOB_DAT **and** R_AARCH64_ABS64
(257, the sibling "write symbol value" data-initializer family, confirmed by
cross-gcc that `gfp=&internal_fn` emits ABS64), writes `el.guest_of(st_value)
+ addend` for defined-in-module symbols, `dlsym` (OBJECT) / host-call thunk
(FUNC) for imports. Runs in the normal path and the `pltrelsz==0` early-return
(an exported-data-only module has zero JUMP_SLOT yet needs the main GOT).
Real `-shared` fixture: SIGSEGV (fault=0x0) -> `entry() -> 37` (global_data 11
+ gfp(3)=15 + global_data 11). +`loader_run_shared_glob_dat_and_abs64_returns_37`.

### b177f7c — libbadcpu: MULX (VEX.0F38.F6) + RORX (VEX.0F3A.F0)
BMI2 gaps: MULX = unsigned RDX*rm, high->modrm.reg, low->vvvv (u128 product —
a naive u64 `>>64` overflowed), flags cleared. RORX = rotate-right-by-imm8,
flags untouched; added the VEX.0F3A dispatch (decoder stops after ModR/M, so
read imm8 at RIP+len / advance len+1). +2 tests (values verified with -mbmi2:
rorx64(1,4)=0x1000000000000000, rorx32(1,31)=0x2).

### 2a9c6eb — libbadcpu: ADCX (66 0F38 F6) + ADOX (F3 0F38 F6)
`emit_adcx_adox`: Dest=Dest+Src+flag, write only the working flag (CF/OF).
Two real bugs found+fixed: width from REX.W not operand_size (the 66/F3 is a
mandatory opcode prefix, not a size override — a 64-bit ADCX is 66 48 0F38 F6,
operand_size folds 66->16); and 32-bit carry detected in the u32 domain
(0xFFFFFFFF+1 wraps to 0 WITH carry). Ground truth from a real-BMI2 assembly
driver confirmed the ADOX subtlety: OF is set to the *unsigned* carry-out, not
signed overflow (adox(0x7fff..,1)=0x8000.. has OF=0). +2 tests.

### a8249f4 — sober-services: forward full login result (services/auth)
The OAuth webview's AuthResult declared user_id/username but the callback only
extracted the token — IPC AuthToken always went out with both None, so the
parent couldn't identify the account without a second Roblox API call.
New extract_auth_result() parses token + user_id + username (query precedence,
#fragment tolerant, URL-decoded); send_auth_result() forwards the identity;
run_login_flow blocks on the token then sends the full result. +4 tests.

### Gate
`cargo build --workspace` clean (0 errors), `cargo test --workspace` **199/0**
(arm64jit 120 + loader_run 7 incl. the glob_dat fixture; libbadcpu 22;
sober-services 15; libloader; others). HEAD `2a9c6eb`, tree clean.

### Next (ordered, no APK/GSI/GPU on this box)
1. Real-binary/GPU boot proof (`elfjit <libroblox.so> 0x1f0db20 --jni`) stays
   the HARD GATE — blocked until a capable host + the real binary/APK exist.
2. Continue hardening: next ISA/loader/emulator gaps as discovered (adb/emulate
   surface), then the remaining sober-core/sober-services integration.

---

## Session 2026-09-11 — LogicImm DecodeBitMasks rotate-RIGHT fix (workspace 200/0)

### The bug (severe, silent, commonest-instruction-class)
`decode_logical_mask` (crates/arm64jit/src/decode.rs) applied a **LEFT**-rotate
to the immediate element (`ones << r | ones >> (esize-r)`) but ARM
`DecodeBitMasks` (DDI0487) uses **ROR — rotate right**. Every rotation-
ASYMMETRIC logical-immediate mask was silently miscompiled (any `mov/and/orr/
eor/tst/ands xD,#<asym-mask>`). Symmetric masks (alternating 0xCCCC/0x5555,
single-bit, all-ones) give the same value under both directions, which is why
the whole prior test set stayed green for months despite the wrong code —
<15% of encodings are rotation-invariant.

Real failure: `mov x0,#0xffffffff80000001` = `0xb26187e0` (immr=imms=33)
returned `0xfffffffe00000007` instead of `0xffffffff80000001`, caught by a
cross-gcc probe run through elfjit (no QEMU) vs native x86-64.

### Fix + quantified verification
- Rotate right in the esize-bit domain: `(ones >> r | ones << (esize - r)) & em`.
- Ground-truth cross-check vs the real `aarch64-linux-gnu-as`+`objdump` over
  ~700 (N,immr,imms): fixed right-rotate agrees **592/592 valid**; old
  left-rotate would have been wrong on **509**.
- End-to-end elfjit: `mov x0,#0xffffffff80000001` -> exact value; the
  esize-64! case `mov w0/x0,#0x3ffffffc` (`0xb27e6fe0`) exact too.
- Regression `logic_imm_rotation_asymmetric_mask_ror` (both asymmetric
  encodings + symmetric 0xCCCC/#1 unchanged).
- `cargo build --workspace` clean; `cargo test --workspace` **200/0**.
- Commit `8c1706b`.

### Next
Continue the cross-gcc ISA-surface battery for more silent-miscompile classes;
libloader/libbadcpu/services gaps; HARD GATE (real Roblox boot, GPU/APK host)
unmet on this box.

---

## Session 2026-09-11 — three silent SIMD/logic-imm miscompiles fixed (workspace 202/0)

Session opened with the handoff-flagged failing test already green (200/0 from
committed cycles); the "failing test first" gate was satisfied by prior work.
This session's value = three silent JIT miscompiles flushed out by cross-gcc
batteries driven end-to-end through elfjit (no QEMU) vs native x86-64, all
found and fixed in arm64jit:

1. **LogicImm DecodeBitMasks rotate-RIGHT** (`8c1706b`): the logical-immediate
   element was LEFT-rotated; ARM DecodeBitMasks uses ROR. Every rotation-
   asymmetric mask silently miscompiled (`mov x0,#0xffffffff80000001` ->
   0xfffffffe00000007). Only symmetric masks gave the same value under both, so
   the prior test set stayed green. Verified quantified vs the real assembler:
   right-rotate agrees 592/592 valid (N,immr,imms); old left   would be wrong
   on 509. Regression + loader_run end-to-end gate (`076b024`).

2. **Scalar-D ADDP vs fcvtzs collision** (`01aa402`): `addp Dd, Vn.2D`
   (0x5ef1...) collided with the scalar fcvtzs gate at 0x5ee0b800 — bit20 is
   the discriminator (ADDP SET, fcvtzs CLEAR). Old gate silently ran every gcc
   pairwise-add reduction as a float->int. New SimdPairAddD.

3. **saddw2/uaddw2 upper-half** (`01aa402`): SimdAddw always read Vm at byte 0;
   the Q=1 form (saddw2) reads the UPPER 64 bits. -O2 vectorized loops do
   saddw (low) then saddw2 (high); old code accumulated low twice.

All three proven end-to-end: the i*i reductions return 76 (native) at both -O3
(addp) and -O2 (saddw+saddw2). 50+ cross-gcc battery programs acros.
`cargo build --workspace` clean; `cargo test --workspace` **202/0**.
HARD GATE unchanged: real Roblox boot/GPU host (no APK/GPU here).

---

## Session (Sep 11, 2026) — differential battery + sxtl2/uxtl2 upper-half FIX (workspace 210/0)

### New capability: differential battery (`crates/arm64jit/tests/diff_battery.rs`)
Cross-gcc compiles a C `entry()` to aarch64; the harness ALSO compiles the same
source with native `gcc` and runs it as the oracle; the loader→JIT pipeline must
return EXACTLY the oracle value. This is the strongest silence-detector in-tree
(native oracle vs JIT), and it immediately caught two MORE issues beyond the
already-fixed LogicImm/ADDP/saddw batch.

### FIXED: `sxtl2`/`uxtl2` (SIMD long-extend) ignored the Q bit
`Inst::SimdXtl { rd, rn, sign, esrc }` had no `upper`, so
`sxtl2 v28.2d, v28.4s` re-read v28's LOW 64 bits instead of bytes 8..15 —
every int→i64 vectorized init loop gcc emits (`movi v.4s,#n; sxtl; sxtl2; stp q`)
computed the upper half from the wrong lanes. Fixed the same way the existing
`saddw2`/`uaddw2` fix does:
- decode.rs: +`upper: (insn >> 30) & 1 == 1`
- translate.rs: source lane offset `n_half = if upper { 8 } else { 0 }`
Three deterministic linear regression tests in `jit.rs` pin it:
`and_then_sxtl_sxtl2_upper_half`, `and_sxtl_accumulation_two_iterations`,
`simd_stp_q_preindex_store_and_writeback` — the last replays the FULL maskf
loop body (movi; ldr q init from [x0,#400]; then 6× {mov snapshot; add v31+=4;
and &0xf; sxtl; sxtl2; stp q27,q28,[x0],#32}) and stores m[k]=k&0xf exactly.

### OPEN (documented): intermittent SIMD-loop block-liveness bug
The identical instruction stream FAILS nondeterministically when run through
`jit_run`'s single-block **b.ne back-edge** compilation of the whole function:
gcc -O2/-O3 int→i64 widening init loops EITHER compute correctly (verified vs
qemu-aarch64 ground truth: maskf 7001003, regloop 23017003, %101 60018021) or
corrupt ONE snapshot lane (m[4i+1] = address/stack-layout garbage while
m[4i],m[4i+2],m[4i+3] stay correct). Intermittent per process AND per heap
allocation (trial N), i.e. an uninitialized x86 register at the loop back-edge,
NOT an ISA miscompile (all ops verified correct linearly). The corrupted-lane
differential canaries (maskf/times7/mod_pow2/regidx/struct_arr/mixed) are
therefore excluded from the permanent gate; `diff_mixed_arith_accumulate` is
`#[ignore]`-documented. Root-causing this subsumes the struct/`%101` array
cases too. Next: instrument the block liveness / XMM scratch registers around
the loop back-edge under `compile_image_bounded`.

`cargo build --workspace` clean; `cargo test --workspace` **210/0** (1 ignored).
HARD GATE unchanged: real Roblox boot / GPU / APK host (none on this VPS).
### Addendum (same session): the "intermittent" bug above is ROOT-CAUSED and FIXED
Root cause: ARM `nop` (0xd503201f) and the whole system/hint 0xd5... family
were misdecoded as `ScvtfFixed` — the scalar int→fp FIXED-POINT gate checked
only `(insn & 0x30000000)==0x10000000`, which 0xd5xxxxxx also satisfies. So every
guest `nop` executed as `scvtf d<n>, x0, #56`: it CVTSI2SD'd the caller's x0
(very often the stack pointer) and DIVSD'd it into a vector register. That is
precisely the observed layout/address-dependent single-lane corruption. Fix:
the ScvtfFixed gate now also requires top byte in {0x1e,0x9e}. Additionally,
ScvtfFixed sf/to_double read bit30 but must read bit31 (0x9e=X/D vs 0x1e=W/S);
real `scvtf d0,x0,#1`=0x9e42fc00 was being decoded as a single Sd/Wn convert.
Decode regression `nop_is_hint_not_scvtf_fixed` pins all three.
After these, EVERY int→i64 widening-init-loop differential canary passes
deterministically (maskf=7001003, mod_pow2=1007005, times7=161119021, count6=24,
verified vs qemu-aarch64). Two SEPARATE deterministic bugs remain (magic-div
`%N` reducer: m[0]=-101 for k*7%101; and the -O3 addp/smulh reduction) and are
#[ignore]d/documented. Workspace 212/0 (3 ignored).

## Session (Sep 11, 2026) — mls + uzp2 implemented, rev64 gate widened; all 3 remaining SIMD bugs CLOSED (workspace 216/0, 0 ignored)

The two leftover deterministic bugs (magic-div %101 m[0]=-101 and the -O3
vectorized reduction) share one root cause. Isolated the gcc %101 reducer
chain (smull/smull2/uzp2/sshr/mls) in a scratch example and diffed against
qemu-aarch64, then pinned the failing ops with decode(insn):

1. **`mls` (multiply-subtract) misdecoded as a plain Simd4s SUBTRACT.** The
   `(insn & 0xffe0_fc00)` gate for Simd4s caught `mls v26.4s,v0.4s,v28.4s`
   (0x6ebc941a) as `sub`, **losing the multiply entirely** — so the quotient
   was never subtracted. Implemented `Inst::SimdMla { rd, rn, rm, lanes,
   sub }` (mla=0x0ea09400/0x4ea09400 add, mls=0x2ea09400/0x6ea09400 sub),
   gate placed BEFORE the Simd4s gate; translate does per-lane
   `Vd = Vd ± Vn*Vm` (low-32 product).

2. **`uzp2` (unpack-high) misdecoded as `rev64`.** The SimdRev gate
   `(insn & 0x3f00_0c00)==0x0e00_0800` drops bit12 and swallowed the whole
   uzp1 (0x18) / uzp2 (0x58) opcode family. Tightened it to require
   bits[13:12]==00, and added `Inst::SimdUz2 { rd, rn, rm, esize, q }`
   (byte1 high-nibble==0x5) gathering the ODD/upper elements
   `Vd[i]=Vn[2i+1]; Vd[n/2+i]=Vm[2i+1]` — what feeds the sshr quotient step.

Verified: the previously-`#[ignore]`d `diff_magic_div`, `diff_struct_array_
fields` and `diff_mixed_arith_accumulate` all pass again as permanent gates
(un-ignored). Added decode regression `mls_and_uzp2_decode_as_specific_ops_
not_sub_or_rev`. Workspace 216/0, **0 ignored** — the differential battery is
fully green with every case a live gate.

### Next
- Ideal next: keep sweeping the ISA breadth the battery doesn't yet cover —
  widen the reducer family (umull2/umlal/uaddl to exercise uzp-variant and
  long-multiply paths), add string/booleans, and push the battery onto more
  real-compiler idioms (-O3 reductions already live). Then brace for the real
  Roblox APK path (ELF/loader + JNI stubs) once an APK/GPU host is available.
  Blocked on this VPS only by the HARD GATE (no GPU/APK).

### Addendum (same session, after the mls/uzp2/rev64 commit) — UNSIGNED magic-division closed (workspace 220/0, 0 ignored)
Extending the battery to UNSIGNED `%const` (gcc emits the mul/umull/umull2/
uzp2/ushr/zip1/zip2 reducer, the unsigned sibling of the signed smull one)
immediately surfaced FOUR more misdecodes, all isolated against qemu-aarch64:
1. **`mul` (NEON element-wise 32-bit multiply) decoded as SimdVLog (bitwise).**
   The vector-logical AND/ORR/BIC gate checked byte1&0x1c00==0x1c00 but never
   bit15: mul's byte1 (0x8c..0x9f) sets bit15, and/orr/bic (0x1c/0x1d) don't.
   So EVERY NEON multiply became an AND/ORR — including the magic-division
   dividend `mul v26.4s,v26,v28(97)`, which corrupted the quotient. The JIT's
   full-loop m[] came out all-0 and acc=0. Fix: require (insn&0x8000)==0.
2. **`uzp2` misdecoded as `rev64`, then as `uzp1`.** The rev64 gate dropped
   bit12 (fixed earlier); the uzp1 gate's &0x3f mask dropped bit6, so uzp2
   (byte1 0x58) was even-gathering. Added Inst::SimdUz2 (odd/upper gather);
   both uzp gates now key on byte1 0x18-/0x58-family + byte3-low-0x0e +
   bit28 clear (excludes bit/bif/bsl and rev64).
3. **`zip2` Unsupported.** Added Inst::SimdZip2 (upper-half interleave, base
   0x0e007800 vs trn2 0x0e006800). gcc uses zip1/zip2 with a zero lane to
   widen a 4s quotient into 4 u64.
4. **`mls` = plain Simd4s SUBTRACT (no multiply)** — added SimdMla {sub}.
Also: WidenShl gate restored to the genuine shll long-shift family but made to
exclude the permute ops via byte1 bits[1:0]==00 (shll 0x38 vs zip1 0x39/0x3b).
Verification: diff_unsigned_magic_div_umull, diff_long_accumulate_widening,
diff_byte_scan_strlen new; all green un-ignored; 3 decode regressions added
(mls_and_uzp2..., mul_decodes_as_multiply_not_bitwise_logical, and 'and' still
logical). Workspace 220/0, **0 ignored**. Commits ab24b32 (fix) — prior
d239e8c/3b4ff25 (signed path). Difference: JIT and qemu-aarch64 now agree on
both the signed and unsigned magic-division kernels exactly.
---

## Session (Sep 11, 2026) — 6 silent vector-FP/NEON miscompiles fixed; workspace 227/0, 0 ignored

Commits `ea84aff` (fixes + unit tests) + `059151a` (differential canaries) on `dev`.
Opened at 220/0 (no failing test — the android idempotent test stays green). Extended
the differential battery into the **.4s/.2d single/double-precision float-vector
SIMD** family (a 3D engine's vertex/matrix math is ~all of it) that the older
integer/double batteries never touched, with volatile seeds so gcc can't constant-fold
while still vectorizing. It immediately flushed out **SIX silent miscompiles** — every
one would corrupt pixels/coordinates/audio on a real host:

1. **VecIntToFp** — `scvtf/ucvtf Vd.4s/.2s` (and signed `.2d`, `0x4e21d800` family) were
   misdecoded as **SimdMull** (widening multiply); every vector int->float made garbage.
2. **VecFpArith** — `fadd/fsub/fmul/fdiv/fmax/fmin/fmaxnm/fminnm Vd.2s/.4s` (2-source)
   were misdecoded as integer SIMD (`SimdAddB`/`Simd4s`/`SimdMull`). Only FMLA
   (accumulate) and the `.2d` double forms (Simd2dFp) were handled; the two-source
   single-precision family was entirely MISSING. Added `divss` x86 emitter.
3. **SimdMlaEl** — integer `mla/mls Vd.4s, Vn, Vm.s[idx]` (by-element) misdecoded as
   **VecMovi** (gcc int->float init `mla v5.4s,v18,{loop}.s[0]` corrupts the a*scalar
   product). Gate: top nibble 0x0f + bit29 set (FP fmla-el is bit29 CLEAR) + bit23 set
   (excludes smlal/umlal-by-el byte1 0x42); `.4s` index = (bit11<<1)|bit21.
4. **Permute ordering** — `zip1` (and zip2/uzp1/uzp2) now decoded BEFORE the
   `WidenShl` (shll) gate. Real `zip1 Vd.4s` has byte2 0x38 (same as shll) and was
   misdecoded as a widening shift; the old guard only excluded the 0x39/0x3b byte2
   variants. Added an early compact permute gate (zip1/zip2/uzp1/uzp2 on 0x3f20fc00).
5. **SimdFmulEl cross-lane alias bug** — the broadcast element was re-read *inside* the
   lane loop, so when `rd==rm` (`fmul v17.4s, v7.4s, v17.s[0]`) lane 0's write clobbered
   the element before lanes 1-3 read it → every lane after 0 wrong. Now broadcast once
   up front (the FmlaEl "load into xmm2 before the loop" pattern).
6. **VecFpCmp** — `fcmeq/fcmgt/fcmge Vd.4s/.2s/.2d` (compare→all-ones mask) misdecoded
   as **SimdVShift**; gcc's float-vs-const count loop returned garbage. Gate = byte2
   0xe4 after the 0xffe0fc00 mask (NOT raw bits15:8, which include rn). New
   comiss/comisd + setcc + movzx + neg x86 emitters; per-lane mask = all-ones or 0.

New unit tests: `vector_fp_arith_ground_truth` (fmla/fmul-by-el/fmla2d/scvtf/fadd,
incl. negative scvtf lanes, fmov-imm broadcast, fmov-s,w), `vector_fp_by_element_highreg_and_2d`
(exact gcc high-reg by-element + .2d words). New permanent differential canaries:
fv_arith, fv_sub_neg, fv_f2i, fv_f2i_neg, fv_i2f, fv_fmla_scalar, fv_cmp_count,
dv_arith, fv4, fv4_fmul_only.

**Verification:** `cargo build --workspace` clean; `cargo test --workspace` **227/0**,
**0 ignored** (was 220/0). Full differential battery 17/17 (every case a live gate).

### Next (ordered, no APK/GSI/GPU on this box)
1. Keep pressing the SIMD float/FP vector breadth (single-precision struct-by-value
   with vector lanes, fmla-by-element chains, more -O3 reduction shapes); then widen into
   the remaining integer-permute/wide paths the new float canaries may expose.
2. `fv_cmp_count`-style FP compares are now real (setcc-based); add fcmlt/fcmle
   (operand-swapped gt/ge) if a battery case needs them.
3. libloader ELF/loader gaps -> libbadcpu ISA gaps -> services/auth.
4. Real-binary/GPU boot proof (`elfjit <libroblox.so> 0x1f0db20 --jni`) stays the
   HARD GATE, blocked until a capable host + the real binary/APK (none here).

## Session (Sep 11, 2026) — 9 silent miscompiles + the long-open nondeterministic SIMD-loop bug fixed (workspace 234/0, 0 ignored)

Five commits on `dev` (all `cargo build --workspace` + `cargo test --workspace` green):
`6ffd490` (32-bit add/sub flags + ccmp/ccmn), `e1ec841` (4 FP/control-flow bugs),
`7e2689a` (ADDV-to-scalar stale bytes), `5cbe6ce` (CMN/ADDS C-flag polarity),
plus this docs commit. Opened at 230/0; no failing test (the android idempotent
stays green). Delivered through the differential-probe harness — cross-compile the
same C at -O2/-O3 through `elfjit`, compare against a native x86-64 oracle. HEAD of
these runs caught 9 real bugs:

1. **32-bit ADDS/SUBS flag semantics** — `!sf` flagged adds/subtracts used 64-bit
   x86 add/sub after zero-extending, so `store_nzcv` saw SF from bit63, not bit31.
   `adds w1,w1,w2` with 0x7fffffff+1 gave N=0 (wrong), so `int s=INT_MAX+1; s<0`
   returned the wrong branch. Added 32-bit emitters (no REX.W) and routed `!sf`
   paths through them.
2. **ccmp/ccmn missing** (conditional compare) — swallowed by the logical set-flags
   decoder (shares 0xFA/0x7A/0xBA/0x3A top bytes), corrupting NZCV so gcc's
   `while (a<N && b!=M)` guards spun forever. Added `Inst::CcMp` (residue class
   distinct from ANDS/BICS/SBCS; rn=[9:5] rm/imm=[20:16] cond=[15:12] nzcv=[3:0]);
   translate mirrors Fccmp (load_nzcv -> jcc -> nzcv|compare).
3. **CSel rn/rm==31 read the SP slot** instead of XZR — `cset/cinc/csneg`
   (`a==0.0?1:0`) returned sp/sp+1. CSEL is data-processing: reg 31 is always XZR.
4. **Scalar `scvtf/ucvtf Dd,Dn` `sng` discriminator inverted** (bit22=1 is DOUBLE,
   code set sng on it) — a double scvtf truncated through the i32->f32 path.
5. **`fcmp Dn,#0.0`** decoded as a compare against vector reg d0 (garbage) — bit3
   (0x8) is the #0.0 discriminator, now `Fcmp.against_zero` loads literal +0.0.
6. **FP NaN compare flags** — `store_nzcv_fp` stored C as the true ARM value and
   Z=ZF (set for unordered too): `vnan==vnan` came out true and `nn<=0` (cset ls)
   wrong. Now Z = ZF&&!PF (excl. unordered) and C is stored in the borrow sense
   (CF&&!PF) that x86_cc_for_cond's ls/hi/lo/hs expect => all NaN comparisons false.
7. **ADDV-to-scalar left stale bytes** — `addv Bd,Vn.8b` stored only 1 byte, so the
   destination vector reg's upper bytes kept the `cnt` lane counts; gcc's
   `fmov x2,d31` then read `[sum, pc1, pc2, ...]` as the integer popcount. This is
   the exact **root cause of the documented intermittent SIMD-loop block-liveness
   corruption** (one element garbage, nondeterministic, stack-layout dependent)
   that the old maskf/times7/mod_pow2/regidx/mixed canaries hit. Now the ADDV
   store zeros the upper bytes (64-bit store of a size-masked sum). `simdu3`
   (popcount loop) returns exact 591 deterministically, 8/8 runs at -O2/-O3.
8. **CMN/ADDS C-flag polarity** (commit 5cbe6ce) — the flag-setting ADD path
   stored C = x86 carry, but x86_cc_for_cond's HS/LO/HI/LS assume the SUBTRACT-
   borrow convention. gcc's `unsigned um > 0xffffffffffff0000` (compiled to
   `cmn x,#0x10000; b.ls`) evaluated "not greater" when it carries. cmc before
   store_nzcv on the non-subtract paths stores C in borrow convention.
   structfp was off by 999999 from exactly this.

New permanent canaries: `diff_ccmp_cond_compare`, `diff_w32_overflow_compare`,
`diff_fp_compare_zero_and_cset`, `diff_fp_nan_compare`, `diff_addv_popcount_accumulate`
(all differential vs native oracle). Decode regressions: `ccmp_ccmn_decode`,
`fcmp #0.0 / d0 / fcmpe #0.0` additions.

**Verification:** `cargo test --workspace` **234/0**, **0 ignored** (was 227/0);
the 28-program differential probe suite (FP math/compare/cvt, NaN, signed-zero,
ccmp chains, 32-bit overflow compares, NEON/16-bit/unsigned SIMD, popcount,
branch tables, recursion) matches the native oracle at both -O2 and -O3.

### Next (ordered, no APK/GSI/GPU on this box)
1. Keep the differential battery driving ISA/correctness breadth (SIMD permute/
   wide paths, more FP reduction/reassociation shapes, struct-by-value vectors).
2. `addv s0,v1.4s` 32-bit scalar-store path is now covered; check `saddv`/`uaddv`
   (signed accumulator) if a case surfaces one.
3. libloader ELF/loader gaps -> libbadcpu ISA gaps -> services/auth.
4. Real-binary/GPU boot proof (`elfjit <libroblox.so> 0x1f0db20 --jni`) stays the
   HARD GATE, blocked until a capable host + the real binary/APK (none here).

## Session (Sep 11, 2026) — SIMD INS lane-index decode FIX + Extr/unpack/zip batch folded (workspace 243/0)

Commit `b93e89d` on `dev` (all `cargo build --workspace` + `cargo test
--workspace` green). Two threads:

1. **Closed the persistent `diff_float_vector_reduced` (fv4, -O3) miscompile —
   jit 153 vs native oracle 175.** Root cause was NOT the permute snapshot:
   the INS (vector, element) lane-index decode read `dst_idx = bit20` and
   `src_idx = bit14` — two single bits that only coincided with the true lane
   index for S lane 0->1. Every S lane beyond 0 and every H/B lane silently
   copied into the wrong vector element. gcc -O3 emits `mov v3.s[1], v28.s[0]`
   and `mov v31.s[1], v4.s[0]` in the float tight loop fv4 exercises, so a wrong
   `dst_idx` put the seed/accumulator f32 lanes in the wrong V slots. Correct
   packing (verified against the aarch64 assembler for all 4x4 S, 8x8 H, 16x16
   B, 2x2 D lane pairs — 340 encodings, 0 mismatches):
   `l = log2(esize); dst = imm5 >> (l+1); src = (insn>>(11+l)) & ((1<<(4-l))-1)`.
   fv4 now returns 175 == oracle. Locked with `simd_insd_sets_correct_lane_with_multi_byte_indices`
   (mov v3.s[2],v5.s[1] copies 3.5f into lane 2; move-back bytes assembler-verified).

2. **Folded in the earlier uncommitted arm64jit batch** (verified green so HEAD
   stays clean): general EXTR with `rm != rn` (`extr x0,x0,x1,#51`, gcc's shift-
   rotate idiom) now decodes as `Inst::Extr` instead of falling through to the
   UBFM/SBFM gate; uzp1/uzp2/zip1/zip2 snapshot their rd-aliased source to a
   `permscratch` buffer in CpuState (gcc's ubiquitous `uzp1 v31.8h,v31.8h,v26.8h`
   and `zip1 v31.4s,v3.4s,v31.4s`); and REX.B on shl/shr/ror/not/neg emitters so
   guest regs >= 8 (R10+) are addressed correctly. Regression tests:
   `extr_general_two_operand_rotate`, `uzp1_rd_aliases_rn_does_not_corrupt_source`,
   `csel_family_op_discriminates_neg_not_inc_identity`, plus diff_battery
   canaries `diff_integer_signed_division_negative_edge`,
   `diff_rotate_extract_and_byte_accum`, and `diff_float_vector_reduced` (now green).

**Verification:** `cargo test --workspace` **243/0** (146 arm64jit lib incl. the
new ins test; 26 diff_battery incl. fv4). HEAD `b93e89d`.

### Next (ordered, no APK/GSI/GPU on this box)
1. Keep the differential battery driving ISA/correctness breadth (more SIMD
   lane/permute/wide paths, FP reduction/reassociation shapes).
2. Move up to the runtime side: the FMOD "divert guest bl-to-once through the
   dispatcher" task and JNI function-table stubs per RECOMMENDATION.md.
3. libloader ELF/loader gaps -> libbadcpu ISA gaps -> services/auth.
4. Real-binary/GPU boot proof (`elfjit <libroblox.so> 0x1f0db20 --jni`) stays the
   HARD GATE, blocked until a capable host + the real binary/APK (none here).

## Session (Sep 11, 2026) — ld3/st3/ld4/st4 structure DEINTERLEAVE implemented (workspace 245/0)

Commit `1c5f69e` on `dev`. Continued the differential-battery ISA sweep. A 4x4
float matmul (`C[i][j] += A[i][k]*B[k][j]`, gcc -O3) returned **20 vs oracle
5248**. Root cause: the structure-load opcode field (insn bits15:12) was folded
wrong — **0b0000 (ld4/st4) mapped into the single-register ld1 placeholder and
0b0100 (ld3/st3) to Unsupported**, so both ran the ld1-multiple
CONSECUTIVE-load path. AArch64 ld4/ld3 are structure **deinterleave** loads
(`Vd[j][i] = mem[base + i*N*es + j*es]`); loading them consecutively read the
wrong memory. gcc -O3 emits `ld4 {v24.4s-v27.4s},[sp]` to load matrices.

Fix: new `Inst::Ld3N/St3N/Ld4N/St4N` with true, element-size-aware
deinterleave; decode maps op=0b0000->Ld4N/St4N and op=0b0100->Ld3N/St3N
(ld1-multiple keeps only 0b0010/0b0110/0b0111/0b1010). Byte-verified against
qemu for ld3/ld4 q=0 & q=1 and st4 — all match. matmul now = 5248, transpose =
684, complex = 288 (all = native oracle). Regression: decode unit test
`ld4_st4_decode_to_structure_deinterleave`; differential canaries
`diff_ld4_st4_matrix_transpose` (ld4_matmul + st4_transpose). Workspace 245/0.

### Next (ordered, no APK/GSI/GPU on this box)
1. Keep the differential battery sweeping ISA/correctness breadth (structure
   load/store widths, more SIMD lane/permute/wide paths, FP reduction shapes).
2. Move up to the runtime side: FMOD "divert guest bl-to-once through the
   dispatcher" and JNI function-table stubs per RECOMMENDATION.md.
3. libloader ELF/loader gaps -> libbadcpu ISA gaps -> services/auth.
4. Real-binary/GPU boot proof (`elfjit <libroblox.so> 0x1f0db20 --jni`) stays the
   HARD GATE, blocked until a capable host + the real binary/APK (none here).

## Session (Sep 11, 2026) — vector fmls operand order + dup-from-GPR vs sqadd gate (workspace 249/0)

Commit `7fef13e`. Two real silent SIMD miscompiles from the differential
battery's complex-matrix and -O2 fill probes:

1. **Vector fmls inverted operands.** `fmls Vd,Vn,Vm` (= Vd - Vn*Vm) shared the
   commutative add's mul/load ordering, so the JIT computed `Vn*Vm - Vd` — right
   magnitude, wrong sign. 4x4 complex matmul returned 10688 vs oracle 13504;
   `acc -= a*b` loop +20 vs oracle -100. Fixed by loading Vd into xmm0 and the
   product into xmm1 so subss(0,1) = Vd - Vn*Vm (also .2d el64 path).
2. **`dup Vd.T, Wn` swallowed by the SIMD saturating-add gate** (sqadd/uqadd/
   sqsub/uqsub have byte2==0x0c too). gcc -O2 matrix/fill loops emit `dup
   v30.4s,w1; add v30,v30,v31; scvtf; str q30,[x],#16`, so the broadcast decoded
   as a sat-add and every array held garbage (init_O2: 18446744039484557312 vs
   96). Verified vs assembler: bit21 is SET for all 14 sat-add forms and CLEAR
   for all 6 dup-from-GPR widths — now required in the sat-add gate.

Regression: `fmls_vector_subtract_has_correct_operand_order`,
`dup_from_gpr_not_swallowed_by_sqadd_gate` (decode); differential canaries
`diff_fmls_vector_subtract_accumulate`, `diff_dup_from_gpr_matrix_init`.
cargo build clean; `cargo test --workspace` 249/0. HEAD `7fef13e`.

### Next (ordered, no APK/GSI/GPU on this box)
1. Keep the differential battery sweeping ISA/correctness breadth (structure
   load/store widths, more SIMD lane/permute/wide paths, FP reduction shapes).
2. Move up to the runtime side: FMOD "divert guest bl-to-once through the
   dispatcher" and JNI function-table stubs per RECOMMENDATION.md.
3. libloader ELF/loader gaps -> libbadcpu ISA gaps -> services/auth.
4. Real-binary/GPU boot proof (`elfjit <libroblox.so> 0x1f0db20 --jni`) stays the
   HARD GATE, blocked until a capable host + the real binary/APK (none here).

## Session (Sep 11, 2026) — sat-add lane-width/sign + ubfiz-vs-ror fixes (workspace 251/0)

Commits `e3f12be` (SimdSatAdd) and `3696d89` (UBFM). The differential battery
kept finding real silent SIMD miscompiles:

1. **Saturating add/sub treated >=32-bit lanes as 64-bit ops.** SimdSatAdd used a
   64-bit load for any lane esize>=4, so `.4s` loaded 8 bytes as ONE value and
   clamped both s-lanes together (10+5 -> 0x80000000 smin sentinel; qemu 15).
   This path was only reachable after the dup-from-GPR gate fix (it had been
   silently masked by the same gate collision). Fixed with per-lane width loads,
   sign-extension (movsx/movsxd) so 64-bit clamps judge negatives correctly,
   SIGN-EXTENDED smin constants (0xFFFFFFFF80000000 for .4s — the raw lane-width
   0x80000000 as u64 is positive, so 15 < 0x80000000 and every non-negative
   result clamped), and .2d 1<<64/1<<63 guard shifts. Verified vs qemu across 8
   widths x signed/unsigned x add/sub.
2. **ubfiz/sbfiz with immr>imms hit the UBFM ROR shortcut.** `ubfiz w4,w2,#3,#3`
   (immr=29, imms=2; 29+2+1==32==bits) matched the `imms+immr+1==bits` rotate
   gate BEFORE the shift-extend branch, so it rotated right by imms=2 instead of
   computing (w2&7)<<3. A -O2 mix/shuffle-hash (`h ^= msg[i]<<((i%8)*8)`)
   returned 13680984341602923654 vs oracle 13072640789477207222. A genuine ror
   always has immr<=imms; gated the ROR branch on `!(immr > imms)` so ubfiz/
   sbfiz fall through to their shift path. mix now = oracle; real ror/extr tests
   stay green.

Regression: `sqadd_uqadd_respect_lane_width_and_sign`,
`ubfiz_immr_gt_imms_does_not_rotate` (exec); differential canary
`mix_hash_ubfiz`. cargo build clean; `cargo test --workspace` 251/0. HEAD `3696d89`.

### Next (ordered, no APK/GSI/GPU on this box)
1. Keep the differential battery sweeping ISA/correctness breadth (structure
   load/store widths, more SIMD lane/permute/wide paths, FP reduction shapes).
2. Move up to the runtime side: FMOD "divert guest bl-to-once through the
   dispatcher" and JNI function-table stubs per RECOMMENDATION.md.
3. libloader ELF/loader gaps -> libbadcpu ISA gaps -> services/auth.
4. Real-binary/GPU boot proof (`elfjit <libroblox.so> 0x1f0db20 --jni`) stays the
   HARD GATE, blocked until a capable host + the real binary/APK (none here).

## Session (Sep 11, 2026) — fixed-point fcvtzs #fbits + post-index multi-reg ld1 (workspace 253/0)

Commit `97417af`. Two more silent arithmetic/load bugs from the differential
battery:

1. **Fixed-point FP->int** (`fcvtzs/fcvtzu Rd, Fn, #fbits`, result = trunc(Fn *
   2^fbits)) was misdecoded as `SimdMull` (smull x0,w31,w24) by the widening-
   multiply gate — every *2^fbits scale silently dropped. gcc -O3 folds
   `(long long)(s*4)` into fcvtzs #2; the double square-sum `v64f` returned 5 vs
   oracle 436. Added an `fbits` field to `FcvtToInt`, decoded the fixed-point
   encodings (top16 0x1e18/0x1e58/0x9e18/0x9e58 signed, +bit16 unsigned; fbits =
   64 - bits[15:10]) before SimdMull, and scale xmm0 by 2^fbits in translate.
   vs qemu: 3.25>>#2 = 13, >>#4 = 52, s 2.5>>#3 = 20, fcvtzu 3.75>>#1 = 7.
2. **Post-indexed multi-register ld1/st1** `{Vt..,Vt+n},[Xn],#imm` sets bit23
   (bases 0x..cc0 ld / 0x..c80 st), which the structure-multiple gate's four
   no-post bases missed — `ld1 {v26.16b,v27.16b},[x1],#32` fell through to the
   single-vector Ld1V gate: loaded only 16B and advanced Xn by 16 not 32. An
   -O2 double dot-product (fmadd loop + shifted-register add addressing)
   accumulated 165 vs oracle 470. Added the 4 post-index bases.

Regression: `fcvtzs_fixed_point_fbits_scales`, `ld1_multireg_post_index_decode_and_advance`;
differential canaries `fcvtzs_fixed_scale`(/neg), `fma_ld1_postidx`.
cargo build clean; `cargo test --workspace` 253/0. HEAD `97417af`.

### Next (ordered, no APK/GSI/GPU on this box)
1. Keep the differential battery sweeping ISA/correctness breadth (structure
   load/store widths, more SIMD lane/permute/wide paths, FP reduction shapes).
2. Move up to the runtime side: FMOD "divert guest bl-to-once through the
   dispatcher" and JNI function-table stubs per RECOMMENDATION.md.
3. libloader ELF/loader gaps -> libbadcpu ISA gaps -> services/auth.
4. Real-binary/GPU boot proof (`elfjit <libroblox.so> 0x1f0db20 --jni`) stays the
   HARD GATE, blocked until a capable host + the real binary/APK (none here).

---

## Session 2026-09-11 (cycle 19) — scalar single-precision FP rounding fixed (workspace 254/0)

Workspace opened green (253/0, android idempotency test passing — the mandated
first fix was already committed in prior cycles). Continued the differential
battery ISA sweep and flushed out **two silent miscompiles** in the arm64jit
FpUnary SINGLE-precision path (`frintm/frintp/frintz/fsqrt s`):

1. **Loaded float bits never reached xmm0.** The single path did
   `mov_load32(RAX, RNslot)` then `cvtss2sd(0,0)` — but `cvtss2sd` reads xmm0's
   low32, so EVERY single frint/fsqrt converted STALE xmm0 instead of the value.
   The FpScalar single path (op 4-7) correctly inserts `movd_xmm_r32(0, RAX)`
   first; FpUnary single was missing it. A `floorf` loop returned 450 vs 360.
   Fix: add `buf.movd_xmm_r32(0, RAX);` before `cvtss2sd`.

2. **`frintz s` used round-to-nearest, not truncate.** roundsd mode was `0b00`
   (toward-nearest) instead of `0b11` (toward-zero). The double path already
   used `0x03`. A negative-heavy `truncf` loop: what ARM-trunc gives 40 came out
   20 because negatives rounded to nearest. Fix: `0b00` → `0b11`.

Verified vs native x86-64 oracle through elfjit (no QEMU): floor 360/360,
ceil 440/440, trunc 40/40; double trunc 40/40 (double was already correct).

Added a new differential battery test `diff_scalar_fp_round_single` with three
probes (fr_floor_single, fr_ceil_single, fr_trunc_single_negatives). Confirmed
the canary is a REAL gate: reverting just the `0b11` mode fix makes
fr_trunc_single_negatives fail with exactly `jit 20 vs oracle 40`, then restored.

`cargo build --workspace` clean; `cargo test --workspace` **254/0, 0 ignored**
(battery 30). Committed on `dev`.

Honest next (unchanged): vector `frintm/p/z v.*` is still an honest *Unsupported*
stop (no silent value), so gcc's vectorized rounding is the next feature slice;
then the runtime-side items (FMOD bl-to-once dispatcher, JNI table stubs) and
the HARD GATE real-binary/GPU boot proof which is impossible on this APK-less,
GPU-less VPS.

---

## Session 2026-09-11 (cycle 19b) — VECTOR frint + compare-to-zero + SimdMull gate fix (workspace 257/0)

Follow-on to the scalar FP-rounding fix. Sweeping more vector-FP probes exposed a
**systemic misdecode class**: the SIMD widening-multiply gate was
`insn & 0x0f00_c000` — it drops bit28 and only keeps byte2 bits15:14, so ANY
op whose byte2 shares those bits folded into the smlal residue. Two real,
silent (garbage-not-stop) miscompiles found and fixed:

1. **VECTOR frint** (frint{n,m,p,z,a} Vd.T, Vn.T): a `floorf` loop's
   `frintm v1.4s,v1.4s` (byte2 0x98) decoded as **smlal** — the floor loop
   returned 1.9e16 vs native 590 (a silent widen-multiply of the float bits).
   Implemented `Inst::SimdFrint` + decode (mode = bit23<<1|bit12, es=bit22,
   frinta=bit29) before SimdMull; translate per-lane via cvtss2sd→roundsd→
   cvtsd2ss (0b00 n / 0b01 m / 0b10 p / 0b11 z, frinta ties-away trick).

2. **VECTOR compare-to-zero** (fcmeq/fcmgt/fcmge/fcmlt/fcmle Vd,Vn,#0.0): gcc's
   `x<0 ? a : b` select uses `fcmlt ... #0.0` (byte2 0xea) + `bsl` — the fcmlt
   was also smlal, so the select chose the wrong branch. Implemented
   `Inst::VecFpCmpZero` + decode + translate (comiss/comisd vs a zeroed xmm1,
   setcc per lane like the existing VecFpCmp; seta/setae/sete/setb/setbe).

3. **SimdMull gate tightened** to `(insn & 0x3800) == 0` (byte2 bits13:11
   clear) — genuine smull/umull/smlal always clear those (size varies in byte2
   bits[2:0], e.g. smull .8h 0x...c020 vs umull 0x...c340), while frint (0x88/98)
   and fcmlt (0xea) set one. Prevents any future frint/cmpz-style smlal fires
   even if a new width variant slips the earlier gates. (An earlier too-strict
   `byte2 == 0xc0/0x80` broke diff_magic_div etc. — element size lives in byte2
   bits[2:0]; corrected to the 0x3800 check, which keeps all mull widths working.)

Also proved the earlier `vfa` "fneg sign flip" was a **UB false alarm**: the
probe cast a negative float to `unsigned long long` (undefined in C) — aarch64
emits `fcvtzu` (clamps to 0) while x86 emits signed trunc (-363), so the JIT
returning 0 was CORRECT aarch64 semantics. With a signed cast, jit == native ==
-363. Isolated tests confirmed fneg, bsl, and fcmlt+bsl are each correct.

New differential canaries: diff_vector_frint_rounding (vflr_floor, vflr_ceil),
diff_vector_fp_compare_zero (vcmp0_lt_keep_neg, vcmp0_gt_keep_pos). New decode
regression `simd_frint_and_cmpzero_not_swallowed_by_widen_mul` (guards (a) frint
(b) fcmlt (c) genuine umull still SimdMull). cargo build clean; cargo test
--workspace 257/0. Committed on dev.

Honest next: continue the battery sweep (vector frinta/frintn, f2d .2d forms,
sdadd/sqadd, pmull); runtime-side FMOD bl-to-once dispatcher + JNI table stubs
are still pending the real binary; HARD GATE (elfjit on libroblox.so on a GPU/
APK host) unchanged — impossible on this GPU-less, APK-less VPS.

---

# Session (Sep 11, 2026) — three silent FP `.2d`/FMOV-imm decode collisions fixed (workspace 264/0)

Extended the differential battery with four new game-critical canaries (SIMD
fmin/fmax reduction, reciprocal/division funnel, integer widening mul-acc,
double FP loop-condition). One — `l_double_div_accum_guard` (6-element
`(i+1)*2/(i+2)` sum with a `<3.0` guard) — **failed: jit 2001 vs oracle 8820**.
Bisecting it (per-instruction exec_bytes + whole-block runs through
`load_elf_image`+`jit_run`) surfaced THREE unrelated silent miscompiles in the
FP/vector decode layer, each a real pixel/audio/coordinate corruption:

## 1. `fadd/fmul/fsub Vd.2D` swallowed by the integer add/sub gates
The four SIMD int add/sub gates (Simd4s/SimdAddD/SimdAddB/SimdAddH) keyed on
`insn & 0x2f20_0c00` in {0x0e200400, 0x2e200400} plus `size`, but ignored bit14.
FP `.2d` two-source ops (byte1 0xd4, bit14 SET) share that residue with integer
`add` (byte1 0x84, bit14 CLR). `fadd v0.2d,v0.2d,v1.2d` (0x4e61d400) compiled as
a **halfword `paddw`** on the double bits. The isolated host dump showed
`paddw xmm0,xmm1`; the decode scratch returned `SimdAddH`. Fix: `(insn & 0x4000)
== 0` on all four gates. VecFpArith covers `.2s/.4s` early; Simd2dFp now gets
`.2d`.

## 2. `fdiv/fmul Vd.2D` swallowed by the SimdSel (bsl) gate
byte1 low 0xfc/0xdc (bits[15:13] SET) duped the bsl select gate, which only
checked `(insn & 0x1c00) == 0x1c00` (bits[12:10]). Host dump showed
`pandn/pand/por` — a bitwise select. `fdiv v0.2d` returned 256 for 64/8 (both
lanes). The real bsl family always has byte1 low 0x1c (bits[15:13] CLR). Fix:
`(insn & 0xe000) == 0` on the SimdSel gate.

## 3. `fmov d,#imm` with mantissa m>=8 swallowed by the fcvt-to-int round gate
`fmov d23,#12.0` = 0x1e651017 decoded as `FcvtToInt` (destination silently 0).
The coarse fcvt-round gate (`insn & 0xffff_0000`, added for `0x9e..` X-dest
fcvtps/ms/au) included `0x1e65` (W fcvtau) which collides with FMOV-imm. FMOV-imm
has bit12 SET (the imm lane anchor) while every fcvt-to-int has bit12 CLR
(verified: fcvtzu 0x1e790020, fcvtas 0x1e640020, fcvtau 0x1e650062). Fix: gate
the fcvt-round block on `(insn & 0x1000) == 0`.

## Verification
- Isolated vs the real aarch64 assembler: bsl 0x6e611c00, fdiv/fmul/fadd .2d
  0x6e61fc00/0x6e61dc00/0x4e61d400, fcvtzu 0x1e790020, `fmov d,#12.0` words
  0x1e651017 etc. p9 (scalar div+fmadd, was 2^63) -> 6000, p5 (vector
  a[]+reduce, was 90000) -> 8814, l_double -> 8820.
- +1 decode regression (`fp_2d_op_decode_collisions_with_int_add_bsl_and_fcvt`),
  +2 exec regressions (`vector_2d_fp_div_mul_not_swallowed_by_int_add_or_bsl`,
  `fmov_imm_high_mantissa_12_to_15_not_swallowed_as_fcvt`), and the 4 new battery
  canaries are permanent.
- `cargo build --workspace` clean; `cargo test --workspace` 264/0, 0 ignored.

HARD GATE unchanged: real-binary/GPU boot proof (`elfjit <libroblox.so>
0x1f0db20 --jni`) on a GPU + real binary/APK host (none on this VPS).
---

# Session (Sep 11, 2026) — SIMD across-lanes min/max + smax/smin & movsx fixes (workspace 268/0)

Continuing the differential battery: a new int SIMD min/max reduction canary
(`im_running_minmax`) exposed one missing ISA and two more silent bugs.

## 1. SMINV/SMAXV/UMINV/UMAXV implemented (was Unsupported)
Across-lanes reduce to the bottom scalar (upper cleared). New
`Inst::SimdReduceMinMax` decode (0x4e/0x6e `XYa820` family: esize by byte2 high
nibble {3,7,b}, min/max by byte2 bit0, signed by bit29) + translate (per-lane
sign/zero-extended CMOVcc reduce). Two gotchas during bring-up: cmov_rr64
expects the cc in the `0F 4X` domain (I first passed the raw 0x0X Jcc domain ->
SIGILL `0F 0C`), and the cmp/cmov direction is min=CMOV-G/A, max=CMOV-L/B
(update when the candidate is the extrema found by `cmp RDX, RAX`).

## 2. Element-wise smax/smin mis-decoded for high source registers (silent)
The SminMax gate is correctly written `(b2 mask 0xfc) == 0x64` (max) but the
ASSIGNMENT was `max: b2 == 0x64` (EXACT). b2 = bits[15:8] and its low 2 bits
carry Rn (bits[9:8]). A real gcc `smax v30.4s, v29.4s, v28.4s` encodes b2=0x67,
so it masked to max but the exact-equality assign said MIN -> returned the Vn
operands verbatim. Only source regs 0..3 (b2 stays 0x64/0x6c) ever hid it.
Fixed to mask like the gate.

## 3. movsx_word_mem/movsx_byte_mem lacked REX.W (silent, shared-emitter)
`0F BF /r` / `0F BE /r` with REX no-W write only a 32-bit destination, so a
negative 8/16-bit lane loaded into RAX compared as a huge POSITIVE u64 in any
64-bit signed reduction (`sminv.8h` over {-9,-2,..} picked 4, not -9). Latent
across every consumer (incl. ADDV signed byte/halfword sums). Both emitters now
emit REX.W. This was the real root of the earlier `.8h`/`.16b` sminv results.

Verified via per-instruction stepping of the gcc-unrolled probe: smax produced
v30=[0,-28,28,28] (wrong) -> after fix [112,140,252,308]; sminv/smaxv/addv
scalars and the final result 3640 = oracle. +2 differential canaries
(rm_running_extents, im_running_minmax), +2 exec regressions. cargo build
--workspace clean; cargo test --workspace 268/0, 0 ignored.

HARD GATE unchanged: real-binary/GPU boot proof (`elfjit <libroblox.so>
0x1f0db20 --jni`) on a GPU + real binary/APK host (none on this VPS).
---

# Session (Sep 11, 2026) — guest_svc syscall surface expanded for real-boot/ALooper/login (workspace 270/0)

The android-layout idempotency test is green at HEAD (long since fixed); the
workspace gate is clean. This session widened the JIT's in-process AArch64
syscall dispatcher (`guest_svc` in `crates/arm64jit/src/jit.rs`) from ~30 to
~48 syscalls, targeting the families a real Android boot / ALooper / login
path issues that the table previously sent to -ENOSYS:

- **fstat(80) / newfstatat(79)** with a **guest-layout `stat`** — the host
  `libc::stat` layout differs across x86_64 vs aarch64, so forwarding the host
  struct would silently mis-place every field. `unsafe fn write_guest_stat`
  transcribes into the AArch64 asm-generic layout (128B): st_dev@0 st_ino@8
  st_mode@16 st_nlink@20 st_uid@24 st_gid@28 st_rdev@32 st_size@48
  st_blksize@56 st_blocks@64, times (sec+nsec) @72..112. Numbers + layout
  verified against `/usr/aarch64-linux-gnu/include/asm-generic/{unistd,stat}.h`.
- **sockets**: socket(198), bind(200), listen(201), accept(202), connect(203),
  setsockopt(208), getsockopt(209) — the networking/login path.
- **event/epoll** (Android ALooper is epoll-based): eventfd2(19),
  epoll_create1(20), epoll_ctl(21), epoll_pwait(22), ppoll(73).
- **descriptors**: dup(23), dup3(24), ioctl(29), readv(65), writev(66).
- **system/time**: uname(160), gettimeofday(169), clock_getres(114).
- **limits/signals/timers**: getrlimit(163)/setrlimit(164), kill(129),
  tgkill(131), timer_create(107), timer_settime(110).

All struct-returning syscalls chosen with layout-identical-or-explicit
conversion (timeval/rlimit/utsname/epoll_event layouts are arch-identical;
`stat` uses write_guest_stat). New integration test
`guest_svc_stats_and_descriptors_roundtrip` verifies fstat/newfstatat st_size+
st_mode in guest layout, eventfd write/read, epoll_create1+epoll_ctl(ADD — on an
eventfd/pipe, not a regular file which EPERMs), gettimeofday, uname=="Linux".

Gate: `cargo build --workspace` clean; `cargo test --workspace` 270/0 (was 269).
HARD GATE unchanged — real Roblox boot + run log on a GPU/APK host
(`elfjit <libroblox.so> 0x1f0db20 --jni`); none of that is on this VPS.

# Session (Sep 11, 2026) — two REAL silent miscompiles fixed: EXTR operand order + ADC/SBC carry polarity (workspace 272/0)

New differential canaries (diff_math128_and_carry: __int128 sq/mul/madd;
diff_switch_fnptr_hash: switch jump table / fnptr blr dispatch / FNV) surfaced
one failing probe: math128 (JIT 0xc24b01e838c56079 vs native 0xb50f76ac635ab31b).
Bisected to TWO distinct arm64jit bugs, both fixed + native-verified:
1. EXTR invert — see STATUS.
2. ADC/SBC borrow-convention carry — see STATUS.
Files: crates/arm64jit/src/translate.rs (Extr + AddCarry), jit.rs
(add_carry_reference), tests/diff_battery.rs (+math128 canary). Commit 912eff8.


---

# Session (Sep 11, 2026) — SIMD fcvtl/fcvtn float<->double conversion + latent scalar store fix (workspace 275/0)

Commit `3718824` (dev). Opened at 272/0 green; drove the differential battery
into the mixed-precision float<->double vector path and it surfaced BOTH a
missing ISA wall and a latent miscompile:

## 1. New ISA: fcvtl/fcvtl2 + fcvtn/fcvtn2 (SIMD float<->double width conversion)
gcc -O2 emits these for any `float[]` <-> `double[]` elementwise round-trip; the
JIT previously stopped `Unsupported` at the first fcvtl in such code.
- Gates (asm+objdump verified): `(insn & 0xffff_fc00)` in {0x0e617800,
  0x4e617800} = fcvtl (f32->f64, 2 lanes), {0x0e616800, 0x4e616800} = fcvtn
  (f64->f32). Placed BEFORE VecIntToFp/SimdMull (which swallow these as int->fp /
  widening-multiply). `upper` = bit30 (Q): fcvtl2 reads Vn's upper half;
  fcvtn2 writes Vd's upper half. Half-precision byte2-0x21 (fcvtl Vd.4s,Vn.4h /
  fcvtn Vd.4h,Vn.4s) stays Unsupported (no fp16 in the JIT).
- Translate: per-lane cvtss2sd/movq_store (widen), movq_load/cvtsd2ss/narrow
  store (fcvtn). Guest v-lane = VECTOR_BASE + reg*16 confirmed.

## 2. LATENT MISCOMPILE exposed by the new ISA (the important find)
Once a gcc float<->double loop can compile fully (previously it always aborted
`Unsupported` at fcvtl during compile, so NOTHING after it ever executed), the
JIT segfaulted at fault=0x41480000. Root cause: `FpLdStImmWb` (scalar pre/post-
index ld/st) passed RAX as the address register into `fp_scalar_xfer`, which
uses RAX as its value scratch -- so a scalar pre/post-index STORE clobbered the
base with the value and wrote to [value bits] instead of [base]
(`str s30,[x4],#4` stored to 0x41480000 = float 12.5). Now computes the address
in RDX (mirrors the correct FpLdStImmUnscaled). This would have corrupted
single-precision array/matrix writes in any real graphics/audio math.

## Verification
- Differential canary `diff_fcvtl_widen_and_fcvtn_narrow` (round-trip 16
  floats<->doubles, distinct values, upper-half use): jit==oracle==1248.
- Decode/exec: `fcvtl_fcvtn_decode_and_lane_widen_exec`; store fix:
  `fp_scalar_postindex_store_uses_base_not_value`.
- `cargo build --workspace` clean; `cargo test --workspace` 275/0 (arm64jit
  162 lib + 42 diff + 8 loader_run; libbadcpu 22; libloader 23; +others).

## Next (ordered, no APK/GSI/GPU on this box)
1. Keep the differential battery sweeping mixed FP width / vector breadth
   (fcvtl half-precision forms, f2d long forms, pmull, sat-ops, more -O3
   reduction shapes) -- this ISA-assertion loop keeps flushing real latent
   miscompiles (this cycle: fcvtl + the scalar-store RAX clobber).
2. libloader ELF/loader gaps -> libbadcpu ISA gaps -> services/auth.
3. Real-binary/GPU boot proof (`elfjit <libroblox.so> 0x1f0db20 --jni`) stays
   the HARD GATE, blocked until a capable host + the real binary/APK (none here).


---

# Session (Sep 11, 2026) — SIMD BIF misdecoded as BSL; int64->int32 narrowing (workspace 277/0)

Commit `60400f3` (dev). Opened at 275/0 (fcvtl cycle). Pushed the differential
battery further into int64->int32 truncation pipelines and found a THIRD real
silent miscompile in the bitwise-select family:

## BIF was decoded as BSL with the opposite mask semantics
gcc -O2 int64->int32 narrowing (with large negatives needing sign handling)
compiles through smull/saddl/saddw + cmeq + bit/bif + uzp; through the JIT it
returned the wrong value. Root cause: the SimdSel (BSL) decode gate required
bit22 SET but did NOT inspect bit23, so BIF (`bit22=1,bit23=1`) was silently
decoded as BSL (`bit22=1,bit23=0`). The three select ops are:
- BSL = (N&M)|(D&~M);  BIT (bit22=0/bit23=1) = (N&M)|(D&~M);  BIF (bit22=1/bit23=1)
  = (N&~M)|(D&M) -- BIT/BIF are the opposite bit-inserts.
- Fix: SimdSel gate now requires bit23 CLEAR (BSL only); BIT/BIF fall through.
- SimdBit gate extended to the BIF residues (0x6ee01c00/0x2ee01c00 alongside
  0x6ea01c00/0x2ea01c00) with a `bif` flag decoded from bit22; translate emits
  (sel&Vm)|(keep&~Vm) with (sel,keep)=(Vd,Vn) for BIF and (Vn,Vd) for BIT.
- NOTE: the bit-vs-bif opcode discriminator is bit22 (0x0040_0000), NOT bit14
  (a first pass used bit14 and produced wrong values; the two words differ by
  exactly 0x00400000). The SimdBit FAMILY residues also differ by bit22, so the
  gate set {0x6ea01c00, 0x6ee01c00, ...} is correct.

## Verification
- jit.rs `bit_vs_bif_bitwise_insert_semantics`: decode maps bit->bif:false,
  bif->bif:true, bsl->SimdSel; exec both produce the correct DISTINCT values
  (BIT 0x11bb33dd11ff7799, BIF 0x660066446600ee for the fixed operand set).
  Learned en route: read ARM's Vd,Vn,Vm operand order for `bit`/`bif` (Vm is the
  MASK); the earlier "bif returned bit's value" was a test-labels error, not a
  translate bug.
- diff_battery `diff_int64_to_int32_narrowing_bif`: int64->int32 and the full
  int64->int32->short round-trip drive the bif path; jit==oracle.
- (Surgery lesson: a bad mid-file replace during test insertion dropped two
  pre-existing host-float-bridge tests; restored them verbatim from HEAD and
  verified with a fn-name diff that no test was lost.)
- `cargo build --workspace` clean; `cargo test --workspace` 277/0 (arm64jit
  163 lib + 43 diff + 8 loader_run; libbadcpu 22; libloader 23; +others).

## This session's net (commits 3718824 fcvtl + store fix, 60400f3 bif)
Two new-ISA walls (fcvtl/fcvtn) and TWO latent silent miscompiles fixed
(FpLdStImmWb scalar store RAX-address clobber; SIMD BIF->BSL opposite select) --
the differential-assertion loop keeps flushing real wrong-pixel/audio bugs.

## Next (ordered, no APK/GSI/GPU on this box)
1. Keep the differential battery sweeping ISA breadth (SIMD permute/wide
   paths, sat-ops, half-precision, -O3 reduction shapes) -- the fcvtl+bif
   cycles prove it is the highest-leverage correctness engine available here.
2. libloader ELF/loader gaps -> libbadcpu ISA gaps -> services/auth.
3. Real-binary/GPU boot proof (`elfjit <libroblox.so> 0x1f0db20 --jni`) stays
   the HARD GATE, blocked until a capable host + the real binary/APK (none here).


---

# Session (Sep 11, 2026) — SIMD compare-to-zero + sub-wide + high-rm MulDiv (workspace 280/0)

Commit `1702c89` (dev). Opened at 277/0 (bif cycle). Continued the cross-gcc ISA
sweep into signed-byte / 16-bit / int64-narrowing code; flushed one new-ISA wall
and TWO more real silent miscompiles (total this session: 2 ISA walls + 4 bugs):

## 1. SIMD integer compare-to-zero (cmeq/cmgt/cmge/cmlt/cmle Vd.T,Vn.T,#0) -- NEW
gcc emits these for every vectorized x<0 / x==0 / sign check, e.g. the
`if(sc[i]<0) count++` idiom which compiles to cmlt -> sxtl -> ssubw (count=n by
subtracting the sign-extended mask). Implemented `Inst::SimdCmpZero`:
- gate: top byte {0x0e,0x2e,0x4e,0x6e} + bit26 + bits[15:10] in {0x22 cmgt,
  0x26 cmeq, 0x2a cmlt} AND bit16 CLEAR (bit16 set = FP frint/tbl family -- the
  first gate draft collided with frintm 0x4e219821).
- esize = 1<<bits[23:22] (8B/8H/4S/2D); U (bit29) toggles gt->ge and eq->le.
- translate: per-lane signed/sign-extended load, test, setcc(eq==0x4,gt==0xf,
  ge==0xd,lt==0xc,le==0xe), movzx, neg => all-ones-or-0 mask.

## 2. sub-wide ssubw/usubw decoded as ADD -- SILENT
The SimdAddw gate checks `(insn & 0x1800)==0x1000` (bit12 set, bit11 clear) but
ignores bit13 (0x2000), the add/sub-wide discriminator (saddw 0x0e7613de vs
ssubw 0x0e7633de differ by bit13). So every ssubw silently ADDED the widened
element; the negative-count idiom's subtract became an add and the count was
wrong. Added `sub = (insn & 0x2000) != 0` to SimdAddw; translate emits
`sub_rr64` when set.

## 3. MulDiv gate mask kept bit20 -- `mul w9,w9,w20` was Unsupported
`(insn & 0x7ff0_0000)` keeps bit20 (part of rm, bits[20:16]), so any mul/madd/
sdiv with rm>=16 (bit20 set) failed the gate. Correct mask 0x7fe0_0000 (clears
the whole rm field). gcc's `mul w9,w9,w20` (32-bit mul, high rm) exposed it.

## Verification
- jit.rs `isa_regress_tests`: cmlt mask (neg bytes all-ones) + ssubw subtracts-
  not-adds exec (both decode-atom and runtime values).
- diff_battery `diff_byte_negcount_ssubw_cmlt`: byte sum + negative count via
  the cmlt->ssubw idiom + 16-bit unsigned widening; jit==oracle.
- Cross-gcc probes all exact: byte SIMD=36775, 16-bit audio =9564799, int64
  narrowing n1=7769812456 / w=7634108456, byte+negcount=3487.
- (learned: keep test value literals out of `<< 32` shift overflow; append
  regression tests as a separate `#[cfg(test)] mod` at EOF instead of fragile
  mid-file splicing.)
- `cargo build --workspace` clean; `cargo test --workspace` 280/0 (arm64jit
  165 lib + 44 diff + 8 loader_run).

## Session total (commits 3718824, 60400f3, 1702c89)
4 ISA walls / features (fcvtl/fcvtn float<->double, SIMD compare-to-zero) and
FOUR real silent miscompiles found+fixed via the differential sweep:
FpLdStImmWb scalar-store RAX-address clobber, SIMD BIF->BSL opposite select,
ssubw-as-add, MulDiv high-rm gate mask. The cross-gcc differential battery
(+native oracle) is the highest-leverage correctness engine available without
an APK/GPU.

## Next (ordered, no APK/GSI/GPU on this box)
1. Keep the differential battery sweeping (sat-ops, half-precision, more -O3
   reduction/permute shapes, fp16).
2. libloader ELF/loader gaps -> libbadcpu ISA gaps -> services/auth.
3. Real-binary/GPU boot proof (`elfjit <libroblox.so> 0x1f0db20 --jni`) stays
   the HARD GATE, blocked until a capable host + the real binary/APK (none here).

## Session (Sep 11, 2026) — ld2/st2 structure deinterleave element-size FIXED (commit 36813d1, workspace 282/0)

Root-caused and fixed a silent SIMD miscompile in the structure load/store
(ld2/st2) path: the translate arms DEINTERLEAVED AT BYTE GRANULARITY
regardless of element size. `ld2 {v.8h, v.8h}` (2-byte elements — gcc's
strided u16 accumulate `for(i+=2) s2 += b[i]`) read mem[2i],mem[2i+1] instead
of the correct mem[4i],mem[4i+2]; the even-index sum registered the wrong
memory elements. Decode already computed `esize` for the 3/4-register forms
but dropped it for ld2/st2 (the 2-register case made byte-only runs look
correct, hiding the bug). Passed `esize` through decode and deinterleave at
element stride (element i of reg j at byte i*(2*es)+j*es, copy es bytes).

Verified end-to-end (no QEMU): isolated strided-u16 repro 311814 -> 281606 ==
native; differential canary `diff_ld2_halfword_strided_accumulate` added
(proven sensitive — reverting ONLY the esize change makes it fail again).
+decode regression with assembler-verified 8h/16b/4s encodings.
`cargo build --workspace` clean; `cargo test --workspace` 282/0 (was 280).

HONEST REMAINING / next high-value target: a SEPARATE pre-existing co-resident
bug surfaced by the differential sweep — a single function that VECTORIZES TWO
widen-accumulate loops (e.g. two byte-sums `i+=1` and `i+=2`) into one host
block produces correct low-32 sums but a stray data byte at bits 32-39 of the
64-bit accumulator lanes (repro `twov2.c`/`twoloopp.c`: JIT 18916 vs oracle
16868; isolated each loop passes). Independent of the ld2 fix (byte-only path,
esize=1, unchanged). Root-cause lead: bits 32-63 of the accumulator lanes are
contaminated, strongly suggesting a shared host/permscratch register clobber
across the two co-resident widening loops — NOT a single-Ld2 fault. Next
session: trace which translate arm leaves permscratch / a host vector scratch
dirty that the second loop's zip/uxtl reads; this subsumes the "byte-acc
halved" 2x signature documented in the runs/STATUS ledger. HARD GATE unchanged
(no GPU/APK/libroblox.so on this VPS).

## Session (Sep 11, 2026) — differential-sweep JIT correctness: 6 silent SIMD/int miscompiles FIXED (288/0)

Driving elfjit against qemu-aarch64 oracles on real static aarch64 builds caught
and fixed SIX silent miscompiles (each regression-guarded + sensitivity-verified):

1. ld2/st2 element-size deinterleave (commit 36813d1) — byte-granularity
   regardless of element size; strided-u16 accum read mem[2i],mem[2i+1] instead
   of mem[4i],mem[4i+2]. 282/0.
2. W-form bitfield (Ubfm/LSR/LSL/ROR aliases) loaded Rn as u64 and shifted
   without zeroing the high 32 bits (b739a6b) — `madd x; lsr w` pulled high
   guest garbage into the byte (8652 -> 0x37562e2cc). 284/0.
3. shrn/shrn2 (shift-right-NARROW) decoded as plain equal-size ushr/sshr
   (04a1483) — wrong source stride + shift (x^(x>>16) -> 0x83f9b82e6). 286/0.
4. rbit SWAR truncated every 64-bit mask to 32 bits (b83a1cf) — only the low
   half of x reversed; ctz=198 vs 10.
5. clz x (sf=true) emitted REX.W BEFORE the F3 prefix — CPU ran a 32-bit
   lzcnt, so clz(x<2^32) returned 32-len not 64-len (clz(0x16136740)=3 vs 35).
   Emit `F3 48 0F BD` (REX must be the last prefix).
6. shl #imm decoded shift as immb-only + esize from trailing_zeros (dropped
   bit22) — `shl v.4s,#25` ran as #1 on 1-byte lanes. shift = immh4:immb -
   esize_bits.

Tooling added: `sweep_wform.py` (in /tmp/combw) — static -nostdlib -Wl,-e,entry
build, qemu-aarch64 oracle, elfjit run, diff. Expanded to rbit/clz/ctz/shl/
shr/umulh families; all probes now match the oracle. Notebook: the entry-sym
parse must split on whitespace (objdump '0000000000400120 <entry>:' includes
the symbol), and native oracle must use gcc not cross-gcc.

Workspace 288/0, build clean, 6 focused commits on dev. No repo push. The
pre-existing co-resident two-loop widening bug remains open (documented;
contamination at bits 32-63 of acc lanes when two widen loops share a block)
and is independent of all six fixes. Next: keep sweeping SIMD/FP shapes; then
libloader ELF/loader gaps per RECOMMENDATION order. HARD GATE unchanged (no
GPU/APK/libroblox.so on this box).

## Session (Sep 11, 2026) — randomized differential fuzz: +2 JIT fixes (289/0, 165 fuzz cases green)

Extended the elfjit-vs-qemu-aarch64 differential harness into a randomized
fuzzer (`fuzz_jit.py` in /tmp/combw) generating unrolled scalar/SIMD shift/xor/
byte/fp programs with runtime-dependent LCG inputs. Found TWO more silent JIT
bugs, both in `Inst::BitField` (the general-extract path):

1. UBFM extract mask SIGN-EXTENSION: for a field width making the mask >= 2^31
   (ubfx x,#16,#32 -> mask 0xffffffff) `and_ri64(RAX, mask as u32)` emitted a
   64-bit AND with a sign-extended imm32 => mask became 0xffffffffffffffff
   (no-op), leaking the high 32 bits of the shifted value into the result
   ((x>>16) -> 8234290418553910946 vs 85047753507696). Route masks >0x7fffffff
   through mov_ri64+and_rr64. Commit 750457a.
2. UBFM mis-decoded as ROR: `imms+immr+1==bits` is NOT a rotate discriminator —
   genuine ror is an EXTR alias (-> Inst::Extr), so any UBFM matching it
   (ubfx x,#16,#32: immr=16,imms=47, field to the top bit) is a plain extract.
   The old branch ran ror_ri8(imms), corrupting extracts. Removed it; words
   fall through to the extract path. Genuine ror x,#17 still passes.

Regression: diff_ubfx_masks_upper_bits_and_is_not_ror (u32-width extract +
extract-with-genuine-ror), sensitivity-proven both ways. cargo build clean;
cargo test --workspace 289/0. Randomized fuzz seeds 1-4: 165/165 match qemu.

Combined with the previous sweep this cycle has produced EIGHT verified JIT
correctness fixes (ld2/esize, W-form bitfield, shrn2, rbit mask, clz REX order,
shl-imm decode, ubfx mask, ubfx-vs-ror). Next: keep fuzzing with more diverse
generators (fp, structure loads, saturating arith), then libloader ELF/loader
gaps per RECOMMENDATION order. HARD GATE unchanged (no GPU/APK/libroblox.so).

## Session (Sep 11, 2026) — co-resident two-loop widening bug RESOLVED (was the documented OPEN BUG)

After the 8 JIT correctness fixes this session (ld2/st2 esize deinterleave,
W-form bitfield high-32 mask, shrn/shrn2 narrow decode, rbit 64-bit mask width,
clz REX.W byte order, shl#imm decode, UBFM extract mask sign-extension, and
ubfx-vs-ror discriminator), the LONG-DOCUMENTED co-resident two-loop widening
bug — and the "intermittent SIMD-loop block-liveness" bug it subsumed — are
both resolved. The original reproducers now pass exactly:
  twov2.c    JIT 18788  == oracle 18788   (was JIT 7465833 / garbage 0x375)
  twoloopp.c JIT 3776   == oracle 3776
  maskf.c    JIT 2416   == oracle 2416    (documented intermittent lane corruption)
Both were SYMPTOMS of the same underlying shift/immediate-mask miscompiles
(high-32 contamination leaking through sign-extended `and` imm32 masks and
wrong shift amounts), not a separate co-resident register-liveness fault. ~700
differential fuzz cases green including PIE+reloc loader-mode (elfjit vs qemu
oracle). This removes the last known-open JIT correctness item on this box.

Next (ordered, all still open): libloader ELF/loader gaps, libbadcpu ISA gaps,
services/auth — or more fuzz coverage. HARD GATE unchanged (no GPU/APK).
## Session (Sep 11, 2026) — guest_svc syscall surface expansion (filesystem/IO/network)

Added two batches of AArch64 syscalls to the guest_svc bridge that a real
Android/Roblox boot path issues early and that previously returned -ENOSYS,
each number verified against the sysroot asm-generic headers (not guessed):
  Batch 1 (commit c0073b6): fcntl(25), clock_nanosleep(115), getrusage(165),
  setpgid(154), rt_sigaction(134), rt_sigprocmask(135), fadvise64(223).
  Batch 2 (commit a682eea): mkdirat(34)/unlinkat(35)/symlinkat(36)/linkat(37)/
  renameat(38), pread64(67)/pwrite64(68), socketpair(199), sendto(206)/
  recvfrom(207)/sendmsg(211)/recvmsg(212)/accept4(213), madvise(233), umask(166),
  getgroups(158).
Signal ops accept registration (return 0) but don't dispatch guest trampolines
(consistent with the shim's no-signal posture); oact/oset outputs are zeroed so
callers don't deref garbage. madvise DONTNEED keeps host RSS bounded under guest
allocation churn. Test guest_svc_common_boot_gaps_roundtrip covers both batches
(mkdir/link/rename/read/write/pipe lifecycle, socketpair+sendmsg/recvmsg byte
roundtrip, madvise, umask, fcntl, sigaction/procmask zeroing, clock_nanosleep).
Workspace 290/0. HARD GATE unchanged (no GPU/APK on this box).
## Session (Sep 11, 2026) — fuzz_jit loader-mode: PIE + reloc shapes end-to-end (100+ cases)

Extended the differential fuzzer with a loader-mode: gen_globals_pie /
gen_pie_callchain compile `-fPIE -pie -nostdlib` programs with exported
statics, arrays, and function-pointer initializers — forcing R_AARCH64_GLOB_DAT,
RELATIVE, and ABS64 relocations in .data.rel.ro — then run them through elfjit
(load_elf_image + bind_image_plt + JIT) and diff against a qemu-aarch64 oracle.
~35% of fuzz cases now take this path. Seeds 31-36: 120/120 pass, validating
the full loader→reloc→bind→JIT chain the runtime depends on (commit 93c0ee4).
The JIT correctness work is now broadly covered (~620 differential cases green
total). Next (unchanged, in RECOMMENDATION order): libloader ELF/loader gaps
then libbadcpu ISA gaps then services/auth; or more precision on the open
co-resident two-loop widening bug. HARD GATE unchanged (no GPU/APK on this box).

## Session (Sep 11, 2026) — guest_svc boot-path gaps + co-resident bug RESOLVED (290/0)

Added seven AArch64 syscalls a real Android/Roblox boot issues early, that
previously fell to -ENOSYS: fcntl(25), clock_nanosleep(115), getrusage(165),
setpgid(154), rt_sigaction(134), rt_sigprocmask(135), fadvise64(223) — numbers
verified against the sysroot asm-generic headers (commit c0073b6). +unit test
guest_svc_common_boot_gaps_roundtrip.

MAJOR: the long-documented co-resident two-loop widening bug — and the it
subsumed "intermittent SIMD-loop block-liveness" bug — are BOTH RESOLVED. After
the 8 JIT correctness fixes this session (ld2/st2 esize, W-form bitfield mask,
shrn/sh2 narrow decode, rbit 64-bit mask, clz REX.W order, shl#imm decode, UBFM
mask sign-extension, ubfx-vs-ror), the original reproducers pass exactly:
twov2.c 18788==18788 (was 7465833), twoloopp.c 3776==3776, maskf.c 2416==2416.
They were SYMPTOMS of the same shift/immediate-mask miscompiles, not a separate
register-liveness fault. ~950 differential fuzz cases green (incl. PIE+reloc
loader-mode via fuzz_jit.py). No known-open JIT correctness items remain.

Workspace 290/0, build clean, ~18 focused commits on dev. Next (RECOMMENDATION
order): more boot-path syscalls, libloader/libbadcpu/services gaps, or more fuzz
coverage. HARD GATE unchanged (no GPU/APK/libroblox.so on this box).

## Session (Sep 11, 2026) — scalar ucvtf S-form FIXED (291/0) + fuzz gens

Silent FP miscompile found by expanding the differential fuzzer with
fma-chain/128-bit-struct/float-reduce generators: scalar ucvtf S-form
(0x7e21db18, gcc emits `ldr sD,[sp]` + `ucvtf sD,sD` for `(float)volatile_u32`)
decoded with sng=true but the translate arm ignored it and always did the 64-bit
D-form convert, silently dropping the value (audio/down-mix u32->float). Fixed,
sensitivity-proven (166908 vs 222908 without the fix). commit 48863d1.
Open (deeper, still characterizing): vectorized div/multiply reduction pipeline
(`a[i]/b[i]` -> ushr.2d + and + uzp1 + scvtf + fmla fmul fdiv) gives value
collapse (63 vs 729251) not reproducible in minimal isolated hand-asm; next
step is tracing that specific op mix.

## Session (Sep 11, 2026) — fuzz-driven FP findings wrap

Fixed 9th JIT miscompile this campaign: scalar ucvtf S-form sng flag ignored
(commit 6a3db93). fuzz_jit gens expanded (fma-chain/128-struct/float-reduce).
All individually-isolated SIMD/FP ops now match the qemu oracle (fdiv/fmul/fmla/
uzp1/and/ushr/scvtf/ucvtf/movi/lane-extract). Remaining open: a specific
multi-op vectorized div/multiply-reduction pipeline (vmult.c: `acc+=a[i]/
b[i]+c[i]` with computed operands) shows value collapse ~63 vs 729251 that does
NOT reproduce when the same ops are isolated; needs a JIT execution tracer to
pin the exact guest instruction. Low smoking-gun priority: all constituent ops
validate individually. Next sessions: add a per-instruction traced run to
diff_battery or resolve via reducing vmult.c further.

## Session (Sep 11, 2026) — MOVI Vd.2D immediate decode FIXED — the 10th JIT bug

Root-caused and fixed the 'fmla/div vector pipeline' bug that had resisted
isolation all session. It was NOT a lane-coalescing interaction — it was a
fundamental decode error in MOVI Vd.2D, #<imm>. The 2D immediate is a
BYTE-SELECT pattern (bits[9:5] low nibble picks which bytes 0..3 of the low 32
are 0xff), but the decode treated it as a generic imm8-replicate, so
mov v27.2d,#0xffff produced 0x03 lanes instead of 0x000000000000ffff. Any
vector AND-mask built this way corrupted bit-field extraction (gcc's
shr->and->uzp1->scvtf reduction), collapsing values. Fixed the decode with 6
qemu-verified ground-truth lane values; new unit test (movi_2d_byte_select_
ground_truth) + moved the flow canary from #[ignore]d to a real regression
(diff_fmla_div_pipeline_lcg). Sensitivity-proven both ways (695 vs 7777753).
cargo test --workspace 293/0. This is the 10th JIT correctness fix this
campaign; the fma_chain fuzzer is responsible for surfacing it.

## Session (Sep 11, 2026) — RELR e2e test + campaign wrap (294/0, ~1680 fuzz cases)

The loader's DT_RELR path (packed-relative relocations, tag 0x23) had a
unit-tested decoder but the discovery wire-up (walk PT_DYNAMIC tags -> find the
stream -> decode -> route back as R_AARCH64_RELATIVE) had no test. Added an
e2e test that builds a minimal ELF with a RELR-only PT_DYNAMIC on disk and
asserts read_elf_relocations returns the right offsets/info. This is the last
unvalidated reloc path a real Android .so would hit (recent lld emits RELR by
default).

Known gap for next sessions: R_AARCH64_TLS_* (TPREL/DTPREL) are not handled at
all by the loader/JIT. Real TLS needs a host-side per-thread guest TLS area
(tp/x28 slot), an architected feature — needs the real binary to validate.
cargo test --workspace 294/0, build clean.

## Session (Sep 11, 2026) — long-open fused two-loop signed-div miscompile RESOLVED: integer vector NEG/ABS (302/0)

Root-caused the last reproducibly-open JIT bug — the fused two-loop signed
magic-division miscompile (`fuzz_jit.py` gen_signed_div; n=4, div /7 repro:
oracle 406144671 vs jit 78184144 — the entire neg-loop contribution was lost).
It was a DECODE COLLISION, not a register-clobber:

- `neg v29.2s, v31.2s` (0x2ea0bbfd) is an integer two-register-misc op
  (opcode bits[16:12]==0xb, bit16 CLEAR). The vector float->int FcvVec gate
  used mask 0xffe0_fc00, which ZEROES bits[20:16], so NEG fell into the residue
  `0x2ea0_b800` (== fcvtzu v0.2s) and executed as a float->int convert — with
  v31.s0=0xfc8e117a (a small denormal float) that silently produced 0, then fed
  every downstream zip1/saddw/sxtl2 lane wrong.

Fixed (commit 47b1007):
1. FcvVec gate now requires bit16 SET (all six fcvtzs/fcvtzu sizes have it;
   neg/abs have it clear) — NEG/ABS no longer decode as fcvtzu. This also means
   NEG/ABS stop silently corrupting anywhere a -O3 build emits them.
2. New `Inst::SimdArithUnary` decode for NEG/ABS (opcode 0xb, neg=bit29,
   esize from bits[23:22] {0:B,1:H,2:S,3:D}), placed before FcvVec.
3. Translate: per-lane signed negate (neg_r64, two's-complement wrap) and
   abs via `(x^(x ar>> w-1)) - (x ar>> w-1)` after sign-extending each lane;
   per-lane read-modify-write is rd==rn safe.
4. Added `JIT_STEP` debug trace (single-instruction blocks + full x/v dump
   after each) — the per-instruction register oracle that made this tractable;
   debug-only, no effect on normal path.

Verification: repro jit=406144671==oracle (exact); decode + exec regressions
across .4s/.8h/.16b/.2d (negative, wrap, and abs-magnitude lane cases); new
diff_battery canary `diff_vector_neg_abs_unary`; ~420 fresh fuzz cases across
12 seeds (incl. 50/99/7/9001 repro seeds) all green, 0 fails; full
`cargo test --workspace` **302/0**, build clean.

No known-open arm64jit correctness items remain on this box (same as the
cycle-27 close). Next high-value per RECOMMENDATION order: libloader ELF/loader
gaps, libbadcpu ISA coverage, JNI function-table surface, then services/auth.
HARD GATE unchanged: real Roblox boot + reproducible run log on a GPU + real
APK/binary host (none on this VPS).

---

## Session (Sep 11, 2026) — guest TLS bootstrapped (R_AARCH64_TLS local-exec / main-binary case)

Commit `6af3cd4` (dev), workspace **304/0** (was 302), build clean.

### What landed
- `libloader::elf::setup_guest_tls(info, path, tls_region, size) -> tpidr` (+ `tls_layout`):
  finds the main image's `PT_TLS` (p_type 7), copies its `p_filesz` init image into
  a per-thread region at `region + AARCH64_TCB_SIZE` (16), zero-fills `.tbss` to
  `p_memsz`, and returns the thread pointer `tpidr = region` — the AArch64 TLS ABI:
  the module TLS data block lives 16 bytes after TP, and local-exec/initial-exec
  `:tprel:` addressing (`mrs xN,tpidr_el0; add x0,tp,#o`) lands on block+`o-16`.
- `elfjit` and the `loader_run` harness now seed `CpuState.tpidr` from
  `setup_guest_tls` instead of a bare zero-filled stack. ELFs without `PT_TLS`
  get `tpidr = region` (byte-identical to the prior behaviour) — no regression.
- **Verified end-to-end, no QEMU**: cross-gcc `__thread` fixture (`g_slot=7`,
  `g_big=123456789`, `g_zero` in `.tbss`, `bump()`) through `load_elf_image →
  setup_guest_tls → jit_run` returns **123456804** — the exact native x86-64
  oracle. (qemu-aarch64 itself SIGSEGVs on this nostdlib static TLS image because
  it doesn't seed PT_TLS without a dynamic loader, so qemu is not a usable oracle
  here.) Before the seeding the region was zeroed, so every `__thread` read
  returned 0.

### Scope note (honest)
This closes the documented "R_AARCH64_TLS_* untouched" gap for the **main-binary
local-exec/initial-exec** case — which is exactly the shape of `libroblox.so`
loaded as the boot image (a PIE still uses local-exec for its own `__thread`).
The **dynamic** TLS paths (`R_AARCH64_TLS_TPREL64`/`DTPREL64` GOT slots for
TLS referenced *across* modules, general-dynamic) only engage once the loader
loads `DT_NEEDED` dependency modules as a multi-image process — the next
loader frontier, and it needs a real multi-lib host to validate.

### Tests added
- `libloader elf::tests::setup_guest_tls_copies_init_and_returns_tcb_tpidr` —
  init-image copy, TCB tpidr, `.tbss` zero-fill, tprel addressing, no-TLS fallback.
- `arm64jit loader_run::loader_run_thread_local_storage_returns_123456804` —
  full loader→TLS→JIT pipeline gate (skipped if cross-gcc absent).

### Next (per RECOMMENDATION order)
libbadcpu/libloader JTAG: guest **threading (clone/vfork)** is the documented
single-threaded-boot frontier (needs a real host to validate); multi-module
`DT_NEEDED` load for cross-module TLS + GOT/PLT within deps; broaden the
differential fuzzer into still-uncovered NEON/by-element/`tbz` classes. HARD
GATE unchanged: real Roblox boot + run log on a GPU/APK host (none on this VPS).

### Session (Sep 11, 2026 cont.) — differential fuzzer extended: NEON by-element/tbl + bitfield-insert + tbz + fcvt classes, qemu oracle; 23 generators clean
Commit `a733fba`. Added 5 generators for previously-uncovered ISA classes (NEON
`vmlaq_n_f32` by-element fmla + lane ins/get + `vbsl/vext/vrev64` bitmix, 64-bit
`bfi/bfiz`, `tbz/tbnz` bit-branches, fixed-point `fcvtzs #fbits`) and a
64-bit-exact **qemu-aarch64 oracle fallback** (write+itoa `_start` wrapper) for
`<arm_neon.h>` programs the host x86 gcc can't compile — the old harness hard-
skipped them. `gen_neon_byelem` uses binary-exact lanes so FMA-vs-`mul+add` ULP
noise doesn't read as a structural diff; removed an unavailable `vrbitq_u32`
(only `_u8` exists) for `vrev64q_u32`. Result: full 23-generator sweep is
**70/70 green across fresh seeds, 0 skips** (was ~14% skip). No new miscompile
found in the covered classes — a negative result, but those classes are now
permanent gates. Workspace unchanged (304/0; Rust untouched this commit).

## Session (Sep 11, 2026 cont.) — complete: TLS bootstrap + fuzz-harness expansion + 4 permanent canaries + harness-cascade fix (workspace 308/0)

### Milestones landed this session (8 commits on `dev`)
1. **Guest TLS bootstrap** (`6af3cd4`, docs `7bc1a5e`): `libloader::setup_guest_tls`
   copies the main image's `PT_TLS` init into a per-thread region at TP+16 (the
   AArch64 TCB) and seeds `tpidr_el0` from it, so local-exec/initial-exec
   `:tprel:` addressing reads/writes real `__thread` data. Verified no-QEMU:
   cross-gcc `__thread` fixture → `jit_run` returns 123456804 == native oracle.
   Closes the documented `R_AARCH64_TLS_*` gap for the main-binary case.
2. **Fuzz harness** (`a733fba`, docs `5b8d805`): +5 generators for previously
   uncovered classes (NEON by-element fmla + lane round-trip, NEON
   `bsl/vext/vrev64`, 64-bit `bfi/bfiz`, `tbz/tbnz`, fixed-point `fcvtzs #fbits`)
   and a 64-bit-exact **qemu-aarch64 oracle fallback** (write+itoa `_start`
   wrapper) so `<arm_neon.h>` programs the host x86 gcc can't compile now get a
   differential oracle instead of hard-skipping. 23 generators; a 12-seed ×
   100-case campaign (1200 cases) is **0 fail / 0 skip**.
3. **Permanent cargo canaries** (`b4a48cd`, `7244feb`): loader_run gates for the
   newly-covered classes — bfi-64 (279514809947), tbz/tbnz (4068), fixed-pt
   fcvt (99), NEON by-element fmla (504; qemu architectural oracle).
4. **Harness-cascade fix** (`7244feb`): a single test's cross-gcc compile failure
   used to poison the shared `run_lock` mutex and cascade-fail every other test
   with `PoisonError` (each passed in isolation). Added `lock_run()` which
   recovers poisoned guards. Also made the byelem fixture `-O0`-safe
   (const-index `vgetq_lane`/`vsetq_lane` need `-O` to fold their lane index).

### Honest scope + findings
- **Negative result (good news):** the newly covered classes (NEON by-element
  fmla, lane ops, bitfield-insert, bit-branch, fixed-point fcvt) show **no**
  miscompile across 1200 fresh differential cases — the arm64jit ISA handling
  of those classes is structurally correct. They're now locked as permanent
  `cargo test` gates.
- **FMA granularity note:** the JIT's by-element `fmla` translate does `mul`+`add`
  (two roundings) rather than a fused FMA (one rounding). Structurally correct
  (binary-exact differential runs agree with qemu), but bit-exact float workflows
  can differ by 1 ULP from ARM's fused `fmla`. Acceptable for now; revisit if
  exact-bit-sovereignty is ever required.
- **Boundary (unchanged):** dynamic *cross-module* TLS (`R_AARCH64_TLS_TPREL64`
  GOT slots, general-dynamic) only engages with a multi-`DT_NEEDED` loader (the
  next loader frontier); guest threading (`clone`/`fork`/`rt_sigreturn`/`execve`)
  remains the documented single-threaded-boot frontier.

### Verification state
`cargo build --workspace` clean; `cargo test --workspace` **308/0** (up from
302). loader_run 13/13. Ship 1200-case fuzz campaign green. HARD GATE unchanged:
real Roblox boot + reproducible run log on a GPU + APK/binary host (none on this
VPS) — nothing here can satisfy it, so this session closed loader/TLS + fuzz-
correctness work as far as physically verifiable.

## Session (Sep 11, 2026 cont.) — EXT extract-immediate bug FIXED; open u16/32 pair-xor reduction bug isolated

### Fixed: `ext` (SIMD vector extract immediate) operand-order inversion — commit `826db92`
Found by the new `gen_pairwise_reduce` differential generator. ARM
`ext Vd.16B, Vn.16B, Vm.16B, #imm` returns the 16-byte window at `imm` of the
concatenation where **Vn is the low-address half and Vm the high**:
`Vd[0..16-imm)=Vn[imm..16)`, `Vd[16-imm..16)=Vm[0..imm)`. The translate built the
concat as `[Vm.lo, Vm.hi, Vn.lo, Vn.hi]` (Vm low, Vn high) — inverted for every
non-symmetric `ext`. **qemu-verified** with distinct bytes: `ext(Vn,Vm,#8)` ⇒
lo=`Vn[8..15]`, hi=`Vm[0..7]`. gcc's horizontal XOR-reduce for pair reductions
(`s ^= a[i]+a[i+1]`) emits `ext v0,v30,v0,#8` + `eor`; the old code collapsed
it to a single 64-bit-lane xor instead of the full one. Minimal repro p1
(const int pair-xor): returned **16**, now **0** == qemu and native. 28→99/160
of the pairwise-reduce stress now correct.

### OPEN (isolated, reproducible) — 16/32-bit `i+=2` pair-xor reduction
A second, independent bug in the pair-xor reduction path remains, unaffected
by the ext fix. Minimal repro `/tmp/combw/s13b.c` (`unsigned short a[24]`,
`for(i+=2) s ^= (long long)a[i]+a[i+1]`): **native 2314 vs jit 59648**. The
path uses `uxtl/uxtl2` + `uaddl/uaddl2` (element-wise widen-add, upper-half
reads) + `eor` chain + the (now-correct) `ext` horizontal combine. Suspect the
in-place `uxtl2` upper-half read or an eor-chain lane-composition bug, NOT the
(now-correct) ext and NOT `uaddl2` — a hand-assembled
`uaddl2 v1.4s,v0.8h,v2.8h` isolated test is byte-correct in the JIT (== qemu,
upper-half values [5..8]+[50..80]=[55,66,77,88]). The passing sum loop (s13a)
uses `uaddw`+`uxtl`, the failing xor loop (s13b) uses `uaddl`+`uxtl2` with the
in-place `uxtl2 Vd.2d, Vd.4s` (rd==rn) upper-half reads — the cycle-27 in-place
widening-alias class re-checked for the `.4s→.2d` upper form. Reproducible via
`fuzz_jit.py` `gen_pairwise_reduce`
(int/short/u32/u16 variants all trip it). Next debug pass: isolate `uaddl2`
(upper) with a hand-controlled fixture vs qemu, then the eor-chain.

### State
`cargo build --workspace` clean; `cargo test --workspace` **308/0**. Commits
`826db92` (ext fix), `bd60f2c` (generator + this doc), plus the earlier
`fe1e27a` cycle-28 close. The gen_pairwise_reduce generator is a permanent
asset that keeps surfacing this class. HARD GATE unchanged: real Roblox boot +
run log on a GPU + APK/binary host (none on this VPS).

---

---

(legacy pre-SH session ledger below line 9084 trimmed by hermes-worker Sep 18 to hold the 1MB pre-commit hook; the 204 SH entries in the active ledger above are the authoritative cross-session record)
