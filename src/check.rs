use std::{collections::HashMap, fmt::Display};

use anyhow::Result;
use async_stream::stream;
use futures::{future::try_join_all, Future};
use kube::Client;
use kube::{api::DynamicObject, Resource};
use kube::{api::GroupVersionKind, client::Discovery};
use lazy_static::lazy_static;
use prettytable::{format, Cell, Row, Table};
use serde_json::json;

use k8s_openapi::api::authorization::v1::SelfSubjectAccessReview;

lazy_static! {
    static ref ALL_VERBS: Vec<&'static str> =
        vec!["Get", "List", "Watch", "Create", "Delete", "Update", "Patch"];
}

#[derive(Clone, Debug)]
pub struct Config {
    pub display_group: bool,
    pub namespace: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            display_group: false,
            namespace: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ResourceCheckResult {
    pub gvk: GroupVersionKind,
    pub results: HashMap<&'static str, CheckResult>,
}

pub trait GroupVersionKindHelper {
    fn kind(&self) -> String;
    fn group(&self) -> String;
}

impl GroupVersionKindHelper for GroupVersionKind {
    fn kind(&self) -> String {
        DynamicObject::kind(&self).to_string()
    }

    fn group(&self) -> String {
        DynamicObject::group(&self).to_string()
    }
}

impl GroupVersionKindHelper for ResourceCheckResult {
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
              "resource": gvk.kind(),
              "namespace": namespace,
              "verb": verb,
            },
        }
    }))?;

    let (reqp, _) =
        SelfSubjectAccessReview::create_self_subject_access_review(&ssar, Default::default())?;
    let res = client
        .request::<SelfSubjectAccessReview>(
            http::Request::post(reqp.uri())
                .body(reqp.body().clone())
                .unwrap(),
        )
        .await?;
    let status = res.status.unwrap();
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

async fn check_resource_global(gvk: &GroupVersionKind) -> Result<ResourceCheckResult> {
    check_resource(gvk, None).await
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

pub async fn check_all() -> Result<FullResult> {
    let resources = list_resources().await?;
    let future_results: Vec<_> = resources
        .iter()
        .map(|gvk| check_resource_global(gvk))
        .collect();
    let results = try_join_all(future_results).await?;

    Ok(FullResult {
        config: Config::default(),
        results,
    })
}

impl Display for FullResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rows: Vec<Row> = Vec::new();

        let mut titles = Vec::new();
        if self.config.display_group {
            titles.push(Cell::new("Group").style_spec("b"));
        }
        titles.push(Cell::new("Kind").style_spec("b"));
        titles.extend(
            ALL_VERBS
                .iter()
                .map(|v| Cell::new(v).style_spec("b"))
                .collect::<Vec<Cell>>(),
        );
        rows.push(Row::new(titles));

        rows.extend(self.results.iter().map(|result| {
            let mut row = Vec::new();
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
                                Cell::new("✔").style_spec("Fgc")
                            } else {
                                Cell::new("✖").style_spec("Frc")
                            }
                        }
                        None => Cell::new("✖").style_spec("Frc"),
                    })
                    .collect::<Vec<Cell>>(),
            );
            Row::new(row)
        }));

        let mut table = Table::init(rows);

        let format = format::FormatBuilder::new().padding(1, 1).build();
        table.set_format(format);
        table.fmt(f)
    }
}
