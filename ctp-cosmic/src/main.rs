//! ctp-cosmic: Catppuccin for the COSMIC desktop.
//!
//! One binary for everything catppuccin-cosmic-setup computes: the palette
//! (for the shell scripts), desktop themes in the COSMIC 1.10 format, and
//! procedural wallpapers. Colors come from the official `catppuccin` crate.

mod folders;
mod image;
mod noise;
mod palette;
mod prompt;
mod steps;
mod styles;
mod theme;
mod tui;
mod wallpaper;
mod zed;

use std::process::ExitCode;

const USAGE: &str = "\
ctp-cosmic: Catppuccin for the COSMIC desktop

Usage:
  ctp-cosmic palette ...                     palette lookups for scripts
  ctp-cosmic theme ...                       desktop themes (COSMIC 1.10)
  ctp-cosmic wallpaper ...                   procedural wallpapers
  ctp-cosmic register-wallpaper-folder ...   add a folder to COSMIC's wallpaper picker
  ctp-cosmic prompt ...                      bash prompt and its ~/.bashrc block
  ctp-cosmic steps ...                       the installer steps (setup.sh reads them)
  ctp-cosmic zed ...                         configure Zed settings (theme and icon theme)
  ctp-cosmic tui --repo DIR                  interactive setup (setup.sh opens it)
  ctp-cosmic --version

Run `ctp-cosmic <command> --help` for each command's options.
";

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_default();
    let rest: Vec<String> = args.collect();
    let result = match command.as_str() {
        "palette" => palette::main(rest),
        "theme" => theme::main(rest).map(|()| 0),
        "wallpaper" => wallpaper::main(rest).map(|()| 0),
        "register-wallpaper-folder" => folders::main(rest).map(|()| 0),
        "prompt" => prompt::main(rest).map(|()| 0),
        "steps" => steps::main(rest).map(|()| 0),
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
        "" => Err(format!("missing command\n\n{USAGE}")),
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
