//! User Interface abstraction layer
//!
//! This module provides an abstraction over user interface interactions,
//! allowing for testable code by separating business logic from UI dependencies.

use anyhow::Result;
use dialoguer::{Confirm, FuzzySelect, Input, MultiSelect, Select};
use std::collections::VecDeque;

/// Trait for user interface interactions
///
/// This trait abstracts all user input operations, making the code testable
/// by allowing mock implementations for testing and real implementations for production.
pub trait UserInterface {
    /// Display a selection menu and return the selected index
    fn select(&self, prompt: &str, items: &[String]) -> Result<usize>;

    /// Display a fuzzy-searchable selection menu and return the selected index
    fn fuzzy_select(&self, prompt: &str, items: &[String]) -> Result<usize>;

    /// Get text input from user
    fn input(&self, prompt: &str) -> Result<String>;

    /// Get text input with a default value
    fn input_with_default(&self, prompt: &str, default: &str) -> Result<String>;

    /// Ask for user confirmation (yes/no)
    #[allow(dead_code)]
    fn confirm(&self, prompt: &str) -> Result<bool>;

    /// Ask for user confirmation with a default value
    fn confirm_with_default(&self, prompt: &str, default: bool) -> Result<bool>;

    /// Display a multi-selection menu and return selected indices
    #[allow(dead_code)]
    fn multiselect(&self, prompt: &str, items: &[String]) -> Result<Vec<usize>>;
}

/// Production implementation using dialoguer
pub struct DialoguerUI;

impl UserInterface for DialoguerUI {
    fn select(&self, prompt: &str, items: &[String]) -> Result<usize> {
        let selection = Select::new()
            .with_prompt(prompt)
            .items(items)
            .interact_opt()?;
        selection.ok_or_else(|| anyhow::anyhow!("User cancelled selection"))
    }

    fn fuzzy_select(&self, prompt: &str, items: &[String]) -> Result<usize> {
        let selection = FuzzySelect::new()
            .with_prompt(prompt)
            .items(items)
            .interact_opt()?;
        selection.ok_or_else(|| anyhow::anyhow!("User cancelled fuzzy selection"))
    }

    fn input(&self, prompt: &str) -> Result<String> {
        let input = Input::<String>::new().with_prompt(prompt).interact_text()?;
        Ok(input)
    }

    fn input_with_default(&self, prompt: &str, default: &str) -> Result<String> {
        let input = Input::<String>::new()
            .with_prompt(prompt)
            .default(default.to_string())
            .interact_text()?;
        Ok(input)
    }

    fn confirm(&self, prompt: &str) -> Result<bool> {
        let confirmed = Confirm::new().with_prompt(prompt).interact_opt()?;
        confirmed.ok_or_else(|| anyhow::anyhow!("User cancelled confirmation"))
    }

    fn confirm_with_default(&self, prompt: &str, default: bool) -> Result<bool> {
        let confirmed = Confirm::new()
            .with_prompt(prompt)
            .default(default)
            .interact_opt()?;
        confirmed.ok_or_else(|| anyhow::anyhow!("User cancelled confirmation"))
    }

    fn multiselect(&self, prompt: &str, items: &[String]) -> Result<Vec<usize>> {
        let selections = MultiSelect::new()
            .with_prompt(prompt)
            .items(items)
            .interact_opt()?;
        selections.ok_or_else(|| anyhow::anyhow!("User cancelled multiselection"))
    }
}

/// Mock implementation for testing
///
/// Uses interior mutability to allow mutable access through immutable references,
/// enabling testable UI interactions in the UserInterface trait.
pub struct MockUI {
    selections: std::cell::RefCell<VecDeque<usize>>,
    inputs: std::cell::RefCell<VecDeque<String>>,
    confirms: std::cell::RefCell<VecDeque<bool>>,
    multiselects: std::cell::RefCell<VecDeque<Vec<usize>>>,
}

impl Default for MockUI {
    fn default() -> Self {
        Self::new()
    }
}

impl MockUI {
    /// Create a new MockUI instance
    pub fn new() -> Self {
        Self {
            selections: std::cell::RefCell::new(VecDeque::new()),
            inputs: std::cell::RefCell::new(VecDeque::new()),
            confirms: std::cell::RefCell::new(VecDeque::new()),
            multiselects: std::cell::RefCell::new(VecDeque::new()),
        }
    }

    /// Add a selection response (for select() calls)
    #[allow(dead_code)]
    pub fn with_selection(self, selection: usize) -> Self {
        self.selections.borrow_mut().push_back(selection);
        self
    }

    /// Add an input response (for input() calls)
    #[allow(dead_code)]
    pub fn with_input(self, input: impl Into<String>) -> Self {
        self.inputs.borrow_mut().push_back(input.into());
        self
    }

    /// Add a confirmation response (for confirm() calls)
    #[allow(dead_code)]
    pub fn with_confirm(self, confirm: bool) -> Self {
        self.confirms.borrow_mut().push_back(confirm);
        self
    }

    /// Add a multiselect response (for multiselect() calls)
    #[allow(dead_code)]
    pub fn with_multiselect(self, selections: Vec<usize>) -> Self {
        self.multiselects.borrow_mut().push_back(selections);
        self
    }

