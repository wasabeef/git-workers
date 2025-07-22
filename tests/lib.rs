//! Test library root module
//!
//! This file enables the hierarchical test structure.

#[cfg(test)]
mod unit {
    mod commands;
    mod core;
    mod infrastructure;
    mod ui;
}

#[cfg(test)]
mod integration {
    mod git_flow;
    mod list_ui_display_test;
    mod multi_repo;
    mod repository_info_display_test;
    mod worktree_lifecycle;
}

#[cfg(test)]
mod e2e {
    mod workflow;
}

#[cfg(test)]
mod performance {
    mod benchmark;
}
