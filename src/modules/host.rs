use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct Host;

impl Module for Host {
    fn name(&self) -> &'static str {
        "host"
    }
    fn label(&self) -> &'static str {
        "Host"
    }
    fn collect(&self, _ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        Some(ModuleOutput::new(
            self.name(),
            self.label(),
            crate::probe::sysfs::host_model().unwrap_or_else(|| "unknown".to_string()),
        ))
    }
}
