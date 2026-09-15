# SH168 — data-persistence pre-staging (latent readiness, objective 2b)

Cycle: Sep 15 2026, hermes-worker. Recon: deleg_6a9bf0d6 (task-1, READ-ONLY). Tool: scripts/prestage_data_init.py.
Related: SH131b (filesdir global seed), SH129 (cookie-ingress ABI), docs/frontier-sh167-dm-alloc-capture.md.

## Goal
Make a FUTURE real Roblox session's data persistence "just work" the moment a session forms
(GPU-host / real-input migration — the standing live-DM / app-shell migration gate). Readiness-only,
latent-but-correct.

## Verified engine facts (cross-checked against libroblox.so + APK)
- **`rbx-storage.db` opens from the CACHE dir, NOT the SH131b files-dir global**: literal
  "rbx-storage files from CacheDir" (fileoff 3516152) -> guest
  `/data/user/0/com.roblox.client/cache/rbx-storage.db`. SH131b seeded the FILES-dir string at
  0x10726d600; the db lives under cache/ — keep both, they serve different dirs.
- It is a CONTENT/asset cache (KVS by BLOB id), NOT auth. DDL (verbatim, off 4573535):
  `CREATE TABLE IF NOT EXISTS files (id BLOB PRIMARY KEY NOT NULL, content BLOB, size INTEGER DEFAULT 0 NOT NULL,
   hits INTEGER DEFAULT 0 NOT NULL, atime INTEGER DEFAULT 0 NOT NULL, category INTEGER DEFAULT 0 NOT NULL,
   score INTEGER DEFAULT 0 NOT NULL, ttl INTEGER DEFAULT 0 NOT NULL);` + indexes
  files_atime_idx/size_idx/category_idx/score_idx + files_content_null_idx (partial, WHERE content IS NULL).
- First open = idempotent `BEGIN EXCLUSIVE TRANSACTION; CREATE TABLE IF NOT EXISTS files(...); ...
  COMMIT; PRAGMA optimize` with journal_mode=WAL, locking_mode=EXCLUSIVE, page_size 4096. A pre-created
  db with EXACTLY this schema short-circuits the missing-file / empty-init branch and lands straight
  into WAL+EXCLUSIVE with auto-created -wal/-shm sidecars.
- Auth (.ROBLESECURITY + user id) is NATIVE storage behind the SH129 cookie-jar-init wall
  (jar ctor NULL at 0x21fce24; nativeSetMultipleCookies 0x102202ff8 / read-back 0x1021ff6b0
  netscape-line) — INDEPENDENT plane, not pre-seedable, unblocks only when a real session forms.

## The tool
`scripts/prestage_data_init.py [ROOT]` (python3, stdlib sqlite3; idempotent):
1. mkdir -p {ROOT}/data/user/0/com.roblox.client/{files,cache,databases,shared_prefs,code_cache,app_webview}.
2. Create a valid EMPTY SQLite `cache/rbx-storage.db` (page_size 4096, user_version 0, header
   `SQLite format 3\0`) carrying the engine's exact `files` DDL + 5 indexes.
3. Self-verifies PRAGMA integrity_check + 8-col + 5-index.
Root = $SOBER_ANDROID_ROOT, else argv[1], else /var/lib/open-sober/data (dry-write).

## Safety
Host-disk ONLY; never touches guest memory or the seeded files-dir global. With SOBER_ANDROID_ROOT UNSET
(the current working boot), fsmap passes guest /data paths through verbatim and NEVER references the
staged tree — zero boot disturbance (verified: the temporary pre-stage under /tmp/prestage-test did not
appear in any of the working ladder runs). Verified: `python3 scripts/prestage_data_init.py /tmp/prestage-test`
produces integrity=ok, pagesize 4096, user_version 0, 8 cols, 5 indexes, correct header magic.

## Activation
When a real session launches, mount SOBER_ANDROID_ROOT=<stage root> -> the engine's
cache-dir path remaps onto the pre-created db; first open does an idempotent CREATE-no-op, WAL+EXCLUSIVE
engages, INSERT OR REPLACE rows + -wal/-shm persist across restarts.

## Honest boundary
This unblocks the DATA plane only. Sign-in memory additionally needs the SH129 cookie-jar-init advance
once a real session forms (structural). Both remain behind the live-DM/app-shell migration gate.