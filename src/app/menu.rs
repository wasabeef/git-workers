use crate::constants::*;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MenuItem {
    ListWorktrees,
    SearchWorktrees,
    CreateWorktree,
    DeleteWorktree,
    BatchDelete,
    CleanupOldWorktrees,
    SwitchWorktree,
    RenameWorktree,
    EditHooks,
    Exit,
}

impl fmt::Display for MenuItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MenuItem::ListWorktrees => write!(f, "{MENU_LIST_WORKTREES}"),
            MenuItem::SearchWorktrees => write!(f, "{MENU_SEARCH_WORKTREES}"),
            MenuItem::CreateWorktree => write!(f, "{MENU_CREATE_WORKTREE}"),
            MenuItem::DeleteWorktree => write!(f, "{MENU_DELETE_WORKTREE}"),
            MenuItem::BatchDelete => write!(f, "{MENU_BATCH_DELETE}"),
            MenuItem::CleanupOldWorktrees => write!(f, "{MENU_CLEANUP_OLD}"),
            MenuItem::SwitchWorktree => write!(f, "{MENU_SWITCH_WORKTREE}"),
            MenuItem::RenameWorktree => write!(f, "{MENU_RENAME_WORKTREE}"),
            MenuItem::EditHooks => write!(f, "{MENU_EDIT_HOOKS}"),
            MenuItem::Exit => write!(f, "{MENU_EXIT}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_item_display_variants() {
        let items = [
            (MenuItem::ListWorktrees, MENU_LIST_WORKTREES),
            (MenuItem::SearchWorktrees, MENU_SEARCH_WORKTREES),
            (MenuItem::CreateWorktree, MENU_CREATE_WORKTREE),
            (MenuItem::DeleteWorktree, MENU_DELETE_WORKTREE),
            (MenuItem::BatchDelete, MENU_BATCH_DELETE),
            (MenuItem::CleanupOldWorktrees, MENU_CLEANUP_OLD),
            (MenuItem::SwitchWorktree, MENU_SWITCH_WORKTREE),
            (MenuItem::RenameWorktree, MENU_RENAME_WORKTREE),
            (MenuItem::EditHooks, MENU_EDIT_HOOKS),
            (MenuItem::Exit, MENU_EXIT),
        ];

        for (item, expected) in items {
            let formatted = format!("{item}");
            assert!(!formatted.is_empty());
            assert!(formatted.contains(expected));
        }
    }
}
