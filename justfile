set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]
just := just_executable()

# Lists all the runnable recipes
@_default:
    just --list --unsorted

##################### Running #####################

# Run in debug
[group('running')]
run:
    cargo run
alias r := run

# Run in release
[group('running')]
run-release:
    cargo run --release
alias rr := run-release

# Build binary
[group('running')]
build:
    cargo build
alias b := build

# Build in release
[group('running')]
build-release:
    cargo build --release
alias br := build-release

##################### Testing #####################

# Play a tournament against itself
[group('testing')]
self-play:
    cargo run --release --package self-play
alias sp := self-play
