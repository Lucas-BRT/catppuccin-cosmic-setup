//! TUI state and key handling, kept free of terminal I/O so it can be tested.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::palette;
pub use crate::steps::STEPS;
use crate::styles;

/// FROSTED values: keep the current setting, off, or a blur strength.
pub const FROSTED: [&str; 16] = [
    "auto",
    "off",
    "extremely_low",
    "extremely_low_2",
    "very_low",
    "very_low_2",
    "low",
    "low_2",
    "medium",
    "medium_2",
    "high",
    "high_2",
    "very_high",
    "very_high_2",
    "extremely_high",
    "extremely_high_2",
];

/// COSMIC's light mode flavor: off, or Latte with automatic switching.
pub const LIGHT: [&str; 2] = ["off", "latte"];

/// First entry of `Config::profiles`.
pub const NO_PROFILE: &str = "none";

/// Corner roundness of the desktop theme.
pub const ROUNDNESS: [&str; 3] = ["round", "slightlyround", "square"];

/// Slideshow intervals COSMIC Settings offers (only with every style).
pub const ROTATE: [&str; 7] = ["off", "5m", "10m", "15m", "30m", "1h", "2h"];

/// Start of the font entries that download a Nerd Font first.
pub const DOWNLOAD: &str = "+ ";

/// First entry of `Config::fonts`.
pub const KEEP_FONT: &str = "keep current";

/// The rows of the configuration screen, top to bottom.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Field {
    Profile,
    SaveAs,
    Flavor,
    Light,
    Accent,
    Frosted,
    Roundness,
    Wallpaper,
    Slideshow,
    Font,
    Step(usize),
    Install,
}

pub fn fields() -> Vec<Field> {
    let mut f = vec![
        Field::Profile,
        Field::SaveAs,
        Field::Flavor,
        Field::Light,
        Field::Accent,
        Field::Frosted,
        Field::Roundness,
        Field::Wallpaper,
        Field::Slideshow,
        Field::Font,
    ];
    f.extend((0..STEPS.len()).map(Field::Step));
    f.push(Field::Install);
    f
}

/// What the user chose, turned into `apply` arguments and environment.
#[derive(Clone, Debug)]
pub struct Config {
    pub flavor: usize,
    /// Index into `LIGHT`; only with a dark flavor.
    pub light: usize,
    /// Index into `profiles`; choosing one loads its values.
    pub profile: usize,
    pub profiles: Vec<String>,
    /// Name typed under "Save as": the choices are saved as a profile after a successful run.
    pub save_as: String,
    pub accent: usize,
    pub frosted: usize,
    pub roundness: usize,
    /// Index into `wallpaper_choices()` (the styles, then "all").
    pub style: usize,
    pub rotate: usize,
    /// Index into `fonts`; 0 keeps the terminal's current font.
    pub font: usize,
    pub fonts: Vec<String>,
    pub steps: [bool; STEPS.len()],
}

impl Default for Config {
    fn default() -> Self {
        let flavors = palette::flavors();
        let accents = palette::accents();
        Self {
            flavor: flavors.iter().position(|f| *f == "mocha").unwrap_or(0),
            light: 0,
            profile: 0,
            profiles: vec![NO_PROFILE.to_string()],
            save_as: String::new(),
            accent: accents.iter().position(|a| *a == "mauve").unwrap_or(0),
            frosted: 0,
            roundness: 0,
            style: 0,
            rotate: 0,
            font: 0,
            fonts: vec![KEEP_FONT.to_string()],
            // Optional steps (icons) stay off unless asked for.
            steps: STEPS.each_ref().map(|s| !s.optional),
        }
    }
}

