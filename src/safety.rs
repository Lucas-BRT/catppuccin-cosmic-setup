//! Guards the commands that change the user's files: refuse to run as root,
//! run one at a time, and write files atomically.

use std::fs::{self, File};
use std::path::{Path, PathBuf};

use crate::apply::cache_dir;

/// Whether this process runs as root, from its effective uid.
fn is_root() -> bool {
    fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find_map(|l| l.strip_prefix("Uid:"))
                .and_then(|l| l.split_whitespace().nth(1).map(str::to_owned))
        })
        .is_some_and(|euid| euid == "0")
}

/// The installer configures one user's desktop. As root it would write into
/// root's home and leave the user's untouched.
pub fn refuse_root() -> Result<(), String> {
    if is_root() {
        return Err(
            "do not run this as root: it configures your own user's desktop (no sudo needed)"
                .into(),
        );
    }
    Ok(())
}

/// Held while a command that changes files runs; released when dropped (or
/// when the process exits).
pub struct Lock(#[allow(dead_code)] File);

/// Take the one-at-a-time lock, or fail if another run holds it.
pub fn lock() -> Result<Lock, String> {
    refuse_root()?;
    let dir = cache_dir()?;
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let path = dir.join("lock");
    let file = File::create(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    file.try_lock()
        .map_err(|_| "another ctp-cosmic run is in progress".to_string())?;
    Ok(Lock(file))
}

/// Replace `path` with `content` in one step: write next to it, then rename
/// over it, so an interruption never leaves a truncated file. A symlink is
/// followed, so it stays a symlink (dotfile managers), and the permissions
/// of the old file are kept.
pub fn write_atomic(path: &Path, content: &str) -> Result<(), String> {
    let err = |e: std::io::Error| format!("{}: {e}", path.display());
    let target: PathBuf = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let name = target
        .file_name()
        .ok_or_else(|| format!("{}: not a file path", path.display()))?
        .to_string_lossy()
        .into_owned();
    let tmp = target.with_file_name(format!(".{name}.ctp-tmp"));
    fs::write(&tmp, content).map_err(err)?;
    if let Ok(meta) = fs::metadata(&target) {
        let _ = fs::set_permissions(&tmp, meta.permissions());
    }
    fs::rename(&tmp, &target).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        err(e)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::{PermissionsExt, symlink};

    fn scratch(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("ctp-safety-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn atomic_write_keeps_symlink_and_mode() {
        let d = scratch("atomic");
        let real = d.join("dots/bashrc");
        fs::create_dir_all(real.parent().unwrap()).unwrap();
        fs::write(&real, "old").unwrap();
        fs::set_permissions(&real, fs::Permissions::from_mode(0o600)).unwrap();
        let link = d.join(".bashrc");
        symlink(&real, &link).unwrap();

        write_atomic(&link, "new").unwrap();

        assert!(
            fs::symlink_metadata(&link)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(fs::read_to_string(&real).unwrap(), "new");
        assert_eq!(
            fs::metadata(&real).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert!(!d.join("dots/.bashrc.ctp-tmp").exists());
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn atomic_write_creates_a_missing_file() {
        let d = scratch("create");
        write_atomic(&d.join("new"), "x").unwrap();
        assert_eq!(fs::read_to_string(d.join("new")).unwrap(), "x");
        fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn a_second_lock_fails_while_the_first_is_held() {
        let d = scratch("lock");
        let path = d.join("lock");
        let first = File::create(&path).unwrap();
        first.try_lock().unwrap();
        let second = File::create(&path).unwrap();
        assert!(second.try_lock().is_err());
        drop(first);
        assert!(second.try_lock().is_ok());
        fs::remove_dir_all(&d).unwrap();
    }
}
