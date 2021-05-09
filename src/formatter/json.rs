use std::collections::HashMap;
use std::fmt::Display;

use serde::Serialize;

use crate::types;
use crate::types::GroupVersionKindHelper;

pub struct Json {
    result: JsonFullResult,
}

#[derive(Serialize)]
struct JsonResourceResult {
    group: String,
    kind: String,
    verb_allowed: HashMap<String, bool>,
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
            .map(|(k, v)| (k.to_lowercase(), v.allowed))
            .collect::<HashMap<String, bool>>();
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
