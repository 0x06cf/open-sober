# Open-Sober STATUS.md (worker ledger)
## Session (Sep 16, 2026, hermes-worker): SH207 — RESOLVER-MAP GATE CLOSED AT FRESH SOURCE-DEPTH + futex-requeue flake FIXED. Doc docs/frontier-sh207-resolver-source-empty.md. Workspace green (387/0 arm64jit + all crates). Commit 9f39594.
- **FRESH MEASUREMENT (2/2 clean ladder runs):** the bulk-registrar **SOURCE map 0x106dca0e90 is EMPTY {0,0}** — the one container SH191/194 never read (dest 0xe70 + register 0xf60 only). Region-watch confirms registrar 0x2208ae8 exercised in-ladder (0x102208ae8/b6c/cf8/e38 = rehash+insert blocks) but with source+dest both EMPTY it cleanly **no-ops**, NOT corruption-destined. Reconciles SH191/194 "resolver EMPTY" with SH194 "registrar region fires every run"; SH192/194's bad_weak_ptr = driving registrar standalone w/ fabricated key (not repeated, measured only).
- **VERDICT (do-not-re-tread):** getService name→classid resolution sits behind the live class-registry world-build that populates the source — Route-B live-DM structural gate UNCHANGED. No headless seed populates the resolver source.
- **GENUINE PROD FIX:** `futex_requeue_actually_moves_waiter` flake fixed (waiter WAIT timeout 5s→60s; REQUEUE side spins ~20s; short timeout made the flake under parallel load). Verified green under full workspace load.
- **+1 hermetic** sh207 (three registry-map addresses/adjacency/source-selector 0x20-past-dest).
## Session (Sep 16, 2026, hermes-worker): SH202 — ON-DEMAND single-site V2 singleton-dispatch family patcher (SH201 §6's named lever). Commit 783cd28. Doc docs/frontier-sh202-v2ondemand.md. FIRST full V2 ladder completion at HEAD. Workspace green (564/0, +3; arm64jit lib 386/0, elfjit example 64/0).
- **CODE (arm64jit/src/jit.rs, default-inert, env JIT_ROUTEB_V2_ONDEMAND):** `v2_family_bl_target_l` (imm26 branch decode, CORRECT sign-ext in i64: subtract 2^26=0x400_0000 NOT 2^30 — the SH201-style bug, falsified by a backward-bl regression test), `v2_family_blr_from_guest` (pure family classifier: bl 0x6249eb8 getter within 16 back + ldr x8,[x0] after + past-0x60 ldr x8,[x8,#N] within 4 of blr), `v2_ondemand_object` (stable leaked 0x80 all-leaf singleton), `v2_ondemand_patch_at` (classify+patch+drop cache+return rewind pc). Hooked into the outside-image stop in jit_run_inner; rewinds state.pc to the window start and continues. Clears ONLY the exact blr the run dispatches through (avoids SH201's family-wide over-patch SIGABRT). +4 hermetic incl. real-image guard (3 stop sites + 4 SH200 sites ALL classify).
- **EMPIRICAL (real libroblox.so, llvmpipe, canonical 9-rung ladder):** with V2_ONDEMAND=1 the run patches exactly the site(s) it dispatches through (1-3/run) and returns past them. **FULL LADDER COMPLETION OBSERVED**: V2InitWithParams Ok(0x3e8) → V2StartAppWithParams Ok(0x3e8) → V1 AppStart__ Ok → V2UpdateSurface Ok (XID 0x200000) → SendAppEventOnAppReady Ok(0x3e8) → EXIT 124, 0 crash (runs/sh202-ondemand-full-ladder.txt). FIRST full V2-ladder completion (V2Start was "observed, run-variable" at SH200). **Honest boundary:** 1/4 runs completes clean; 3/4 patch the family then advance DEEPER into a NEW reachable wall — SIGSEGV pc=0x7f00000022b0 (host-call slot) fault [x0+0x28] lr=0x102b53a78 — a host thunk deref'ing a NULL/small arg, the next seed target (forward motion, not regression). **Default path unchanged** (V2_ONDEMAND off ⇒ 0 patches, EXIT 124, SH200/201 baseline).
- **STANDING:** Route-B live-DM world-build = structural gate (unchanged). The new post-family fault (host thunk slot 0x22b0, fault [x0+0x28], lr 0x102b53a78) is the next unblocked seed: drive whatever object lr's caller threads into it, or confirm it's the same live-world-build gate (do-not-chase if so). Keep do-init→app-shell→governor continuation + live-DM line as the primary front.
## Session (Sep 15, 2026, hermes-worker): SH178 — Route-B cone CORRECTION: the "ExperienceController::join make_shared<DataModel>" premise is MISATTRIBUTED (reloc-level law wall). Workspace green (550/0). Doc docs/frontier-sh178-routeb-join-misattribution.md. No code change.
- **3-agent fresh Route-B cone (operator's "return to Route B" directive) + independent packed-RELA decode:**
  (1) The text the prior cones cited for "the only make_shared<DataModel> is inlined at
  ExperienceController::join" is MISLABELLED. file 0x2206c40/0x2206d74 = GlobalInit do-init
  (ldar once-guard -> bl 0x2173b3c -> store [0x106a68408]; operator-news only 0x28/0x40/0x100/0x1000,
  no DM-size). file 0x2173b3c = string-intern GetOrCreate (strcmp + hash-bucket), NOT an RTApp
  app-registry / DM allocator. 0x2206db8 dispatch needs [obj+#32]=real vtable'd obj whose +0x30=
  real ctor = the SH156 JIT_ROUTEB_DM_SEED path to the governor dead-head. The genuine DM alloc
  site was NEVER located — it sits in a region no cone reached.
  (2) LAW-LEVEL WALL PROVEN AT THE RELOC LEVEL (main-loop decode of 568,272 packed ANDROID_RELA):
  the reloc writing guest 0x106391908 (setDataModelToCurrent getter return / SH172 holder) =
  (0x6391908, R_AARCH64_RELATIVE, addend 0x6358d40) -> loader writes base+0x6358d40 from an
  all-zero on-disk .data.rel.ro cell; 490 relocs populate [0x1063915a0,0x106392600). The DM
  holder/registry is RELOCATION- + SESSION-populated (wired consumer std::function + live current
  shared_ptr exist only post-session) — circular with the construction we lack. Not seedable.
  (3) R1 (synthetic CoreScript) reconfirmed dead for the CORRECT reason: rbxasset://scripts/
  CoreScripts at 3 pull-only URI-builder sites; CoreScriptLoader 0x101f1d8ac has ZERO direct bl
  callers; filesdir 0x10726d600 never joins a CoreScripts Lua path. R2 (UniversalApp.rbxm) still
  blocked on the same live-DM (0x229006/0x288090/0x2d87b18). Type-4 producer [0x106829ea8]
  latent-only. Present-walker node-list = Route-A plane (do-not-polish).
- **VERDICT:** do NOT build the headless dynamic-DM-trace attempt (re-treads SH156/164c, targets a
  misattributed address, expected fault on .data.rel.ro registry). Live-DM = LAW-level migration
  wall; the SH169 DELEGATE+VALIDATE capture latch is the ready observer for a real app-launch
  (GPU host / real input). Future re-chase must first locate the genuine join inline via a fresh
  packed-RELA decode of the DM vtable/ctor, not reuse 0x2206d74/0x2173b3c.
- Cookie line fully persisted (SH175/176/177 committed, 3006fd3/8ca920e/060eeb7). Workspace
  550/0. /tmp cleaned (453M used on 7.7G).