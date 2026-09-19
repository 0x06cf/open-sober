# Frontier SH407 + SH408 — app-start MAIN body runs END-TO-END; LSM init crosses the SH285 reader; persistence lane stays the whack-a-mole terminal

Date: 2026-09-21, hermes-worker, single-agent (cone suppressed). Workspace green
before and after (cargo test --workspace EXIT 0; arm64jit lib 456 with the new sh407
hermetic). Two probes (runs/capture_sh407_appstart_main_terminal.sh +
runs/capture_sh408_filesdir_lsm.sh), logs outside repo
(/home/hermes-worker/runs/sh407-appstart-main-terminal.txt,
/home/hermes-worker/runs/sh408-filesdir-lsm.txt).

## Why this cycle

SH406 left the do-init MAIN arm draining into the SH341 pool-pop persistence loop,
its true terminal ambiguous (log ended mid-pool-pop, no crash line). The named open
question: does that world-build ADVANCE past the persistence drain into new
construction regions (DMCONT / app-shell ctor / governor / scriptctx), or PARK on
the same pcs? SH407 and SH408 answer it decisively with region-watch over the whole
app-start body + the LSM init band.

## SH407 — the app-start MAIN body runs END-TO-END (biggest app-start reach on record)

Real libroblox.so, SH406 env (JIT_ROUTEB_DONEPATH_MAIN=1 + SETFIX=1 +
GOVFLAG=1) + region-watch over [0x10258b5d8,0x102590000) ∪ DMCONT ∪ app-shell
band ∪ governor ∪ scriptctx ∪ pool-pop band.

- **Every block-entry pc of the app-start dispatch body fires: 0x10258b5d8 … 0x10258bbb0
  (14 distinct pcs, each 1×), and NOTHING past 0x10258bbb0.** The body runs through its
  terminal block 0x10258bbb0 — SH362/404/405's "unreachable" body is now fully
  executable headlessly, end-to-end. 0x10258c000 (next fn) does not enter.
- The do-init MAIN arm then drains into the SH341 pool-pop persistence lane: **190
  valid-key pool-pops, caller LR=0x10626b6dc, SETFIX fired (substituting the leaked
  container)**, and the run ends EXIT 139 (host-side SIGSEGV; no guestpc fault line —
  the log just stops on the pool-pops).
- **DMCONT [0x102bd1d68,0x102bd2600) = 0 region hits** on the MAIN arm (the "16"
  count is the region-watch spec string, not hits). The MAIN-arm continuation does
  NOT advance to the do-init-completion construction gate.

## SH408 — files-dir / R1 rungs added; the LSM init ADVANCES past the SH285 reader

Same env + `--v2boot-set-filesdir` + `--v2boot-r1-stage` (the real host surface: the
app files dir [0x10726d600], which the SH406/407 env NEVER seeds). Region-watch over
the real LSM ctor 0x1db0dfc, initStorageManager 0x1db1050, the SH285 reader
0x1d99e30 + its 0x101db1b08 wall, the append 0x1d9a15c, pool-pop 0x1d9a5a0.

- **initStorageManagerNative 0x101db1050 is ENTERED** (pcs 0x101db1050, 0x1084,
  0x10b0), the reader 0x1d99e30 band entered (0x101d99e6c..0x101d99fe8), the append
  0x1d9a15c band entered (0x101d9a18c..0x1a1f4), the operator-new band entered
  (0x101db1a38..0x101db1c60), and insert-leaf pcs 0x101db1d04/0x1d8c fire.
- **SH285 reader wall pc 0x101db1b08 = 0 hits** — the SH285 fault terminal from
  SH260/284/285/344/348/371/372 is CROSSED this run (0 hits = optioned past).
- BUT the seed rungs did NOT fire (no `--v2boot-setfilesdir`/`[r1]` line in the log —
  set-filesdir is eprintln, so unreached). So the deeper LSM reach is run-reach, not
  the files-dir seed's doing (the seed simply never ran on this env).
- The run still ends EXIT 139 with **fault=0x0** — a NULL deref in the just-entered
  LSM insert/opnew band. This is the SH349/350/358/396-measured unconstructed-object
  whack-a-mole family: crossing the SH285 reader just lands one fencepost deeper in
  the SAME path-dependent persistence lane (SH385: node value set only by a real LSM
  session ctor). Not a DM advance.

## Interpretation (honest: fenceposts, not a DM)

- **SH407** locks the biggest forward reach on record: the app-start MAIN body is now
  fully executable end-to-end. The MAIN-arm continuation drains to the LSM pool-pop
  persistence lane and never reaches DMCONT. This closes the SH405 "Next" question
  (same lane, not do-init-completion) with the body fully cleared.
- **SH408** deepens the persistence-lane closure one more level: even when the run
  reaches INTO initStorageManager + the SH285 reader (reader crossed, 0x101db1b08 = 0
  hits), it next-faults at NULL in the operator-new/insert-leaf band — the standing
  measured-returned unconstructed-object whack-a-mole. Consistent with SH385/396:
  only a real LocalStorageManager session ctor constructs the value the reader needs.
- Does NOT manufacture a DataModel. DM-root [0x106a68818]=0, once-slot 0x400000b
  sentinel, MH_* false, DMCONT unreached from the MAIN arm. Route-B live-DM gate
  UNCHANGED.

## Files

- crates/arm64jit/src/jit.rs: +hermetic `sh407_appstart_body_full_span_runs_end_to_end_and_lsm_init_crosses_reader`
  (arm64jit lib 455->456) real-image gated, byte-pins the full app-start body span
  (0x10258b5d8 prologue .. 0x10258bbb0 terminal, 0x10258c000 next-fn) AND the LSM-init
  deepen (ctor 0x101db0dfc, initStorage 0x101db1050/0x1084, opnew 0x101db1a38, reader
  footer 0x101db1b08 CROSSED, insert 0x101db1d04, reader 0x1d99e30, append 0x1d9a15c,
  pool-pop 0x1d9a5a0).
- runs/capture_sh407_appstart_main_terminal.sh + runs/capture_sh408_filesdir_lsm.sh
  (probes). Logs outside repo.

## Next

The app-start MAIN body is fully cleared; DMCONT stays 0-hits from the MAIN arm and
is reachable only via the settings-state/send-appevent env (SH371) where it also
terminates at the LSM lane (SH372). The persistence lane is measured-returned at two
successive fenceposts (reader crossed, opnew/insert fault). Per SH349/350/396, do NOT
re-drive LSM skips or re-manufacture the map. The one genuinely-open surface is the
REAL session-compat runtime (SH400 substrate + a real LSM/EGL/onAppReady host drive),
which is the standing SESSION-CTOR deliverable.