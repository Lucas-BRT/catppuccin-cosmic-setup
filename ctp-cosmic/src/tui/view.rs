//! Drawing the TUI. The interface itself uses the flavor and accent being
//! configured, so the choice is previewed everywhere at once.

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph, Widget, Wrap};

use super::app::{Config, FROSTED, Field, Progress, ROTATE, STEPS, Setup, Status, fields};
use crate::palette::{self, Palette};

/// Colors of the interface, taken from the selected flavor and accent.
#[derive(Clone, Copy)]
pub struct Theme {
    pub p: Palette,
    pub accent: &'static str,
}

impl Theme {
    pub fn of(cfg: &Config) -> Self {
        Self {
            p: Palette::new(cfg.flavor_id()).unwrap(),
            accent: cfg.accent_id(),
        }
    }
    pub fn c(&self, name: &str) -> Color {
        let [r, g, b] = self.p.get(name);
        Color::Rgb(r as u8, g as u8, b as u8)
    }
    fn base(&self) -> Style {
        Style::new().fg(self.c("text")).bg(self.c("base"))
    }
    fn accent(&self) -> Color {
        self.c(self.accent)
    }
    fn dim(&self) -> Style {
        Style::new().fg(self.c("overlay1"))
    }
    fn block<'a>(&self, title: &'a str) -> Block<'a> {
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::new().fg(self.c("surface2")))
            .title(Span::styled(
                format!(" {title} "),
                Style::new().fg(self.accent()).add_modifier(Modifier::BOLD),
            ))
    }
}

/// Wallpaper drawn with "▀": each cell holds two pixels (fg on top, bg below).
pub struct HalfBlockImage<'a> {
    pub rgb: &'a [u8],
    pub width: usize,
}

impl Widget for HalfBlockImage<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let px = |x: usize, y: usize| {
            let i = (y * self.width + x) * 3;
            Color::Rgb(self.rgb[i], self.rgb[i + 1], self.rgb[i + 2])
        };
        for row in 0..area.height as usize {
            for col in 0..area.width as usize {
                if let Some(cell) = buf.cell_mut((area.x + col as u16, area.y + row as u16)) {
                    cell.set_char('▀')
                        .set_fg(px(col, row * 2))
                        .set_bg(px(col, row * 2 + 1));
                }
            }
        }
    }
}

