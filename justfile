# Workspace tasks. The CLI is the root crate; the graphical app is in gui/
# (see gui/justfile for installing it).

default:
    @just --list

# Builds the CLI (release)
build:
    cargo build --release --locked

# Builds the graphical app (release)
build-gui:
    cargo build --release --locked -p ctp-cosmic-gui

# Tests the CLI and the graphical app
test:
    cargo test --locked --workspace

# Clippy for everything
check:
    cargo clippy --locked --workspace --all-targets

# Formats the whole workspace
fmt:
    cargo fmt --all

# Runs the graphical app
gui *args:
    cargo run --release --locked -p ctp-cosmic-gui -- {{args}}
