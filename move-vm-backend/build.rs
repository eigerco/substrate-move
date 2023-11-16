use std::process::Command;

fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    //println!("cargo:rerun-if-changed=../deposit/sources/deposit.move");
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
}
