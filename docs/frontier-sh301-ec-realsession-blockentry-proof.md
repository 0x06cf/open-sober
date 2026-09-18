# Frontier SH301 — EC-world realsession-reader frontier pinned at block-entry level

Date: Sep 18, 2026, hermes-worker. Single-agent (cone suppressed). Pure hardening +
doctrine closure of SH300's "dormant-by-measurement" verdict — no production path
edited, no new seed.

## tl;dr

SH300's realsession flag seed ([0x106d31e28]=1 at EC-world entry 0x102e24598) was
claimed dormant-by-measurement, but that verdict relied on entry-level **pc-set**
tooling (JIT_TRACE / JIT_REGION_WATCH), which the objector can wave away with
"the EC body compiles as ONE straight-line block, so its interior pcs and inlined
`bl` targets are blind to entry-level probes." SH301 closes that objection with a
**block-entry-target** test that is structurally immune to single-block blindness.

## The decisive evidence (block-entry level, ~10 probe configs, real libroblox.so)

Within the EC body 0x102e24598, region-watch/REGIONDUMP fires at exactly:

- `0x102e24598` (block entry) and `0x102e245f4` (resume after `bl 0x23f1354`);
- **`0x102b504e4`** — the interior prefix string-assign (`bl 0x2b504e4`
  @ `0x2e24610`/`0x2e24618`) **fires as a guest block entry.**

That last point is the crux: it PROVES interior guest `bl` targets DO open block
entries in this JIT (they are not inlined mid-block). Therefore, IF the realsession
reader at `0x2e246f4` were reached, at least one of its two `bl` targets MUST also
open a block entry. Across every config, **neither fires**:

- `0x1023c5538` — real `/nativeAppBridgeV2InitWithParams` AppBridge-V2 factory
  (flag=1 path, `bl 0x23c5538` @ `0x2e24704`);
- `0x1023c1b0c` — benign singleton builder (flag=0 path, `bl 0x23c1b0c` @ `0x2e24730`).

## Conclusion

The EC body provably soft-returns BEFORE `0x2e246f4` — the reader `cbz w8` on
[0x106d31e28] is unreached headlessly regardless of the seeded flag value. SH300's
flag seed is CORRECT and latched (fires every run, persists), but its reader never
runs; the "dormant" is GENUINE, NOT probe-blindness. Do-not-re-tread SH300 (do not
re-seed 0x106d31e28; the actual gate is the body's pre-reader continuation, the
SH299-NEXT-GATE class — `bl 0x23f1354` / the app-request build 0x2e24678..).

## SH301 code

- `sh301_ec_body_block_entry_doctrine_pinned()` (jit.rs, real-image guard family as
  sh202/sh298): byte-pins the 9 frontier anchors — EC entry 0x2e24598=0xa9ba7bfd,
  entry-bl 0x2e245f0=0x97d73359, prefix string-add 0x2e24610=0x91002280, prefix bl
  0x2e24618=0x97f4afb3, reader 0x2e246f4=0xb001f868 / 0x2e246f8=0x3978a108 /
  0x2e246fc=0x34000188, real-bl 0x2e24704=0x97d6838d, benign-bl 0x2e24730=0x97d674f7
  — so a drifted real-image constant fails loudly in batch (sh233 doctrine).

## HONEST (do-not-over-claim)

- Confirms + hardens SH300; makes the `bl`-target proof explicit and test-enforced.
- No DM manufactured (MH_* false, DM-root 0). Route-B live-DM gate UNCHANGED.
- SH174 capture-latch stays the single forward hook. The EC-world real-init path is
  armed-and-correct; still dormant because its reader is unrereachable headlessly.

## Verify

- `cargo test -p arm64jit --lib -- sh301` = 1 passed.
- `cargo test -p arm64jit --lib` = 404/0.
- `cargo build --workspace` EXIT 0.

## NEXT GATE

Unchanged from SH300: drive the dmfn/EC-body pre-reader continuation (the `bl
0x23f1354` marshaller soft-return / app-request build 0x2e24678..0x2e25200) so the
realsession reader becomes reachable — a live-object / SESSION-CTOR class, not a
static seed. Keep single-drive discipline; do-not-re-tread broad seeds.