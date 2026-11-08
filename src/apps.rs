//! `ctp-cosmic app`: Catppuccin for any app, from a declarative manifest.
//!
//! An app is a folder with an `app.toml` and the templates it names. Apps
//! shipped with ctp-cosmic are built in; yours go in
//! `~/.config/catppuccin-cosmic-setup/apps/<name>/` and win over a built-in
//! of the same name. Run one with `ctp-cosmic apply NAME`: it gets the same
//! backup, lock and `uninstall` as the other steps. See docs/APPS.md.
//!
//! A manifest only says which files to write and from what; it cannot run
//! commands, and every target stays inside $HOME.

use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::Deserialize;

use crate::apply::{home, load_state, state_dir};
use crate::palette::{self, Palette};
use crate::steps::STEPS;
use crate::{safety, ui};

const USAGE: &str = "\
ctp-cosmic app: Catppuccin for other apps

Usage:
  ctp-cosmic app list                        the apps you can theme
  ctp-cosmic app check NAME [--flavor F] [--accent A]
                                             whether its files are up to date
  ctp-cosmic app remove NAME                 take what it wrote out again

Apply one with `ctp-cosmic apply NAME`. Your own apps go in
~/.config/catppuccin-cosmic-setup/apps/NAME/app.toml (see docs/APPS.md).
";

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    name: String,
    label: Option<String>,
    /// Programs whose presence means the app is installed.
    #[serde(default)]
    detect: Vec<String>,
    /// `~/...` paths whose presence means the same (Flatpak, snap).
    #[serde(default)]
    detect_paths: Vec<String>,
    /// How to find where the files go: "firefox" (its default profile).
    /// Without it every `target` is a `~/...` path.
    locate: Option<String>,
    #[serde(rename = "file")]
    files: Vec<FileSpec>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FileSpec {
    target: String,
    template: String,
    mode: Mode,
    /// For `block`: the comment syntax, "#", "//" or "/* */".
    comment: Option<String>,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
enum Mode {
    /// The file is entirely ours.
    File,
    /// A marked block in a file that may hold your own content.
    Block,
}

pub struct App {
    m: Manifest,
    /// The template text of each `[[file]]`, in order.
    templates: Vec<String>,
    local: bool,
}

pub enum Outcome {
    Skipped(String),
    Waiting(String),
    Done(Vec<PathBuf>),
}

/// A path made only of plain names: no `..`, no root, not empty.
fn plain(rel: &str) -> bool {
    !rel.is_empty()
        && Path::new(rel)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}

fn under_home(home: &Path, tilde: &str) -> Result<PathBuf, String> {
    match tilde.strip_prefix("~/") {
        Some(rel) if plain(rel) => Ok(home.join(rel)),
        _ => Err(format!("'{tilde}' must be a ~/... path inside your home")),
    }
}

impl App {
    fn new(
        source: &str,
        text: &str,
        template: impl Fn(&str) -> Result<String, String>,
        local: bool,
    ) -> Result<App, String> {
        let m: Manifest = toml::from_str(text).map_err(|e| format!("{source}: {e}"))?;
        let bad = |msg: String| format!("{source}: {msg}");
        let name_ok = !m.name.is_empty()
            && m.name
                .starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
            && m.name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        if !name_ok {
            return Err(bad(format!(
                "name '{}' must be lowercase letters, digits and '-'",
                m.name
            )));
        }
        if STEPS.iter().any(|s| s.name == m.name) {
            return Err(bad(format!("'{}' is the name of a built-in step", m.name)));
        }
        if m.files.is_empty() {
            return Err(bad("no [[file]] entries".into()));
        }
        match m.locate.as_deref() {
            None | Some("firefox") => {}
            Some(other) => return Err(bad(format!("unknown locate '{other}' (firefox)"))),
        }
        for p in &m.detect_paths {
            under_home(Path::new("/"), p).map_err(bad)?;
        }
        let mut templates = Vec::new();
        for f in &m.files {
            match (&m.locate, plain(&f.target)) {
                (Some(_), false) => {
                    return Err(bad(format!(
                        "target '{}' must be a plain relative path",
                        f.target
                    )));
                }
                (None, _) => {
                    under_home(Path::new("/"), &f.target).map_err(bad)?;
                }
                _ => {}
            }
            match (f.mode, &f.comment) {
                (Mode::Block, None) => {
                    return Err(bad(format!("'{}': mode block needs a comment", f.target)));
                }
                (Mode::Block, Some(c)) if !(1..=2).contains(&c.split_whitespace().count()) => {
                    return Err(bad(format!(
                        "'{}': comment must be \"#\", \"//\" or \"/* */\"",
                        f.target
                    )));
                }
                (Mode::File, Some(_)) => {
                    return Err(bad(format!(
                        "'{}': comment is only for mode block",
                        f.target
                    )));
                }
                _ => {}
            }
            if !plain(&f.template) {
                return Err(bad(format!(
                    "template '{}' must be a plain relative path",
                    f.template
                )));
            }
            templates.push(template(&f.template).map_err(bad)?);
        }
        Ok(App {
            m,
            templates,
            local,
        })
    }

    pub fn label(&self) -> &str {
        self.m.label.as_deref().unwrap_or(&self.m.name)
    }

    fn installed(&self, home: &Path) -> bool {
        if self.m.detect.is_empty() && self.m.detect_paths.is_empty() {
            return true;
        }
        let path = std::env::var_os("PATH").unwrap_or_default();
        let dirs: Vec<PathBuf> = std::env::split_paths(&path)
            .chain([home.join(".local/bin")])
            .collect();
        self.m
            .detect
            .iter()
            .any(|b| dirs.iter().any(|d| d.join(b).is_file()))
            || self
                .m
                .detect_paths
                .iter()
                .any(|p| under_home(home, p).is_ok_and(|p| p.exists()))
    }

    /// Every file to write: (index of its `[[file]]`, absolute path). With
    /// `locate` only for profiles that exist; empty when there is none yet.
    fn targets(&self, home: &Path) -> Result<Vec<(usize, PathBuf)>, String> {
        let mut out = Vec::new();
        match self.m.locate.as_deref() {
            Some("firefox") => {
                for dir in firefox_profiles(home) {
                    out.extend(
                        self.m
                            .files
                            .iter()
                            .enumerate()
                            .map(|(i, f)| (i, dir.join(&f.target))),
                    );
                }
            }
            _ => {
                for (i, f) in self.m.files.iter().enumerate() {
                    out.push((i, under_home(home, &f.target)?));
                }
            }
        }
        Ok(out)
    }

    /// The files this app may write, relative to $HOME: what the backup
    /// watches before the run.
    pub fn watch(&self, home: &Path) -> Result<Vec<String>, String> {
        Ok(self
            .targets(home)?
            .into_iter()
            .filter_map(|(_, p)| p.strip_prefix(home).ok().map(|r| r.display().to_string()))
            .collect())
    }

    /// Things worth telling you after `apply`. For Firefox: a theme add-on
    /// that is active keeps its own colors for some elements and clashes with
    /// these.
    pub fn notices(&self, home: &Path) -> Vec<String> {
        if self.m.locate.as_deref() != Some("firefox") {
            return Vec::new();
        }
        firefox_profiles(home)
            .iter()
            .filter_map(|dir| fs::read_to_string(dir.join("prefs.js")).ok())
            .filter_map(|prefs| active_theme(&prefs))
            .map(|id| {
                format!(
                    "{}: the theme {id} is active and clashes with these colors; choose \"System theme\" under Add-ons > Themes",
                    self.label()
                )
            })
            .collect()
    }

    fn markers(&self, spec: &FileSpec) -> (String, String) {
        let mut c = spec.comment.as_deref().unwrap_or("#").split_whitespace();
        let (open, close) = (c.next().unwrap_or("#"), c.next());
        let tail = close.map(|c| format!(" {c}")).unwrap_or_default();
        (
            format!(
                "{open} >>> catppuccin-cosmic-setup:{} >>>{tail}",
                self.m.name
            ),
            format!(
                "{open} <<< catppuccin-cosmic-setup:{} <<<{tail}",
                self.m.name
            ),
        )
    }

    pub fn apply(&self, flavor: &str, accent: &str, home: &Path) -> Result<Outcome, String> {
        let p = palette::check(flavor, Some(accent))?;
        if !self.installed(home) {
            return Ok(Outcome::Skipped(format!(
                "{} not found, skipping",
                self.label()
            )));
        }
        let targets = self.targets(home)?;
        if targets.is_empty() {
            return Ok(Outcome::Waiting(format!(
                "no {0} profile yet: start {0} once, then run `ctp-cosmic apply {1}` again",
                self.label(),
                self.m.name
            )));
        }
        // Render everything first, so a bad template writes nothing.
        let rendered = targets
            .iter()
            .map(|(i, _)| {
                render(&self.templates[*i], p, flavor, accent)
                    .map_err(|e| format!("{}: {e}", self.m.name))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut done = Vec::new();
        for ((i, path), text) in targets.iter().zip(rendered) {
            let spec = &self.m.files[*i];
            let new = match spec.mode {
                Mode::File => text,
                Mode::Block => {
                    let (start, end) = self.markers(spec);
                    with_block(&read(path)?, &text, &start, &end)
                }
            };
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
            }
            safety::write_atomic(path, &new)?;
            done.push(path.clone());
        }
        Ok(Outcome::Done(done))
    }

    /// Each file and whether it already holds what `apply` would write.
    fn check(
        &self,
        flavor: &str,
        accent: &str,
        home: &Path,
    ) -> Result<Vec<(PathBuf, bool)>, String> {
        let p = palette::check(flavor, Some(accent))?;
        self.targets(home)?
            .into_iter()
            .map(|(i, path)| {
                let text = render(&self.templates[i], p, flavor, accent)?;
                let have = read(&path)?;
                let spec = &self.m.files[i];
                let current = match spec.mode {
                    Mode::File => have == text,
                    Mode::Block => {
                        let (s, e) = self.markers(spec);
                        with_block(&have, &text, &s, &e) == have
                    }
                };
                Ok((path, current))
            })
            .collect()
    }

    /// Take out what `apply` wrote: whole files are deleted, blocks removed
    /// (and a file that was only our block is deleted too).
    fn remove(&self, home: &Path) -> Result<usize, String> {
        let mut n = 0;
        for (i, path) in self.targets(home)? {
            if !path.exists() {
                continue;
            }
            let spec = &self.m.files[i];
            match spec.mode {
                Mode::File => {
                    fs::remove_file(&path).map_err(|e| format!("{}: {e}", path.display()))?
                }
                Mode::Block => {
                    let (s, e) = self.markers(spec);
                    let have = read(&path)?;
                    let rest = without_block(&have, &s, &e);
                    if rest == have {
                        continue;
                    }
                    if rest.trim().is_empty() {
                        fs::remove_file(&path).map_err(|e| format!("{}: {e}", path.display()))?;
                    } else {
                        safety::write_atomic(&path, &rest)?;
                    }
                }
            }
            n += 1;
        }
        Ok(n)
    }
}

fn read(path: &Path) -> Result<String, String> {
    match fs::read_to_string(path) {
        Ok(t) => Ok(t),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

/// `text` without the block between the marker lines (unchanged if the block
/// is missing or unfinished).
fn without_block(text: &str, start: &str, end: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let Some(a) = lines.iter().position(|l| l.trim() == start) else {
        return text.to_string();
    };
    let Some(b) = lines[a..]
        .iter()
        .position(|l| l.trim() == end)
        .map(|b| a + b)
    else {
        return text.to_string();
    };
    lines[..a]
        .iter()
        .chain(&lines[b + 1..])
        .map(|l| format!("{l}\n"))
        .collect()
}

/// `text` with the block replaced by a fresh one at the end.
fn with_block(text: &str, body: &str, start: &str, end: &str) -> String {
    let mut out = without_block(text, start, end);
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&format!("{start}\n{}\n{end}\n", body.trim_end()));
    out
}

/// Fill in the placeholders: `@base@` is `#1e1e2e`, `@base:rgb@` is
/// `30, 30, 46`, `@base:bare@` is `1e1e2e`; any color name works, and
/// `accent` is the chosen accent. Also `@accent_name@`, `@flavor@`,
/// `@flavor_name@` and `@scheme@` (dark or light). Anything else between
/// two `@` is an error, so a typo is not written out silently.
fn render(text: &str, p: Palette, flavor: &str, accent: &str) -> Result<String, String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find('@') {
        out.push_str(&rest[..at]);
        let after = &rest[at + 1..];
        let len = after
            .bytes()
            .take_while(|b| {
                b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b':')
            })
            .count();
        if len > 0 && after.as_bytes().get(len) == Some(&b'@') {
            out.push_str(&value(&after[..len], p, flavor, accent)?);
            rest = &after[len + 1..];
        } else {
            out.push('@');
            rest = after;
        }
    }
    out.push_str(rest);
    Ok(out)
}

fn value(key: &str, p: Palette, flavor: &str, accent: &str) -> Result<String, String> {
    match key {
        "flavor" => return Ok(flavor.into()),
        "flavor_name" => return Ok(p.name()),
        "accent_name" => return Ok(accent.into()),
        "scheme" => return Ok(if p.is_dark() { "dark" } else { "light" }.into()),
        _ => {}
    }
    let (name, form) = key.split_once(':').unwrap_or((key, "hex"));
    let name = if name == "accent" { accent } else { name };
    let [r, g, b] = p
        .try_get(name)
        .ok_or_else(|| format!("unknown placeholder @{key}@"))?;
    let (r, g, b) = (r as u8, g as u8, b as u8);
    match form {
        "hex" => Ok(format!("#{r:02x}{g:02x}{b:02x}")),
        "bare" => Ok(format!("{r:02x}{g:02x}{b:02x}")),
        "rgb" => Ok(format!("{r}, {g}, {b}")),
        _ => Err(format!(
            "unknown placeholder @{key}@ (forms: hex, bare, rgb)"
        )),
    }
}

/// The theme add-on selected in a profile's `prefs.js`, unless it is
/// Firefox's own "System theme" (or the preference is missing).
fn active_theme(prefs: &str) -> Option<String> {
    let line = prefs
        .lines()
        .find(|l| l.contains("\"extensions.activeThemeID\""))?;
    let id = line.rsplit('"').nth(1)?;
    (id != "default-theme@mozilla.org").then(|| id.to_string())
}

/// The profile(s) Firefox marks as default, in every place it can live
/// (regular, Flatpak, snap), from each `profiles.ini`.
fn firefox_profiles(home: &Path) -> Vec<PathBuf> {
    let roots = [
        // Firefox 147 and newer follow the XDG layout; older ones use ~/.mozilla.
        ".config/mozilla/firefox",
        ".mozilla/firefox",
        // Flatpak and snap keep their own copy. The `config/` and `.config/`
        // ones are the XDG layout, which Flatpak and snap are expected to use
        // from Firefox 147; not checked against a real install. Only folders
        // that exist are used, so a wrong guess costs nothing.
        ".var/app/org.mozilla.firefox/.mozilla/firefox",
        ".var/app/org.mozilla.firefox/config/mozilla/firefox",
        "snap/firefox/common/.mozilla/firefox",
        "snap/firefox/common/.config/mozilla/firefox",
    ];
    let mut out: Vec<PathBuf> = Vec::new();
    for root in roots.map(|r| home.join(r)) {
        let Ok(ini) = fs::read_to_string(root.join("profiles.ini")) else {
            continue;
        };
        for p in profiles_in(&root, &ini) {
            if p.is_dir() && !out.contains(&p) {
                out.push(p);
            }
        }
    }
    out
}

/// The default profile paths of one `profiles.ini`: the `[Install…]`
/// sections' `Default`, else the `[Profile…]` marked `Default=1`.
fn profiles_in(root: &Path, ini: &str) -> Vec<PathBuf> {
    let mut sections: Vec<(String, Vec<(String, String)>)> = Vec::new();
    for line in ini.lines().map(str::trim) {
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            sections.push((name.to_string(), Vec::new()));
        } else if let (Some((k, v)), Some(last)) = (line.split_once('='), sections.last_mut()) {
            last.1.push((k.trim().to_string(), v.trim().to_string()));
        }
    }
    let get = |kv: &[(String, String)], key: &str| {
        kv.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone())
    };
    let from_root = |p: String, relative: bool| {
        if relative {
            root.join(p)
        } else {
            PathBuf::from(p)
        }
    };

    let installs: Vec<PathBuf> = sections
        .iter()
        .filter(|(n, _)| n.starts_with("Install"))
        .filter_map(|(_, kv)| get(kv, "Default"))
        .map(|p| from_root(p, true))
        .collect();
    if !installs.is_empty() {
        return installs;
    }
    sections
        .iter()
        .filter(|(n, kv)| n.starts_with("Profile") && get(kv, "Default").as_deref() == Some("1"))
        .filter_map(|(_, kv)| {
            let relative = get(kv, "IsRelative").as_deref() != Some("0");
            get(kv, "Path").map(|p| from_root(p, relative))
        })
        .collect()
}

