use crate::config;
use crate::types;

use comfy_table::{presets::NOTHING, Attribute, Cell, CellAlignment, Color, Table};
use std::fmt::Display;

pub struct Pretty {
    config: config::Config,
    result: types::FullResult,
}

impl Pretty {
    pub fn new(config: config::Config, result: types::FullResult) -> Self {
        Self { config, result }
    }
}

impl Display for Pretty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = Table::new();
        table.load_preset(NOTHING);

        let column_count = self.config.verbs.len() + if self.config.display_group { 2 } else { 1 };

        let mut titles = Vec::with_capacity(column_count);
        if self.config.display_group {
            titles.push(Cell::new("Group").add_attribute(Attribute::Bold));
        }
        titles.push(Cell::new("Kind").add_attribute(Attribute::Bold));
        titles.extend(
            self.config
                .verbs
                .iter()
                .map(|v| Cell::new(v).add_attribute(Attribute::Bold))
                .collect::<Vec<Cell>>(),
        );
        table.set_header(titles);

        let mut resource_result = self.result.items.clone();
        resource_result.sort_by(|a, b| {
            if self.config.display_group {
                match a.resource.group.cmp(&b.resource.group) {
                    std::cmp::Ordering::Equal => a.resource.kind.cmp(&b.resource.kind),
                    std::cmp::Ordering::Less => std::cmp::Ordering::Less,
                    std::cmp::Ordering::Greater => std::cmp::Ordering::Greater,
                }
            } else {
                a.resource.kind.cmp(&b.resource.kind)
            }
        });
        resource_result.iter().for_each(|result| {
            let mut row: Vec<Cell> = Vec::with_capacity(column_count);
            let resource = result.resource.clone();
            if self.config.display_group {
                row.push(Cell::new(
                    resource.group.unwrap_or_else(|| String::from("")),
                ));
            }
            row.push(Cell::new(resource.kind));
            row.extend(
                self.config
                    .verbs
                    .iter()
                    .map(|v| match &result.items.get(v) {
                        Some(r) => {
                            if r.allowed {
                                Cell::new("✔")
                                    .fg(Color::AnsiValue(34))
                                    .set_alignment(CellAlignment::Center)
                            } else {
                                Cell::new("✖")
                                    .fg(Color::Red)
                                    .set_alignment(CellAlignment::Center)
                            }
                        }
                        None => Cell::new(""),
                    })
                    .collect::<Vec<Cell>>(),
            );
            table.add_row(row);
        });

        table.fmt(f)
    }
}
