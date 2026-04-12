# Git Workers 本体リファクタリング計画

## 目的

このドキュメントは、`git-workers` の本体コードを安全にリファクタリングするための実装計画書です。

重要:

- 現在の動作を 100% 数学的に保証することはできない
- その代わり、この計画では「現在の動作を壊す変更は merge させない」ためのガードレールを最大化する
- 挙動差分の疑いが少しでも出た場合は、先へ進まず test 追加または計画修正を優先する

今回の目的は次の 4 つです。

1. `main` を薄くし、`menu loop` と command dispatch を application 層へ移す
2. `commands` に混在している `UI / use case / pure logic / side effect` を分離する
3. 巨大化した `infrastructure/git.rs` を責務ごとに分割する
4. 既存の挙動と `public API` を段階的に維持したまま、将来の機能追加と refactor を容易にする

この計画は「構造変更」と「動作変更」を明確に分離して進める前提で作成する。

最初の実装では、`constants.rs` と `ui.rs` の全面移設は行わない。これらは参照箇所が広く、早い段階で動かすと diff が不要に大きくなるため、まずは `app / usecases / adapters` の責務境界を作ることを優先する。

## スコープ

対象:

- `src/main.rs`
- `src/lib.rs`
- `src/menu.rs`
- `src/commands/*.rs`
- `src/core/*.rs`
- `src/config.rs`
- `src/repository_info.rs`
- `src/utils.rs`
- `src/ui.rs`
- `src/git_interface.rs`
- `src/infrastructure/*.rs`

非対象:

- shell wrapper 自体の挙動変更
- 新機能追加
- CLI 引数仕様の拡張
- hook の仕様変更
- config format の互換性破壊

## 最重要原則: 挙動固定

このリファクタリングの最優先事項は、現在の挙動を固定したまま内部構造だけを改善することである。

そのため、初期フェーズでは次を必須ルールとする。

1. 構造変更 PR では、既存 test の期待値を変更しない
2. 構造変更 PR では、ユーザー向け文言を変更しない
3. 構造変更 PR では、hook 実行順・switch file 書き込み順・config 探索順を変更しない
4. 既存 test で守れていない挙動に触る前には、先に test を追加する
5. 差分の安全性を説明できない変更は commit しない

## 保証の定義

この計画でいう「現在の動作を担保する」とは、少なくとも次を満たす状態を指す。

- 既存の public API 呼び出し結果が変わらない
- 既存 test suite が green のまま維持される
- command ごとの主要フローで観測可能な side effect が変わらない
- shell integration, hook, config discovery, repository info 表示の契約が変わらない

観測可能な side effect には次を含む。

- return value
- error の有無
- worktree / branch / path の最終状態
- `GW_SWITCH_FILE` への書き込み
- hook の発火有無と順序
- config file の探索結果
- list / switch / create / delete / rename のユーザー向け表示

## 挙動固定のための非交渉ルール

以下は妥協しない。

- `red test` のまま次フェーズへ進まない
- `構造変更` と `動作変更` を同じ PR に混ぜない
- test で未固定の挙動に触る場合、先に回帰 test を足す
- 「たぶん同じ動作」の推測で merge しない
- 差分説明で `internal change only` と書く PR は、観測可能な差分が 0 であることを test で示す

## 現状診断

### 1. `main.rs` が厚い

現在の [main.rs](/Users/a12622/git/git-workers/src/main.rs) は次の責務を同時に持つ。

- `clap` による引数処理
- terminal 初期化
- 画面クリア
- header 表示
- menu item 構築
- menu 選択処理
- command dispatch
- `switch` 後の特別な exit 制御

このため、`CLI entry point` の変更が UI 全体や command 呼び出しと強く結びついている。

### 2. `commands` が use case と adapter を兼務している

現在の `commands` 層は、例えば [create.rs](/Users/a12622/git/git-workers/src/commands/create.rs) で次を同時に実施している。

- prompt 文言の表示
- `DialoguerUI` の呼び出し
- validation
- worktree path 判定
- Git 操作 orchestration
- hook 実行
- file copy
- switch file 書き込み
- progress 表示

同様の混在は [delete.rs](/Users/a12622/git/git-workers/src/commands/delete.rs), [rename.rs](/Users/a12622/git/git-workers/src/commands/rename.rs), [switch.rs](/Users/a12622/git/git-workers/src/commands/switch.rs), [list.rs](/Users/a12622/git/git-workers/src/commands/list.rs), [shared.rs](/Users/a12622/git/git-workers/src/commands/shared.rs) に広く存在する。

