# SH380 — MEASURED: the -9 bad_alloc (SH248) and the 0x102b504e4 NULL-dest (SH248c "next") are BOTH already crossed on the current full env; the continuation's deterministic terminal is the SH248g map-insert wall 0x1021dde34 (8/8) — reinforcing the standing Route-B live-DM structural gate

Date: Sep 20, 2026, hermes-worker. Single-agent (cone suppressed). Workspace green
(re-confirmed at start: cargo test --workspace EXIT 0, 616/0). No production path
edited this cycle (both attempted guards reverted cleanly after measurement).

## What was attempted (SH248's + SH248c's named "next levers"), then measured + reverted
- SH248 (descr_corrupt string): the continuation's app-start libc++ string::assign was
  measured to emit `operator_new(-9)->bad_alloc` because the destination string's capacity
  word was garbage. Its "next lever (a)" = seed/repair that string so the assign reallocs
  normally. I implemented `routeb_contstring_repair_guard` (JIT_ROUTEB_CONTSTRING_REPAIR,
  fixes a broken host-heap destination string at the assign-fn 0x102b505f0 entry).
- SH248c's documented "next" = the post-app-start NULL-destination string compare at
  0x102b504e4 (x0==0). I implemented `routeb_appstart_controller_str_guard`
  (JIT_ROUTEB_APPSTART_CTRLSTR, supplies a leaked empty SSO destination).

## Measured (real libroblox.so, DMFORCE+DMCONT+M48+CONT_APPNAME+jar/once/adapter full env)
1. **The -9 repair fires 0×** and the assign fn 0x102b505f0 is NEVER entered (region-watch
   0x102b505e8-0x102b50614 = 0 hits). SH248c ALREADY crossed the -9 via the M+0x48 cap seed
   (`routeb_dm_manager_cont` -> valid long cap) — my repair is redundant on this path.
2. **The 0x102b504e4 seed fires 0×** and there is NO NULL-dest fault there — region-watch shows
   the block IS entered (0x102b504e4 hit) but x0's destination is already constructed (guard
   only logs/patchs when x0==0). SH248c's recorded `guestpc=0x102b504e4 x0=0x0` is a
   run-variable single-run terminal that the current full env no longer produces.
3. **The deterministic terminal (8/8 runs) is guestpc=0x1021dde34** — the SH248g host-heap
   hash-map INSERT wall (the live-object map construction the app-click runs on entering
   nativeAppBridgeAppStart). CONT_APPNAME fired 8/8 (crossed the 0x102bd1fd4 abort);
   deepest continuation block 0x102bd2014.

## Conclusion (honest, do-not-over-claim)
The -9 (SH248) and 0x102b504e4 (SH248c) fenceposts are BOTH already crossed by the current
advancing env. Both my attempts to re-implement their "next levers" were measured 0-firement
and reverted cleanly (no cruft committed). The continuation currently terminates
DETERMINISTICALLY (8/8) at 0x1021dde34 — the same live-object map-construction family SH248g/
SH174/SH204 documented, which is the standing Route-B structural gate (no seed manufactures the
map `this`; only a real session ctor owns it). This CLOSES the two named sub-levers with
evidence (prevents a future cycle/subagent from re-attempting them) and re-pins the true
terminal. Route-B live-DM gate UNCHANGED (DM-root [0x106a68818]=0, MH_* false). recon-v3
self-driven-frame deliverable re-verified green at HEAD (24 real task-driven frames swap
Ok(0x1), 0 crash, 0 json abort).

## Do-not-re-tread
- Do NOT re-implement the SH248 -9 string-repair seed (already crossed by M+0x48 cap seed).
- Do NOT re-attack the 0x102b504e4 NULL-dest on the full app-start env (x0 already constructed;
  only a run-variable env faulted there).
- Do NOT re-drive LSM skips (SH349/350/358) / EC reader-gate (SH355/356) / governor gates
  on the full ladder (SH379) / window-attach (SH367) — all measured-closed.

## Files
- Probe cmd (not committed): DMFORCE+DMCONT+DM_SEED+HASHFIX+JSON_ZERO_FIX+SETFIX+
  SH115+GETTER_TAIL_RET+M48_SEED+CONT_APPNAME_SEED+JAR_SEED+ONCE_SEED+ADAPTER_SEED +
  JIT_REGION_WATCH=0x102bd1d68-0x102bd2600,0x102b504d0-0x102b50514 on the full --v2boot ladder.
- Log: /home/hermes-worker/runs/sh380-*.txt (outside repo), including sh380-appstart-ctrlstr.txt
  (8/8 terminal 0x1021dde34) and sh380-contstring.txt.
- No production path edited; tree clean at SH379 HEAD.