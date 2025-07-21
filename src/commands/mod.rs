// 既存 API の完全な互換性維持
mod create;
mod delete;
mod list;
mod rename;
pub mod shared;
mod switch;

// 公開インターフェース（変更なし）
pub use create::{
    create_worktree, create_worktree_with_ui, determine_worktree_path, validate_worktree_creation,
    BranchSource, WorktreeCreateConfig,
};
// Re-export validation functions from core module
pub use super::core::{validate_custom_path, validate_worktree_name};
pub use delete::{
    analyze_deletion, delete_worktree, delete_worktree_with_ui, execute_deletion,
    get_deletable_worktrees, DeletionAnalysis, WorktreeDeleteConfig,
};
pub use list::{list_worktrees, list_worktrees_with_ui};
pub use rename::{
    analyze_rename_requirements, execute_rename, get_renameable_worktrees, rename_worktree,
    rename_worktree_with_ui, validate_rename_operation, RenameAnalysis, WorktreeRenameConfig,
};
pub use shared::{
    batch_delete_worktrees, cleanup_old_worktrees, create_search_items, edit_hooks,
    find_config_file_path, get_worktree_icon, prepare_batch_delete_items, search_worktrees,
    validate_search_selection, BatchDeleteConfig, SearchAnalysis, SearchConfig,
};
pub use switch::{
    analyze_switch_target, execute_switch, sort_worktrees_for_display, switch_worktree,
    switch_worktree_with_ui, SwitchAnalysis, WorktreeSwitchConfig,
};
