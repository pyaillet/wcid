use anyhow::Result;
use clap::{AppSettings, Clap};
mod check;

#[derive(Clap)]
#[clap(
    version = "1.0",
    author = "Pierre-Yves Aillet <pyaillet@gmail.com>",
    about = "WCID What Can I Do is an RBAC enumerator for Kubernetes"
)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    #[clap(short, long)]
    pub display_group: bool,
    #[clap(short, long)]
    pub namespace: Option<String>,
    #[clap(short, long, default_value = "pretty")]
    pub format: String,
    #[clap(short = 's', long)]
    pub hide_forbidden: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let config = check::config::Config {
        display_group: opts.display_group,
        namespace: opts.namespace,
        hide_forbidden: opts.hide_forbidden,
    };
    let checker = check::Checker::new(config.clone());
    let result = checker.check_all().await?;
    let formatter = check::formatter::Formatter::new(opts.format, config, result);
    println!("{}", formatter);

    Ok(())
}
