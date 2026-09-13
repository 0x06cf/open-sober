# SH73 — next artifact options (real Roblox auth surface now renders headlessly)

SH73/74/75 landed the real **login/auth** surface through the engine's own
emitter path — real `reversevignette` + real RO-BLOX wordmark + real
`noconnection` chip + solid field + green button, all 10/10 probes present=true
(wordmark/chip/field/button byte-exact), captured runs/sh73-emitter-login.png.
Options (A) more-login-assets and (C) solid UI prims are DONE (SH74/75). Next
unblocked steps:

## (B) Animate the login composite (<1 hr)
SH71's `RENDEREMITTER_SPIN` rotates/per-frames a sprite. Apply a per-frame
change to the login surface so it reads as a LIVE login screen across
present-walker frames — e.g. a progress bar that grows per frame (the loading
state) or a gently drifting wordmark. Same radial-sweep/probe verification.

## (D) Login-placeholder text labels (<1 hr)
The solid field + green button read as a form but carry no text. If a font
asset exists in the extracted `content/fonts` / bmfont set, rasterize "Log In"
/Sign-up text into an atlas row and place it on the button — a more legible
login button. Requires glyph rasterization; font HXD/OTF may already be in the
APK (SH57 found "BuilderIcons fonts").

## Standing wall (unchanged, out of reach statically)
The engine never self-populates a session: type-4 producer vector [0x106829ea8]
external glue; nativeGameGlobalInit parks; no in-image path constructs a
GuiObject and appends to the scene list. A true self-driven login screen still
needs the Lua app-shell / game-activity session path. Every SH72-75 increment is
harness-authored geometry + real artwork through the engine's own emitter.

## Recommendation (B)
Animate first (cheap, makes the surface unmistakably "live"); (D) text is the
next legibility step. Do NOT attempt per-node emitter vt[+24] (SH64 desync
class).