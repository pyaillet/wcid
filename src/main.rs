use anyhow::Result;
use clap::{AppSettings, Clap};
mod check;
mod config;
mod constants;
mod discovery;
mod formatter;
mod types;

#[derive(Clap)]
#[clap(
    version = "0.1",
    about = "WCID What Can I Do is an RBAC permission enumerator for Kubernetes"
)]
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
        long_about = "Only show resources for which an action is allowed"
    )]
    pub hide_forbidden: bool,
    #[clap(
        short = 'V',
        long,
        long_about = "Only query for a subset of verbs (separated by commas)"
    )]
    pub verbs: Option<String>,
}

fn handle_verbs(verbs: Option<String>) -> Vec<&'static str> {
    match verbs {
        Some(s) => constants::ALL_VERBS
            .iter()
            .filter(|verb| {
                s.split(',')
                    .find(|supplied_verb| supplied_verb.to_lowercase() == verb.to_lowercase())
                    .is_some()
            })
            .cloned()
            .collect::<Vec<&'static str>>(),
        None => constants::DEFAULT_VERBS.to_vec(),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let config = config::Config {
        display_group: opts.display_group,
        namespace: opts.namespace,
        hide_forbidden: opts.hide_forbidden,
        verbs: handle_verbs(opts.verbs),
    };
    let checker = check::Checker::new(config.clone()).await;
    let result = checker.check_all().await?;
    let formatter = formatter::Formatter::new(opts.format, config, result);
    println!("{}", formatter);

    Ok(())
}
