use anyhow::Result;
use console::Term;

use crate::app::menu::MenuItem;
use crate::support::terminal::clear_screen;
use crate::usecases;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuAction {
    Continue,
    Exit,
    ExitAfterSwitch,
}

pub fn handle_menu_item(item: &MenuItem, term: &Term) -> Result<MenuAction> {
    clear_screen(term);

    match item {
        MenuItem::ListWorktrees => usecases::list_worktrees::list_worktrees()?,
        MenuItem::CreateWorktree => {
            if usecases::create_worktree::create_worktree()? {
                return Ok(MenuAction::ExitAfterSwitch);
            }
        }
        MenuItem::DeleteWorktree => usecases::delete_worktree::delete_worktree()?,
        MenuItem::SwitchWorktree => {
            if usecases::switch_worktree::switch_worktree()? {
                return Ok(MenuAction::ExitAfterSwitch);
            }
        }
        MenuItem::SearchWorktrees => {
            if usecases::search_worktrees::search_worktrees()? {
                return Ok(MenuAction::ExitAfterSwitch);
            }
        }
        MenuItem::BatchDelete => usecases::delete_worktree::batch_delete_worktrees()?,
        MenuItem::CleanupOldWorktrees => usecases::cleanup_worktrees::cleanup_old_worktrees()?,
        MenuItem::RenameWorktree => usecases::rename_worktree::rename_worktree()?,
        MenuItem::EditHooks => usecases::edit_hooks::edit_hooks()?,
        MenuItem::Exit => return Ok(MenuAction::Exit),
    }

    Ok(MenuAction::Continue)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_menu_item_exit() -> Result<()> {
        let term = Term::stdout();
        let result = handle_menu_item(&MenuItem::Exit, &term)?;
        assert_eq!(result, MenuAction::Exit);
        Ok(())
    }
}
