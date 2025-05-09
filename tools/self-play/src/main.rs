use std::process::{Child, Command};

use anyhow::{Context, bail};
use clap::Parser;

/// Name for the git stash to put any uncommited changes into
const GIT_STASH_NAME: &str = "__internal__ self play stash";

/// Utility to set up the engine to play against an older version of itself
#[derive(Parser, Debug)]
struct Args {
    /// Time control to use
    #[arg(short, long, default_value = "40/60")]
    tc: String,

    /// Number of games to run concurrently
    #[arg(short, long, default_value_t = 4)]
    concurrency: u8,

    /// Elo 0 to use for the SPRT test
    #[arg(short = 'n', long)]
    elo0: f32,

    /// Elo 1 to use for the SPRT test
    #[arg(short = 'a', long)]
    elo1: f32,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

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
        let child = Command::new("git").args(["stash", "pop"]).spawn();
        drive_spawned_child(child, "git stash pop", false)?; // Ignore exit code, as this can fail if there is no stash
    }

    // Test
    let ext = if cfg!(windows) { ".exe" } else { "" };

    println!(
        "Testing {} (experimental) against {} (master)",
        &experimental_rev, &maseter_rev
    );
    let child = Command::new(format!("cutechess-cli{}", ext))
        .args([
            "-engine",
            &format!("cmd=./target/experimental/release/patch{}", ext),
            "name=patch-experimental",
            "-engine",
            &format!("cmd=./target/master/release/patch{}", ext),
            "name=patch-master",
            "-concurrency",
            &args.concurrency.to_string(),
            "-each",
            "proto=uci",
            &format!("tc={}", args.tc),
            "timemargin=500",
            "-rounds",
            "10000",
            "-sprt",
            &format!("elo0={}", args.elo0),
            &format!("elo1={}", args.elo1),
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
