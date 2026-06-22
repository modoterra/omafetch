use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct Battery;

impl Module for Battery {
    fn name(&self) -> &'static str {
        "battery"
    }

    fn label(&self) -> &'static str {
        "Battery"
    }

    fn collect(&self, _ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        let value = crate::probe::sysfs::battery()
            .map(format_battery)
            .unwrap_or_else(|| "unknown".to_string());

        Some(ModuleOutput::new(self.name(), self.label(), value))
    }
}

fn format_battery(info: crate::probe::sysfs::BatteryInfo) -> String {
    match (info.capacity, info.status) {
        (Some(capacity), Some(status)) => format!("{capacity}% {status}"),
        (Some(capacity), None) => format!("{capacity}%"),
        (None, Some(status)) => status,
        (None, None) => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_battery_capacity_and_status() {
        assert_eq!(
            format_battery(crate::probe::sysfs::BatteryInfo {
                capacity: Some(87),
                status: Some("charging".to_string())
            }),
            "87% charging"
        );
    }
}