fn footer(f: &mut Frame, area: Rect, t: Theme, keys: &[(&str, &str)], message: Option<&str>) {
    let mut spans = Vec::new();
    if let Some(m) = message {
        spans.push(Span::styled(
            format!(" {m}   "),
            Style::new().fg(t.c("yellow")),
        ));
    }
    for (k, what) in keys {
        spans.push(Span::styled(
            format!(" {k} "),
            Style::new().fg(t.c("base")).bg(t.accent()),
        ));
        spans.push(Span::styled(format!(" {what}  "), t.dim()));
    }
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// The configuration screen. `preview` renders the wallpaper for a size.
pub fn draw_setup(f: &mut Frame, s: &Setup, preview: &mut dyn FnMut(usize, usize) -> Vec<u8>) {
    let t = Theme::of(&s.cfg);
    f.render_widget(Block::new().style(t.base()), f.area());
    let [main, foot] =
        Layout::vertical([Constraint::Min(10), Constraint::Length(1)]).areas(f.area());
    let [left, right] =
        Layout::horizontal([Constraint::Length(44), Constraint::Min(20)]).areas(main);

    // Left: the fields.
    let block = t.block("Catppuccin for COSMIC");
    let inner = block.inner(left);
    f.render_widget(block, left);
    let c = &s.cfg;
    let mut lines: Vec<Line> = Vec::new();
    let focused = s.current();
    let row = |field: Field, label: &str, value: String, enabled: bool| -> Line<'static> {
        let on = field == focused;
        let marker = if on { "›" } else { " " };
        let value_style = if !enabled {
            t.dim()
        } else if on {
            Style::new().fg(t.accent()).add_modifier(Modifier::BOLD)
        } else {
            Style::new().fg(t.c("text"))
        };
        let shown = if on && enabled {
            format!("‹ {value} ›")
        } else {
            format!("  {value}  ")
        };
        let mut line = Line::from(vec![
            Span::styled(format!("{marker} "), Style::new().fg(t.accent())),
            Span::styled(format!("{label:<11}"), Style::new().fg(t.c("subtext1"))),
            Span::styled(shown, value_style),
        ]);
        if on {
            line = line.style(Style::new().bg(t.c("surface0")));
        }
        line
    };
    let p = Palette::new(c.flavor_id()).unwrap();
    lines.push(Line::raw(""));
    lines.push(row(Field::Flavor, "Flavor", p.name(), true));
    lines.push(row(
        Field::Accent,
        "Accent",
        capitalize(c.accent_id()),
        true,
    ));
    // Accent swatches, the selected one underlined.
    let mut sw = vec![Span::raw("  ".to_string() + &" ".repeat(11))];
    let mut marks = sw.clone();
    for (i, a) in palette::accents().iter().enumerate() {
        sw.push(Span::styled("█▌", Style::new().fg(t.c(a))));
        marks.push(Span::styled(
            if i == c.accent { "▀ " } else { "  " },
            Style::new().fg(t.c("text")),
        ));
    }
    lines.push(Line::from(sw));
    lines.push(Line::from(marks));
    lines.push(row(
        Field::Frosted,
        "Frosted",
        FROSTED[c.frosted].replace('_', " "),
        true,
    ));
    let wall_on = c.steps[5];
    lines.push(row(
        Field::Wallpaper,
        "Wallpaper",
        c.style_id().into(),
        wall_on,
    ));
    let rotate = if c.slideshow_available() {
        ROTATE[c.rotate]
    } else {
        "—"
    };
    lines.push(row(
        Field::Slideshow,
        "Slideshow",
        rotate.into(),
        wall_on && c.slideshow_available(),
    ));
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "  Steps",
        Style::new()
            .fg(t.c("subtext1"))
            .add_modifier(Modifier::BOLD),
    ));
    for (i, (_, label)) in STEPS.iter().enumerate() {
        let on = focused == Field::Step(i);
        let check = if c.steps[i] { "◉" } else { "○" };
        let mut line = Line::from(vec![
            Span::styled(if on { "› " } else { "  " }, Style::new().fg(t.accent())),
            Span::styled(
                format!("{check} "),
                Style::new().fg(if c.steps[i] {
                    t.accent()
                } else {
                    t.c("overlay0")
                }),
            ),
            Span::styled(label.to_string(), Style::new().fg(t.c("text"))),
        ]);
        if on {
            line = line.style(Style::new().bg(t.c("surface0")));
        }
        lines.push(line);
    }
    lines.push(Line::raw(""));
    let install_on = focused == Field::Install;
    lines.push(Line::from(vec![
        Span::raw(if install_on { "› " } else { "  " }),
        Span::styled(
            "  Install  ",
            if install_on {
                Style::new()
                    .fg(t.c("base"))
                    .bg(t.accent())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(t.accent()).bg(t.c("surface0"))
            },
        ),
    ]));
    debug_assert_eq!(fields().len(), 6 + STEPS.len());
    f.render_widget(Paragraph::new(lines), inner);

    // Right: live preview.
    let block = t.block("Preview");
    let inner = block.inner(right);
    f.render_widget(block, right);
    let [img, _, prompt, swatches] = Layout::vertical([
        Constraint::Min(4),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);
    if img.width > 0 && img.height > 0 {
        let (w, h) = (img.width as usize, img.height as usize * 2);
        let rgb = preview(w, h);
        f.render_widget(
            HalfBlockImage {
                rgb: &rgb,
                width: w,
            },
            img,
        );
    }
    let fg = |n: &str| Style::new().fg(t.c(n));
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(" you", fg("blue")),
            Span::styled("@", fg("overlay0")),
            Span::styled("pop-os ", fg("teal")),
            Span::styled("~/Projects ", fg("yellow")),
            Span::styled("main ", fg(c.accent_id())),
            Span::styled("❯", fg("green")),
        ])),
        prompt,
    );
    let names = [
        "rosewater",
        "flamingo",
        "pink",
        "mauve",
        "red",
        "maroon",
        "peach",
        "yellow",
        "green",
        "teal",
        "sky",
        "sapphire",
        "blue",
        "lavender",
        "text",
        "overlay1",
        "surface1",
        "base",
        "mantle",
        "crust",
    ];
    let strip: Vec<Span> = names
        .iter()
        .map(|n| Span::styled("██", Style::new().fg(t.c(n))))
        .collect();
    f.render_widget(
        Paragraph::new(Line::from([vec![Span::raw(" ")], strip].concat())),
        swatches,
    );

    footer(
        f,
        foot,
        t,
        &[
            ("↑↓", "move"),
            ("←→", "change"),
            ("space", "toggle"),
            ("enter", "install"),
            ("q", "quit"),
        ],
        s.message.as_deref(),
    );
}

