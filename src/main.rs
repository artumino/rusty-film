use clap::{Parser, Subcommand};
use enum_dispatch::enum_dispatch;
use import::ImportArgs;
use std::future::Future;
mod import;

#[derive(Debug, Parser)]
#[clap(name = "rusty-film", version)]
pub struct App {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
#[enum_dispatch(CommandRunner)]
pub enum Command {
    Import(ImportArgs),
}

#[enum_dispatch]
pub trait CommandRunner {
    fn run(&self) -> impl Future<Output = anyhow::Result<()>>;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = App::parse();
    app.command.run().await?;
    Ok(())
}