### 3. `shared.rs` が責務の受け皿になっている

[shared.rs](/Users/a12622/git/git-workers/src/commands/shared.rs) には次が同居している。

- search
- batch delete
- cleanup
- hook editor 起動
- config path 探索
- worktree icon 表示

これは `共通化` というより `分類待ちの処理の集積` に近く、変更時の見通しが悪い。

### 4. `infrastructure/git.rs` が巨大で、内部責務が曖昧

[git.rs](/Users/a12622/git/git-workers/src/infrastructure/git.rs) は巨大であり、少なくとも次の責務を含む。

- repository open / discovery
- worktree list / create / remove / rename
- branch 操作
- tag 取得
- path / parent 判定
- lock file 管理
- 表示補助に近い情報集約

これは `Git adapter` と `domain 判断` と `運用上の便宜` が 1 箇所に集まっている状態である。

### 5. `config` と `repository_info` に command 実行が散っている

[config.rs](/Users/a12622/git/git-workers/src/config.rs) と [repository_info.rs](/Users/a12622/git/git-workers/src/repository_info.rs) は、設定探索や repository context 判定のために `git` command を直接実行している。

その結果、

- path 探索ロジック
- repository type 判定
- 表示向け文字列整形

が密結合している。

### 6. `public API` は既に広い

[lib.rs](/Users/a12622/git/git-workers/src/lib.rs) と [commands/mod.rs](/Users/a12622/git/git-workers/src/commands/mod.rs) は広めの re-export を提供しており、test もこれに依存している。

特に次の API は互換維持の影響が大きい。

- `git_workers::commands::*`
- `git_workers::git::GitWorktreeManager`
- `git_workers::repository_info::get_repository_info`
- `git_workers::commands::find_config_file_path`

### 7. 現在の挙動を十分に固定できていない箇所がまだある

既存 test suite は強くなってきているが、リファクタリングの安全網としては次の契約をさらに明文化する余地がある。

- command ごとの標準フローの表示
- hook 実行順
- config 探索順
- repository info 表示の完全な分岐
- list 表示順と marker

したがって、実装前に「この refactor で絶対に壊したくない挙動」を test で固定する作業を継続する。

## リファクタリング方針

### 基本原則

1. 最初の数 PR は `構造変更のみ` とし、挙動変更を混ぜない
2. 旧 API はいったん `facade` と `re-export` で維持する
3. use case 単位で module を移し、毎回 test を通す
4. `UI` と `Git` の両方を直接触る関数は減らす
5. `pure function` は `domain` か `support` へ移し、I/O から切り離す

### 完了後に目指す構造

```text
src/
  main.rs
  lib.rs

  app/
    mod.rs
    run.rs
    menu.rs
    actions.rs
    presenter.rs

  domain/
    mod.rs
    worktree.rs
    branch.rs
    repo_context.rs
    paths.rs
    validation.rs

  usecases/
    mod.rs
    create_worktree.rs
    delete_worktree.rs
    rename_worktree.rs
    switch_worktree.rs
    list_worktrees.rs
    search_worktrees.rs
    cleanup_worktrees.rs
    edit_hooks.rs

  adapters/
    mod.rs
    git/
      mod.rs
      git_worktree_repository.rs
      repo_discovery.rs
      worktree_lock.rs
    ui/
      mod.rs
      dialoguer.rs
    config/
      mod.rs
      loader.rs
    shell/
      mod.rs
      switch_file.rs
      editor.rs
    filesystem/
      mod.rs
      file_copy.rs
      ops.rs

  support/
    mod.rs
    terminal.rs
    styles.rs
```

補足:

- `src/constants.rs` は中盤以降まで現位置維持でよい
- `src/ui.rs` も trait の互換維持のため、しばらく facade として残してよい
- `src/core/*.rs` も中盤までは現位置維持でよく、最初は rename より依存の薄化を優先する

## 設計原則

### `app`

役割:

- 対話フロー
- menu loop
- command dispatch
- 表示文字列の composition

禁止事項:

- Git repository への直接アクセス
- config file 探索
- branch / path の判断ロジック

### `usecases`

役割:

- 1 つのユーザー操作に対応する orchestration
- `input -> decision -> side effect -> result` の流れをまとめる

