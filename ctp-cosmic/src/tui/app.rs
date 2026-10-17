//! TUI state and key handling, kept free of terminal I/O so it can be tested.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::palette;
use crate::styles;

/// Installer steps the user can pick, as (install.sh name, label).
pub const STEPS: [(&str, &str); 6] = [
    ("cosmic", "Desktop theme"),
    ("terminal", "Terminal colors"),
    ("bash", "Bash prompt"),
    ("zed", "Zed check"),
    ("icons", "Papirus icons"),
    ("wallpaper", "Wallpaper"),
];

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

/// Slideshow intervals COSMIC Settings offers (only with every style).
pub const ROTATE: [&str; 7] = ["off", "5m", "10m", "15m", "30m", "1h", "2h"];

/// The rows of the configuration screen, top to bottom.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Field {
    Flavor,
    Accent,
    Frosted,
    Wallpaper,
    Slideshow,
    Step(usize),
    Install,
}

pub fn fields() -> Vec<Field> {
    let mut f = vec![
        Field::Flavor,
        Field::Accent,
        Field::Frosted,
        Field::Wallpaper,
        Field::Slideshow,
    ];
    f.extend((0..STEPS.len()).map(Field::Step));
    f.push(Field::Install);
    f
}

/// What the user chose, turned into install.sh arguments and environment.
#[derive(Clone, Debug)]
pub struct Config {
    pub flavor: usize,
    pub accent: usize,
    pub frosted: usize,
    /// Index into `wallpaper_choices()` (the styles, then "all").
    pub style: usize,
    pub rotate: usize,
    pub steps: [bool; STEPS.len()],
}

impl Default for Config {
    fn default() -> Self {
        let flavors = palette::flavors();
        let accents = palette::accents();
        Self {
            flavor: flavors.iter().position(|f| *f == "mocha").unwrap_or(0),
            accent: accents.iter().position(|a| *a == "mauve").unwrap_or(0),
            frosted: 0,
            style: 0,
            rotate: 0,
            steps: [true; STEPS.len()],
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
    pub fn slideshow_available(&self) -> bool {
        self.style_id() == "all"
    }

    /// Step names to pass to install.sh, in installer order.
    pub fn step_args(&self) -> Vec<String> {
        STEPS
            .iter()
            .zip(self.steps)
            .filter(|(_, on)| *on)
            .map(|((name, _), _)| name.to_string())
            .collect()
    }

    /// Environment for install.sh.
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
            Field::Flavor => cycle(&mut c.flavor, palette::flavors().len(), forward),
            Field::Accent => cycle(&mut c.accent, palette::accents().len(), forward),
            Field::Frosted => cycle(&mut c.frosted, FROSTED.len(), forward),
            Field::Wallpaper => cycle(&mut c.style, wallpaper_choices().len(), forward),
            Field::Slideshow if c.slideshow_available() => {
                cycle(&mut c.rotate, ROTATE.len(), forward)
            }
            Field::Slideshow => {
                self.message = Some("The slideshow needs Wallpaper set to \"all\"".into())
            }
            Field::Step(i) => c.steps[i] = !c.steps[i],
            Field::Install => {}
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> Action {
        self.message = None;
        let n = fields().len();
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
                if self.cfg.step_args().is_empty() {
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

/// Progress of one step while install.sh runs.
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

/// The progress screen: fed install.sh's output line by line.
pub struct Progress {
    pub steps: Vec<StepProgress>,
    pub log: Vec<(bool, String)>, // (from stderr, text)
    pub todos: Vec<String>,
    pub exit: Option<i32>,
    current: Option<usize>,
    warned: bool,
}

/// Remove ANSI escape sequences (install.sh colors its messages).
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
        for ((key, label), on) in STEPS.iter().zip(cfg.steps) {
            if on {
                steps.push(StepProgress {
                    key,
                    label,
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

    /// Which step an install.sh "==> ..." header starts.
    fn step_for_header(header: &str) -> Option<&'static str> {
        let rules: [(&str, &str); 7] = [
            ("Backing up", "backup"),
            ("COSMIC theme", "cosmic"),
            ("COSMIC Terminal", "terminal"),
            ("Bash prompt", "bash"),
            ("Zed", "zed"),
            ("Wallpaper", "wallpaper"),
            ("Papirus", "icons"),
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
        press(&mut s, KeyCode::Right); // flavor wraps to latte
        assert_eq!(s.cfg.flavor_id(), "latte");
        press(&mut s, KeyCode::Down);
        press(&mut s, KeyCode::Left); // accent before mauve
        assert_eq!(s.cfg.accent_id(), "pink");
        // Slideshow is locked until the wallpaper is "all".
        s.field = 4;
        press(&mut s, KeyCode::Right);
        assert_eq!(s.cfg.rotate, 0);
        assert!(s.message.is_some());
        s.field = 3;
        press(&mut s, KeyCode::Left); // glow → all
        assert_eq!(s.cfg.style_id(), "all");
        s.field = 4;
        press(&mut s, KeyCode::Right);
        assert!(s.cfg.env().contains(&("WALLPAPER_ROTATE", "5m".into())));
    }

    #[test]
    fn steps_toggle_and_need_one() {
        let mut s = Setup::default();
        assert_eq!(s.cfg.step_args().len(), STEPS.len());
        for i in 0..STEPS.len() {
            s.field = 5 + i;
            press(&mut s, KeyCode::Char(' '));
        }
        assert!(s.cfg.step_args().is_empty());
        assert_eq!(press(&mut s, KeyCode::Enter), Action::None);
        s.field = 5 + 2;
        press(&mut s, KeyCode::Char(' '));
        assert_eq!(s.cfg.step_args(), ["bash"]);
        assert_eq!(press(&mut s, KeyCode::Enter), Action::Install);
        assert_eq!(press(&mut s, KeyCode::Char('q')), Action::Quit);
    }

    #[test]
    fn progress_follows_install_output() {
        let cfg = Config {
            steps: [true, true, false, false, false, true],
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
}
