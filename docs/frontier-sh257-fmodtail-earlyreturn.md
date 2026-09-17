# SH257 (Sep 17, 2026, hermes-worker): CORRECT SH239's "do-init is absorbed by the FMOD
# tail / never completes construction" framing — with the FULL SH248c-f seed set, the
# app-shell init body's FMOD iterate 0x5fb30b4 COMPLETES via its empty-container
# early-return and do-init climbs to the governor. Single-agent (cone suppressed).
#
# ## Question (genuinely-open thread, SH244's explicit next + SH239's terminal)
# SH239 measured the app-shell/global-init ctor 0x102207b50 body runs 61 blocks to
# 0x102208eac and its terminal tail targets the FMOD/AAudio iterate 0x5fb30b4
# (0x102208ebc: `b 0x5fb30b4`) — and said do-init "never completes construction, absorbed
# by the sound/AAudio tail (0x5fb30b4)". SH244's explicit next: "does the tail's
# early-return path ever come back?" Both were measured WITHOUT the SH248c-f app-start
# seed set fully applied. This cycle re-runs with the full seed set and watches the
# FMOD iterate's own early-return.
#
# ## Measured (real libroblox.so, full DMCONT+SH245+SH248c-f env, EXIT 134, 0 region crashes)
# The FMOD iterate 0x5fb30b4 is a *container-iterator with an empty-early-return*:
#   0x5fb30d8 ldp x8,x9,[x0,#8]    ; [x0+8]=begin, [x0+16]=end of the audio-queue container
#   0x5fb30dc cmp x8,x9
#   0x5fb30e0 b.eq 0x5fb3134       ; begin==end -> SKIP the audio body
#   ...audio init body (0x5fb30e4..0x5fb3130)...
#   0x5fb3134 ldr x8,[x20]         ; canary-reload
#   0x5fb3154 ret                  ; RETURNS to the caller
# In the full-seed run this early-return FIRES (region hit 0x105fb3134), so the tail
# RESOLVES instead of absorbing control. The chain then climbs:
#   post-do-init 0x1023eff4c (sub sp,#0x180, region hit) -> governor 0x102e9fa84 (region
#   hit) -> govtail 0x102ea30dc (region hit) -> THEN nativeAppBridgeAppStart runs and dies
#   at the STANDING live-object map wall 0x1021dde34 (SH248g/h/249..256).
#
# ## What this does and does NOT do
# - DOES: corrects SH239's "never completes, FMOD-absorbed" with a measured early-return on
#   the real binary; do-init construction now demonstrably ADVANCES to the governor.
#   Adds +1 hermetic real-image guard (sh257) pinning the corrected contract (the FMOD
#   empty-check ldp/cmp/b.eq + early-return canary-reload + post-do-init entry + governor
#   entry) so a drift fails loudly.
# - DOES NOT: a live DataModel. After do-init->governor->govtail, app-start still dies at
#   0x1021dde34 (the SH174/SH204 live-object map wall). Route-B live-DM structural gate
#   UNCHANGED; SH174 capture-latch stays the single forward hook.
#
# ## Code / files
# - crates/arm64jit/examples/elfjit.rs: +sh257 hermetic (14 real-image byte-pins).
# - Repro runs/capture_sh257_fmodtail.sh (probe) + runs/capture_sh257b_fmdtail_feed.sh
#   (full seed, watches do-init-completion chain). Evidence logs runs/sh257*.txt gitignored.
# - cargo build --workspace EXIT 0; cargo test --workspace green; examples 95/0 (was 94).
# - Route-B live-DM gate UNCHANGED; recon-v3 deliverables stay shipped + verified.