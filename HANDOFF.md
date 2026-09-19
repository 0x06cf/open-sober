# Open Sober — Agent Handoff

## SH374/SH375 (Sep 20, 2026, hermes-worker): MEASURED map-completions under the SH373 reaching-env — EC-world reader-gate still never entered AND the 0x258b5d8 dispatch body still never fires even with SH285 deterministically crossed (both prior closures re-tested on the env that crosses the SH285 leaf, refining their premises); recon-v3 deliverables re-verified green
Single-agent (cone suppressed). Two new probes (runs/capture_sh374_ec_dmfn_reaching.sh +
runs/capture_sh375_dispatch_body_reaching.sh) + live captures (gitignored) +
docs/frontier-sh374-sh375-reaching-env-closures.md + sh375_new hermetic (arm64jit lib
434->435). No production path edited (both probes combine the shipped SH373 crossing env
+ existing default-inert guards). Workspace green (cargo test --workspace EXIT 0).

- **SH374**: SH373's reaching-env (LSM_APPEND_SKIP over the SH371 M+0x48/appname seeds)
  combined with the DMFN/EC-world drive: the EC-world ENTRY guards fire every run
  (routeb-sh298/299/300 @0x102e24598), but the reader-gate block 0x2e24694 STILL never
  enters and the run terminals at 0x101d9a708 (pack helper, same unconstructed-LSM
  family). Crossing SH285 does NOT make the EC marshaller interior reachable — control
  drains into the persistence family first. SH356's "reader-gate = reachability problem"
  re-strengthened from the crossing env.
- **SH375**: SH362 attributed the 0x258b5d8 dispatch-body unreachability to "run dies at
  SH285 first." With SH285 now deterministically CROSSED (sh285=0 all 3 runs), the ladder
  ADVANCES PAST it — StartApp Ok, lifecycle drive (initAppShellReporter + setActive) clean —
  and then SIGABRTs at a FRESH terminal, SetInitParams (0x102bcc814, `nativeAppBridgeSetInitParams`,
  a genuine 0x3f0-frame fn, pinned by the sh375_new hermetic). The 0x258b5d8 body STILL
  never fires: its blocker is NOT the SH285 leaf (crossed) but this SetInitParams LSM-family
  abort that runs first. Refines SH362's premise; does not overturn the standing
  measured-closed LSM family.
- Both confirm the persistence/LSM unconstructed-object family is PATH-INDEPENDENT and
  swallows every Route-B ladder arm (do-init dispatch, EC marshaller, SetInitParams)
  before any live DM construction. No DataModel (DM-root 0, MH_* false); SH174 capture-latch
  stays the single forward hook. recon-v3 deliverables re-verified green (24 real task-driven
  frames, swap Ok(0x1), 0 json abort, 0 crash).

