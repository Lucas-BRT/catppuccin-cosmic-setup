//! The messages steps print, in the palette's own colors. The interactive
//! setup reads them back from the output (`==>` starts a step, ` ✓` is done,
//! ` !` a warning, ` ➜` something left for the user), so keep the shapes.

fn line(rgb: &str, mark: &str, msg: &str) {
    println!("\x1b[38;2;{rgb}m{mark}\x1b[0m {msg}");
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
