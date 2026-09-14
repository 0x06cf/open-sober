# SH131b — Seed the engine's files-directory global (persistence base path)

## Goal
Deeper toward the end goal's data-persistence path (rule 2b): get the REAL
client's datastore base path onto persistent host disk. The standing SH114
getters are latent because getFilesDir only fires when the app-data-model builds,
which is behind the Lua app-shell wall (SH126). This de-gates the base-path
plane from that wall.

## Finding (deleg_fbb8faf7, disasm-verified on the real binary)
The client stores its data dir via
`Java_com_roblox_engine_jni_NativeSettingsInterface_nativeSetFilesDirectory`
(file 0x21f7654 = guest 0x1021f7654): it copies the extracted path into the
24-byte global `std::string` at **file 0x726d600 = guest 0x10726d600**
(disasm `adrp 726d000; add x8,x8,#0x600` + `str q0,[x8]`).
**Address-correction:** the earlier SH114 comment (and the subagent's first pass)
claimed guest `0x1026d600` — WRONG: that is in the read-only code segment
[0x100000000,0x1062d8190). The real slot is `0x10726d600`, which lives in the
rw- segment [0x1067d67c0,0x107333c3c). Also, driving the native itself is a
dead-end: its GetStringUTFChars helper reads a real Java-arena jstring, not our
fabricated `new_string_utf_handle`, so the materialized string came back empty
(verified: slot all-zeros after the native returned Ok).

## Fix (elfjit.rs, opt-in `--v2boot-set-filesdir` ladder rung)
Host-seed the same 24-byte libc++ `std::string` slot the engine's consumers
read, in LONG form (the 36-byte path exceeds libc++'s 22-byte SSO):
- `[0..8]` = `__data_` → a guest-arena buffer holding the path + NUL;
- `[8..16]` = `__size_` = 36;
- `[16..24]` = `__cap_` = 36, bit0 **clear** (the libc++ `__cap_ & 1` long
  discriminator).
Isolated helper `seed_libcpp_long_string(global, buf, bytes)` + hermetic tests
`sh131_seed_libcpp_long_string_places_path_and_sizes` / `_rejects_bad_args`.

## Verification (real libroblox.so, opt-in)
`--v2boot-set-filesdir` rung runs clean in the combined ladder:
```
[elfjit:v2boot-setfilesdir] seeded libc++ string @ [0x10726d600] = "/data/user/0/com.roblox.client/files" (ptr=... size=36)
[elfjit:v2boot-setfilesdir] verify read-back [0x10726d600] ptr=... size=36 = "/data/user/0/com.roblox.client/files" SEEDED
```
EXIT 0, no regression: the SH130 combined capture (runs/capture_sh130.sh) still
EXIT 0 with 24 real task-driven frames.

## Scope (honest)
The global is now seeded, so the engine's OWN SQLite datastore (rbx-storage.db)
opens with a real base path → routes through fsmap to persistent host disk —
once a session actually opens it. That open is still behind the Lua app-shell /
DataModel wall (SH126); this is a latent-but-correct, boot-safe prerequisite
(plain data stores, no guest byte changes), test-covered. The 24-frame plane
and all opt-in gates are unchanged.

## Files
- crates/arm64jit/examples/elfjit.rs: `seed_libcpp_long_string` + the
  `--v2boot-set-filesdir` rung + 2 hermetic tests.
- crates/arm64jit/src/jni.rs SH114 comment still says 0x1026d600 (historical,
  not an address the getters write); corrected in this doc + rung.