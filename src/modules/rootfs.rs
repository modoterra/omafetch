use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct RootFs;

impl Module for RootFs {
    fn name(&self) -> &'static str {
        "rootfs"
    }

    fn label(&self) -> &'static str {
        "Root"
    }

    fn collect(&self, _ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        let value = crate::probe::procfs::rootfs()
            .map(|root| {
                root.replace("subvol=/@", "@")
                    .replace("compress=", "")
                    .replace("  ", " ")
            })
            .unwrap_or_else(|| "unknown".to_string());

        Some(ModuleOutput::new(self.name(), self.label(), value))
    }
}
