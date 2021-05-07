use kube::Client;
use serde_json::json;

use k8s_openapi::api::authorization::v1::SelfSubjectAccessReview;

#[derive(Clone)]
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

    fn namespace(&mut self, namespace: String) -> Self {
        self.namespace = Some(namespace);
        self.clone()
    }

    fn group(&mut self, group: String) -> Self {
        self.group = Some(group);
        self.clone()
    }

    fn verb(&mut self, verb: String) -> Self {
        self.verb = Some(verb);
        self.clone()
    }

    fn resource(&mut self, resource: String) -> Self {
        self.resource = Some(resource);
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

async fn check(review: &ResourceAttributes) -> bool {
    let client = match Client::try_default().await {
        Ok(client) => client,
        Err(_e) => {
            panic!("Cannot create client");
        }
    };

    let ssar: SelfSubjectAccessReview = match serde_json::from_value(json!({
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
    })) {
        Ok(ssar) => ssar,
        Err(e) => {
            println!("Error {:?}", e);
            return false;
        }
    };

    let result =
        SelfSubjectAccessReview::create_self_subject_access_review(&ssar, Default::default());
    match result {
        Ok((reqp, _)) => {
            let res = client
                .request::<SelfSubjectAccessReview>(
                    http::Request::post(reqp.uri())
                        .body(reqp.body().clone())
                        .unwrap(),
                )
                .await
                .unwrap();
            res.status.unwrap().allowed
        }
        Err(_) => return false,
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ra = ResourceAttributesBuilder::new()
        .group("".into())
        .resource("pods".into())
        .verb("get".into())
        .build();
    println!("{:?}", check(&ra).await);
    Ok(())
}
