//! ctp-cosmic: Catppuccin for the COSMIC desktop.
//!
//! One binary for everything catppuccin-cosmic-setup computes: the palette
//! (for the shell scripts), desktop themes in the COSMIC 1.10 format, and
//! procedural wallpapers. Colors come from the official `catppuccin` crate.

mod apply;
mod background;
mod backup;
mod folders;
mod health;
mod icons;
mod image;
mod noise;
mod palette;
mod prompt;
mod steps;
mod styles;
mod terminal;
mod theme;
mod tui;
mod ui;
mod uninstall;
mod update;
mod wallpaper;
mod zed;

use std::process::ExitCode;

const USAGE: &str = "\
ctp-cosmic: Catppuccin for the COSMIC desktop

Usage:
  ctp-cosmic [apply] ...                     put Catppuccin on the desktop, terminal,
                                             prompt, Zed, icons and wallpaper
                                             (interactive in a terminal, no options)
  ctp-cosmic status ...                      is what the last apply chose still in place?
  ctp-cosmic doctor ...                      is everything apply needs on this machine?
  ctp-cosmic update ...                      install the latest release
  ctp-cosmic uninstall ...                   undo the last apply
  ctp-cosmic theme ...                       desktop themes (COSMIC 1.10)
  ctp-cosmic wallpaper ...                   procedural wallpapers
  ctp-cosmic terminal ...                    COSMIC Terminal color schemes and font
  ctp-cosmic palette ...                     palette lookups
  ctp-cosmic --version

The pieces apply is made of are commands too: backup, background, icons,
prompt, register-wallpaper-folder, zed.
Run `ctp-cosmic <command> --help` for each command's options.
";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_default();
    let rest: Vec<String> = args.collect();
    let result = match command.as_str() {
        "apply" => apply::main(rest),
        "doctor" => health::doctor(rest),
        "status" => health::status(rest),
        "uninstall" => uninstall::main(rest).map(|()| 0),
        "update" => update::main(rest),
        "backup" => backup::main(rest).map(|()| 0),
        "background" => background::main(rest).map(|()| 0),
        "icons" => icons::main(rest).map(|()| 0),
        "palette" => palette::main(rest),
        "theme" => theme::main(rest).map(|()| 0),
        "wallpaper" => wallpaper::main(rest).map(|()| 0),
        "register-wallpaper-folder" => folders::main(rest).map(|()| 0),
        "prompt" => prompt::main(rest).map(|()| 0),
        "terminal" => terminal::main(rest),
        "zed" => zed::main(rest).map(|()| 0),
        "tui" => tui::main(rest),
        "-V" | "--version" => {
            println!("ctp-cosmic {}", env!("CARGO_PKG_VERSION"));
            Ok(0)
        }
        "-h" | "--help" => {
            print!("{USAGE}");
            Ok(0)
        }
        "" => apply::main(rest),
        other => Err(format!("unknown command '{other}'\n\n{USAGE}")),
    };
    match result {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            eprintln!("ctp-cosmic: {e}");
            ExitCode::FAILURE
        }
    }
}
