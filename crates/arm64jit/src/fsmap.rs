// SPDX-License-Identifier: MIT
//
// Guest filesystem path remapping — the persistence enabler for the real
// client's data plane.
//
// The JIT's `guest_svc` file syscalls pass guest path pointers *verbatim* to
// the host libc (guest vaddr == host addr, no translation). A real Roblox
// Android session reads/writes its datastore, shared-preferences, cache and
// login/session cookies under Android *mount roots*:
//
//     /data/data/com.roblox.client/...        <-- app private data
//     /sdcard/...  /storage/emulated/0/...     <-- external storage
//     /cache/...                               <-- app cache
//
// On the host those absolute paths resolve against the host root, which either
// does not exist (ENOENT) or is not writable by the runtime process (EPERM) —
// so the client cannot persist anything. This module maps those guest roots to
// a real host directory that backs them, giving the client a *persistent*
// on-disk store: a value the client writes under `/data/...` is a real file on
// the host that survives a restart (i.e. "remembers sign-in").
//
// The mapped root is configured once (env `SOBER_ANDROID_ROOT`, or a test
// setter). When unset the remap is inactive and guest paths pass through
// unchanged — the existing boot behavior is untouched until a session root is
// provided. Only ABSOLUTE paths under a known writable Android root are
// remapped; relative (dirfd-relative) and all other absolute paths (`/proc`,
// `/system`, `/tmp`, ...) pass through, so reads of real system state are not
// disturbed.

use std::ffi::CStr;
use std::ffi::{CString, OsStr};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// The persistent host directory that backs the guest Android file system.
/// Defaults to `SOBER_ANDROID_ROOT` (absolute path) when the environment var is
/// set; `None` means remapping is disabled and paths pass through unchanged.
pub fn configured_root() -> Option<PathBuf> {
    static ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();
    ROOT.get_or_init(|| {
        std::env::var("SOBER_ANDROID_ROOT")
            .ok()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
    })
    .clone()
}

/// Test hook: pin the mapped root directly (the environment variable cannot be
/// depended on inside the parallel test harness). Clears any prior value.
pub fn set_root_for_tests(root: PathBuf) {
    let _ = configured_root(); // ensure the OnceLock is initialized before we clear
    // The OnceLock can't be re-set; use an override cell that takes precedence.
    *override_root().lock().unwrap() = Some(root);
}

/// Shared serialization lock for the parallel test harness: every test that
/// mutates the global fsmap root override (`set_root_for_tests`) must hold this
/// SAME lock for its whole critical section, so tests across modules (jit.rs,
/// fsmap.rs, session.rs's R1 stage/serve) never clobber each other's override
/// mid-test. Before this was shared, jit.rs (FS_ROOT_LOCK), fsmap.rs
/// (FSMAP_ROOT_LOCK) and session.rs (CACHE_LOCK) used three independent locks
/// and a jit.rs fsmap test could race the sh419 session test (both write the
/// override concurrently) -> intermittent sh419 failure.
pub(crate) fn test_root_mutex() -> &'static std::sync::Mutex<()> {
    static ROOT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    &ROOT_LOCK
}

fn override_root() -> &'static std::sync::Mutex<Option<PathBuf>> {
    static OVR: OnceLock<std::sync::Mutex<Option<PathBuf>>> = OnceLock::new();
    OVR.get_or_init(|| std::sync::Mutex::new(None))
}

fn active_root() -> Option<PathBuf> {
    override_root().lock().unwrap().clone().or_else(configured_root)
}

/// Public alias of the resolved (override-or-env) root, used by host staging paths (e.g. the R1
/// CoreScript mirror) so the same test override that `remap_path` honors also drives staging.
pub fn staging_root() -> Option<PathBuf> {
    active_root()
}

/// A guest filesystem path that has been resolved into a host path under the
/// configured Android root. Kept as a `CString` so its pointer is directly
/// consumable by the host libc call that takes it.
pub struct RemappedPath {
    host: CString,
    /// The directory that must exist before the path is used with O_CREAT /
    /// mkdirat; `None` if the target's parent is already guaranteed by the
    /// caller (or the path is a bare root).
    pub parent: Option<CString>,
}

impl RemappedPath {
    pub fn as_ptr(&self) -> *const libc::c_char {
        self.host.as_ptr()
    }
    pub fn host_path(&self) -> &Path {
        Path::new(OsStr::from_bytes(self.host.as_bytes()))
    }
}

