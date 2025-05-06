use std::process::{Child, Command};

use anyhow::{Context, bail};

/// Name for the git stash to put any uncommited changes into
const GIT_STASH_NAME: &str = "__internal__ self play stash";

fn main() -> anyhow::Result<()> {
    // Build current rev
    println!("Building experimental");
    let experimental_rev = get_rev();
    let child = Command::new("cargo")
        .args(["build", "--release"])
        .env("CARGO_TARGET_DIR", "./target/experimental")
        .spawn();
    drive_spawned_child(child, "experimental build", true)?;

    // Stash changes, if any
    println!("Stashing uncommitted changes");
    let output = Command::new("git")
        .args(["stash", "-m", GIT_STASH_NAME])
        .output()
        .expect("Failed to start command: git stash")
        .stdout;
    let output = String::from_utf8(output).expect("Failed to parse command output from utf-8");
    let stashed = output != "No local changes to save\n";

    // Swap to master
    println!("Switching to master");
    let child = Command::new("git").args(["checkout", "master"]).spawn();
    drive_spawned_child(child, "checkout master", true)?;

    // Build master
    println!("Building master");
    let maseter_rev = get_rev();
    let child = Command::new("cargo")
        .args(["build", "--release"])
        .env("CARGO_TARGET_DIR", "./target/master")
        .spawn();
    drive_spawned_child(child, "master build", true)?;

    // Reset git state
    // Note: Since all the build artifacts end up in the gitignore'd target folder,
    // once they are built, we can freely mess with git however we wish.
    // We reset git before running the test for convenience, as tests can take a long time.
    println!("Switching back to experimental branch");
    let child = Command::new("git").args(["switch", "-"]).spawn();
    drive_spawned_child(child, "git switch", true)?;

    if stashed {
        println!("Applying previously stashed changes");
        let child = Command::new("git")
            .args(["stash", "pop", GIT_STASH_NAME])
            .spawn();
        drive_spawned_child(child, "git stash pop", false)?; // Ignore exit code, as this can fail if there is no stash
    }

    // Test
    println!(
        "Testing {} (experimental) against {} (master)",
        &experimental_rev, &maseter_rev
    );
    let child = Command::new("cutechess-cli.exe")
        .args([
            "-engine",
            "cmd=./target/experimental/release/patch.exe",
            "name=patch-experimental",
            "-engine",
            "cmd=./target/master/release/patch.exe",
            "name=patch-master",
            "-each",
            "proto=uci",
            "tc=20/20",
            "-rounds",
            "1000",
            "-sprt",
            "elo0=0",
            "elo1=10",
            "alpha=0.05",
            "beta=0.05",
        ])
        .spawn();
    drive_spawned_child(child, "cutechess", true)?;

    Ok(())
}

fn get_rev() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .expect("Failed to start command: git rev")
        .stdout;

    let mut output = String::from_utf8(output).expect("Failed to parse command output from utf-8");
    output.pop();
    output
}

/// Waits for a spawned child process to finish, spitting out errors if anything goes wrong
///
/// Inputs:
/// - `child`: The spawned child to drive
/// - `command_name`: Semantic command name; only used for error messages
/// - `require_success`: Return an error if the command returns a non-zero exit code
fn drive_spawned_child(
    child: std::io::Result<Child>,
    command_name: &str,
    require_success: bool,
) -> anyhow::Result<()> {
    let exit = child
        .context(format!("Failed to start command: {}", command_name))?
        .wait()
        .context(format!("Command is not running: {}", command_name))?;

    if !exit.success() && require_success {
        bail!("Command failed: {}", command_name);
    }

    Ok(())
}
