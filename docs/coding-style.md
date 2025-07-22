# Git Workers - コーディングスタイルガイド

このドキュメントでは、Git Workers プロジェクトにおけるコードの統一感とスタイル規則を定義します。

## 基本原則

### 1. 一貫性の優先

- 既存のコードパターンに合わせる
- 新しいパターンを導入する際は全体に適用する
- 部分的な変更より全体的な統一感を重視

### 2. 可読性の重視

- 意図が明確なコード
- 適切な命名規則
- 冗長性の排除

## フォーマット規則

### String フォーマット

**インライン変数構文の使用（必須）**

```rust
// ✅ 推奨
format!("Created worktree '{name}' at {path}")
println!("Found {count} items")
eprintln!("Error: {error}")

// ❌ 非推奨（古い形式）
format!("Created worktree '{}' at {}", name, path)
println!("Found {} items", count)
eprintln!("Error: {}", error)
```

**適用範囲**

- `format!`
- `println!`, `eprintln!`
- `log::info!`, `log::warn!`, `log::error!`
- `panic!`
- その他すべてのフォーマット系マクロ

**例外**
format! マクロで文字列リテラルが必要な場合（anyhow! エラーなど）は従来形式を使用：

```rust
// anyhow! では文字列リテラルが必要
anyhow!("Invalid path: {}", path)  // OK
```

## 定数管理

### 定数の集約化

**constants.rs での集中管理**

```rust
// ✅ 推奨 - constants.rs に定義
pub const ERROR_USER_CANCELLED: &str = "User cancelled operation";
pub const MSG_WORKTREE_CREATED: &str = "Worktree created successfully";

// 使用箇所
return Err(anyhow!(ERROR_USER_CANCELLED));
```

**ハードコーディングの禁止**

```rust
// ❌ 避ける - 直接文字列
return Err(anyhow!("User cancelled operation"));

// ✅ 推奨 - 定数使用
return Err(anyhow!(ERROR_USER_CANCELLED));
```

### 定数の命名規則

**プレフィックス規則**

- `ERROR_*`: エラーメッセージ
- `MSG_*`: 一般的なメッセージ
- `ICON_*`: アイコン文字
- `FORMAT_*`: フォーマット文字列
- `DEFAULT_*`: デフォルト値

**例**

```rust
pub const ERROR_WORKTREE_NOT_FOUND: &str = "Worktree not found";
pub const MSG_WORKTREE_DELETED: &str = "Worktree deleted successfully";
pub const ICON_CURRENT_BRANCH: &str = " ";
pub const DEFAULT_BRANCH_NAME: &str = "main";
```

### テスト定数

**テスト専用定数の管理**

```rust
// ✅ 推奨 - cfg(test) アノテーション使用
#[cfg(test)]
const TEST_WORKTREE_NAME: &str = "test-worktree";
#[cfg(test)]
const TEST_BRANCH_NAME: &str = "test-branch";

// テスト内で使用
#[test]
fn test_create_worktree() {
    let name = TEST_WORKTREE_NAME;
    // ...
}
```

## エラーハンドリング

### 統一的なエラーメッセージ

**メッセージ構造**

```rust
// 基本パターン: "動作が失敗した理由"
ERROR_WORKTREE_CREATE_FAILED: "Failed to create worktree"
ERROR_BRANCH_NOT_FOUND: "Branch not found"
ERROR_PERMISSION_DENIED: "Permission denied"

// 詳細パターン: "動作が失敗した理由: {詳細}"
ERROR_CONFIG_READ_FAILED: "Failed to read configuration: {}"
ERROR_WORKTREE_PATH_INVALID: "Invalid worktree path: {}"
```

**anyhow! エラーの統一**

```rust
// ✅ 推奨パターン
return Err(anyhow!("Failed to create worktree: {}", error));

// 避けるパターン
return Err(anyhow!("Worktree creation error: {}", error));
return Err(anyhow!("Error creating worktree: {}", error));
```

## コード構造

### ファイル構成

**モジュール構成の原則**

```
src/
├── commands/          # コマンド実装（機能別）
├── core/             # コアロジック
├── infrastructure/   # 外部依存（Git, ファイルシステム）
├── constants.rs      # 全定数の集約
├── ui.rs            # UI 抽象化
└── utils.rs         # ユーティリティ
```

**import の順序**

```rust
// 1. 標準ライブラリ
use std::path::PathBuf;
use std::collections::HashMap;

// 2. 外部クレート（アルファベット順）
use anyhow::Result;
use colored::Colorize;

// 3. 内部モジュール（階層順）
use crate::constants::*;
use crate::core::validation;
use crate::ui::UserInterface;
```

### 関数設計

**命名規則**

