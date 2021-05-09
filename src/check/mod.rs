pub mod config;
pub mod constants;
pub mod formatter;
pub mod types;

use anyhow::Result;
use futures::future::try_join_all;
use kube::Client;
use kube::{api::GroupVersionKind, client::Discovery};
use serde_json::json;
use std::collections::HashMap;

use k8s_openapi::api::authorization::v1::SelfSubjectAccessReview;

use types::CheckResult;
use types::FullResult;
use types::ResourceCheckResult;

use crate::check::types::GroupVersionKindHelper;

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
    let mut items: HashMap<&'static str, CheckResult> = HashMap::new();
    for verb in constants::ALL_VERBS.iter() {
        items.insert(
            verb,
            check_resource_verb(&gvk, verb, namespace.clone()).await?,
        );
    }
    Ok(ResourceCheckResult {
        gvk: gvk.clone(),
        items,
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

pub struct Checker {
    pub config: config::Config,
}

impl Checker {
    pub fn new(config: config::Config) -> Self {
        Self { config }
    }

    pub async fn check_all(&self) -> Result<FullResult> {
        let resources = list_resources().await?;
        let future_results: Vec<_> = resources
            .iter()
            .map(|gvk| check_resource(gvk, self.config.namespace.clone()))
            .collect();
        let items = try_join_all(future_results).await?;

        Ok(FullResult {
            config: self.config.clone(),
            items,
        })
    }
}
