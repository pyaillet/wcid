use anyhow::Result;
use clap::{AppSettings, Clap};
mod check;
mod config;
mod constants;
mod formatter;
mod types;

#[derive(Clap)]
#[clap(
    version = "1.0",
    author = "Pierre-Yves Aillet <pyaillet@gmail.com>",
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let config = config::Config {
        display_group: opts.display_group,
        namespace: opts.namespace,
        hide_forbidden: opts.hide_forbidden,
    };
    let checker = check::Checker::new(config.clone());
    let result = checker.check_all().await?;
    let formatter = formatter::Formatter::new(opts.format, config, result);
    println!("{}", formatter);

    Ok(())
}