```rust
// ✅ 動詞 + 目的語パターン
fn create_worktree() -> Result<()>
fn delete_branch() -> Result<()>
fn validate_path() -> Result<()>

// ✅ is/has などの述語パターン
fn is_current_worktree() -> bool
fn has_uncommitted_changes() -> bool
```

**引数の順序**

```rust
// 1. 主要オブジェクト
// 2. 設定・オプション
// 3. UI インターフェース
fn create_worktree(
    manager: &WorktreeManager,
    name: &str,
    options: &CreateOptions,
    ui: &dyn UserInterface
) -> Result<bool>
```

## テストコード

### テスト構成

**テストファイル命名**

- 単体テスト: `tests/unit/module_name.rs`
- 統合テスト: `tests/integration/feature_name.rs`
- E2E テスト: `tests/e2e/workflow_name.rs`

**テスト関数命名**

```rust
// パターン: test_[対象]_[条件]_[期待結果]
#[test]
fn test_create_worktree_with_valid_name_succeeds() -> Result<()>

#[test]
fn test_delete_worktree_when_not_exists_fails() -> Result<()>
```

### Mock の使用

**UI Mock パターン**

```rust
let ui = TestUI::new()
    .with_input(TEST_WORKTREE_NAME)
    .with_selection(0)
    .with_confirmation(true);
```

## リファクタリング指針

### 段階的改善

1. **動作変更なし**のリファクタリングを先行
2. **機能追加**は別コミット
3. **テスト追加**で安全性確保

### 品質チェック

**必須チェック項目**

```bash
# フォーマット確認
cargo fmt --check

# Clippy 警告ゼロ
cargo clippy --all-features -- -D warnings

# テスト通過
cargo test --all-features

# 型チェック
cargo check --all-features
```

## コメント規則

### コメントの言語統一

**英語コメントの徹底使用（必須）**

```rust
// ✅ 推奨 - 英語コメント
// Check if the worktree exists before deletion
fn delete_worktree(name: &str) -> Result<()> {
    // Validate worktree name format
    if name.is_empty() {
        return Err(anyhow!("Worktree name cannot be empty"));
    }

    // Execute deletion command
    // ...
}

// ❌ 非推奨 - 日本語コメント
// ワークツリーが存在するかチェック
fn delete_worktree(name: &str) -> Result<()> {
    // ワークツリー名のフォーマットを検証
    // ...
}
```

**コメント品質基準**

```rust
// ✅ Good - Clear and concise
// Calculate relative path from project root
let relative_path = calculate_relative_path(&base_path, &target_path)?;

// ✅ Good - Explains complex logic
// Use fuzzy matching for branch selection to improve UX
// when dealing with large numbers of branches
let selection = fuzzy_select_branch(&branches)?;

// ❌ Avoid - Stating the obvious
// Set variable to true
let is_valid = true;

// ❌ Avoid - Outdated or incorrect comments
// TODO: This will be removed in v2.0 (but never removed)
```

**ドキュメントコメント（///）の使用**

````rust
/// Creates a new worktree with the specified configuration
///
/// # Arguments
/// * `manager` - The worktree manager instance
/// * `name` - Name for the new worktree
/// * `config` - Creation configuration options
///
/// # Returns
/// * `Ok(true)` - Worktree created and switched to
/// * `Ok(false)` - Worktree created but not switched
/// * `Err(_)` - Creation failed
///
/// # Examples
/// ```
/// let result = create_worktree(&manager, "feature-branch", &config)?;
/// ```
pub fn create_worktree(
    manager: &WorktreeManager,
    name: &str,
    config: &CreateConfig
) -> Result<bool> {
    // Implementation...
}
````

**TODO/FIXME/NOTE コメント**

```rust
// TODO: Add support for custom hooks in v0.7.0
// FIXME: Handle edge case when git directory is corrupted
// NOTE: This behavior matches Git's native worktree command
// HACK: Temporary workaround for upstream bug #1234
```

### 既存コメントの英語化

**段階的移行方針**

1. 新規コード：英語コメント必須
2. 既存コード：修正時に英語化
3. 重要なコメント：優先的に英語化

**英語化対象の優先順位**

1. 公開 API（`pub`）のドキュメントコメント
2. 複雑なロジックの説明コメント
3. TODO/FIXME コメント
4. テストコメント
5. 一般的なインラインコメント

## 今後の拡張

### 新機能追加時の指針

1. **constants.rs** への文字列集約
2. **インライン変数構文** の徹底使用
3. **統一的なエラーメッセージ** フォーマット
4. **適切なテスト** カバレッジ
5. **英語コメント** の使用

### 既存コードの改善

定期的に以下を実行：

- format! マクロの統一性チェック
- ハードコード文字列の constants.rs 移行
- エラーメッセージの統一性確認
- テスト定数の整理
- **日本語コメントの英語化**

---

このガイドラインに従うことで、Git Workers プロジェクトの品質と保守性を継続的に向上させることができます。
