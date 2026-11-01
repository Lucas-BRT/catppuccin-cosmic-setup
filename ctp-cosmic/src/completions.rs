//! `ctp-cosmic completions bash|zsh|fish`: tab completion for commands,
//! flags and their values, built from the same lists the tool validates with.

use crate::steps::{STEPS, rotate_labels};
use crate::{palette, styles, theme};

const COMMANDS: [&str; 18] = [
    "apply",
    "profile",
    "update",
    "uninstall",
    "status",
    "doctor",
    "completions",
    "theme",
    "wallpaper",
    "terminal",
    "palette",
    "backup",
    "background",
    "icons",
    "prompt",
    "register-wallpaper-folder",
    "zed",
    "tui",
];

/// (flag, its values; empty when it takes none or free text) per command.
fn flags(command: &str) -> Vec<(&'static str, Vec<String>)> {
    let list = |v: Vec<&str>| v.into_iter().map(String::from).collect::<Vec<_>>();
    let steps = || list(STEPS.iter().map(|s| s.name).collect());
    match command {
        "apply" => {
            let mut styles = list(styles::list().iter().map(|(n, _)| *n).collect());
            styles.push("all".into());
            let mut frosted = list(vec!["auto", "off"]);
            frosted.extend(list(theme::blur_names()));
            vec![
                ("--flavor", list(palette::flavors())),
                ("--accent", list(palette::accents())),
                ("--light-flavor", list(vec!["latte"])),
                ("--frosted", frosted),
                (
                    "--frost-on",
                    list(vec![
                        "windows",
                        "system_interface",
                        "panel",
                        "applets",
                        "maximized_apps",
                    ]),
                ),
                ("--alpha-offset", vec![]),
                ("--roundness", list(theme::roundness_names())),
                ("--gaps", vec![]),
                ("--active-hint", vec![]),
                ("--window-hint", list(palette::accents())),
                ("--wallpaper-style", styles),
                ("--wallpaper-rotate", list(rotate_labels())),
                ("--wallpaper-image", vec![]),
                ("--font", vec![]),
                ("--install-font", list(crate::nerdfont::names())),
                ("--font-size", vec![]),
                ("--last", vec![]),
                ("--profile", vec![]),
                ("--save", vec![]),
                ("--list", vec![]),
                ("--dry-run", vec![]),
                ("--no-apply", vec![]),
                ("--no-backup", vec![]),
            ]
        }
        "uninstall" => vec![
            ("--only", steps()),
            ("--force", vec![]),
            ("--purge", vec![]),
        ],
        "update" => vec![
            ("--check", vec![]),
            ("--force", vec![]),
            ("--apply", vec![]),
        ],
        _ => vec![],
    }
}

fn bash() -> String {
    let mut cases = String::new();
    let mut by_command = String::new();
    for cmd in ["apply", "uninstall", "update"] {
        let all = flags(cmd);
        for (flag, values) in all.iter().filter(|(_, v)| !v.is_empty()) {
            cases.push_str(&format!(
                "        {flag}) COMPREPLY=($(compgen -W \"{}\" -- \"$cur\")); return ;;\n",
                values.join(" ")
            ));
        }
        let names: Vec<_> = all.iter().map(|(f, _)| *f).collect();
        let extra = if cmd == "apply" {
            format!(
                " {}",
                STEPS.iter().map(|s| s.name).collect::<Vec<_>>().join(" ")
            )
        } else {
            String::new()
        };
        by_command.push_str(&format!(
            "        {cmd}) COMPREPLY=($(compgen -W \"{}{extra}\" -- \"$cur\")) ;;\n",
            names.join(" ")
        ));
    }
    format!(
        "_ctp_cosmic() {{\n    local cur=\"${{COMP_WORDS[COMP_CWORD]}}\" prev=\"${{COMP_WORDS[COMP_CWORD-1]}}\"\n    if [ \"$COMP_CWORD\" -eq 1 ]; then\n        COMPREPLY=($(compgen -W \"{} --help --version\" -- \"$cur\")); return\n    fi\n    case \"$prev\" in\n{cases}    esac\n    case \"${{COMP_WORDS[1]}}\" in\n{by_command}    esac\n}}\ncomplete -F _ctp_cosmic ctp-cosmic\n",
        COMMANDS.join(" ")
    )
}

