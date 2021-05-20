use anyhow::Result;
use clap::{AppSettings, Clap};
mod check;
mod config;
mod constants;
mod discovery;
mod formatter;
mod types;

#[derive(Clap)]
#[clap(about = "WCID What Can I Do is an RBAC permission enumerator for Kubernetes")]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    #[clap(
        short,
        long,
        long_about = "Display the API group before the resource kind"
    )]
    pub display_group: bool,
    #[clap(
        short,
        long,
        long_about = "Check the permissions for a specific namespace"
    )]
    pub namespace: Option<String>,
    #[clap(
        short,
        long,
        default_value = "pretty",
        long_about = "Output format: json or pretty"
    )]
    pub format: String,
    #[clap(
        short = 's',
        long,
        long_about = "Only show resources for which at least an action is allowed"
    )]
    pub hide_forbidden: bool,
    #[clap(
        short = 'V',
        long,
        long_about = "Query for a defined set of verbs (separated by commas)\n[default: Get, Create, Delete, Update]\nAvailable: Get, List, Watch, Create, Delete, DeleteCollection, Update, Patch"
    )]
    pub verbs: Option<String>,
    #[clap(short = 'S', long, long_about = "Show subresources")]
    pub show_subresources: bool,
    #[clap(long = "as", long_about = "Enumerate rights as another user")]
    pub impersonate: Option<String>,
    #[clap(
        long = "sa",
        long_about = "Enumerate rights as the service account, the service account must be provided as <namespace>:<service_account_name>"
    )]
    pub service_account: Option<String>,
}

fn handle_verbs(verbs: Option<String>) -> Vec<&'static str> {
    match verbs {
        Some(s) => constants::ALL_VERBS
            .iter()
            .filter(|verb| {
                s.split(',')
                    .any(|supplied_verb| supplied_verb.to_lowercase() == verb.to_lowercase())
            })
            .cloned()
            .collect::<Vec<&'static str>>(),
        None => constants::DEFAULT_VERBS.to_vec(),
    }
}

fn qualify_sa(service_account: String) -> String {
    format!("system:serviceaccount:{}", service_account)
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let impersonate = match (opts.impersonate, opts.service_account) {
        (Some(_), Some(_)) => panic!("Can not use --as with --sa altogether"),
        (Some(impersonate), None) => Some(impersonate),
        (None, Some(service_account)) => Some(qualify_sa(service_account)),
        (None, None) => None,
    };
    let config = config::Config {
        display_group: opts.display_group,
        namespace: opts.namespace,
        hide_forbidden: opts.hide_forbidden,
        verbs: handle_verbs(opts.verbs),
        subresources: opts.show_subresources,
        impersonate,
    };
    let checker = check::Checker::new(config.clone()).await;
    let result = checker.check_all().await?;
    let formatter = formatter::Formatter::new(opts.format, config, result);
    println!("{}", formatter);

    Ok(())
}
