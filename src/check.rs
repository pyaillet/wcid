use std::{collections::HashMap, fmt::Display};

use anyhow::Result;
use futures::{stream, StreamExt, TryStreamExt};
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
struct ResourceCheckResult {
    gvk: GroupVersionKind,
    results: HashMap<&'static str, CheckResult>,
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
struct CheckResult {
    verb: &'static str,
    allowed: bool,
    denied: bool,
}

#[derive(Clone, Debug)]
pub struct FullResult {
    results: Vec<ResourceCheckResult>,
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
    gvk: GroupVersionKind,
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

async fn check_resource_global(gvk: GroupVersionKind) -> Result<ResourceCheckResult> {
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
    let results = stream::iter(resources)
        .then(check_resource_global)
        .try_collect::<Vec<ResourceCheckResult>>()
        .await?;
    Ok(FullResult { results })
}

impl Display for FullResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = Table::new();

        let format = format::FormatBuilder::new().padding(1, 1).build();
        table.set_format(format);

        let mut titles = vec![
            Cell::new("Group").style_spec("B"),
            Cell::new("Kind").style_spec("B"),
        ];
        titles.extend(
            ALL_VERBS
                .iter()
                .map(|v| Cell::new(v).style_spec("B"))
                .collect::<Vec<Cell>>(),
        );
        table.add_row(Row::new(titles));

        self.results.iter().for_each(|result| {
            let mut row = vec![Cell::new(&result.group()), Cell::new(&result.kind())];
            row.extend(
                ALL_VERBS
                    .iter()
                    .map(|v| match &result.results.get(v) {
                        Some(r) => {
                            if r.allowed {
                                Cell::new("✔")
                            } else {
                                Cell::new("✖")
                            }
                        }
                        None => Cell::new("✖"),
                    })
                    .collect::<Vec<Cell>>(),
            );
            table.add_row(Row::new(row));
        });

        table.fmt(f)
    }
}
