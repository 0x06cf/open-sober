# Open-Sober run ledger (hermes-worker)

## Session (Sep 18, 2026): SH304 — SESSION-GATED type-4 producer (recon-v3 self-drive handoff), implemented + hermetic-tested.

**State at HEAD**: `dev` (pre-commit). Workspace green (cargo test --workspace: arm64jit 405/0 + all suites; elfjit examples 130/0 = 127 baseline + 3 new sh304; TEST_EXIT 0). Cargo build --workspace 0 errors. Recon-v3 deliverable re-verified green at HEAD first (24 task frames swap Ok(0x1), 197 pops, 0 json abort, EXIT 124). No research subagents (cone suppressed — operator directive).

**This cycle, in order:**
1. Established baseline: recon-v3 SELF-DRIVEN FRAMES re-verified green (`capture_taskv4_frame.sh`), workspace green.
2. Re-measured the primary-lever SESSION-CTOR drive (`capture_sh269_session_ctor_exec.sh`) at HEAD: bus1-3 = MessageBus.subscribe Ok(0x3e8) + once-guard latches 0x1 + DM-root 0; game reached; appev-A/B at the documented GOVFLAG walls (0x102ea0b9c / 0x102bb803c preload-overrides). Route-B live-DM gate UNCHANGED — all named levers are built and at measured structural walls.
3. Audited the tree against the reconciled recon-v3 / SESSION PRODUCER HANDOFF deliverables. Found the ONE genuinely-absent named deliverable: the **session-gated type-4 producer** (`--taskv4-seed session`) that replaces the harness seed on a real session and stays inert otherwise.
4. Implemented it (elfjit.rs, default-inert): `session_producer_gate` (pure, hermetic), `session_live_dm()` (page-guarded holder [0x106391908] / DM-root [0x106a68818]), `type4_session_gated_thunk` (GATED -> real present via engine make-current/frame-fn/swap; UNGATED -> inert no present), wired into the `--taskv4-seed` install site.
5. Added 3 hermetic sh304 gate tests; verified (arm64jit examples 130/0).
6. MEASURED on real libroblox.so (`runs/capture_sh304_session_producer.sh`): registered at 0x7f00000001d0, boot dispatches log UNGATED (app_ready=false, live_dm=true) — inert, GATED=0, 0 json abort, EXIT 124. Session-less boot stays frame-free (only the standalone --renderinit/--renderframe SH18/19 swap, independent of the gate).
7. Held the 1MB pre-commit hook: condensing elfjit.rs verbosity (SH115/117/118/122/156/157/159/177/223/226/236/255/264/272/276 + option-doc prose) back-filled the ~2.5KB addition; all facts/addresses preserved, no behavior touched. elfjit.rs now +47B margin.
8. Updated repo HANDOFF.md + this ledger + frontier doc. NOT pushed to origin (operator pushes).

**Honest status (unchanged)**: Route-B live-DM structural gate UNCHANGED — MH_* false until a real do-init owns a live DM, DM-root [0x106a68818] stays a harness seed (never a genuine make_shared<DataModel>). SH174 capture-latch (arm *(0x106391908) at a real session make_shared) remains the single forward hook. The new session-gated producer is latent-but-correct: it fires the instant a real session advances (the exact recon-v3 §B / SESSION-PRODUCER-HANDOFF contract), and its hermetic test + real-binary inert measurement prove the gate today.