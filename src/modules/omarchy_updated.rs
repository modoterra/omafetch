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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_upgraded_line_wins() {
        let input = concat!(
            "[2026-07-01T09:00:00-0400] [ALPM] installed omafetch (0.1.0-1)\n",
            "[2026-07-15T10:00:00-0400] [ALPM] upgraded linux (6.15.1-1 -> 6.15.2-1)\n",
            "[2026-08-01T12:34:56-0400] [ALPM] upgraded omarchy (4.0.0-1 -> 4.0.0-2)\n",
            "[2026-08-01T12:35:00-0400] [ALPM] running '30-systemd-update.hook'\n",
        );

        assert_eq!(
            last_upgrade_from(input),
            Some("2026-08-01 12:34:56".to_string())
        );
    }

    #[test]
    fn empty_or_missing_upgrade_yields_none() {
        assert_eq!(last_upgrade_from(""), None);
        assert_eq!(
            last_upgrade_from("[2026-08-01T12:34:56-0400] [ALPM] installed foo (1-1)\n"),
            None
        );
    }
}