fn capitalize(s: &str) -> String {
    s[..1].to_uppercase() + &s[1..]
}

const SPINNER: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// The progress screen while (and after) install.sh runs.
pub fn draw_progress(f: &mut Frame, cfg: &Config, p: &Progress, tick: usize) {
    let t = Theme::of(cfg);
    f.render_widget(Block::new().style(t.base()), f.area());
    let [main, foot] =
        Layout::vertical([Constraint::Min(10), Constraint::Length(1)]).areas(f.area());
    let [left, right] =
        Layout::horizontal([Constraint::Length(36), Constraint::Min(20)]).areas(main);
    let todo_height = if p.todos.is_empty() || p.running() {
        0
    } else {
        p.todos.len() as u16 + 2
    };
    let [steps_area, todo_area] =
        Layout::vertical([Constraint::Min(4), Constraint::Length(todo_height)]).areas(left);

    let block = t.block("Installing");
    let inner = block.inner(steps_area);
    f.render_widget(block, steps_area);
    let mut lines = vec![Line::raw("")];
    for s in &p.steps {
        let (glyph, color) = match s.status {
            Status::Pending => ("·", t.c("overlay0")),
            Status::Running => (SPINNER[tick % SPINNER.len()], t.accent()),
            Status::Done => ("✓", t.c("green")),
            Status::Warning => ("!", t.c("yellow")),
            Status::Failed => ("✗", t.c("red")),
        };
        let label = match (s.status, p.running()) {
            (Status::Pending, false) => format!("{} (skipped)", s.label),
            _ => s.label.to_string(),
        };
        lines.push(Line::from(vec![
            Span::styled(
                format!("  {glyph} "),
                Style::new().fg(color).add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                label,
                if s.status == Status::Pending {
                    t.dim()
                } else {
                    Style::new().fg(t.c("text"))
                },
            ),
        ]));
    }
    f.render_widget(Paragraph::new(lines), inner);

    if todo_height > 0 {
        let block = t.block("Left for you");
        let inner = block.inner(todo_area);
        f.render_widget(block, todo_area);
        let lines: Vec<Line> = p
            .todos
            .iter()
            .map(|l| Line::styled(format!(" {l}"), Style::new().fg(t.c("text"))))
            .collect();
        f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
    }

    let block = t.block("Log");
    let inner = block.inner(right);
    f.render_widget(block, right);
    let skip = p.log.len().saturating_sub(inner.height as usize);
    let lines: Vec<Line> = p.log[skip..]
        .iter()
        .map(|(err, l)| {
            Line::styled(
                format!(" {l}"),
                if *err {
                    Style::new().fg(t.c("red"))
                } else {
                    Style::new().fg(t.c("subtext0"))
                },
            )
        })
        .collect();
    f.render_widget(Paragraph::new(lines), inner);

    let (keys, msg): (&[(&str, &str)], Option<String>) = match p.exit {
        None => (
            &[("ctrl-c", "stop")],
            Some(format!("{} working…", SPINNER[tick % SPINNER.len()])),
        ),
        Some(0) => (&[("q", "quit")], Some("Done.".into())),
        Some(code) => (
            &[("q", "quit")],
            Some(format!("install.sh failed (exit {code}); see the log")),
        ),
    };
    footer(f, foot, t, keys, msg.as_deref());
}

#[cfg(test)]
mod tests {
    use super::super::app::{Config, Progress, Setup};
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn text(buf: &Buffer) -> String {
        let a = buf.area;
        (0..a.height)
            .map(|y| {
                (0..a.width)
                    .map(|x| buf[(x, y)].symbol())
                    .collect::<String>()
                    + "\n"
            })
            .collect()
    }

