use anyhow::Result;

use crate::ui::UserInterface;

pub type HookContext = crate::infrastructure::hooks::HookContext;

pub fn execute_hooks(hook_type: &str, context: &HookContext) -> Result<()> {
    crate::infrastructure::hooks::execute_hooks(hook_type, context)
}

pub fn execute_hooks_with_ui(
    hook_type: &str,
    context: &HookContext,
    ui: &dyn UserInterface,
) -> Result<()> {
    crate::infrastructure::hooks::execute_hooks_with_ui(hook_type, context, ui)
}