/// Rewrite an absolute guest path under a writable Android root into the
/// corresponding host path under `active_root()`. Returns `None` when (a) no
/// root is configured, (b) the path is relative, or (c) the path is not under a
/// mapped root — the caller then passes the guest pointer through unchanged.
///
/// The four mounted roots are the standard Android persistent mount points the
/// client's data plane (datastore, shared_prefs, cache, session/cookie store)
/// writes to. Guest paths under the (read-only) boot images `/system`, `/vendor`,
/// `/apex`, `/odm` and the virtual `/proc`/`/dev` trees are deliberately NOT
/// mapped.
pub fn remap_path(guest: *const libc::c_char) -> Option<RemappedPath> {
    let root = active_root()?;
    if guest.is_null() {
        return None;
    }
    // SAFETY: the guest passes a NUL-terminated C string (guest==host src).
    let bytes = unsafe { CStr::from_ptr(guest) }.to_bytes();
    if bytes.is_empty() || bytes[0] != b'/' {
        return None; // relative (dirfd-relative) or empty -> leave alone
    }
    // Map the leading root by the longest-matching prefix so
    // `/storage/emulated/0` is handled before the shorter `/storage`.
    let candidates: [(&[u8], &str); 5] = [
        (b"/storage/emulated", "storage/emulated"),
        (b"/storage", "storage"),
        (b"/data", "data"),
        (b"/sdcard", "sdcard"),
        (b"/cache", "cache"),
    ];
    for (prefix, dir) in candidates {
        if bytes.starts_with(prefix) {
            let rest = &bytes[prefix.len()..];
            let host_rel = if rest.is_empty() {
                dir.to_string()
            } else if rest[0] == b'/' {
                // path.join with a leading-'/' would discard the root
                format!("{dir}{}", String::from_utf8_lossy(rest))
            } else {
                format!("{dir}/{}", String::from_utf8_lossy(rest))
            };
            let host = root.join(host_rel);
            let host_c = CString::new(host.as_os_str().as_bytes()).ok()?;
            let parent = parent_of(&host);
            return Some(RemappedPath {
                parent,
                host: host_c,
            });
        }
    }
    None
}

fn parent_of(p: &Path) -> Option<CString> {
    let par = p.parent()?;
    if par.as_os_str().is_empty() {
        return None;
    }
    CString::new(par.as_os_str().as_bytes()).ok()
}

