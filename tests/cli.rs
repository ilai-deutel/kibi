#![cfg(not(target_os = "wasi"))] // Not supported yet

use log::info;

struct Output {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}

fn run_kibi(args: &[&str]) -> Result<Output, Box<dyn std::error::Error>> {
    let _ = env_logger::builder().is_test(true).try_init();
    let binary_path = std::env!("CARGO_BIN_EXE_kibi");
    let mut command = std::process::Command::new(binary_path);
    command.args(args);
    info!("Running {command:?}");
    let start = std::time::Instant::now();
    let output = command.output()?;
    info!(
        "{}Exited after {:?} with {:#?}",
        if output.status.success() { "✔️" } else { "❌" },
        start.elapsed(),
        output
    );
    Ok(Output {
        status: output.status,
        stdout: String::from_utf8(output.stdout)?,
        stderr: String::from_utf8(output.stderr)?,
    })
}

#[test]
fn version() -> Result<(), Box<dyn std::error::Error>> {
    let output = run_kibi(&["--version"])?;
    assert!(output.status.success());
    assert_eq!(output.stdout, format!("kibi {}\n", std::env!("KIBI_VERSION")));
    Ok(())
}

#[test]
fn invalid_option() -> Result<(), Box<dyn std::error::Error>> {
    let output = run_kibi(&["--invalid"])?;
    assert!(!output.status.success());
    assert_eq!(output.stderr, "Error: UnrecognizedOption(\"--invalid\")\n");
    Ok(())
}

#[test]
fn too_many_arguments() -> Result<(), Box<dyn std::error::Error>> {
    let output = run_kibi(&["abc", "def"])?;
    assert!(!output.status.success());
    assert_eq!(
        output.stderr,
        format!(
            "Error: TooManyArguments([\"{}\", \"abc\", \"def\"])\n",
            std::env!("CARGO_BIN_EXE_kibi").escape_debug()
        )
    );
    Ok(())
}

#[test]
fn no_argument() -> Result<(), Box<dyn std::error::Error>> {
    let output = run_kibi(&[])?;
    // Can't test without a terminal
    assert!(!output.status.success());
    assert!(&output.stderr.contains("Error: Io"));
    Ok(())
}

#[test]
fn with_file_name() -> Result<(), Box<dyn std::error::Error>> {
    // Can't test without a terminal
    let output = run_kibi(&["test.txt"])?;
    assert!(!output.status.success());
    assert!(output.stderr.contains("Error: Io"));
    Ok(())
}