struct Builtin {
    toml: &'static str,
    files: &'static [(&'static str, &'static str)],
}

const BUILTIN: [Builtin; 1] = [Builtin {
    toml: include_str!("../apps/firefox/app.toml"),
    files: &[
        (
            "userChrome.css",
            include_str!("../apps/firefox/userChrome.css"),
        ),
        (
            "userContent.css",
            include_str!("../apps/firefox/userContent.css"),
        ),
        ("user.js", include_str!("../apps/firefox/user.js")),
    ],
}];

/// Where your own apps live.
fn local_dir() -> Result<PathBuf, String> {
    Ok(state_dir()?.join("apps"))
}

/// Every app: the built-in ones, then yours (a same name replaces the
/// built-in). A broken app of yours is reported, not fatal.
fn load_all() -> Result<(Vec<App>, Vec<String>), String> {
    let mut apps = Vec::new();
    for b in &BUILTIN {
        let template = |name: &str| {
            b.files
                .iter()
                .find(|(n, _)| *n == name)
                .map(|(_, t)| t.to_string())
                .ok_or_else(|| format!("no template '{name}'"))
        };
        apps.push(App::new("built-in app", b.toml, template, false)?);
    }
    let mut problems = Vec::new();
    let dir = local_dir()?;
    let mut found: Vec<PathBuf> = fs::read_dir(&dir)
        .map(|d| d.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();
    found.sort();
    for folder in found.into_iter().filter(|p| p.join("app.toml").is_file()) {
        let manifest = folder.join("app.toml");
        let loaded = fs::read_to_string(&manifest)
            .map_err(|e| format!("{}: {e}", manifest.display()))
            .and_then(|text| {
                let template = |name: &str| {
                    let f = folder.join(name);
                    fs::read_to_string(&f).map_err(|e| format!("{}: {e}", f.display()))
                };
                App::new(&manifest.display().to_string(), &text, template, true)
            });
        match loaded {
            Ok(app) => {
                apps.retain(|a| a.m.name != app.m.name);
                apps.push(app);
            }
            Err(e) => problems.push(e),
        }
    }
    Ok((apps, problems))
}

/// The app called `name`, if there is one.
pub fn find(name: &str) -> Result<Option<App>, String> {
    Ok(load_all()?.0.into_iter().find(|a| a.m.name == name))
}

/// Whether `name` is an app (for `apply` to accept it as a step).
pub fn exists(name: &str) -> bool {
    find(name).ok().flatten().is_some()
}

/// (name, label) of every app, read once: `apply --list` and the interactive
/// setup both list them.
pub fn catalog() -> &'static [(String, String)] {
    static CATALOG: std::sync::OnceLock<Vec<(String, String)>> = std::sync::OnceLock::new();
    CATALOG.get_or_init(|| {
        load_all()
            .map(|(apps, _)| {
                apps.into_iter()
                    .map(|a| (a.m.name.clone(), a.label().to_string()))
                    .collect()
            })
            .unwrap_or_default()
    })
}

