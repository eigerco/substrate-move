use std::error::Error;
use std::process::Command;

fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(feature = "stdlib-bytecode")]
    build_stdlib_with_smove()?;

    Ok(())
}

#[allow(dead_code)]
fn build_stdlib_with_smove() -> Result<(), Box<dyn Error>> {
    let smove_run = Command::new("smove")
        .args(["bundle"])
        .output()
        .expect("failed to execute process");

    if !smove_run.status.success() {
        let stderr = std::str::from_utf8(&smove_run.stderr)?;

        let e = Box::<dyn Error + Send + Sync>::from(stderr);
        return Err(e);
    }

    // Rerun in case Move source files are changed.
    println!("cargo:rerun-if-changed=sources/");
    Ok(())
}
