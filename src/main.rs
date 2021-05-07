use anyhow::Result;
use k8s_openapi::api::{apps::v1::Deployment, core::v1::Pod};
use kube::{
    api::{Api, ListParams},
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

macro_rules! test_req_all {
    ($kind:ident, $verb:ident, $call:ident, $params:expr) => {{
        let client = Client::try_default().await?;

        let c_api: Api<$kind> = Api::all(client);
        let req = Request {
            kind: "$kind".to_string(),
            verb: Verb::$verb,
        };
        let reqres = match c_api.$call($params).await {
            Ok(_l) => RequestResult::ok(req),
            Err(kube::Error::Api(ae)) => {
                if ae.code == 403 {
                    RequestResult::forbidden(req)
                } else {
                    RequestResult::error(req)
                }
            }
            Err(_e) => RequestResult::error(req),
        };
        reqres
    }};
}

macro_rules! test_req_ns {
    ($ns:expr, $kind:ident, $verb:ident, $call:ident, $params:expr) => {{
        let client = Client::try_default().await?;

        let c_api: Api<$kind> = Api::namespaced(client, $ns);
        let req = Request {
            kind: "$kind".to_string(),
            verb: Verb::$verb,
        };
        let reqres = match c_api.$call($params).await {
            Ok(_l) => RequestResult::ok(req),
            Err(kube::Error::Api(ae)) => {
                if ae.code == 403 {
                    RequestResult::forbidden(req)
                } else {
                    RequestResult::error(req)
                }
            }
            Err(_e) => RequestResult::error(req),
        };
        reqres
    }};
}

async fn test_get<T: Resource>(name: &str, namespace: &str) -> RequestResult
where
    <T as Resource>::DynamicType: Default,
    T: Clone,
    T: std::fmt::Debug,
    T: serde::de::DeserializeOwned,
{
    let req = Request {
        kind: std::any::type_name::<T>().to_string(),
        verb: Verb::Get,
    };
    if let Ok(client) = Client::try_default().await {
        let c_api: Api<T> = Api::namespaced(client, namespace);
        match c_api.get(name).await {
            Ok(_l) => RequestResult::ok(req),
            Err(kube::Error::Api(ae)) => {
                if ae.code == 403 {
                    RequestResult::forbidden(req)
                } else {
                    RequestResult::error(req)
                }
            }
            Err(_e) => RequestResult::error(req),
        }
    } else {
        RequestResult::error(req)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("{:?}", test_get::<Pod>("name", "default").await);
    println!("{:?}", test_get::<Deployment>("name", "default").await);
    Ok(())
}
