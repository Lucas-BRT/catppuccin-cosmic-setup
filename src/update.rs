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
  ctp-cosmic update [--check] [--force] [--apply]

--check   only say whether a newer release exists
--force   reinstall even if this is already the latest
--apply   afterwards, repeat your last choices (ctp-cosmic apply --last)
";

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

pub fn main(args: Vec<String>) -> Result<u8, String> {
    let (mut check, mut force, mut apply) = (false, false, false);
    for a in &args {
        match a.as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                return Ok(0);
            }
            "--check" => check = true,
            "--force" => force = true,
            "--apply" => apply = true,
            other => return Err(format!("unexpected argument '{other}'\n\n{USAGE}")),
        }
    }
    let current = env!("CARGO_PKG_VERSION");
    let tag = latest_tag()?;
    let newer = parse_version(&tag) > parse_version(current);
    if !newer && !force {
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
    let asset = format!("ctp-cosmic-{arch}-linux");
    let base = format!("https://github.com/{REPO}/releases/download/{tag}");
    let sums = String::from_utf8_lossy(&curl(&[&format!("{base}/SHA256SUMS")])?).into_owned();
    let want =
        hash_for(&sums, &asset).ok_or_else(|| format!("{asset} is not in {tag}'s SHA256SUMS"))?;

    // Next to the binary, so the final rename stays on one filesystem and a
    // running ctp-cosmic is never overwritten in place.
    let part = exe.with_file_name(".ctp-cosmic.part");
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
    fs::set_permissions(&part, fs::Permissions::from_mode(0o755)).map_err(|e| e.to_string())?;
    fs::rename(&part, &exe).map_err(|e| format!("{}: {e}", exe.display()))?;
    println!("ctp-cosmic {current} -> {tag}");

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
