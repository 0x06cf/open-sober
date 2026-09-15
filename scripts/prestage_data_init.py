#!/usr/bin/env python3
"""SH168a — Open-Sober data-persistence pre-staging (readiness, latent).

Creates the host file tree a REAL Roblox session will persist into, so that the
moment a session forms (post GPU-host / real-input migration, SOBER_ANDROID_ROOT
mounted) the engine's own datastore opens land on an already-valid, writable,
schema-correct root. Source: READ-ONLY recon deleg_6a9bf0d6 (task-1), cross-checked
against the engine's own SQLite DDL literal in libroblox.so.

FACTS VERIFIED AGAINST THE BINARY:
- rbx-storage.db opens from the CACHE dir, NOT the SH131b-seeded files-dir global:
  guest /data/user/0/com.roblox.client/cache/rbx-storage.db ("rbx-storage files from
  CacheDir"). It is a CONTENT/asset cache (KVS by BLOB id), NOT auth.
- The engine's first open is an idempotent
  'BEGIN EXCLUSIVE TRANSACTION; CREATE TABLE IF NOT EXISTS files(...); ... COMMIT; PRAGMA optimize'
  tuned with journal_mode=WAL, locking_mode=EXCLUSIVE, page_size 4096. A pre-created db
  carrying EXACTLY that schema short-circuits the missing-file / empty-init branch and
  lands straight into WAL+EXCLUSIVE mode with auto-created -wal/-shm sidecars.
- Auth (.ROBLESECURITY) is native-storage behind the SH129 cookie-jar-init wall — this
  script does NOT and cannot touch it (independent plane).

SAFETY: host-disk ONLY. Never touches guest memory or the seeded files-dir global.
Idempotent. If SOBER_ANDROID_ROOT is unset the script still emits the tree under $R
== default {R}/data/user/0/com.roblox.client/... but the CURRENT boot passes guest paths
through verbatim (fsmap inactive when SOBER_ANDROID_ROOT unset), so the staged tree is
unreferenced by the working boot — zero disturbance.

Usage:
  python3 scripts/prestage_data_init.py [ROOT]
  # ROOT default: $SOBER_ANDROID_ROOT; if neither, prints the tree it WOULD create
  #               under /var/lib/open-sober/data without writing (dry-run).
Leaves:
  {R}/data/user/0/com.roblox.client/{files,cache,databases,shared_prefs,code_cache,app_webview}/
  {R}/data/user/0/com.roblox.client/cache/rbx-storage.db  (valid empty SQLite, engine schema)
"""
import os
import sqlite3
import sys

# Engine's register_files-schema, byte-verbatim from the binary DDL.
ENGINE_FILES_DDL = (
    "CREATE TABLE IF NOT EXISTS files ("
    "id BLOB PRIMARY KEY NOT NULL, content BLOB, "
    "size INTEGER DEFAULT 0 NOT NULL, hits INTEGER DEFAULT 0 NOT NULL, "
    "atime INTEGER DEFAULT 0 NOT NULL, category INTEGER DEFAULT 0 NOT NULL, "
    "score INTEGER DEFAULT 0 NOT NULL, ttl INTEGER DEFAULT 0 NOT NULL)"
)
ENGINE_INDEXES = [
    "CREATE INDEX IF NOT EXISTS files_atime_idx ON files(atime)",
    "CREATE INDEX IF NOT EXISTS files_size_idx ON files(size)",
    "CREATE INDEX IF NOT EXISTS files_category_idx ON files(category)",
    "CREATE INDEX IF NOT EXISTS files_score_idx ON files(score)",
    "CREATE INDEX IF NOT EXISTS files_content_null_idx ON files(category) WHERE content IS NULL",
]

PKG = "com.roblox.client"
APP_DATA = ["files", "cache", "databases", "shared_prefs", "code_cache", "app_webview"]


def resolve_root() -> str:
    env = os.environ.get("SOBER_ANDROID_ROOT")
    if env:
        return env
    if len(sys.argv) > 1:
        return sys.argv[1]
    return "/var/lib/open-sober/data"


def make_rbxstorage(path: str) -> None:
    """Create a valid empty rbx-storage.db carrying the engine's exact content schema."""
    con = sqlite3.connect(path)
    try:
        con.execute(ENGINE_FILES_DDL)
        for idx in ENGINE_INDEXES:
            con.execute(idx)
        con.commit()
        # Verify.
        row = con.execute("PRAGMA integrity_check").fetchone()
        assert row and row[0].lower() == "ok", f"integrity: {row}"
        cols = con.execute("SELECT COUNT(*) FROM pragma_table_info('files')").fetchone()
        assert cols[0] == 8, f"expected 8 columns, got {cols[0]}"
        idxn = con.execute("SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND sql IS NOT NULL").fetchone()
        assert idxn[0] == 5, f"expected 5 named indexes, got {idxn[0]}"
    finally:
        con.close()


def main() -> int:
    root = resolve_root()
    app_ctx = os.path.join(root, "data", "user", "0", PKG)
    made_dirs = []
    for d in APP_DATA:
        p = os.path.join(app_ctx, d)
        if not os.path.isdir(p):
            os.makedirs(p, exist_ok=True)
            made_dirs.append(p)
    db = os.path.join(app_ctx, "cache", "rbx-storage.db")
    if not os.path.exists(db):
        make_rbxstorage(db)
    else:
        # Idempotent: only ensure a pre-existing db still has the engine schema.
        con = sqlite3.connect(db)
        ok = con.execute("PRAGMA integrity_check").fetchone()
        con.close()
        if ok[0].lower() != "ok":
            print(f"WARNING: {db} exists but failed integrity_check ({ok[0]}); not overwriting.", flush=True)
    print(f"prestaged app-data root: {app_ctx}")
    print(f"rbx-storage.db: {db} (exists={os.path.exists(db)}, size={os.path.getsize(db) if os.path.exists(db) else 0})")
    for p in made_dirs:
        print(f"  created dir: {p}")
    r = os.environ.get("SOBER_ANDROID_ROOT")
    print("NOTE: activate by launching the session with SOBER_ANDROID_ROOT={!r} set; the current"
          " (unset) boot passes guest paths through verbatim and does not reference this tree.".format(r or root))
    return 0


if __name__ == "__main__":
    sys.exit(main())