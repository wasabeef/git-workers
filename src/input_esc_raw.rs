use colored::*;
use console::{Key, Term};
use std::io::{self, Write};

/// Custom input function that properly handles ESC key using raw terminal mode
pub fn input_with_esc_support_raw(prompt: &str, default: Option<&str>) -> Option<String> {
    // Display prompt similar to dialoguer style
    print!("{} {} ", "?".green().bold(), prompt);
    if let Some(def) = default {
        print!("{} ", format!("[{}]", def).bright_black());
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
                    print!("\r\x1b[K");
                    // Redraw prompt and current buffer
                    print!("{} {} ", "?".green().bold(), prompt);
                    if let Some(def) = default {
                        print!("{} ", format!("[{}]", def).bright_black());
                    }
                    print!("{}", buffer);
                    io::stdout().flush().unwrap();
                }
            }
            Ok(Key::Char(c)) => {
                if c == '\x15' {
                    // Ctrl+U - clear line
                    buffer.clear();
                    print!("\r\x1b[K");
                    print!("{} {} ", "?".green().bold(), prompt);
                    if let Some(def) = default {
                        print!("{} ", format!("[{}]", def).bright_black());
                    }
                    io::stdout().flush().unwrap();
                } else if c == '\x17' {
                    // Ctrl+W - delete word
                    // Delete last word
                    let trimmed = buffer.trim_end();
                    let last_space = trimmed.rfind(' ').map(|i| i + 1).unwrap_or(0);
                    buffer.truncate(last_space);
                    print!("\r\x1b[K");
                    print!("{} {} ", "?".green().bold(), prompt);
                    if let Some(def) = default {
                        print!("{} ", format!("[{}]", def).bright_black());
                    }
                    print!("{}", buffer);
                    io::stdout().flush().unwrap();
                } else {
                    buffer.push(c);
                    print!("{}", c);
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
pub fn input_esc_raw(prompt: &str) -> Option<String> {
    input_with_esc_support_raw(prompt, None)
}

/// Wrapper for input with default value and ESC support
pub fn input_esc_with_default_raw(prompt: &str, default: &str) -> Option<String> {
    input_with_esc_support_raw(prompt, Some(default))
}
