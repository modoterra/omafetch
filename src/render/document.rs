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
