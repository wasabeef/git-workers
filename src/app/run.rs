use anyhow::Result;
use colored::*;
use console::Term;
use std::io::{self, Write};

use crate::app::actions::{handle_menu_item, MenuAction};
use crate::app::menu::MenuItem;
use crate::app::presenter::build_header_lines;
use crate::constants;
use crate::support::terminal::{clear_screen, setup_terminal_config};
use crate::ui::{DialoguerUI, UserInterface};

pub fn run() -> Result<()> {
    let term = Term::stdout();
    setup_terminal_config();

    loop {
        clear_screen(&term);
        let _ = io::stdout().flush();

        for line in build_header_lines() {
            println!("{line}");
        }

        let menu_items = [
            MenuItem::ListWorktrees,
            MenuItem::SwitchWorktree,
            MenuItem::SearchWorktrees,
            MenuItem::CreateWorktree,
            MenuItem::DeleteWorktree,
            MenuItem::BatchDelete,
            MenuItem::CleanupOldWorktrees,
            MenuItem::RenameWorktree,
            MenuItem::EditHooks,
            MenuItem::Exit,
        ];

        let display_items: Vec<String> = menu_items.iter().map(ToString::to_string).collect();
        let ui = DialoguerUI;
        let selection = match ui.select_with_default(
            constants::PROMPT_ACTION,
            &display_items,
            constants::DEFAULT_MENU_SELECTION,
        ) {
            Ok(selection) => selection,
            Err(_) => {
                clear_screen(&term);
                println!("{}", constants::INFO_EXITING.bright_black());
                break;
            }
        };

        match handle_menu_item(&menu_items[selection], &term)? {
            MenuAction::Continue => continue,
            MenuAction::Exit => {
                clear_screen(&term);
                println!("{}", constants::INFO_EXITING.bright_black());
                break;
            }
            MenuAction::ExitAfterSwitch => break,
        }
    }

    Ok(())
}
