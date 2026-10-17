//! `ctp-cosmic tui`: interactive setup. Pick flavor, accent, Frosted Glass,
//! wallpaper and steps with a live preview, then watch install.sh run.
//!
//! install.sh opens this when it runs in a terminal with no options; the
//! choices are passed back to install.sh as arguments and environment
//! variables, so the TUI and scripted installs run exactly the same code.

mod app;
mod view;

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use ratatui::DefaultTerminal;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use app::{Action, Config, Progress, Setup, Status};

const USAGE: &str = "\
ctp-cosmic tui: interactive Catppuccin setup

Usage:
  ctp-cosmic tui --repo DIR   DIR holds install.sh (the setup to run)

install.sh opens this by itself when run in a terminal without options.
";

/// One line of install.sh output: (from stderr, text).
type Msg = (bool, String);

/// The rendered wallpaper preview and what it shows:
/// (flavor, accent, style, width, height).
type PreviewCache = Option<((usize, usize, usize, usize, usize), Vec<u8>)>;

/// Start install.sh with the chosen options, streaming its output.
fn spawn(repo: &Path, cfg: &Config, tx: mpsc::Sender<Msg>) -> Result<std::process::Child, String> {
    let mut cmd = Command::new(repo.join("install.sh"));
    cmd.args(cfg.step_args())
        .envs(cfg.env())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = cmd.spawn().map_err(|e| format!("install.sh: {e}"))?;
    for (is_err, pipe) in [
        (
            false,
            child
                .stdout
                .take()
                .map(|p| Box::new(p) as Box<dyn std::io::Read + Send>),
        ),
        (
            true,
            child
                .stderr
                .take()
                .map(|p| Box::new(p) as Box<dyn std::io::Read + Send>),
        ),
    ] {
        let tx = tx.clone();
        if let Some(pipe) = pipe {
            std::thread::spawn(move || {
                for line in BufReader::new(pipe).lines().map_while(Result::ok) {
                    let _ = tx.send((is_err, line));
                }
            });
        }
    }
    Ok(child)
}

/// What happened, printed after the interface closes so it stays in the
/// terminal's scrollback.
fn summary(cfg: &Config, p: &Progress) -> String {
    let mut out = format!(
        "catppuccin-cosmic-setup: {} {}\n",
        cfg.flavor_id(),
        cfg.accent_id()
    );
    for s in &p.steps {
        let mark = match s.status {
            Status::Done => "✓",
            Status::Warning => "!",
            Status::Failed => "✗",
            Status::Pending | Status::Running => "·",
        };
        out.push_str(&format!("  {mark} {}\n", s.label));
    }
    if !p.todos.is_empty() {
        out.push_str("Left for you:\n");
        for t in &p.todos {
            out.push_str(&format!("  ➜ {t}\n"));
        }
    }
    if let Some(code) = p.exit.filter(|c| *c != 0) {
        out.push_str(&format!("install.sh failed (exit {code}). Last lines:\n"));
        for (_, l) in p
            .log
            .iter()
            .rev()
            .take(10)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            out.push_str(&format!("  {l}\n"));
        }
    }
    out
}

fn run(terminal: &mut DefaultTerminal, repo: &Path) -> Result<Option<(Config, Progress)>, String> {
    let io = |e: std::io::Error| e.to_string();
    let mut setup = Setup::default();
    // Cache the preview: only re-render when what it shows changes.
    let mut cache: PreviewCache = None;

    loop {
        terminal
            .draw(|f| {
                let c = setup.cfg.clone();
                let mut preview = |w: usize, h: usize| {
                    let key = (c.flavor, c.accent, c.style, w, h);
                    if cache.as_ref().map(|(k, _)| *k) != Some(key) {
                        // "all" previews the default style.
                        let style = if c.style_id() == "all" {
                            "glow"
                        } else {
                            c.style_id()
                        };
                        let rgb = crate::wallpaper::render_one(
                            style,
                            c.flavor_id(),
                            c.accent_id(),
                            (w, h),
                        );
                        cache = Some((key, rgb));
                    }
                    cache.as_ref().unwrap().1.clone()
                };
                view::draw_setup(f, &setup, &mut preview);
            })
            .map_err(io)?;
        if let Event::Key(key) = event::read().map_err(io)?
            && key.kind == KeyEventKind::Press
        {
            match setup.on_key(key) {
                Action::Quit => return Ok(None),
                Action::Install => break,
                Action::None => {}
            }
        }
    }

    let cfg = setup.cfg.clone();
    let backup = std::env::var("NO_BACKUP").map(|v| v != "1").unwrap_or(true);
    let mut progress = Progress::new(&cfg, backup);
    let (tx, rx) = mpsc::channel();
    let mut child = spawn(repo, &cfg, tx)?;
    let started = Instant::now();
    loop {
        while let Ok((err, l)) = rx.try_recv() {
            progress.line(err, &l);
        }
        if progress.running()
            && let Some(status) = child.try_wait().map_err(io)?
        {
            // Let the reader threads flush the last lines first.
            std::thread::sleep(Duration::from_millis(50));
            while let Ok((err, l)) = rx.try_recv() {
                progress.line(err, &l);
            }
            progress.exited(status.code().unwrap_or(1));
        }
        let tick = (started.elapsed().as_millis() / 80) as usize;
        terminal
            .draw(|f| view::draw_progress(f, &cfg, &progress, tick))
            .map_err(io)?;
        if event::poll(Duration::from_millis(80)).map_err(io)?
            && let Event::Key(key) = event::read().map_err(io)?
            && key.kind == KeyEventKind::Press
        {
            let ctrl_c =
                key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL);
            if progress.running() && ctrl_c {
                let _ = child.kill();
            } else if !progress.running()
                && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc | KeyCode::Enter)
            {
                return Ok(Some((cfg, progress)));
            }
        }
    }
}

pub fn main(args: Vec<String>) -> Result<u8, String> {
    let repo = match args.as_slice() {
        [flag, dir] if flag == "--repo" => PathBuf::from(dir),
        [h] if h == "-h" || h == "--help" => {
            print!("{USAGE}");
            return Ok(0);
        }
        _ => return Err(format!("invalid arguments\n\n{USAGE}")),
    };
    if !repo.join("install.sh").is_file() {
        return Err(format!("{}: no install.sh here", repo.display()));
    }
    let mut terminal = ratatui::try_init().map_err(|e| format!("cannot open the terminal: {e}"))?;
    let result = run(&mut terminal, &repo);
    ratatui::restore();
    match result? {
        None => {
            println!("Nothing was installed.");
            Ok(0)
        }
        Some((cfg, progress)) => {
            print!("{}", summary(&cfg, &progress));
            Ok(if progress.exit == Some(0) { 0 } else { 1 })
        }
    }
}
