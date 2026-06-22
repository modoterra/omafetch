use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct KernelConfig;

impl Module for KernelConfig {
    fn name(&self) -> &'static str {
        "kernel-config"
    }

    fn label(&self) -> &'static str {
        "Config"
    }

    fn collect(&self, _ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        let values = crate::probe::sysfs::kernel_runtime_config();
        let value = if values.is_empty() {
            "unknown".to_string()
        } else {
            values.join("\n")
        };

        Some(ModuleOutput::new(self.name(), self.label(), value))
    }
}
