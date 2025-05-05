use std::process::Command;

use anyhow::bail;

fn main() -> Result<(), anyhow::Error> {
    // Build current rev
    println!("Building experimental");
    let experimental_rev = get_rev();
    let mut handle = Command::new("cargo")
        .args(["build", "--release"])
        .env("CARGO_TARGET_DIR", "./target/experimental")
        .spawn()
        .expect("Failed to start experimental build");
    let exit = handle.wait().expect("Build command is not running");
    if !exit.success() {
        bail!("Build failed: experimental");
    }

    // Swap to master
    println!("Switching to master");
    let mut handle = Command::new("git")
        .args(["checkout", "master"])
        .spawn()
        .expect("Failed to start git checkout");
    let exit = handle.wait().expect("git checkout command is not running");
    if !exit.success() {
        bail!("git checkout failed");
    }

    // Build master
    println!("Building master");
    let maseter_rev = get_rev();
    let mut handle = Command::new("cargo")
        .args(["build", "--release"])
        .env("CARGO_TARGET_DIR", "./target/master")
        .spawn()
        .expect("Failed to start master build");
    let exit = handle.wait().expect("Build command is not running");
    if !exit.success() {
        bail!("Build failed: master");
    }

    // Test
    println!(
        "Testing {} (experimental) against {} (master)",
        &experimental_rev, &maseter_rev
    );
    let mut handle = Command::new("cutechess-cli.exe")
        .args([
            "-engine",
            "cmd=./target/experimental/release/fold-chess.exe",
            "-engine",
            "cmd=./target/master/release/fold-chess.exe",
            "-each",
            "proto=uci",
            "tc=40/60",
            "-rounds",
            "10",
        ])
        .spawn()
        .expect("Failed to start cutechess command");
    let exit = handle.wait().expect("cutechess command is not running");
    if !exit.success() {
        bail!("cutechess command failed");
    }

    // Swap back to original branch
    println!("Switching back to experimental branch");
    let mut handle = Command::new("git")
        .args(["switch", "-"])
        .spawn()
        .expect("Failed to start git switch");
    let exit = handle.wait().expect("git switch command is not running");
    if !exit.success() {
        bail!("git switch failed");
    }

    Ok(())
}

fn get_rev() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .expect("Failed to get git rev")
        .stdout;

    let mut output = String::from_utf8(output).expect("Failed to parse command output from utf-8");
    output.pop();
    output
}
