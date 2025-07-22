//! Test implementation of UserInterface for testing

use anyhow::Result;
use git_workers::ui::UserInterface;
use std::sync::{Arc, Mutex};

/// Test implementation of UserInterface that provides pre-programmed responses
#[derive(Clone)]
pub struct TestUI {
    inputs: Arc<Mutex<Vec<String>>>,
    selections: Arc<Mutex<Vec<usize>>>,
    confirmations: Arc<Mutex<Vec<bool>>>,
    expect_error: Arc<Mutex<bool>>,
}

impl TestUI {
    /// Create a new TestUI with no pre-programmed responses
    pub fn new() -> Self {
        Self {
            inputs: Arc::new(Mutex::new(Vec::new())),
            selections: Arc::new(Mutex::new(Vec::new())),
            confirmations: Arc::new(Mutex::new(Vec::new())),
            expect_error: Arc::new(Mutex::new(false)),
        }
    }

    /// Add an input response
    pub fn with_input(self, input: &str) -> Self {
        self.inputs.lock().unwrap().push(input.to_string());
        self
    }

    /// Add a selection response
    pub fn with_selection(self, selection: usize) -> Self {
        self.selections.lock().unwrap().push(selection);
        self
    }

    /// Add a confirmation response
    pub fn with_confirmation(self, confirm: bool) -> Self {
        self.confirmations.lock().unwrap().push(confirm);
        self
    }

    /// Indicate that the next operation should simulate an error/cancellation
    pub fn with_error(self) -> Self {
        *self.expect_error.lock().unwrap() = true;
        self
    }
}

impl UserInterface for TestUI {
    fn multiselect(&self, _prompt: &str, items: &[String]) -> Result<Vec<usize>> {
        // For tests, we don't support multiselect - just return empty selection
        let _ = items;
        Ok(vec![])
    }
    fn input(&self, _prompt: &str) -> Result<String> {
        let mut inputs = self.inputs.lock().unwrap();
        if *self.expect_error.lock().unwrap() {
            *self.expect_error.lock().unwrap() = false;
            return Err(anyhow::anyhow!("User cancelled input"));
        }
        if inputs.is_empty() {
            return Err(anyhow::anyhow!("No more test inputs"));
        }
        Ok(inputs.remove(0))
    }

    fn input_with_default(&self, _prompt: &str, default: &str) -> Result<String> {
        let mut inputs = self.inputs.lock().unwrap();
        if *self.expect_error.lock().unwrap() {
            *self.expect_error.lock().unwrap() = false;
            return Err(anyhow::anyhow!("User cancelled input"));
        }
        if inputs.is_empty() {
            Ok(default.to_string())
        } else {
            Ok(inputs.remove(0))
        }
    }

    fn select(&self, _prompt: &str, _items: &[String]) -> Result<usize> {
        let mut selections = self.selections.lock().unwrap();
        if *self.expect_error.lock().unwrap() {
            *self.expect_error.lock().unwrap() = false;
            return Err(anyhow::anyhow!("User cancelled selection"));
        }
        if selections.is_empty() {
            return Err(anyhow::anyhow!("No more test selections"));
        }
        Ok(selections.remove(0))
    }

    fn select_with_default(
        &self,
        _prompt: &str,
        _items: &[String],
        default: usize,
    ) -> Result<usize> {
        let mut selections = self.selections.lock().unwrap();
        if *self.expect_error.lock().unwrap() {
            *self.expect_error.lock().unwrap() = false;
            return Err(anyhow::anyhow!("User cancelled selection"));
        }
        if selections.is_empty() {
            Ok(default)
        } else {
            Ok(selections.remove(0))
        }
    }

    fn fuzzy_select(&self, _prompt: &str, items: &[String]) -> Result<usize> {
        // For tests, fuzzy select behaves like regular select
        self.select(_prompt, items)
    }

    fn confirm(&self, _prompt: &str) -> Result<bool> {
        let mut confirmations = self.confirmations.lock().unwrap();
        if *self.expect_error.lock().unwrap() {
            *self.expect_error.lock().unwrap() = false;
            return Err(anyhow::anyhow!("User cancelled confirmation"));
        }
        if confirmations.is_empty() {
            return Err(anyhow::anyhow!("No more test confirmations"));
        }
        Ok(confirmations.remove(0))
    }

    fn confirm_with_default(&self, _prompt: &str, default: bool) -> Result<bool> {
        let mut confirmations = self.confirmations.lock().unwrap();
        if *self.expect_error.lock().unwrap() {
            *self.expect_error.lock().unwrap() = false;
            return Err(anyhow::anyhow!("User cancelled confirmation"));
        }
        if confirmations.is_empty() {
            Ok(default)
        } else {
            Ok(confirmations.remove(0))
        }
    }
}

impl Default for TestUI {
    fn default() -> Self {
        Self::new()
    }
}