    /// Check if all configured responses have been consumed
    #[allow(dead_code)]
    pub fn is_exhausted(&self) -> bool {
        self.selections.borrow().is_empty()
            && self.inputs.borrow().is_empty()
            && self.confirms.borrow().is_empty()
            && self.multiselects.borrow().is_empty()
    }
}

impl UserInterface for MockUI {
    fn select(&self, _prompt: &str, _items: &[String]) -> Result<usize> {
        self.selections
            .borrow_mut()
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("No more selections configured for MockUI"))
    }

    fn fuzzy_select(&self, _prompt: &str, _items: &[String]) -> Result<usize> {
        // For testing, fuzzy select behaves the same as regular select
        self.selections
            .borrow_mut()
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("No more selections configured for MockUI"))
    }

    fn input(&self, _prompt: &str) -> Result<String> {
        self.inputs
            .borrow_mut()
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("No more inputs configured for MockUI"))
    }

    fn input_with_default(&self, _prompt: &str, default: &str) -> Result<String> {
        // Try to get configured input, fall back to default
        if let Some(input) = self.inputs.borrow_mut().pop_front() {
            Ok(input)
        } else {
            Ok(default.to_string())
        }
    }

    fn confirm(&self, _prompt: &str) -> Result<bool> {
        self.confirms
            .borrow_mut()
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("No more confirmations configured for MockUI"))
    }

    fn confirm_with_default(&self, _prompt: &str, default: bool) -> Result<bool> {
        // Try to get configured confirmation, fall back to default
        if let Some(confirm) = self.confirms.borrow_mut().pop_front() {
            Ok(confirm)
        } else {
            Ok(default)
        }
    }

    fn multiselect(&self, _prompt: &str, _items: &[String]) -> Result<Vec<usize>> {
        self.multiselects
            .borrow_mut()
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("No more multiselects configured for MockUI"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_ui_creation() {
        let mock_ui = MockUI::new()
            .with_selection(1)
            .with_input("test-branch")
            .with_confirm(true)
            .with_multiselect(vec![0, 2]);

        // MockUI should be created successfully
        assert_eq!(mock_ui.selections.borrow().len(), 1);
        assert_eq!(mock_ui.inputs.borrow().len(), 1);
        assert_eq!(mock_ui.confirms.borrow().len(), 1);
        assert_eq!(mock_ui.multiselects.borrow().len(), 1);
    }

    #[test]
    fn test_mock_ui_exhaustion_check() {
        let mock_ui = MockUI::new();
        assert!(mock_ui.is_exhausted());

        let mock_ui = MockUI::new().with_selection(0);
        assert!(!mock_ui.is_exhausted());
    }

    #[test]
    fn test_dialoguer_ui_trait_implementation() {
        let _ui = DialoguerUI;
        // DialoguerUI should implement UserInterface trait
        // This test just verifies the struct can be instantiated
    }

    #[test]
    fn test_mock_ui_functional_behavior() -> Result<()> {
        let mock_ui = MockUI::new()
            .with_selection(2)
            .with_selection(3) // For fuzzy_select
            .with_input("feature-branch")
            .with_confirm(false)
            .with_confirm(true) // For confirm_with_default fallback
            .with_multiselect(vec![1, 3]);

        // Test that the methods return configured values
        assert_eq!(
            mock_ui.select("test", &["a".to_string(), "b".to_string()])?,
            2
        );
        assert_eq!(
            mock_ui.fuzzy_select("test", &["a".to_string(), "b".to_string()])?,
            3
        );
        assert_eq!(mock_ui.input("test")?, "feature-branch");
        assert!(!mock_ui.confirm("test")?);
        assert!(mock_ui.confirm_with_default("test", false)?);
        assert_eq!(mock_ui.multiselect("test", &["a".to_string()])?, vec![1, 3]);

        // Now the mock should be exhausted
        assert!(mock_ui.is_exhausted());

        Ok(())
    }

    #[test]
    fn test_mock_ui_input_with_default() -> Result<()> {
        let mock_ui = MockUI::new().with_input("custom-input");

        // Should return configured input
        assert_eq!(
            mock_ui.input_with_default("test", "default")?,
            "custom-input"
        );

        // Should now fall back to default since no more inputs configured
        assert_eq!(mock_ui.input_with_default("test", "fallback")?, "fallback");

        Ok(())
    }

    #[test]
    fn test_mock_ui_confirm_with_default() -> Result<()> {
        let mock_ui = MockUI::new().with_confirm(false);

        // Should return configured confirmation
        assert!(!mock_ui.confirm_with_default("test", true)?);

        // Should now fall back to default since no more confirmations configured
        assert!(mock_ui.confirm_with_default("test", true)?);

        Ok(())
    }

    #[test]
    fn test_mock_ui_error_on_exhaustion() {
        let mock_ui = MockUI::new();

        // Should error when no responses are configured
        assert!(mock_ui.select("test", &["a".to_string()]).is_err());
        assert!(mock_ui.fuzzy_select("test", &["a".to_string()]).is_err());
        assert!(mock_ui.input("test").is_err());
        assert!(mock_ui.confirm("test").is_err());
        assert!(mock_ui.multiselect("test", &["a".to_string()]).is_err());
    }
}