    #[test]
    fn setup_screen_shows_fields_and_preview() {
        let mut term = Terminal::new(TestBackend::new(100, 28)).unwrap();
        let s = Setup::default();
        term.draw(|f| draw_setup(f, &s, &mut |w, h| vec![0x80; w * h * 3]))
            .unwrap();
        let buf = term.backend().buffer().clone();
        let out = text(&buf);
        if std::env::var("SHOW_TUI").is_ok() {
            println!("{out}");
        }
        for want in [
            "Catppuccin for COSMIC",
            "Mocha",
            "Mauve",
            "Frosted",
            "Install",
            "Preview",
            "Papirus icons",
        ] {
            assert!(out.contains(want), "missing {want}");
        }
        // The preview is drawn with half blocks in the image's colors.
        assert!(out.contains('▀'));
    }

    #[test]
    fn progress_screen_shows_steps_and_todos() {
        let mut term = Terminal::new(TestBackend::new(100, 24)).unwrap();
        let cfg = Config::default();
        let mut p = Progress::new(&cfg, true);
        for l in [
            "==> Backing up to /x",
            "==> COSMIC theme (mocha)",
            " ➜ Import the terminal colors",
            "==> COSMIC Terminal",
        ] {
            p.line(false, l);
        }
        term.draw(|f| draw_progress(f, &cfg, &p, 3)).unwrap();
        let running = text(term.backend().buffer());
        assert!(running.contains("Installing") && running.contains("working"));
        p.exited(0);
        term.draw(|f| draw_progress(f, &cfg, &p, 3)).unwrap();
        let done = text(term.backend().buffer());
        if std::env::var("SHOW_TUI").is_ok() {
            println!("{done}");
        }
        assert!(done.contains("Left for you") && done.contains("Import the terminal colors"));
        assert!(done.contains("(skipped)") && done.contains("Done."));
    }
}

/// Debug aid: `TUI_PNG=out.png cargo test snapshot_png -- --ignored` writes an
/// approximate picture of the setup screen (cell colors, glyphs as bars).
#[cfg(test)]
mod snapshot {
    use super::super::app::Setup;
    use super::*;
    use ratatui::{Terminal, backend::TestBackend};

    fn rgb(c: Color, fallback: [u8; 3]) -> [u8; 3] {
        match c {
            Color::Rgb(r, g, b) => [r, g, b],
            _ => fallback,
        }
    }

    #[test]
    #[ignore]
    fn snapshot_png() {
        let (cw, ch, cols, rows) = (8usize, 16usize, 110u16, 30u16);
        let mut setup = Setup::default();
        for (k, n) in [("flavor", 0usize), ("accent", 0), ("style", 0)] {
            if let Ok(v) = std::env::var(format!("TUI_{}", k.to_uppercase())) {
                let v: usize = v.parse().unwrap();
                match k {
                    "flavor" => setup.cfg.flavor = v,
                    "accent" => setup.cfg.accent = v,
                    _ => setup.cfg.style = v,
                }
            }
            let _ = n;
        }
        let mut term = Terminal::new(TestBackend::new(cols, rows)).unwrap();
        term.draw(|f| {
            let c = setup.cfg.clone();
            draw_setup(f, &setup, &mut |w, h| {
                crate::wallpaper::render_one(c.style_id(), c.flavor_id(), c.accent_id(), (w, h))
            })
        })
        .unwrap();
        let buf = term.backend().buffer().clone();
        let (w, h) = (cols as usize * cw, rows as usize * ch);
        let mut img = vec![0u8; w * h * 3];
        for y in 0..rows {
            for x in 0..cols {
                let cell = &buf[(x, y)];
                let bg = rgb(cell.bg, [30, 30, 46]);
                let fg = rgb(cell.fg, [205, 214, 244]);
                let sym = cell.symbol();
                for py in 0..ch {
                    for px in 0..cw {
                        let color = match sym {
                            "▀" => {
                                if py < ch / 2 {
                                    fg
                                } else {
                                    bg
                                }
                            }
                            " " => bg,
                            "█" | "▌" => fg,
                            _ if (4..12).contains(&py) && (1..7).contains(&px) => fg,
                            _ => bg,
                        };
                        let i = ((y as usize * ch + py) * w + x as usize * cw + px) * 3;
                        img[i..i + 3].copy_from_slice(&color);
                    }
                }
            }
        }
        crate::image::write_png(
            std::path::Path::new(&std::env::var("TUI_PNG").unwrap()),
            w,
            h,
            &img,
        )
        .unwrap();
    }
}
