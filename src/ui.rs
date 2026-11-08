//! The messages steps print, in the palette's own colors. The interactive
//! setup reads them back from the output (`==>` starts a step, ` ✓` is done,
//! ` !` a warning, ` ➜` something left for the user), so keep the shapes.

/// Colors only on a terminal, and never with NO_COLOR (https://no-color.org).
fn line(rgb: &str, mark: &str, msg: &str) {
    use std::io::IsTerminal;
    if std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none() {
        println!("\x1b[38;2;{rgb}m{mark}\x1b[0m {msg}");
    } else {
        println!("{mark} {msg}");
    }
}

pub fn info(msg: &str) {
    line("137;180;250", "==>", msg); // blue
}
pub fn ok(msg: &str) {
    line("166;227;161", " ✓", msg); // green
}
pub fn warn(msg: &str) {
    line("249;226;175", " !", msg); // yellow
}
pub fn todo(msg: &str) {
    line("203;166;247", " ➜", msg); // mauve
}
