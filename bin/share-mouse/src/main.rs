use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tokio::signal;

mod args;
mod server;

#[tokio::main]
async fn main() -> Result<()> {
    let args = args::Args::parse();
    let server = Arc::new(server::Server::new(args));
    server.start().await;
    signal::ctrl_c().await?;
    Ok(())
}
