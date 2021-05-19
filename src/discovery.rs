use anyhow::Result;
use futures::future::try_join_all;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::APIResource;
use kube::Client;

fn extract_version_group(group_version: String) -> (Option<String>, Option<String>) {
    let gv = group_version
        .split('/')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    if gv.len() > 1 {
        (gv.get(0).cloned(), gv.get(1).cloned())
    } else {
        (None, gv.get(0).cloned())
    }
}

fn extract_version(group_version: String) -> Option<String> {
    extract_version_group(group_version).1
}

fn extract_group(group_version: String) -> Option<String> {
    extract_version_group(group_version).0
}

pub async fn discover_resources(client: &Client) -> Result<Vec<APIResource>> {
    let api_groups = client.list_api_groups().await?;
    let core_api_versions = client.list_core_api_versions().await?;
    let api_resources = try_join_all(
        api_groups
            .groups
            .iter()
            .filter_map(|g| g.preferred_version.clone())
            .map(|g| {
                let version = g.group_version;
                async move { client.list_api_group_resources(&version).await }
            }),
    )
    .await?;
    let core_resources = try_join_all(core_api_versions.versions.first().map(|v| {
        let version = v;
        async move { client.list_core_api_resources(&version).await }
    }))
    .await?;
    let all_resources = api_resources
        .iter()
        .chain(core_resources.iter())
        .cloned()
        .flat_map(|list| {
            list.resources
                .iter()
                .map(|r| {
                    let mut resource = r.clone();
                    resource.version = extract_version(list.group_version.clone());
                    resource.group = extract_group(list.group_version.clone());
                    resource
                })
                .collect::<Vec<APIResource>>()
        })
        .collect::<Vec<APIResource>>();
    Ok(all_resources)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use kube::Client;

    #[ignore]
    #[tokio::test]
    async fn test_discovery() -> Result<()> {
        let client = Client::try_default()
            .await
            .expect("Unable to create the kube client");
        let resources = super::discover_resources(&client).await?;
        assert!(resources.len() > 1);
        Ok(())
    }
}
