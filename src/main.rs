use anyhow::Result;
use clap::{AppSettings, Clap};
mod check;

#[derive(Clap)]
#[clap(
    version = "1.0",
    author = "Pierre-Yves Aillet <pyaillet@gmail.com>",
    about = "\n\nWCID What Can I Do is an RBAC enumerator for Kubernetes"
)]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    #[clap(short, long)]
    pub display_group: bool,
    #[clap(short, long)]
    pub namespace: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let config = check::Config {
        display_group: opts.display_group,
        namespace: opts.namespace,
    };
    let checker = check::Checker::new(config);
    let result = checker.check_all().await?;
    println!("{}", result);
    Ok(())
}
