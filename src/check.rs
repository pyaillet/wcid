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
struct ResourceAttributes {
    group: String,
    resource: String,
    namespace: Option<String>,
    verb: String,
}

#[derive(Clone)]
struct ResourceAttributesBuilder {
    group: Option<String>,
    resource: Option<String>,
    namespace: Option<String>,
    verb: Option<String>,
}

impl ResourceAttributesBuilder {
    fn new() -> Self {
        Self {
            group: None,
            resource: None,
            namespace: None,
            verb: None,
        }
    }

    fn namespace(&mut self, namespace: &str) -> Self {
        self.namespace = Some(namespace.to_string());
        self.clone()
    }

    fn group(&mut self, group: &str) -> Self {
        self.group = Some(group.to_string());
        self.clone()
    }

    fn verb(&mut self, verb: &str) -> Self {
        self.verb = Some(verb.to_string());
        self.clone()
    }

    fn resource(&mut self, resource: &str) -> Self {
        self.resource = Some(resource.to_string());
        self.clone()
    }

    fn build(self) -> ResourceAttributes {
        let group = self
            .group
            .expect("Incomplete resource attribute declaration");
        let resource = self
            .resource
            .expect("Incomplete resource attribute declaration");
        let verb = self
            .verb
            .expect("Incomplete resource attribute declaration");
        ResourceAttributes {
            group,
            resource,
            verb,
            namespace: self.namespace,
        }
    }
}

#[derive(Debug)]
struct CheckResult {
    attributes: ResourceAttributes,
    allowed: bool,
    denied: bool,
}

async fn check(review: &ResourceAttributes) -> Result<CheckResult> {
    let client = Client::try_default().await?;

    let ssar: SelfSubjectAccessReview = serde_json::from_value(json!({
        "apiVersion": "authorization.k8s.io/v1",
        "kind": "SelfSubjectAccessReview",
        "metadata": {},
        "spec": {
            "resourceAttributes": {
              "group": review.group,
              "resource": review.resource,
              "namespace": review.namespace,
              "verb": review.verb,
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
        attributes: review.clone(),
        allowed: status.allowed,
        denied: status.denied.unwrap_or(false),
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
        Ok(v) => {
            for gvk in v {
                for verb in ALL_VERBS.iter() {
                    let ra = ResourceAttributesBuilder::new()
                        .group(DynamicObject::group(&gvk).as_ref())
                        .resource(DynamicObject::kind(&gvk).as_ref())
                        .verb(verb)
                        .build();
                    println!("{:?}", check(&ra).await);
                }
            }
        }
        Err(_) => {
            println!("Unable to list resources");
        }
    };
    Ok(())
}
