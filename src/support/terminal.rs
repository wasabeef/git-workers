use console::Term;
use std::env;

use crate::constants;

pub fn clear_screen(term: &Term) {
    let _ = term.clear_screen();
}

pub fn setup_terminal_config() {
    #[cfg(windows)]
    {
        let _ = colored::control::set_virtual_terminal(true);
    }

    if env::var(constants::ENV_NO_COLOR).is_ok() {
        colored::control::set_override(false);
    } else if env::var(constants::ENV_FORCE_COLOR).is_ok()
        || env::var(constants::ENV_CLICOLOR_FORCE).unwrap_or_default()
            == constants::ENV_CLICOLOR_FORCE_VALUE
    {
        colored::control::set_override(true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear_screen_basic() {
        let term = Term::stdout();
        clear_screen(&term);
    }

    #[test]
    fn test_setup_terminal_config_variants() {
        std::env::set_var(constants::ENV_NO_COLOR, "1");
        setup_terminal_config();
        std::env::remove_var(constants::ENV_NO_COLOR);

        std::env::set_var(constants::ENV_FORCE_COLOR, "1");
        setup_terminal_config();
        std::env::remove_var(constants::ENV_FORCE_COLOR);

        std::env::set_var(
            constants::ENV_CLICOLOR_FORCE,
            constants::ENV_CLICOLOR_FORCE_VALUE,
        );
        setup_terminal_config();
        std::env::remove_var(constants::ENV_CLICOLOR_FORCE);
    }
}
