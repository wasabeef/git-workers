# Worktree Entry Latency Plan

## Goal

`create` と `batch delete` の初期レスポンスを改善する。

今回の主眼は「既存の挙動を壊さずに速くすること」であり、UI 文言や選択フローの変更は目的に含めない。

## Current Status

以下は実装済み。

- Phase 1
  - `create` は `has_linked_worktrees()` を使う
- Phase 2
  - `batch delete` は `list_worktrees_basic()` を使う
  - `is_branch_unique_to_worktree()` は軽量一覧ベース
- Phase 3
  - `rename / switch / search` も軽量一覧を使う

以下は意図的に未変更。

- `list`
  - 詳細表示のため、引き続き `list_worktrees()` を使う
- `WorktreeInfo`
  - public API と既存 test を壊さないため維持する
- detailed status 取得
  - `has_changes / last_commit / ahead_behind` は `list` 系のために残す

## Observed Symptoms

- `create` は worktree 数が増えると、画面に入って最初の prompt が出るまで遅くなる
- `batch delete` は worktree 数が増えると、画面表示前と選択後の summary 生成で遅くなる

## Root Cause Analysis

### 1. `create` の入口が重い

対象:
- [create_worktree_with_ui](/Users/a12622/git/git-workers/src/usecases/create_worktree.rs:79)

現状:
- 冒頭で `manager.list_worktrees()?` を呼んでいる
- 使い道は `has_worktrees = !existing_worktrees.is_empty()` のみ

問題:
- 件数確認しかしたくないのに、一覧表示向けの重い取得をしている

### 2. `batch delete` の入口が重い

対象:
- [batch_delete_worktrees_internal](/Users/a12622/git/git-workers/src/usecases/delete_worktree.rs:240)

現状:
- 冒頭で `manager.list_worktrees()?` を呼んでいる
- 候補生成には `name / branch / current` 程度しか使っていない

問題:
- 入口で full status を取りにいっている

### 3. `batch delete` の後段も重い

対象:
- [is_branch_unique_to_worktree](/Users/a12622/git/git-workers/src/infrastructure/git.rs:1156)

現状:
- 内部で再び `self.list_worktrees()?` を呼んでいる
- 選択した worktree 数ぶん呼ばれる

問題:
- 選択後の summary 生成が `O(selected_count * full_list_cost)` になる

### 4. 重い本体 API

対象:
- [GitWorktreeManager::list_worktrees](/Users/a12622/git/git-workers/src/infrastructure/git.rs:240)

現状:
- 各 worktree ごとに以下を取得している
  - `Repository::open(path)`
  - branch 名
  - `repo.statuses(...)`
  - last commit

結論:
- `list_worktrees()` は一覧画面には妥当だが、軽量画面の入口には重すぎる

## Design Principle

重い API を最適化して全部を一度に変えるのではなく、用途別に API を分ける。

### Keep

- `list_worktrees()`
  - 一覧画面向けの詳細 API として残す

### Add

- `has_linked_worktrees()`
  - linked worktree の有無だけを確認する軽量 API
- `list_worktrees_basic()`
  - 軽量一覧 API

### Precise Semantics

- `has_linked_worktrees()`
  - `main` worktree は数えない
  - `git worktree list` 上の linked worktree が 1 件以上あるかだけを見る
  - `create` の first-worktree 分岐と完全に同じ意味で使う
- `list_worktrees_basic()`
  - 候補表示や選択解決に必要な最小情報だけ返す
  - `list_worktrees()` の軽量版であり、表示順と identity 解決規則は現行と同じにする

## Proposed Data Model

新規に lightweight な struct を追加する。

例:

```rust
pub struct BasicWorktreeInfo {
    pub name: String,
    pub git_name: String,
    pub path: PathBuf,
    pub branch: String,
    pub is_current: bool,
    pub is_locked: bool,
}
```

### Included

- `name`
- `git_name`
- `path`
- `branch`
- `is_current`
- `is_locked`

### Excluded

- `has_changes`
- `last_commit`
- `ahead_behind`

理由:
- `create / batch delete / rename / switch / search` の入口では不要

### Identity Contract

- `name`
  - UI 表示名として使う
  - renamed worktree の表示も現行と同じくこの値を使う
- `git_name`
  - destructive operation や内部解決に使う
  - `remove`, `rename`, orphaned branch 判定の対象解決はこの値を基準にする