禁止事項:

- `println!` ベタ書き
- `dialoguer` の直接使用
- `git2` の直接使用

移行期ルール:

- PR 3 から PR 5 の間は、既存コードを安全に移すため `UserInterface` trait と既存 terminal helper への依存を一時許容する
- ただし新規に `dialoguer` や `git2` を `usecases` へ直接持ち込まない
- 最終的な純化は `presenter` と adapter 境界が揃った後に行う

### `domain`

役割:

- 純粋な validation
- path 解決
- rename / delete / switch 可否の判定
- repository context の model 化

禁止事項:

- file I/O
- process 実行
- terminal 出力

補足:

- hook 実行は `domain` に置かない
- hook の event 名や payload の model 化だけが必要なら `domain` に置けるが、実行本体は `adapters` か `usecases` に置く

### `adapters`

役割:

- `git2`, `std::process::Command`, `dialoguer`, filesystem などの外部依存

禁止事項:

- UI フローの全体制御
- 複数ユースケースにまたがる分岐の保持

### `support`

役割:

- constants
- terminal helper
- style helper
- 共通の小さな util

## 旧構造から新構造への対応表

| 現在 | 移行先 | 備考 |
| --- | --- | --- |
| `src/main.rs` | `src/app/run.rs` + 薄い `src/main.rs` | `main` は entry point に限定 |
| `src/menu.rs` | `src/app/menu.rs` | menu 定義のみ保持 |
| `src/commands/create.rs` | `src/usecases/create_worktree.rs` + `src/core/validation.rs` + `src/domain/paths.rs` | 初期フェーズでは `core` を残す |
| `src/commands/delete.rs` | `src/usecases/delete_worktree.rs` | batch delete と共通化候補あり |
| `src/commands/rename.rs` | `src/usecases/rename_worktree.rs` | rename 判定は `domain` へ |
| `src/commands/switch.rs` | `src/usecases/switch_worktree.rs` | shell integration は adapter へ |
| `src/commands/list.rs` | `src/usecases/list_worktrees.rs` + `src/app/presenter.rs` | table 表示を presenter 化 |
| `src/commands/shared.rs` | 複数ファイルへ解体 | そのまま残さない |
| `src/config.rs` | `src/adapters/config/loader.rs` + `Config` 定義残留 or 後続で model 分離 | loader 分離が先 |
| `src/repository_info.rs` | `src/domain/repo_context.rs` + `src/app/presenter.rs` | 判定と表示を分離 |
| `src/utils.rs` | `src/support/terminal.rs` + `src/adapters/shell/switch_file.rs` | 小さく分けるが、初手では一括移動しない |
| `src/ui.rs` | `src/adapters/ui/dialoguer.rs` + `src/ui.rs` 互換 facade | trait は当面 `src/ui.rs` から見せ続ける |
| `src/git_interface.rs` | `src/adapters/git/` に寄せる or 互換 facade | 現状は活用度が低いので後回し可 |
| `src/infrastructure/git.rs` | `src/adapters/git/*.rs` | 最初は facade 維持 |

## 実装フェーズ

### Phase 0: 前提固定

目的:

- 以後の refactor に対する安全網を固定する

実施:

- 現在の test suite を baseline とする
- `cargo test --all-features -- --test-threads=1` を基準コマンドとして固定
- 既存 public API 依存を一覧化して、削除禁止対象を明示
- command 契約を追加で固定する必要がある箇所を洗い出す
- `構造変更前の観測可能な挙動` を先に test 化する

完了条件:

- baseline test が green
- 本ドキュメントに互換維持対象を明記済み
- refactor 対象 command の挙動について、少なくとも主要フローの契約 test がある

### Phase 1: 受け皿 module 作成

目的:

- 新しい module 構造を追加し、実体の移動前に依存先を用意する

実施:

- `src/app/mod.rs`
- `src/domain/mod.rs`
- `src/usecases/mod.rs`
- `src/adapters/mod.rs`
- `src/support/mod.rs`

この段階では、新規 module は空に近い状態でよい。`pub mod` と最小限の `pub use` のみを配置する。

注意:

- `src/constants.rs` はまだ動かさない
- `src/ui.rs` はまだ動かさない
- `src/core/*.rs` はまだ rename しない
- `src/infrastructure/mod.rs` の公開面は維持する

完了条件:

