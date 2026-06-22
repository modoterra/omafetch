use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct Omarchy;

impl Module for Omarchy {
    fn name(&self) -> &'static str {
        "omarchy"
    }
    fn label(&self) -> &'static str {
        "Omarchy"
    }

    fn collect(&self, ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        let version = ctx
            .omarchy
            .version
            .as_ref()
            .map(|version| format!("v{version}"))
            .unwrap_or_else(|| "unknown".to_string());

        Some(ModuleOutput::new(self.name(), self.label(), version))
    }
}
