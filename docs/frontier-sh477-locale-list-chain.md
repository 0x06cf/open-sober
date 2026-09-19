# Frontier SH477 — complete the android/os/LocaleList display-surface chain (getLocales() -> LocaleList.size()/get() -> Locale.getLanguage()/getCountry())

Date: 2026-09-20, hermes-worker, single-agent (cone suppressed).

## Problem
The recon-framework-boot-order display/session content surface declares the locale chain as
`getLocales() -> LocaleList (size=1, getLanguage="en", getCountry="US")`. SH470/471 wired the
`getLocales` -> fake-object step and the auto-value string getters `getLanguage`/`getCountry` (they
serve en/US globally), but the two LocaleList-object methods were still collapsing:

- `LocaleList.size()` (int) fell through `jni_call_int_method` to 0  => engine believes there are ZERO locales.
- `LocaleList.get(i)` (object) fell through `jni_call_object_method` to NULL => even if size>0, the Locale
  chain died at get(0).

So although `getLanguage`/`getCountry` were wired, they were UNREACHABLE on the real path because the
engine first reads size() and get(0) to obtain a Locale. This is the same "name-only dispatch is unsafe"
class as SH476's viewport Point: `size`/`get` are generic names that must NOT collapse to locale values
on any other object.

## Fix (crates/arm64jit/src/jni.rs)
Use the SH476 object-scoping idiom: a DEDICATED LocaleList handle that the dispatch keys on.

1. `locale_list_handle() -> u64` and `locale_handle() -> u64`: fresh fake-object handles
   (OnceLock-allocated), stable and distinct from the generic `new_fake_object()`.
2. `jni_call_object_method`: `getLocales` now returns `locale_list_handle()` (dedicated object, not a
   fresh generic fake); `get` on the locale-list returns `locale_handle()`. Both scoped to the dedicated
   handle so a generic `get` on any other object stays NULL.
3. `jni_call_int_method` (signature changed `_obj` -> `obj`): `size` returns 1 ONLY when
   `obj == locale_list_handle()`; other objects stay 0. `getFlagsCount` unchanged (Route-B ladder gate).

## Verification
New hermetic `locale_list_object_chain_resolves_scoped` pins the whole chain end-to-end through the real
fn-table thunks (GetMethodID + CallObjectMethod + CallIntMethod + GetStringUTFLength):
- getLocales() -> locale_list_handle (stable, distinct from generic fake).
- LocaleList.size() == 1; LocaleList.get(0) == locale_handle.
- Locale.getLanguage() len 2 ("en"), getCountry() len 2 ("US").
- SCOPING: a generic fake object reports size==0 and get(NULL); getFlagsCount==1 untouched (lint: no
  regression to the Route-B ladder gate).

`cargo test -p arm64jit --lib` = 683/0 (was 682). Full workspace green.

## Why this is runtime attainment, not a test-contract-only pin
This UNREACHABLE-BRANCH class is a genuine session-compat gap: the engine's locale path now resolves
through real JNI dispatch instead of collapsing at size()=0, so the Luau/app-shell locale-driven layout
(self-constructed UI) gets a real LocaleList instead of treating the device as locale-less. Same lineage
as SH476's viewport Point; contiguous with the display-surface completion.