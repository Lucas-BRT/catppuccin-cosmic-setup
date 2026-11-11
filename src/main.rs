//! ctp-cosmic: Catppuccin for the COSMIC desktop.
//!
//! One binary for everything catppuccin-cosmic-setup computes: the palette
//! (for the shell scripts), desktop themes in the COSMIC 1.10 format, and
//! procedural wallpapers. Colors come from the official `catppuccin` crate.

use std::process::ExitCode;

use ctp_cosmic::{
    apply, apps, background, backup, completions, folders, health, icons, palette, profiles,
    prompt, terminal, theme, tui, uninstall, update, wallpaper, zed,
};

const USAGE: &str = "\
ctp-cosmic: Catppuccin for the COSMIC desktop

Usage:
  ctp-cosmic [apply] ...                     put Catppuccin on the desktop, terminal,
                                             prompt, Zed, icons and wallpaper
                                             (interactive in a terminal, no options)
  ctp-cosmic status ...                      is what the last apply chose still in place?
  ctp-cosmic doctor ...                      is everything apply needs on this machine?
  ctp-cosmic completions bash|zsh|fish       tab completion
  ctp-cosmic profile ...                     named sets of choices (apply --save/--profile)
  ctp-cosmic update ...                      install the latest release
  ctp-cosmic uninstall ...                   undo the last apply
  ctp-cosmic theme ...                       desktop themes (COSMIC 1.10)
  ctp-cosmic wallpaper ...                   procedural wallpapers
  ctp-cosmic terminal ...                    COSMIC Terminal color schemes and font
  ctp-cosmic palette ...                     palette lookups
  ctp-cosmic gui                             the graphical app (ctp-cosmic-gui)
  ctp-cosmic --version

The pieces apply is made of are commands too: backup, background, icons,
prompt, register-wallpaper-folder, zed.
Run `ctp-cosmic <command> --help` for each command's options.
";

/// Replaces this process with `ctp-cosmic-gui`: the one next to this binary,
/// else the one on PATH. It is a separate program (it needs libcosmic).
fn gui(args: Vec<String>) -> Result<u8, String> {
    use std::os::unix::process::CommandExt;
    const NAME: &str = "ctp-cosmic-gui";
    let beside = std::env::current_exe()
        .ok()
        .and_then(|exe| Some(exe.parent()?.join(NAME)))
        .filter(|p| p.is_file());
    let err = std::process::Command::new(beside.as_deref().unwrap_or(NAME.as_ref()))
        .args(args)
        .exec();
    Err(if err.kind() == std::io::ErrorKind::NotFound {
        format!("{NAME} is not installed; build it with `just install` in gui/ of the repository")
    } else {
        format!("{NAME}: {err}")
    })
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_default();
    let rest: Vec<String> = args.collect();
    let result = match command.as_str() {
        "apply" => apply::main(rest),
        "profile" => profiles::main(rest).map(|()| 0),
        "completions" => completions::main(rest).map(|()| 0),
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
        "app" => apps::main(rest),
        "tui" => tui::main(rest),
        "gui" => gui(rest),
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
