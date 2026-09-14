# SH140 — Login link row: "Forgot password?" + "Sign up" (engine emitter, composite glyphs)

## What

The synthesized Roblox login surface (SH69-78) now includes the signature
**link row under the green button**: the blue "Forgot password?" and "Sign up"
labels, rendered through the **engine's own real geometry emitter** (0x105b35288,
GL_BLEND over the dark backdrop) from the APK's own SourceSansPro-Bold.ttf.

Critically, "Forgot password?"'s **`?` is a COMPOSITE glyph** in that font
(SH79 added recursive composite-glyph decoding but it stayed latent — every
prior shipped label used only simple letters). This is the **first visible
artifact to exercise SH79's composite path end-to-end**.

## Change (crates/arm64jit/examples/elfjit.rs, login_ui_textures + placements)

- Two new entries in the SH77 text-label rasterize table:
  `("Forgot password?", [59,130,246,255], 640, 48, 0.048)` and
  `("Sign up", [59,130,246,255], 512, 48, 0.048)`.
- Two new placements below the button (cy -0.74 / -0.85), each with a probe
  texel chosen to be a **3-wide solid stroke core** (x-1, x, x+1 all a=255) so
  GL_LINEAR sampling reads the full baked link color byte-exact:
  `("Forgot password?", …, 464, 5)` and `("Sign up", …, 189, 8)`.

## Verification

- **Hermetic** (`sh140_forgot_password_link_label_composite_question_rasterizes`):
  the shipped "Forgot password?" `?` decodes via composite recursion to contours;
  the full shipped label rasterizes >50 opaque px at the baked link color; and a
  3-wide solid stroke core exists at (464,5) / (189,8) for stable probing.
- **Real binary** (`runs/capture_emitter_login_text.sh` — standalone emitter
  path, stable): `engine emitter Ok ret=0x0 count=72 sprites=11 swap=Ok(1)` —
  sprites went 9 → **11**; **11/11 present=true** including both new link probes
  reading **`(59,130,246,255)` diff=[0,0,0,0]`** at their projected positions
  (byte-exact over the opaque backdrop); `present walker Ok(ret=0x1)`; 0 crash.
- Workspace **537/0**; elfjit example **42/0** (was 41).

## Scope / honesty

Host-authored geometry through the engine's own emitter path + real APK font +
real GL_BLEND — NOT engine self-constructed UI (the SH131d structural wall is
unchanged). This completes the login-surface exhibit with its recognizable
fingerprint link row and demonstrates SH79's composite-glyph support in a
shipped artifact.