fn need(name: &str) -> Result<App, String> {
    find(name)?.ok_or_else(|| format!("unknown app '{name}' (see: ctp-cosmic app list)"))
}

fn list() -> Result<(), String> {
    let home = home()?;
    let (apps, problems) = load_all()?;
    for a in &apps {
        println!(
            "{:<12} {:<14} {:<9} {}",
            a.m.name,
            a.label(),
            if a.local { "yours" } else { "built-in" },
            if a.installed(&home) {
                "installed"
            } else {
                "not found"
            }
        );
    }
    problems
        .iter()
        .for_each(|p| ui::warn(&format!("skipped: {p}")));
    println!("\nYour own apps: {}", local_dir()?.display());
    Ok(())
}

pub fn main(args: Vec<String>) -> Result<u8, String> {
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args.as_slice() {
        ["list"] => list()?,
        ["remove", name] => {
            let _lock = safety::lock()?;
            let n = need(name)?.remove(&home()?)?;
            ui::ok(&format!("{n} file(s) cleaned"));
        }
        ["check", name, rest @ ..] => {
            let last = load_state().ok();
            let (mut flavor, mut accent) = last
                .map(|o| (o.flavor, o.accent))
                .unwrap_or(("mocha".into(), "mauve".into()));
            let mut it = rest.iter();
            while let Some(flag) = it.next() {
                let v = it
                    .next()
                    .ok_or(format!("{flag} needs a value"))?
                    .to_string();
                match *flag {
                    "--flavor" => flavor = v,
                    "--accent" => accent = v,
                    other => return Err(format!("unexpected argument '{other}'\n\n{USAGE}")),
                }
            }
            let app = need(name)?;
            let files = app.check(&flavor, &accent, &home()?)?;
            if files.is_empty() {
                ui::warn("nothing to check yet (no profile or files)");
                return Ok(1);
            }
            app.notices(&home()?).iter().for_each(|n| ui::warn(n));
            for (path, current) in &files {
                println!(
                    "{} {}",
                    if *current { "ok     " } else { "differs" },
                    path.display()
                );
            }
            return Ok(u8::from(files.iter().any(|(_, c)| !c)));
        }
        ["-h" | "--help"] | [] => print!("{USAGE}"),
        _ => return Err(format!("invalid arguments\n\n{USAGE}")),
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mocha() -> Palette {
        Palette::new("mocha").unwrap()
    }

    fn scratch(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("ctp-apps-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    fn app(manifest: &str, template: &str) -> Result<App, String> {
        App::new("test", manifest, |_| Ok(template.to_string()), true)
    }

    #[test]
    fn built_in_apps_are_valid() {
        let (apps, problems) = load_all().unwrap();
        assert!(apps.iter().any(|a| a.m.name == "firefox"));
        assert!(problems.is_empty() || problems.iter().all(|p| !p.contains("built-in")));
        // Every template renders in every flavor and accent.
        for flavor in palette::flavors() {
            let p = Palette::new(flavor).unwrap();
            for accent in palette::accents() {
                for a in &apps {
                    for t in &a.templates {
                        render(t, p, flavor, accent).unwrap();
                    }
                }
            }
        }
    }

    #[test]
    fn placeholders() {
        let r = |t: &str| render(t, mocha(), "mocha", "mauve");
        assert_eq!(r("a @base@ b").unwrap(), "a #1e1e2e b");
        assert_eq!(r("@base:rgb@").unwrap(), "30, 30, 46");
        assert_eq!(r("@accent:bare@ @accent_name@").unwrap(), "cba6f7 mauve");
        assert_eq!(
            r("@flavor@ @flavor_name@ @scheme@").unwrap(),
            "mocha Mocha dark"
        );
        // Things that are not placeholders stay as they are.
        assert_eq!(
            r("@-moz-document me@host @media (x) @@").unwrap(),
            "@-moz-document me@host @media (x) @@"
        );
        assert!(r("@nope@").is_err());
        assert!(r("@base:wat@").is_err());
    }

    #[test]
    fn manifests_are_checked() {
        let ok = "name = \"x\"\n[[file]]\ntarget = \"~/.config/x/theme\"\ntemplate = \"t\"\nmode = \"file\"\n";
        assert!(app(ok, "").is_ok());
        let bad = |s: &str| {
            assert!(
                app(
                    &ok.replace(s.split('|').next().unwrap(), s.split('|').nth(1).unwrap()),
                    ""
                )
                .is_err(),
                "{s}"
            )
        };
        bad("~/.config/x/theme|~/../etc/passwd");
        bad("~/.config/x/theme|/etc/passwd");
        bad("~/.config/x/theme|.config/x");
        bad("\"t\"|\"../t\"");
        bad("name = \"x\"|name = \"cosmic\"");
        bad("name = \"x\"|name = \"X y\"");
        bad("mode = \"file\"|mode = \"block\"");
        bad("mode = \"file\"|mode = \"run\"");
        assert!(app(&format!("{ok}hook = \"rm -rf ~\"\n"), "").is_err());
        assert!(app(&format!("run = \"x\"\n{ok}"), "").is_err());
    }

    #[test]
    fn blocks_keep_your_content_and_are_idempotent() {
        let (s, e) = ("// >>> a >>>", "// <<< a <<<");
        let once = with_block("mine = 1;\n", "x", s, e);
        assert_eq!(once, "mine = 1;\n// >>> a >>>\nx\n// <<< a <<<\n");
        assert_eq!(with_block(&once, "x", s, e), once);
        assert_eq!(
            with_block(&once, "y", s, e),
            "mine = 1;\n// >>> a >>>\ny\n// <<< a <<<\n"
        );
        assert_eq!(without_block(&once, s, e), "mine = 1;\n");
        // An unfinished block is left alone rather than eating the file.
        assert_eq!(
            without_block("a\n// >>> a >>>\nb\n", s, e),
            "a\n// >>> a >>>\nb\n"
        );
    }

    #[test]
    fn firefox_default_profile_is_found() {
        let root = Path::new("/r");
        let ini = "[Profile0]\nName=default\nIsRelative=1\nPath=old.default\n\n\
                   [Profile1]\nName=default-release\nIsRelative=1\nPath=abc.default-release\nDefault=1\n\n\
                   [Install4F96D1932A9F858E]\nDefault=abc.default-release\nLocked=1\n";
        assert_eq!(
            profiles_in(root, ini),
            [Path::new("/r/abc.default-release")]
        );
        // Without an [Install] section, the profile marked Default=1; absolute paths stay.
        let ini = "[Profile0]\nIsRelative=0\nPath=/data/ff\nDefault=1\n[Profile1]\nPath=other\n";
        assert_eq!(profiles_in(root, ini), [Path::new("/data/ff")]);
        assert!(profiles_in(root, "").is_empty());
    }

    #[test]
    fn apply_check_and_remove_a_located_app() {
        let home = scratch("firefox");
        let profile = home.join(".mozilla/firefox/abc.default-release");
        fs::create_dir_all(&profile).unwrap();
        fs::write(
            home.join(".mozilla/firefox/profiles.ini"),
            "[Install1]\nDefault=abc.default-release\n",
        )
        .unwrap();
        fs::create_dir_all(profile.join("chrome")).unwrap();
        fs::write(profile.join("user.js"), "user_pref(\"mine\", 1);\n").unwrap();

        let ff = find("firefox").unwrap().unwrap();
        // Detected by `detect_paths` in a throwaway home.
        fs::create_dir_all(home.join("snap/firefox")).unwrap();

        let Outcome::Done(files) = ff.apply("mocha", "mauve", &home).unwrap() else {
            panic!("expected Done");
        };
        assert_eq!(files.len(), 3);
        let css = fs::read_to_string(profile.join("chrome/userChrome.css")).unwrap();
        assert!(css.contains("--toolbar-background-color: #1e1e2e") && css.contains("#cba6f7"));
        let js = fs::read_to_string(profile.join("user.js")).unwrap();
        assert!(
            js.starts_with("user_pref(\"mine\", 1);\n")
                && js.contains("legacyUserProfileCustomizations")
        );
        assert!(
            ff.check("mocha", "mauve", &home)
                .unwrap()
                .iter()
                .all(|(_, ok)| *ok)
        );
        assert!(
            ff.check("latte", "mauve", &home)
                .unwrap()
                .iter()
                .any(|(_, ok)| !ok)
        );
        assert!(
            ff.watch(&home)
                .unwrap()
                .contains(&".mozilla/firefox/abc.default-release/user.js".to_string())
        );

        // Running again changes nothing; a new flavor replaces the block, not appends.
        ff.apply("mocha", "mauve", &home).unwrap();
        ff.apply("latte", "blue", &home).unwrap();
        let js = fs::read_to_string(profile.join("user.js")).unwrap();
        assert_eq!(js.matches(">>> catppuccin-cosmic-setup:firefox").count(), 1);

        assert_eq!(ff.remove(&home).unwrap(), 3);
        assert_eq!(
            fs::read_to_string(profile.join("user.js")).unwrap(),
            "user_pref(\"mine\", 1);\n"
        );
        assert!(!profile.join("chrome/userChrome.css").exists());
        fs::remove_dir_all(&home).unwrap();
    }

    #[test]
    fn firefox_profile_is_found_in_the_xdg_folder() {
        let home = scratch("xdg");
        let root = home.join(".config/mozilla/firefox");
        fs::create_dir_all(root.join("9f9kx489.default-release")).unwrap();
        fs::write(
            root.join("profiles.ini"),
            "[Profile0]\nPath=9f9kx489.default-release\n[Install4F96D1932A9F858E]\nDefault=9f9kx489.default-release\n",
        )
        .unwrap();
        assert_eq!(
            firefox_profiles(&home),
            [root.join("9f9kx489.default-release")]
        );
        fs::remove_dir_all(&home).unwrap();
    }

    #[test]
    fn a_theme_addon_is_noticed() {
        let on = "user_pref(\"extensions.activeThemeID\", \"{f5525f34-4102}\");\n";
        assert_eq!(active_theme(on).as_deref(), Some("{f5525f34-4102}"));
        let off = "user_pref(\"extensions.activeThemeID\", \"default-theme@mozilla.org\");\n";
        assert_eq!(active_theme(off), None);
        assert_eq!(active_theme("user_pref(\"other\", 1);\n"), None);

        let home = scratch("addon");
        let profile = home.join(".config/mozilla/firefox/p.default");
        fs::create_dir_all(&profile).unwrap();
        fs::write(
            home.join(".config/mozilla/firefox/profiles.ini"),
            "[Install1]\nDefault=p.default\n",
        )
        .unwrap();
        let ff = find("firefox").unwrap().unwrap();
        assert!(ff.notices(&home).is_empty());
        fs::write(profile.join("prefs.js"), on).unwrap();
        assert!(ff.notices(&home)[0].contains("{f5525f34-4102}"));
        fs::remove_dir_all(&home).unwrap();
    }

    #[test]
    fn no_profile_yet_waits() {
        let home = scratch("noprofile");
        fs::create_dir_all(home.join("snap/firefox")).unwrap();
        let ff = find("firefox").unwrap().unwrap();
        assert!(matches!(
            ff.apply("mocha", "mauve", &home).unwrap(),
            Outcome::Waiting(_)
        ));
        fs::remove_dir_all(&home).unwrap();
    }
}
