use std::io::Write;
use std::process::{Command, Stdio};

#[test]
#[ignore = "Interactive test - requires terminal and is time-consuming"]
fn test_esc_key_handling() {
    // テスト用のgitリポジトリを作成
    let temp_dir = tempfile::tempdir().unwrap();
    let repo_path = temp_dir.path();

    // gitリポジトリを初期化
    Command::new("git")
        .args(["init"])
        .current_dir(repo_path)
        .output()
        .expect("Failed to init git repo");

    // 初期コミットを作成
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

    // バイナリのパスを取得
    let binary_path = std::env::current_dir().unwrap().join("target/debug/gw");

    // プロセスを起動
    let mut child = Command::new(&binary_path)
        .current_dir(repo_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start process");

    let stdin = child.stdin.as_mut().expect("Failed to get stdin");

    // 1. Create worktreeを選択 (4番目の項目)
    stdin.write_all(b"\x1b[B\x1b[B\x1b[B\r").unwrap(); // 下矢印3回 + Enter

    // 2. ESCキーを送信してキャンセル
    stdin.write_all(b"\x1b").unwrap(); // ESC

    // 3. Exitを選択
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

    println!("STDOUT:\n{}", stdout);
    println!("STDERR:\n{}", stderr);

    // ESCキーでキャンセルされたことを確認
    assert!(stdout.contains("Operation cancelled") || stderr.contains("Operation cancelled"));
}

#[test]
#[ignore = "Interactive test - requires terminal and is time-consuming"]
fn test_search_esc_handling() {
    let temp_dir = tempfile::tempdir().unwrap();
    let repo_path = temp_dir.path();

    // gitリポジトリを初期化
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

    // 1. Search worktreesを選択 (3番目の項目)
    stdin.write_all(b"\x1b[B\x1b[B\r").unwrap(); // 下矢印2回 + Enter

    // 2. ESCキーを送信してキャンセル
    stdin.write_all(b"\x1b").unwrap(); // ESC

    // 3. Exitを選択
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

    println!("STDOUT:\n{}", stdout);
    println!("STDERR:\n{}", stderr);

    // ESCキーでキャンセルされたことを確認
    assert!(stdout.contains("Search cancelled") || stderr.contains("Search cancelled"));
}
