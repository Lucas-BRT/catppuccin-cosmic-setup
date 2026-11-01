//! The installer's steps and slideshow intervals: the single list that
//! setup.sh, the scripts and the TUI all read.

/// One installer step. `script` is the file under scripts/ that runs it.
pub struct Step {
    pub name: &'static str,
    pub label: &'static str,
    pub script: &'static str,
    /// Runs only when named explicitly; never part of the default selection.
    pub optional: bool,
}

/// In installer order.
pub const STEPS: [Step; 6] = [
    Step {
        name: "cosmic",
        label: "Desktop theme",
        script: "10-cosmic-theme.sh",
        optional: false,
    },
    Step {
        name: "terminal",
        label: "Terminal colors",
        script: "20-terminal.sh",
        optional: false,
    },
    Step {
        name: "bash",
        label: "Bash prompt",
        script: "30-bash-prompt.sh",
        optional: false,
    },
    Step {
        name: "zed",
        label: "Zed theme",
        script: "40-zed.sh",
        optional: false,
    },
    Step {
        name: "icons",
        label: "Papirus icons (optional)",
        script: "50-icons.sh",
        optional: true,
    },
    Step {
        name: "wallpaper",
        label: "Wallpaper",
        script: "60-wallpaper.sh",
        optional: false,
    },
];

/// Intervals COSMIC Settings → Wallpaper offers: (label, seconds).
const ROTATE: [(&str, u32); 6] = [
    ("5m", 300),
    ("10m", 600),
    ("15m", 900),
    ("30m", 1800),
    ("1h", 3600),
    ("2h", 7200),
];

/// Seconds for a label ("30m") or the same number as text ("1800").
pub fn rotate_seconds(value: &str) -> Result<u32, String> {
    ROTATE
        .iter()
        .find(|(label, secs)| *label == value || secs.to_string() == value)
        .map(|(_, secs)| *secs)
        .ok_or_else(|| {
            let labels: Vec<_> = ROTATE.iter().map(|(l, _)| *l).collect();
            format!(
                "invalid WALLPAPER_ROTATE '{value}' (choose: {})",
                labels.join(", ")
            )
        })
}

const USAGE: &str = "\
ctp-cosmic steps: what setup.sh runs

Usage:
  ctp-cosmic steps list             name, script and required|optional, tab-separated
  ctp-cosmic steps rotate VALUE     slideshow interval in seconds (5m ... 2h)
";

pub fn main(args: Vec<String>) -> Result<(), String> {
    match args.iter().map(String::as_str).collect::<Vec<_>>()[..] {
        ["list"] => {
            for s in &STEPS {
                let kind = if s.optional { "optional" } else { "required" };
                println!("{}\t{}\t{kind}", s.name, s.script);
            }
            Ok(())
        }
        ["rotate", value] => {
            println!("{}", rotate_seconds(value)?);
            Ok(())
        }
        [] | ["-h" | "--help"] => {
            print!("{USAGE}");
            Ok(())
        }
        _ => Err(format!("bad arguments\n\n{USAGE}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotate_accepts_labels_and_seconds_only() {
        assert_eq!(rotate_seconds("30m"), Ok(1800));
        assert_eq!(rotate_seconds("7200"), Ok(7200));
        assert!(rotate_seconds("1m").is_err());
    }
}
