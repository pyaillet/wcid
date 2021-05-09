use std::fmt::Display;

use super::config;
use super::types;

pub enum Formatter {
    Pretty(pretty::Pretty),
    Json(json::Json),
}

impl Formatter {
    pub fn new(format: String, config: config::Config, result: types::FullResult) -> Self {
        if format == "json" {
            Formatter::Json(json::Json::new(result))
        } else {
            Formatter::Pretty(pretty::Pretty::new(config, result))
        }
    }
}

impl Display for Formatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Formatter::Pretty(p) => p.fmt(f),
            Formatter::Json(j) => j.fmt(f),
        }
    }
}

pub mod pretty {

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
}

pub mod json {

    use std::collections::HashMap;
    use std::fmt::Display;

    use serde::Serialize;

    use super::super::types;

    use crate::check::types::GroupVersionKindHelper;

    pub struct Json {
        result: JsonFullResult,
    }

    #[derive(Serialize)]
    struct JsonResourceResult {
        group: String,
        kind: String,
        verb_allowed: HashMap<&'static str, bool>,
    }

    #[derive(Serialize)]
    struct JsonFullResult {
        items: Vec<JsonResourceResult>,
    }

    impl From<types::ResourceCheckResult> for JsonResourceResult {
        fn from(value: types::ResourceCheckResult) -> Self {
            let verb_allowed = value
                .items
                .into_iter()
                .map(|(k, v)| (k, v.allowed))
                .collect::<HashMap<&'static str, bool>>();
            Self {
                group: value.gvk.group(),
                kind: value.gvk.kind(),
                verb_allowed,
            }
        }
    }

    impl From<types::FullResult> for JsonFullResult {
        fn from(value: types::FullResult) -> Self {
            Self {
                items: value
                    .items
                    .into_iter()
                    .map(|i| i.into())
                    .collect::<Vec<JsonResourceResult>>(),
            }
        }
    }

    impl Json {
        pub fn new(full_result: types::FullResult) -> Self {
            Self {
                result: full_result.into(),
            }
        }
    }

    impl Display for Json {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match serde_json::to_string(&self.result) {
                Ok(output) => f.write_str(&output),
                Err(_e) => Err(std::fmt::Error),
            }
        }
    }
}