impl Config {
    /// The configuration screen set to previously saved choices.
    pub fn from_opts(o: &crate::apply::Opts, mut fonts: Vec<String>) -> Self {
        fonts.insert(0, KEEP_FONT.to_string());
        let find = |list: &[&str], v: &str| list.iter().position(|x| *x == v).unwrap_or(0);
        let rotate = o.rotate.as_deref().map_or(0, |r| find(&ROTATE, r));
        Self {
            flavor: find(&palette::flavors(), &o.flavor),
            light: usize::from(o.light_flavor.is_some()),
            profile: 0,
            profiles: vec![NO_PROFILE.to_string()],
            save_as: String::new(),
            accent: find(&palette::accents(), &o.accent),
            frosted: find(&FROSTED, &o.frosted),
            roundness: find(&ROUNDNESS, &o.roundness),
            style: find(&wallpaper_choices(), &o.style),
            rotate,
            font: o
                .font
                .as_ref()
                .and_then(|f| fonts.iter().position(|x| x == f))
                .unwrap_or(0),
            fonts,
            steps: STEPS
                .each_ref()
                .map(|s| o.steps.iter().any(|n| n == s.name)),
        }
    }
}

pub fn wallpaper_choices() -> Vec<&'static str> {
    let mut v: Vec<&'static str> = styles::list().iter().map(|(n, _)| *n).collect();
    v.push("all");
    v
}

impl Config {
    pub fn flavor_id(&self) -> &'static str {
        palette::flavors()[self.flavor]
    }
    pub fn accent_id(&self) -> &'static str {
        palette::accents()[self.accent]
    }
    pub fn style_id(&self) -> &'static str {
        wallpaper_choices()[self.style]
    }
    pub fn flavor_is_dark(&self) -> bool {
        palette::Palette::new(self.flavor_id()).is_some_and(|p| p.is_dark())
    }
    pub fn slideshow_available(&self) -> bool {
        self.style_id() == "all"
    }

    /// Step names to pass to `apply`, in installer order.
    pub fn step_args(&self) -> Vec<String> {
        STEPS
            .iter()
            .zip(self.steps)
            .filter(|(_, on)| *on)
            .map(|(s, _)| s.name.to_string())
            .collect()
    }

    /// Environment for `apply`.
    pub fn env(&self) -> Vec<(&'static str, String)> {
        let mut env = vec![
            ("FLAVOR", self.flavor_id().to_string()),
            ("ACCENT", self.accent_id().to_string()),
            ("FROSTED", FROSTED[self.frosted].to_string()),
            ("WALLPAPER_STYLE", self.style_id().to_string()),
            ("NO_TUI", "1".to_string()),
        ];
        if self.slideshow_available() && self.rotate > 0 {
            env.push(("WALLPAPER_ROTATE", ROTATE[self.rotate].to_string()));
        }
        if self.light > 0 && self.flavor_is_dark() {
            env.push(("LIGHT_FLAVOR", LIGHT[self.light].to_string()));
        }
        if self.roundness > 0 {
            env.push(("ROUNDNESS", ROUNDNESS[self.roundness].to_string()));
        }
        if self.font > 0 {
            // "+ JetBrainsMono Nerd Font (download)" installs, others just select.
            match self.fonts[self.font].strip_prefix(DOWNLOAD) {
                Some(rest) => env.push((
                    "INSTALL_FONT",
                    rest.split_whitespace()
                        .next()
                        .unwrap_or_default()
                        .to_string(),
                )),
                None => env.push(("TERMINAL_FONT", self.fonts[self.font].clone())),
            }
        }
        env
    }
}

/// What the event loop should do after a key press.
#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    None,
    Quit,
    Install,
}

/// The configuration screen.
#[derive(Default)]
pub struct Setup {
    pub cfg: Config,
    pub field: usize,
    pub message: Option<String>,
}

fn cycle(value: &mut usize, len: usize, forward: bool) {
    *value = if forward {
        (*value + 1) % len
    } else {
        (*value + len - 1) % len
    };
}

impl Setup {
    pub fn current(&self) -> Field {
        fields()[self.field]
    }

