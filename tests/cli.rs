#![cfg(not(target_os = "wasi"))] // Not supported yet

use rstest::rstest;

struct Output {
    status: std::process::ExitStatus,
    stdout: String,
    stderr: String,
}

fn run_kibi(args: &[&str]) -> Result<Output, Box<dyn std::error::Error>> {
    let binary_path = std::env!("CARGO_BIN_EXE_kibi");
    let mut command = std::process::Command::new(binary_path);
    command.args(args);
    eprintln!("Running command {binary_path} {args}", args = args.join(" "));
    let start = std::time::Instant::now();
    let output = command.output()?;
    eprintln!(
        "{icon} Exited after {duration} with\n     Status: {status}\n     Stdout: {stdout}\n     \
         Stderr: {stderr}",
        icon = if output.status.success() { "✔️" } else { "❌" },
        duration = start.elapsed().as_secs_f32(),
        status = output.status,
        stdout = String::from_utf8_lossy(&output.stdout),
        stderr = String::from_utf8_lossy(&output.stderr),
    );
    Ok(Output {
        status: output.status,
        stdout: String::from_utf8(output.stdout)?,
        stderr: String::from_utf8(output.stderr)?,
    })
}

#[rstest]
#[case(&["--version"])]
#[case(&["--version", "--"])]
fn version(#[case] args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let output = run_kibi(args)?;
    assert!(output.status.success());
    assert_eq!(output.stdout, format!("kibi {}\n", std::env!("CARGO_PKG_VERSION")));
    Ok(())
}

#[rstest]
#[case(&["-i"])]
#[case(&["-i", "--"])]
#[case(&["--invalid"])]
#[case(&["--invalid", "--"])]
#[case(&["--version", "abc"])]
#[case(&["--version", "--", "abc"])]
#[case(&["--version", "abc", "--"])]
fn invalid_option(#[case] args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let output = run_kibi(args)?;
    assert!(!output.status.success());
    assert_eq!(output.stderr, format!("Error: BadOption(\"{}\")\n", &args[0]));
    Ok(())
}

#[rstest]
#[case(&["abc", "def"])]
#[case(&["abc", "--version"])]
#[case(&["--", "abc", "def"])]
#[case(&["--", "abc", "--version"])]
#[case(&["abc", "--", "def"])]
#[case(&["abc", "--", "--version"])]
#[case(&["abc", "--", "--"])]
fn too_many_arguments(#[case] args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let output = run_kibi(args)?;
    assert!(!output.status.success());
    assert_eq!(
        output.stderr,
        format!(
            "Error: TooManyArguments([\"{}\", {}])\n",
            std::env!("CARGO_BIN_EXE_kibi").escape_debug(),
            args.iter().map(|arg| format!("{arg:?}")).collect::<Vec::<_>>().join(", ")
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

#[rstest]
#[case(&["abc"])]
#[case(&["--", "abc"])]
#[case(&["--", "-not-an-option"])]
#[case(&["abc", "--"])]
#[case(&["--", "--"])]
fn with_file_name(#[case] args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    // Can't test without a terminal
    let output = run_kibi(args)?;
    assert!(!output.status.success());
    assert!(output.stderr.contains("Error: Io"));
    Ok(())
}
