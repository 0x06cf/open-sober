# Open-Sober run status (hermes-worker)

## SH476 — wire the PlatformParams viewport{Width,Height}Mm display surface
Completed this cycle: `getScreenPhysicalSizeInMillimeters` (Java static) -> Point object -> `x`/`y`
int FIELDS (338/190 Mm), scoped to a DEDICATED Point handle (SH476 idiom) so a generic fake object's
`x`/`y` stays 0. Wired slot 114 (CallStaticObjectMethod) + GetIntField dispatch. Hermetic test
`viewport_point_mm_surface_resolves_through_dispatch`. arm64jit 682/0 green, recon-v3 deliverable
re-verified (24 real task-driven frames). Commit SH476.

## SH477 — complete the android/os/LocaleList display chain
`getLocales() -> LocaleList.size()==1 -> Lock.get(0) -> Locale.getLanguage()="en"/getCountry()="US"`.
Scoped `size` (int) + `get` (object) to a dedicated LocaleList handle (SH476 idiom): the engine's
locale path now resolves through real JNI dispatch instead of collapsing at size()=0/NULL. Hermetic
test `locale_list_object_chain_resolves_scoped`. arm64jit 683/0 green. Commit SH477.

## Standing
- Route-B live-DM wall (engine self-constructs login/home GuiObjects) is structurally intact: the
  session-ctor / do-init path needs real Activity-session compat, not another seed (operator directives
  Sep 15/17/18: build-the-runtime, cone suppressed ZERO subagents, migration is not a stopping point).
- Display-surface lineage (SH469-477) complete: DisplayMetrics/Configuration + viewport Mm + locale.
- OWN screens / in-app login / sign-in persistence remain the end goal, gated on the live-DM/unclocked
  session.