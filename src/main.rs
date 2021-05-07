use anyhow::Result;
use k8s_openapi::api::{apps::v1::Deployment, core::v1::Pod};
use kube::{
    api::{Api, ListParams, ObjectList},
    Client, Resource,
};
use serde;

#[derive(Clone, Copy, Debug)]
enum Verb {
    Get,
    List,
}

#[derive(Clone, Debug)]
struct Request {
    kind: String,
    verb: Verb,
}

#[derive(Clone, Copy, Debug)]
enum MyResult {
    Ok,
    Forbidden,
    Error,
}

enum ObjectResult<T: Clone> {
    Unit(Result<T, kube::Error>),
    List(Result<ObjectList<T>, kube::Error>),
}

#[derive(Clone, Debug)]
struct RequestResult {
    request: Request,
    result: MyResult,
}

impl RequestResult {
    fn new(request: Request, result: MyResult) -> Self {
        Self { request, result }
    }

    fn forbidden(request: Request) -> Self {
        RequestResult::new(request, MyResult::Forbidden)
    }

    fn ok(request: Request) -> Self {
        RequestResult::new(request, MyResult::Ok)
    }

    fn error(request: Request) -> Self {
        RequestResult::new(request, MyResult::Error)
    }
}

struct TesterNamespace<T> {
    object: Option<T>,
    namespace: String,
}

impl<T: Resource> TesterNamespace<T>
where
    <T as Resource>::DynamicType: Default,
    T: Clone,
    T: std::fmt::Debug,
    T: serde::de::DeserializeOwned,
{
    fn new(ns: &str) -> Self {
        Self {
            object: None,
            namespace: ns.to_string(),
        }
    }

    fn name(&self) -> String {
        match &self.object {
            Some(o) => o.name(),
            _ => "NOT FOUND".to_string(),
        }
    }

    fn handle_result(req: Request, result: ObjectResult<T>) -> (RequestResult, Option<T>) {
        match result {
            ObjectResult::Unit(Ok(_)) => (RequestResult::ok(req), None),
            ObjectResult::List(Ok(l)) => (RequestResult::ok(req), l.items.first().cloned()),
            ObjectResult::Unit(Err(kube::Error::Api(ae)))
            | ObjectResult::List(Err(kube::Error::Api(ae))) => {
                if ae.code == 403 {
                    (RequestResult::forbidden(req), None)
                } else {
                    (RequestResult::error(req), None)
                }
            }
            _ => (RequestResult::error(req), None),
        }
    }

    async fn test_all(&mut self) -> Vec<RequestResult> {
        let mut v = Vec::new();
        v.push(self.test_list().await);
        v.push(self.test_get().await);
        v
    }

    async fn test_get(&self) -> RequestResult {
        let req = Request {
            kind: std::any::type_name::<T>().to_string(),
            verb: Verb::Get,
        };
        if let Ok(client) = Client::try_default().await {
            let c_api: Api<T> = Api::namespaced(client, &self.namespace);
            Self::handle_result(req, ObjectResult::Unit(c_api.get(&self.name()).await)).0
        } else {
            RequestResult::error(req)
        }
    }

    async fn test_list(&mut self) -> RequestResult {
        let req = Request {
            kind: std::any::type_name::<T>().to_string(),
            verb: Verb::List,
        };
        if let Ok(client) = Client::try_default().await {
            let c_api: Api<T> = Api::namespaced(client, &self.namespace);
            let (result, object) = Self::handle_result(
                req,
                ObjectResult::List(c_api.list(&ListParams::default()).await),
            );
            self.object = object;
            result
        } else {
            RequestResult::error(req)
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut tp = TesterNamespace::<Pod>::new("default");
    println!("{:?}", tp.test_all().await);
    let mut td = TesterNamespace::<Deployment>::new("default");
    println!("{:?}", td.test_all().await);

    Ok(())
}
