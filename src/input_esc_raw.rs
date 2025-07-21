//! Custom input handling with ESC key support
//!
//! This module provides input functions that properly handle the ESC key
//! for cancellation, which is not natively supported by the dialoguer crate.
//! It uses raw terminal mode to capture individual keystrokes.
//!
//! # Features
//!
//! - ESC key cancellation support
//! - Default value support with visual indication
//! - Line editing capabilities (backspace, Ctrl+U, Ctrl+W)
//! - Consistent styling with dialoguer prompts
//!
//! # Key Bindings
//!
//! - `ESC`: Cancel input and return `None`
//! - `Enter`: Accept input (or default if empty)
//! - `Backspace`: Delete previous character
//! - `Ctrl+U`: Clear entire line
//! - `Ctrl+W`: Delete previous word

use crate::constants::*;
use colored::*;
use console::{Key, Term};
use std::io::{self, Write};

/// Custom input function that properly handles ESC key using raw terminal mode
///
/// This is the core implementation that provides line editing capabilities
/// with ESC cancellation support. It reads individual keystrokes in raw mode
/// to provide immediate feedback and proper ESC handling.
///
/// # Arguments
///
/// * `prompt` - The prompt text to display to the user
/// * `default` - Optional default value shown in brackets and used if user presses Enter with empty input
///
/// # Returns
///
/// * `Some(String)` - The entered text (or default if applicable)
/// * `None` - If the user pressed ESC or an error occurred
///
/// # Terminal Handling
///
/// The function uses ANSI escape sequences for cursor control:
/// - `\r` - Carriage return to move cursor to start of line
/// - `\x1b[K` - Clear from cursor to end of line
///
/// This allows for proper redrawing when editing the input.
pub fn input_with_esc_support_raw(prompt: &str, default: Option<&str>) -> Option<String> {
    // Display prompt similar to dialoguer style
    let question_mark = ICON_QUESTION.green().bold();
    print!("{question_mark} {prompt} ");
    if let Some(def) = default {
        let default_text = FORMAT_DEFAULT_VALUE.replace("{}", def).bright_black();
        print!("{default_text} ");
    }
    io::stdout().flush().unwrap();

    let term = Term::stdout();
    let mut buffer = String::new();

    loop {
        match term.read_key() {
            Ok(Key::Escape) => {
                println!();
                return None; // ESC pressed
            }
            Ok(Key::Enter) => {
                println!();
                if buffer.is_empty() && default.is_some() {
                    return default.map(|s| s.to_string());
                }
                return Some(buffer);
            }
            Ok(Key::Backspace) => {
                if !buffer.is_empty() {
                    buffer.pop();
                    // Move cursor to beginning of line and clear to end
                    print!("{ANSI_CLEAR_LINE}");
                    // Redraw prompt and current buffer
                    let question_mark = ICON_QUESTION.green().bold();
                    print!("{question_mark} {prompt} ");
                    if let Some(def) = default {
                        let default_text = FORMAT_DEFAULT_VALUE.replace("{}", def).bright_black();
                        print!("{default_text} ");
                    }
                    print!("{buffer}");
                    io::stdout().flush().unwrap();
                }
            }
            Ok(Key::Char(c)) => {
                if c == CTRL_U {
                    // Ctrl+U - clear line
                    buffer.clear();
                    print!("{ANSI_CLEAR_LINE}");
                    let question_mark = ICON_QUESTION.green().bold();
                    print!("{question_mark} {prompt} ");
                    if let Some(def) = default {
                        let default_text = FORMAT_DEFAULT_VALUE.replace("{}", def).bright_black();
                        print!("{default_text} ");
                    }
                    io::stdout().flush().unwrap();
                } else if c == CTRL_W {
                    // Ctrl+W - delete word
                    // Delete last word
                    let trimmed = buffer.trim_end();
                    let last_space = trimmed.rfind(CHAR_SPACE).map(|i| i + 1).unwrap_or(0);
                    buffer.truncate(last_space);
                    print!("{ANSI_CLEAR_LINE}");
                    let question_mark = ICON_QUESTION.green().bold();
                    print!("{question_mark} {prompt} ");
                    if let Some(def) = default {
                        let default_text = FORMAT_DEFAULT_VALUE.replace("{}", def).bright_black();
                        print!("{default_text} ");
                    }
                    print!("{buffer}");
                    io::stdout().flush().unwrap();
                } else {
                    buffer.push(c);
                    print!("{c}");
                    io::stdout().flush().unwrap();
                }
            }
            Ok(_) => {} // Ignore other keys
            Err(_) => {
                println!();
                return None; // Error reading key
            }
        }
    }
}

/// Wrapper for simple input with ESC support
///
/// Convenience function for input without a default value.
///
/// # Arguments
///
/// * `prompt` - The prompt text to display
///
/// # Returns
///
/// * `Some(String)` - The entered text
/// * `None` - If the user pressed ESC
///
/// # Example
///
/// ```no_run
/// use git_workers::input_esc_raw::input_esc_raw;
///
/// match input_esc_raw("Enter worktree name") {
///     Some(name) => println!("Creating worktree: {}", name),
///     None => println!("Operation cancelled"),
/// }
/// ```
#[allow(dead_code)]
pub fn input_esc_raw(prompt: &str) -> Option<String> {
    input_with_esc_support_raw(prompt, None)
}

/// Wrapper for input with default value and ESC support
///
/// Convenience function for input with a default value. The default is shown
/// in brackets and will be used if the user presses Enter without typing.
///
/// # Arguments
///
/// * `prompt` - The prompt text to display
/// * `default` - The default value to use if no input is provided
///
/// # Returns
///
/// * `Some(String)` - The entered text or default value
/// * `None` - If the user pressed ESC
///
/// # Example
///
/// ```no_run
/// use git_workers::input_esc_raw::input_esc_with_default_raw;
///
/// match input_esc_with_default_raw("Days to keep", "30") {
///     Some(days) => println!("Will keep worktrees for {} days", days),
///     None => println!("Operation cancelled"),
/// }
/// ```
pub fn input_esc_with_default_raw(prompt: &str, default: &str) -> Option<String> {
    input_with_esc_support_raw(prompt, Some(default))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_esc_raw_function_exists() {
        // Test that the function signature is correct
        // We can't actually test the interactive behavior in unit tests
        // but we can ensure the function compiles and is callable
        let _f: fn(&str) -> Option<String> = input_esc_raw;
    }

    #[test]
    fn test_input_esc_with_default_function_exists() {
        // Test that the function signature is correct
        let _f: fn(&str, &str) -> Option<String> = input_esc_with_default_raw;
    }

    #[test]
    fn test_input_with_esc_support_raw_function_exists() {
        // Test that the core function signature is correct
        let _f: fn(&str, Option<&str>) -> Option<String> = input_with_esc_support_raw;
    }

    #[test]
    fn test_constants_are_accessible() {
        // Test that all required constants are accessible
        assert_eq!(ICON_QUESTION, "?");
        assert_eq!(CTRL_U, '\u{15}');
        assert_eq!(CTRL_W, '\u{17}');
        assert_eq!(CHAR_SPACE, ' ');
        assert_eq!(ANSI_CLEAR_LINE, "\r\x1b[K");
    }
}
