//! `ctp-cosmic update`: replace this binary with the latest release, checked
//! against the release's SHA256SUMS. Uses curl and sha256sum, like install.sh.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

const REPO: &str = "Lucas-BRT/catppuccin-cosmic-setup";
const USAGE: &str = "\
ctp-cosmic update: install the latest release over this binary

Usage:
  ctp-cosmic update [--check] [--force] [--apply] [--gui]

--check   only say whether a newer release exists
--force   reinstall even if this is already the latest
--apply   afterwards, repeat your last choices (ctp-cosmic apply --last)
--gui     only install or update the graphical app (ctp-cosmic-gui), next to
          this binary, with its launcher entry

The graphical app is updated too when it is installed next to this binary.
";

const GUI: &str = "ctp-cosmic-gui";
/// The launcher entry and icon of the app, as the release names them.
const APP_ID: &str = "io.github.Lucas_BRT.CtpCosmic";

/// "v0.4.1" or "0.4.1-dev" as (0, 4, 1).
fn parse_version(s: &str) -> Option<(u32, u32, u32)> {
    let core = s.trim().trim_start_matches('v').split('-').next()?;
    let mut n = core.split('.').map(|p| p.parse::<u32>().ok());
    Some((n.next()??, n.next()??, n.next()??))
}

/// The tag from the URL GitHub redirects /releases/latest to.
fn tag_from_url(url: &str) -> Option<&str> {
    url.trim().rsplit_once("/tag/").map(|(_, tag)| tag)
}

/// The hash `sha256sum` printed for `name` in a SHA256SUMS file.
fn hash_for<'a>(sums: &'a str, name: &str) -> Option<&'a str> {
    sums.lines().find_map(|l| {
        let (hash, file) = l.split_once(char::is_whitespace)?;
        (file.trim().trim_start_matches('*') == name).then_some(hash)
    })
}

fn curl(args: &[&str]) -> Result<Vec<u8>, String> {
    let out = Command::new("curl")
        .args(["-fsSL", "--proto", "=https", "--tlsv1.2"])
        .args(args)
        .output()
        .map_err(|e| format!("curl: {e}"))?;
    if out.status.success() {
        Ok(out.stdout)
    } else {
        Err(format!("download failed: {}", args.last().unwrap_or(&"")))
    }
}

fn latest_tag() -> Result<String, String> {
    let url = format!("https://github.com/{REPO}/releases/latest");
    let out = curl(&["-I", "-o", "/dev/null", "-w", "%{url_effective}", &url])?;
    let out = String::from_utf8_lossy(&out).into_owned();
    tag_from_url(&out)
        .map(str::to_string)
        .ok_or_else(|| format!("no release found at {url}"))
}

fn sha256(path: &Path) -> Result<String, String> {
    let out = Command::new("sha256sum")
        .arg(path)
        .output()
        .map_err(|e| format!("sha256sum: {e}"))?;
    String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .next()
        .map(str::to_string)
        .ok_or_else(|| "sha256sum printed nothing".to_string())
}

/// Downloads `asset` from the release at `base`, checks it against `sums`
/// and puts it in place of `dest` (next to it first, so the rename stays on
/// one filesystem and a running program is never overwritten in place).
fn replace(
    base: &str,
    sums: &str,
    asset: &str,
    tag: &str,
    dest: &Path,
    mode: u32,
) -> Result<(), String> {
    let want =
        hash_for(sums, asset).ok_or_else(|| format!("{asset} is not in {tag}'s SHA256SUMS"))?;
    if let Some(dir) = dest.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let name = dest.file_name().unwrap_or_default().to_string_lossy();
    let part = dest.with_file_name(format!(".{name}.part"));
    let fetched = curl(&[
        "-o",
        &part.display().to_string(),
        &format!("{base}/{asset}"),
    ]);
    let verified = fetched.and_then(|_| {
        if sha256(&part)? == want {
            Ok(())
        } else {
            Err(format!(
                "{asset} does not match {tag}'s SHA256SUMS; refusing to install it"
            ))
        }
    });
    if let Err(e) = verified {
        let _ = fs::remove_file(&part);
        return Err(e);
    }
    fs::set_permissions(&part, fs::Permissions::from_mode(mode)).map_err(|e| e.to_string())?;
    fs::rename(&part, dest).map_err(|e| format!("{}: {e}", dest.display()))
}

