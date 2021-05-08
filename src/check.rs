use std::{collections::HashMap, fmt::Display};

use anyhow::Result;
use comfy_table::{presets::NOTHING, Attribute, Cell, Color, Table};
use futures::future::try_join_all;
use kube::Client;
use kube::{api::DynamicObject, Resource};
use kube::{api::GroupVersionKind, client::Discovery};
use serde_json::json;

use k8s_openapi::api::authorization::v1::SelfSubjectAccessReview;

const ALL_VERBS: [&str; 7] = [
    "Get", "List", "Watch", "Create", "Delete", "Update", "Patch",
];

#[derive(Clone, Debug)]
pub struct ResourceCheckResult {
    pub gvk: GroupVersionKind,
    pub results: HashMap<&'static str, CheckResult>,
}

pub trait GroupVersionKindHelper {
    fn plural(&self) -> String;
    fn kind(&self) -> String;
    fn group(&self) -> String;
}

impl GroupVersionKindHelper for GroupVersionKind {
    fn plural(&self) -> String {
        DynamicObject::plural(&self).to_string()
    }

    fn kind(&self) -> String {
        DynamicObject::kind(&self).to_string()
    }

    fn group(&self) -> String {
        DynamicObject::group(&self).to_string()
    }
}

impl GroupVersionKindHelper for ResourceCheckResult {
    fn plural(&self) -> String {
        DynamicObject::plural(&self.gvk).to_string()
    }

    fn kind(&self) -> String {
        self.gvk.kind()
    }

    fn group(&self) -> String {
        self.gvk.group()
    }
}

#[derive(Clone, Debug)]
pub struct CheckResult {
    verb: &'static str,
    allowed: bool,
    denied: bool,
}

#[derive(Clone, Debug)]
pub struct FullResult {
    pub config: Config,
    pub results: Vec<ResourceCheckResult>,
}

async fn check_resource_verb(
    gvk: &GroupVersionKind,
    verb: &'static str,
    namespace: Option<String>,
) -> Result<CheckResult> {
    let client = Client::try_default().await?;

    let ssar: SelfSubjectAccessReview = serde_json::from_value(json!({
        "apiVersion": "authorization.k8s.io/v1",
        "kind": "SelfSubjectAccessReview",
        "metadata": {},
        "spec": {
            "resourceAttributes": {
              "group": gvk.group(),
              "resource": gvk.plural(),
              "namespace": namespace,
              "verb": verb.to_ascii_lowercase(),
            },
        }
    }))?;

    let (reqp, _) =
        SelfSubjectAccessReview::create_self_subject_access_review(&ssar, Default::default())?;
    let res = client
        .request::<SelfSubjectAccessReview>(
            http::Request::post(reqp.uri())
                .body(reqp.body().clone())
                .expect("Unable to prepare HTTP request"),
        )
        .await?;
    let status = res.status.expect("K8s answered with an empty status");
    Ok(CheckResult {
        verb,
        allowed: status.allowed,
        denied: status.denied.unwrap_or(false),
    })
}

async fn check_resource(
    gvk: &GroupVersionKind,
    namespace: Option<String>,
) -> Result<ResourceCheckResult> {
    let mut results: HashMap<&'static str, CheckResult> = HashMap::new();
    for verb in ALL_VERBS.iter() {
        results.insert(
            verb,
            check_resource_verb(&gvk, verb, namespace.clone()).await?,
        );
    }
    Ok(ResourceCheckResult {
        gvk: gvk.clone(),
        results,
    })
}

async fn list_resources() -> Result<Vec<GroupVersionKind>> {
    let client = Client::try_default().await?;

    let discovery = Discovery::new(&client).await?;
    let mut v = Vec::new();

    for group in discovery.groups() {
        let ver = group.preferred_version_or_guess();
        for gvk in group.resources_by_version(ver) {
            v.push(gvk);
        }
    }
    Ok(v)
}

#[derive(Clone, Debug)]
pub struct Config {
    pub display_group: bool,
    pub namespace: Option<String>,
}

pub struct Checker {
    pub config: Config,
}

impl Checker {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub async fn check_all(&self) -> Result<FullResult> {
        let resources = list_resources().await?;
        let future_results: Vec<_> = resources
            .iter()
            .map(|gvk| check_resource(gvk, self.config.namespace.clone()))
            .collect();
        let results = try_join_all(future_results).await?;

        Ok(FullResult {
            config: self.config.clone(),
            results,
        })
    }
}

impl Display for FullResult {
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
            ALL_VERBS
                .iter()
                .map(|v| Cell::new(v).add_attribute(Attribute::Bold))
                .collect::<Vec<Cell>>(),
        );
        table.set_header(titles);

        self.results.iter().for_each(|result| {
            let mut row: Vec<Cell> = Vec::with_capacity(column_count);
            if self.config.display_group {
                row.push(Cell::new(&result.group()));
            }
            row.push(Cell::new(&result.kind()));
            row.extend(
                ALL_VERBS
                    .iter()
                    .map(|v| match &result.results.get(v) {
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
