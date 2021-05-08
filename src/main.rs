use anyhow::Result;
mod check;

#[tokio::main]
async fn main() -> Result<()> {
    check::check_all().await
}