/// Ensure the immediate parent directory of a remapped path exists (recursively),
/// so `openat(...,O_CREAT)` / `mkdirat` on a deep guest path never fails with
/// ENOENT just because the intermediate /data/user/0/com.roblox.client/... chain
/// has not been created yet. `create` is true only for O_CREAT-style opens.
pub fn ensure_parents(path: Option<&RemappedPath>, create: bool) {
    if !create {
        return;
    }
    if let Some(p) = path {
        if let Some(par) = p.parent.as_ref() {
            let _ = std::fs::create_dir_all(
                Path::new(OsStr::from_bytes(par.as_bytes())),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Idealize: fsmap-root override mutation serializes on the shared
    /// `test_root_mutex` (see fsmap.rs) — the same lock jit.rs + session.rs
    /// fsmap-root tests hold, so this module's tests can't race theirs.
    /// SH423: the durable persistence contract (objective 2b "remembers
    /// sign-in") is proven at the fsmap layer: a guest datastore path under a
    /// mapped Android root resolves to a REAL host file on disk, and a value
    /// written there survives a simulated restart (override cleared) and is
    /// readable back through a fresh, independent remap. This is the actual
    /// persistence hook the real client's session/cookie store lands on; it was
    /// exercised nowhere (jit.rs only harnesses fsmap for R1 CoreScript staging).
    #[test]
    fn sh423_durable_datastore_write_survives_remap_restart() {
        let _g = test_root_mutex().lock().unwrap();
        // A persistent HOST root that must outlive the writes (real disk dir).
        let root = std::env::temp_dir().join(format!("os-fsmap-durable-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        set_root_for_tests(root.clone());

        // The guest path the real client would open to persist its session
        // datastore (contacts fsmap's longest-prefix /data root).
        let guest = b"/data/user/0/com.roblox.client/databases/rbx-session.db\0";
        let rbp = remap_path(guest.as_ptr() as *const libc::c_char)
            .unwrap_or_else(|| panic!("guest datastore path must remap"));

        // Proving parent scaffolding: the deep /data/user/0/... chain must be
        // created so an O_CREAT-style open never ENOENTs (SH-fsmap contract).
        ensure_parents(Some(&rbp), true);
        let host = rbp.host_path().to_path_buf();
        assert!(
            host.starts_with(&root),
            "host path must stay under the configured root: {host:?}"
        );
        assert!(
            host.ends_with("data/user/0/com.roblox.client/databases/rbx-session.db"),
            "mapped path must preserve the guest dirs: {host:?}"
        );

        // Write the persisted value as the engine would (a real file on disk).
        const VALUE: &[u8] = b"open-sober|remembered-session|0x06cf";
        std::fs::write(&host, VALUE).unwrap();
        // The write must be ON DISK (present even with the in-memory override gone).
        assert!(
            std::fs::metadata(&host).is_ok() && std::fs::metadata(&host).unwrap().len() == VALUE.len() as u64,
            "persisted value must land as a real host file on disk"
        );

        // Simulate a RESTART: the runtime is a fresh process that re-derives
        // its root from env, so the in-memory override is entirely cleared
        // (back to None). The value must survive purely because it is a real
        // file already on disk under the root.
        *override_root().lock().unwrap() = None;
        // Re-arm the SAME on-disk root via a fresh override (fresh boot's root).
        set_root_for_tests(root.clone());

        // A FRESH, independent remap resolves the host file again and reads the
        // exact value back — the "remembers sign-in" round-trip.
        let rbp2 = remap_path(guest.as_ptr() as *const libc::c_char)
            .unwrap_or_else(|| panic!("re-remap of guest datastore path must resolve"));
        let read_back = std::fs::read(rbp2.host_path())
            .unwrap_or_else(|e| panic!("value must survive restart + re-remap: {e} path={:?}", rbp2.host_path()));
        assert_eq!(
            read_back, VALUE,
            "durable round-trip: written value must read back after simulated restart"
        );

        set_root_for_tests(PathBuf::new()); // clear override for later tests
        let _ = std::fs::remove_dir_all(&root);
    }

    /// SH423: the 5-root longest-prefix remap precedence — `/storage/emulated`
    /// must be handled before the shorter `/storage`, and only ABSOLUTE paths
    /// under a mapped writable root are rewritten; relative paths, nulls, and
    /// unmapped absolute roots (`/proc`, `/system`) pass through unchanged.
    #[test]
    fn sh423_remap_precedence_and_passthrough() {
        let _g = test_root_mutex().lock().unwrap();
        let root = std::env::temp_dir().join(format!("os-fsmap-prec-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        set_root_for_tests(root.clone());

        // Longest-prefix: /storage/emulated before /storage.
        let ge = b"/storage/emulated/0/Android/data/com.roblox.client\0";
        let re = remap_path(ge.as_ptr() as *const libc::c_char).unwrap();
        assert!(re.host_path().starts_with(&root.join("storage/emulated")),
            "/storage/emulated must map under storage/emulated: {:?}", re.host_path());
        // Plain /storage (shorter prefix) is still a distinct dir.
        let gs = b"/storage/foo\0";
        let rs = remap_path(gs.as_ptr() as *const libc::c_char).unwrap();
        assert!(rs.host_path().starts_with(&root.join("storage/foo")));

        // Unmapped absolute roots pass through.
        let gp = b"/proc/self/maps\0";
        assert!(remap_path(gp.as_ptr() as *const libc::c_char).is_none(),
            "/proc is not a writable Android root");
        let gsys = b"/system/build.prop\0";
        assert!(remap_path(gsys.as_ptr() as *const libc::c_char).is_none());

        // Relative + null pass through.
        assert!(remap_path(b"relative/path\0".as_ptr() as *const libc::c_char).is_none());
        assert!(remap_path(std::ptr::null::<libc::c_char>()).is_none());

        // No root configured -> inactive entirely (override cleared to None,
        // env unset in the harness -> active_root() is None).
        *override_root().lock().unwrap() = None;
        let gd = b"/data/foo\0";
        assert!(remap_path(gd.as_ptr() as *const libc::c_char).is_none(),
            "with no root, even /data passes through unchanged");

        let _ = std::fs::remove_dir_all(&root);
    }
}