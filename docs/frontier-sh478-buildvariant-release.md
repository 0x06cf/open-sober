# SH478 — set InitParams.buildVariant to the real production variant ("release")

Single-agent (cone suppressed). recon-v3 immediate-priority deliverables re-verified
GREEN at fresh SH477 HEAD first (capture_taskv4_frame.sh attempt 1: 24 real
task-driven frames `present swap Ok(0x1)`, 195 node pops, 0 json abort, 0 crash,
EXIT 124 stable idle). Do-init/Route-B baseline re-probed on the real binary this
cycle too (capture_sh415: substrate 14/16 atoms completed jit_run (11 non-zero) +
2/16 stopped, once-guard bit0=1, DM-root [0x106a68818]=0x0 -> LIVE DM=false,
MH_FLAGS_LOADED/ENGINE_INITIALIZED/APP_READY true, AppBridgeV2 vt 0x1063a3410,
0 crash). Workspace green at final HEAD (cargo test --workspace EXIT 0; arm64jit
683/0; DELIBERATE — the two buildVariant pins are modified, not a net-new test, so
the lib count is unchanged; ~864 workspace total).

## The gap closed: buildVariant was served "" though the authoritative recon names "release"

`docs/recon-framework-boot-order.md` (the authoritative InitParams field map, line
74) names `buildVariant="release"`. The JIT's `auto_value_string_getter` served
`getBuildVariant` -> `""`. MEASURED in the real libroblox.so: the engine consumes
buildVariant in config/telemetry identity (`BuildVariant`,
`AddBuildVariantToGlobalTags`, `RobloxTelemetryAddAppBuildVariantToPoints`) and
holds `release`/`debug`/`production` literals to compare against it. An empty
variant is a degenerate value with no correct-world interpretation for a shipping
production client; `release` is the unambiguous constant. This is exactly the
session-content correctness class the SH469-477 lineage lands (a getter the engine
reads served a collapsed/wrong value).

- Production change (crates/arm64jit/src/jni.rs, one arm): `getBuildVariant` now
  returns `"release"` (7 bytes) instead of `""` (0). The value flows into the
  serialized AppBridge params the json writer reads; a valid 7-char SSO length vs
  the old 0-length is strictly safer for the RBX::json::Writer (a real, readable,
  in-SSO length, not a degenerate empty — same family as the SH134 baseURL /
  SH475 selectedTheme precedent).
- Test pins updated:
  - `jni_auto_value_params_getters_resolve_via_fn_table`: `getBuildVariant` moves
    out of the empty-default arm into `assert_eq!(len, 7, "release")` through the
    REAL CallObjectMethod + GetStringUTFLength dispatch.
  - `sh134_base_url_and_appbridge_string_getters_resolve_to_readable_jstring`:
    `getBuildVariant` removed from the "0-length" set (it is no longer empty; its
    length-7 pin lives in the fn-table test above). Neither test was weakened —
    the empty-set assert for the OTHER getters (userAgent/appStarterPlace/
    appStarterScript/username) is untouched.

Honest: NOT a DM / NOT a live-DM step (Route-B live-DM gate UNCHANGED; DM-root
[0x106a68818]=0, structural per SH462/467). Not a fix of a reachable-end-to-end
bug — the params serialization is on the latent session path; the change makes the
config identity the engine serializes CORRECT when that path advances, replacing a
degenerate empty with the real production variant. Latent-but-correct like every
axis. No re-treads (baseURL SH134 is a different value; this value had NO prior
owner). Production code only in jni.rs (off the 1MiB hooks; jit.rs/elfjit.rs/
session.rs untouched).

- Files: docs/frontier-sh478-buildvariant-release.md + crates/arm64jit/src/jni.rs
  (production value + 2 test pins). Commit (SH478).