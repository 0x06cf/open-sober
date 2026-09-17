# SH240 — DMCONT continuation re-measured FRESH at HEAD (block-entry-definitive negative) + dispatch chain pinned

Session: Sep 17, 2026 (hermes-worker). Real libroblox.so (host
`~/.cache/open-sober/robbox/libroblox.so`, 109,193,800 B). +1 hermetic
(`sh240_dmcont_dispatch_chain_pinned_fresh_negative`, real-image guard family as
sh239/238/237, skip-if-absent). Workspace green (examples 85/0). Single-agent
(cone suppressed). Repro `runs/capture_sh240b_dmcont_flow.sh`.

## Why

The operator's standing Route-B line explicitly names "keep grinding the DMCONT
continuation". SH228 (commit 4b6700b, an OLDER HEAD) measured the engine-init
dispatcher 0x102bd8ce8's `bl sub_2bd8dac` (-> vt+0x1f0 = continueAfterFlagsLoaded_)
as unreached headlessly. Since 4b6700b, ~7 commits (SH229/233/234/235/237/238/239)
landed, including real-image-guard infrastructure and new SH161b/SH217 state.
This cycle re-measures the DMCONT continuation at the NEWEST HEAD under the exact
working DMCONT env (JIT_ROUTEB_DMFORCE=1 + JIT_ROUTEB_DMCONT=1, manager holder
0x102727550 seeded so the getter materializes a manager whose vt[+0x1f0] is the
REAL continueAfterFlagsLoaded_ 0x102bd1d68).

## Measured (real binary, canonical --v2boot completing ladder)

- **3/3 clean (EXIT 124, 0 SIGSEGV/SIGABRT/stack-smash), ladder done, SendAppEvent
  returned Ok(0x3e8).** Manager routing fired every run:
  `SH165 manager singleton holder 0x102727550 -> continuation-routed manager
  (vt[+0x1f0]=REAL continueAfterFlagsLoaded_ (0x102bd1d68)) at pc=0x102bd1b98` —
  fnB is entered and the seeded manager is materialized.
- **JIT_REGION_WATCH (narrow, block-entry) on the interior dispatcher resumption
  pcs** — after `blr vt+0xf8` (0x2bd8d30), after `blr vt+0x108` (0x2bd8d54),
  sub_2bd8dac (0x102bd8dac), and continueAfterFlagsLoaded_ (0x102bd1d68) —
  **0 hits in all 3 runs.**
- **JIT_DUMP_REGION on [0x102bd8ce8, 0x102bd9060) fired exactly ONCE, at
  block-entry 0x102bd8ce8** (the dispatcher prologue). No other pc in the whole
  dispatcher+sub window was ever a block entry.

Block-entry-definitive: the dispatcher 0x102bd8ce8 is translated + entered, but
the guest control flow NEVER reaches 0x2bd8d18 (post-getter resume), 0x2bd8d30 /
0x2bd8d54 (post-leaf resumes), 0x2bd8d60 (`bl sub_2bd8dac`), sub 0x102bd8dac, or
continueAfterFlagsLoaded_ 0x102bd1d68. The dispatcher terminates at/inside its
first call-boundary (the getter `bl 2174c04` or earlier leaf path) without
falling through to the unconditional `bl sub_2bd8dac`.

## Verdict (do-not-re-tread)

SH228's negative is **re-confirmed at the newest HEAD** with fresh, precise,
block-entry-definitive measurement: the DMCONT manufactured-manager continuation
stays **LATENT**. Routing vt[+0x1f0] to the REAL continueAfterFlagsLoaded_ is
necessary-but-insufficient — the dispatcher never even reaches the `bl sub`
that would dispatch it, because a fabricated all-leaf manager supplies no genuine
flags-loaded/network-fetch state for the dispatcher's resolve/leaf path. The
DMCONT lever fires the instant a real flags-loaded engine-init state emerges
(a live session), unchanged. Route-B live-DM structural gate UNCHANGED.

## Code

`sh240_dmcont_dispatch_chain_pinned_fresh_negative` byte-pins the full chain so a
future drive/fix starts from drift-verified anchors (all verified on the real
libroblox.so, 1 passed / 84 filtered):
- fnB 0x102bd1b98 = 0xd10103ff (sub sp,#0x40) ; `bl 0x2bd8ce8` at 0x102bd1c10 =
  0x94001c36.
- dispatcher 0x102bd8ce8 = 0xd10143ff (sub sp,#0x50).
- getter 0x2174c18 = 0xb0028808 (adrp x8,7275000) ; 0x2174c28 = 0xc8dffd00
  (`ldar x0,[0x102727550]` — the seeded manager holder).
- dispatcher leaf blrs: 0x102bd8d2c / 0x102bd8d50 = 0xd63f0100 (vt+0xf8 / +0x108).
- `bl sub_2bd8dac` at 0x102bd8d60 = 0x94000013.
- sub_2bd8dac 0x102bd8dac = 0xd104c3ff (sub sp,#0x130) ; 0x102bd8e20 = 0xf940f908
  (ldr x8,[x8,#496] = vt+0x1f0) ; 0x102bd8e28 = 0xd63f0100 (blr → continueAfterFlagsLoaded_).
- continueAfterFlagsLoaded_ 0x102bd1d68 = 0xa9ba7bfd (stp x29,x30,[sp,#-96]!).
- 4-alignment + in-window for all twelve .text sites.

No production path edited. recon-v3 plane re-verified green at this HEAD
(capture_taskv4_frame.sh: 24 task-driven frames, present #19..#23 swap Ok(0x1),
192 node pops, no json abort, EXIT 124).

## Standing (unchanged)

Route-B live-DM = structural gate. SH174 capture-latch arming *(0x106391908) at a
real make_shared<DataModel> = the single forward hook. DMCONT is WIRED + ARMED but
LATENT (fresh at HEAD). recon-v3 immediate-priority deliverables stay shipped +
verified.