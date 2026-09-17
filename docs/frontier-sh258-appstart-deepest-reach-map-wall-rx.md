# Frontier SH258 — App-start reaches its deepest point (0x102339d44), map wall proven W-off at that depth

Status: MEASURED (fresh, THIS HEAD, full combined seed set). Route-B live-DM structural gate UNCHANGED.
SH174 capture-latch (arm *(0x106391908) at a real make_shared<DataModel>) stays the single forward hook.

## What was measured

Fresh run at this HEAD (`runs/capture_sh258_postgovtail.sh`) with the FULL combined SH248c-f seed
set — jar/once/adapter/appname + SH245 getter-tail + M48 + SETFIX + DMCONT + the three
`JIT_ROUTEB_APPSART_*` seeds all ON simultaneously (a state no prior session ran together; the
SH257 run used jar/once/adapter but recorded only the FMOD-tail/do-init chain, not the app-start depth).

Result — the do-init -> governor -> app-start continuation now reaches **0x102339d44**, inside the
deep app-start orchestrator fn 0x2339d0c at its `bl 21dac2c` (a once-guarded settings/registry
singleton factory: reads flags byte [0x106a6f430], __call_once 0x284ce54, builds a registry object).
Region hits this run: `0x102339004/00c/018/050/07c/1f8/208/c3c/d0c/d44`, then EXIT 134 at the SAME
standing live-object map wall 0x1021dde34.

- **Deepest app-start reach ever recorded.** SH251 recorded 0x10233907c; the extra gate-climb to
  0x102339d44 is the SH248d/e/f jar/once/adapter seeds doing their job.
- **The wall is identical and now provably W-off at that depth.** Register dump at the SIGSEGV:
  `x20=x21=0x100548ca9` (the map-`this`), `x19=0x40c29c7e746e86a1` (per-insert garbage hash),
  `x22=0x11` (stride-0x2a0 live-array index), `x23=0x2`. Guest 0x100548ca9 = file 0x548ca9, inside
  the single R-E (R-X, write=OFF) exec LOAD segment [file 0x0, 0x62d8190). So the map-`this` is a
  pointer into execute-only code memory — no static seed (SH248g), runtime repair (SH248h),
  count-clamp (SH250), dynamic-ctor trace (SH251), or any other write lever in this JIT can reach it
  (SH249's segment-protection proof, now re-confirmed at the deepest reach).
- **Live-array allocator clusters unchanged.** The three stride-0x2a0 live-object array allocators
  {0x1df48c0, 0x1eb9af4, 0x1eba550} remain at **0 region hits** — that array is never constructed
  headlessly (SH254). The map-insert walks an under-allocated array because its owner was never
  built, not because any seedable gate is in the way.

## Note (fn 0x21dac2c at the deepest reach)

The immediate callee at the deepest pc (0x2339d44 -> bl 0x21dac2c) is a once-guarded singleton
factory that builds the app's settings/registry object (flags byte [0x106a6f430] primary, __call_once
284ce54, populates a 64-byte registry structure via 21dad40/21e126c/21e1470/21e1668/21e1830/21e1a34/
2e88f5c). It is NOT a forward DM ctor — it builds a registry object that the subsequent map-insert
registers into, and that map is the W-off-then-unbuilt one. Seeding its once-guard would only make
the same registry object; it does not allocate the stride-0x2a0 live array, so it is not a crossable
gate on its own.

## Pinned contract (hermetic sh258, real-image guard, skip-if-absent)

- deep orchestrator entry 0x102339d0c = sub sp,#0x160  (0xd10583ff)
- deepest reach bl          0x102339d44 = bl 0x21dac2c (0x97fa83ba)
- wall map-`this` 0x100548ca9 resolves into the image (host_addr_of != 0)
- 4-alignment of both pcs

## verify

`cargo test -p arm64jit --example elfjit sh258` = 1 passed on the real libroblox.so.
Repro: `runs/capture_sh258_postgovtail.sh` (log runs/sh258-postgovtail.txt).

## Honest (do-not-over-claim)

Fresh authoritative measurement + regression pin of the deepest app-start reach. Does NOT manufacture
a DataModel; Route-B live-DM structural gate UNCHANGED. With the full seed set the engine self-climbs
to the deepest app-start point and the wall is the same live-object graph needing a real upstream
session ctor (SH174 capture-latch = the single forward hook). No production code path edited, no
feature-flag cruft, default-inert, workspace green.