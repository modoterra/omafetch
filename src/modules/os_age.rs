use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct OsAge;

impl Module for OsAge {
    fn name(&self) -> &'static str {
        "os-age"
    }

    fn label(&self) -> &'static str {
        "Age"
    }

    fn collect(&self, _ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        let value = crate::probe::filesystem::path_age_days("/")
            .map(|days| format!("{days} days"))
            .unwrap_or_else(|| "unknown".to_string());

        Some(ModuleOutput::new(self.name(), self.label(), value))
    }
}
