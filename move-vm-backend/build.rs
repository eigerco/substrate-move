use std::error::Error;
use std::process::Command;

fn main() -> Result<(), Box<dyn Error>> {
    // Build move projects for the test purposes.
    #[cfg(feature = "build-move-projects-for-test")]
    build_move_projects()?;

    // SMOVE build our deposit code
    Command::new("move")
        .args(["build", "-p", "../deposit"])
        .output()
        .expect("failed to execute process");
    // copy new binary module so it can be properly included
    Command::new("cp")
        .args([
            "../deposit/build/deposit/bytecode_modules/deposit.mv",
            "../contracts",
        ])
        .output()
        .expect("failed to copy new deposit.mv");
    // SMOVE build our deposit script
    Command::new("move")
        .args(["build", "-p", "../deposit/executor"])
        .output()
        .expect("failed to execute process");
    println!("script copy");
    Command::new("cp")
        .args([
            "../deposit/executor/build/executor/bytecode_scripts/transfer.mv",
            "../contracts",
        ])
        .output()
        .expect("failed to copy new transfer.mv");
    // std module required
    Command::new("cp")
        .args([
            "../deposit/build/deposit/bytecode_modules/dependencies/MoveStdlib/signer.mv",
            "../contracts",
        ])
        .output()
        .expect("failed to copy new transfer.mv");
    Ok(())
}

#[allow(dead_code)]
fn build_move_projects() -> Result<(), Box<dyn Error>> {
    println!("cargo:warning=Building move projects in tests/assets folder");

    let smove_run = Command::new("bash")
        .args(["tests/assets/move-projects/smove-build-all.sh"])
        .output()
        .expect("failed to execute script which builds necessary move modules");

    if !smove_run.status.success() {
        let stderr = std::str::from_utf8(&smove_run.stderr)?;

        let e = Box::<dyn Error + Send + Sync>::from(stderr);
        return Err(e);
    }

    println!("cargo:warning=Move projects built successfully");
    // Rerun in case Move source files are changed.
    println!("cargo:rerun-if-changed=tests/assets/move-projects");

    Ok(())
}