- build が通る
- 既存 import path を壊さない
- 追加した受け皿 module が runtime behavior に影響していないことを test で確認できる

### Phase 2: `main` の簡素化

目的:

- `main.rs` から loop と dispatch を外す

実施:

- `src/app/run.rs` を作成
- `MenuAction` を `app` 配下へ移す
- menu loop を `app::run()` へ移す
- [main.rs](/Users/a12622/git/git-workers/src/main.rs) は次だけを持つ
  - CLI parse
  - `--version`
  - `app::run()`

この phase では command の実体や import path は変えない。移すのは loop と dispatch だけに限定する。

完了条件:

- 挙動不変
- `main.rs` が薄くなる
- `main.rs` の test 不要部分が減る
- 起動後の menu 表示順、exit 挙動、`switch` 後の終了動作が一致する

### Phase 3: menu と dispatch の整理

目的:

- menu 表示定義と action 実行を分離する

実施:

- [menu.rs](/Users/a12622/git/git-workers/src/menu.rs) を `app/menu.rs` へ移す
- `MenuItem` に対応する action dispatch を `app/actions.rs` へ分離
- menu 表示文字列と実行関数の対応を 1 箇所に集約

完了条件:

- menu item の追加時に修正箇所が明確
- `main` と command module の直接結合が減る

### Phase 4: `shared.rs` の解体

目的:

- いちばん曖昧な file をなくす

分解先:

- `search_worktrees` -> `usecases/search_worktrees.rs`
- `batch_delete_worktrees` -> `usecases/delete_worktree.rs` or `usecases/batch_delete_worktrees.rs`
- `cleanup_old_worktrees` -> `usecases/cleanup_worktrees.rs`
- `edit_hooks` -> `usecases/edit_hooks.rs`
- `find_config_file_path` -> 当面 `commands/shared.rs` から facade しつつ、移設先は `adapters/config/loader.rs`
- `get_worktree_icon` -> presenter 導入後に `app/presenter.rs`

順序ルール:

- `shared.rs` は一気に消さず、先に移設先 module を作ってから機能ごとに縮退させる
- `presenter` 未導入の段階では `get_worktree_icon` だけを無理に動かさない
- `config loader` 未導入の段階では `find_config_file_path` を facade 経由で保持する

完了条件:

- `shared.rs` が空になるか削除可能になる
- 新しい module 名だけで責務が推測できる
- 移設した責務について、移設前後で観測可能な挙動差分がない

### Phase 5: command ごとの use case 化

目的:

- `commands` を `usecases` へ段階移行する

実施順:

1. `switch`
2. `list`
3. `delete`
4. `rename`
5. `create`

理由:

- `switch` は比較的小さく、shell integration 切り出しの起点になる
- `list` は表示責務の分離に向く
- `delete` と `rename` は中規模で、共通の pattern を確立しやすい
- `create` は副作用が最も多く、最後に回す方が安全

完了条件:

- 各旧 file は `pub use` の facade に縮退できる
- `usecases` が UI 非依存の構造へ寄る

### Phase 6: `GitWorktreeManager` の分割

目的:

- 巨大な Git adapter を責務ごとに小さくする

第一段階:

- `worktree_lock.rs`
- `repo_discovery.rs`
- `git_worktree_repository.rs`

第二段階:

- branch 操作
- tag 取得
- status / metadata 集約

方針:

- `GitWorktreeManager` 自体の型名はしばらく維持する
- 実装を内側 module へ委譲し、利用側の breakage を防ぐ

完了条件:

- `infrastructure/git.rs` が facade レベルまで縮小
- 各責務ごとに unit test 可能

### Phase 7: `config / repository_info / shell integration` 分離

目的:

- repository context と表示を切り離す
- shell integration を adapter として独立させる

実施:

- `find_config_file_path*` を `adapters/config/loader.rs` へ
- `get_repository_info*` の判定ロジックを `domain/repo_context.rs` へ
- 表示用フォーマットを `app/presenter.rs` へ
- `write_switch_path` を `adapters/shell/switch_file.rs` へ
- editor 起動を `adapters/shell/editor.rs` へ

完了条件:

- context 判定ロジックを pure に近づけられる
- shell 依存部が局所化される

### Phase 8: 公開 API の整理

目的:

- 互換性を維持しつつ、今後の公開境界を明確化する

実施:

