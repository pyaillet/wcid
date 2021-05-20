use anyhow::Result;

use futures::future::try_join_all;
use tokio::task;

use kube::Client;

use serde_json::json;

use std::collections::HashMap;

use k8s_openapi::{
    api::authorization::v1::SelfSubjectAccessReview, apimachinery::pkg::apis::meta::v1::APIResource,
};

use crate::config;
use crate::constants;
use crate::discovery;

use crate::types::CheckResult;
use crate::types::FullResult;
use crate::types::ResourceCheckResult;

async fn check_resource_verb<'a>(
    client: &'a Client,
    resource: &'a APIResource,
    verb: &'static str,
    namespace: Option<String>,
    impersonate: Option<String>,
) -> Result<CheckResult> {
    let ssar: SelfSubjectAccessReview = serde_json::from_value(json!({
        "apiVersion": "authorization.k8s.io/v1",
        "kind": "SelfSubjectAccessReview",
        "metadata": {},
        "spec": {
            "resourceAttributes": {
              "group": resource.group,
              "resource": resource.name,
              "namespace": namespace,
              "verb": verb.to_ascii_lowercase(),
            },
        }
    }))?;

    let (reqp, _) =
        SelfSubjectAccessReview::create_self_subject_access_review(&ssar, Default::default())?;
    let mut request_builder = http::Request::post(reqp.uri());
    request_builder = if let Some(impersonate) = impersonate {
        request_builder.header("Impersonate-User", impersonate)
    } else {
        request_builder
    };
    let http_request = request_builder
        .body(reqp.body().clone())
        .expect("Unable to prepare HTTP request");
    let res = client
        .request::<SelfSubjectAccessReview>(http_request)
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
    resource: APIResource,
    namespace: Option<String>,
    impersonate: Option<String>,
) -> Result<ResourceCheckResult> {
    let mut items: HashMap<&'static str, CheckResult> = HashMap::new();
    for verb in constants::ALL_VERBS
        .iter()
        .filter(|v| resource.verbs.contains(&v.to_ascii_lowercase()))
    {
        items.insert(
            verb,
            check_resource_verb(
                &client,
                &resource,
                verb,
                namespace.clone(),
                impersonate.clone(),
            )
            .await?,
        );
    }
    Ok(ResourceCheckResult {
        resource: resource.clone(),
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

    pub async fn check_all(&self) -> Result<FullResult> {
        let resources: Vec<APIResource> = discovery::discover_resources(&self.client).await?;
        let future_results: Vec<_> = resources
            .iter()
            .filter(|resource| self.config.subresources || resource.name.find('/').is_none())
            .map(|resource| {
                let client = self.client.clone();
                let ns = self.config.namespace.clone();
                let impersonate = self.config.impersonate.clone();
                let resource = resource.clone();
                task::spawn(async { check_resource(client, resource, ns, impersonate).await })
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