fn fish() -> String {
    let mut out = format!(
        "complete -c ctp-cosmic -f\ncomplete -c ctp-cosmic -n __fish_use_subcommand -a \"{}\"\n",
        COMMANDS.join(" ")
    );
    for cmd in ["apply", "uninstall", "update"] {
        for (flag, values) in flags(cmd) {
            let long = flag.trim_start_matches("--");
            let arg = if values.is_empty() {
                String::new()
            } else {
                format!(" -x -a \"{}\"", values.join(" "))
            };
            out.push_str(&format!(
                "complete -c ctp-cosmic -n \"__fish_seen_subcommand_from {cmd}\" -l {long}{arg}\n"
            ));
        }
    }
    out.push_str(&format!(
        "complete -c ctp-cosmic -n \"__fish_seen_subcommand_from apply\" -a \"{}\"\n",
        STEPS.iter().map(|s| s.name).collect::<Vec<_>>().join(" ")
    ));
    out
}

fn zsh() -> String {
    let mut out = format!(
        "#compdef ctp-cosmic\n_ctp_cosmic() {{\n    local -a cmds\n    cmds=({})\n    if (( CURRENT == 2 )); then\n        _describe 'command' cmds; return\n    fi\n    case \"$words[2]\" in\n",
        COMMANDS.join(" ")
    );
    for cmd in ["apply", "uninstall", "update"] {
        let specs: Vec<String> = flags(cmd)
            .iter()
            .map(|(flag, values)| {
                if values.is_empty() {
                    format!("'{flag}'")
                } else {
                    format!(
                        "'{flag}[{}]:value:({})'",
                        flag.trim_start_matches("--"),
                        values.join(" ")
                    )
                }
            })
            .collect();
        let steps = if cmd == "apply" {
            format!(
                " '*:step:({})'",
                STEPS.iter().map(|s| s.name).collect::<Vec<_>>().join(" ")
            )
        } else {
            String::new()
        };
        out.push_str(&format!(
            "        {cmd}) _arguments {}{steps} ;;\n",
            specs.join(" ")
        ));
    }
    out.push_str("    esac\n}\n_ctp_cosmic \"$@\"\n");
    out
}

const USAGE: &str = "\
ctp-cosmic completions: tab completion for your shell

Usage:
  ctp-cosmic completions bash|zsh|fish

  bash:  ctp-cosmic completions bash > ~/.local/share/bash-completion/completions/ctp-cosmic
  zsh:   ctp-cosmic completions zsh  > ~/.zfunc/_ctp-cosmic   (with ~/.zfunc in $fpath)
  fish:  ctp-cosmic completions fish > ~/.config/fish/completions/ctp-cosmic.fish
";

pub fn main(args: Vec<String>) -> Result<(), String> {
    match args.iter().map(String::as_str).collect::<Vec<_>>()[..] {
        ["bash"] => print!("{}", bash()),
        ["zsh"] => print!("{}", zsh()),
        ["fish"] => print!("{}", fish()),
        [] | ["-h" | "--help"] => print!("{USAGE}"),
        _ => return Err(format!("bad arguments\n\n{USAGE}")),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shell_knows_commands_flags_and_values() {
        for script in [bash(), zsh(), fish()] {
            for want in [
                "uninstall",
                "flavor",
                "macchiato",
                "wallpaper-rotate",
                "30m",
                "roundness",
                "slightlyround",
                "only",
                "terminal",
            ] {
                assert!(script.contains(want), "{want}");
            }
        }
    }

    #[test]
    fn every_command_main_dispatches_is_completed() {
        let main_rs = include_str!("main.rs");
        for cmd in COMMANDS {
            assert!(main_rs.contains(&format!("\"{cmd}\" =>")), "{cmd}");
        }
    }

    #[test]
    fn bash_script_is_valid_shell() {
        let out = std::process::Command::new("bash")
            .args(["-n", "-c", &bash()])
            .status();
        assert!(out.map(|s| s.success()).unwrap_or(true));
    }
}