- [lib.rs](/Users/a12622/git/git-workers/src/lib.rs) の re-export を棚卸し
- 内部専用 API を `pub(crate)` 化
- 互換維持が必要なものは deprecation comment を付けて facade 経由にする

完了条件:

- `lib.rs` から見て「何が public API か」が読みやすい
- `commands` と `infrastructure` の将来的な縮退余地ができる

## フェーズごとの PR 推奨分割

### PR 1

対象:

- `app / domain / usecases / adapters / support` の受け皿追加
- `main.rs` の簡素化
- `menu` と dispatch の切り出し

具体的に触る file の目安:

- `src/main.rs`
- `src/lib.rs`
- `src/menu.rs`
- `src/app/mod.rs`
- `src/app/run.rs`
- `src/app/menu.rs`
- `src/app/actions.rs`

種類:

- 構造変更のみ

### PR 2

対象:

- `shared.rs` の解体
- `search / batch delete / cleanup / edit hooks / config path` の再配置

種類:

- 構造変更のみ

### PR 3

対象:

- `switch` と `list` の use case 化
- `switch_file` adapter 導入
- presenter 導入

種類:

- 構造変更のみ

### PR 4

対象:

- `delete` と `rename` の use case 化

種類:

- 構造変更中心

### PR 5

対象:

- `create` の use case 化
- `file_copy / hooks / branch source` まわりの境界整理

種類:

- 構造変更中心

### PR 6

対象:

- `GitWorktreeManager` の内部分割
- `config / repository_info` の分離

種類:

- 構造変更中心

## 互換維持ルール

以下は初期フェーズでは壊さない。

- `git_workers::commands` module path
- `git_workers::commands::*`
- `git_workers::git` module path
- `git_workers::git::GitWorktreeManager`
- `git_workers::git::WorktreeInfo`
- `git_workers::infrastructure` module path
- `git_workers::infrastructure::git::GitWorktreeManager`
- `git_workers::infrastructure::WorktreeInfo`
- `git_workers::hooks` module path
- `git_workers::file_copy` module path
- `git_workers::filesystem` module path
- `git_workers::repository_info::get_repository_info`
- `git_workers::repository_info::get_repository_info_at_path`
- `git_workers::commands::find_config_file_path`
- `git_workers::ui` module path
- `git_workers::config` module path
- `git_workers::menu` module path
- `git_workers::core` module path
- 既存 test が import している型と関数名

方針:

- module path と名前を残す
- 実体だけ新しい module に移す
- import path の変更は最後にまとめて行う

## 実装ルール

### 1. 先に facade を作る

例:

- `commands/switch.rs` はいきなり削除しない
- 中身を `usecases/switch_worktree.rs` に移し、`commands/switch.rs` は再公開のみへ縮退させる

### 2. pure logic は先に切り出す

候補:

- validation
- rename / delete 可否判定
- path resolve
- display sort

補足:

- 既存の [core/validation.rs](/Users/a12622/git/git-workers/src/core/validation.rs) は、当面 `domain` の最終形に無理に移さず、まず `commands` からの依存を薄くすることを優先する
- `core -> domain` の rename は中盤以降でもよい
- `core` を無理に rename しないことで、初期 PR の diff と import 変更を抑える

### 3. side effect を adapter に寄せる

候補:

- `dialoguer`
- `git2`
- `Command::new`
- `write_switch_path`
- editor 起動
- file copy

補足:

- hook 実行も side effect として扱い、最終的には `adapters` または adapter を呼ぶ `usecases` 側へ寄せる
- hook 実行本体を `domain` に置かない

### 4. 文言整理は構造変更と分ける

理由:

- test failure の原因切り分けを容易にするため

## test 戦略

各 PR で最低限行うこと:

- `cargo test --all-features -- --test-threads=1`

必要に応じて追加:

- command 単位の targeted test
- `cargo fmt --check`
- `cargo clippy --all-features -- -D warnings`

重点監視対象:

- `tests/custom_path_behavior_test.rs`
- `tests/unit/commands/*.rs`
- `tests/integration/worktree_lifecycle.rs`
- `tests/integration/repository_info_display_test.rs`
- `tests/unit/infrastructure/git.rs`

追加で強化対象:

- `main` loop と menu dispatch の契約 test
- `search / switch / list` の表示契約 test
- `find_config_file_path` の探索順契約 test
- `repository_info` の分岐ごとの表示契約 test
- `create` の hook / file copy / switch file の順序契約 test

