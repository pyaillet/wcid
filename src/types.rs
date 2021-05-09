use kube::api::GroupVersionKind;
use kube::{api::DynamicObject, Resource};
use std::collections::HashMap;

use crate::config;

#[derive(Clone, Debug)]
pub struct ResourceCheckResult {
    pub gvk: GroupVersionKind,
    pub items: HashMap<&'static str, CheckResult>,
}

pub trait GroupVersionKindHelper {
    fn plural(&self) -> String;
    fn kind(&self) -> String;
    fn group(&self) -> String;
}

impl GroupVersionKindHelper for GroupVersionKind {
    fn plural(&self) -> String {
        DynamicObject::plural(&self).to_string()
    }

    fn kind(&self) -> String {
        DynamicObject::kind(&self).to_string()
    }

    fn group(&self) -> String {
        DynamicObject::group(&self).to_string()
    }
}

impl GroupVersionKindHelper for ResourceCheckResult {
    fn plural(&self) -> String {
        DynamicObject::plural(&self.gvk).to_string()
    }

    fn kind(&self) -> String {
        self.gvk.kind()
    }

    fn group(&self) -> String {
        self.gvk.group()
    }
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
