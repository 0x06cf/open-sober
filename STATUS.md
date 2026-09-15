# Open-Sober STATUS.md (worker ledger)
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