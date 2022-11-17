use std::process::Command;

fn main() {
    let version = match Command::new("git").args(["describe", "--tags", "--match=v*"]).output() {
        Ok(output) if output.status.success() =>
            String::from_utf8_lossy(&output.stdout[1..]).replacen('-', ".r", 1).replace('-', "."),
        _ => env!("CARGO_PKG_VERSION").into(),
    };
    println!("cargo:rustc-env=KIBI_VERSION={version}");
}