    fn change(&mut self, forward: bool) {
        let field = self.current();
        let c = &mut self.cfg;
        match field {
            Field::Profile => {
                cycle(&mut c.profile, c.profiles.len(), forward);
                if c.profile > 0
                    && let Ok(o) = crate::profiles::load(&c.profiles[c.profile])
                {
                    let (profile, profiles, save_as) =
                        (c.profile, c.profiles.clone(), c.save_as.clone());
                    *c = Config::from_opts(&o, c.fonts[1..].to_vec());
                    (c.profile, c.profiles, c.save_as) = (profile, profiles, save_as);
                }
            }
            Field::SaveAs => {}
            Field::Flavor => cycle(&mut c.flavor, palette::flavors().len(), forward),
            Field::Light => cycle(&mut c.light, LIGHT.len(), forward),
            Field::Accent => cycle(&mut c.accent, palette::accents().len(), forward),
            Field::Frosted => cycle(&mut c.frosted, FROSTED.len(), forward),
            Field::Roundness => cycle(&mut c.roundness, ROUNDNESS.len(), forward),
            Field::Wallpaper => cycle(&mut c.style, wallpaper_choices().len(), forward),
            Field::Slideshow if c.slideshow_available() => {
                cycle(&mut c.rotate, ROTATE.len(), forward)
            }
            Field::Slideshow => {
                self.message = Some("The slideshow needs Wallpaper set to \"all\"".into())
            }
            Field::Font => cycle(&mut c.font, c.fonts.len(), forward),
            Field::Step(i) => c.steps[i] = !c.steps[i],
            Field::Install => {}
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> Action {
        self.message = None;
        let n = fields().len();
        // On the name field, letters are text, not shortcuts.
        if self.current() == Field::SaveAs && !key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char(ch) => {
                    if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                        self.cfg.save_as.push(ch);
                    }
                    return Action::None;
                }
                KeyCode::Backspace => {
                    self.cfg.save_as.pop();
                    return Action::None;
                }
                _ => {}
            }
        }
        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
            KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
            KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
                cycle(&mut self.field, n, false);
                Action::None
            }
            KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
                cycle(&mut self.field, n, true);
                Action::None
            }
            KeyCode::Left | KeyCode::Char('h') => {
                self.change(false);
                Action::None
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Char(' ') => {
                self.change(true);
                Action::None
            }
            KeyCode::Enter => {
                let name = &self.cfg.save_as;
                if !name.is_empty() && !crate::profiles::valid_name(name) {
                    self.message = Some("Bad profile name (it cannot start with a dot)".into());
                    Action::None
                } else if self.cfg.step_args().is_empty() {
                    self.message = Some("Select at least one step".into());
                    Action::None
                } else {
                    Action::Install
                }
            }
            _ => Action::None,
        }
    }
}

/// Progress of one step while `apply` runs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    Pending,
    Running,
    Done,
    Warning,
    Failed,
}

pub struct StepProgress {
    pub key: &'static str,
    pub label: &'static str,
    pub status: Status,
}

/// The progress screen: fed apply's output line by line.
pub struct Progress {
    pub steps: Vec<StepProgress>,
    pub log: Vec<(bool, String)>, // (from stderr, text)
    pub todos: Vec<String>,
    pub exit: Option<i32>,
    current: Option<usize>,
    warned: bool,
}

