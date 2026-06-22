use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct Disk;

impl Module for Disk {
    fn name(&self) -> &'static str {
        "disk"
    }

    fn label(&self) -> &'static str {
        "Disk"
    }

    fn collect(&self, _ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        let disks = crate::probe::filesystem::mounted_disk_usages();
        let value = if disks.is_empty() {
            "unknown".to_string()
        } else {
            disks
                .into_iter()
                .map(|usage| {
                    let used = usage.total_bytes.saturating_sub(usage.available_bytes);
                    format!(
                        "{}  {} / {}",
                        usage.mount,
                        crate::render::text::format_bytes(used),
                        crate::render::text::format_bytes(usage.total_bytes)
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        Some(ModuleOutput::new(self.name(), self.label(), value))
    }
}
