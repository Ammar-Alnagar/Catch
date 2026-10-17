use std::ffi::OsStr;
use std::path::Path;
use std::process::{Command, Stdio};
use anyhow::{anyhow, Result};

pub fn run_command<I, S>(cmd_args: I, cwd: Option<&Path>, verbose: bool, check: bool, capture_output: bool) -> Result<std::process::Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let args: Vec<_> = cmd_args.into_iter().collect();
    if args.is_empty() {
        return Err(anyhow!("Empty command passed to run_command"));
    }

    let cmd_str = args
        .iter()
        .map(|a| a.as_ref().to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(" ");

    println!("Running: {}", cmd_str);

    let mut command = Command::new(&args[0]);
    if args.len() > 1 {
        command.args(&args[1..]);
    }

    if let Some(dir) = cwd {
        command.current_dir(dir);
    }

    if capture_output {
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());
    } else {
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());
    }

    let output = command
        .output()
        .map_err(|e| anyhow!("Executable not found or failed to launch: {}: {}", args[0].as_ref().to_string_lossy(), e))?;

    if check && !output.status.success() {
        eprintln!("Error executing command: {}", cmd_str);
        if verbose && !output.stdout.is_empty() {
            eprintln!("Stdout:\n{}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            eprintln!("Stderr:\n{}", String::from_utf8_lossy(&output.stderr));
        }
        return Err(anyhow!(
            "Command failed with exit code {:?}: {}",
            output.status.code(),
            cmd_str
        ));
    }

    Ok(output)
}