/// Remove ANSI escape sequences (apply colors its messages).
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for c in chars.by_ref() {
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

impl Progress {
    pub fn new(cfg: &Config, backup: bool) -> Self {
        let mut steps = Vec::new();
        if backup {
            steps.push(StepProgress {
                key: "backup",
                label: "Backup",
                status: Status::Pending,
            });
        }
        for (step, on) in STEPS.iter().zip(cfg.steps) {
            if on {
                steps.push(StepProgress {
                    key: step.name,
                    label: step.label,
                    status: Status::Pending,
                });
            }
        }
        Self {
            steps,
            log: Vec::new(),
            todos: Vec::new(),
            exit: None,
            current: None,
            warned: false,
        }
    }

    /// Which step an `apply` "==> ..." header starts.
    fn step_for_header(header: &str) -> Option<&'static str> {
        let rules: [(&str, &str); 8] = [
            ("Backing up", "backup"),
            ("COSMIC theme", "cosmic"),
            ("COSMIC Terminal", "terminal"),
            ("Bash prompt", "bash"),
            ("Zed", "zed"),
            ("Wallpaper", "wallpaper"),
            ("Papirus", "icons"),
            ("GTK", "gtk"),
        ];
        rules
            .iter()
            .find(|(prefix, _)| header.starts_with(prefix))
            .map(|(_, key)| *key)
    }

    fn finish_current(&mut self) {
        if let Some(i) = self.current.take() {
            self.steps[i].status = if self.warned {
                Status::Warning
            } else {
                Status::Done
            };
        }
        self.warned = false;
    }

    pub fn line(&mut self, from_stderr: bool, raw: &str) {
        let line = strip_ansi(raw);
        let trimmed = line.trim();
        if let Some(header) = trimmed.strip_prefix("==> ") {
            if let Some(key) = Self::step_for_header(header)
                && let Some(i) = self.steps.iter().position(|s| s.key == key)
            {
                self.finish_current();
                self.steps[i].status = Status::Running;
                self.current = Some(i);
            }
        } else if trimmed.starts_with("! ") {
            self.warned = true;
        } else if let Some(todo) = trimmed.strip_prefix("➜ ") {
            self.todos.push(todo.to_string());
        }
        if !trimmed.is_empty() {
            self.log.push((from_stderr, line));
        }
    }

    pub fn exited(&mut self, code: i32) {
        if code == 0 {
            self.finish_current();
        } else if let Some(i) = self.current.take() {
            self.steps[i].status = Status::Failed;
        }
        self.exit = Some(code);
    }

    pub fn running(&self) -> bool {
        self.exit.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::KeyEvent;

    fn press(s: &mut Setup, code: KeyCode) -> Action {
        s.on_key(KeyEvent::from(code))
    }

    #[test]
    fn keys_change_the_config() {
        let mut s = Setup::default();
        assert_eq!(s.cfg.flavor_id(), "mocha");
        goto(&mut s, Field::Flavor);
        press(&mut s, KeyCode::Right); // flavor wraps to latte
        assert_eq!(s.cfg.flavor_id(), "latte");
        goto(&mut s, Field::Accent);
        press(&mut s, KeyCode::Left); // accent before mauve
        assert_eq!(s.cfg.accent_id(), "pink");
        // Slideshow is locked until the wallpaper is "all".
        goto(&mut s, Field::Slideshow);
        press(&mut s, KeyCode::Right);
        assert_eq!(s.cfg.rotate, 0);
        assert!(s.message.is_some());
        goto(&mut s, Field::Wallpaper);
        press(&mut s, KeyCode::Left); // glow → all
        assert_eq!(s.cfg.style_id(), "all");
        goto(&mut s, Field::Slideshow);
        press(&mut s, KeyCode::Right);
        assert!(s.cfg.env().contains(&("WALLPAPER_ROTATE", "5m".into())));
    }

    fn goto(s: &mut Setup, f: Field) {
        s.field = fields().iter().position(|x| *x == f).unwrap();
    }

    #[test]
    fn slideshow_intervals_are_valid() {
        for label in &ROTATE[1..] {
            assert!(crate::steps::rotate_seconds(label).is_ok(), "{label}");
        }
    }

    #[test]
    fn light_mode_and_downloaded_fonts_reach_the_environment() {
        let mut s = Setup::default();
        s.cfg.fonts.push("+ Hack Nerd Font (download)".into());
        goto(&mut s, Field::Light);
        press(&mut s, KeyCode::Right);
        assert!(s.cfg.env().contains(&("LIGHT_FLAVOR", "latte".to_string())));
        // A light flavor has no use for a light mode of its own.
        s.cfg.flavor = palette::flavors()
            .iter()
            .position(|f| *f == "latte")
            .unwrap();
        assert!(!s.cfg.env().iter().any(|(k, _)| *k == "LIGHT_FLAVOR"));
        goto(&mut s, Field::Font);
        press(&mut s, KeyCode::Right);
        assert!(s.cfg.env().contains(&("INSTALL_FONT", "Hack".to_string())));
        assert!(!s.cfg.env().iter().any(|(k, _)| *k == "TERMINAL_FONT"));
    }

    #[test]
    fn saved_choices_fill_the_screen() {
        let o = crate::apply::Opts {
            flavor: "latte".into(),
            accent: "peach".into(),
            frosted: "medium".into(),
            roundness: "square".into(),
            style: "all".into(),
            rotate: Some("30m".into()),
            font: Some("Fira Mono".into()),
            steps: vec!["bash".into(), "icons".into()],
            ..Default::default()
        };
        let c = Config::from_opts(&o, vec!["Fira Mono".into()]);
        assert_eq!(
            (c.flavor_id(), c.accent_id(), c.style_id()),
            ("latte", "peach", "all")
        );
        assert_eq!(c.step_args(), ["bash", "icons"]);
        let env = c.env();
        for (k, v) in [
            ("FROSTED", "medium"),
            ("ROUNDNESS", "square"),
            ("WALLPAPER_ROTATE", "30m"),
            ("TERMINAL_FONT", "Fira Mono"),
        ] {
            assert!(env.contains(&(k, v.to_string())), "{k}");
        }
    }

    #[test]
    fn font_row_cycles_and_reaches_the_environment() {
        let mut s = Setup::default();
        s.cfg.fonts.push("Fira Mono".into());
        assert!(!s.cfg.env().iter().any(|(k, _)| *k == "TERMINAL_FONT"));
        goto(&mut s, Field::Font);
        assert_eq!(s.current(), Field::Font);
        press(&mut s, KeyCode::Right);
        assert!(
            s.cfg
                .env()
                .contains(&("TERMINAL_FONT", "Fira Mono".to_string()))
        );
        press(&mut s, KeyCode::Right);
        assert_eq!(s.cfg.font, 0);
    }

    #[test]
    fn steps_toggle_and_need_one() {
        let mut s = Setup::default();
        // Everything but the optional icons step starts selected.
        assert_eq!(
            s.cfg.step_args(),
            ["cosmic", "terminal", "bash", "zed", "wallpaper"]
        );
        for i in (0..STEPS.len()).filter(|i| !STEPS[*i].optional) {
            goto(&mut s, Field::Step(i));
            press(&mut s, KeyCode::Char(' '));
        }
        assert!(s.cfg.step_args().is_empty());
        assert_eq!(press(&mut s, KeyCode::Enter), Action::None);
        goto(&mut s, Field::Step(2));
        press(&mut s, KeyCode::Char(' '));
        assert_eq!(s.cfg.step_args(), ["bash"]);
        assert_eq!(press(&mut s, KeyCode::Enter), Action::Install);
        assert_eq!(press(&mut s, KeyCode::Char('q')), Action::Quit);
    }

    #[test]
    fn progress_follows_install_output() {
        let cfg = Config {
            steps: [true, true, false, false, true, false, false],
            ..Config::default()
        };
        let mut p = Progress::new(&cfg, true);
        for l in [
            "\u{1b}[38;2;137;180;250m==>\u{1b}[0m Backing up to /x",
            " ✓ Backup saved",
            "==> COSMIC theme (mocha, mauve accent, solid)",
            " ✓ Theme applied",
            "==> COSMIC Terminal",
            " ➜ In COSMIC Terminal: View → Color schemes… → Import and pick:",
            "==> Wallpaper (mocha, mauve accent, glow)",
            " ! something odd",
        ] {
            p.line(false, l);
        }
        assert_eq!(p.steps[0].status, Status::Done);
        assert_eq!(p.steps[2].status, Status::Done);
        assert_eq!(p.steps[3].status, Status::Running);
        assert_eq!(p.todos.len(), 1);
        p.exited(0);
        assert_eq!(p.steps[3].status, Status::Warning);
        assert!(!p.running());

        let mut p = Progress::new(&cfg, false);
        p.line(false, "==> COSMIC theme (x)");
        p.exited(1);
        assert_eq!(p.steps[0].status, Status::Failed);
        assert_eq!(p.steps[1].status, Status::Pending);
    }

    #[test]
    fn typing_a_profile_name() {
        let mut s = Setup::default();
        goto(&mut s, Field::SaveAs);
        let key = |c| KeyEvent::new(c, KeyModifiers::NONE);
        // q, j and space are text here, not shortcuts.
        for ch in ['q', 'j', ' ', 'a', '/'] {
            assert_eq!(s.on_key(key(KeyCode::Char(ch))), Action::None);
        }
        assert_eq!(s.cfg.save_as, "qja");
        s.on_key(key(KeyCode::Backspace));
        assert_eq!(s.cfg.save_as, "qj");
        assert_eq!(s.current(), Field::SaveAs);
        s.cfg.save_as = ".bad".into();
        assert_eq!(s.on_key(key(KeyCode::Enter)), Action::None);
        assert!(s.message.is_some());
    }
}
