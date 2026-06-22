use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct OmarchyUpdated;

impl Module for OmarchyUpdated {
    fn name(&self) -> &'static str {
        "omarchy-updated"
    }

    fn label(&self) -> &'static str {
        "Updated"
    }

    fn collect(&self, _ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        let value = crate::probe::filesystem::read_to_string("/var/log/pacman.log")
            .and_then(|input| last_upgrade_from(&input))
            .unwrap_or_else(|| "unknown".to_string());

        Some(ModuleOutput::new(self.name(), self.label(), value))
    }
}

fn last_upgrade_from(input: &str) -> Option<String> {
    input
        .lines()
        .rev()
        .find(|line| line.contains(" upgraded "))
        .and_then(|line| line.strip_prefix('['))
        .and_then(|line| line.split_once(']'))
        .map(|(date, _)| format_pacman_timestamp(date))
}

fn format_pacman_timestamp(value: &str) -> String {
    let without_offset = value
        .rsplit_once('-')
        .map(|(date, _)| date)
        .unwrap_or(value);
    without_offset.replace('T', " ")
}
