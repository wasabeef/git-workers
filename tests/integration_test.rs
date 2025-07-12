use std::io::Write;
use std::process::{Command, Stdio};

#[test]
#[ignore = "Interactive test - requires terminal and is time-consuming"]
fn test_esc_key_handling() {
    // Create a git repository for testing
    let temp_dir = tempfile::tempdir().unwrap();
    let repo_path = temp_dir.path();

    // Initialize git repository
    Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to init git repo");

    // Create initial commit
    std::fs::write(repo_path.join("README.md"), "# Test").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(repo_path)
        .output()
        .expect("Failed to add files");

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to commit");

    // Get binary path
    let binary_path = std::env::current_dir().unwrap().join("target/debug/gw");

    // Start process
    let mut child = Command::new(&binary_path)
        .current_dir(repo_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start process");

    let stdin = child.stdin.as_mut().expect("Failed to get stdin");

    // 1. Select Create worktree (4th item)
    stdin.write_all(b"\x1b[B\x1b[B\x1b[B\r").unwrap(); // 下矢印3回 + Enter

    // 2. Send ESC key to cancel
    stdin.write_all(b"\x1b").unwrap(); // ESC

    // 3. Select Exit
    stdin
        .write_all(b"\x1b[B\x1b[B\x1b[B\x1b[B\x1b[B\r")
        .unwrap(); // 下矢印5回 + Enter

    stdin.flush().unwrap();
    let _ = stdin;

    let output = child
        .wait_with_output()
        .expect("Failed to wait for process");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("STDOUT:\n{stdout}");
    println!("STDERR:\n{stderr}");

    // Confirm that it was cancelled with ESC key
    assert!(stdout.contains("Operation cancelled") || stderr.contains("Operation cancelled"));
}

#[test]
#[ignore = "Interactive test - requires terminal and is time-consuming"]
fn test_search_esc_handling() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo_path = temp_dir.path();

    // Initialize git repository
    Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to init git repo");

    std::fs::write(repo_path.join("README.md"), "# Test").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(repo_path)
        .output()
        .expect("Failed to add files");

    Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to commit");

    let binary_path = std::env::current_dir().unwrap().join("target/debug/gw");

    let mut child = Command::new(&binary_path)
        .current_dir(repo_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start process");

    let stdin = child.stdin.as_mut().expect("Failed to get stdin");

    // 1. Select Search worktrees (3rd item)
    stdin.write_all(b"\x1b[B\x1b[B\r").unwrap(); // 下矢印2回 + Enter

    // 2. Send ESC key to cancel
    stdin.write_all(b"\x1b").unwrap(); // ESC

    // 3. Select Exit
    stdin
        .write_all(b"\x1b[B\x1b[B\x1b[B\x1b[B\x1b[B\x1b[B\r")
        .unwrap(); // 下矢印6回 + Enter

    stdin.flush().unwrap();
    let _ = stdin;

    let output = child
        .wait_with_output()
        .expect("Failed to wait for process");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    println!("STDOUT:\n{stdout}");
    println!("STDERR:\n{stderr}");

    // Confirm that it was cancelled with ESC key
    assert!(stdout.contains("Search cancelled") || stderr.contains("Search cancelled"));
}
