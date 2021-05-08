use anyhow::Result;
mod check;

#[tokio::main]
async fn main() -> Result<()> {
    let result = check::check_all().await?;
    println!("{}", result);
    Ok(())
}
