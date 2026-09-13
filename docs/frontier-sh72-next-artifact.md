# SH72 — next artifact options (multi-sprite composite of real UI now lands headlessly)

SH72 landed the plumbing: the engine's own emitter draws SEVERAL real Roblox UI
sprites at distinct positions in one frame from a shared atlas (spinner + robux
icon + jump button, alpha-composited over a dark backdrop), 8/8 pixel probes
present=true, captured frame runs/sh72-emitter-multi.png. The next unblocked
steps, in reachable order:

## (A) Pull a login-screen-representative real sprite set (<1 hr)

The three sprites are loading/game UI. The Roblox LOGIN surface renders the
Roblox logo + username/password fields + a green "Log In" button. Candidates
already extracted (ct 0/2/3/4/6 all decode offline):
- `Controls/DesignSystem/ButtonA/B/X/Y*.png` — colored buttons (login-button-like).
- `LegacyRbxGui` / `TopBar` — Roblox wordmark/logo assets.
Swap `RENDEREMITTER_MULTI_TEXTURES` to a login-biased list (the set is
env-configurable already) → a recognizable "login" composite through the engine.

## (B) Make the composite ANIMATED (progressed loading screen) (<1 hr)

SH71 spins the single spinner; extend the multi path so the spinner box also
rotates per frame (RENDEREMITTER_SPIN with the multi function), keeping the
other sprites static — a live loading screen with multiple real elements.

## (C) Default SELF-LOAD + graceful fallback (<1 hr)

Make `RENDEREMITTER_MULTI` the home-layout default; `real_ui_textures()` already
skips failed sprites with a WARN (falls back to whatever loads). Low risk, keeps
SH68 baseline reproducible via the existing env.

## Standing wall (unchanged, out of reach statically)

The engine never self-populates a session: type-4 producer vector [0x106829ea8]
is external framework glue (no in-code store — SH46/53); nativeGameGlobalInit
0x102206404 runs real init then parks (SH54/55); no in-image path constructs a
GuiObject and appends to the scene list. A true self-driven login screen
requires the Lua app-shell / game-activity path beyond the park.

## Recommendation (A first)

The multi-sprite plane is proven; the highest-value next increment is (A) — point
the same machinery at real Roblox login ui sprites (logo + fields + "Log In"
button) so the composite reads as a login screen, then (B) animate it. Both are
pure env/data changes on the SH72 function (the atlas/UV/blend/probe machinery is
done). Do NOT attempt per-node emitter vt[+24] (SH64 desync, per SH69 next-artifact
note).