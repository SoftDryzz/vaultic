use crate::cli::TemplateAction;
use crate::core::errors::Result;

/// Execute `vaultic template` subcommands.
pub fn execute(action: &TemplateAction) -> Result<()> {
    match action {
        TemplateAction::Sync { output } => {
            // TODO: implement in Task 3
            let _ = output;
            crate::cli::output::success("template sync: not yet implemented");
            Ok(())
        }
    }
}