/// Installs or updates the graphical app next to `exe`; with `fresh`, its
/// launcher entry and icon too. A release without the app is only mentioned.
fn update_gui(base: &str, sums: &str, tag: &str, exe: &Path, fresh: bool) -> Result<(), String> {
    let asset = format!("{GUI}-{}-linux", std::env::consts::ARCH);
    if hash_for(sums, &asset).is_none() {
        println!("{tag} has no graphical app for this machine");
        return Ok(());
    }
    replace(base, sums, &asset, tag, &exe.with_file_name(GUI), 0o755)?;
    println!("{GUI} -> {tag}");
    if fresh {
        let data = std::env::var_os("XDG_DATA_HOME")
            .map(std::path::PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".local/share"))
            })
            .ok_or("HOME not set")?;
        for (file, dir) in [
            (format!("{APP_ID}.desktop"), "applications"),
            (format!("{APP_ID}.svg"), "icons/hicolor/scalable/apps"),
        ] {
            replace(base, sums, &file, tag, &data.join(dir).join(&file), 0o644)?;
        }
    }
    Ok(())
}

pub fn main(args: Vec<String>) -> Result<u8, String> {
    let (mut check, mut force, mut apply, mut gui_only) = (false, false, false, false);
    for a in &args {
        match a.as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                return Ok(0);
            }
            "--check" => check = true,
            "--force" => force = true,
            "--apply" => apply = true,
            "--gui" => gui_only = true,
            other => return Err(format!("unexpected argument '{other}'\n\n{USAGE}")),
        }
    }
    let current = env!("CARGO_PKG_VERSION");
    let tag = latest_tag()?;
    let newer = parse_version(&tag) > parse_version(current);
    if !gui_only && !newer && !force {
        println!("ctp-cosmic {current} is up to date (latest release: {tag})");
        return Ok(0);
    }
    if check {
        println!("ctp-cosmic {tag} is available (you have {current}): ctp-cosmic update");
        return Ok(0);
    }

    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    if exe.components().any(|c| c.as_os_str() == "target") {
        return Err(
            "this is a cargo build, not an installed binary; update replaces the installed one"
                .into(),
        );
    }
    let arch = std::env::consts::ARCH;
    let base = format!("https://github.com/{REPO}/releases/download/{tag}");
    let sums = String::from_utf8_lossy(&curl(&[&format!("{base}/SHA256SUMS")])?).into_owned();
    if !gui_only {
        replace(
            &base,
            &sums,
            &format!("ctp-cosmic-{arch}-linux"),
            &tag,
            &exe,
            0o755,
        )?;
        println!("ctp-cosmic {current} -> {tag}");
    }
    let installed = exe.with_file_name(GUI).is_file();
    if gui_only || installed {
        update_gui(&base, &sums, &tag, &exe, gui_only && !installed)?;
    }
    if gui_only {
        return Ok(0);
    }

    if apply {
        let status = Command::new(&exe)
            .args(["apply", "--last"])
            .status()
            .map_err(|e| e.to_string())?;
        return Ok(status.code().unwrap_or(1) as u8);
    }
    println!("To repeat your last choices with the new version: ctp-cosmic apply --last");
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_numerically() {
        assert_eq!(parse_version("v0.10.2"), Some((0, 10, 2)));
        assert_eq!(parse_version("0.4.0-dev"), Some((0, 4, 0)));
        assert_eq!(parse_version("nightly"), None);
        assert!(parse_version("v0.10.0") > parse_version("0.9.9"));
        assert!(parse_version("v0.4.0") <= parse_version("0.4.0"));
        assert!(parse_version("junk") < parse_version("0.0.1"));
    }

    #[test]
    fn tag_and_hash_are_picked_out() {
        let url = "https://github.com/Lucas-BRT/catppuccin-cosmic-setup/releases/tag/v0.5.0";
        assert_eq!(tag_from_url(url), Some("v0.5.0"));
        assert_eq!(tag_from_url("https://github.com/x/releases"), None);
        let sums = "aaa111  ctp-cosmic-x86_64-linux\nbbb222 *ctp-cosmic-aarch64-linux\nccc333  install.sh\n";
        assert_eq!(hash_for(sums, "ctp-cosmic-x86_64-linux"), Some("aaa111"));
        assert_eq!(hash_for(sums, "ctp-cosmic-aarch64-linux"), Some("bbb222"));
        assert_eq!(hash_for(sums, "other"), None);
    }
}
