//! Performance benchmarks for Git Workers
//!
//! This module contains benchmarks for measuring performance
//! of critical operations like worktree creation and branch listing.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use git_workers::git::GitWorktreeManager;
use tempfile::TempDir;

fn benchmark_worktree_creation(c: &mut Criterion) {
    // Setup
    let temp_dir = TempDir::new().unwrap();
    std::process::Command::new("git")
        .arg("init")
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    
    // Create initial commit
    std::fs::write(temp_dir.path().join("README.md"), "# Test").unwrap();
    std::process::Command::new("git")
        .arg("add")
        .arg(".")
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg("Initial commit")
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    
    let manager = GitWorktreeManager::new_from_path(temp_dir.path()).unwrap();
    
    c.bench_function("worktree_creation", |b| {
        let mut counter = 0;
        b.iter(|| {
            let name = format!("bench-{counter}");
            let path = temp_dir.path().join(&name);
            manager.create_worktree_from_head(&path, &name).unwrap();
            counter += 1;
            black_box(name);
        });
    });
}

fn benchmark_branch_listing(c: &mut Criterion) {
    // Setup with multiple branches
    let temp_dir = TempDir::new().unwrap();
    std::process::Command::new("git")
        .arg("init")
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    
    // Create initial commit
    std::fs::write(temp_dir.path().join("README.md"), "# Test").unwrap();
    std::process::Command::new("git")
        .arg("add")
        .arg(".")
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg("Initial commit")
        .current_dir(temp_dir.path())
        .output()
        .unwrap();
    
    // Create many branches
    for i in 0..100 {
        std::process::Command::new("git")
            .args(["branch", &format!("branch-{i}")])
            .current_dir(temp_dir.path())
            .output()
            .unwrap();
    }
    
    let manager = GitWorktreeManager::new_from_path(temp_dir.path()).unwrap();
    
    c.bench_function("branch_listing", |b| {
        b.iter(|| {
            let (local, remote) = manager.list_all_branches().unwrap();
            black_box((local, remote));
        });
    });
}

criterion_group!(benches, benchmark_worktree_creation, benchmark_branch_listing);
criterion_main!(benches);