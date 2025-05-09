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

# Check code
[group('testing')]
check:
    cargo check
alias c := check

# Play a tournament against itself, looking for an improvement in ELO
[group('testing')]
self-play-gain *args:
    cargo run --release --package self-play -- --elo0 0 --elo1 10 {{ args }}
alias spg := self-play-gain

# Play a tournament against itself, looking for a regression in ELO
[group('testing')]
self-play-regression *args:
    cargo run --release --package self-play -- --elo0=-10 --elo1 0 {{ args }}
alias spr := self-play-regression