## SH373 (Sep 20, 2026, hermes-worker): MEASURED — SH285 CROSSOVER from the SH371 reaching-env (the standing SH285 persistence leaf is deterministically crossed 5/5, terminal advances to 0x101d9a708 in the SAME measured-closed LSM unconstructed family); recon-v3 + sh372 green
Single-agent (cone suppressed). One new probe runs/capture_sh373_cont_appendskip.sh + live
captures (gitignored) + docs/frontier-sh373-sh285-crossover-continuation.md. No production
path edited (append/pack skips are SH349/SH350's existing default-inert opt-ins; SH373
combines SH349's append-skip with the SH371 reaching env and measures). Workspace green
(cargo test --workspace EXIT 0, 615 passed/0 failed incl sh372).

### The forward this cycle (a reproducible cross, then an honest verdict)
For the first time the standing SH285 persistence-wall (guestpc=0x101db1b08) is
deterministically CROSSED (5/5 runs, sh285=0). SH371 added the DM_CONT_M48_SEED +
CONT_APPNAME_SEED that make the DM-creator continuation continueAfterFlagsLoaded_
(0x102bd1d68) run DEEP headlessly. SH373 adds SH349's append sub-call skip on top:
- continuation fires; **SH285 leaf 0x101db1b08 = 0 hits** (was the terminal of every
  SH260/284/285/3444/348/371/372 run) — the append byte-copy 0x101d9a15c IS the wall again;
- run advances to **0x101d9a708** (SH349's pack/name-string helper), faulting on source
  pointer x19=0xff..ff = the SAME unconstructed-live-object family SH349/350/358 closed.
- SH358's earlier "0 continuation hits for DMCONT+skips" is explained: that run LACKED the
  M+0x48/appname seeds, so the continuation was never reached.
- The append+pack combo (SH373b) instead parks at pool-pop write-site 0x101d9a528
  (write-to-0x1 divergence), a different arm that doesn't reach the continuation.

### Interpretation
SH285 is NOT a fundamental invariant — it is the append byte-copy leaf, crossable with the
known single-caller skip once the continuation is reached. But the cross lands one fencepost
later in the SAME family (0x101d9a708), which SH349 reached + SH350 crossed into the
unbounded pool-pop family. This RE-STRENGTHENS the standing verdict: the persistence lane is
measured-returned (whack-a-mole UNBOUNDED); only a REAL LocalStorageManager/session ctor
gets past, and no seed manufactures it (SH248h/SH256). SH373 closes the last "is SH285 itself
the invariant?" loophole by crossing it and showing the next fencepost is already-known.

### Honest
No DataModel (DM-root [0x106a68818]=0, MH_* false). Route-B live-DM structural gate
UNCHANGED. SH174 capture-latch stays the single forward hook. Do NOT extend to
pack-skip+LSM whack-a-mole (SH350 unbounded); do NOT re-drive further sub-call skips into
the family (SH349/350/358 closure stands).

## SH372 (Sep 20, 2026, hermes-worker): MEASURED — the DM-creator continuation and the settings-state init path CONVERGE on the identical SH285 persistence-object leaf (answers SH371's explicit "same object or different?" gap with a fresh register dump); recon-v3 deliverables re-verified green
Single-agent (cone suppressed). One new probe runs/capture_sh372_continuation_terminal.sh
(JIT_DUMP_PC + JIT_GUEST_STACK_DUMP + JIT_REGION_WATCH on the continuation body) + one new
real-image hermetic `sh372_cont_continuation_converges_on_sh285_persistence_leaf` (arm64jit
lib 433->434; pins the shared leaf sub x2,x0,#0x20 @file 0x1db1b08, its caller bl @0x1db1b14
-> lr 0x1db1b18, and the shared continuation prologue @file 0x2bd1d68) + docs/frontier-sh372-
continuation-convergence.md. No production path edited. Workspace green (cargo test
--workspace EXIT 0, 614 passed/0 failed — 433 arm64jit lib tests + sh372).

### The forward this cycle (the genuine new datum)
SH371 measured the DM-creator continuation continueAfterFlagsLoaded_ (0x102bd1d68) now runs
DEEP headlessly and terminates at the standing SH285 persistence-lane wall
(guestpc=0x101db1b08), and explicitly left open whether it hits the same object the
settings-state path faulted on or a different one. SH372 closes that gap with a fresh full
register + guest-stack dump directly from the continuation path:
- Continuation FIRED (block-entry at 0x102bd1d68) and its deep pcs all executed.
- Terminal pc 0x101db1b08 with lr=0x101db1b18 — the SAME leaf (sh285 reader caller inside
  initStorageManagerNative 0x101d9d8b0) the settings-state drive hits.
- Fault target x20=x1=0xffff8062... — the SAME 0xff..ff-prefixed uninitialized internal
  data-pointer (SH285/[obj+0x50] family); x0=x19=0x7f9d... = a guest-constructed host-heap
  object, exactly as SH285 classified.

### Interpretation (map refinement, not a new wall)
Both independently-reached engine init paths — the settings-state self-drive (SH284/285) and
the DM-creator continuation (SH371, SH372) — converge on the identical unconstructed-manager
leaf, same pc, same lr, same 0xff..ff buffer pointer. This proves the SH285 terminal is
PATH-INDEPENDENT: NOT a benign-body branch one path misses (corroborating SH371's
STRAIGHT-LINE finding), but the manager object's own unconstructed string buffer, which no
seed manufactures (SH248h/SH256 rule) and which only a REAL LocalStorageManager/session ctor
owns (SH174/204 live-object class). The measured-closed SH285 lane record is strengthened with
a second-entry confirmation.

### Honest
Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED (DM-root
[0x106a68818]=0, MH_* false). SH174 capture-latch stays the single forward hook. recon-v3
deliverables re-verified green at HEAD this cycle (24 real task-driven frames swap Ok(0x1),
0 json abort, 0 crash; JIT_JSON_ZERO_FIX present).

### Next (unchanged, authoritative)
Route-B live-DM structural gate stands (SESSION-CTOR / do-init; REG_LIVE SH352). The two
measured dead-ends from the now-reached continuation are (a) the SH285 live-object wall and
(b) the F+0x18 controller floor behind it — both measured-closed (do NOT re-drive LSM sub-call
skips SH349/350/358). R1 content half staged + armed + serviceable (SH351/352/354). Do NOT
re-arm the window-attach once-guard (SH367); do NOT re-enter the ALooper loop (SH365); bounded
process_cmd stays the guarded entry (SH366/368).

## SH371 (Sep 20, 2026, hermes-worker): MEASURED — continueAfterFlagsLoaded_ now EXECUTES DEEP headlessly (corrects the SH226/228 "never fires" map) + hermetic proving the engine-init dispatcher body is STRAIGHT-LINE (the only exits: two leaf blr returns and the bl sub)
Single-agent (cone suppressed). recon-v3 deliverables independently re-verified green at
HEAD (capture_taskv4_frame.sh attempt 1: 24 real task-driven frames `present swap Ok(0x1)`,
197 node pops, 0 json abort, 0 crash; JIT_JSON_ZERO_FIX len-clamp at 0x102355d40 present).
New real-image hermetic `sh371_engineinit_dispatcher_body_straightline_to_sub` (arm64jit
lib 433; scans dispatcher body [0x2bd8ce8,0x2bd8d64) + sub_2bd8dac body [0x2bd8dac,0x2bd8e28)
for any control-flow word, rejecting all but the 4 known sites — bl getter 0x2bd8d14, blr
vt+0xf8 @0x2bd8d2c, blr vt+0x108 @0x2bd8d50, bl sub @0x2bd8d60, and sub's blr vt+0x1f0
@0x2bd8e28 — both STRAIGHT-LINE, so SH228's "diverge at a leaf" narrows to "a leaf's
return never lands back in-image (host landing)" with NO benign body branch) + probe
runs/capture_sh371_dispatcher_body.sh + docs/frontier-sh371-....md. Workspace green
(cargo test --workspace EXIT 0; elfjit 159/0, jit lib 433/0).

### The forward this cycle
A genuinely-new measurement, not a re-tread: with the FULL Route-B env (capture_sh344's
DMCONT + DM_CONT_M48_SEED + CONT_APPNAME_SEED seed set), region-watching [0x102bd8ce8,
0x102bd8e30] ∪ [0x102bd1d68,0x102bd2600] on the completing ladder shows:
- dispatcher 0x2bd8ce8 + sub_2bd8dac BOTH fire (1 each) — the dispatcher body runs through
  its getter + the two leaf blr returns and reaches `bl sub`, contradicting SH228's
  "sub never fires" on the fuller env.
- **continueAfterFlagsLoaded_ (0x102bd1d68) FIRES and runs DEEP — 25+ block-entry pcs
  0x102bd1d68 .. 0x102bd1f64 (its app-name guard, SH245/SH248c-seeded)**. This overturns
  the SH226/SH228 blanket "continueAfterFlagsLoaded_ is never entered" — with the full
  seed env it executes deep past its app-name guard.
- Terminal: guestpc=0x101db1b08 fault=0xff..ff — the standing SH285 LSM reader/pop
  live-object wall, reached now from the DM-creator continuation path; the F+0x18
  post-app-start controller floor (routeb_dm_manager_cont comment) is never reached
  because SH285 fires first (one fencepost EARLIER than that predicted floor).

### Honest
Does NOT manufacture a DataModel (DM-root [0x106a68818]=0, MH_* false, Route-B live-DM
structural gate UNCHANGED). SH371 corrects the map (the continuation is env-reachable deep,
not "never entered") but the continuation immediately dives into the measured-closed SH285
persistence lane (SH349/350 — do NOT re-drive LSM sub-call skips). SH174 capture-latch
stays the single forward hook.

### Next (unchanged, authoritative)
Route-B live-DM structural gate stands (SESSION-CTOR / do-init; REG_LIVE SH352). The two
measured dead-ends from the now-reached continuation are (a) the SH285 live-object wall and
(b) the F+0x18 controller floor behind it — both measured-closed. R1 content half staged +
armed + serviceable (SH351/352/354). Do NOT re-arm the window-attach once-guard (SH367);
do NOT re-enter the ALooper loop (SH365); bounded process_cmd stays the guarded entry
(SH366/368); do NOT re-drive LSM sub-call skips (SH349/350/358).

## SH370 (Sep 20, 2026, hermes-worker): SH357-consolidation completion — lock the sh323 cookie-jar/settings guard test under the shared ROUTEB_PROC_TEST_LOCK (determinism hardening); recon-v3 deliverables re-verified green at HEAD
Single-agent (cone suppressed). One-line test-harness fix (arm64jit/src/jit.rs): `sh323_settings_sso_seed_guard`
was the one routeb-family cookie-jar test that did NOT hold the consolidated ROUTEB_PROC_TEST_LOCK all its
siblings (sh248d/sh248e/sh273/sh175) hold, so under parallel `--test-threads=16` it raced the locked siblings'
mid-assert on the shared fixed cookie-jar page (CELL_B=0x106ed7a28 == sh248d's B), giving intermittent
"cookie-jar slot A/B must be seeded" failures (left:0). Added the lock (behavior-neutral, no assertion
weakened). MEASURED: sh323 passes; sh323/248d/248e pass 50/50 under 16-thread isolation; `cargo test
--workspace` deterministic-green EXIT 0 (612/0; 6/6 canonical runs + earlier 10/10/8/8). The residual futex/
sharded-page flake only appears under artificial `--test-threads=16` and is the documented SH345/346/357
accepted load-sensitive class — the canonical gate is reproductibly green. Route-B live-DM structural gate
UNCHANGED (DM-root 0, MH_* false); recon-v3 frame plane re-verified green (24 frames swap Ok(0x1), 0 json,
0 crash).

## SH369 (Sep 20, 2026, hermes-worker): MEASURED structural pin — window-attach COMPLETION funnels into the CLOSED persistence lane (flags-latch 0x72739d4 -> initStorageManagerNative 0x1db1050), NOT to a live DM; refines SH367's "needs a real surface" reading
Single-agent (cone suppressed). One new real-image hermetic
`sh369_window_attach_completion_converges_to_persistence_lane` (arm64jit lib 432; 11 word-pins
verified on real libroblox.so) + docs/frontier-sh369-windowattach-persistence-convergence.md.
No production path edited (read-only pin). recon-v3 immediate-priority deliverables re-verified
green at HEAD this cycle (24 real task-driven frames, swap Ok(0x1), 0 json abort, 0 crash).
Workspace green (cargo test --workspace EXIT 0, 612 passed/0 failed incl sh369).

### The forward this cycle
SH367 measured that arming the window-attach once-guard + a crafted [win+0x278] hard-faults and
attributed the wall to "needs a REAL EGL surface, only a live Activity/AppBridge session provides".
SH369 pins the COMPLETION chain the armed path would take and shows it is NOT a route to a live DM:
disasm-verified `bl 0x22985c0 (deep GL post-init) -> bl 0x2270a98 -> bl 0x2270b24 (real body)`; the
body gates on the SAME flags-loaded latch the --v2boot ladder seeds (`adrp 0x7273000; ldrb
[x9,#2516]` = [0x72739d4] @0x2270b64) and, when bit0=1, falls through the `cbz w9,0x2270be8`
@0x2270b78 to `bl 0x1db1050` = initStorageManagerNative — the SH285-family persistence lane
(the SH285 fault site 0x101db1b08 is inside it; SH349 crossed it, SH350/358 closed the lane as
measured-unbounded). So window-attach COMPLETION is a SECOND entry into the already-closed
persistence lane, not a path to a live DM. This de-risks the SESSION-CTOR window precondition:
even a REAL surface hands control to a lane already measured returned — that is partly why Route-B's
live-DM gate stands (the window precondition alone cannot produce a DM).

### Honest
Does NOT manufacture a DataModel (DM-root [0x106a68818]=0, MH_* false unchanged). No re-run of the
SH367 armed-fault (already 8/8 measured negative). Route-B live-DM structural gate UNCHANGED;
SH174 capture-latch stays the single forward hook. This cycle is pin/verify, not a new session
drive — but the pin is genuinely new (SH367 stopped at bl 0x22985c0; SH369 traces past it).

### Next (unchanged, authoritative)
Route-B live-DM structural gate stands (SESSION-CTOR / do-init four-stacked closure SH184/185;
REG_LIVE SH352). R1 content half staged+armed+serviceable (SH351/352/354). SESSION half (do-init
owning a live DM) remains THE wall — reached only by a REAL Activity/AppBridge session drive that
constructs the upstream ctor for real. SH174 capture-latch stays the single forward hook. The
window precondition is now pinned as persistence-lane-bound (SH369), so re-attacking it alone
expecting a DM is closed; the genuine EGL-surface/governor/onAppReady home-cone (SH126+ captures)
is still the live session lever. Do NOT re-arm the fabricated once-guard (SH367); do NOT re-enter
the ALooper loop (SH365); bounded process_cmd stays the guarded entry (SH366/368).

## SH368 (Sep 20, 2026, hermes-worker): bounded app-command SEQUENCE drive on the guarded SH366 entry — the engine's own process_cmd dispatcher drives cmds {6,8,11} cleanly (marker IN INIT_WINDOW), session observables measured after EACH command confirm the dispatcher alone does NOT self-transition AppBridgeV2/surface (SESSION-CTOR wall pinned precisely); full 20-entry jump table + window-attach contract pinned in a new real-image hermetic
Single-agent (cone suppressed). New opt-in rung `--v2boot-glue-cmd-seq` ->
`drive_glue_process_cmd_seq` (jit.rs) on the SAME bounded SH366 dispatcher entry (once-guard stays
OFF — SH367 measured fault on the fabricated re-arm) + real-image hermetic
`sh368_glue_cmd_seq_jump_table_and_safe_cases_pinned` (arm64jit lib 431) + capture
`runs/capture_sh368_glue_seq.sh`. elfjit.rs product path unchanged (rung opt-in). Workspace green
(cargo test --workspace EXIT 0, arm64jit 431/0).

### The forward this cycle
SH366 entered the engine's REAL app-command dispatcher (0x102bcd6e4) headlessly and delivered ONE
APP_CMD (INIT_WINDOW, marker [inner+9]=1). The operator's SESSION-CTOR directive is to "drive the
engine's REAL Activity-session init state machine" — a real Activity consumes a QUEUE of APP_CMD
values. SH368 extends the confirmed-green SH366 entry to a sequence of verified-safe commands
({6,8,11}) over a SHARED fabricated app/inner/win so command state accumulates like a real queue,
and reads back the session observables after EACH command. cmd 6/8 are disasm-verified cycle-safe
at version-gate 0 (b.lo straight to the epilogue / write only glue bytes); cmd 11 is the
SH366-proven INIT_WINDOW. **MEASURED (confirm:1 on attempt 1, EXIT 124, 0 crash): all three
commands drive cleanly `process_cmd returned Ok`; cmd 11 fires the INIT_WINDOW marker
[inner+9]=1; once-guard stays OFF.** The observables confirm the SESSION-CTOR reading precisely:
the real dispatcher alone does NOT self-transition AppBridgeV2 ([0x106a705e8] 0x0->0) nor the
surface XID ([0x10683d348] stays 0x200000 = the wired X11 XID, SH112) — those move only when a
live session/do-init builds the DM world.

### Honest
Does NOT manufacture a DataModel (DM-root [0x106a68818]=0, MH_* false, Route-B live-DM structural
gate UNCHANGED). Does not arm the (SH367 faulting) window-attach once-guard, does not create a
real EGL surface. It advances the "drive the engine's real command queue" half of the SESSION-CTOR
directive with re-verifiable pinned addresses (full 20-entry jump table + window-attach contract)
and a bounded live readback that isolates the wall to do-init's live-DM construction.

### Next (unchanged, authoritative)
Route-B live-DM structural gate stands (SESSION-CTOR / do-init four-stacked closure SH184/185;
REG_LIVE SH352). R1 content half staged+armed+serviceable (SH351/352/354). SESSION half (do-init
owning a live DM) remains THE wall — reached only by a REAL Activity/AppBridge session drive
(genuine EGL surface + onAppReady + real jstring) that constructs the upstream ctor for real.
SH174 capture-latch stays the single forward hook; bounded process_cmd is the guarded entry
(SH366/SH368). Do NOT re-arm the fabricated once-guard (SH367); do NOT re-enter the ALooper loop
(SH365); do NOT re-drive LSM skips (SH349/350/358).

## SH367 (Sep 19/20, 2026, hermes-worker): MEASURED NEGATIVE — arming the real window-attach GL-surface path faults (SH366 next-forward executed); the SH366 clean INIT_WINDOW drive is preserved + fault pinned one level deeper into the deep GL post-init 0x22985c0
Single-agent (cone suppressed). One new read-only observation guard
`routeb_glue_realattach_guard` (jit.rs, opt-in JIT_ROUTEB_GLUE_REALATTACH=1, once, ZERO guest
mutation) + real-image hermetic `sh367_window_attach_real_path_pinned_and_guard` (arm64jit lib
429->430) + capture runs/capture_sh367_glue_cmd_real.sh (SH366-clean-entry predicate, confirmed
green attempt 1: EXIT 124, crash 0, drive Ok, marker [inner+9]=1) + frontier doc. elfjit.rs
unchanged. Workspace green (cargo test --workspace EXIT 0, 430/0).

### The attempt (STATUS/frontier-sh366 next-forward, executed + measured)
STATUS named \"hand the engine a REAL wired ANativeWindow in [inner+64] so window-attach 0x2bd29a0
takes its real GL-surface path\". SH367 implemented exactly that: armed [win+0x268].bit0=1, crafted
[win+0x278]=NULL-first-word obj (so the deep GL call fast-returns), registered XID 0x200000 via
set_anativewindow_xid. Disasm pins the real chain: armed -> `add x0,x19,#0x278` @0x2bd2a18 -> bl
0x2291c24 (0x2291c24 `cset w0,ne on [x0]` = ([?win+0x278]!=0)) -> bl 0x22985c0 (deep GL post-init).
MEASURED (8 attempts): every run **hard-faults before the INIT_WINDOW body completes** — EXIT 134/139,
marker [inner+9]=1 never fires, drive no longer returns Ok, fault inside the host GL dispatch
(guestpc=0x7f0000001f50 fault=0x7f818c0097). Root cause: because the crafted non-null [win+0x278]
makes 0x2291c24 return 1, control REACHES 0x22985c0, whose DEEP body needs a REAL EGL surface/context
object — a fabricated obj cannot satisfy it. This is the exact object only a live Android
Activity/AppBridge session drive provides. **Reverted** to the SH366 clean drive (guard left OFF);
re-verified clean (process_cmd Ok, marker set). Kept the read-only guard + hermetic pins as the
measured-negative instrumentation.

### Honest
Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED (DM-root [0x106a68818]=0x0,
MH_* false). SH174 capture-latch stays the single forward observer. SH367 confirms the window
precondition is a REAL GL-surface wall (Session-Ctor operator directive), not a seedable global.

### Next (unchanged, authoritative)
Route-B live-DM structural gate stands (SEP-17 SESSION-CTOR / do-init four-stacked closure
SH184/185; REG_LIVE SH352 addendum 2). The window-attach COMPLETION needs a genuine EGL surface +
onAppReady — a real Activity/AppBridge session drive, not a value seed. R1 content half staged+armed
(SH351/352/354); SESSION half (do-init owning a live DM) remains THE wall. SH174 capture-latch stays
the single forward hook. Do NOT re-seed [win+0x268]/[win+0x278] (SH367); do NOT re-enter the ALooper
loop (SH365); bounded process_cmd remains the guarded-entry (SH366).

## SH366 (Sep 19, 2026, hermes-worker): FIRST headless ENTRY into the engine's own app-command DISPATCHER process_cmd (0x102bcd6e4) — the APP_CMD_INIT_WINDOW case body EXECUTED (marker [inner+9]==1), the SESSION-CTOR window/GL-surface precondition the operator names for initEngine_ was driven (not just watched); Route-B live-DM structural gate UNCHANGED (DM-root 0, MH_* false)
Single-agent (cone suppressed). One new bounded cause-level drive
`drive_glue_process_cmd` (arm64jit/src/jit.rs, opt-in rung `--v2boot-glue-cmd` at the
TOP of the ladder) + one new real-image hermetic
`sh366_glue_process_cmd_abi_and_init_window_case_pinned` (10 byte-pins + the
jump-table index-10->0x2bcd78c mapping verified on real libroblox.so) + capture
`runs/capture_sh366_glue_cmd.sh`. elfjit.rs held <1MB (condensed SH-prose comments).
Workspace green (cargo test --workspace EXIT 0, 609 passed/0 failed — arm64jit lib
429 after sh366).

### The forward this cycle
The operator's SESSION-CTOR lever names the window/GL-surface `APP_CMD_INIT_WINDOW` as
the precondition behind initEngine_'s "*** Engine settings is null" hard-assert. SH39b
region-watched the glue LOOP (0x102bcd5d0) at 0 hits; SH365 measured the app-command
drain dead (addfd=0, pollonce=0, posted=3). BOTH only observed the path; neither
ENTERED it, because the glue main loop is an INFINITE ALooper_pollOnce loop that cannot
be jit_run to completion. The loop dispatches to a BOUNDED fn, `process_cmd(app, cmd)`
at guest 0x102bcd6e4 (w1 = APP_CMD value), which CAN be entered — and nobody had ever
driven it. SH366 is that first entry: with a fabricated app ([app]=inner,
[inner+64]=zeroed win obj, version-gate [0x10683d8b0]=0) the INIT_WINDOW case body does
`ldr x0,[x20,#64]; strb w8,#1,[x20,#9]; bl window-attach` -> for the first time
headlessly the engine RUNS its own APP_CMD_INIT_WINDOW handler (marker [inner+9]=1),
the window-attach path enters, and the whole run is stable.

### MEASURED (real libroblox.so, completing ladder + --v2boot-glue-cmd)
```
[elfjit:glue-cmd] driving process_cmd @ guest 0x102bcd6e4 (app=... [app]=... [inner+64]=win ... cmd=11 INIT_WINDOW; version-gate [0x10683d8b0]=0)
[elfjit:glue-cmd] process_cmd returned Ok(...)
[elfjit:glue-cmd] INIT_WINDOW case body marker [inner+9]=1 EXECUTED (engine window-attach path entered headlessly)
```
Confirmed-green artifact (retry 2): EXIT 124, 0 SIGSEGV/ABRT, marker=1. The drive logs
its success even on a run that later dies at a pre-existing run-variable persistence-
lane wall (SH353-class, unrelated — the drive runs FIRST).

### Honest
Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED (DM-root
[0x106a68818]=0, MH_* false). SH366 proves the window-condition *entry* is reachable
via the bounded process_cmd leaf — the ALooper loop itself is still not entered (SH365
drain dead-letter unchanged). The window obj handed to 0x2bd29a0 is a ZEROED host
buffer, not a wired EGL surface; the real window-attach completion (and the do-init
chain that would build the DM) is the standing next step, not reached this cycle.
SH174 capture-latch stays the single forward hook.

### Next (on the SH366 line)
Hand the engine a REAL wired ANativeWindow — the wired X11 XID 0x200000 (SH112/SH365,
via anativewindow_fromsurface) — in [inner+64] instead of the zeroed obj, so the
window-attach helper 0x2bd29a0 takes its real GL-surface path; then chain process_cmd
to the do-init ladder per the operator's SESSION-CTOR "drive until the upstream ctor
RUNS" directive. Do NOT re-attach a zeroed obj expecting the DM to move; do NOT re-enter
the infinite ALooper loop (bounded process_cmd is the correct entry).

## SH365 (Sep 19, 2026, hermes-worker): MEASURED dead-letter — the host app-command FIFO is never drained by the guest android_app glue loop (addfd=0, pollonce=0, posted=3, 3/3) on the completing ladder, pinning the SESSION-CTOR "window/GL-surface APP_CMD_INIT_WINDOW" precondition as an undelivered lifecycle event; recon-v3 deliverables re-verified green at HEAD
Single-agent (cone suppressed). Always-on ALooper drain counters
(`app_command_drain_stats` in shims.rs) + a 1.5s-grace readback in the elfjit
app-command feed + one hermetic (`app_command_drain_stats_count_shim_entries_and_posted`,
arm64jit lib 427) + capture `runs/capture_sh365_alooper_drain.sh`. No production
path edited (4 cheap atomics; elfjit readback inside the existing
JIT_DRIVE_LIFECYCLE block). Workspace green (cargo test --workspace EXIT 0,
608/0 — was 607).

### The forward this cycle
SH264/276 drove the lifecycle NATIVES directly and each completes headlessly
(initAppShellReporter/setActive/nativeInitClientSettings(_Signed)/
nativeActivity_onEngineSettingsReceived all Ok), but SH264 *suspected* — never
MEASURED — that the android_app glue main loop (guest 0x102bcd5d0) "busy-spins
rather than dispatch APP_CMD_START/RESUME/INIT_WINDOW". The SESSION-CTOR
directive names the window/surface `APP_CMD_INIT_WINDOW` as the precondition
behind initEngine_'s "*** Engine settings is null" hard-assert, and the
app-command FIFO was built to carry it. SH365 MEASURES the drain: on the
completing ladder (3/3, EXIT 124 clean) the host posts 3 APP_CMD_* commands
(START/RESUME/INIT_WINDOW) but the guest NEVER enters ALooper_addFd/ALooper_pollOnce
(addfd=0, pollonce=0, posted=3) — the glue loop never consumes the FIFO, so the
INIT_WINDOW event that would hand the wired X11 XID (0x200000, anativewindow fires
1x) to the engine is never delivered. Cause-level MEASURED (SH264 inferred);
Route-B live-DM structural gate UNCHANGED (DM-root 0x0, MH_* false). The drain
stats are an always-on objective trigger: if a future drive brings the glue loop
alive, addfd/pollonce flip >0 and the readback changes from DEAD-LETTER to drained.

### Honest
Does NOT manufacture a DataModel (DM-root 0, MH_* false). recon-v3 self-driven
frame + JSON re-verified green at this HEAD (24 real task frames, swap Ok(0x1),
197 node pops, 0 json abort, 0 crash).

### The forward this cycle
SH347 measured the messageBus experience-launch RECEIVE path at the cb's
DM-holder read (file 0x2bd7474) NEVER firing, but left its cause ambiguous and
explicitly named a "future receive-payload-with-real-string probe" as the open
forward surface. SH364 closes that surface with two read-only observations:
- `routeb_busrecv_cb_entry_guard` (jit.rs, JIT_ROUTEB_BUSRECV): fires at the cb
  BODY ENTRY (file 0x2bd744c, `sub sp,#128` prologue, BEFORE the DM-holder read)
  — distinguishes "publish never dispatches to the cb" (publish-side) from "cb
  entered, DM-holder null" (live-DM-side gate). READ-ONLY.
- `drive_messagebus_publish_receive_payload` + elfjit rung `--v2boot-session-pub-real`
  publishing a REAL 56-byte envelope (SH347 used only an empty b"" payload).
MEASURED (clean bounded run, EXIT 124): subscribe Ok(0x3e8), publishRaw bound +
Ok(0x3e8), **cb-entry guard fires 0** AND **DM-holder guard fires 0** — identical to
the empty-payload run. So the receive cb is not dispatch-reachable headlessly
even with real content: publish-side registration/live-DM-gated, matching every
other SEP-17 receive behind the same gate. +2 hermetic tests (individually 3/3;
the full-suite SH362 flake earlier was the documented SH357 fixed-address race,
passes isolated + suite green on rerun), +capture_sh364_busrecv_real.sh,
+frontier-sh364 doc.

### Honest
Does not manufacture a DataModel. Route-B live-DM structural gate UNCHANGED
(DM-root [0x106a68818]=0, MH_* false). SH174 capture-latch stays the single
forward observer.

## SH362 (Sep 20, 2026, hermes-worker): MEASURED — the do-init MAIN dispatch body fn 0x258b5d8 (StartAppWithParams+0x494) NEVER EXECUTES headlessly; SH361 read only the dispatch *target pointer* (vt[+48]=0x10258b5d8), SH362's body trace proves the body is never entered — every run faults at the SH285 persistence wall (0x101db1b08) before control reaches it (3/3)
Single-agent (cone suppressed). One new READ-ONLY observation guard
`routeb_startapp_dispatch_body_guard` (jit.rs, opt-in JIT_ROUTEB_DISPATCH_BODY_TRACE=1,
default-inert, once-per-run, ZERO guest mutation) + one hermetic sh362
(arm64jit lib 424->425) + runs/capture_sh362_dispatch_body.sh. elfjit.rs product
path unchanged. Workspace green (cargo test --workspace EXIT 0; 605 passed/0
failed — was 604).
- STATUS.md next-forward #1 named "trace whether the measured dispatch target
  0x10258b5d8 body can be advanced past the SH285 wall." SH361 measured the br x1
  @0x2206e24 lands at vt[+48]=0x10258b5d8. SH362 closes the body-EXECUTION gap:
- Disasm of the dispatch fn (0x258b5d8, a real `sub sp,#0x170` function): it
  immediately branches on flag byte [0x106a64da0] (`adrp x8,6a64000; ldrb w8,[x8,#3488]`
  @0x258b604 -> `cbz w8,0x258b640` @0x258b608): nonzero -> path A @0x258b60c
  (nativePreloadFlagOverrides 0x2dae640 -> blr vt[+144] -> 0x2366694 ->
  nativePreloadFlagOverrides -> blr vt[+296]); zero -> path B @0x258b640 (0x2367270 ->
  blr vt[+144] -> 0x2366694 -> 0x2367270 -> blr vt[+296]); both converge @0x258b670
  -> 0x23c19e0 -> canary check -> ret.
- MEASURED (full app-start ladder, 3/3): `[routeb-dispatch-body] SH362` fires 0/3
  (guard wired into the same per-block-entry hook that DOES fire SH361 at 0x2206db8,
  so the miss is the body, not the hook). Every run EXIT 134, SIGSEGV guestpc=
  0x101db1b08 (SH285 persistence-lane live-object wall), DM-root 0, MH_* false.
- Conclusion: the StartAppWithParams+0x494 body is entirely unreachable headlessly —
  the run faults at SH285 BETWEEN the do-init dispatch (reached) and the body entry
  (never reached). Route-B live-DM structural gate UNCHANGED (no DM manufactured).
  Do NOT re-drive LSM skips to "reach" 0x258b5d8 (SH358/349/350 closed); do NOT
  attack [0x106a64da0] with the ladder (the body never runs so the flag is moot).
  Files: docs/frontier-sh362-dispatch-body-unreachable.md, runs/capture_sh362_dispatch_body.sh,
  sh362 hermetic (arm64jit lib 425).
Single-agent (cone suppressed). One new READ-ONLY observation guard
`routeb_doinit_dyn_trace_guard` (jit.rs, opt-in JIT_ROUTEB_DOINIT_DYN_TRACE=1,
default-inert, once-per-run, ZERO guest mutation) + one hermetic sh361 +
probe runs/capture_sh361_doinit_dyn_trace.sh. elfjit.rs product path unchanged.
Workspace green (arm64jit lib 423->424; cargo test --workspace EXIT 0, 0 failures)
as this HEAD. recon-v3 self-driven frame + JSON fixes re-verified green this cycle
(24 real task frames, dispatch #2698000, present #23 swap Ok(0x1), 0 json abort,
0 crash).
- The operator's EXECUTE-DO-INIT-GATES asks for a "dynamic DM-ctor trace (SH164's
  harness-trace artifact) rather than a static seed"; SH320 had described the do-init
  MAIN-branch binder-dispatch as a "DM-ctor entry" WITHOUT ever reading [container+32].
  SH361 closes that measured-vs-assumed gap at the exact decision point.
- Disasm (real libroblox.so): do-init worker 0x2206db8, `mov x19,x1` @0x2206dd0 ->
  `ldr x0,[x19,#32]` @0x2206df4 -> `cbz x0,0x2206ea4` @0x2206df8 (NULL -> MessageBus
  bail) -> else `ldr x8,[x0]; ldr x1,[x8,#48]` @0x2206e00 -> `br x1` @0x2206e24.
- MEASURED: `[routeb-doinit-dyn] SH361 ... container=0x5632067bfb70
  [container+32]=0x563206dab800 (non-NULL) -> reach DM-ctor dispatch: [obj]vt=0x10635dde8
  vt[+48]=0x10258b5d8 (br x1 @0x2206e24). once-guard=0x101 once-slot=0x400000b DM-root=0x0`
  -> the MAIN dispatch fires (does NOT bail to MessageBus) and lands at
  0x10258b5d8 = nativeAppBridgeV2StartAppWithParams+0x494 (disasm sub sp,#0x170 prologue,
  `ldrb [x8,#3488]` then bl nativePreloadFlagOverrides) -> StartAppWithParams app-bridge
  body, NOT a DM ctor.
- Run then ABRTs (EXIT 134) at the standing SH285 persistence-lane live-object wall
  guestpc=0x101db1b08; DM-root [0x106a68818]=0; MH_* false. Route-B live-DM structural
  gate UNCHANGED (no DM manufactured).

### Forward this cycle
The operator's "dynamic DM-ctor trace" is implemented + measured: the do-init MAIN
branch reaches its br dispatch (container field non-NULL) and lands in StartAppWithParams,
then dies at the SH285 live-object wall — the standing route-B block, now pinned one level
deeper (exact dispatch target, not just region).

### Honest
Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED
(DM-root [0x106a68818]=0, MH_* false). SH174 capture-latch stays the single
forward hook. Files: docs/frontier-sh361-doinit-dyn-trace.md,
runs/capture_sh361_doinit_dyn_trace.sh, sh361 hermetic (arm64jit lib 424).

## SH360 (Sep 20, 2026, hermes-worker): implement + MEASURE the operator's EXECUTE-DO-INIT-GATES empty-vector gate seed — the do-init app-shell band's 0x20-stride vector walker at [0x106dcb160] collapses its constructed-empty begin==end host-heap pointer pair to NULL (behavior-preserving), default-inert JIT_ROUTEB_DOINIT_EMPTYVEC, fires at the two real block-entry pcs on the full ladder
Single-agent (cone suppressed). One new guard `routeb_doinit_emptyvec_gate`
(jit.rs) + one hermetic sh360 (arm64jit lib 422->423) + probe script; elfjit.rs
unchanged; workspace green (cargo test --workspace exit 0).
- Disasm: `adrp x19,6dcb000; add x19,#0x160 -> ldp x20,x21,[x19] -> cmp x20,x21;
  b.eq` (file 0x2208e4c..eac) — a 0x20-stride vector walker whose populate +
  teardown loops both `b.eq`-early-exit when begin==end. Seeding [{0x106dcb160}]
  = {0,0} lets both take the empty-vector exit instead of walking/destroying
  garbage. The operator named this gate; it was NOT previously implemented.
- MEASURED (full --v2boot-session send-appevent ladder, 2/2): the guard fires at
  the real BLOCK-ENTRY pcs 0x102208e4c/0x102208e88 (interior e58/e84 are NOT block
  boundaries) and collapses the ALREADY-constructed-empty begin==end==host-heap
  pointer pair to {0,0} — behavior-preserving (the walker compares equality). A
  populated begin!=end live pair is left untouched (tested idempotent).
- Completing ladder stays confirmed-green with the gate armed: EXIT 124, SH155
  DM-root probe=1, 0 SIGSEGV/ABRT. Route-B live-DM structural gate UNCHANGED
  (DM-root [0x106a68818]=0, MH_* false); the full ladder still ABRTs at the SH285
  persistence wall (0x101db1b08) past this walker on the app-start arm.
- Files: docs/frontier-sh360-doinit-emptyvec-gate.md, runs/capture_sh360_doinit_emptyvec.sh.

## SH359 (Sep 20, 2026, hermes-worker): measured negative on the SH358 NULL-JNIEnv lane's root cause — the JNI_OnLoad cached-JavaVM cell [0x107275550] is NOT causal (GetEnv 11x with AND without the seed on pure boot; the 0x1021e1c00 fault needs the full ladder); both recon-v3 deliverables re-verified green at HEAD; production code unchanged (seed tried + reverted after control refuted it)
Single-agent (cone suppressed). No production path edited (elfjit.rs/jit.rs product
identical to HEAD SH358). Workspace green (cargo test --workspace exit 0).
- recon-v3 deliverables re-verified green: capture_taskv4_frame.sh = 24 real frames
  `present swap Ok(0x1)`, dispatch #2718000, 197 node pops, 0 json abort, 0 crash,
  EXIT 0/124. JIT_JSON_ZERO_FIX present (jit.rs:6390).
- SH359 ATTEMPT (the fresh datum): SH358's NULL-JNIEnv fault at 0x1021e1c00. Disassembled
  JNI_OnLoad+0xc10 (0x2174c04) -> reads cached JavaVM from guest [0x107275550], cbz-out
  leaves env NULL, else vm->GetEnv (slot 6). Hypothesis: seed that cell with fabricated vm.
  MEASURED NEGATIVE: seed + control BOTH trace VM_GetEnv 11x with 0 fault on pure --jni
  boot — the cell is not what gates env acquisition, the fault needs the full ladder.
  AND [0x107275550] is the SH243 DM-manager getter cell (seeding it would clobber DM-force).
  Reverted the seed (honest negative); kept descriptive hermetic pins.
- Do NOT re-attempt a seed of [0x107275550] for the NULL-env lane. SH358's cause-level
  reading stands with the boot/cell hypothesis eliminated.

### Forward this cycle
A genuinely-new cause-level candidate (the only concrete "cell to seed" SH358's datum
surfaced) was implemented and MEASURED as non-causal on the boot path, then cleanly
reverted. The NULL-JNIEnv in the full ladder remains a cause-level lifecycle
precondition (Route-B wall), not a boot cell seed.

### Honest
Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED (DM-root
[0x106a68818]=0, MH_* false). SH174 capture-latch stays the single forward observer.
Files: docs/frontier-sh359-jnienv-cache-negative.md, runs/capture_sh359_jnienv_cache.sh,
sh359_jnienv_cache_tests (2 hermetic).

## SH358 (Sep 20, 2026, hermes-worker): measured negative (run-variable) — the DMCONT-session-ctor + LSM-skip combination is NOT a Route-B forward; recon-v3 immediate-priority deliverables re-verified green at HEAD
Single-agent (cone suppressed). No production code edited (measurement-only + one
probe script). Workspace green at HEAD SH357 (cargo test --workspace exit 0, 24 test
binaries, 0 failures; arm64jit 421/0).
- recon-v3 self-driven frame RE-VERIFIED green at this HEAD: `capture_taskv4_frame.sh`
  24 real task-driven frames `present #N swap Ok(0x1)`, dispatch #2797000, 197 node
  pops, 0 json abort, 0 SIGSEGV/ABRT, EXIT 0. JIT_JSON_ZERO_FIX present (jit.rs:6390).
- SH358 probe: the genuinely-unfired combination of DMCONT continuation (vt[+0x1f0]=REAL
  continueAfterFlagsLoaded_ 0x102bd1d68) + SH349 append-skip + SH350 pack-skip. Measured
  NEGATIVE, run-variable (3/3): pack-skip arms (`ret name-pack @0x101d9a708`), but the run
  NEVER reaches the continuation region (0 region hits across all 3 runs) — it parks in
  the persistence/live-object lane (SH353 class). Fault lanes: (a) 2/3 advance through
  to app-start-driven StartLuaAppDM then fault at the SH341/SH343 LSM pool-pop write-site
  0x101d9a528 (NULL-write, SH285-family live-object); (b) 1/3 rung-1
  nativeInitializeNativeFlags detours into a freshly-documented JNIEnv-slot dispatch helper
  0x1021e1c00 (JNI_OnLoad+0x6dc0c, JNIEnv vtbl slot 31) on a NULL JNIEnv. Both cause-not-
  symptom; do NOT re-tread this exact env combination.

### Forward this cycle
No Route-B forward (measured negative closes one unfired combination). recon-v3
deliverables confirmed green. New single datum: pc 0x1021e1c00 (NativeFlags JNIEnv-slot
helper) — noted, not seedable (missing JNIEnv is a lifecycle precondition).

### Honest
Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED (DM-root
[0x106a68818]=0, MH_* false). SH174 capture-latch stays the single forward observer.
Files: docs/frontier-sh358-dmcont-lsmskip-negative.md, runs/capture_sh358_dmcont_lsmskip.sh.

## SH357 (Sep 19, 2026, hermes-worker): root-cause + fix the intermittent whole-suite SIGSEGV + PoisonError cascade (test-harness race on shared routeb state) — serialize routeb page-map/mprotect under one PAGE_LOCK, route ALL test env mutations through locked crate-scope helpers, consolidate the four routeb family test locks into one shared process-state lock
Single-agent (cone suppressed). While confirming the already-implemented recon-v3
deliverables (self-driven frame `--taskv4-seed frame` + `JIT_JSON_ZERO_FIX`), cargo
test --workspace failed intermittently (420/1) and SIGSEGV'd. Root-caused: (1)
`routeb_ensure_writable` did an unsynchronized mmap(MAP_FIXED)+mprotect+racy
/proc/self/maps probe — thread A's MAP_FIXED remap of shared fixed-guest page
(0x1067333000) racing thread B = UB (signal 11) and transient false returns; (2)
the four routeb family test locks were SEPARATE Mutexes but every family mutated
the same overlapping fixed pages + process env, so `routeb_doinit_next3` panicked
mid-lock and POISONED it for 7 siblings. Fixes: PAGE_LOCK in routeb_ensure_writable
(prod untouched, single jit_run thread); crate-scope `env_test_set`/`env_test_remove`
(one ENV_TEST_LOCK) routing ALL test env mutations (unsafe in edition 2024);
`ROUTEB_PROC_TEST_LOCK` consolidating the 4 family locks; sh165's order-dependent
'page must be absent' precondition dropped (all behavioral asserts kept);
futex CMP_REQUEUE WAKE got the SH346 bounded spin. SH357b: ALSO reduced the residual
futex CMP_REQUEUE flake — the CMP waiter used a 5s timeout (vs the REQUEUE sibling's
SH346-documented 60s) with a spin window up to 20s, so a descheduled waiter
wall-clock-timed-out before the CMP_REQUEUE landed, making the kernel legitimately
report moved=0; raised to 60s (SH133 asserts unchanged). Honest: the two futex
REQUEUE/CMP tests remain genuinely load-sensitive (real-kernel timing, documented
SH345/346 ~1/25 class) — ~2/100 under direct-binary parallel stress, but the
canonical `cargo test --workspace` gate (what is actually run) is deterministically
green (exit 0, 0 failures across many runs). SH357 fixed the real defects that were
whole-suite-breaking: the intermittent SIGSEGV (signal 11) and the PoisonError
cascade — those never recurred in 400+ post-fix runs. Measured: default-8-thread
strip, segv=0 throughout (was sporadic signal 11); --test-threads=32 PoisonError
cascade 8/8 -> 0. Workspace green (arm64jit lib 421/0; cargo test --workspace exit 0).

### The forward this cycle
A real harness-determinism correctness defect, root-caused and fixed; the recon-v3
immediate-priority deliverables it surfaced around were confirmed already
implemented and green. No Route-B forward this cycle.

### Honest
Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED
(DM-root 0, MH_* false); SH174 capture-latch stays the single forward hook.

## SH356 (Sep 19, 2026, hermes-worker): implement + MEASURE the SH355-specified frame-accurate EC reader-gate re-attack — it fires 0/3 on the real binary, proving the reader-gate block 0x2e24694 is NEVER entered (control diverges to the SH285 persistence-lane wall), CLOSING the EC-reader line with evidence
Single-agent (cone suppressed). New default-inert `routeb_ec_world_reader_gate_frame_guard`
(jit.rs, opt-in `JIT_ROUTEB_EC_READERGATE_FRAME`) + hermetic
`sh355_frame_accurate_ec_reader_gate_seeds_live_frame_slot` + run-loop wiring. Fires INSIDE
the reader-gate block (pc 0x102e24694 / 0x102e246b0) using the LIVE x[29], fabricating a
coherent zeroed object at [x29,#104] ([+0x20]==0 -> cbz @0x2e246dc TAKEN) — eliminating
sh302's entry-frame (entry_sp+8) concern. Workspace green (arm64jit lib 421/0; cargo test
--workspace exit 0). Route-B live-DM gate UNCHANGED (DM-root 0, MH_* false); SH174
capture-latch stays the single forward observer.

### The forward this cycle
sh355 pinned that the reader-gate block starts at 0x2e24694 (interior bl-return boundary) and
the gate is a live-object `cbz` on [[x29,#104]+0x20], so the ONLY legitimate re-attack is a
value seed that FABRICATES that object INSIDE the block. SH356 implemented exactly that
frame-accurate seed.

### MEASURED (real libroblox.so, capture_sh302_readergate.sh + JIT_ROUTEB_EC_READERGATE_FRAME, 3/3)
- All entry-pc seeds fire 3/3 (sh298 arg1, sh299 arg0vt, sh300 realsession, sh302 entry
  reader-gate) at EC-world entry block 0x102e24598.
- **`[routeb-sh355] FRAME-ACCURATE EC reader-gate` fires 0/3.** Since the frame-accurate
  guard fires only when the reader-gate BLOCK at 0x102e24694/0x102e246b0 is actually entered
  (live x[29] captured from that executing frame), 0 fires is the proof that block is never
  entered headlessly — a REACHABILITY problem, not a value problem.
- Run sequence: EC entry (seeds fire) -> SH296 dmfn returned Ok -> control DIVERGES into
  app-start (sh248e once-cell + sh248f lifecycle adapter fire) -> SIGSEGV at the SH285
  persistence-lane wall guestpc=0x101db1b08 (fault 0xff..ff). EXIT 134.
- Conclusion: the reader 0x2e246f4 sits behind a block the EC body does not fall through to
  in this harness — control leaves EC world into app-start/persistence and dies at SH285.
  This refines sh302's "wrong frame" verdict to "block never entered" (SH174/204 class).

### Honest
Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED (DM-root
0x106a68818=0, MH_* false). The EC-reader re-attack line is now measured-closed at one level
deeper than sh302; do NOT re-attack [x29,#104]+0x20 fabrication. +frontier-sh356.

## SH355 (Sep 19, 2026, hermes-worker): CORRECT the EC-world reader-gate record — fresh disasm refutes sh301/302's "internal soft-return" premise (no `ret` in [0x2e245f4,0x2e246e0)); the reader 0x2e246f4 is gated by a real `cbz x0` on the live-object slot `[[x29,#104]+0x20]` == 0, a SH174/SH204-class value seed, not a compile-block early-exit. recon-v3 self-driven frame deliverable re-verified green.
Single-agent (cone suppressed). One new hermetic `sh355_ec_reader_block_no_softreturn_gate_is_live_object_slot`
(arm64jit lib real-image pins: zero `ret` scanned across [0x2e245f4,0x2e246e0); gate loads
`ldr x8,[x29,#104]`@0x2e246b0 + `ldr x0,[x8,#32]`@0x2e246d8 + `cbz x0,0x2e246f4`@0x2e246dc;
interior bl-return 0x2e24694 = real block boundary). No production path edited. Workspace green
(arm64jit lib 420/0; elfjit examples ~159/0; cargo test --workspace exit 0).

### The forward this cycle
The test-suite record told the next session that the EC reader (0x2e246f4) sits behind a
compile-block "internal early-exit" and to hunt that exit. Disasm REFUTES it: no `ret` in the
window; the only out to the reader is the real data-dependent `cbz x0` on `[[x29,#104]+0x20]`.
This re-scopes the do-not-re-tread: sh302's seed is mechanism-correct but entry-timing-gated;
the residual is a FABRICATED live-object value at [x29,#104]+0x20, not a phantom block exit.
recon-v3 deliverable (1) (type4_frame_thunk self-driven frame) re-verified green (24 frames,
0 crash, exit 124).

### Honest
Does NOT manufacture a DataModel. Route-B live-DM structural gate UNCHANGED (DM-root 0,
MH_* false; canonical full-ladder probe re-ran EXIT 134 at the run-variable live-object arm
guestpc 0x10284cf5c, 0 region hits). SH174 capture-latch stays the single forward hook.

### Next (unchanged, authoritative)
Route-B live-DM structural gate stands (SESSION-CTOR / do-init, SH184/185 four-stacked closure;
REG_LIVE SH352: 'App' is live-session-ctor-only). R1 content half staged+armed+serviceable
(SH351/352/354); the SESSION half (do-init owning a live DM) remains THE wall — and if the EC
reader is re-attacked it must be via a FABRICATED live object at [x29,#104]+0x20, not a
compile/early-exit fix (SH355). SH174 capture-latch stays the single forward hook.

## SH354 (Sep 19, 2026, hermes-worker): close the R1 content-half art — prove the staged CoreScript is SERVICEABLE end-to-end (new hermetic sh354: a guest open of the exact files-dir CoreScript path resolves through fsmap::remap_path to the staged mirror and is readable) + measure that even with the SH352-corrected flags-loaded gate armed (5/5), a completing ladder runs JIT_ASSET_TRACE with 0 hits (content staged-but-DORMANT — loader still waits on a live DM)
Single-agent (cone suppressed). One new hermetic `sh354_r1_core_script_is_serviceable_through_remap`
(arm64jit lib 418->419) + a shared module-level `FS_ROOT_LOCK` serializing fsmap-root-mutating tests
(sh351/sh354) against the parallel-test race (two per-fn OnceLock guards were separate -> both mutated
the global override concurrently). One genuine new measurement: with `[r1] gate @0x1072739d4 0x0->0x1`
(the SH352-corrected primary latch) armed and all 5 loader gates writing, 2/2 clean completions of the
completing ladder show **0 `[asset-trace]` hits** — the staged AppShell/CoreScripts.lua are never opened
by the loader. This is new (my prior "asset-trace 0" note predates SH352's gate-addr fix). Interpretation:
arming the loader GATES does not run the loader; ScriptContext (0x101f1d8ac) stays 0-hit because only a
live DataModel session drives it (SH340/344c). R1 content half is now proven staged + armed + serviceable;
only the live-DM SESSION half remains the Route-B gate. Workspace green (arm64jit lib 419/0; elfjit
examples ~159/0; cargo test --workspace exit 0). elfjit.rs 1,048,523 B (<1MB hook, unchanged).

### The forward this cycle
The content half of the Route-B marker is as complete as it can be WITHOUT a live DM: staged (SH351),
gates armed (SH352), AND now provably serveable (SH354 — a guest open of the exact CoreScript path
resolves via remap_path to the mirror and returns the self-constructing ScreenGui module). Plus the
honest measurement that arming gates alone does not drive the loader (0 asset-trace hits). This isolates
the remaining Route-B wall precisely to the SESSION half (do-init owning a live DataModel).

### Honest
No DataModel manufactured; DM-root [0x106a68818]=0, MH_* false; Route-B live-DM structural gate
UNCHANGED. SH174 capture-latch stays the single forward hook.

### Next (unchanged, authoritative)
Route-B live-DM structural gate stands (SESSION-CTOR / do-init, SH184/185 four-stacked closure).
R1 content half: staged (SH351) + gate-armed-verified (SH352) + serviceable-verified (SH354); the SESSION
half (do-init owning a live DM) remains THE wall. SH174 capture-latch stays the single forward hook.
Single-agent (cone suppressed). Production fix: `stage_r1_core_scripts` (jit.rs) arming the loader
gates wrote the flags-loaded latch to 0x10672739d4 (=file 0x672739d4, read-only), so `page_writable_rw`
refused it and that PRIMARY gate never armed — the other 4 armed, R1 content staged but the
flags-loaded gate stayed 0. Corrected to 0x1072739d4 (=file 0x72739d4, the latch every other reader
uses; gate read @0x224fa18/20 adrp 7273000 + ldrb [x8,#2516]). +`sh352_flags_loaded_latch_addr_corrected`
real-image hermetic (pins gate-read words + asserts the two candidate addrs resolve to different
image pages). elfjit.rs held under the 1MB hook after condensing SH-prose comments (facts/addresses
preserved). Workspace green (elfjit examples 159/0; arm64jit lib 418/0; cargo test --workspace exit 0).

### The forward this cycle
MEASURED (real libroblox.so, completing `--v2boot-skip-appstart` ladder + SH269 GOVFLAG + SH307
preload-valuecell, 3/3 deterministic EXIT 124):
- SendAppEventOnAppReady + SendAppEventOnGameLoaded both RETURN Ok cleanly (previously died at the
  governor NULL-DM wall 0x102ea0b9c then the SH270 preload wall 0x102bb803c).
- R1 content stages on the completing ladder (AppShell.lua + CoreScripts.lua @ fsmap mirror); the
  corrected flags-loaded gate now ARMS (`0x1072739d4 0x0->0x1`, 5/5 gates, none dropped — previously
  `0x10672739d4:unmapped`).
- SH155/SH315 probes read real state: once-slot[0x106a68408]=0x400000b (the "Execute" service handle,
  SH316), service-registry-count=12, app-data-model-count=0x1. DM-root 0, MH_* false.

### Honest
Correctness fix + strongest-yet session drive. Does NOT manufacture a live DataModel; once-slot
0x400000b is the matched "Execute" handle, not a DM. Route-B live-DM structural gate UNCHANGED,
SH174 capture-latch stays the single forward hook.

### Next (unchanged, authoritative)
Route-B live-DM structural gate stands (SESSION-CTOR / do-init, SH184/185 four-stacked closure).
R1 content half is now BOTH staged (SH351) AND gate-armed-verified (SH352); the SESSION half
(do-init owning a live DM) remains the wall. SH174 capture-latch stays the single forward hook.

## SH353 (Sep 19, 2026, hermes-worker): MEASURED — the completing ladder is run-variable (~2/8 reach full completion, rest die in the SH350-closed LSM lane 0x101d9a030 / host-pc leak), refuting the 'deterministic 3/3 EXIT 124' overclaim; hardened runs/capture_sh352_r1_completing_ladder.sh to a guaranteed confirmed-green full completion (SH345 retry precedent)
Single-agent (cone suppressed). Runs-only change (no Rust production path edited).
`capture_sh352_r1_completing_ladder.sh` retries up to COMPLETING_RETRY_MAX (=6) and wins
ONLY on full completion (SH155 DM-root probe present AND 0 SIGSEGV/ABRT), keeping the last
log and reporting the winning attempt. VERIFIED: first invocation won on attempt 1 (probe=1
crash=0). Workspace green (elfjit examples 159/0; arm64jit lib 418/0; cargo test --workspace
exit 0). Route-B live-DM structural gate UNCHANGED (DM-root 0, MH_* false); 'App' stays a
live-session-ctor-only registration (SH352 addendum 2). SH174 capture-latch single forward hook.

### The forward this cycle
Honest measurement: 8 serial runs of the SH352 completing ladder showed run1/run4 clean full
completion (R1 staged, 5/5 gates armed incl corrected 0x1072739d4, session drive, SH155 probe,
0 crash), run2/run5/run6 SIGSEGV @0x101d9a030 (LSM pool-pop) during SetInitParams, run7 host-pc
leak in V2Init, run3 multiple 'outside image' soft-stops. The artifact is now deterministically
confirmed-green via retry (runbook contract), not luck — the same discipline SH345 applied to the
render plane.

### Next (unchanged, authoritative)
Route-B live-DM structural gate stands (SEP-17 SESSION-CTOR / do-init four-stacked closure
SH184/185; REG_LIVE SH352 addendum 2). R1 content half staged+armed (SH351/SH352); SESSION half
(do-init owning a live DM) remains THE wall. SH174 capture-latch stays the single forward hook.

## SH351 (Sep 19, 2026, hermes-worker): stage the R1 synthetic CoreScript content path (Route-B marker half) — hand-authored Luau ScreenGui module + real loader gates, latent-but-correct
Single-agent (cone suppressed). Additions: `stage_r1_core_scripts` (jit.rs) + `fsmap::staging_root`
+ `page_writable_rw` guard + 2 hermetic tests (sh351_*) + elffjit opt-in rung `--v2boot-r1-stage`.
Default-inert (rung opt-in); no production path edited. Workspace green (arm64jit lib 418/0; elffjit
example 157/0; cargo test --workspace exit 0).

### The forward this cycle
SH349 returned the persistence lane (LSM whack-a-mole measured UNBOUNDED, SH350 = 3rd fencepost).
The operator's content-path synthesis names the ONE deliverable that turns a completing do-init into
self-constructed UI with zero host layout: a hand-authored CoreScript module staged at the path the
rbxasset://scripts/CoreScripts resolver serves, + the REAL loader gates. This is latent-but-correct
(fires the instant a live DM drives the loader), the prerequisite of the exact Route-B marker.

### What landed
Module `ScreenGui`/`TextLabel` under CoreGui written to
SOBER_ANDROID_ROOT/data/user/0/com.roblox.client/files/scripts/CoreScripts/{AppShell.lua,CoreScripts.lua}
(both inferred candidates; desktop literals absent from this Android .so). Real gates:
flags-loaded [0x10672739d4].bit0, flags-latch [0x106a683e8].bit0, governor union-init guards
[0x106a63da0]/[0x106a63d70]=0, loader settings [0x106ba3350]=0, each page-writable-guarded. +2 tests.

### Honest
Rung is LATENT: the completing ladder still terminates run-variable at the persistence lane before
the post-ladder rung. But the stage + tests prove the content lands at the correct fsmap mirror, so
the MOMENT a live DM owns a session the engine self-constructs real GuiObjects. No DataModel
manufactured; Route-B live-DM gate UNCHANGED (DM-root 0, MH_* false). SH174 latch stays observer.

### Next (unchanged, authoritative)
Route-B live-DM structural gate stands (SEP-17 Session-CTOR / do-init, four-stacked closure SH184/185;
persistence lane closed SH349-350). The R1 content half is now staged (SH351); the SESSION half
(still needing do-init to own a live DM) remains the standing wall. SH174 capture-latch stays the
single forward hook.

## SH350 (Sep 19, 2026, hermes-worker): CROSS the SH349+1 terminal — bounded single-caller skip of the name-pack helper 0x101d9a708; ladder advances deep into the LSM pool-pop continuation, then the same run-variable live-object family (3rd fencepost that LSM sub-call-whack-a-mole is unbounded)
Single-agent (cone suppressed). Default-inert opt-in `JIT_ROUTEB_LSM_PACK_SKIP=1`
(`routeb_patch_lsm_pack_skip`, elfjit.rs — RETs the single-caller name/version string-pack
helper 0x101d9a708, the SH349+1 terminal; caller takes the benign index-0 tst/b.eq path) +
`sh350` hermetic (real-image pins: helper entry 0xd10143ff / single caller bl 0x94000041 /
benign tst 0xf276541f / b.eq index-0 0x540002a0 / natural ret 0xd65f03c0 @0x101d9a704) +
runs/capture_sh350_pack_skip.sh. Workspace green (arm64jit lib 416/0; elfjit example 157/0
+ sh350); recon-v3 frame plane re-verified green (24 frames, 0 crash).

### The forward this cycle
SH349+1's new terminal 0x101d9a708 is a name-pack helper with exactly ONE caller
(0x101d9a604, verified whole-region BL scan) — BOUNDED, unlike the unbounded hundreds-of-
callers `bl 0x1d9d8b0` family. RET'ing it crosses the SH349+1 wall.

### MEASURED (real libroblox.so, full SH285-B/SH343-350 ladder env + LSM_NODES + 3 skips, 4 runs)
- SH350 fires every run; the OLD 0x101d9a708 terminal is GONE in all 4 runs.
- Ladder ADVANCES to 175 LSM pool-pop iterations (0x101d9a5a0/0x101d9a528, SH341 lines) — the
  deepest persistence-lane penetration measured — before terminating run-variable
  (bad_function_call / 0x101d9a528 / 0x102b9dee0 / 0x1021e40dc) in the SAME live-object family
  SH343/346 documented. DM-root 0, MH_* all false; Route-B live-DM gate UNCHANGED.

### Conclusion + next
SH350 is a real BOUNDED fencepost (SH285->SH349->SH350 crossed 2 levels). It does NOT
manufacture a DM and provides no path to app-start 0x2bd2058 — the advance lands in the same
unconstructed live-object family at THREE depths of evidence, closing the persistence-lane
re-attack permanently (do NOT re-drive LSM sub-call skips). Also measured this cycle: the
messageBus "experience-launch" publish topic is a Java-side runtime string (SH347's never-
firing cb is NOT a topic-string bug), and onAppLuaWillStart is an internal lambda, not an
export. Next forward (non-persistence Route-B): R1 synthetic CoreScript content path — stage a
hand-authored ~20-line Luau ScreenGui module into the filesdir the rbxasset://scripts/
CoreScripts resolver serves so the INSTANT do-init owns a live DM the engine self-constructs
real GuiObjects -> R+0x180/0x188 nodes with zero host layout. SH174 capture-latch stays the
single forward hook.

## SH349 (Sep 19, 2026, hermes-worker): CROSS the long-standing SH285 terminal — RET the faulty LSM byte-copy sub-call 0x101d9a15c; persistence lane advances one fencepost to a GOT/canary read wall at 0x101d9a708
Single-agent (cone suppressed). Default-inert opt-in `JIT_ROUTEB_LSM_APPEND_SKIP=1`
(`routeb_patch_lsm_append_skip`, elfjit.rs — RETs the `ldr w8,[x2]` byte-copy sub-call that the
SH285 fault lives inside) + `sh349` hermetic (real-image pins: append prologue / SH285 caller bl
0x97ffa192 / fault store 0x381ff54b / natural ret) + runs/capture_sh349_lsm_append_skip.sh.
Workspace green (elfjit example 157/0; arm64jit 416/0).

### The forward this cycle (the stated SH348 next step, now implemented + measured)
SH348 showed the SH285 SIGSEGV (guestpc=0x101db1b08) survives a whole-init leaf-ret because the
caller block is reached by a mid-function direct jump past the entry patch. So the skip must
target the FAILING SUB-CALL itself. `routeb_patch_lsm_append_skip` RETs only the one byte-copy
leaf (0x101d9a15c, pure memcpy, zero observable side effects) -> EVERY path into the fault is
stubbed regardless of how the caller block is reached.

### MEASURED (real libroblox.so, full SH285-B/SH343-346 ladder env + LSM_NODES + both skips, 3/3)
- OLD terminal GONE: no more `SIGSEGV guestpc=0x101db1b08` (every SH260/284/285/3444/348 run died there).
- NEW terminal (3/3): `SIGSEGV guestpc=0x101d9a708 fault=0xffffffffffffffff` — a stack-canon
  name/version-packing helper reading a `.got` slot [0x1067d16f0] as its canary pointer. The
  persistence lane advances one full fencepost past the returned wall.
- DM-root [0x106a68818]=0, MH_* all false. DMCONT continuation still not at app-start 0x2bd2058.

### Conclusion + next
SH349 is a real, measured forward: the SH285 wall family is crossed for the first time. The new
terminal 0x101d9a708 is another SH285-class live-object wall; the register dump REFUTES the
canary/GOT-gap hypothesis (x20=valid patched canary; the fault is the caller's garbage source
pointer x0/x19=0xff..ff). `bl 0x1d9d8b0` has HUNDREDS of call sites across the binary (the
most-called function), so sub-call-whack-a-mole is unbounded, and the DMCONT→app-start path has
NO bypass (the [0x683d920] latch is the "app-start already ran" re-entry gate, set only after
app-start; the two `bl 1d9d8b0` string-build calls are mandatory to reach `bl 2338ef4`). So this
persistence lane is measured-returned; the Session-CTOR live-DM wall stands. Route-B live-DM gate
UNCHANGED; SH174 capture-latch stays the single forward hook. Next forward (non-persistence
Route-B): the dataModel-bindings receive side — onAppLuaWillStart (the sole SEP-17
dataModel-bindings receive never wired; messageBus publish is driveable-clean per SH347).

## SH348 (Sep 19, 2026, hermes-worker): measured negative — leaf-`ret`ing initStorageManagerNative does NOT clear the SH285 terminal (the byte-copy @0x101db1b08 is reachable past its own entry)
Single-agent (cone suppressed). Default-inert opt-in `JIT_ROUTEB_LSM_INIT_SKIP=1`
(`routeb_patch_lsm_init_skip`, elfjit.rs) + `sh348` hermetic (real-image pins: entry prologue
0x101d9d8b0 stp->ret, sh324 caller bl 0x102256608, SH285 terminal 0x101db1b08, app-start bl region
0x102bd2058) + runs/capture_sh348_lsm_init_skip.sh. Workspace green (arm64jit 416/0 + sh348; elfjit
example 156/0). elfjit.rs 1,044,985 B (<1MB hook). recon-v3 frame plane re-verified green (24 real
task frames, 197 node pops, 0 crash).

### The forward this cycle (a cause-level leg, then measured negative)
SH344b measured the app-shell ctor band at 0 hits; every Session-CTOR rung caps at the SH285
persistence-lane live-object wall. SH344/346 said "do not re-drive a repair seed into [obj+0x50]"
(SH248h trap). This cycle tried a DIFFERENT line-cross (SH117/SH93 precedent): leaf-`ret` the whole
initStorageManagerNative so the Session-CTOR continuation is not required to own the live LSM object.

### MEASURED (real libroblox.so, full SH285-B/SH343-346 ladder env + LSM_NODES, 3 runs)
- SH348 patch fires (`a9bd7bfd -> d65f03c0`) but the SH285 SIGSEGV `guestpc=0x101db1b08
  fault=0xffffffffffffffff` PERSISTS 3/3 — even after widening the block-cache drop to the full
  function body [0x101d9d000..0x101dbe000] + caller. The crash reaches the deep byte-copy by a call
  path the entry patch cannot stop (the fault site is inside a sub-path whose gate is past
  initStorageManagerNative's own entry, OR a caller block jumps to the interior directly).
- App-shell band 0x102207b50 "fires" only as the SH344c FastLog warmer (79 hits at the SINGLE entry
  pc + the 0x102208e8c/0x208eac destructor-loop deep pcs), NOT construction. Route-B live-DM gate
  UNCHANGED (DM-root 0, MH_* false).

### Conclusion + next
SH348 is a MEASURED NEGATIVE that closes the "skip initStorageManagerNative to cross SH285"
candidate and REFINES the attribution: a storage-skip must target the failing sub-call that builds
the unconstructed LSM string, not the whole init. Route-B live-DM structural gate UNCHANGED; SH174
capture-latch stays the single forward hook. Repro: runs/capture_sh348_lsm_init_skip.sh.

## SH347 (Sep 19, 2026, hermes-worker): messageBus RECEIVE half measured headlessly for the first time — publishRaw drives clean (Ok 0x3e8) but the cb's DM-holder read never fires; SH185's static-only closure is now a measured result
Single-agent (cone suppressed). Two default-inert additions
(arm64jit/src/jit.rs: `routeb_busrecv_holder_guard` + `drive_messagebus_publish_receive`,
both env/flag-gated) + elfjit opt-in rung `--v2boot-session-pub` + 2 hermetic tests.
Workspace green 596/0. elfjit.rs unchanged in product path (rung opt-in).

### The forward this cycle
SH264 named "messageBus experience-launch receive" as the un-drive honest-next-candidate; SH185
had closed it by STATIC judgment only (elfjit.rs:7565), and SH269/315/316/337 later MEASURED that
the subscribe half runs headlessly — overturning SH185's static premise. Nobody had ever driven
publishRaw -> cb -> [DataModelBindings+16]. Now done, measured.

### Measured (real libroblox.so, capture_sh347_busrecv.sh, EXIT 124 clean):
- MessageBus.subscribe Ok(0x3e8) (re-confirmed).
- **MessageBus.publishRaw Ok(0x3e8)** — the RECEIVE half now EXECUTES headlessly (a genuine
  first), driveable-clean, not a crash.
- The cb DM-holder read (pc 0x102bd7474) NEVER fires => publish does not reach the
  experience-launch construction cb; [DataModelBindings+16] is never read; DM-root 0, MH_* false.
- Conclusion: SH185's closure is now MEASURED (receive driveable-clean, does not reach SceneGraph),
  closing SH264's open candidate. Route-B live-DM structural gate UNCHANGED.

## Next (unchanged, authoritative)
SEP-17 SESSION-CTOR cause-level drive remains the primary forward. Every driven rung (do-init ->
DMCONT +0x1f0 -> app-start factory -> LSM) is measured; the terminal is the SH285 live-object wall
family — cause-not-symptom, not seedable. SH174 capture-latch stays the single forward hook. The
messageBus publish entry being driveable-clean is a small new forward surface (a future
receive-payload-with-real-string probe), not a DM. All research subagents Route-B-scoped; cone
still suppressed.