- renamed worktree が存在しても、現行の `name` と `git_name` の役割分担を変えない
- `list_worktrees_basic()` と `list_worktrees()` は、同じ worktree に対して同じ `name / git_name` の組を返す

## Safe Refactoring Scope

### Phase 1

`create` のみ軽量化する。

- `manager.list_worktrees()?` をやめる
- `manager.has_linked_worktrees()?` を導入して置き換える
- この phase では `BasicWorktreeInfo` と `list_worktrees_basic()` はまだ導入しない

期待効果:
- 件数依存の初動遅延をかなり削減できる

### Phase 2

`batch delete` を軽量化する。

- `manager.list_worktrees()?` を `manager.list_worktrees_basic()?` に置き換える
- `is_branch_unique_to_worktree()` を軽量一覧ベースに切り替える
- 候補順と選択 index の対応は現行のまま維持する

期待効果:
- 入口の待ち時間を削減
- summary 生成の二次的な遅延も削減

### Phase 3

同じ構造の他画面を軽量化する。

候補:
- `rename`
- `switch`
- `search`

状態:
- 実装済み

### Explicitly Out of Scope

- `list` の軽量化
- 文言変更
- メニュー順変更
- hook の仕様変更
- path / branch の表示仕様変更

## Compatibility Rules

以下は絶対に変えない。

- `create`
  - worktree 0 件のときだけ location 選択を出す
  - 1 件以上なら location 選択を出さない
  - ここでいう件数は linked worktree 数だけを意味する
- `batch delete`
  - current worktree を候補に出さない
  - orphaned branch 判定の意味を変えない
  - 候補順を変えない
  - 選択 index と削除対象の対応を変えない
  - renamed worktree がある場合も、表示は `name`、削除対象解決は現行と同じ identity を使う
- `rename / switch / search`
  - current 判定を変えない
  - 選択対象の path / branch を変えない
- `list`
  - 表示内容は現状維持

## Test Plan

### Existing Behavior Lock

既存の以下を回帰として守る。

- `create` 系 test
- `batch delete` 系 test
- `switch / rename / search` 系 test
- `custom_path_behavior_test`
- `worktree lifecycle` 系 integration test

### New Tests

#### `create`

- `has_linked_worktrees() == false` なら first-worktree location prompt が出る
- `has_linked_worktrees() == true` なら first-worktree location prompt が出ない
- `main` worktree しかない repo では location prompt が出る
- linked worktree が 1 件以上ある repo では location prompt が出ない

#### `batch delete`

- `list_worktrees_basic()` ベースでも候補一覧が現状と一致する
- `is_branch_unique_to_worktree()` の結果が現状と一致する
- 候補順が現状と一致する
- 選択 index と実際の削除対象の対応が現状と一致する
- renamed worktree がある場合も `name / git_name` の解決が現状と一致する

#### `basic vs full`

- 同じ repo で
  - `name`
  - `git_name`
  - `path`
  - `branch`
  - `is_current`
  - `is_locked`
  が `list_worktrees_basic()` と `list_worktrees()` で一致する

## Validation Commands

最低限、各 phase ごとに以下を通す。

```bash
cargo fmt --check
cargo clippy --all-features -- -D warnings
cargo test --all-features -- --test-threads=1
```

必要なら targeted 実行:

```bash
cargo test --all-features unit::commands::create -- --test-threads=1
cargo test --all-features unit::commands::delete -- --test-threads=1
```

## Success Criteria

- `create` の初期 prompt までの待ち時間が件数増加に対して改善される
- `batch delete` の画面表示前と summary 生成前の待ち時間が改善される
- `rename / switch / search` の一覧表示前の待ち時間も改善される
- full test が green
- 観測可能な挙動差分がない

## Implementation Notes

- まず `infrastructure::git` に軽量 API を追加する
- 既存の `WorktreeInfo` は残す
- `BasicWorktreeInfo` から `WorktreeInfo` への安易な変換は作らない
  - 不要な status 取得を誘発しやすいため
- `batch delete` の branch uniqueness は、一覧取得結果の再利用を優先する
- `create` は `list_worktrees_basic()` を待たず、`has_linked_worktrees()` 単独で先に改善する

## Recommended First Patch

最初の 1 patch はここまでに限定する。

1. `GitWorktreeManager::has_linked_worktrees()`
2. `create` を `has_linked_worktrees()` に切り替え
3. first-worktree 分岐の回帰 test 追加

この単位なら、効果が大きく、差分も最小で済む。
