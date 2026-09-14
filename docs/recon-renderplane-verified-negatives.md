# Recon — emitter/render draw-loop: verified-negative findings + pinned wrapper ABI

## Why this doc exists
Three consecutive read-only subagents (deleg_485f301c, deleg_a548c65b,
deleg_f28b2be5) each surfaced a candidate "next render fix". Every one resolved
to **already-closed on HEAD, or a false lead**. This records the verdicts so the
autonomous loop does NOT re-chase them. Verified against source + real
`libroblox.so` (objdump), not assumed.

## 1. `glDrawBuffer(GL_NONE)` emitter-drop — FALSE LEAD (STALE)
- docs/frontier-sh67-emitter-pixel-probe.md is literally titled *"GL_NONE /
  draw-buffer hypothesis DISPROVEN"*. llvmpipe reports `GL_DRAW_BUFFER=0x0`
  for the default FBO yet still rasterizes; setting glDrawBuffers(GL_BACK)
  does NOT change the reported value, and pixels still land.
- Emitter pixels are VERIFIED present today (SH67d: 12/12 tiles byte-exact;
  SH72 multi; SH77 login text; SH140 link row 11/11).
- The singular `glDrawBuffer` is a desktop-only symbol Mesa's `libGLESv2.so.2`
  does not export — only the ES3 plural `glDrawBuffers` exists (elfjit.rs:2340-42).
- DO NOT spend time on this.

## 2. FP16 / SIMD decoder gaps — ALREADY CLOSED (STALE doc)
- docs/fp16-decode-gap.md and docs/fp16-scan/text.unsupported.txt predate
  commits dce3c8c (SIMD orr/bic .2S/.4S) and 092e829 (mrs fpcr + ldpsw).
- VERIFIED on HEAD: `scandecode` over the real libroblox.so .text window
  (0x102d95980..0x1062d5a84) = **1,114,177 instructions, 0 Unsupported,
  0 PANIC hits — 100% decode coverage.** orr/bic .2S/.4S (VecMovi decode.rs
  ~3254-3359), fmaxnm/fminnm (VecFpArith 1469 / SimdFp16As 2404), fcvtas/fcvtzu
  (FcvtToInt) are all handled. decode() never panics (translator traps on
  Unsupported; no Rust panic).
- Regenerate the scan if it's ever needed: `cargo build --example scandecode &&
  target/debug/examples/scandecode libroblox.so 0x102d95980 0x1062d5a84`.

## 3. Emitter draw resolves to a no-op — WRAPPER/EMITTER IS NOT BROKEN
objdump of geometry wrapper **0x5b35288** (file vaddrs; guest +0x100000000):
```
5b3529c mov w22,w1     ; x[1]=MODE-TABLE INDEX
5b352a0 mov w1, w3     ; x[3]=geom_key -> w1 for primitive_setup
5b352a4 mov w21,w5     ; x[5]=path (0=array, nonzero=indexed)
5b352a8 mov w20,w4     ; x[4]=COUNT
5b352ac mov w23,w2     ; x[2]=FIRST/stride-multiplier
5b352b4 bl 0x5b353d0   ; primitive_setup (always, first)
```
- count rides **x[4]/w20 in BOTH paths** (`mov w1,w20`).
- mode = `mode_table[x[1]]` @ **0x10225780**: [0]=4 GL_TRIANGLES, [1]=1 GL_LINES,
  [2]=0 GL_POINTS, [3]=5 GL_TRIANGLE_STRIP. **NO GL_TRIANGLE_FAN: index >=4 reads
  zero-padding -> GL_POINTS(0)**, a legal near-noop.
- glDrawArrays GOT 0x67d2318, glDrawElements GOT 0x67d2310 = real Mesa (SH67b).
- Every atlas driver disables GL_CULL_FACE(0x0B44)/GL_DEPTH_TEST(0x0B71)/
  GL_SCISSOR_TEST(0x0C11) before the draw; glCullFace never called.
- All four atlas paths (quad x1=3,x4=4; grid x1=0,x4=6N; home x1=0,x4=30;
  multi x1=0,x4=6·(1+N), x5=0) render byte-exact.
- **The ONLY count=0 no-op in source is `--renderframe-drawprobe`
  (elfjit.rs ~8108-8128: x4=0, empty prim-list).** It's a by-design dispatch
  probe (SH24 proved slot 9); it can NEVER render — primitive_setup's
  `cmp x9,x8; b.eq` early-returns with no attrib wiring — so do not "fix" it
  with a count.

### The one real hazard class (defensive, for future drives)
SILENT near-noop happens only two drive-side ways: **count in the wrong
register (must be x[4], NOT x[5])** or **mode index >=4** (no fan). If any NEW
coherent-renderer drive routes through GLES dispatch slots 9/10 @0x106d3b2f0,
it MUST run `--renderframe-seedgles` first (slots are bridge pointers ONLY after
seeding; unseeded -> SH19-class silent no-op). The mesh SH141/142 path already
uses the coherent renderer + `--renderframe-seedgles` + correct x4 count and
RENDERS (verified).

## Net
The render-plane base is healthy: real mesh geometry (SH141) + real APK texture
(SH142) render through the engine's own wrapper with byte-exact/probe-verified
output, decode coverage is 100%, and the emitter loop is closed. The standing
blockers to "real client renders its own screens + remembers sign-in" remain the
STRUCTURAL SH131d Lua app-shell wall / nativeInit 0x10232090c / cookie-jar init,
not anything in the render-plane base. Rerun `scandecode` to refresh counts only
if a new decoder commit lands.