### test gate

各 PR は、少なくとも次の gate をすべて満たしたときのみ次へ進める。

1. `cargo fmt --check`
2. `cargo test --all-features -- --test-threads=1`
3. その PR で触った責務に対応する targeted test の成功

必要なら追加で行うこと:

- `cargo clippy --all-features -- -D warnings`
- 実際の CLI 手動確認

### 変更前後比較の原則

構造変更 PR では、変更前後で次を比較する。

- 公開関数の return value
- file system side effect
- config discovery result
- hook side effect
- 表示文字列

比較を自動化できるものは test で自動化し、自動化しにくいものは PR の説明に比較観点を明記する。

## 想定リスク

### 1. re-export の循環依存

リスク:

- `commands -> usecases -> adapters -> lib re-export` の循環が起きる

対策:

- `crate::` の参照方向を最初に統一する
- `lib.rs` は末端で re-export するだけに寄せる

### 2. `GitWorktreeManager` 分割時の可視性崩れ

リスク:

- internal helper を移した瞬間に `pub(crate)` 不足で compile error が大量発生する

対策:

- 先に facade method を残す
- 内部 module に切り出すのは method 単位で行う

### 3. `repository_info` の挙動差分

リスク:

- bare / worktree / regular repo の表示が subtle に変わる

対策:

- 既存 integration test を保持
- 文字列整形と context 判定を分ける

### 4. `create` の副作用順序崩れ

リスク:

- hook 実行順
- file copy の source 判定
- switch file 書き込みのタイミング

対策:

- `create` は最後に移す
- 現行の回帰 test を増やした上で着手する

### 5. 最終形の理想を先に入れすぎて移行 PR が膨らむ

リスク:

- `core -> domain`
- `ui -> adapters/ui`
- `utils -> support`

を同時に進めて、構造変更だけのはずが大規模 rename になる

対策:

- 初期 PR は rename より facade と委譲を優先する
- `core`, `ui`, `constants` は据え置き前提で進める
- 「最終配置」と「今回動かす範囲」を毎 PR で分けて記述する

### 6. 「構造変更だけ」のつもりで観測可能な差分を入れてしまう

リスク:

- 表示順
- error message
- hook 実行順
- config 探索順

のような契約を unintentionally 変える

対策:

- 触る前に契約 test を足す
- 観測可能な差分がありうる箇所は PR 説明で列挙する
- 少しでも差分が疑わしい場合は、その PR を `構造変更` ではなく `動作変更` に格上げする

## anti-goals

この計画では、次をやらない。

- 1 PR で 2 つ以上の大きな責務移行を同時に完了させようとしない
- 構造変更 PR で文言整理や error message 変更をしない
- facade を外す前に import path の一括変更をしない
- `GitWorktreeManager` の型名を early phase で変えない
- shell integration の仕様を変えない

## 各 PR の Definition of Done

### PR 1 DoD

- `main.rs` が entry point だけになっている
- menu loop と dispatch が `app` 配下へ移っている
- 既存の menu 表示順と `switch` 後の終了挙動が一致している
- `cargo test --all-features -- --test-threads=1` が green
- `main` の変更前後で観測可能な動作差分がないことを説明できる

### PR 2 DoD

- `shared.rs` から少なくとも 2 つ以上の明確な責務が移設されている
- `shared.rs` に新しい関数を追加していない
- `find_config_file_path` と `search / cleanup / hooks` の移設先が文書化されている
- `cargo test --all-features -- --test-threads=1` が green

### PR 3-5 DoD

- 対象 command file が facade 化または薄い wrapper 化されている
- `usecases` 側に移った orchestration が test で保護されている
- 直接の `dialoguer` / `git2` 依存を増やしていない
- `cargo test --all-features -- --test-threads=1` が green

### PR 6 DoD

- `GitWorktreeManager` は型名維持のまま内部分割されている
- `config / repository_info / shell` の adapter 境界が読み取れる
- 既存 module path の互換が維持されている
- `cargo test --all-features -- --test-threads=1` が green

## レビュー観点の強化版

各 PR のレビューでは、通常の動作確認に加えて次を見る。

1. 依存方向が一方向になっているか
2. facade が暫定措置として薄いか
3. 構造変更 PR に挙動変更が混ざっていないか
4. import path 互換と module path 互換の両方が守られているか
5. 次の PR でさらに削れる場所が増えているか

