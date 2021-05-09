use anyhow::Result;

use futures::future::try_join_all;
use tokio::task;

use kube::Client;
use kube::{api::GroupVersionKind, client::Discovery};

use serde_json::json;

use std::collections::HashMap;

use k8s_openapi::api::authorization::v1::SelfSubjectAccessReview;

use crate::config;
use crate::constants;

use crate::types::CheckResult;
use crate::types::FullResult;
use crate::types::ResourceCheckResult;

use crate::types::GroupVersionKindHelper;

async fn check_resource_verb<'a>(
    client: &'a Client,
    gvk: &'a GroupVersionKind,
    verb: &'static str,
    namespace: Option<String>,
) -> Result<CheckResult> {
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
    client: Client,
    gvk: GroupVersionKind,
    namespace: Option<String>,
) -> Result<ResourceCheckResult> {
    let mut items: HashMap<&'static str, CheckResult> = HashMap::new();
    for verb in constants::ALL_VERBS.iter() {
        items.insert(
            verb,
            check_resource_verb(&client, &gvk, verb, namespace.clone()).await?,
        );
    }
    Ok(ResourceCheckResult {
        gvk: gvk.clone(),
        items,
    })
}

pub struct Checker {
    pub config: config::Config,
    pub client: Client,
}

impl Checker {
    pub async fn new(config: config::Config) -> Self {
        let client = Client::try_default()
            .await
            .expect("Unable to create the kube client");
        Self { config, client }
    }

    async fn list_resources(&self) -> Result<Vec<GroupVersionKind>> {
        let discovery = Discovery::new(&self.client).await?;
        let mut v = Vec::new();

        for group in discovery.groups() {
            let ver = group.preferred_version_or_guess();
            for gvk in group.resources_by_version(ver) {
                v.push(gvk);
            }
        }
        Ok(v)
    }

    pub async fn check_all(&self) -> Result<FullResult> {
        let resources = self.list_resources().await?;
        let future_results: Vec<_> = resources
            .iter()
            .map(|gvk| {
                let client = self.client.clone();
                let ns = self.config.namespace.clone();
                let gvk = gvk.clone();
                task::spawn(async { check_resource(client, gvk, ns).await })
            })
            .collect();
        let items = try_join_all(future_results).await?;

        Ok(FullResult {
            config: self.config.clone(),
            items: items
                .into_iter()
                .filter_map(|r| r.ok())
                .filter(|r| !self.config.hide_forbidden || r.items.iter().any(|(_, v)| v.allowed))
                .collect::<Vec<ResourceCheckResult>>(),
        })
    }
}
