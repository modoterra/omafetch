use crate::modules::types::ModuleOutput;

pub struct RenderDocument<'a> {
    pub rows: Vec<RenderRow<'a>>,
}

pub enum RenderRow<'a> {
    Output {
        output: &'a ModuleOutput,
        value: &'a str,
        show_label: bool,
    },
    Gap,
}

impl<'a> RenderDocument<'a> {
    pub fn from_outputs(outputs: &'a [ModuleOutput], show_group_gaps: bool) -> Self {
        let mut rows = Vec::new();

        for output in outputs {
            if show_group_gaps && starts_group(output.name) && !rows.is_empty() {
                rows.push(RenderRow::Gap);
            }

            for (index, value) in output.value.lines().enumerate() {
                rows.push(RenderRow::Output {
                    output,
                    value,
                    show_label: index == 0,
                });
            }
        }

        Self { rows }
    }

    pub fn label_width(&self) -> usize {
        self.rows
            .iter()
            .filter_map(|row| match row {
                RenderRow::Output {
                    output, show_label, ..
                } if *show_label => Some(output.label.len()),
                RenderRow::Output { .. } => None,
                RenderRow::Gap => None,
            })
            .max()
            .unwrap_or(0)
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }
}

fn starts_group(name: &str) -> bool {
    matches!(name, "theme" | "wm" | "display" | "memory" | "packages")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output(name: &'static str, label: &str, value: &str) -> ModuleOutput {
        ModuleOutput::new(name, label, value)
    }

    fn row_summary(document: &RenderDocument<'_>) -> Vec<String> {
        document
            .rows
            .iter()
            .map(|row| match row {
                RenderRow::Gap => "gap".to_string(),
                RenderRow::Output {
                    output,
                    value,
                    show_label,
                } => format!(
                    "{}:{}:{}",
                    output.name,
                    if *show_label {
                        output.label.as_str()
                    } else {
                        ""
                    },
                    value
                ),
            })
            .collect()
    }

    #[test]
    fn inserts_group_gaps_in_default_output() {
        let outputs = [
            output("omarchy", "Omarchy", "v1"),
            output("theme", "Theme", "Dazzle"),
            output("host", "Host", "Box"),
            output("wm", "WM", "Hyprland"),
            output("display", "Display", "DP-1"),
            output("memory", "Memory", "1.0 GiB"),
            output("packages", "Packages", "Pacman (1)"),
        ];

        assert_eq!(
            row_summary(&RenderDocument::from_outputs(&outputs, true)),
            vec![
                "omarchy:Omarchy:v1",
                "gap",
                "theme:Theme:Dazzle",
                "host:Host:Box",
                "gap",
                "wm:WM:Hyprland",
                "gap",
                "display:Display:DP-1",
                "gap",
                "memory:Memory:1.0 GiB",
                "gap",
                "packages:Packages:Pacman (1)",
            ]
        );
    }

    #[test]
    fn compact_output_has_no_group_gaps() {
        let outputs = [
            output("omarchy", "Omarchy", "v1"),
            output("theme", "Theme", "Dazzle"),
            output("wm", "WM", "Hyprland"),
        ];

        assert_eq!(
            row_summary(&RenderDocument::from_outputs(&outputs, false)),
            vec!["omarchy:Omarchy:v1", "theme:Theme:Dazzle", "wm:WM:Hyprland",]
        );
    }

    #[test]
    fn multi_line_values_keep_label_on_first_row() {
        let outputs = [output(
            "disk",
            "Disk",
            "root  1.0 GiB / 2.0 GiB\nhome  3.0 GiB / 4.0 GiB",
        )];

        assert_eq!(
            row_summary(&RenderDocument::from_outputs(&outputs, false)),
            vec![
                "disk:Disk:root  1.0 GiB / 2.0 GiB",
                "disk::home  3.0 GiB / 4.0 GiB"
            ]
        );
    }
}
