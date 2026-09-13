# SH73 — next artifact options (real Roblox auth surface now renders headlessly)

SH73 landed the first real **login/auth** Roblox surface through the engine's own
emitter path: the real `reversevignette` backdrop + the RO-BLOX wordmark, both
pixel-verified (wordmark byte-exact), captured frame
runs/sh73-emitter-login.png. Next unblocked steps:

## (A) More login-complete Auth set (<1 hr)
The same `ExtraContent/textures/ui/LuaApp/graphic/Auth/` folder also has
`classind_16.png` and `noconnection.png` (+ `FPSBackground.png`). Placing
`noconnection` (a connection-loss chip) under the wordmark mimics the real
offline-login state. `real_ui_textures` already skips failed sprites with a
warn and supports absolute paths — pure data/placement change on the SH73
function.

## (B) Animate the auth surface (<1 hr)
SH71's `RENDEREMITTER_SPIN` rotates a sprite per frame. Apply the same per-frame
rotation/scroll to the login composite (spin the wordmark or drift the vignette
scroll) for a "live" auth screen across present-walker frames — same
radial-sweep verification.

## (C) "Purple Login-button" treatment (<1 hr)
The extracted set has no real green "Log In" PNG (Roblox draws those as solid
rounded rects). SH68 already proved the engine emitter composites a
semi-transparent panel + a green button bar (solid-prim layering over the same
draw path). Combine SH68's solid button bar WITH the SH73 real auth artwork in
one frame → a fuller login screen: real wordmark + real vignette + a UI-solid
green "Log In" bar. This is the next visual step up; both halves are proven, so
it is a composition problem, not a render reach problem.

## Standing wall (unchanged, out of reach statically)
The engine never self-populates a session: type-4 producer vector [0x106829ea8]
external glue; nativeGameGlobalInit parks; no in-image path constructs a
GuiObject and appends to the scene list. A true self-driven login screen still
needs the Lua app-shell / game-activity session path. Every SH72/73 increment is
harness-authored geometry + real artwork through the engine's own emitter.

## Recommendation (A → B)
(A) is the cheapest and moves closest to a complete login read; then (B) for a
live surface; (C) is the visual ceiling and worth doing once A+B land — it is
the point where the frame stops being "a wordmark" and becomes "a login screen".
Do NOT attempt per-node emitter vt[+24] (SH64 desync class, per SH69 next-artifact
note).