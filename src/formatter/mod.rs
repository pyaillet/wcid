use std::fmt::Display;

use crate::config;
use crate::types;

mod json;
mod pretty;

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
