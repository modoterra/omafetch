use crate::modules::types::{Module, ModuleContext, ModuleOutput};

pub struct Packages;

impl Module for Packages {
    fn name(&self) -> &'static str {
        "packages"
    }
    fn label(&self) -> &'static str {
        "Packages"
    }
    fn collect(&self, _ctx: &ModuleContext<'_>) -> Option<ModuleOutput> {
        let value = match crate::probe::filesystem::count_dirs("/var/lib/pacman/local") {
            Some(pacman) => match aur_package_count() {
                Some(aur) => format!("Pacman ({pacman}) AUR ({aur})"),
                None => format!("Pacman ({pacman})"),
            },
            None => "unknown".to_string(),
        };

        Some(ModuleOutput::new(self.name(), self.label(), value))
    }
}

fn aur_package_count() -> Option<usize> {
    crate::probe::command::run_capture("pacman", &["-Qqm"]).map(|output| {
        output
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count()
    })
}
