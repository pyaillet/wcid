use super::super::config;
use super::super::constants;
use super::super::types;

use crate::check::types::GroupVersionKindHelper;

use comfy_table::{presets::NOTHING, Attribute, Cell, Color, Table};
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

        let column_count = if self.config.display_group { 9 } else { 8 };

        let mut titles = Vec::with_capacity(column_count);
        if self.config.display_group {
            titles.push(Cell::new("Group").add_attribute(Attribute::Bold));
        }
        titles.push(Cell::new("Kind").add_attribute(Attribute::Bold));
        titles.extend(
            constants::ALL_VERBS
                .iter()
                .map(|v| Cell::new(v).add_attribute(Attribute::Bold))
                .collect::<Vec<Cell>>(),
        );
        table.set_header(titles);

        self.result.items.iter().for_each(|result| {
            let mut row: Vec<Cell> = Vec::with_capacity(column_count);
            if self.config.display_group {
                row.push(Cell::new(&result.group()));
            }
            row.push(Cell::new(&result.kind()));
            row.extend(
                constants::ALL_VERBS
                    .iter()
                    .map(|v| match &result.items.get(v) {
                        Some(r) => {
                            if r.allowed {
                                Cell::new("✔").fg(Color::Green)
                            } else {
                                Cell::new("✖").fg(Color::Red)
                            }
                        }
                        None => {
                            println!("Not found");
                            Cell::new("✖").fg(Color::Red)
                        }
                    })
                    .collect::<Vec<Cell>>(),
            );
            table.add_row(row);
        });

        table.fmt(f)
    }
}