## rollback 方針

もし途中の PR で設計が合わないと判明した場合は、次の順で戻す。

1. facade を残したまま新しい module の利用だけ止める
2. `app` や `usecases` に移した実体を旧 module に戻すのではなく、旧 module から再委譲する
3. import path を戻さず、実体配置だけ調整する

この方針により、revert しても外部 API の揺れを最小化できる。

## 100 点に近づけるための運用ルール

この計画を 70 点から 100 点に近づけるため、実装時は次を守る。

1. 各 PR の冒頭に「今回動かす責務 / 動かさない責務」を明記する
2. 各 PR で facade を 1 段階以上薄くする
3. 新しい module を作るたびに、その module が直接依存してよいものを明記する
4. `final shape` ではなく `next safe step` を優先して commit を切る
5. 迷ったときは rename より wrapper 化、wrapper 化より委譲の明確化を優先する
6. 挙動固定の根拠を test 名で言えない変更は、まだ安全ではないとみなす
7. `保証` という言葉は、test と比較観点で裏づけできるときだけ使う

## 実装開始前チェックリスト

- [ ] baseline test が green
- [ ] 互換維持対象 API を再確認した
- [ ] 今回の PR が `構造変更` か `動作変更` か明記した
- [ ] 新しい module 名が責務を表している
- [ ] `shared` のような曖昧名を増やしていない
- [ ] `main.rs` に新しい責務を戻していない
- [ ] 新しい code path に最低 1 つ以上の test がある

## PR 1 の実装タスク詳細

### タスク 1: `app` module 追加

- `src/app/mod.rs` を追加する
- `run`, `menu`, `actions` を公開する
- ここでは business logic を持たせない

### タスク 2: `app::menu` 新設

- `src/app/menu.rs` を追加する
- 既存の `MenuItem` をここへ移す
- [src/menu.rs](/Users/a12622/git/git-workers/src/menu.rs) は一時的に `pub use crate::app::menu::*;` の facade にする

### タスク 3: `app::actions` 新設

- `handle_menu_item` と `MenuAction` を `src/app/actions.rs` へ移す
- 既存 command 呼び出しの順序と分岐は変えない
- `ExitAfterSwitch` の扱いも現状維持する

### タスク 4: `app::run` 新設

- menu loop 全体を `src/app/run.rs` へ移す
- header 表示と screen clear もここで扱う
- `DialoguerUI` の生成位置はこの phase では維持でよい

### タスク 5: `main.rs` を薄くする

- `Cli` の parse
- `--version` 処理
- `app::run()` 呼び出し

ここまでに縮小する。

### タスク 6: 互換 export を整える

- [src/lib.rs](/Users/a12622/git/git-workers/src/lib.rs) で `pub mod app;` を追加する
- ただし既存の `commands`, `infrastructure`, `repository_info` の export は変更しない

### タスク 7: 検証

- `cargo fmt`
- `cargo test --all-features -- --test-threads=1`

## PR 1 のレビュー用チェックポイント

1. [src/main.rs](/Users/a12622/git/git-workers/src/main.rs) に loop や dispatch が残っていないか
2. `MenuItem` の表示順と表示文字列が変わっていないか
3. `switch` 実行後の終了挙動が変わっていないか
4. 既存 test の import path が壊れていないか
5. `app` が command 実装詳細に依存しすぎていないか

## 最初の実装対象

最初に着手する作業は次の組み合わせが最も安全である。

1. `src/app/mod.rs` と `src/app/run.rs` を追加
2. `main.rs` の menu loop を `app::run()` へ移動
3. `src/app/menu.rs` を作り、既存 `menu.rs` を facade 化
4. `src/app/actions.rs` に command dispatch を移動

この 4 点なら、比較的少ない file 変更で構造改善の土台を作れる。

## レビュー観点

レビューでは、次を優先して見る。

1. 挙動差分がないか
2. 依存方向が改善しているか
3. 新しい module 名が今後も耐えられるか
4. facade が一時対応として妥当か
5. 次の PR でさらに分解しやすくなっているか

## 保留事項

この計画では、以下は意図的に後回しにする。

- 非対話 CLI mode の導入
- trait ベースの use case 入力モデル再設計
- 全 message / constants の再編成
- `git_interface.rs` の全面見直し
- benchmark / debug test の配置整理

これらは本体の責務整理が終わってからの方が安全である。
