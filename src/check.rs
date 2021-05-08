use anyhow::Result;
use kube::Client;
use kube::{api::DynamicObject, Resource};
use kube::{api::GroupVersionKind, client::Discovery};
use lazy_static::lazy_static;
use serde_json::json;

use k8s_openapi::api::authorization::v1::SelfSubjectAccessReview;

lazy_static! {
    static ref ALL_VERBS: Vec<&'static str> =
        vec!["Get", "List", "Watch", "Create", "Delete", "Update", "Patch"];
}

#[derive(Clone, Debug)]
struct ResourceCheckResult {
    gvk: GroupVersionKind,
    results: Vec<CheckResult>,
}

#[derive(Clone, Debug)]
struct CheckResult {
    verb: String,
    allowed: bool,
    denied: bool,
}

async fn check_resource_verb(
    gvk: &GroupVersionKind,
    verb: &str,
    namespace: Option<String>,
) -> Result<CheckResult> {
    let client = Client::try_default().await?;

    let ssar: SelfSubjectAccessReview = serde_json::from_value(json!({
        "apiVersion": "authorization.k8s.io/v1",
        "kind": "SelfSubjectAccessReview",
        "metadata": {},
        "spec": {
            "resourceAttributes": {
              "group": DynamicObject::group(gvk),
              "resource": DynamicObject::kind(gvk),
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
        verb: verb.to_string(),
        allowed: status.allowed,
        denied: status.denied.unwrap_or(false),
    })
}

async fn check_resource(
    gvk: &GroupVersionKind,
    namespace: Option<String>,
) -> Result<ResourceCheckResult> {
    let mut results: Vec<CheckResult> = Vec::new();
    for verb in ALL_VERBS.iter() {
        results.push(check_resource_verb(gvk, verb, namespace.clone()).await?);
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

pub async fn check_all() -> Result<()> {
    match list_resources().await {
        Ok(resources) => {
            for gvk in resources {
                println!("{:?}", check_resource(&gvk, None).await?)
            }
        }
        Err(_) => {
            println!("Unable to list resources");
        }
    };
    Ok(())
}
