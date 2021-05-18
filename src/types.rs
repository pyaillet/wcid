use std::collections::HashMap;

use k8s_openapi::apimachinery::pkg::apis::meta::v1::APIResource;

use crate::config;

#[derive(Clone, Debug)]
pub struct ResourceCheckResult {
    pub resource: APIResource,
    pub items: HashMap<&'static str, CheckResult>,
}

#[derive(Clone, Debug)]
pub struct CheckResult {
    pub verb: &'static str,
    pub allowed: bool,
    pub denied: bool,
}

#[derive(Clone, Debug)]
pub struct FullResult {
    pub config: config::Config,
    pub items: Vec<ResourceCheckResult>,
}
