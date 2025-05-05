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
        .expect("Failed to build experimental binary");
    let exit = handle.wait().expect("Build command is not running");
    if !exit.success() {
        bail!("Build failed: experimental");
    }

    // Swap to master
    println!("Switching to master");
    let mut handle = Command::new("git")
        .args(["checkout", "master"])
        .spawn()
        .expect("Failed to checkout master");
    let exit = handle.wait().expect("git checkout command is not running");
    if !exit.success() {
        bail!("git checkout failed");
    }

    // Build master
    println!("Building master");
    let maseter_rev = get_rev();
    let mut handle = Command::new("cargo")
        .args(["build", "--release"])
        .spawn()
        .expect("Failed to build master binary");
    let exit = handle.wait().expect("Build command is not running");
    if !exit.success() {
        bail!("Build failed: master");
    }

    // Test
    println!(
        "Testing {} (experimental) against {} (master)",
        &experimental_rev, &maseter_rev
    );

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
