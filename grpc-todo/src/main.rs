use anyhow::Result;
use grpc_todo::run_server;

#[tokio::main]
async fn main() -> Result<()> {
    run_server().await?;

    Ok(())
}
