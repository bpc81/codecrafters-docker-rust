use anyhow::{Context, Result};
use std::process::Stdio;
use std::fs::{
    copy, create_dir, create_dir_all,
    set_permissions, Permissions};
use std::os::unix::fs::{chroot, PermissionsExt};
use std::env::set_current_dir;
use tempfile::TempDir;

fn copy_executable(executable: &String, temp_dir: &TempDir) -> Result<()> {
    let command_path_relative = executable.trim_start_matches("/");
    let target_command = temp_dir.path().join(command_path_relative);
    let target_path = target_command.parent().unwrap();
    create_dir_all(target_path)?;
    copy(executable, target_command)?;

    Ok(())
}

fn create_dev_null(temp_dir: &TempDir) -> Result<()> {
    let path_dev = temp_dir.path().join("dev");
    let path_dev_null = path_dev.join("null");
    create_dir(&path_dev)?;
    set_permissions(&path_dev, Permissions::from_mode(0o555))?;
    create_dir(&path_dev_null)?;
    set_permissions(&path_dev_null, Permissions::from_mode(0o555))?;

    Ok(())
}

fn change_root(temp_dir: TempDir) -> Result<()> {
    chroot(temp_dir.path())?;

    set_current_dir("/")?;

    Ok(())
}

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage!
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];

    let exit_code = run_child(command, command_args)?;
    std::process::exit(exit_code);
}

fn run_child(command: &String, command_args: &[String]) -> Result<i32> {
    let temp_dir = tempfile::tempdir()?;

    copy_executable(command, &temp_dir)?;

    create_dev_null(&temp_dir)?;

    change_root(temp_dir)?;

    let output = std::process::Command::new(command)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .args(command_args)
        .output()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;

    let exit_code = output.status.code().unwrap_or(1);
    Ok(exit_code)
}
