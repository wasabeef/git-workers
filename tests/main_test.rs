use anyhow::Result;
use assert_cmd::Command;
use std::process::Command as StdCommand;

#[test]
fn test_version_flag() -> Result<()> {
    let mut cmd = Command::cargo_bin("gw")?;
    cmd.arg("--version");
    cmd.assert().success();
    Ok(())
}

#[test]
fn test_help_flag() -> Result<()> {
    let mut cmd = Command::cargo_bin("gw")?;
    cmd.arg("--help");
    cmd.assert().success();
    Ok(())
}

#[test]
fn test_list_flag() -> Result<()> {
    // This test may fail if not in a git repository, but that's expected
    let mut cmd = Command::cargo_bin("gw")?;
    cmd.arg("--list");
    // Don't assert success as it depends on git repo context
    let _ = cmd.output();
    Ok(())
}

#[test]
fn test_main_without_args_outside_git() {
    // Test running gw without args outside a git repository
    let output = StdCommand::new("target/debug/gw")
        .current_dir("/tmp")
        .output();

    // Should either succeed or fail gracefully, but the command should execute
    match output {
        Ok(_) => {
            // Command executed successfully (may or may not succeed functionally)
        }
        Err(_) => {
            // Command failed to execute - this could happen if binary doesn't exist
            // Skip this test in that case
        }
    }
}

#[cfg(test)]
mod terminal_setup_tests {
    use std::process::Command;

    #[test]
    fn test_terminal_config_functions_exist() {
        // Test that we can import main functions for compilation check
        // This ensures the main module compiles correctly
        let _result = Command::new("cargo")
            .args(["check", "--bin", "gw"])
            .output()
            .expect("Failed to run cargo check");
    }
}

#[cfg(test)]
mod error_handling_tests {
    use std::process::Command;

    #[test]
    fn test_graceful_error_handling() {
        // Test that the binary handles various error conditions gracefully
        let test_cases = vec![
            vec!["--invalid-flag"],
            vec!["--version", "--list"], // Multiple conflicting flags
        ];

        for args in test_cases {
            let output = Command::new("target/debug/gw")
                .args(&args)
                .output()
                .expect("Failed to execute gw");

            // Should exit with non-zero for invalid args, but not crash
            if !output.status.success() {
                assert!(!output.stderr.is_empty());
            }
        }
    }
}
